// Comprehensive tests for API module
#[cfg(test)]
mod api_tests {
    use super::*;

    #[test]
    fn test_api_response_structures() {
        use crate::responses::{CreditResponse, AgentResponse, StatsResponse};
        
        // Test that response structures can be created
        let credit_response = CreditResponse {
            success: true,
            message: "Credit updated".to_string(),
            new_credit: Some(350),
            new_tier: Some("regular".to_string()),
        };
        
        assert!(credit_response.success);
        assert_eq!(credit_response.new_credit, Some(350));
        
        let agent_response = AgentResponse {
            success: true,
            message: "Agent created".to_string(),
            agent_id: Some(123),
        };
        
        assert!(agent_response.success);
        assert_eq!(agent_response.agent_id, Some(123));
    }

    #[test]
    fn test_error_responses() {
        use crate::responses::ErrorResponse;
        
        let error = ErrorResponse {
            success: false,
            error: "Invalid input".to_string(),
            details: Some("Username too short".to_string()),
        };
        
        assert!(!error.success);
        assert_eq!(error.error, "Invalid input");
        assert!(error.details.is_some());
    }

    #[test]
    fn test_request_validation() {
        use crate::agent::InstallAgentRequest;
        
        // Valid request
        let valid_request = InstallAgentRequest {
            username: "test_agent".to_string(),
            metadata: None,
        };
        
        assert_eq!(valid_request.username, "test_agent");
        
        // Request with metadata
        let request_with_meta = InstallAgentRequest {
            username: "ai_bot".to_string(),
            metadata: Some(serde_json::json!({"model": "gpt-4"})),
        };
        
        assert!(request_with_meta.metadata.is_some());
    }

    #[test]
    fn test_credit_update_request() {
        use crate::credit::UpdateCreditRequest;
        
        let request = UpdateCreditRequest {
            person_id: 42,
            action: "post_upvote".to_string(),
            reason: Some("User upvoted a post".to_string()),
        };
        
        assert_eq!(request.person_id, 42);
        assert_eq!(request.action, "post_upvote");
        assert!(request.reason.is_some());
    }

    #[test]
    fn test_batch_operation_request() {
        use crate::credit::BatchUpdateRequest;
        
        let request = BatchUpdateRequest {
            updates: vec![
                (1, "post_upvote".to_string()),
                (2, "comment_created".to_string()),
                (3, "daily_active".to_string()),
            ],
        };
        
        assert_eq!(request.updates.len(), 3);
        assert_eq!(request.updates[0].0, 1);
        assert_eq!(request.updates[1].1, "comment_created");
    }

    #[test]
    fn test_permission_check_request() {
        use crate::permissions::CheckPermissionRequest;
        
        let request = CheckPermissionRequest {
            person_id: 100,
            permission_type: "post".to_string(),
        };
        
        assert_eq!(request.person_id, 100);
        assert_eq!(request.permission_type, "post");
    }

    #[test]
    fn test_stats_response_structure() {
        use crate::stats::GlobalStatsResponse;
        
        let stats = GlobalStatsResponse {
            total_users: 1000,
            average_credit: 425.5,
            median_credit: 400,
            tier_distribution: vec![
                ("novice".to_string(), 200),
                ("regular".to_string(), 500),
                ("active".to_string(), 200),
                ("veteran".to_string(), 80),
                ("expert".to_string(), 20),
            ],
        };
        
        assert_eq!(stats.total_users, 1000);
        assert_eq!(stats.tier_distribution.len(), 5);
        
        // Verify tier distribution adds up
        let total: i64 = stats.tier_distribution.iter().map(|(_, count)| count).sum();
        assert_eq!(total, 1000);
    }

    #[test]
    fn test_agent_list_request() {
        use crate::agent_list::ListAgentsRequest;
        
        let request = ListAgentsRequest {
            active_only: true,
            limit: 50,
            offset: 0,
        };
        
        assert!(request.active_only);
        assert_eq!(request.limit, 50);
        assert_eq!(request.offset, 0);
    }

    #[test]
    fn test_heartbeat_request() {
        use crate::agent::HeartbeatRequest;
        
        let request = HeartbeatRequest {
            person_id: 42,
            interval: Some(3600),
        };
        
        assert_eq!(request.person_id, 42);
        assert_eq!(request.interval, Some(3600));
    }

    #[test]
    fn test_response_serialization() {
        use crate::responses::CreditResponse;
        
        let response = CreditResponse {
            success: true,
            message: "Success".to_string(),
            new_credit: Some(500),
            new_tier: Some("active".to_string()),
        };
        
        // Test that it can be serialized to JSON
        let json = serde_json::to_string(&response);
        assert!(json.is_ok());
        
        // Test that it can be deserialized back
        let json_str = json.unwrap();
        let deserialized: Result<CreditResponse, _> = serde_json::from_str(&json_str);
        assert!(deserialized.is_ok());
        
        let restored = deserialized.unwrap();
        assert_eq!(restored.success, response.success);
        assert_eq!(restored.new_credit, response.new_credit);
    }
}
