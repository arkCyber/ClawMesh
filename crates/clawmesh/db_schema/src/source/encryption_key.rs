//! Encryption Key Data Models
//! 
//! Aerospace-grade data models for encryption key persistence

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use diesel::prelude::*;

#[cfg(feature = "full")]
use crate::schema::{encryption_key, key_rotation_history};

// ============================================================================
// Encryption Key Models
// ============================================================================

/// Encryption algorithm types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EncryptionAlgorithm {
    #[serde(rename = "AES-256-GCM")]
    Aes256Gcm,
    #[serde(rename = "ChaCha20-Poly1305")]
    ChaCha20Poly1305,
    #[serde(rename = "AES-128-GCM")]
    Aes128Gcm,
}

impl std::fmt::Display for EncryptionAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Aes256Gcm => write!(f, "AES-256-GCM"),
            Self::ChaCha20Poly1305 => write!(f, "ChaCha20-Poly1305"),
            Self::Aes128Gcm => write!(f, "AES-128-GCM"),
        }
    }
}

impl From<String> for EncryptionAlgorithm {
    fn from(s: String) -> Self {
        match s.as_str() {
            "AES-256-GCM" => Self::Aes256Gcm,
            "ChaCha20-Poly1305" => Self::ChaCha20Poly1305,
            "AES-128-GCM" => Self::Aes128Gcm,
            _ => Self::Aes256Gcm, // Default
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = encryption_key)]
#[diesel(primary_key(id))]
pub struct EncryptionKey {
    pub id: String,
    pub user_id: i32,
    pub key_data: Vec<u8>,
    pub algorithm: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub usage_count: i64,
    pub is_active: bool,
}

impl EncryptionKey {
    /// Get the algorithm as enum
    pub fn algorithm_enum(&self) -> EncryptionAlgorithm {
        EncryptionAlgorithm::from(self.algorithm.clone())
    }
    
    /// Check if key is valid (active, not revoked, not expired)
    pub fn is_valid(&self) -> bool {
        self.is_active
            && self.revoked_at.is_none()
            && (self.expires_at.is_none() || self.expires_at.unwrap() > Utc::now())
    }
    
    /// Check if key is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            expires_at <= Utc::now()
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = encryption_key)]
pub struct EncryptionKeyInsertForm {
    pub id: String,
    pub user_id: i32,
    pub key_data: Vec<u8>,
    pub algorithm: String,
    pub expires_at: Option<DateTime<Utc>>,
}

impl EncryptionKeyInsertForm {
    /// Create new key insert form
    pub fn new(user_id: i32, key_data: Vec<u8>, algorithm: EncryptionAlgorithm) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            user_id,
            key_data,
            algorithm: algorithm.to_string(),
            expires_at: None,
        }
    }
    
    /// Set expiration time
    pub fn with_expiration(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = encryption_key)]
pub struct EncryptionKeyUpdateForm {
    pub revoked_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub usage_count: Option<i64>,
    pub is_active: Option<bool>,
}

// ============================================================================
// Key Rotation History Models
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = key_rotation_history)]
pub struct KeyRotationHistory {
    pub id: i32,
    pub user_id: i32,
    pub old_key_id: String,
    pub new_key_id: String,
    pub rotation_reason: Option<String>,
    pub rotated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = key_rotation_history)]
pub struct KeyRotationHistoryInsertForm {
    pub user_id: i32,
    pub old_key_id: String,
    pub new_key_id: String,
    pub rotation_reason: Option<String>,
}

// ============================================================================
// CRUD Implementations
// ============================================================================

#[cfg(feature = "full")]
impl EncryptionKey {
    /// Create a new encryption key
    pub async fn create(
        pool: &mut impl diesel_async::AsyncConnection<Backend = diesel::pg::Pg>,
        form: &EncryptionKeyInsertForm,
    ) -> Result<Self, diesel::result::Error> {
        use crate::schema::encryption_key::dsl::*;
        
        diesel::insert_into(encryption_key)
            .values(form)
            .get_result::<Self>(pool)
            .await
    }
    
    /// Read an encryption key by ID
    pub async fn read(
        pool: &mut impl diesel_async::AsyncConnection<Backend = diesel::pg::Pg>,
        key_id: &str,
    ) -> Result<Self, diesel::result::Error> {
        use crate::schema::encryption_key::dsl::*;
        
        encryption_key
            .find(key_id)
            .first::<Self>(pool)
            .await
    }
    
    /// Update an encryption key
    pub async fn update(
        pool: &mut impl diesel_async::AsyncConnection<Backend = diesel::pg::Pg>,
        key_id: &str,
        form: &EncryptionKeyUpdateForm,
    ) -> Result<Self, diesel::result::Error> {
        use crate::schema::encryption_key::dsl::*;
        
        diesel::update(encryption_key.find(key_id))
            .set(form)
            .get_result::<Self>(pool)
            .await
    }
    
    /// Delete an encryption key
    pub async fn delete(
        pool: &mut impl diesel_async::AsyncConnection<Backend = diesel::pg::Pg>,
        key_id: &str,
    ) -> Result<usize, diesel::result::Error> {
        use crate::schema::encryption_key::dsl::*;
        
        diesel::delete(encryption_key.find(key_id))
            .execute(pool)
            .await
    }
    
    /// Get active key for user
    pub async fn get_active_for_user(
        pool: &mut impl diesel_async::AsyncConnection<Backend = diesel::pg::Pg>,
        uid: i32,
    ) -> Result<Option<Self>, diesel::result::Error> {
        use crate::schema::encryption_key::dsl::*;
        
        encryption_key
            .filter(user_id.eq(uid))
            .filter(is_active.eq(true))
            .filter(revoked_at.is_null())
            .order(created_at.desc())
            .first::<Self>(pool)
            .await
            .optional()
    }
    
    /// Get all active keys for user
    pub async fn get_all_active_for_user(
        pool: &mut impl diesel_async::AsyncConnection<Backend = diesel::pg::Pg>,
        uid: i32,
    ) -> Result<Vec<Self>, diesel::result::Error> {
        use crate::schema::encryption_key::dsl::*;
        
        encryption_key
            .filter(user_id.eq(uid))
            .filter(is_active.eq(true))
            .filter(revoked_at.is_null())
            .order(created_at.desc())
            .load::<Self>(pool)
            .await
    }
    
    /// Revoke a key
    pub async fn revoke(
        pool: &mut impl diesel_async::AsyncConnection<Backend = diesel::pg::Pg>,
        key_id: &str,
    ) -> Result<Self, diesel::result::Error> {
        let form = EncryptionKeyUpdateForm {
            revoked_at: Some(Utc::now()),
            is_active: Some(false),
            last_used_at: None,
            usage_count: None,
        };
        
        Self::update(pool, key_id, &form).await
    }
    
    /// Revoke all keys for user
    pub async fn revoke_all_for_user(
        pool: &mut impl diesel_async::AsyncConnection<Backend = diesel::pg::Pg>,
        uid: i32,
    ) -> Result<usize, diesel::result::Error> {
        use crate::schema::encryption_key::dsl::*;
        
        diesel::update(
            encryption_key
                .filter(user_id.eq(uid))
                .filter(revoked_at.is_null())
        )
        .set((
            revoked_at.eq(Some(Utc::now())),
            is_active.eq(false),
        ))
        .execute(pool)
        .await
    }
    
    /// Increment usage count
    pub async fn increment_usage(
        pool: &mut impl diesel_async::AsyncConnection<Backend = diesel::pg::Pg>,
        key_id: &str,
    ) -> Result<Self, diesel::result::Error> {
        use crate::schema::encryption_key::dsl::*;
        
        diesel::update(encryption_key.find(key_id))
            .set((
                last_used_at.eq(Some(Utc::now())),
                usage_count.eq(usage_count + 1),
            ))
            .get_result::<Self>(pool)
            .await
    }
    
    /// Clean up expired keys
    pub async fn cleanup_expired(
        pool: &mut impl diesel_async::AsyncConnection<Backend = diesel::pg::Pg>,
    ) -> Result<usize, diesel::result::Error> {
        use crate::schema::encryption_key::dsl::*;
        
        diesel::update(
            encryption_key
                .filter(expires_at.is_not_null())
                .filter(expires_at.lt(Utc::now()))
                .filter(is_active.eq(true))
        )
        .set(is_active.eq(false))
        .execute(pool)
        .await
    }
    
    /// Get key count for user
    pub async fn count_for_user(
        pool: &mut impl diesel_async::AsyncConnection<Backend = diesel::pg::Pg>,
        uid: i32,
    ) -> Result<i64, diesel::result::Error> {
        use crate::schema::encryption_key::dsl::*;
        
        encryption_key
            .filter(user_id.eq(uid))
            .filter(is_active.eq(true))
            .filter(revoked_at.is_null())
            .count()
            .get_result(pool)
            .await
    }
}

#[cfg(feature = "full")]
impl KeyRotationHistory {
    /// Create a new rotation history entry
    pub async fn create(
        pool: &mut impl diesel_async::AsyncConnection<Backend = diesel::pg::Pg>,
        form: &KeyRotationHistoryInsertForm,
    ) -> Result<Self, diesel::result::Error> {
        use crate::schema::key_rotation_history::dsl::*;
        
        diesel::insert_into(key_rotation_history)
            .values(form)
            .get_result::<Self>(pool)
            .await
    }
    
    /// Get rotation history for user
    pub async fn get_for_user(
        pool: &mut impl diesel_async::AsyncConnection<Backend = diesel::pg::Pg>,
        uid: i32,
        limit_count: i64,
    ) -> Result<Vec<Self>, diesel::result::Error> {
        use crate::schema::key_rotation_history::dsl::*;
        
        key_rotation_history
            .filter(user_id.eq(uid))
            .order(rotated_at.desc())
            .limit(limit_count)
            .load::<Self>(pool)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_encryption_algorithm_display() {
        assert_eq!(EncryptionAlgorithm::Aes256Gcm.to_string(), "AES-256-GCM");
        assert_eq!(EncryptionAlgorithm::ChaCha20Poly1305.to_string(), "ChaCha20-Poly1305");
        assert_eq!(EncryptionAlgorithm::Aes128Gcm.to_string(), "AES-128-GCM");
    }
    
    #[test]
    fn test_encryption_algorithm_from_string() {
        assert_eq!(
            EncryptionAlgorithm::from("AES-256-GCM".to_string()),
            EncryptionAlgorithm::Aes256Gcm
        );
        assert_eq!(
            EncryptionAlgorithm::from("ChaCha20-Poly1305".to_string()),
            EncryptionAlgorithm::ChaCha20Poly1305
        );
    }
    
    #[test]
    fn test_key_insert_form_creation() {
        let form = EncryptionKeyInsertForm::new(
            1,
            vec![1, 2, 3, 4],
            EncryptionAlgorithm::Aes256Gcm,
        );
        
        assert_eq!(form.user_id, 1);
        assert_eq!(form.key_data, vec![1, 2, 3, 4]);
        assert_eq!(form.algorithm, "AES-256-GCM");
        assert!(form.expires_at.is_none());
    }
    
    #[test]
    fn test_key_insert_form_with_expiration() {
        let expires = Utc::now() + chrono::Duration::days(30);
        let form = EncryptionKeyInsertForm::new(
            1,
            vec![1, 2, 3, 4],
            EncryptionAlgorithm::Aes256Gcm,
        ).with_expiration(expires);
        
        assert!(form.expires_at.is_some());
    }
}
