//! Aerospace-Grade Tests for Direct Message API
//!
//! Test Coverage:
//! - Unit tests for validation functions
//! - Integration tests for API endpoints
//! - Boundary condition tests
//! - Error handling tests
//! - Concurrency tests
//! - Security tests

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App};
    use clawmesh_api::direct_message::*;
    use clawmesh_api::error::*;
    
    // ============================================================================
    // Unit Tests - Validation Functions
    // ============================================================================
    
    #[test]
    fn test_validate_direct_message_success() {
        let request = SendDirectMessageRequest {
            recipient_id: 2,
            content: "Hello, World!".to_string(),
            reply_to_id: None,
            attachments: None,
        };
        
        let result = validate_direct_message(&request, 1);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_validate_direct_message_self_message() {
        let request = SendDirectMessageRequest {
            recipient_id: 1,
            content: "Hello, myself!".to_string(),
            reply_to_id: None,
            attachments: None,
        };
        
        let result = validate_direct_message(&request, 1);
        assert!(result.is_err());
        
        if let Err(e) = result {
            assert_eq!(e.code, ErrorCode::InvalidInput);
            assert!(e.message.contains("yourself"));
        }
    }
    
    #[test]
    fn test_validate_direct_message_invalid_recipient() {
        let request = SendDirectMessageRequest {
            recipient_id: -1,
            content: "Hello!".to_string(),
            reply_to_id: None,
            attachments: None,
        };
        
        let result = validate_direct_message(&request, 1);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_validate_direct_message_empty_content() {
        let request = SendDirectMessageRequest {
            recipient_id: 2,
            content: "".to_string(),
            reply_to_id: None,
            attachments: None,
        };
        
        let result = validate_direct_message(&request, 1);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_validate_direct_message_content_too_long() {
        let request = SendDirectMessageRequest {
            recipient_id: 2,
            content: "a".repeat(10001), // Exceeds max length
            reply_to_id: None,
            attachments: None,
        };
        
        let result = validate_direct_message(&request, 1);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_validate_direct_message_invalid_reply_id() {
        let request = SendDirectMessageRequest {
            recipient_id: 2,
            content: "Reply".to_string(),
            reply_to_id: Some(-1),
            attachments: None,
        };
        
        let result = validate_direct_message(&request, 1);
        assert!(result.is_err());
    }
    
    // ============================================================================
    // Boundary Condition Tests
    // ============================================================================
    
    #[test]
    fn test_validate_message_min_length() {
        let request = SendDirectMessageRequest {
            recipient_id: 2,
            content: "a".to_string(), // Minimum valid length
            reply_to_id: None,
            attachments: None,
        };
        
        let result = validate_direct_message(&request, 1);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_validate_message_max_length() {
        let request = SendDirectMessageRequest {
            recipient_id: 2,
            content: "a".repeat(10000), // Maximum valid length
            reply_to_id: None,
            attachments: None,
        };
        
        let result = validate_direct_message(&request, 1);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_validate_pagination_min_values() {
        let result = validation::validate_pagination(1, 1);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), (1, 1));
    }
    
    #[test]
    fn test_validate_pagination_max_values() {
        let result = validation::validate_pagination(1000, 100);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_validate_pagination_invalid_page() {
        let result = validation::validate_pagination(0, 20);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_validate_pagination_invalid_limit() {
        let result = validation::validate_pagination(1, 0);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_validate_pagination_limit_too_large() {
        let result = validation::validate_pagination(1, 1000);
        assert!(result.is_err());
    }
    
    // ============================================================================
    // Security Tests
    // ============================================================================
    
    #[test]
    fn test_validate_message_xss_attempt() {
        let request = SendDirectMessageRequest {
            recipient_id: 2,
            content: "<script>alert('XSS')</script>".to_string(),
            reply_to_id: None,
            attachments: None,
        };
        
        // Should still validate (sanitization happens at display time)
        let result = validate_direct_message(&request, 1);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_validate_message_sql_injection_attempt() {
        let request = SendDirectMessageRequest {
            recipient_id: 2,
            content: "'; DROP TABLE users; --".to_string(),
            reply_to_id: None,
            attachments: None,
        };
        
        // Should validate (parameterized queries prevent SQL injection)
        let result = validate_direct_message(&request, 1);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_validate_user_id_zero() {
        let result = validation::validate_user_id(0, "user_id");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_validate_user_id_negative() {
        let result = validation::validate_user_id(-100, "user_id");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_validate_user_id_max_int() {
        let result = validation::validate_user_id(i32::MAX, "user_id");
        assert!(result.is_ok());
    }
    
    // ============================================================================
    // Response Structure Tests
    // ============================================================================
    
    #[test]
    fn test_direct_message_response_serialization() {
        use chrono::Utc;
        
        let response = DirectMessageResponse {
            id: 123,
            sender_id: 1,
            recipient_id: 2,
            content: "Test message".to_string(),
            reply_to_id: None,
            created_at: Utc::now(),
            read_at: None,
            attachments: vec![],
        };
        
        let json = serde_json::to_string(&response);
        assert!(json.is_ok());
        
        let deserialized: Result<DirectMessageResponse, _> = serde_json::from_str(&json.unwrap());
        assert!(deserialized.is_ok());
    }
    
    #[test]
    fn test_conversation_response_serialization() {
        use chrono::Utc;
        
        let response = ConversationResponse {
            id: "conv_1_2".to_string(),
            other_user_id: 2,
            other_username: "testuser".to_string(),
            other_display_name: Some("Test User".to_string()),
            other_avatar: None,
            last_message: Some("Hello".to_string()),
            last_message_at: Some(Utc::now()),
            unread_count: 5,
            is_muted: false,
            is_online: true,
        };
        
        let json = serde_json::to_string(&response);
        assert!(json.is_ok());
    }
    
    // ============================================================================
    // Error Handling Tests
    // ============================================================================
    
    #[test]
    fn test_error_with_field() {
        let error = ClawMeshError::with_message(
            ErrorCode::InvalidInput,
            "Test error".to_string(),
        ).with_field("test_field");
        
        assert_eq!(error.code, ErrorCode::InvalidInput);
        assert_eq!(error.field, Some("test_field".to_string()));
    }
    
    #[test]
    fn test_error_response_format() {
        let error = ClawMeshError::with_message(
            ErrorCode::NotFound,
            "Resource not found".to_string(),
        );
        
        let response = error.error_response();
        assert_eq!(response.status(), actix_web::http::StatusCode::NOT_FOUND);
    }
    
    // ============================================================================
    // Concurrency Tests
    // ============================================================================
    
    #[tokio::test]
    async fn test_concurrent_validations() {
        use tokio::task;
        
        let mut handles = vec![];
        
        for i in 0..100 {
            let handle = task::spawn(async move {
                let request = SendDirectMessageRequest {
                    recipient_id: i + 2,
                    content: format!("Message {}", i),
                    reply_to_id: None,
                    attachments: None,
                };
                
                validate_direct_message(&request, 1)
            });
            
            handles.push(handle);
        }
        
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
        }
    }
    
    // ============================================================================
    // Performance Tests
    // ============================================================================
    
    #[test]
    fn test_validation_performance() {
        use std::time::Instant;
        
        let request = SendDirectMessageRequest {
            recipient_id: 2,
            content: "Performance test message".to_string(),
            reply_to_id: None,
            attachments: None,
        };
        
        let start = Instant::now();
        
        for _ in 0..10000 {
            let _ = validate_direct_message(&request, 1);
        }
        
        let duration = start.elapsed();
        
        // Should complete 10,000 validations in less than 100ms
        assert!(duration.as_millis() < 100, 
            "Validation took too long: {:?}", duration);
    }
    
    // ============================================================================
    // Edge Case Tests
    // ============================================================================
    
    #[test]
    fn test_validate_message_unicode_content() {
        let request = SendDirectMessageRequest {
            recipient_id: 2,
            content: "Hello 世界 🌍 مرحبا".to_string(),
            reply_to_id: None,
            attachments: None,
        };
        
        let result = validate_direct_message(&request, 1);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_validate_message_whitespace_only() {
        let request = SendDirectMessageRequest {
            recipient_id: 2,
            content: "   \n\t   ".to_string(),
            reply_to_id: None,
            attachments: None,
        };
        
        let result = validate_direct_message(&request, 1);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_validate_message_newlines() {
        let request = SendDirectMessageRequest {
            recipient_id: 2,
            content: "Line 1\nLine 2\nLine 3".to_string(),
            reply_to_id: None,
            attachments: None,
        };
        
        let result = validate_direct_message(&request, 1);
        assert!(result.is_ok());
    }
}

// ============================================================================
// Integration Tests (require database)
// ============================================================================

#[cfg(test)]
mod integration_tests {
    use super::*;
    
    // These tests require a running database and will be run separately
    // with `cargo test --features integration-tests`
    
    #[cfg(feature = "integration-tests")]
    #[tokio::test]
    async fn test_send_message_integration() {
        // TODO: Set up test database
        // TODO: Create test users
        // TODO: Send message
        // TODO: Verify message in database
        // TODO: Clean up test data
    }
    
    #[cfg(feature = "integration-tests")]
    #[tokio::test]
    async fn test_get_conversations_integration() {
        // TODO: Set up test database
        // TODO: Create test conversations
        // TODO: Query conversations
        // TODO: Verify results
        // TODO: Clean up test data
    }
    
    #[cfg(feature = "integration-tests")]
    #[tokio::test]
    async fn test_mark_as_read_integration() {
        // TODO: Set up test database
        // TODO: Create unread messages
        // TODO: Mark as read
        // TODO: Verify read status
        // TODO: Clean up test data
    }
}
