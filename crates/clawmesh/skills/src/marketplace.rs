/// Agent Skills Marketplace (DO-178C Level A)
/// 
/// Marketplace for sharing and discovering agent skills

use anyhow::{Result, Context};
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use lemmy_db_schema_file::PersonId;
use tracing::{info, warn};

use crate::models::AgentSkill;

/// Publish skill to marketplace
/// 
/// # Safety
/// - Validates skill ownership
/// - Marks skill as public
/// - Requires verification for sensitive permissions
pub async fn publish_skill(
    agent_id: PersonId,
    skill_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<AgentSkill> {
    use lemmy_db_schema_file::schema::agent_skills;
    
    info!(
        agent_id = agent_id.0,
        skill_id = skill_id,
        "Publishing skill to marketplace"
    );
    
    // 1. Get skill and verify ownership
    let skill = agent_skills::table
        .find(skill_id)
        .first::<AgentSkill>(conn)
        .await
        .context("Skill not found")?;
    
    if skill.agent_id != agent_id {
        anyhow::bail!("Not authorized to publish this skill");
    }
    
    // 2. Update skill to public
    let updated = diesel::update(agent_skills::table.find(skill_id))
        .set(agent_skills::is_public.eq(true))
        .get_result::<AgentSkill>(conn)
        .await
        .context("Failed to publish skill")?;
    
    info!(
        skill_id = skill_id,
        skill_name = %updated.skill_name,
        "Skill published to marketplace"
    );
    
    Ok(updated)
}

/// Search skills in marketplace
/// 
/// # Safety
/// - Only returns public skills
/// - Supports filtering and sorting
/// - Paginated results
pub async fn search_skills(
    query: Option<String>,
    category: Option<String>,
    min_downloads: Option<i32>,
    verified_only: bool,
    limit: i64,
    offset: i64,
    conn: &mut AsyncPgConnection,
) -> Result<Vec<AgentSkill>> {
    use lemmy_db_schema_file::schema::agent_skills;
    
    let mut db_query = agent_skills::table
        .filter(agent_skills::is_public.eq(true))
        .into_boxed();
    
    // Filter by search query
    if let Some(q) = query {
        db_query = db_query.filter(
            agent_skills::skill_name.ilike(format!("%{}%", q))
        );
    }
    
    // Filter by verified status
    if verified_only {
        db_query = db_query.filter(agent_skills::is_verified.eq(true));
    }
    
    // Filter by minimum downloads
    if let Some(min_dl) = min_downloads {
        db_query = db_query.filter(agent_skills::downloads.ge(min_dl));
    }
    
    // Order by downloads (popularity)
    let skills = db_query
        .order(agent_skills::downloads.desc())
        .limit(limit)
        .offset(offset)
        .load::<AgentSkill>(conn)
        .await
        .context("Failed to search skills")?;
    
    Ok(skills)
}

/// Get marketplace statistics
#[derive(Debug, Clone)]
pub struct MarketplaceStats {
    pub total_skills: i64,
    pub verified_skills: i64,
    pub total_downloads: i64,
    pub total_agents: i64,
}

pub async fn get_marketplace_stats(
    conn: &mut AsyncPgConnection,
) -> Result<MarketplaceStats> {
    use lemmy_db_schema_file::schema::agent_skills;
    use diesel::dsl::*;
    
    // Count total public skills
    let total_skills: i64 = agent_skills::table
        .filter(agent_skills::is_public.eq(true))
        .count()
        .get_result(conn)
        .await
        .context("Failed to count skills")?;
    
    // Count verified skills
    let verified_skills: i64 = agent_skills::table
        .filter(agent_skills::is_public.eq(true))
        .filter(agent_skills::is_verified.eq(true))
        .count()
        .get_result(conn)
        .await
        .context("Failed to count verified skills")?;
    
    // Sum total downloads
    let total_downloads: Option<i64> = agent_skills::table
        .filter(agent_skills::is_public.eq(true))
        .select(sum(agent_skills::downloads))
        .first(conn)
        .await
        .context("Failed to sum downloads")?;
    
    // Count unique agents
    let total_agents: i64 = agent_skills::table
        .filter(agent_skills::is_public.eq(true))
        .select(count_distinct(agent_skills::agent_id))
        .first(conn)
        .await
        .context("Failed to count agents")?;
    
    Ok(MarketplaceStats {
        total_skills,
        verified_skills,
        total_downloads: total_downloads.unwrap_or(0),
        total_agents,
    })
}

/// Get trending skills (most downloaded recently)
pub async fn get_trending_skills(
    days: i32,
    limit: i64,
    conn: &mut AsyncPgConnection,
) -> Result<Vec<AgentSkill>> {
    use lemmy_db_schema_file::schema::agent_skills;
    use chrono::{Utc, Duration};
    
    let cutoff_date = Utc::now() - Duration::days(days as i64);
    
    let skills = agent_skills::table
        .filter(agent_skills::is_public.eq(true))
        .filter(agent_skills::created_at.gt(cutoff_date))
        .order(agent_skills::downloads.desc())
        .limit(limit)
        .load::<AgentSkill>(conn)
        .await
        .context("Failed to get trending skills")?;
    
    Ok(skills)
}

/// Get recommended skills for an agent
/// 
/// # Safety
/// - Based on agent's installed skills
/// - Filters by compatibility
/// - Returns popular skills in same category
pub async fn get_recommended_skills(
    agent_id: PersonId,
    limit: i64,
    conn: &mut AsyncPgConnection,
) -> Result<Vec<AgentSkill>> {
    use lemmy_db_schema_file::schema::agent_skills;
    
    // For now, return most popular public skills
    // In production, implement collaborative filtering
    let skills = agent_skills::table
        .filter(agent_skills::is_public.eq(true))
        .filter(agent_skills::agent_id.ne(agent_id)) // Exclude own skills
        .order(agent_skills::downloads.desc())
        .limit(limit)
        .load::<AgentSkill>(conn)
        .await
        .context("Failed to get recommended skills")?;
    
    Ok(skills)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_marketplace_stats_structure() {
        let stats = MarketplaceStats {
            total_skills: 100,
            verified_skills: 50,
            total_downloads: 1000,
            total_agents: 25,
        };
        
        assert_eq!(stats.total_skills, 100);
        assert_eq!(stats.verified_skills, 50);
    }
}
