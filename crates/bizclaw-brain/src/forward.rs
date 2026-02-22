//! LLaMA transformer forward pass.
//!
//! Implements the complete LLaMA-2/3 transformer architecture:
//! Embedding → N × (RMSNorm → Attention → RMSNorm → FFN) → RMSNorm → LM Head
//!
//! Reads weights from mmap, dequantizes on-the-fly, computes the forward
//! pass, and produces logits for the next token.

use crate::{kv_cache::KvCache, mmap::MmapModel, model::ModelParams, quant, rope, tensor};
use bizclaw_core::error::{BizClawError, Result};

/// Transformer weights — indices into the GGUF tensor list.
pub struct TransformerWeights {
    // Token embedding table
    pub token_embd: Option<usize>,
    // Output norm
    pub output_norm: Option<usize>,
    // LM head (output projection)
    pub output: Option<usize>,
    // Per-layer weight indices
    pub layers: Vec<LayerWeights>,
}

/// Weights for a single transformer layer.
pub struct LayerWeights {
    pub attn_norm: Option<usize>,
    pub attn_q: Option<usize>,
    pub attn_k: Option<usize>,
    pub attn_v: Option<usize>,
    pub attn_output: Option<usize>,
    pub ffn_norm: Option<usize>,
    pub ffn_gate: Option<usize>, // gate_proj (SiLU activation)
    pub ffn_up: Option<usize>,   // up_proj
    pub ffn_down: Option<usize>, // down_proj
}

impl TransformerWeights {
    /// Build weight index from GGUF tensor names.
    pub fn from_gguf(model: &MmapModel, params: &ModelParams) -> Self {
        let find = |name: &str| -> Option<usize> {
            model.gguf.tensors.iter().position(|t| t.name == name)
        };

        let mut layers = Vec::new();
        for l in 0..params.n_layers {
            layers.push(LayerWeights {
                attn_norm: find(&format!("blk.{l}.attn_norm.weight")),
                attn_q: find(&format!("blk.{l}.attn_q.weight")),
                attn_k: find(&format!("blk.{l}.attn_k.weight")),
                attn_v: find(&format!("blk.{l}.attn_v.weight")),
                attn_output: find(&format!("blk.{l}.attn_output.weight")),
                ffn_norm: find(&format!("blk.{l}.ffn_norm.weight")),
                ffn_gate: find(&format!("blk.{l}.ffn_gate.weight")),
                ffn_up: find(&format!("blk.{l}.ffn_up.weight")),
                ffn_down: find(&format!("blk.{l}.ffn_down.weight")),
            });
        }

        Self {
            token_embd: find("token_embd.weight"),
            output_norm: find("output_norm.weight"),
            output: find("output.weight"),
            layers,
        }
    }
}

/// Run a single-token forward pass through the LLaMA transformer.
///
/// Returns logits of shape [vocab_size].
pub fn forward(
    model: &MmapModel,
    weights: &TransformerWeights,
    params: &ModelParams,
    kv_cache: &mut KvCache,
    token: u32,
    pos: usize,
    logits: &mut [f32],
) -> Result<()> {
    let dim = params.dim as usize;
    let hidden_dim = params.hidden_dim as usize;
    let n_heads = params.n_heads as usize;
    let n_kv_heads = params.n_kv_heads as usize;
    let head_dim = params.head_dim as usize;
    let kv_dim = n_kv_heads * head_dim;
    let vocab_size = params.vocab_size as usize;

    // ---- Step 1: Token embedding lookup ----
    let mut x = vec![0.0f32; dim];
    if let Some(embd_idx) = weights.token_embd {
        let embd_tensor = &model.gguf.tensors[embd_idx];
        let embd_data = model.tensor_data(embd_idx)?;
        let offset = token as usize * dim;
        let row_bytes =
            dim * embd_tensor.ggml_type.type_size() / embd_tensor.ggml_type.block_size();

        // If embedding is F32, direct copy. Otherwise dequantize.
        if embd_tensor.ggml_type == crate::gguf::GgmlType::F32 {
            let byte_offset = offset * 4;
            for (i, xi) in x.iter_mut().enumerate().take(dim) {
                let o = byte_offset + i * 4;
                if o + 4 <= embd_data.len() {
                    *xi = f32::from_le_bytes([
                        embd_data[o],
                        embd_data[o + 1],
                        embd_data[o + 2],
                        embd_data[o + 3],
                    ]);
                }
            }
        } else {
            let row_offset = token as usize * row_bytes;
            if row_offset + row_bytes <= embd_data.len() {
                quant::dequantize_row(
                    &embd_data[row_offset..],
                    &mut x,
                    dim,
                    embd_tensor.ggml_type,
                )?;
            }
        }
    } else {
        return Err(BizClawError::Brain("Missing token_embd.weight".into()));
    }

    // Scratch buffers
    let mut xb = vec![0.0f32; dim]; // after RMSNorm
    let mut xb2 = vec![0.0f32; dim]; // second residual
    let mut q = vec![0.0f32; dim]; // query
    let mut k = vec![0.0f32; kv_dim]; // key
    let mut v = vec![0.0f32; kv_dim]; // value
    let mut att_out = vec![0.0f32; dim]; // attention output
    let mut hb = vec![0.0f32; hidden_dim]; // FFN hidden
    let mut hb2 = vec![0.0f32; hidden_dim]; // FFN gate

    // ---- Step 2: Transformer layers ----
    for l in 0..params.n_layers as usize {
        let layer = &weights.layers[l];

        // 2a. Attention RMSNorm
        if let Some(norm_idx) = layer.attn_norm {
            let norm_w = dequant_weight(model, norm_idx, dim)?;
            tensor::rmsnorm(&mut xb, &x, &norm_w, params.rms_norm_eps);
        } else {
            xb.copy_from_slice(&x);
        }

        // 2b. Q/K/V projections
        matmul_weight(model, layer.attn_q, &xb, &mut q, dim, dim)?;
        matmul_weight(model, layer.attn_k, &xb, &mut k, kv_dim, dim)?;
        matmul_weight(model, layer.attn_v, &xb, &mut v, kv_dim, dim)?;

        // 2c. RoPE on Q and K
        rope::apply_rope_multi_head(&mut q, pos, n_heads, head_dim, params.rope_theta);
        rope::apply_rope_multi_head(&mut k, pos, n_kv_heads, head_dim, params.rope_theta);

        // 2d. Store K/V in cache
        kv_cache.key_at_mut(l, pos).copy_from_slice(&k);
        kv_cache.value_at_mut(l, pos).copy_from_slice(&v);

        let seq_len = pos + 1;

        // 2e. Multi-head attention (with GQA)
        let kv_keys = kv_cache.keys(l, seq_len);
        let kv_values = kv_cache.values(l, seq_len);

        for h in 0..n_heads {
            let kv_h = h * n_kv_heads / n_heads; // GQA: map query head to kv head
            let q_slice = &q[h * head_dim..(h + 1) * head_dim];

            // Build key/value slices for this kv head
            let mut head_keys = vec![0.0f32; seq_len * head_dim];
            let mut head_values = vec![0.0f32; seq_len * head_dim];
            for t in 0..seq_len {
                let k_start = t * kv_dim + kv_h * head_dim;
                let v_start = t * kv_dim + kv_h * head_dim;
                head_keys[t * head_dim..(t + 1) * head_dim]
                    .copy_from_slice(&kv_keys[k_start..k_start + head_dim]);
                head_values[t * head_dim..(t + 1) * head_dim]
                    .copy_from_slice(&kv_values[v_start..v_start + head_dim]);
            }

            // Attention for this head
            let mut head_out = vec![0.0f32; head_dim];
            crate::attention::attention(
                &mut head_out,
                q_slice,
                &head_keys,
                &head_values,
                seq_len,
                head_dim,
            );

            // Copy to full output
            att_out[h * head_dim..(h + 1) * head_dim].copy_from_slice(&head_out);
        }

        // 2f. Output projection
        matmul_weight(model, layer.attn_output, &att_out, &mut xb2, dim, dim)?;

        // 2g. Residual connection
        tensor::elementwise_add(&mut x, &xb2);

        // 2h. FFN RMSNorm
        if let Some(norm_idx) = layer.ffn_norm {
            let norm_w = dequant_weight(model, norm_idx, dim)?;
            tensor::rmsnorm(&mut xb, &x, &norm_w, params.rms_norm_eps);
        } else {
            xb.copy_from_slice(&x);
        }

        // 2i. FFN: SwiGLU
        // gate = silu(xb @ gate_proj)
        // up   = xb @ up_proj
        // down = (gate * up) @ down_proj
        matmul_weight(model, layer.ffn_gate, &xb, &mut hb, hidden_dim, dim)?;
        matmul_weight(model, layer.ffn_up, &xb, &mut hb2, hidden_dim, dim)?;

        tensor::silu(&mut hb);
        tensor::elementwise_mul(&mut hb, &hb2);

        matmul_weight(model, layer.ffn_down, &hb, &mut xb2, dim, hidden_dim)?;

        // 2j. Residual connection
        tensor::elementwise_add(&mut x, &xb2);
    }

    // ---- Step 3: Final RMSNorm ----
    if let Some(norm_idx) = weights.output_norm {
        let norm_w = dequant_weight(model, norm_idx, dim)?;
        tensor::rmsnorm(&mut xb, &x, &norm_w, params.rms_norm_eps);
    } else {
        xb.copy_from_slice(&x);
    }

    // ---- Step 4: LM Head → logits ----
    matmul_weight(model, weights.output, &xb, logits, vocab_size, dim)?;

    Ok(())
}

/// Dequantize a full weight tensor to f32.
fn dequant_weight(model: &MmapModel, tensor_idx: usize, n_elements: usize) -> Result<Vec<f32>> {
    let data = model.tensor_data(tensor_idx)?;
    let tensor = &model.gguf.tensors[tensor_idx];
    let mut output = vec![0.0f32; n_elements];
    quant::dequantize_row(data, &mut output, n_elements, tensor.ggml_type)?;
    Ok(output)
}

/// Matrix-vector multiply using a weight tensor from mmap.
/// output[rows] = weight[rows x cols] @ input[cols]
fn matmul_weight(
    model: &MmapModel,
    tensor_idx: Option<usize>,
    input: &[f32],
    output: &mut [f32],
    rows: usize,
    cols: usize,
) -> Result<()> {
    let idx = tensor_idx.ok_or_else(|| BizClawError::Brain("Missing weight tensor".into()))?;
    let data = model.tensor_data(idx)?;
    let tensor = &model.gguf.tensors[idx];

    // Dequantize entire weight matrix
    let n_elements = rows * cols;
    let mut weight = vec![0.0f32; n_elements];
    quant::dequantize_row(data, &mut weight, n_elements, tensor.ggml_type)?;

    // MatMul
    tensor::matmul(output, &weight, input, rows, cols);
    Ok(())
}
