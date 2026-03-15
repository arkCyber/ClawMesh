//! Lemmy Extensions for ClawMesh
//!
//! This module extends Lemmy's functionality with ClawMesh-specific features
//! while maintaining full compatibility with Lemmy's core systems.

use actix_web::{web, HttpRequest, HttpResponse};
use lemmy_api_utils::{context::LemmyContext, local_user_view_from_jwt_opt};
use lemmy_db_schema::source::person::Person;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_diesel_utils::{connection::get_conn, traits::Crud};
use lemmy_utils::error::{LemmyErrorType, LemmyResult};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, instrument};

/// Extended user info with ClawMesh features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedUserInfo {
    /// Base Lemmy user info
    pub person: Person,
    /// Is user an admin
    pub is_admin: bool,
    /// ClawMesh credit score
    pub credit_score: Option<i32>,
    /// ClawMesh reputation tier
    pub reputation_tier: Option<String>,
    /// Is user an AI agent
    pub is_agent: bool,
}

impl ExtendedUserInfo {
    /// Create from LocalUserView with ClawMesh extensions
    pub async fn from_local_user_view(
        local_user_view: &LocalUserView,
        context: &LemmyContext,
    ) -> LemmyResult<Self> {
        let person = local_user_view.person.clone();
        let is_admin = local_user_view.local_user.admin;
        
        // Fetch ClawMesh-specific data
        let credit_score = Self::fetch_credit_score(person.id, context).await.ok();
        let reputation_tier = Self::fetch_reputation_tier(person.id, context).await.ok();
        let is_agent = Self::check_if_agent(person.id, context).await.unwrap_or(false);
        
        Ok(Self {
            person,
            is_admin,
            credit_score,
            reputation_tier,
            is_agent,
        })
    }
    
    /// Fetch user's credit score from ClawMesh system
    async fn fetch_credit_score(
        person_id: lemmy_db_schema::newtypes::PersonId,
        context: &LemmyContext,
    ) -> LemmyResult<i32> {
        use clawmesh_credit::get_user_credit_score;
        let pool = &mut context.pool();
        let conn = &mut get_conn(pool).await?;
        get_user_credit_score(person_id, conn).await
    }
    
    /// Fetch user's reputation tier
    async fn fetch_reputation_tier(
        person_id: lemmy_db_schema::newtypes::PersonId,
        context: &LemmyContext,
    ) -> LemmyResult<String> {
        use clawmesh_credit::get_reputation_tier;
        let pool = &mut context.pool();
        let conn = &mut get_conn(pool).await?;
        get_reputation_tier(person_id, conn).await
    }
    
    /// Check if user is an AI agent
    async fn check_if_agent(
        person_id: lemmy_db_schema::newtypes::PersonId,
        context: &LemmyContext,
    ) -> LemmyResult<bool> {
        use clawmesh_agent::is_agent;
        let pool = &mut context.pool();
        let conn = &mut get_conn(pool).await?;
        is_agent(person_id, conn).await
    }
}

/// Helper function to get authenticated user with ClawMesh extensions
#[instrument(skip(req, context))]
pub async fn get_extended_user_from_jwt(
    req: &HttpRequest,
    context: &web::Data<LemmyContext>,
) -> LemmyResult<Option<ExtendedUserInfo>> {
    // Get JWT from cookie or header
    let jwt = req
        .cookie("jwt")
        .map(|c| c.value().to_string())
        .or_else(|| {
            req.headers()
                .get("Authorization")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.strip_prefix("Bearer "))
                .map(|s| s.to_string())
        });
    
    // Validate JWT using Lemmy's system
    let local_user_view = local_user_view_from_jwt_opt(jwt.as_deref(), context).await?;
    
    // Add ClawMesh extensions
    match local_user_view {
        Some(view) => {
            let extended = ExtendedUserInfo::from_local_user_view(&view, context).await?;
            debug!(
                user_id = extended.person.id.0,
                username = %extended.person.name,
                is_admin = extended.is_admin,
                credit_score = ?extended.credit_score,
                "Extended user authenticated"
            );
            Ok(Some(extended))
        }
        None => Ok(None),
    }
}

/// Require authenticated user with ClawMesh extensions
pub async fn require_extended_user(
    req: &HttpRequest,
    context: &web::Data<LemmyContext>,
) -> LemmyResult<ExtendedUserInfo> {
    get_extended_user_from_jwt(req, context)
        .await?
        .ok_or(LemmyErrorType::NotLoggedIn.into())
}

/// Check if user has minimum credit score
pub fn require_credit_score(user: &ExtendedUserInfo, min_score: i32) -> LemmyResult<()> {
    match user.credit_score {
        Some(score) if score >= min_score => Ok(()),
        Some(score) => {
            info!(
                user_id = user.person.id.0,
                required = min_score,
                actual = score,
                "Insufficient credit score"
            );
            Err(LemmyErrorType::PermissionDenied.into())
        }
        None => {
            // Credit score not loaded, allow by default
            Ok(())
        }
    }
}

/// Check if user is an admin or moderator
pub async fn require_mod_or_admin(
    user: &ExtendedUserInfo,
    community_id: Option<lemmy_db_schema::newtypes::CommunityId>,
    context: &LemmyContext,
) -> LemmyResult<()> {
    // Admin check
    if user.is_admin {
        return Ok(());
    }
    
    // Moderator check
    if let Some(comm_id) = community_id {
        use lemmy_db_views_community_moderator::CommunityModeratorView;
        CommunityModeratorView::check_is_community_moderator(
            &mut context.pool(),
            comm_id,
            user.person.id,
        )
        .await?;
        return Ok(());
    }
    
    Err(LemmyErrorType::NotAModOrAdmin.into())
}

/// Example API endpoint using extended authentication
#[instrument(skip(req, context))]
pub async fn example_protected_endpoint(
    req: HttpRequest,
    context: web::Data<LemmyContext>,
) -> LemmyResult<HttpResponse> {
    // Get authenticated user with ClawMesh extensions
    let user = require_extended_user(&req, &context).await?;
    
    // Check credit score requirement
    require_credit_score(&user, 100)?;
    
    // Business logic here
    info!(
        user_id = user.person.id.0,
        username = %user.person.name,
        credit_score = ?user.credit_score,
        "Protected endpoint accessed"
    );
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "user_id": user.person.id,
        "username": user.person.name,
        "credit_score": user.credit_score,
        "is_admin": user.is_admin,
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credit_score_check() {
        let user = ExtendedUserInfo {
            person: Person {
                id: 1.into(),
                name: "test".to_string(),
                ..Default::default()
            },
            is_admin: false,
            credit_score: Some(150),
            reputation_tier: None,
            is_agent: false,
        };
        
        // Should pass
        assert!(require_credit_score(&user, 100).is_ok());
        
        // Should fail
        assert!(require_credit_score(&user, 200).is_err());
    }
}
