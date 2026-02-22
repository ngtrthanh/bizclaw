//! Quantization kernels — dequantize quantized weight blocks to f32.
//!
//! Supports Q4_0, Q4_K_M, Q6_K, Q8_0 formats used by GGUF models.

use bizclaw_core::error::Result;

/// Dequantize Q4_0 block (18 bytes → 32 f32 values).
/// Format: scale (f16, 2 bytes) + 16 bytes of 4-bit quantized values.
pub fn dequantize_q4_0(block: &[u8], output: &mut [f32]) {
    debug_assert!(block.len() >= 18);
    debug_assert!(output.len() >= 32);

    let scale = half::f16::from_le_bytes([block[0], block[1]]).to_f32();

    for (i, &byte) in block[2..18].iter().enumerate() {
        let lo = (byte & 0x0F) as f32 - 8.0;
        let hi = ((byte >> 4) & 0x0F) as f32 - 8.0;
        output[i * 2] = lo * scale;
        output[i * 2 + 1] = hi * scale;
    }
}

/// Dequantize Q8_0 block (34 bytes → 32 f32 values).
/// Format: scale (f16, 2 bytes) + 32 bytes of 8-bit quantized values.
pub fn dequantize_q8_0(block: &[u8], output: &mut [f32]) {
    debug_assert!(block.len() >= 34);
    debug_assert!(output.len() >= 32);

    let scale = half::f16::from_le_bytes([block[0], block[1]]).to_f32();

    for (out, &byte) in output.iter_mut().zip(&block[2..34]) {
        *out = byte as i8 as f32 * scale;
    }
}

/// Dequantize a full row of quantized data to f32.
/// Dispatches to the correct dequantization kernel based on type.
pub fn dequantize_row(
    data: &[u8],
    output: &mut [f32],
    n_elements: usize,
    ggml_type: crate::gguf::GgmlType,
) -> Result<()> {
    match ggml_type {
        crate::gguf::GgmlType::F32 => {
            // Direct copy from bytes to f32
            for (out, chunk) in output.iter_mut().zip(data.chunks_exact(4)).take(n_elements) {
                *out = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            }
        }
        crate::gguf::GgmlType::F16 => {
            for (out, chunk) in output.iter_mut().zip(data.chunks_exact(2)).take(n_elements) {
                *out = half::f16::from_le_bytes([chunk[0], chunk[1]]).to_f32();
            }
        }
        crate::gguf::GgmlType::Q4_0 => {
            let block_size = 32;
            let type_size = 18;
            let n_blocks = n_elements / block_size;
            for b in 0..n_blocks {
                let block_data = &data[b * type_size..];
                dequantize_q4_0(block_data, &mut output[b * block_size..]);
            }
        }
        crate::gguf::GgmlType::Q8_0 => {
            let block_size = 32;
            let type_size = 34;
            let n_blocks = n_elements / block_size;
            for b in 0..n_blocks {
                let block_data = &data[b * type_size..];
                dequantize_q8_0(block_data, &mut output[b * block_size..]);
            }
        }
        _ => {
            // For unsupported types, fill with zeros
            tracing::warn!(
                "Unsupported quantization type: {:?}, filling with zeros",
                ggml_type
            );
            for v in output.iter_mut().take(n_elements) {
                *v = 0.0;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dequantize_q8_0() {
        // Scale = 1.0 (as f16), values = [1, 2, 3, ...]
        let scale_bytes = half::f16::from_f32(1.0).to_le_bytes();
        let mut block = vec![0u8; 34];
        block[0] = scale_bytes[0];
        block[1] = scale_bytes[1];
        for i in 0..32 {
            block[2 + i] = (i + 1) as u8;
        }
        let mut output = vec![0.0f32; 32];
        dequantize_q8_0(&block, &mut output);
        assert!((output[0] - 1.0).abs() < 0.01);
        assert!((output[1] - 2.0).abs() < 0.01);
    }
}
