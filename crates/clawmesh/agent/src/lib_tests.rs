// Additional comprehensive tests for agent module
#[cfg(test)]
mod agent_integration_tests {
    use super::*;

    #[test]
    fn test_username_validation_valid() {
        use crate::validation::validate_username;
        
        // Valid usernames
        assert!(validate_username("agent_bot").is_ok());
        assert!(validate_username("my_agent_123").is_ok());
        assert!(validate_username("AgentBot").is_ok());
        assert!(validate_username("agent123").is_ok());
    }

    #[test]
    fn test_username_validation_invalid() {
        use crate::validation::validate_username;
        
        // Too short
        assert!(validate_username("ab").is_err());
        
        // Too long
        assert!(validate_username("this_is_a_very_long_username_that_exceeds_limit").is_err());
        
        // Invalid characters
        assert!(validate_username("agent-bot").is_err());
        assert!(validate_username("agent.bot").is_err());
        assert!(validate_username("agent bot").is_err());
        assert!(validate_username("agent@bot").is_err());
    }

    #[test]
    fn test_metadata_validation_valid() {
        use crate::validation::validate_metadata;
        use serde_json::json;
        
        // Valid metadata
        let meta1 = json!({
            "model": "gpt-4",
            "version": "1.0.0",
            "capabilities": ["chat", "analysis"]
        });
        assert!(validate_metadata(&Some(meta1)).is_ok());
        
        // Empty metadata is valid
        assert!(validate_metadata(&None).is_ok());
        
        // Minimal metadata
        let meta2 = json!({});
        assert!(validate_metadata(&Some(meta2)).is_ok());
    }

    #[test]
    fn test_metadata_validation_invalid() {
        use crate::validation::validate_metadata;
        use serde_json::json;
        
        // Not an object
        let meta1 = json!("string");
        assert!(validate_metadata(&Some(meta1)).is_err());
        
        let meta2 = json!([1, 2, 3]);
        assert!(validate_metadata(&Some(meta2)).is_err());
        
        // Invalid field types
        let meta3 = json!({
            "model": 123
        });
        assert!(validate_metadata(&Some(meta3)).is_err());
        
        let meta4 = json!({
            "capabilities": "not_an_array"
        });
        assert!(validate_metadata(&Some(meta4)).is_err());
    }

    #[test]
    fn test_metadata_size_limit() {
        use crate::validation::validate_metadata;
        use serde_json::json;
        
        // Create large metadata (over 10KB)
        let large_string = "x".repeat(11000);
        let meta = json!({
            "data": large_string
        });
        
        assert!(validate_metadata(&Some(meta)).is_err());
    }

    #[test]
    fn test_heartbeat_interval_validation() {
        use crate::validation::validate_heartbeat_interval;
        
        // Valid intervals
        assert!(validate_heartbeat_interval(300).is_ok());    // 5 minutes
        assert!(validate_heartbeat_interval(3600).is_ok());   // 1 hour
        assert!(validate_heartbeat_interval(14400).is_ok());  // 4 hours
        assert!(validate_heartbeat_interval(86400).is_ok());  // 24 hours
        
        // Invalid intervals
        assert!(validate_heartbeat_interval(100).is_err());   // Too short
        assert!(validate_heartbeat_interval(100000).is_err()); // Too long
    }

    #[test]
    fn test_agent_username_format() {
        use crate::validation::validate_username;
        
        // Test various agent username formats
        let valid_names = vec![
            "bot_assistant",
            "ai_helper",
            "agent_001",
            "AutoMod",
            "content_analyzer",
        ];
        
        for name in valid_names {
            assert!(validate_username(name).is_ok(), "Failed for: {}", name);
        }
    }

    #[test]
    fn test_heartbeat_interval_ranges() {
        use crate::validation::validate_heartbeat_interval;
        
        // Test boundary values
        assert!(validate_heartbeat_interval(299).is_err());   // Just below min
        assert!(validate_heartbeat_interval(300).is_ok());    // Exactly min
        assert!(validate_heartbeat_interval(86400).is_ok());  // Exactly max
        assert!(validate_heartbeat_interval(86401).is_err()); // Just above max
    }

    #[test]
    fn test_metadata_field_validation() {
        use crate::validation::validate_metadata;
        use serde_json::json;
        
        // Test specific field validations
        let meta_with_model = json!({
            "model": "claude-3",
            "version": "2.0"
        });
        assert!(validate_metadata(&Some(meta_with_model)).is_ok());
        
        let meta_with_capabilities = json!({
            "capabilities": ["moderation", "translation", "summarization"]
        });
        assert!(validate_metadata(&Some(meta_with_capabilities)).is_ok());
        
        // Test nested structures
        let meta_nested = json!({
            "model": "gpt-4",
            "config": {
                "temperature": 0.7,
                "max_tokens": 1000
            }
        });
        assert!(validate_metadata(&Some(meta_nested)).is_ok());
    }
}
