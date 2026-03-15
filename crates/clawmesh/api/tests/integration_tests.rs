//! Integration Tests for ClawMesh API
//!
//! Tests the complete API flow including authentication, rate limiting,
//! error handling, and message delivery.

use actix_web::{test, web, App};
use clawmesh_api::{
    ClawMeshError, ClawMeshMetrics, ErrorCode, RateLimiter, RateLimitAction,
    SecurityContext, UserRole,
};

#[actix_web::test]
async fn test_rate_limiter_integration() {
    let limiter = RateLimiter::default();
    
    // First request should succeed
    assert!(limiter.check_limit(1, RateLimitAction::SendMessage).await.is_ok());
    
    // Get remaining quota
    let remaining = limiter.get_remaining(1, RateLimitAction::SendMessage).await;
    assert!(remaining > 0);
    
    // Cleanup
    let removed = limiter.cleanup_inactive().await;
    assert_eq!(removed, 0); // No inactive users yet
}

#[actix_web::test]
async fn test_rate_limiter_exceeded() {
    use clawmesh_api::RateLimitConfig;
    
    let config = RateLimitConfig {
        messages_per_minute: 3,
        burst_size: 0,
        ..Default::default()
    };
    let limiter = RateLimiter::new(config);
    
    // Consume all tokens
    for _ in 0..3 {
        limiter.check_limit(1, RateLimitAction::SendMessage).await.ok();
    }
    
    // Next request should fail
    let result = limiter.check_limit(1, RateLimitAction::SendMessage).await;
    assert!(result.is_err());
    
    if let Err(e) = result {
        assert_eq!(e.code, ErrorCode::RateLimited);
    }
}

#[actix_web::test]
async fn test_security_context_creation() {
    let ctx = SecurityContext::new(1, "testuser".to_string(), UserRole::User);
    
    assert_eq!(ctx.user_id, 1);
    assert_eq!(ctx.username, "testuser");
    assert_eq!(ctx.role, UserRole::User);
    assert!(!ctx.is_admin());
    assert!(!ctx.is_moderator());
}

#[actix_web::test]
async fn test_security_context_permissions() {
    let user = SecurityContext::new(1, "user".to_string(), UserRole::User);
    let admin = SecurityContext::new(2, "admin".to_string(), UserRole::Admin);
    
    // User can act on self
    assert!(user.can_act_on_user(1).is_ok());
    assert!(user.can_act_on_user(2).is_err());
    
    // Admin can act on anyone
    assert!(admin.can_act_on_user(1).is_ok());
    assert!(admin.can_act_on_user(2).is_ok());
    assert!(admin.can_act_on_user(999).is_ok());
}

#[actix_web::test]
async fn test_metrics_collection() {
    let metrics = ClawMeshMetrics::new();
    
    // Increment counters
    metrics.messages_sent_total.inc();
    metrics.messages_sent_total.inc();
    metrics.messages_delivered_total.inc();
    
    // Set gauges
    metrics.users_online.set(100.0);
    metrics.cache_size.set(1024.0);
    
    // Observe histogram
    metrics.message_delivery_duration.observe(0.05);
    metrics.message_delivery_duration.observe(0.10);
    
    // Export metrics
    let export = metrics.export();
    assert!(export.contains("clawmesh_messages_sent_total"));
    assert!(export.contains("clawmesh_users_online"));
    assert!(export.contains("clawmesh_message_delivery_duration"));
}

#[actix_web::test]
async fn test_error_handling() {
    let error = ClawMeshError::new(ErrorCode::InvalidInput);
    assert_eq!(error.code, ErrorCode::InvalidInput);
    
    let error_with_msg = ClawMeshError::with_message(
        ErrorCode::NotFound,
        "User not found",
    );
    assert_eq!(error_with_msg.message, "User not found");
    
    let error_with_field = error_with_msg.with_field("user_id");
    assert_eq!(error_with_field.field, Some("user_id".to_string()));
}

#[actix_web::test]
async fn test_validation_helpers() {
    use clawmesh_api::error::validation;
    
    // Valid user ID
    assert!(validation::validate_user_id(1, "user_id").is_ok());
    assert!(validation::validate_user_id(0, "user_id").is_err());
    assert!(validation::validate_user_id(-1, "user_id").is_err());
    
    // Valid username
    assert!(validation::validate_username("testuser").is_ok());
    assert!(validation::validate_username("").is_err());
    assert!(validation::validate_username("ab").is_err()); // Too short
    
    // Valid message content
    assert!(validation::validate_message_content("Hello, World!").is_ok());
    assert!(validation::validate_message_content("").is_err());
    
    // Valid pagination
    assert!(validation::validate_pagination(1, 20).is_ok());
    assert!(validation::validate_pagination(0, 20).is_err());
    assert!(validation::validate_pagination(1, 0).is_err());
    assert!(validation::validate_pagination(1, 1001).is_err()); // Too large
}

#[actix_web::test]
async fn test_concurrent_rate_limiting() {
    use std::sync::Arc;
    
    let limiter = Arc::new(RateLimiter::default());
    let mut handles = vec![];
    
    // Simulate 50 concurrent users
    for i in 0..50 {
        let limiter_clone = Arc::clone(&limiter);
        let handle = tokio::spawn(async move {
            limiter_clone.check_limit(i, RateLimitAction::SendMessage).await
        });
        handles.push(handle);
    }
    
    // All should succeed (different users)
    for handle in handles {
        assert!(handle.await.unwrap().is_ok());
    }
    
    let stats = limiter.get_stats().await;
    assert_eq!(stats.tracked_users, 50);
}

#[actix_web::test]
async fn test_permission_checks() {
    use clawmesh_api::{Permission, check_auth_permission};
    
    let user = SecurityContext::new(1, "user".to_string(), UserRole::User);
    let admin = SecurityContext::new(2, "admin".to_string(), UserRole::Admin);
    
    // User permissions
    assert!(check_auth_permission(&user, Permission::SendFriendRequest).is_ok());
    assert!(check_auth_permission(&user, Permission::SendDirectMessage).is_ok());
    assert!(check_auth_permission(&user, Permission::AdminAccess).is_err());
    
    // Admin permissions
    assert!(check_auth_permission(&admin, Permission::SendFriendRequest).is_ok());
    assert!(check_auth_permission(&admin, Permission::AdminAccess).is_ok());
    assert!(check_auth_permission(&admin, Permission::ManageUsers).is_ok());
}

#[actix_web::test]
async fn test_credit_requirements() {
    let mut user = SecurityContext::new(1, "user".to_string(), UserRole::User);
    user.credit_score = Some(50);
    
    // Sufficient credit
    assert!(user.require_credit(50).is_ok());
    assert!(user.require_credit(10).is_ok());
    
    // Insufficient credit
    assert!(user.require_credit(100).is_err());
}

#[actix_web::test]
async fn test_metrics_histogram_timing() {
    let metrics = ClawMeshMetrics::new();
    
    let result = metrics.message_delivery_duration.time(|| {
        std::thread::sleep(std::time::Duration::from_millis(10));
        42
    });
    
    assert_eq!(result, 42);
    
    let export = metrics.export();
    assert!(export.contains("clawmesh_message_delivery_duration"));
}

#[actix_web::test]
async fn test_error_response_format() {
    use actix_web::ResponseError;
    
    let error = ClawMeshError::with_message(
        ErrorCode::InvalidInput,
        "Invalid user ID",
    ).with_field("user_id");
    
    let response = error.error_response();
    assert_eq!(response.status(), actix_web::http::StatusCode::BAD_REQUEST);
}

#[actix_web::test]
async fn test_multiple_users_rate_limiting() {
    let limiter = RateLimiter::default();
    
    // Different users should have independent limits
    for user_id in 1..=10 {
        for _ in 0..5 {
            assert!(limiter.check_limit(user_id, RateLimitAction::SendMessage).await.is_ok());
        }
    }
    
    let stats = limiter.get_stats().await;
    assert_eq!(stats.tracked_users, 10);
}

#[actix_web::test]
async fn test_audit_logging() {
    let ctx = SecurityContext::new(1, "testuser".to_string(), UserRole::User);
    
    // Audit log should not panic
    ctx.audit_log("send_message", "direct_message", Some(123));
    ctx.audit_log("friend_request", "friendship", Some(456));
}

#[actix_web::test]
async fn test_system_context() {
    let ctx = SecurityContext::system();
    
    assert_eq!(ctx.user_id, 0);
    assert_eq!(ctx.username, "system");
    assert!(ctx.is_system());
    assert!(ctx.is_admin());
    assert!(ctx.is_moderator());
    
    // System can act on anyone
    assert!(ctx.can_act_on_user(1).is_ok());
    assert!(ctx.can_act_on_user(999).is_ok());
}
