/// ClawMesh Custom Error Types (DO-178C Level A)
/// 
/// Defines comprehensive error handling for all ClawMesh modules

use std::fmt;
use actix_web::{error::ResponseError, http::StatusCode, HttpResponse};
use serde::{Deserialize, Serialize};

/// ClawMesh error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClawMeshError {
    // Agent errors
    AgentNotFound(String),
    AgentAlreadyExists(String),
    AgentInstallationFailed(String),
    AgentHeartbeatTimeout(String),
    AgentAuthenticationFailed(String),
    
    // Reputation errors
    ReputationNotFound(String),
    InvalidVoteType(String),
    VoteAlreadyExists(String),
    ReputationCalculationFailed(String),
    
    // Skills errors
    SkillNotFound(String),
    SkillValidationFailed(String),
    SkillExecutionFailed(String),
    SkillSecurityViolation(String),
    SkillMarketplaceError(String),
    
    // Database errors
    DatabaseError(String),
    DatabaseConnectionFailed(String),
    QueryFailed(String),
    TransactionFailed(String),
    
    // Validation errors
    ValidationError(String),
    InvalidInput(String),
    MissingRequiredField(String),
    
    // Authentication/Authorization errors
    Unauthorized(String),
    Forbidden(String),
    TokenExpired(String),
    InvalidToken(String),
    
    // General errors
    InternalError(String),
    NotImplemented(String),
    ServiceUnavailable(String),
}

impl fmt::Display for ClawMeshError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Agent errors
            ClawMeshError::AgentNotFound(msg) => write!(f, "Agent not found: {}", msg),
            ClawMeshError::AgentAlreadyExists(msg) => write!(f, "Agent already exists: {}", msg),
            ClawMeshError::AgentInstallationFailed(msg) => write!(f, "Agent installation failed: {}", msg),
            ClawMeshError::AgentHeartbeatTimeout(msg) => write!(f, "Agent heartbeat timeout: {}", msg),
            ClawMeshError::AgentAuthenticationFailed(msg) => write!(f, "Agent authentication failed: {}", msg),
            
            // Reputation errors
            ClawMeshError::ReputationNotFound(msg) => write!(f, "Reputation not found: {}", msg),
            ClawMeshError::InvalidVoteType(msg) => write!(f, "Invalid vote type: {}", msg),
            ClawMeshError::VoteAlreadyExists(msg) => write!(f, "Vote already exists: {}", msg),
            ClawMeshError::ReputationCalculationFailed(msg) => write!(f, "Reputation calculation failed: {}", msg),
            
            // Skills errors
            ClawMeshError::SkillNotFound(msg) => write!(f, "Skill not found: {}", msg),
            ClawMeshError::SkillValidationFailed(msg) => write!(f, "Skill validation failed: {}", msg),
            ClawMeshError::SkillExecutionFailed(msg) => write!(f, "Skill execution failed: {}", msg),
            ClawMeshError::SkillSecurityViolation(msg) => write!(f, "Skill security violation: {}", msg),
            ClawMeshError::SkillMarketplaceError(msg) => write!(f, "Skill marketplace error: {}", msg),
            
            // Database errors
            ClawMeshError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            ClawMeshError::DatabaseConnectionFailed(msg) => write!(f, "Database connection failed: {}", msg),
            ClawMeshError::QueryFailed(msg) => write!(f, "Query failed: {}", msg),
            ClawMeshError::TransactionFailed(msg) => write!(f, "Transaction failed: {}", msg),
            
            // Validation errors
            ClawMeshError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            ClawMeshError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            ClawMeshError::MissingRequiredField(msg) => write!(f, "Missing required field: {}", msg),
            
            // Authentication/Authorization errors
            ClawMeshError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            ClawMeshError::Forbidden(msg) => write!(f, "Forbidden: {}", msg),
            ClawMeshError::TokenExpired(msg) => write!(f, "Token expired: {}", msg),
            ClawMeshError::InvalidToken(msg) => write!(f, "Invalid token: {}", msg),
            
            // General errors
            ClawMeshError::InternalError(msg) => write!(f, "Internal error: {}", msg),
            ClawMeshError::NotImplemented(msg) => write!(f, "Not implemented: {}", msg),
            ClawMeshError::ServiceUnavailable(msg) => write!(f, "Service unavailable: {}", msg),
        }
    }
}

impl std::error::Error for ClawMeshError {}

impl ResponseError for ClawMeshError {
    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();
        let error_message = self.to_string();
        
        HttpResponse::build(status_code).json(serde_json::json!({
            "error": error_message,
            "status": status_code.as_u16(),
        }))
    }
    
    fn status_code(&self) -> StatusCode {
        match self {
            // 404 Not Found
            ClawMeshError::AgentNotFound(_) |
            ClawMeshError::ReputationNotFound(_) |
            ClawMeshError::SkillNotFound(_) => StatusCode::NOT_FOUND,
            
            // 400 Bad Request
            ClawMeshError::InvalidVoteType(_) |
            ClawMeshError::ValidationError(_) |
            ClawMeshError::InvalidInput(_) |
            ClawMeshError::MissingRequiredField(_) |
            ClawMeshError::SkillValidationFailed(_) => StatusCode::BAD_REQUEST,
            
            // 401 Unauthorized
            ClawMeshError::Unauthorized(_) |
            ClawMeshError::AgentAuthenticationFailed(_) |
            ClawMeshError::TokenExpired(_) |
            ClawMeshError::InvalidToken(_) => StatusCode::UNAUTHORIZED,
            
            // 403 Forbidden
            ClawMeshError::Forbidden(_) |
            ClawMeshError::SkillSecurityViolation(_) => StatusCode::FORBIDDEN,
            
            // 409 Conflict
            ClawMeshError::AgentAlreadyExists(_) |
            ClawMeshError::VoteAlreadyExists(_) => StatusCode::CONFLICT,
            
            // 503 Service Unavailable
            ClawMeshError::ServiceUnavailable(_) |
            ClawMeshError::DatabaseConnectionFailed(_) => StatusCode::SERVICE_UNAVAILABLE,
            
            // 501 Not Implemented
            ClawMeshError::NotImplemented(_) => StatusCode::NOT_IMPLEMENTED,
            
            // 500 Internal Server Error (default)
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

/// Convert from anyhow::Error to ClawMeshError
impl From<anyhow::Error> for ClawMeshError {
    fn from(err: anyhow::Error) -> Self {
        ClawMeshError::InternalError(err.to_string())
    }
}

/// Convert from diesel::result::Error to ClawMeshError
impl From<diesel::result::Error> for ClawMeshError {
    fn from(err: diesel::result::Error) -> Self {
        match err {
            diesel::result::Error::NotFound => {
                ClawMeshError::DatabaseError("Record not found".to_string())
            }
            diesel::result::Error::DatabaseError(kind, info) => {
                ClawMeshError::DatabaseError(format!("{:?}: {}", kind, info.message()))
            }
            _ => ClawMeshError::DatabaseError(err.to_string()),
        }
    }
}

/// Result type alias for ClawMesh operations
pub type ClawMeshResult<T> = Result<T, ClawMeshError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let error = ClawMeshError::AgentNotFound("agent-123".to_string());
        assert_eq!(error.to_string(), "Agent not found: agent-123");
    }

    #[test]
    fn test_error_status_codes() {
        assert_eq!(
            ClawMeshError::AgentNotFound("test".to_string()).status_code(),
            StatusCode::NOT_FOUND
        );
        
        assert_eq!(
            ClawMeshError::ValidationError("test".to_string()).status_code(),
            StatusCode::BAD_REQUEST
        );
        
        assert_eq!(
            ClawMeshError::Unauthorized("test".to_string()).status_code(),
            StatusCode::UNAUTHORIZED
        );
        
        assert_eq!(
            ClawMeshError::Forbidden("test".to_string()).status_code(),
            StatusCode::FORBIDDEN
        );
        
        assert_eq!(
            ClawMeshError::AgentAlreadyExists("test".to_string()).status_code(),
            StatusCode::CONFLICT
        );
        
        assert_eq!(
            ClawMeshError::ServiceUnavailable("test".to_string()).status_code(),
            StatusCode::SERVICE_UNAVAILABLE
        );
    }

    #[test]
    fn test_error_response() {
        let error = ClawMeshError::AgentNotFound("agent-123".to_string());
        let response = error.error_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_from_anyhow_error() {
        let anyhow_err = anyhow::anyhow!("test error");
        let clawmesh_err: ClawMeshError = anyhow_err.into();
        assert!(matches!(clawmesh_err, ClawMeshError::InternalError(_)));
    }
}
