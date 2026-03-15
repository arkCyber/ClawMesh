use anyhow::{anyhow, Result};
use serde_json::Value;

/// Validate agent username
pub fn validate_username(username: &str) -> Result<()> {
    if username.is_empty() {
        return Err(anyhow!("Username cannot be empty"));
    }

    if username.len() < 3 {
        return Err(anyhow!("Username must be at least 3 characters"));
    }

    if username.len() > 50 {
        return Err(anyhow!("Username must be at most 50 characters"));
    }

    // Check for valid characters (alphanumeric, underscore, hyphen)
    if !username
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return Err(anyhow!(
            "Username can only contain alphanumeric characters, underscores, and hyphens"
        ));
    }

    // Must start with a letter or number
    if let Some(first_char) = username.chars().next() {
        if !first_char.is_alphanumeric() {
            return Err(anyhow!("Username must start with a letter or number"));
        }
    }

    Ok(())
}

/// Validate agent metadata
pub fn validate_metadata(metadata: &Option<Value>) -> Result<()> {
    if let Some(meta) = metadata {
        if !meta.is_object() {
            return Err(anyhow!("Metadata must be a JSON object"));
        }

        // Check metadata size (limit to 10KB)
        let json_str = serde_json::to_string(meta)?;
        if json_str.len() > 10240 {
            return Err(anyhow!("Metadata too large (max 10KB)"));
        }

        // Validate required fields if present
        if let Some(obj) = meta.as_object() {
            // Model field should be a string if present
            if let Some(model) = obj.get("model") {
                if !model.is_string() {
                    return Err(anyhow!("Metadata 'model' field must be a string"));
                }
            }

            // Version field should be a string if present
            if let Some(version) = obj.get("version") {
                if !version.is_string() {
                    return Err(anyhow!("Metadata 'version' field must be a string"));
                }
            }

            // Capabilities should be an array if present
            if let Some(capabilities) = obj.get("capabilities") {
                if !capabilities.is_array() {
                    return Err(anyhow!("Metadata 'capabilities' field must be an array"));
                }
            }
        }
    }

    Ok(())
}

/// Validate heartbeat interval
pub fn validate_heartbeat_interval(interval: i32) -> Result<()> {
    if interval < 300 {
        return Err(anyhow!("Heartbeat interval must be at least 300 seconds (5 minutes)"));
    }

    if interval > 86400 {
        return Err(anyhow!("Heartbeat interval must be at most 86400 seconds (24 hours)"));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_username_validation() {
        assert!(validate_username("valid_bot_123").is_ok());
        assert!(validate_username("bot").is_ok());
        assert!(validate_username("my-agent").is_ok());

        assert!(validate_username("").is_err());
        assert!(validate_username("ab").is_err());
        assert!(validate_username("a".repeat(51).as_str()).is_err());
        assert!(validate_username("invalid bot").is_err());
        assert!(validate_username("_invalid").is_err());
        assert!(validate_username("-invalid").is_err());
    }

    #[test]
    fn test_metadata_validation() {
        let valid_meta = Some(json!({
            "model": "gpt-4",
            "version": "1.0",
            "capabilities": ["chat", "analysis"]
        }));
        assert!(validate_metadata(&valid_meta).is_ok());

        let invalid_meta = Some(json!("not an object"));
        assert!(validate_metadata(&invalid_meta).is_err());

        let invalid_model = Some(json!({
            "model": 123
        }));
        assert!(validate_metadata(&invalid_model).is_err());

        assert!(validate_metadata(&None).is_ok());
    }

    #[test]
    fn test_heartbeat_interval_validation() {
        assert!(validate_heartbeat_interval(3600).is_ok());
        assert!(validate_heartbeat_interval(14400).is_ok());

        assert!(validate_heartbeat_interval(100).is_err());
        assert!(validate_heartbeat_interval(100000).is_err());
    }
}
