//! Rate Limiting Module
//!
//! Implements token bucket algorithm for rate limiting to prevent spam and DoS attacks.
//! Supports 100,000+ concurrent users with minimal memory overhead.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration};
use tracing::{debug, warn, instrument};

use crate::error::{ClawMeshError, ClawMeshResult, ErrorCode};

/// Rate limit configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum messages per minute
    pub messages_per_minute: u32,
    /// Maximum friend requests per hour
    pub friend_requests_per_hour: u32,
    /// Maximum API calls per minute
    pub api_calls_per_minute: u32,
    /// Burst allowance (extra tokens)
    pub burst_size: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            messages_per_minute: 60,
            friend_requests_per_hour: 20,
            api_calls_per_minute: 120,
            burst_size: 10,
        }
    }
}

/// Token bucket for rate limiting
#[derive(Debug, Clone)]
struct TokenBucket {
    /// Available tokens
    tokens: f64,
    /// Maximum tokens (capacity)
    capacity: f64,
    /// Token refill rate (tokens per second)
    refill_rate: f64,
    /// Last refill timestamp
    last_refill: DateTime<Utc>,
}

impl TokenBucket {
    /// Create a new token bucket
    fn new(capacity: u32, refill_rate_per_minute: u32) -> Self {
        let capacity_f64 = capacity as f64;
        Self {
            tokens: capacity_f64,
            capacity: capacity_f64,
            refill_rate: refill_rate_per_minute as f64 / 60.0, // Convert to per-second
            last_refill: Utc::now(),
        }
    }

    /// Refill tokens based on elapsed time
    fn refill(&mut self) {
        let now = Utc::now();
        let elapsed = (now - self.last_refill).num_milliseconds() as f64 / 1000.0;
        
        if elapsed > 0.0 {
            let new_tokens = elapsed * self.refill_rate;
            self.tokens = (self.tokens + new_tokens).min(self.capacity);
            self.last_refill = now;
        }
    }

    /// Try to consume tokens
    fn try_consume(&mut self, count: f64) -> bool {
        self.refill();
        
        if self.tokens >= count {
            self.tokens -= count;
            true
        } else {
            false
        }
    }

    /// Get remaining tokens
    fn remaining(&mut self) -> u32 {
        self.refill();
        self.tokens.floor() as u32
    }

    /// Get time until next token available (seconds)
    fn retry_after(&mut self) -> u32 {
        self.refill();
        
        if self.tokens >= 1.0 {
            0
        } else {
            let tokens_needed = 1.0 - self.tokens;
            (tokens_needed / self.refill_rate).ceil() as u32
        }
    }
}

/// Rate limiter for different action types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RateLimitAction {
    /// Send direct message
    SendMessage,
    /// Send friend request
    SendFriendRequest,
    /// API call
    ApiCall,
}

impl RateLimitAction {
    /// Get the bucket key suffix for this action
    fn key_suffix(&self) -> &'static str {
        match self {
            Self::SendMessage => "msg",
            Self::SendFriendRequest => "freq",
            Self::ApiCall => "api",
        }
    }

    /// Get rate limit for this action
    fn get_limit(&self, config: &RateLimitConfig) -> (u32, u32) {
        match self {
            Self::SendMessage => (config.messages_per_minute, 60),
            Self::SendFriendRequest => (config.friend_requests_per_hour, 3600),
            Self::ApiCall => (config.api_calls_per_minute, 60),
        }
    }
}

/// User rate limit state
struct UserRateLimits {
    /// User ID
    user_id: i32,
    /// Token buckets for different actions
    buckets: HashMap<RateLimitAction, TokenBucket>,
    /// Last access time (for cleanup)
    last_access: DateTime<Utc>,
}

impl UserRateLimits {
    fn new(user_id: i32, config: &RateLimitConfig) -> Self {
        let mut buckets = HashMap::new();
        
        buckets.insert(
            RateLimitAction::SendMessage,
            TokenBucket::new(
                config.messages_per_minute + config.burst_size,
                config.messages_per_minute,
            ),
        );
        
        buckets.insert(
            RateLimitAction::SendFriendRequest,
            TokenBucket::new(
                config.friend_requests_per_hour,
                config.friend_requests_per_hour,
            ),
        );
        
        buckets.insert(
            RateLimitAction::ApiCall,
            TokenBucket::new(
                config.api_calls_per_minute + config.burst_size,
                config.api_calls_per_minute,
            ),
        );

        Self {
            user_id,
            buckets,
            last_access: Utc::now(),
        }
    }

    fn update_access(&mut self) {
        self.last_access = Utc::now();
    }
}

/// Rate limiter service
pub struct RateLimiter {
    /// User rate limits
    limits: Arc<RwLock<HashMap<i32, UserRateLimits>>>,
    /// Configuration
    config: RateLimitConfig,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            limits: Arc::new(RwLock::new(HashMap::with_capacity(10_000))),
            config,
        }
    }

    /// Create with default configuration
    pub fn default() -> Self {
        Self::new(RateLimitConfig::default())
    }

    /// Check if action is allowed for user
    #[instrument(skip(self))]
    pub async fn check_limit(
        &self,
        user_id: i32,
        action: RateLimitAction,
    ) -> ClawMeshResult<()> {
        let mut limits: tokio::sync::RwLockWriteGuard<'_, HashMap<i32, UserRateLimits>> = self.limits.write().await;
        
        let user_limits = limits
            .entry(user_id)
            .or_insert_with(|| UserRateLimits::new(user_id, &self.config));
        
        user_limits.update_access();
        
        let bucket = user_limits.buckets.get_mut(&action).unwrap();
        
        if bucket.try_consume(1.0) {
            debug!(
                user_id = user_id,
                action = ?action,
                remaining = bucket.remaining(),
                "Rate limit check passed"
            );
            Ok(())
        } else {
            let retry_after = bucket.retry_after();
            warn!(
                user_id = user_id,
                action = ?action,
                retry_after = retry_after,
                "Rate limit exceeded"
            );
            
            Err(ClawMeshError::with_message(
                ErrorCode::RateLimited,
                format!("Rate limit exceeded. Try again in {} seconds", retry_after),
            ).with_details(format!("Action: {:?}, User: {}", action, user_id)))
        }
    }

    /// Get remaining quota for user
    pub async fn get_remaining(&self, user_id: i32, action: RateLimitAction) -> u32 {
        let mut limits: tokio::sync::RwLockWriteGuard<'_, HashMap<i32, UserRateLimits>> = self.limits.write().await;
        
        if let Some(user_limits) = limits.get_mut(&user_id) {
            if let Some(bucket) = user_limits.buckets.get_mut(&action) {
                return TokenBucket::remaining(bucket);
            }
        }
        
        // Return default capacity if user not found
        let (limit, _) = action.get_limit(&self.config);
        limit
    }

    /// Clean up inactive users (older than 1 hour)
    pub async fn cleanup_inactive(&self) -> usize {
        let mut limits: tokio::sync::RwLockWriteGuard<'_, HashMap<i32, UserRateLimits>> = self.limits.write().await;
        let threshold = Utc::now() - Duration::hours(1);
        
        let before = limits.len();
        limits.retain(|_, user_limits| user_limits.last_access > threshold);
        let removed = before - limits.len();
        
        if removed > 0 {
            debug!(removed = removed, "Cleaned up inactive rate limit entries");
        }
        
        removed
    }

    /// Get statistics
    pub async fn get_stats(&self) -> RateLimiterStats {
        let limits: tokio::sync::RwLockReadGuard<'_, HashMap<i32, UserRateLimits>> = self.limits.read().await;
        RateLimiterStats {
            tracked_users: limits.len(),
            config: self.config.clone(),
        }
    }
}

/// Rate limiter statistics
#[derive(Debug, Clone)]
pub struct RateLimiterStats {
    /// Number of tracked users
    pub tracked_users: usize,
    /// Current configuration
    pub config: RateLimitConfig,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration as TokioDuration};

    #[test]
    fn test_token_bucket_creation() {
        let bucket = TokenBucket::new(60, 60);
        assert_eq!(bucket.capacity, 60.0);
        assert_eq!(bucket.tokens, 60.0);
    }

    #[test]
    fn test_token_bucket_consume() {
        let mut bucket = TokenBucket::new(10, 60);
        
        assert!(bucket.try_consume(1.0));
        assert_eq!(bucket.remaining(), 9);
        
        assert!(bucket.try_consume(5.0));
        assert_eq!(bucket.remaining(), 4);
        
        assert!(!bucket.try_consume(10.0)); // Not enough tokens
        assert_eq!(bucket.remaining(), 4);
    }

    #[tokio::test]
    async fn test_rate_limiter_basic() {
        let limiter = RateLimiter::default();
        
        // First request should succeed
        assert!(limiter.check_limit(1, RateLimitAction::SendMessage).await.is_ok());
        
        // Should have remaining quota
        let remaining = limiter.get_remaining(1, RateLimitAction::SendMessage).await;
        assert!(remaining > 0);
    }

    #[tokio::test]
    async fn test_rate_limiter_exceed() {
        let config = RateLimitConfig {
            messages_per_minute: 5,
            burst_size: 0,
            ..Default::default()
        };
        let limiter = RateLimiter::new(config);
        
        // Consume all tokens
        for _ in 0..5 {
            assert!(limiter.check_limit(1, RateLimitAction::SendMessage).await.is_ok());
        }
        
        // Next request should fail
        assert!(limiter.check_limit(1, RateLimitAction::SendMessage).await.is_err());
    }

    #[tokio::test]
    async fn test_rate_limiter_refill() {
        let config = RateLimitConfig {
            messages_per_minute: 60, // 1 per second
            burst_size: 0,
            ..Default::default()
        };
        let limiter = RateLimiter::new(config);
        
        // Consume all tokens
        for _ in 0..60 {
            limiter.check_limit(1, RateLimitAction::SendMessage).await.ok();
        }
        
        // Should be rate limited
        assert!(limiter.check_limit(1, RateLimitAction::SendMessage).await.is_err());
        
        // Wait for refill (1 second = 1 token)
        sleep(TokioDuration::from_secs(1)).await;
        
        // Should succeed now
        assert!(limiter.check_limit(1, RateLimitAction::SendMessage).await.is_ok());
    }

    #[tokio::test]
    async fn test_rate_limiter_multiple_users() {
        let limiter = RateLimiter::default();
        
        // Different users should have independent limits
        assert!(limiter.check_limit(1, RateLimitAction::SendMessage).await.is_ok());
        assert!(limiter.check_limit(2, RateLimitAction::SendMessage).await.is_ok());
        assert!(limiter.check_limit(3, RateLimitAction::SendMessage).await.is_ok());
        
        let stats = limiter.get_stats().await;
        assert_eq!(stats.tracked_users, 3);
    }

    #[tokio::test]
    async fn test_cleanup_inactive() {
        let limiter = RateLimiter::default();
        
        // Create some entries
        for i in 0..10 {
            limiter.check_limit(i, RateLimitAction::SendMessage).await.ok();
        }
        
        assert_eq!(limiter.get_stats().await.tracked_users, 10);
        
        // Cleanup won't remove recent entries
        let removed = limiter.cleanup_inactive().await;
        assert_eq!(removed, 0);
    }
}
