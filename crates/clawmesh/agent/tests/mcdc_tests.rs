/// MC/DC (Modified Condition/Decision Coverage) Tests for Agent Module
/// DO-178C Level A Requirement

use clawmesh_agent::validation::{validate_heartbeat_interval, validate_metadata, validate_username};
use serde_json::json;

#[cfg(test)]
mod mcdc_agent_tests {
    use super::*;

    // ========================================================================
    // MC/DC Tests for validate_username
    // ========================================================================
    
    #[test]
    fn mcdc_username_empty() {
        let result = validate_username("");
        assert!(result.is_err());
    }
    
    #[test]
    fn mcdc_username_too_short() {
        let result = validate_username("ab");
        assert!(result.is_err());
    }
    
    #[test]
    fn mcdc_username_too_long() {
        let long_name = "a".repeat(51);
        let result = validate_username(&long_name);
        assert!(result.is_err());
    }
    
    #[test]
    fn mcdc_username_invalid_chars() {
        let result = validate_username("invalid user");
        assert!(result.is_err());
    }
    
    #[test]
    fn mcdc_username_invalid_start() {
        let result = validate_username("_invalid");
        assert!(result.is_err());
    }
    
    #[test]
    fn mcdc_username_valid() {
        let result = validate_username("valid_bot_123");
        assert!(result.is_ok());
    }
    
    #[test]
    fn mcdc_username_at_min_length() {
        let result = validate_username("abc");
        assert!(result.is_ok());
    }
    
    #[test]
    fn mcdc_username_at_max_length() {
        let max_name = "a".repeat(50);
        let result = validate_username(&max_name);
        assert!(result.is_ok());
    }
    
    // ========================================================================
    // MC/DC Tests for validate_heartbeat_interval
    // ========================================================================
    
    #[test]
    fn mcdc_heartbeat_interval_too_short() {
        let result = validate_heartbeat_interval(100);
        assert!(result.is_err());
    }
    
    #[test]
    fn mcdc_heartbeat_interval_too_long() {
        let result = validate_heartbeat_interval(100000);
        assert!(result.is_err());
    }
    
    #[test]
    fn mcdc_heartbeat_interval_valid() {
        let result = validate_heartbeat_interval(3600);
        assert!(result.is_ok());
    }
    
    #[test]
    fn mcdc_heartbeat_interval_at_min() {
        let result = validate_heartbeat_interval(300);
        assert!(result.is_ok());
    }
    
    #[test]
    fn mcdc_heartbeat_interval_at_max() {
        let result = validate_heartbeat_interval(86400);
        assert!(result.is_ok());
    }
    
    #[test]
    fn mcdc_heartbeat_interval_just_under_min() {
        let result = validate_heartbeat_interval(299);
        assert!(result.is_err());
    }
    
    #[test]
    fn mcdc_heartbeat_interval_just_over_max() {
        let result = validate_heartbeat_interval(86401);
        assert!(result.is_err());
    }
    
    // ========================================================================
    // MC/DC Tests for validate_metadata
    // ========================================================================
    
    #[test]
    fn mcdc_metadata_none() {
        let result = validate_metadata(&None);
        assert!(result.is_ok());
    }
    
    #[test]
    fn mcdc_metadata_not_object() {
        let invalid = Some(json!("not an object"));
        let result = validate_metadata(&invalid);
        assert!(result.is_err());
    }
    
    #[test]
    fn mcdc_metadata_too_large() {
        let large_data = "x".repeat(11000);
        let large_meta = Some(json!({ "data": large_data }));
        let result = validate_metadata(&large_meta);
        assert!(result.is_err());
    }
    
    #[test]
    fn mcdc_metadata_invalid_model_type() {
        let invalid = Some(json!({ "model": 123 }));
        let result = validate_metadata(&invalid);
        assert!(result.is_err());
    }
    
    #[test]
    fn mcdc_metadata_invalid_version_type() {
        let invalid = Some(json!({ "version": 123 }));
        let result = validate_metadata(&invalid);
        assert!(result.is_err());
    }
    
    #[test]
    fn mcdc_metadata_invalid_capabilities_type() {
        let invalid = Some(json!({ "capabilities": "not an array" }));
        let result = validate_metadata(&invalid);
        assert!(result.is_err());
    }
    
    #[test]
    fn mcdc_metadata_valid() {
        let valid = Some(json!({
            "model": "gpt-4",
            "version": "1.0",
            "capabilities": ["chat", "analysis"]
        }));
        let result = validate_metadata(&valid);
        assert!(result.is_ok());
    }
    
    #[test]
    fn mcdc_metadata_minimal_valid() {
        let minimal = Some(json!({}));
        let result = validate_metadata(&minimal);
        assert!(result.is_ok());
    }
    
    // ========================================================================
    // MC/DC Coverage Verification
    // ========================================================================
    
    #[test]
    fn mcdc_coverage_summary() {
        // Username validation
        assert!(validate_username("valid_bot").is_ok());
        assert!(validate_username("").is_err());
        assert!(validate_username("ab").is_err());
        assert!(validate_username(&"a".repeat(51)).is_err());
        
        // Heartbeat interval validation
        assert!(validate_heartbeat_interval(3600).is_ok());
        assert!(validate_heartbeat_interval(100).is_err());
        assert!(validate_heartbeat_interval(100000).is_err());
        
        // Metadata validation
        assert!(validate_metadata(&None).is_ok());
        assert!(validate_metadata(&Some(json!({"model": "gpt-4"}))).is_ok());
        assert!(validate_metadata(&Some(json!("invalid"))).is_err());
    }
}
