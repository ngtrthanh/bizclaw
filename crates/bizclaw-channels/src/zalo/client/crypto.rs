//! Zalo encryption — AES-256-ECB for message encryption.
//! Based on reverse-engineered Zalo Web encryption protocol.

use aes::Aes256;
use aes::cipher::{BlockDecrypt, BlockEncrypt, KeyInit, generic_array::GenericArray};

/// Encrypt data using AES-256-ECB (Zalo's message encryption).
pub fn encrypt_aes256(data: &[u8], key: &[u8; 32]) -> Vec<u8> {
    let cipher = Aes256::new(GenericArray::from_slice(key));

    // PKCS7 padding
    let block_size = 16;
    let padding_len = block_size - (data.len() % block_size);
    let mut padded = data.to_vec();
    padded.extend(std::iter::repeat_n(padding_len as u8, padding_len));

    // Encrypt each block
    let mut encrypted = Vec::with_capacity(padded.len());
    for chunk in padded.chunks(block_size) {
        let mut block = GenericArray::clone_from_slice(chunk);
        cipher.encrypt_block(&mut block);
        encrypted.extend_from_slice(&block);
    }

    encrypted
}

/// Decrypt data using AES-256-ECB.
pub fn decrypt_aes256(data: &[u8], key: &[u8; 32]) -> Vec<u8> {
    let cipher = Aes256::new(GenericArray::from_slice(key));

    let block_size = 16;
    let mut decrypted = Vec::with_capacity(data.len());

    for chunk in data.chunks(block_size) {
        if chunk.len() == block_size {
            let mut block = GenericArray::clone_from_slice(chunk);
            cipher.decrypt_block(&mut block);
            decrypted.extend_from_slice(&block);
        }
    }

    // Remove PKCS7 padding
    if let Some(&pad_len) = decrypted.last() {
        let pad_len = pad_len as usize;
        if pad_len <= block_size && pad_len <= decrypted.len() {
            let valid_padding = decrypted[decrypted.len() - pad_len..]
                .iter()
                .all(|&b| b == pad_len as u8);
            if valid_padding {
                decrypted.truncate(decrypted.len() - pad_len);
            }
        }
    }

    decrypted
}

/// Derive an encryption key from Zalo's zpw_enk.
pub fn derive_key(zpw_enk: &str) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(zpw_enk.as_bytes());
    let result = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&result);
    key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = derive_key("test_encryption_key_12345");
        let plaintext = b"Hello from BizClaw!";

        let encrypted = encrypt_aes256(plaintext, &key);
        let decrypted = decrypt_aes256(&encrypted, &key);

        assert_eq!(decrypted, plaintext);
    }
}
