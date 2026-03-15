//! Authentication and Authorization Module
//!
//! Provides aerospace-grade security context management for ClawMesh API.
//! Includes user authentication, permission checking, and audit logging.

use actix_web::{HttpRequest, HttpResponse, FromRequest, dev::Payload};
use std::future::{Ready, ready};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use tracing::{debug, info, warn, error, instrument};

use crate::error::{ClawMeshError, ClawMeshResult, ErrorCode};

/// User role in the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserRole {
    /// Regular user
    User,
    /// Moderator
    Moderator,
    /// Administrator
    Admin,
    /// System (internal operations)
    System,
}

impl Default for UserRole {
    fn default() -> Self {
        Self::User
    }
}

/// Security context for authenticated requests
/// Contains all information needed for authorization decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    /// User ID
    pub user_id: i32,
    /// Username
    pub username: String,
    /// User role
    pub role: UserRole,
    /// Session ID for audit trail
    pub session_id: String,
    /// Request ID for tracing
    pub request_id: String,
    /// Authentication timestamp
    pub authenticated_at: DateTime<Utc>,
    /// IP address (for audit)
    pub ip_address: Option<String>,
    /// User agent (for audit)
    pub user_agent: Option<String>,
    /// Is the user an agent (bot)
    pub is_agent: bool,
    /// User's credit score (cached)
    pub credit_score: Option<i32>,
    /// User's reputation tier (cached)
    pub reputation_tier: Option<String>,
}

impl SecurityContext {
    /// Create a new security context
    #[must_use]
    pub fn new(user_id: i32, username: String, role: UserRole) -> Self {
        Self {
            user_id,
            username,
            role,
            session_id: uuid::Uuid::new_v4().to_string(),
            request_id: uuid::Uuid::new_v4().to_string(),
            authenticated_at: Utc::now(),
            ip_address: None,
            user_agent: None,
            is_agent: false,
            credit_score: None,
            reputation_tier: None,
        }
    }

    /// Create a system context for internal operations
    #[must_use]
    pub fn system() -> Self {
        Self {
            user_id: 0,
            username: "system".to_string(),
            role: UserRole::System,
            session_id: "system".to_string(),
            request_id: uuid::Uuid::new_v4().to_string(),
            authenticated_at: Utc::now(),
            ip_address: None,
            user_agent: None,
            is_agent: false,
            credit_score: None,
            reputation_tier: None,
        }
    }

    /// Check if user has admin privileges
    #[must_use]
    pub fn is_admin(&self) -> bool {
        matches!(self.role, UserRole::Admin | UserRole::System)
    }

    /// Check if user has moderator privileges
    #[must_use]
    pub fn is_moderator(&self) -> bool {
        matches!(self.role, UserRole::Moderator | UserRole::Admin | UserRole::System)
    }

    /// Check if this is a system context
    #[must_use]
    pub fn is_system(&self) -> bool {
        self.role == UserRole::System
    }

    /// Verify user can perform action on target user
    pub fn can_act_on_user(&self, target_user_id: i32) -> ClawMeshResult<()> {
        // System can act on anyone
        if self.is_system() {
            return Ok(());
        }

        // Users can act on themselves
        if self.user_id == target_user_id {
            return Ok(());
        }

        // Admins can act on anyone except other admins
        if self.is_admin() {
            return Ok(());
        }

        Err(ClawMeshError::new(ErrorCode::InsufficientPermissions))
    }

    /// Verify user has minimum credit score
    pub fn require_credit(&self, min_credit: i32) -> ClawMeshResult<()> {
        match self.credit_score {
            Some(score) if score >= min_credit => Ok(()),
            Some(score) => {
                warn!(
                    user_id = self.user_id,
                    required = min_credit,
                    actual = score,
                    "Insufficient credit"
                );
                Err(ClawMeshError::with_message(
                    ErrorCode::InsufficientCredit,
                    format!("Requires {} credit, you have {}", min_credit, score),
                ))
            }
            None => {
                // Credit not loaded, allow by default (will be checked at DB level)
                Ok(())
            }
        }
    }

    /// Log an action for audit trail
    #[instrument(skip(self))]
    pub fn audit_log(&self, action: &str, resource: &str, resource_id: Option<i64>) {
        info!(
            user_id = self.user_id,
            username = %self.username,
            role = ?self.role,
            session_id = %self.session_id,
            request_id = %self.request_id,
            action = action,
            resource = resource,
            resource_id = ?resource_id,
            ip = ?self.ip_address,
            "Audit log"
        );
    }
}

/// Permission types for authorization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permission {
    /// Can send friend requests
    SendFriendRequest,
    /// Can send direct messages
    SendDirectMessage,
    /// Can create groups
    CreateGroup,
    /// Can moderate content
    ModerateContent,
    /// Can manage users
    ManageUsers,
    /// Can access admin panel
    AdminAccess,
}

impl Permission {
    /// Get minimum credit required for this permission
    #[must_use]
    pub const fn min_credit(&self) -> i32 {
        match self {
            Self::SendFriendRequest => 0,
            Self::SendDirectMessage => 0,
            Self::CreateGroup => 100,
            Self::ModerateContent => 500,
            Self::ManageUsers => 1000,
            Self::AdminAccess => 0, // Role-based, not credit-based
        }
    }

    /// Get minimum role required for this permission
    #[must_use]
    pub const fn min_role(&self) -> UserRole {
        match self {
            Self::SendFriendRequest => UserRole::User,
            Self::SendDirectMessage => UserRole::User,
            Self::CreateGroup => UserRole::User,
            Self::ModerateContent => UserRole::Moderator,
            Self::ManageUsers => UserRole::Admin,
            Self::AdminAccess => UserRole::Admin,
        }
    }
}

/// Check if security context has permission
pub fn check_permission(ctx: &SecurityContext, permission: Permission) -> ClawMeshResult<()> {
    // Check role requirement
    let has_role = match permission.min_role() {
        UserRole::User => true,
        UserRole::Moderator => ctx.is_moderator(),
        UserRole::Admin => ctx.is_admin(),
        UserRole::System => ctx.is_system(),
    };

    if !has_role {
        warn!(
            user_id = ctx.user_id,
            permission = ?permission,
            role = ?ctx.role,
            "Permission denied: insufficient role"
        );
        return Err(ClawMeshError::new(ErrorCode::InsufficientPermissions));
    }

    // Check credit requirement
    ctx.require_credit(permission.min_credit())?;

    debug!(
        user_id = ctx.user_id,
        permission = ?permission,
        "Permission granted"
    );

    Ok(())
}

/// Rate limiting configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum requests per minute
    pub requests_per_minute: u32,
    /// Maximum messages per minute
    pub messages_per_minute: u32,
    /// Maximum friend requests per hour
    pub friend_requests_per_hour: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 60,
            messages_per_minute: 30,
            friend_requests_per_hour: 20,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_context_creation() {
        let ctx = SecurityContext::new(1, "testuser".to_string(), UserRole::User);
        assert_eq!(ctx.user_id, 1);
        assert_eq!(ctx.username, "testuser");
        assert_eq!(ctx.role, UserRole::User);
        assert!(!ctx.is_admin());
        assert!(!ctx.is_moderator());
    }

    #[test]
    fn test_system_context() {
        let ctx = SecurityContext::system();
        assert!(ctx.is_system());
        assert!(ctx.is_admin());
        assert!(ctx.is_moderator());
    }

    #[test]
    fn test_role_hierarchy() {
        let user = SecurityContext::new(1, "user".to_string(), UserRole::User);
        let mod_ctx = SecurityContext::new(2, "mod".to_string(), UserRole::Moderator);
        let admin = SecurityContext::new(3, "admin".to_string(), UserRole::Admin);

        assert!(!user.is_moderator());
        assert!(!user.is_admin());

        assert!(mod_ctx.is_moderator());
        assert!(!mod_ctx.is_admin());

        assert!(admin.is_moderator());
        assert!(admin.is_admin());
    }

    #[test]
    fn test_can_act_on_user() {
        let user = SecurityContext::new(1, "user".to_string(), UserRole::User);
        
        // Can act on self
        assert!(user.can_act_on_user(1).is_ok());
        
        // Cannot act on others
        assert!(user.can_act_on_user(2).is_err());
    }

    #[test]
    fn test_admin_can_act_on_anyone() {
        let admin = SecurityContext::new(1, "admin".to_string(), UserRole::Admin);
        
        assert!(admin.can_act_on_user(1).is_ok());
        assert!(admin.can_act_on_user(2).is_ok());
        assert!(admin.can_act_on_user(999).is_ok());
    }

    #[test]
    fn test_credit_requirement() {
        let mut ctx = SecurityContext::new(1, "user".to_string(), UserRole::User);
        ctx.credit_score = Some(50);

        assert!(ctx.require_credit(50).is_ok());
        assert!(ctx.require_credit(10).is_ok());
        assert!(ctx.require_credit(100).is_err());
    }

    #[test]
    fn test_permission_check() {
        let user = SecurityContext::new(1, "user".to_string(), UserRole::User);
        let admin = SecurityContext::new(2, "admin".to_string(), UserRole::Admin);

        assert!(check_permission(&user, Permission::SendFriendRequest).is_ok());
        assert!(check_permission(&user, Permission::AdminAccess).is_err());

        assert!(check_permission(&admin, Permission::SendFriendRequest).is_ok());
        assert!(check_permission(&admin, Permission::AdminAccess).is_ok());
    }
}
