/// 验证功能测试
/// 这些测试不需要数据库连接

#[cfg(test)]
mod tests {
    use clawmesh_agent::validation::*;
    use serde_json::json;

    #[test]
    fn test_username_validation_valid() {
        assert!(validate_username("valid_bot").is_ok());
        assert!(validate_username("bot123").is_ok());
        assert!(validate_username("my-agent").is_ok());
        assert!(validate_username("Agent_Bot_001").is_ok());
    }

    #[test]
    fn test_username_validation_invalid() {
        // 太短
        assert!(validate_username("ab").is_err());
        
        // 太长
        assert!(validate_username(&"a".repeat(51)).is_err());
        
        // 包含空格
        assert!(validate_username("invalid bot").is_err());
        
        // 特殊字符
        assert!(validate_username("bot@123").is_err());
        assert!(validate_username("bot#123").is_err());
        
        // 不以字母数字开头
        assert!(validate_username("_bot").is_err());
        assert!(validate_username("-bot").is_err());
        
        // 空字符串
        assert!(validate_username("").is_err());
    }

    #[test]
    fn test_metadata_validation_valid() {
        let valid_meta = Some(json!({
            "model": "gpt-4",
            "version": "1.0",
            "capabilities": ["chat", "moderation"]
        }));
        assert!(validate_metadata(&valid_meta).is_ok());

        // None 也是有效的
        assert!(validate_metadata(&None).is_ok());

        // 简单对象
        let simple = Some(json!({"key": "value"}));
        assert!(validate_metadata(&simple).is_ok());
    }

    #[test]
    fn test_metadata_validation_invalid() {
        // 不是对象
        let not_object = Some(json!("string"));
        assert!(validate_metadata(&not_object).is_err());

        let not_object_array = Some(json!(["array"]));
        assert!(validate_metadata(&not_object_array).is_err());

        // model 字段类型错误
        let wrong_model_type = Some(json!({
            "model": 123
        }));
        assert!(validate_metadata(&wrong_model_type).is_err());

        // version 字段类型错误
        let wrong_version_type = Some(json!({
            "version": true
        }));
        assert!(validate_metadata(&wrong_version_type).is_err());

        // capabilities 字段类型错误
        let wrong_capabilities_type = Some(json!({
            "capabilities": "not an array"
        }));
        assert!(validate_metadata(&wrong_capabilities_type).is_err());

        // 太大的元数据
        let large_meta = Some(json!({
            "data": "x".repeat(11000)
        }));
        assert!(validate_metadata(&large_meta).is_err());
    }

    #[test]
    fn test_heartbeat_interval_validation_valid() {
        assert!(validate_heartbeat_interval(300).is_ok());   // 5 分钟
        assert!(validate_heartbeat_interval(3600).is_ok());  // 1 小时
        assert!(validate_heartbeat_interval(14400).is_ok()); // 4 小时
        assert!(validate_heartbeat_interval(86400).is_ok()); // 24 小时
    }

    #[test]
    fn test_heartbeat_interval_validation_invalid() {
        assert!(validate_heartbeat_interval(100).is_err());    // 太短
        assert!(validate_heartbeat_interval(299).is_err());    // 刚好低于最小值
        assert!(validate_heartbeat_interval(86401).is_err());  // 刚好超过最大值
        assert!(validate_heartbeat_interval(100000).is_err()); // 太长
    }

    #[test]
    fn test_edge_cases() {
        // 边界值测试
        assert!(validate_username("abc").is_ok());  // 最小长度
        assert!(validate_username(&"a".repeat(50)).is_ok()); // 最大长度
        
        assert!(validate_heartbeat_interval(300).is_ok());   // 最小间隔
        assert!(validate_heartbeat_interval(86400).is_ok()); // 最大间隔
    }
}
