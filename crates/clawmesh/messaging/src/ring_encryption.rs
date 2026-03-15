//! Ring-based Encryption Implementation
//!
//! Production-ready end-to-end encryption using the ring cryptography library.

use ring::aead::{Aad, BoundKey, Nonce, NonceSequence, OpeningKey, SealingKey, UnboundKey, AES_256_GCM, CHACHA20_POLY1305};
use ring::error::Unspecified;
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use tracing::{debug, warn, instrument};
use chrono::{DateTime, Utc, Duration};

use crate::encryption::{EncryptionAlgorithm, EncryptedMessage, EncryptionKey};

/// Nonce sequence for AEAD operations
struct CounterNonceSequence {
    counter: u128,
}

impl CounterNonceSequence {
    fn new() -> Self {
        Self { counter: 0 }
    }
}

impl NonceSequence for CounterNonceSequence {
    fn advance(&mut self) -> Result<Nonce, Unspecified> {
        let mut nonce_bytes = [0u8; 12];
        nonce_bytes[..16.min(12)].copy_from_slice(&self.counter.to_le_bytes()[..16.min(12)]);
        self.counter = self.counter.wrapping_add(1);
        Nonce::try_assume_unique_for_key(&nonce_bytes)
    }
}

/// Ring-based encryption service
pub struct RingEncryptionService {
    algorithm: EncryptionAlgorithm,
    rng: SystemRandom,
}

impl RingEncryptionService {
    /// Create new encryption service
    pub fn new(algorithm: EncryptionAlgorithm) -> Self {
        Self {
            algorithm,
            rng: SystemRandom::new(),
        }
    }

    /// Generate a new encryption key
    #[instrument(skip(self))]
    pub fn generate_key(&self, user_id: i32) -> Result<EncryptionKey, String> {
        let mut key_bytes = vec![0u8; 32]; // 256 bits
        self.rng.fill(&mut key_bytes)
            .map_err(|_| "Failed to generate random key".to_string())?;

        let key_data = base64::encode(&key_bytes);
        
        Ok(EncryptionKey {
            id: uuid::Uuid::new_v4().to_string(),
            user_id,
            public_key: key_data,
            created_at: Utc::now(),
            expires_at: Some(Utc::now() + Duration::days(90)),
            active: true,
        })
    }

    /// Encrypt plaintext
    #[instrument(skip(self, plaintext, key))]
    pub fn encrypt(&self, plaintext: &str, key: &EncryptionKey) -> Result<EncryptedMessage, String> {
        if !key.active {
            return Err("Key has been revoked".to_string());
        }

        if let Some(expires_at) = key.expires_at {
            if Utc::now() > expires_at {
                return Err("Key has expired".to_string());
            }
        }

        let key_bytes = base64::decode(&key.public_key)
            .map_err(|e| format!("Invalid key data: {}", e))?;

        let plaintext_bytes = plaintext.as_bytes();
        let mut nonce_bytes = vec![0u8; 12];
        self.rng.fill(&mut nonce_bytes)
            .map_err(|_| "Failed to generate nonce".to_string())?;

        let nonce = Nonce::try_assume_unique_for_key(&nonce_bytes)
            .map_err(|_| "Invalid nonce".to_string())?;

        let algorithm = match self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => &AES_256_GCM,
            EncryptionAlgorithm::ChaCha20Poly1305 => &CHACHA20_POLY1305,
        };

        let unbound_key = UnboundKey::new(algorithm, &key_bytes)
            .map_err(|_| "Failed to create encryption key".to_string())?;

        let mut sealing_key = SealingKey::new(unbound_key, CounterNonceSequence::new());

        let mut in_out = plaintext_bytes.to_vec();
        sealing_key.seal_in_place_append_tag(Aad::empty(), &mut in_out)
            .map_err(|_| "Encryption failed".to_string())?;

        let ciphertext = base64::encode(&in_out);
        let nonce_b64 = base64::encode(&nonce_bytes);

        debug!(
            key_id = %key.id,
            algorithm = ?self.algorithm,
            "Message encrypted"
        );

        Ok(EncryptedMessage {
            algorithm: self.algorithm.clone(),
            ciphertext,
            iv: nonce_b64.clone(),
            tag: nonce_b64,
            key_id: key.id.clone(),
            encrypted_at: Utc::now(),
        })
    }

    /// Decrypt ciphertext
    #[instrument(skip(self, encrypted, key))]
    pub fn decrypt(&self, encrypted: &EncryptedMessage, key: &EncryptionKey) -> Result<String, String> {
        if !key.active {
            return Err("Key has been revoked".to_string());
        }

        if encrypted.key_id != key.id {
            return Err("Key ID mismatch".to_string());
        }

        let key_bytes = base64::decode(&key.public_key)
            .map_err(|e| format!("Invalid key data: {}", e))?;

        let ciphertext_bytes = base64::decode(&encrypted.ciphertext)
            .map_err(|e| format!("Invalid ciphertext: {}", e))?;

        let nonce_bytes = base64::decode(&encrypted.iv)
            .map_err(|e| format!("Invalid nonce: {}", e))?;

        let nonce = Nonce::try_assume_unique_for_key(&nonce_bytes)
            .map_err(|_| "Invalid nonce".to_string())?;

        let algorithm = match encrypted.algorithm {
            EncryptionAlgorithm::Aes256Gcm => &AES_256_GCM,
            EncryptionAlgorithm::ChaCha20Poly1305 => &CHACHA20_POLY1305,
        };

        let unbound_key = UnboundKey::new(algorithm, &key_bytes)
            .map_err(|_| "Failed to create decryption key".to_string())?;

        let mut opening_key = OpeningKey::new(unbound_key, CounterNonceSequence::new());

        let mut in_out = ciphertext_bytes;
        let plaintext_bytes = opening_key.open_in_place(Aad::empty(), &mut in_out)
            .map_err(|_| "Decryption failed".to_string())?;

        let plaintext = String::from_utf8(plaintext_bytes.to_vec())
            .map_err(|e| format!("Invalid UTF-8: {}", e))?;

        debug!(
            key_id = %key.id,
            "Message decrypted"
        );

        Ok(plaintext)
    }
}

impl Default for RingEncryptionService {
    fn default() -> Self {
        Self::new(EncryptionAlgorithm::Aes256Gcm)
    }
}

/// Key management service with ring
pub struct RingKeyManagementService {
    keys: Arc<RwLock<HashMap<String, EncryptionKey>>>,
    user_keys: Arc<RwLock<HashMap<i32, Vec<String>>>>,
    encryption_service: RingEncryptionService,
}

impl RingKeyManagementService {
    /// Create new key management service
    pub fn new() -> Self {
        Self {
            keys: Arc::new(RwLock::new(HashMap::new())),
            user_keys: Arc::new(RwLock::new(HashMap::new())),
            encryption_service: RingEncryptionService::default(),
        }
    }

    /// Generate and store a new key for user
    #[instrument(skip(self))]
    pub fn generate_key_for_user(&self, user_id: i32) -> Result<EncryptionKey, String> {
        let key = self.encryption_service.generate_key(user_id)?;
        
        let mut keys = self.keys.write();
        let mut user_keys = self.user_keys.write();
        
        keys.insert(key.id.clone(), key.clone());
        user_keys.entry(user_id)
            .or_insert_with(Vec::new)
            .push(key.id.clone());

        debug!(
            user_id = user_id,
            key_id = %key.id,
            "Generated new encryption key"
        );

        Ok(key)
    }

    /// Get key by ID
    pub fn get_key(&self, key_id: &str) -> Option<EncryptionKey> {
        self.keys.read().get(key_id).cloned()
    }

    /// Get active key for user
    pub fn get_active_key(&self, user_id: i32) -> Option<EncryptionKey> {
        let user_keys = self.user_keys.read();
        let key_ids = user_keys.get(&user_id)?;
        
        let keys = self.keys.read();
        
        // Find first active, non-expired key
        for key_id in key_ids.iter().rev() {
            if let Some(key) = keys.get(key_id) {
                if key.active {
                    if let Some(expires_at) = key.expires_at {
                        if Utc::now() <= expires_at {
                            return Some(key.clone());
                        }
                    } else {
                        return Some(key.clone());
                    }
                }
            }
        }
        
        None
    }

    /// Revoke a key
    #[instrument(skip(self))]
    pub fn revoke_key(&self, key_id: &str) -> Result<(), String> {
        let mut keys = self.keys.write();
        
        if let Some(key) = keys.get_mut(key_id) {
            key.active = false;
            warn!(key_id = %key_id, "Key revoked");
            Ok(())
        } else {
            Err("Key not found".to_string())
        }
    }

    /// Rotate keys for user
    #[instrument(skip(self))]
    pub fn rotate_key(&self, user_id: i32) -> Result<EncryptionKey, String> {
        // Revoke old keys
        let user_keys = self.user_keys.read();
        if let Some(key_ids) = user_keys.get(&user_id) {
            let mut keys = self.keys.write();
            for key_id in key_ids {
                if let Some(key) = keys.get_mut(key_id) {
                    key.active = false;
                }
            }
        }
        drop(user_keys);

        // Generate new key
        self.generate_key_for_user(user_id)
    }

    /// Clean up expired keys
    pub fn cleanup_expired_keys(&self) -> usize {
        let mut keys = self.keys.write();
        let now = Utc::now();
        
        let expired_keys: Vec<String> = keys.iter()
            .filter(|(_, key)| {
                if let Some(expires_at) = key.expires_at {
                    now > expires_at
                } else {
                    false
                }
            })
            .map(|(key_id, _)| key_id.clone())
            .collect();

        let count = expired_keys.len();
        for key_id in expired_keys {
            keys.remove(&key_id);
        }

        if count > 0 {
            debug!(count = count, "Cleaned up expired keys");
        }

        count
    }
}

impl Default for RingKeyManagementService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_generation() {
        let service = RingEncryptionService::default();
        let key = service.generate_key(100).unwrap();
        
        assert_eq!(key.user_id, 100);
        assert!(!key.key_data.is_empty());
        assert!(!key.revoked);
    }

    #[test]
    fn test_encrypt_decrypt() {
        let service = RingEncryptionService::default();
        let key = service.generate_key(100).unwrap();
        
        let plaintext = "Hello, World!";
        let encrypted = service.encrypt(plaintext, &key).unwrap();
        
        assert_ne!(encrypted.ciphertext, plaintext);
        assert_eq!(encrypted.key_id, key.key_id);
        
        let decrypted = service.decrypt(&encrypted, &key).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_key_management() {
        let kms = RingKeyManagementService::new();
        
        let key = kms.generate_key_for_user(100).unwrap();
        assert_eq!(key.user_id, 100);
        
        let retrieved = kms.get_key(&key.id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, key.id);
    }

    #[test]
    fn test_key_revocation() {
        let kms = RingKeyManagementService::new();
        let key = kms.generate_key_for_user(100).unwrap();
        
        kms.revoke_key(&key.id).unwrap();
        
        let retrieved = kms.get_key(&key.id).unwrap();
        assert!(!retrieved.active);
    }

    #[test]
    fn test_key_rotation() {
        let kms = RingKeyManagementService::new();
        let old_key = kms.generate_key_for_user(100).unwrap();
        
        let new_key = kms.rotate_key(100).unwrap();
        
        assert_ne!(old_key.id, new_key.id);
        
        let old_retrieved = kms.get_key(&old_key.id).unwrap();
        assert!(!old_retrieved.active);
    }

    #[test]
    fn test_active_key_retrieval() {
        let kms = RingKeyManagementService::new();
        kms.generate_key_for_user(100).unwrap();
        
        let active = kms.get_active_key(100);
        assert!(active.is_some());
        assert!(active.unwrap().active);
    }

    #[test]
    fn test_encrypt_with_revoked_key() {
        let service = RingEncryptionService::default();
        let mut key = service.generate_key(100).unwrap();
        key.active = false;
        
        let result = service.encrypt("test", &key);
        assert!(result.is_err());
    }

    #[test]
    fn test_different_algorithms() {
        let aes_service = RingEncryptionService::new(EncryptionAlgorithm::Aes256Gcm);
        let chacha_service = RingEncryptionService::new(EncryptionAlgorithm::ChaCha20Poly1305);
        
        let aes_key = aes_service.generate_key(100).unwrap();
        let chacha_key = chacha_service.generate_key(100).unwrap();
        
        let plaintext = "Test message";
        
        let aes_encrypted = aes_service.encrypt(plaintext, &aes_key).unwrap();
        let chacha_encrypted = chacha_service.encrypt(plaintext, &chacha_key).unwrap();
        
        assert_ne!(aes_encrypted.ciphertext, chacha_encrypted.ciphertext);
        
        let aes_decrypted = aes_service.decrypt(&aes_encrypted, &aes_key).unwrap();
        let chacha_decrypted = chacha_service.decrypt(&chacha_encrypted, &chacha_key).unwrap();
        
        assert_eq!(aes_decrypted, plaintext);
        assert_eq!(chacha_decrypted, plaintext);
    }
}
