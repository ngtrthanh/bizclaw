//! Memory-mapped model loading.
//!
//! Uses mmap to load model weights directly from disk without copying
//! them into process memory. This is critical for running on devices
//! with limited RAM (e.g., Raspberry Pi with 512MB).

use memmap2::Mmap;
use std::fs::File;
use std::path::Path;
use bizclaw_core::error::{BizClawError, Result};

use crate::gguf::GgufFile;

/// A memory-mapped GGUF model file.
pub struct MmapModel {
    /// The parsed GGUF header with metadata and tensor index.
    pub gguf: GgufFile,
    /// Memory-mapped file data.
    mmap: Mmap,
}

impl MmapModel {
    /// Load a GGUF model file using mmap.
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Err(BizClawError::ModelLoad(format!(
                "Model file not found: {}",
                path.display()
            )));
        }

        let file = File::open(path)
            .map_err(|e| BizClawError::ModelLoad(format!("Failed to open model: {e}")))?;

        // Parse GGUF header
        let mut reader = std::io::BufReader::new(&file);
        let gguf = GgufFile::parse(&mut reader)?;

        tracing::info!(
            "GGUF model: arch={}, tensors={}, data_offset={}",
            gguf.architecture().unwrap_or("unknown"),
            gguf.tensors.len(),
            gguf.data_offset
        );

        // Memory-map the entire file
        let mmap = unsafe {
            Mmap::map(&file)
                .map_err(|e| BizClawError::ModelLoad(format!("mmap failed: {e}")))?
        };

        tracing::info!(
            "Model loaded via mmap: {} ({:.1} MB)",
            path.display(),
            mmap.len() as f64 / (1024.0 * 1024.0)
        );

        Ok(Self { gguf, mmap })
    }

    /// Get a raw byte slice for a tensor's data.
    pub fn tensor_data(&self, tensor_index: usize) -> Result<&[u8]> {
        let tensor = self.gguf.tensors.get(tensor_index)
            .ok_or_else(|| BizClawError::ModelLoad(format!(
                "Tensor index {} out of range (total: {})",
                tensor_index, self.gguf.tensors.len()
            )))?;

        let start = (self.gguf.data_offset + tensor.offset) as usize;
        let size = tensor.size_bytes() as usize;
        let end = start + size;

        if end > self.mmap.len() {
            return Err(BizClawError::ModelLoad(format!(
                "Tensor '{}' data out of bounds: offset={}, size={}, file_size={}",
                tensor.name, start, size, self.mmap.len()
            )));
        }

        Ok(&self.mmap[start..end])
    }

    /// Get tensor data by name.
    pub fn tensor_data_by_name(&self, name: &str) -> Result<&[u8]> {
        let index = self.gguf.tensors.iter()
            .position(|t| t.name == name)
            .ok_or_else(|| BizClawError::ModelLoad(format!("Tensor not found: {name}")))?;
        self.tensor_data(index)
    }

    /// Get model architecture (e.g., "llama").
    pub fn architecture(&self) -> &str {
        self.gguf.architecture().unwrap_or("unknown")
    }

    /// Get total size of the model file in bytes.
    pub fn file_size(&self) -> usize {
        self.mmap.len()
    }

    /// Get number of tensors.
    pub fn tensor_count(&self) -> usize {
        self.gguf.tensors.len()
    }
}
