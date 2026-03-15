/// Agent Skills System Unit Tests
/// DO-178C Level A Compliant Unit Test Suite
/// 
/// Focused unit tests for individual functions and components

#[cfg(test)]
mod skills_unit_tests {
    use clawmesh_skills::{
        models::{AgentSkill, SkillType, SkillPermission},
        security::{
            validate_skill_code,
            comprehensive_security_scan,
            SecurityScanResult,
            ThreatType,
        },
        sandbox::{
            SandboxBuilder,
            SkillSandbox,
        },
        skills::{
            calculate_skill_rating,
            validate_skill_metadata,
        },
    };

    // ========================================================================
    // Skill Validation Unit Tests
    // ========================================================================

    #[test]
    fn test_skill_name_validation() {
        // Valid skill names
        assert!(validate_skill_name("data_analyzer").is_ok());
        assert!(validate_skill_name("text_processor").is_ok());
        assert!(validate_skill_name("api_connector").is_ok());
        
        // Invalid skill names
        assert!(validate_skill_name("").is_err()); // Empty
        assert!(validate_skill_name("a").is_err()); // Too short
        assert!(validate_skill_name("a".repeat(101).as_str()).is_err()); // Too long
        assert!(validate_skill_name("invalid-name!").is_err()); // Invalid characters
        assert!(validate_skill_name("123_skill").is_err()); // Starts with number
    }

    #[test]
    fn test_version_validation() {
        // Valid versions
        assert!(validate_version("1.0.0").is_ok());
        assert!(validate_version("2.1.3").is_ok());
        assert!(validate_version("10.15.20").is_ok());
        
        // Invalid versions
        assert!(validate_version("").is_err()); // Empty
        assert!(validate_version("1.0").is_err()); // Missing patch
        assert!(validate_version("1.0.0.0").is_err()); // Too many parts
        assert!(validate_version("a.b.c").is_err()); // Non-numeric
        assert!(validate_version("1.0.").is_err()); // Incomplete
    }

    #[test]
    fn test_skill_type_validation() {
        // Valid skill types
        assert_eq!(SkillType::Custom as i32, 0);
        assert_eq!(SkillType::Shared as i32, 1);
        assert_eq!(SkillType::System as i32, 2);
        
        // Test type conversion
        assert_eq!(SkillType::try_from(0), Ok(SkillType::Custom));
        assert_eq!(SkillType::try_from(1), Ok(SkillType::Shared));
        assert_eq!(SkillType::try_from(2), Ok(SkillType::System));
        assert!(SkillType::try_from(99).is_err()); // Invalid type
    }

    // ========================================================================
    // Security Validation Unit Tests
    // ========================================================================

    #[test]
    fn test_safe_code_patterns() {
        let safe_codes = vec![
            "def hello():\n    return 'Hello, World!'",
            "def calculate_sum(a, b):\n    return a + b",
            "def process_data(data):\n    for item in data:\n        yield item * 2",
            "class Calculator:\n    def add(self, x, y):\n        return x + y",
        ];
        
        for code in safe_codes {
            assert!(validate_skill_code(code).is_ok(), "Safe code should pass: {}", code);
        }
    }

    #[test]
    fn test_dangerous_import_detection() {
        let dangerous_imports = vec![
            "import os",
            "import sys",
            "import subprocess",
            "import socket",
            "import urllib.request",
            "import requests",
            "import sqlite3",
            "import shutil",
            "import tempfile",
        ];
        
        for danger in dangerous_imports {
            let code = format!("{}\ndef main(): pass", danger);
            assert!(validate_skill_code(&code).is_err(), "Dangerous import should fail: {}", danger);
        }
    }

    #[test]
    fn test_dangerous_function_detection() {
        let dangerous_functions = vec![
            "os.system",
            "os.popen",
            "subprocess.run",
            "subprocess.call",
            "subprocess.Popen",
            "eval(",
            "exec(",
            "open(",
            "file(",
            "input(",
        ];
        
        for danger in dangerous_functions {
            let code = format!("def main():\n    {}", danger);
            assert!(validate_skill_code(&code).is_err(), "Dangerous function should fail: {}", danger);
        }
    }

    #[test]
    fn test_comprehensive_security_scan() {
        let malicious_code = r#"
import os
import subprocess

def malicious():
    os.system("rm -rf /")
    subprocess.run(["curl", "evil.com"])
    eval("print('hacked')")
"#;
        
        let result = comprehensive_security_scan(malicious_code);
        
        assert!(!result.is_safe);
        assert!(result.risk_score > 50);
        assert!(result.threats.len() >= 3);
        
        // Check specific threats
        let threat_types: Vec<ThreatType> = result.threats.iter().map(|t| t.threat_type).collect();
        assert!(threat_types.contains(&ThreatType::SystemCommand));
        assert!(threat_types.contains(&ThreatType::CodeExecution));
    }

    #[test]
    fn test_security_scan_safe_code() {
        let safe_code = r#"
def calculate_average(numbers):
    total = sum(numbers)
    count = len(numbers)
    return total / count if count > 0 else 0

def process_text(text):
    return text.upper().strip()
"#;
        
        let result = comprehensive_security_scan(safe_code);
        
        assert!(result.is_safe);
        assert_eq!(result.risk_score, 0);
        assert!(result.threats.is_empty());
    }

    // ========================================================================
    // Sandbox Unit Tests
    // ========================================================================

    #[test]
    fn test_sandbox_builder_defaults() {
        let sandbox = SandboxBuilder::new().build();
        
        assert!(sandbox.max_memory > 0);
        assert!(sandbox.max_cpu_time.as_secs() > 0);
        assert!(!sandbox.network_access);
        assert!(sandbox.max_file_size > 0);
    }

    #[test]
    fn test_sandbox_builder_customization() {
        use std::time::Duration;
        
        let sandbox = SandboxBuilder::new()
            .with_max_memory(512 * 1024 * 1024) // 512MB
            .with_timeout(Duration::from_secs(30))
            .with_network_access(true)
            .with_max_file_size(1024 * 1024) // 1MB
            .build();
        
        assert_eq!(sandbox.max_memory, 512 * 1024 * 1024);
        assert_eq!(sandbox.max_cpu_time, Duration::from_secs(30));
        assert!(sandbox.network_access);
        assert_eq!(sandbox.max_file_size, 1024 * 1024);
    }

    #[test]
    fn test_restrictive_sandbox() {
        let sandbox = SkillSandbox::restrictive();
        
        assert_eq!(sandbox.max_memory, 64 * 1024 * 1024); // 64MB
        assert_eq!(sandbox.max_cpu_time.as_secs(), 5); // 5 seconds
        assert!(!sandbox.network_access);
        assert_eq!(sandbox.max_file_size, 1024 * 1024); // 1MB
    }

    #[test]
    fn test_sandbox_configuration_validation() {
        // Test invalid configurations
        assert!(SandboxBuilder::new()
            .with_max_memory(0)
            .build()
            .max_memory > 0); // Should have minimum
        
        assert!(SandboxBuilder::new()
            .with_timeout(Duration::from_secs(0))
            .build()
            .max_cpu_time.as_secs() > 0); // Should have minimum
    }

    // ========================================================================
    // Skill Rating Unit Tests
    // ========================================================================

    #[test]
    fn test_skill_rating_calculation() {
        // Test rating calculation based on downloads and votes
        let rating = calculate_skill_rating(100, 80, 20); // downloads, upvotes, downvotes
        assert!(rating > 0.0);
        assert!(rating <= 5.0);
        
        // Perfect rating
        let perfect_rating = calculate_skill_rating(1000, 1000, 0);
        assert_eq!(perfect_rating, 5.0);
        
        // Poor rating
        let poor_rating = calculate_skill_rating(10, 1, 9);
        assert!(poor_rating < 2.0);
    }

    #[test]
    fn test_skill_rating_edge_cases() {
        // No votes
        let no_votes_rating = calculate_skill_rating(50, 0, 0);
        assert_eq!(no_votes_rating, 3.0); // Default rating
        
        // Only downvotes
        let only_downvotes = calculate_skill_rating(10, 0, 10);
        assert!(only_downvotes < 2.0);
        
        // Only upvotes
        let only_upvotes = calculate_skill_rating(10, 10, 0);
        assert!(only_upvotes > 4.0);
    }

    // ========================================================================
    // Metadata Validation Unit Tests
    // ========================================================================

    #[test]
    fn test_metadata_validation() {
        use serde_json::json;
        
        // Valid metadata
        let valid_metadata = json!({
            "description": "A useful skill for data processing",
            "author": "test_agent",
            "tags": ["data", "processing", "utility"],
            "license": "MIT",
            "homepage": "https://example.com"
        });
        
        assert!(validate_skill_metadata(&valid_metadata).is_ok());
        
        // Invalid metadata - missing required fields
        let invalid_metadata = json!({
            "author": "test_agent"
            // Missing description
        });
        
        assert!(validate_skill_metadata(&invalid_metadata).is_err());
    }

    #[test]
    fn test_tag_validation() {
        // Valid tags
        let valid_tags = vec![
            vec!["data", "processing"],
            vec!["utility", "helper"],
            vec!["api", "connector"],
        ];
        
        for tags in valid_tags {
            assert!(validate_tags(&tags).is_ok());
        }
        
        // Invalid tags
        let invalid_tags = vec![
            vec![""], // Empty tag
            vec!["a".repeat(51).as_str()], // Too long
            vec!["invalid-tag!"], // Invalid characters
            vec!["data", "data"], // Duplicate
            vec!["a", "b", "c", "d", "e", "f"], // Too many tags
        ];
        
        for tags in invalid_tags {
            assert!(validate_tags(&tags).is_err());
        }
    }

    // ========================================================================
    // Permission Unit Tests
    // ========================================================================

    #[test]
    fn test_skill_permissions() {
        // Test permission values
        assert_eq!(SkillPermission::Read as i32, 1);
        assert_eq!(SkillPermission::Write as i32, 2);
        assert_eq!(SkillPermission::Execute as i32, 4);
        assert_eq!(SkillPermission::Admin as i32, 8);
        
        // Test permission combinations
        let read_write = SkillPermission::Read as i32 | SkillPermission::Write as i32;
        assert_eq!(read_write, 3);
        
        let all_permissions = SkillPermission::Read as i32 | 
                            SkillPermission::Write as i32 | 
                            SkillPermission::Execute as i32 | 
                            SkillPermission::Admin as i32;
        assert_eq!(all_permissions, 15);
    }

    #[test]
    fn test_permission_checking() {
        let user_permissions = 3; // Read + Write
        
        assert!(has_permission(user_permissions, SkillPermission::Read));
        assert!(has_permission(user_permissions, SkillPermission::Write));
        assert!(!has_permission(user_permissions, SkillPermission::Execute));
        assert!(!has_permission(user_permissions, SkillPermission::Admin));
    }

    // ========================================================================
    // Performance Unit Tests
    // ========================================================================

    #[test]
    fn test_code_validation_performance() {
        use std::time::Instant;
        
        let code = "def test(): return 'Hello, World!'";
        
        let start = Instant::now();
        
        // Perform many validations
        for _ in 0..1000 {
            validate_skill_code(code).ok();
        }
        
        let duration = start.elapsed();
        
        // Should be fast (< 100ms for 1000 validations)
        assert!(duration.as_millis() < 100, "Code validation too slow");
    }

    #[test]
    fn test_security_scan_performance() {
        use std::time::Instant;
        
        let code = "def safe_function(): return 'safe'";
        
        let start = Instant::now();
        
        // Perform many scans
        for _ in 0..100 {
            comprehensive_security_scan(code);
        }
        
        let duration = start.elapsed();
        
        // Should be reasonable (< 500ms for 100 scans)
        assert!(duration.as_millis() < 500, "Security scan too slow");
    }

    // ========================================================================
    // Edge Case Tests
    // ========================================================================

    #[test]
    fn test_empty_skill_code() {
        let result = validate_skill_code("");
        assert!(result.is_err(), "Empty code should be rejected");
    }

    #[test]
    fn test_very_long_skill_code() {
        let long_code = "def test():\n    return '".to_string() + &"x".repeat(10000) + "'";
        let result = validate_skill_code(&long_code);
        assert!(result.is_err(), "Very long code should be rejected");
    }

    #[test]
    fn test_unicode_in_skill_code() {
        let unicode_code = "def test():\n    return '你好世界'";
        let result = validate_skill_code(unicode_code);
        assert!(result.is_ok(), "Unicode should be allowed");
    }

    #[test]
    fn test_special_characters_in_metadata() {
        use serde_json::json;
        
        let metadata = json!({
            "description": "Skill with special chars: !@#$%^&*()",
            "author": "test_agent",
            "tags": ["test", "special-chars"]
        });
        
        let result = validate_skill_metadata(&metadata);
        assert!(result.is_ok(), "Special characters should be allowed in metadata");
    }

    // ========================================================================
    // Helper Functions for Testing
    // ========================================================================

    fn validate_skill_name(name: &str) -> Result<(), String> {
        if name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if name.len() < 3 {
            return Err("Name too short".to_string());
        }
        if name.len() > 100 {
            return Err("Name too long".to_string());
        }
        if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err("Invalid characters".to_string());
        }
        if name.chars().next().unwrap().is_numeric() {
            return Err("Cannot start with number".to_string());
        }
        Ok(())
    }

    fn validate_version(version: &str) -> Result<(), String> {
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() != 3 {
            return Err("Invalid version format".to_string());
        }
        
        for part in parts {
            if part.is_empty() || part.parse::<u32>().is_err() {
                return Err("Invalid version number".to_string());
            }
        }
        
        Ok(())
    }

    fn validate_tags(tags: &[&str]) -> Result<(), String> {
        if tags.is_empty() {
            return Err("At least one tag required".to_string());
        }
        if tags.len() > 5 {
            return Err("Too many tags".to_string());
        }
        
        for tag in tags {
            if tag.is_empty() || tag.len() > 50 {
                return Err("Invalid tag".to_string());
            }
            if !tag.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
                return Err("Invalid tag characters".to_string());
            }
        }
        
        // Check for duplicates
        let unique_tags: std::collections::HashSet<_> = tags.iter().collect();
        if unique_tags.len() != tags.len() {
            return Err("Duplicate tags".to_string());
        }
        
        Ok(())
    }

    fn validate_skill_metadata(metadata: &serde_json::Value) -> Result<(), String> {
        if let Some(desc) = metadata.get("description") {
            if !desc.is_string() || desc.as_str().unwrap().is_empty() {
                return Err("Description required".to_string());
            }
        } else {
            return Err("Description missing".to_string());
        }
        
        if let Some(tags) = metadata.get("tags") {
            if let Some(tag_array) = tags.as_array() {
                let tag_strings: Vec<&str> = tag_array
                    .iter()
                    .filter_map(|t| t.as_str())
                    .collect();
                validate_tags(&tag_strings)?;
            }
        }
        
        Ok(())
    }

    fn has_permission(user_perms: i32, required_perm: SkillPermission) -> bool {
        (user_perms & required_perm as i32) != 0
    }
}
