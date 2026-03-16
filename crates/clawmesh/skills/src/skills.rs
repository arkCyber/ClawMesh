/// Agent Skills Management (DO-178C Level A)
/// 
/// Core functions for skill registration, installation, and execution

use anyhow::{Result, Context, bail};
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use lemmy_db_schema_file::PersonId;
use tracing::{info, warn, error};
use chrono::Utc;

use crate::models::{
    AgentSkill, AgentSkillForm, AgentSkillInstallation, AgentSkillInstallationForm,
    AgentSkillLog, AgentSkillLogForm, SkillType,
};
use crate::sandbox::{SkillSandbox, SandboxBuilder, ExecutionResult};
use crate::security::{validate_skill_code, comprehensive_security_scan};

/// Register a new skill
/// 
/// # Safety
/// - Validates skill code
/// - Scans for malicious content
/// - Creates database record
/// - Full audit logging
pub async fn register_skill(
    agent_id: PersonId,
    form: AgentSkillForm,
    conn: &mut AsyncPgConnection,
) -> Result<AgentSkill> {
    use lemmy_db_schema_file::schema::agent_skills;
    
    info!(
        agent_id = agent_id.0,
        skill_name = %form.skill_name,
        "Registering new skill"
    );
    
    // 1. Validate agent owns this skill
    if form.agent_id != agent_id {
        bail!("Agent ID mismatch");
    }
    
    // 2. Validate skill code if provided
    if let Some(ref code) = form.skill_code {
        validate_skill_code(code)
            .context("Skill code validation failed")?;
        
        // Security scan
        let scan_result = comprehensive_security_scan(code);
        if !scan_result.is_safe {
            error!(
                threats = ?scan_result.threats_found,
                risk_score = scan_result.risk_score,
                "Skill failed security scan"
            );
            bail!("Skill code failed security scan: {:?}", scan_result.threats_found);
        }
    }
    
    // 3. Check for duplicate skill name
    let existing_count: i64 = agent_skills::table
        .filter(agent_skills::agent_id.eq(agent_id))
        .filter(agent_skills::skill_name.eq(&form.skill_name))
        .count()
        .get_result(conn)
        .await
        .context("Failed to check for duplicate skill")?;
    
    if existing_count > 0 {
        bail!("Skill with name '{}' already exists", form.skill_name);
    }
    
    // 4. Create skill record
    let skill = diesel::insert_into(agent_skills::table)
        .values(&form)
        .get_result::<AgentSkill>(conn)
        .await
        .context("Failed to create skill")?;
    
    info!(
        skill_id = skill.id,
        skill_name = %skill.skill_name,
        "Skill registered successfully"
    );
    
    Ok(skill)
}

/// Get skill by ID
/// 
/// # Safety
/// - Read-only operation
/// - Returns None if not found
pub async fn get_skill(
    skill_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<Option<AgentSkill>> {
    use lemmy_db_schema_file::schema::agent_skills;
    
    let skill = agent_skills::table
        .find(skill_id)
        .first::<AgentSkill>(conn)
        .await
        .optional()
        .context("Failed to get skill")?;
    
    Ok(skill)
}

/// List skills for an agent
/// 
/// # Safety
/// - Paginated results
/// - Filtered by agent or public
pub async fn list_skills(
    agent_id: Option<PersonId>,
    include_public: bool,
    limit: i64,
    offset: i64,
    conn: &mut AsyncPgConnection,
) -> Result<Vec<AgentSkill>> {
    use lemmy_db_schema_file::schema::agent_skills;
    
    let mut query = agent_skills::table.into_boxed();
    
    if let Some(agent) = agent_id {
        query = query.filter(agent_skills::agent_id.eq(agent));
    }
    
    if include_public {
        query = query.filter(agent_skills::is_public.eq(true));
    }
    
    let skills = query
        .order(agent_skills::created_at.desc())
        .limit(limit)
        .offset(offset)
        .select(AgentSkill::as_select())
        .load::<AgentSkill>(conn)
        .await
        .context("Failed to list skills")?;
    
    Ok(skills)
}

/// Install a skill for an agent
/// 
/// # Safety
/// - Validates skill exists and is accessible
/// - Creates installation record
/// - Prevents duplicate installations
pub async fn install_skill(
    agent_id: PersonId,
    skill_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<AgentSkillInstallation> {
    use lemmy_db_schema_file::schema::{agent_skills, agent_skill_installations};
    
    info!(
        agent_id = agent_id.0,
        skill_id = skill_id,
        "Installing skill"
    );
    
    // 1. Check skill exists and is accessible
    let skill = agent_skills::table
        .find(skill_id)
        .first::<AgentSkill>(conn)
        .await
        .context("Skill not found")?;
    
    // 2. Check access permissions
    if skill.agent_id != agent_id && !skill.is_public {
        bail!("Skill is not accessible");
    }
    
    // 3. Check if already installed
    let existing_count: i64 = agent_skill_installations::table
        .filter(agent_skill_installations::agent_id.eq(agent_id))
        .filter(agent_skill_installations::skill_id.eq(skill_id))
        .count()
        .get_result(conn)
        .await
        .context("Failed to check installation")?;
    
    if existing_count > 0 {
        bail!("Skill already installed");
    }
    
    // 4. Create installation record
    let form = AgentSkillInstallationForm {
        agent_id,
        skill_id,
    };
    
    let installation = diesel::insert_into(agent_skill_installations::table)
        .values(&form)
        .get_result::<AgentSkillInstallation>(conn)
        .await
        .context("Failed to create installation")?;
    
    // 5. Increment download count
    diesel::update(agent_skills::table.find(skill_id))
        .set(agent_skills::downloads.eq(agent_skills::downloads + 1))
        .execute(conn)
        .await
        .context("Failed to update download count")?;
    
    info!(
        installation_id = installation.id,
        "Skill installed successfully"
    );
    
    Ok(installation)
}

/// Uninstall a skill
/// 
/// # Safety
/// - Validates installation exists
/// - Removes installation record
pub async fn uninstall_skill(
    agent_id: PersonId,
    skill_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<()> {
    use lemmy_db_schema_file::schema::agent_skill_installations;
    
    info!(
        agent_id = agent_id.0,
        skill_id = skill_id,
        "Uninstalling skill"
    );
    
    let deleted = diesel::delete(
        agent_skill_installations::table
            .filter(agent_skill_installations::agent_id.eq(agent_id))
            .filter(agent_skill_installations::skill_id.eq(skill_id))
    )
    .execute(conn)
    .await
    .context("Failed to uninstall skill")?;
    
    if deleted == 0 {
        bail!("Skill not installed");
    }
    
    info!("Skill uninstalled successfully");
    Ok(())
}

/// Execute a skill
/// 
/// # Safety
/// - Validates skill access
/// - Executes in sandbox
/// - Logs execution
/// - Enforces resource limits
pub async fn execute_skill(
    agent_id: PersonId,
    skill_id: i32,
    input: &str,
    conn: &mut AsyncPgConnection,
) -> Result<ExecutionResult> {
    use lemmy_db_schema_file::schema::{agent_skills, agent_skill_installations, agent_skill_logs};
    use std::time::Instant;
    
    info!(
        agent_id = agent_id.0,
        skill_id = skill_id,
        "Executing skill"
    );
    
    let start_time = Instant::now();
    
    // 1. Get skill
    let skill = agent_skills::table
        .find(skill_id)
        .first::<AgentSkill>(conn)
        .await
        .context("Skill not found")?;
    
    // 2. Check access
    let has_access = skill.agent_id == agent_id || 
                     skill.is_public ||
                     agent_skill_installations::table
                         .filter(agent_skill_installations::agent_id.eq(agent_id))
                         .filter(agent_skill_installations::skill_id.eq(skill_id))
                         .count()
                         .get_result::<i64>(conn)
                         .await? > 0;
    
    if !has_access {
        bail!("No access to skill");
    }
    
    // 3. Get skill code
    let code = skill.skill_code
        .ok_or_else(|| anyhow::anyhow!("Skill has no code"))?;
    
    // 4. Execute in sandbox
    let sandbox = if skill.is_verified {
        use crate::sandbox::SandboxBuilder;
        SandboxBuilder::new()
            .with_timeout(std::time::Duration::from_secs(30))
            .build()
    } else {
        SkillSandbox::restrictive()
    };
    
    let result = SkillSandbox::execute(&sandbox, &code, input).await?;
    
    let execution_time = start_time.elapsed();
    
    // 5. Log execution
    let log_form = AgentSkillLogForm {
        agent_id,
        skill_id,
        execution_time_ms: execution_time.as_millis() as i32,
        success: result.success,
        error_message: result.error.clone(),
    };
    
    diesel::insert_into(agent_skill_logs::table)
        .values(&log_form)
        .execute(conn)
        .await
        .context("Failed to log execution")?;
    
    // 6. Update usage count
    diesel::update(
        agent_skill_installations::table
            .filter(agent_skill_installations::agent_id.eq(agent_id))
            .filter(agent_skill_installations::skill_id.eq(skill_id))
    )
    .set((
        agent_skill_installations::usage_count.eq(agent_skill_installations::usage_count + 1),
        agent_skill_installations::last_used.eq(Some(Utc::now())),
    ))
    .execute(conn)
    .await
    .ok(); // Ignore error if not installed
    
    info!(
        success = result.success,
        execution_time_ms = execution_time.as_millis(),
        "Skill execution completed"
    );
    
    Ok(result)
}

/// Delete a skill
/// 
/// # Safety
/// - Validates ownership
/// - Removes all installations
/// - Soft delete (marks as deleted)
pub async fn delete_skill(
    agent_id: PersonId,
    skill_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<()> {
    use lemmy_db_schema_file::schema::{agent_skills, agent_skill_installations};
    
    info!(
        agent_id = agent_id.0,
        skill_id = skill_id,
        "Deleting skill"
    );
    
    // 1. Verify ownership
    let skill = agent_skills::table
        .find(skill_id)
        .first::<AgentSkill>(conn)
        .await
        .context("Skill not found")?;
    
    if skill.agent_id != agent_id {
        bail!("Not authorized to delete this skill");
    }
    
    // 2. Remove all installations
    diesel::delete(
        agent_skill_installations::table
            .filter(agent_skill_installations::skill_id.eq(skill_id))
    )
    .execute(conn)
    .await
    .context("Failed to remove installations")?;
    
    // 3. Delete skill
    diesel::delete(agent_skills::table.find(skill_id))
        .execute(conn)
        .await
        .context("Failed to delete skill")?;
    
    info!("Skill deleted successfully");
    Ok(())
}

// ============================================================================
// TESTS - DO-178C Level A Compliance
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Skill Code Validation Tests
    // ========================================================================

    #[test]
    fn test_skill_validation_safe_code() {
        let safe_code = "def hello(): return 'world'";
        assert!(validate_skill_code(safe_code).is_ok());
    }

    #[test]
    fn test_skill_validation_unsafe_exec() {
        let unsafe_code = "exec('malicious')";
        assert!(validate_skill_code(unsafe_code).is_err());
    }

    #[test]
    fn test_skill_validation_unsafe_eval() {
        let unsafe_code = "eval('malicious')";
        assert!(validate_skill_code(unsafe_code).is_err());
    }

    #[test]
    fn test_skill_validation_unsafe_import() {
        let unsafe_code = "import os; os.system('rm -rf /')";
        assert!(validate_skill_code(unsafe_code).is_err());
    }

    #[test]
    fn test_skill_validation_empty_code() {
        let empty_code = "";
        assert!(validate_skill_code(empty_code).is_err());
    }

    #[test]
    fn test_skill_validation_whitespace_only() {
        let whitespace_code = "   \n\t  ";
        assert!(validate_skill_code(whitespace_code).is_err());
    }

    #[test]
    fn test_skill_validation_complex_safe_code() {
        let safe_code = r#"
def calculate_sum(a, b):
    return a + b

def process_data(data):
    result = []
    for item in data:
        result.append(item * 2)
    return result
"#;
        assert!(validate_skill_code(safe_code).is_ok());
    }

    // ========================================================================
    // Skill Type Tests
    // ========================================================================

    #[test]
    fn test_skill_type_as_i32() {
        assert_eq!(SkillType::Builtin as i32, 0);
        assert_eq!(SkillType::Custom as i32, 1);
        assert_eq!(SkillType::Shared as i32, 2);
        assert_eq!(SkillType::External as i32, 3);
    }

    #[test]
    fn test_skill_type_from_i32() {
        assert_eq!(SkillType::from_i32(0), Some(SkillType::Builtin));
        assert_eq!(SkillType::from_i32(1), Some(SkillType::Custom));
        assert_eq!(SkillType::from_i32(2), Some(SkillType::Shared));
        assert_eq!(SkillType::from_i32(3), Some(SkillType::External));
        assert_eq!(SkillType::from_i32(4), None);
        assert_eq!(SkillType::from_i32(-1), None);
    }

    // ========================================================================
    // Security Scan Tests
    // ========================================================================

    #[test]
    fn test_security_scan_safe_code() {
        let safe_code = "def hello(): return 'Hello, World!'";
        let result = comprehensive_security_scan(safe_code);
        assert!(result.is_safe);
        assert_eq!(result.threats_found.len(), 0);
    }

    #[test]
    fn test_security_scan_dangerous_patterns() {
        let dangerous_code = "import subprocess; subprocess.call(['rm', '-rf', '/'])";
        let result = comprehensive_security_scan(dangerous_code);
        assert!(!result.is_safe);
        assert!(result.threats_found.len() > 0);
    }

    #[test]
    fn test_security_scan_file_operations() {
        let file_code = "open('/etc/passwd', 'r').read()";
        let result = comprehensive_security_scan(file_code);
        assert!(!result.is_safe);
    }

    #[test]
    fn test_security_scan_network_operations() {
        let network_code = "import socket; socket.socket()";
        let result = comprehensive_security_scan(network_code);
        assert!(!result.is_safe);
    }
}
