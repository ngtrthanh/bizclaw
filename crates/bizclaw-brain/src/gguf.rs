//! GGUF format parser — reads model metadata and tensor info from GGUF v3 files.
//!
//! GGUF (GGML Universal File) format:
//! - Magic: "GGUF" (4 bytes)
//! - Version: u32 (must be 3)
//! - Tensor count: u64
//! - Metadata KV count: u64
//! - Metadata key-value pairs
//! - Tensor infos
//! - Padding to alignment boundary
//! - Tensor data

use bizclaw_core::error::{BizClawError, Result};
use std::collections::HashMap;
use std::io::{Read, Seek};

/// GGUF magic number.
const GGUF_MAGIC: u32 = 0x46554747; // "GGUF" in little-endian

/// Supported GGUF version.
const GGUF_VERSION: u32 = 3;

/// GGUF metadata value types.
#[derive(Debug, Clone)]
pub enum GgufValue {
    U8(u8),
    I8(i8),
    U16(u16),
    I16(i16),
    U32(u32),
    I32(i32),
    U64(u64),
    I64(i64),
    F32(f32),
    F64(f64),
    Bool(bool),
    String(String),
    Array(Vec<GgufValue>),
}

impl GgufValue {
    pub fn as_u32(&self) -> Option<u32> {
        match self {
            GgufValue::U32(v) => Some(*v),
            GgufValue::I32(v) => Some(*v as u32),
            GgufValue::U64(v) => Some(*v as u32),
            _ => None,
        }
    }

    pub fn as_u64(&self) -> Option<u64> {
        match self {
            GgufValue::U64(v) => Some(*v),
            GgufValue::U32(v) => Some(*v as u64),
            _ => None,
        }
    }

    pub fn as_f32(&self) -> Option<f32> {
        match self {
            GgufValue::F32(v) => Some(*v),
            GgufValue::F64(v) => Some(*v as f32),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            GgufValue::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            GgufValue::Bool(v) => Some(*v),
            _ => None,
        }
    }
}

/// GGML tensor types (quantization formats).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum GgmlType {
    F32 = 0,
    F16 = 1,
    Q4_0 = 2,
    Q4_1 = 3,
    Q5_0 = 6,
    Q5_1 = 7,
    Q8_0 = 8,
    Q8_1 = 9,
    Q2K = 10,
    Q3K = 11,
    Q4K = 12,
    Q5K = 13,
    Q6K = 14,
    Q8K = 15,
    IQ2XXS = 16,
    IQ2XS = 17,
    IQ3XXS = 18,
    IQ1S = 19,
    IQ4NL = 20,
    IQ3S = 21,
    IQ2S = 22,
    IQ4XS = 23,
}

impl GgmlType {
    /// Parse from u32.
    pub fn from_u32(v: u32) -> Result<Self> {
        match v {
            0 => Ok(GgmlType::F32),
            1 => Ok(GgmlType::F16),
            2 => Ok(GgmlType::Q4_0),
            3 => Ok(GgmlType::Q4_1),
            6 => Ok(GgmlType::Q5_0),
            7 => Ok(GgmlType::Q5_1),
            8 => Ok(GgmlType::Q8_0),
            9 => Ok(GgmlType::Q8_1),
            10 => Ok(GgmlType::Q2K),
            11 => Ok(GgmlType::Q3K),
            12 => Ok(GgmlType::Q4K),
            13 => Ok(GgmlType::Q5K),
            14 => Ok(GgmlType::Q6K),
            15 => Ok(GgmlType::Q8K),
            _ => Err(BizClawError::GgufParse(format!("Unknown GGML type: {v}"))),
        }
    }

    /// Block size in elements for quantized types.
    pub fn block_size(&self) -> usize {
        match self {
            GgmlType::F32 | GgmlType::F16 => 1,
            GgmlType::Q4_0 | GgmlType::Q4_1 => 32,
            GgmlType::Q5_0 | GgmlType::Q5_1 => 32,
            GgmlType::Q8_0 | GgmlType::Q8_1 => 32,
            GgmlType::Q2K
            | GgmlType::Q3K
            | GgmlType::Q4K
            | GgmlType::Q5K
            | GgmlType::Q6K
            | GgmlType::Q8K => 256,
            _ => 32,
        }
    }

    /// Bytes per block for quantized types.
    pub fn type_size(&self) -> usize {
        match self {
            GgmlType::F32 => 4,
            GgmlType::F16 => 2,
            GgmlType::Q4_0 => 18, // 2 + 32/2
            GgmlType::Q4_1 => 20, // 2 + 2 + 32/2
            GgmlType::Q5_0 => 22, // 2 + 4 + 32/2
            GgmlType::Q5_1 => 24, // 2 + 2 + 4 + 32/2
            GgmlType::Q8_0 => 34, // 2 + 32
            GgmlType::Q8_1 => 40, // 4 + 4 + 32
            GgmlType::Q2K => 84,
            GgmlType::Q3K => 110,
            GgmlType::Q4K => 144,
            GgmlType::Q5K => 176,
            GgmlType::Q6K => 210,
            GgmlType::Q8K => 292,
            _ => 0,
        }
    }
}

/// Information about a tensor stored in the GGUF file.
#[derive(Debug, Clone)]
pub struct TensorInfo {
    pub name: String,
    pub n_dims: u32,
    pub dims: Vec<u64>,
    pub ggml_type: GgmlType,
    pub offset: u64,
}

impl TensorInfo {
    /// Total number of elements in this tensor.
    pub fn n_elements(&self) -> u64 {
        self.dims.iter().product::<u64>()
    }

    /// Size of this tensor in bytes.
    pub fn size_bytes(&self) -> u64 {
        let n = self.n_elements() as usize;
        let bs = self.ggml_type.block_size();
        let ts = self.ggml_type.type_size();
        (n.div_ceil(bs) * ts) as u64
    }
}

/// Parsed GGUF file header — metadata + tensor index.
#[derive(Debug)]
pub struct GgufFile {
    pub version: u32,
    pub metadata: HashMap<String, GgufValue>,
    pub tensors: Vec<TensorInfo>,
    pub data_offset: u64,
    pub alignment: u64,
}

impl GgufFile {
    /// Parse a GGUF file from a reader.
    pub fn parse<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        // Read magic
        let magic = read_u32(reader)?;
        if magic != GGUF_MAGIC {
            return Err(BizClawError::GgufParse(format!(
                "Invalid GGUF magic: 0x{:08X} (expected 0x{:08X})",
                magic, GGUF_MAGIC
            )));
        }

        // Read version
        let version = read_u32(reader)?;
        if version != GGUF_VERSION {
            return Err(BizClawError::GgufParse(format!(
                "Unsupported GGUF version: {version} (expected {GGUF_VERSION})"
            )));
        }

        // Read counts
        let tensor_count = read_u64(reader)?;
        let metadata_kv_count = read_u64(reader)?;

        // Read metadata
        let mut metadata = HashMap::new();
        for _ in 0..metadata_kv_count {
            let key = read_string(reader)?;
            let value = read_value(reader)?;
            metadata.insert(key, value);
        }

        // Get alignment (default 32)
        let alignment = metadata
            .get("general.alignment")
            .and_then(|v| v.as_u64())
            .unwrap_or(32);

        // Read tensor infos
        let mut tensors = Vec::with_capacity(tensor_count as usize);
        for _ in 0..tensor_count {
            let name = read_string(reader)?;
            let n_dims = read_u32(reader)?;
            let mut dims = Vec::with_capacity(n_dims as usize);
            for _ in 0..n_dims {
                dims.push(read_u64(reader)?);
            }
            let type_id = read_u32(reader)?;
            let ggml_type = GgmlType::from_u32(type_id)?;
            let offset = read_u64(reader)?;

            tensors.push(TensorInfo {
                name,
                n_dims,
                dims,
                ggml_type,
                offset,
            });
        }

        // Calculate data offset (aligned to alignment)
        let current_pos = reader
            .stream_position()
            .map_err(|e| BizClawError::GgufParse(e.to_string()))?;
        let data_offset = current_pos.div_ceil(alignment) * alignment;

        Ok(GgufFile {
            version,
            metadata,
            tensors,
            data_offset,
            alignment,
        })
    }

    /// Get model architecture name.
    pub fn architecture(&self) -> Option<&str> {
        self.metadata.get("general.architecture")?.as_str()
    }

    /// Get model name.
    pub fn model_name(&self) -> Option<&str> {
        self.metadata.get("general.name")?.as_str()
    }

    /// Get a u32 metadata value with a key prefix.
    pub fn get_u32(&self, key: &str) -> Option<u32> {
        self.metadata.get(key)?.as_u32()
    }

    /// Get a f32 metadata value.
    pub fn get_f32(&self, key: &str) -> Option<f32> {
        self.metadata.get(key)?.as_f32()
    }
}

// ===== Low-level reading helpers =====

fn read_u8<R: Read>(r: &mut R) -> Result<u8> {
    let mut buf = [0u8; 1];
    r.read_exact(&mut buf)
        .map_err(|e| BizClawError::GgufParse(e.to_string()))?;
    Ok(buf[0])
}

fn read_u32<R: Read>(r: &mut R) -> Result<u32> {
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf)
        .map_err(|e| BizClawError::GgufParse(e.to_string()))?;
    Ok(u32::from_le_bytes(buf))
}

fn read_i32<R: Read>(r: &mut R) -> Result<i32> {
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf)
        .map_err(|e| BizClawError::GgufParse(e.to_string()))?;
    Ok(i32::from_le_bytes(buf))
}

fn read_u64<R: Read>(r: &mut R) -> Result<u64> {
    let mut buf = [0u8; 8];
    r.read_exact(&mut buf)
        .map_err(|e| BizClawError::GgufParse(e.to_string()))?;
    Ok(u64::from_le_bytes(buf))
}

fn read_i64<R: Read>(r: &mut R) -> Result<i64> {
    let mut buf = [0u8; 8];
    r.read_exact(&mut buf)
        .map_err(|e| BizClawError::GgufParse(e.to_string()))?;
    Ok(i64::from_le_bytes(buf))
}

fn read_f32<R: Read>(r: &mut R) -> Result<f32> {
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf)
        .map_err(|e| BizClawError::GgufParse(e.to_string()))?;
    Ok(f32::from_le_bytes(buf))
}

fn read_f64<R: Read>(r: &mut R) -> Result<f64> {
    let mut buf = [0u8; 8];
    r.read_exact(&mut buf)
        .map_err(|e| BizClawError::GgufParse(e.to_string()))?;
    Ok(f64::from_le_bytes(buf))
}

fn read_string<R: Read>(r: &mut R) -> Result<String> {
    let len = read_u64(r)? as usize;
    if len > 1024 * 1024 {
        // 1MB max string
        return Err(BizClawError::GgufParse(format!("String too long: {len}")));
    }
    let mut buf = vec![0u8; len];
    r.read_exact(&mut buf)
        .map_err(|e| BizClawError::GgufParse(e.to_string()))?;
    String::from_utf8(buf).map_err(|e| BizClawError::GgufParse(e.to_string()))
}

fn read_value<R: Read>(r: &mut R) -> Result<GgufValue> {
    let type_id = read_u32(r)?;
    match type_id {
        0 => Ok(GgufValue::U8(read_u8(r)?)),
        1 => Ok(GgufValue::I8(read_u8(r)? as i8)),
        2 => {
            let mut buf = [0u8; 2];
            r.read_exact(&mut buf)
                .map_err(|e| BizClawError::GgufParse(e.to_string()))?;
            Ok(GgufValue::U16(u16::from_le_bytes(buf)))
        }
        3 => {
            let mut buf = [0u8; 2];
            r.read_exact(&mut buf)
                .map_err(|e| BizClawError::GgufParse(e.to_string()))?;
            Ok(GgufValue::I16(i16::from_le_bytes(buf)))
        }
        4 => Ok(GgufValue::U32(read_u32(r)?)),
        5 => Ok(GgufValue::I32(read_i32(r)?)),
        6 => Ok(GgufValue::F32(read_f32(r)?)),
        7 => Ok(GgufValue::Bool(read_u8(r)? != 0)),
        8 => Ok(GgufValue::String(read_string(r)?)),
        9 => {
            // Array: element_type (u32) + count (u64) + elements
            let elem_type = read_u32(r)?;
            let count = read_u64(r)? as usize;
            if count > 10_000_000 {
                return Err(BizClawError::GgufParse(format!("Array too large: {count}")));
            }
            let mut arr = Vec::with_capacity(count);
            for _ in 0..count {
                let val = match elem_type {
                    0 => GgufValue::U8(read_u8(r)?),
                    4 => GgufValue::U32(read_u32(r)?),
                    5 => GgufValue::I32(read_i32(r)?),
                    6 => GgufValue::F32(read_f32(r)?),
                    8 => GgufValue::String(read_string(r)?),
                    10 => GgufValue::U64(read_u64(r)?),
                    11 => GgufValue::I64(read_i64(r)?),
                    12 => GgufValue::F64(read_f64(r)?),
                    _ => {
                        return Err(BizClawError::GgufParse(format!(
                            "Unknown array element type: {elem_type}"
                        )));
                    }
                };
                arr.push(val);
            }
            Ok(GgufValue::Array(arr))
        }
        10 => Ok(GgufValue::U64(read_u64(r)?)),
        11 => Ok(GgufValue::I64(read_i64(r)?)),
        12 => Ok(GgufValue::F64(read_f64(r)?)),
        _ => Err(BizClawError::GgufParse(format!(
            "Unknown value type: {type_id}"
        ))),
    }
}
