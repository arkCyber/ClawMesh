/// Agent Skills Data Models (DO-178C Level A)
/// 
/// Defines data structures for skill management

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use lemmy_db_schema_file::PersonId;
use serde::{Deserialize, Serialize};

/// Skill types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillType {
    Builtin,    // System-provided skills
    Custom,     // User-created skills
    Shared,     // Skills from marketplace
    External,   // Third-party integrations
}

impl SkillType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SkillType::Builtin => "builtin",
            SkillType::Custom => "custom",
            SkillType::Shared => "shared",
            SkillType::External => "external",
        }
    }
}

/// Agent skill record
#[derive(Debug, Clone, Queryable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = agent_skills)]
pub struct AgentSkill {
    pub id: i32,
    pub agent_id: PersonId,
    pub skill_name: String,
    pub skill_type: SkillType,
    pub skill_code: Option<String>,
    pub skill_metadata: Option<serde_json::Value>,
    pub version: String,
    pub is_public: bool,
    pub is_verified: bool,
    pub downloads: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl AgentSkill {
    /// Check if skill is safe to execute
    pub fn is_safe(&self) -> bool {
        self.is_verified || self.skill_type == SkillType::Builtin
    }
    
    /// Get skill identifier
    pub fn identifier(&self) -> String {
        format!("{}@{}", self.skill_name, self.version)
    }
}

/// Form for creating/updating agent skill
#[derive(Debug, Clone, Insertable, AsChangeset)]
#[diesel(table_name = agent_skills)]
pub struct AgentSkillForm {
    pub agent_id: PersonId,
    pub skill_name: String,
    pub skill_type: SkillType,
    pub skill_code: Option<String>,
    pub skill_metadata: Option<serde_json::Value>,
    pub version: String,
    pub is_public: bool,
}

/// Skill metadata structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMetadata {
    pub description: String,
    pub author: String,
    pub category: String,
    pub tags: Vec<String>,
    pub dependencies: Vec<String>,
    pub min_reputation: Option<i32>,
    pub permissions: SkillPermissions,
}

/// Skill permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillPermissions {
    pub network_access: bool,
    pub file_read: Vec<String>,
    pub file_write: Vec<String>,
    pub database_access: bool,
    pub api_access: Vec<String>,
    pub max_memory_mb: u64,
    pub max_cpu_seconds: u64,
}

impl Default for SkillPermissions {
    fn default() -> Self {
        Self {
            network_access: false,
            file_read: vec![],
            file_write: vec![],
            database_access: false,
            api_access: vec![],
            max_memory_mb: 128,
            max_cpu_seconds: 10,
        }
    }
}

impl SkillPermissions {
    /// Create restrictive permissions (minimal access)
    pub fn restrictive() -> Self {
        Self {
            network_access: false,
            file_read: vec![],
            file_write: vec![],
            database_access: false,
            api_access: vec![],
            max_memory_mb: 64,
            max_cpu_seconds: 5,
        }
    }
    
    /// Create permissive permissions (for trusted skills)
    pub fn permissive() -> Self {
        Self {
            network_access: true,
            file_read: vec!["./data/*".to_string()],
            file_write: vec!["./data/*".to_string()],
            database_access: true,
            api_access: vec!["*".to_string()],
            max_memory_mb: 512,
            max_cpu_seconds: 60,
        }
    }
    
    /// Validate permissions are within safe limits
    pub fn validate(&self) -> Result<(), String> {
        if self.max_memory_mb > 1024 {
            return Err("Memory limit too high (max 1GB)".to_string());
        }
        if self.max_cpu_seconds > 300 {
            return Err("CPU time limit too high (max 5 minutes)".to_string());
        }
        Ok(())
    }
}

/// Skill installation record
#[derive(Debug, Clone, Queryable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = agent_skill_installations)]
pub struct AgentSkillInstallation {
    pub id: i32,
    pub agent_id: PersonId,
    pub skill_id: i32,
    pub installed_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
    pub usage_count: i32,
}

/// Form for creating skill installation
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = agent_skill_installations)]
pub struct AgentSkillInstallationForm {
    pub agent_id: PersonId,
    pub skill_id: i32,
}

/// Skill execution log
#[derive(Debug, Clone, Queryable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = agent_skill_logs)]
pub struct AgentSkillLog {
    pub id: i32,
    pub agent_id: PersonId,
    pub skill_id: i32,
    pub execution_time_ms: i32,
    pub success: bool,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Form for creating skill log
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = agent_skill_logs)]
pub struct AgentSkillLogForm {
    pub agent_id: PersonId,
    pub skill_id: i32,
    pub execution_time_ms: i32,
    pub success: bool,
    pub error_message: Option<String>,
}

// Use schema from lemmy_db_schema_file
use lemmy_db_schema_file::schema::{agent_skills, agent_skill_installations, agent_skill_logs};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_type() {
        assert_eq!(SkillType::Builtin.as_str(), "builtin");
        assert_eq!(SkillType::Custom.as_str(), "custom");
    }

    #[test]
    fn test_permissions_validation() {
        let valid = SkillPermissions::default();
        assert!(valid.validate().is_ok());
        
        let invalid = SkillPermissions {
            max_memory_mb: 2048,
            ..Default::default()
        };
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_restrictive_permissions() {
        let perms = SkillPermissions::restrictive();
        assert!(!perms.network_access);
        assert!(!perms.database_access);
        assert_eq!(perms.max_memory_mb, 64);
    }
}
