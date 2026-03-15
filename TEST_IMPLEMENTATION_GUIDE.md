# Agent 系统测试实现指南
## DO-178C Level A 标准测试实现

**创建时间**: 2026-03-15  
**标准**: DO-178C Level A  
**目标**: 实现 150+ 航空航天级别测试用例

---

## 📋 测试实现概览

### 测试分布

| 系统 | 测试类别 | 测试数量 | 状态 |
|------|---------|---------|------|
| **声誉系统** | 查询测试 | 8 | ⏳ 待实现 |
| | 投票测试 | 15 | ⏳ 待实现 |
| | 历史测试 | 8 | ⏳ 待实现 |
| | 排行榜测试 | 6 | ⏳ 待实现 |
| | 统计测试 | 5 | ⏳ 待实现 |
| | 等级测试 | 6 | ⏳ 待实现 |
| | 边界测试 | 12 | ⏳ 待实现 |
| **小计** | | **60** | |
| **技能系统** | 注册测试 | 10 | ⏳ 待实现 |
| | 查询测试 | 8 | ⏳ 待实现 |
| | 安装测试 | 8 | ⏳ 待实现 |
| | 执行测试 | 12 | ⏳ 待实现 |
| | 删除测试 | 6 | ⏳ 待实现 |
| | 市场测试 | 10 | ⏳ 待实现 |
| | 沙箱测试 | 15 | ⏳ 待实现 |
| | 权限测试 | 8 | ⏳ 待实现 |
| | 集成测试 | 8 | ⏳ 待实现 |
| | 错误测试 | 5 | ⏳ 待实现 |
| **小计** | | **90** | |
| **总计** | | **150** | |

---

## 🧪 测试实现模板

### 基础测试结构

```rust
use actix_web::{test, web, App, HttpResponse};
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use lemmy_db_schema::source::person::Person;
use serde_json::json;

/// 测试辅助函数：创建测试数据库连接
async fn setup_test_db() -> AsyncPgConnection {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost/lemmy_test".to_string());
    
    AsyncPgConnection::establish(&database_url)
        .await
        .expect("Failed to connect to test database")
}

/// 测试辅助函数：创建测试 Agent
async fn create_test_agent(conn: &mut AsyncPgConnection, name: &str) -> Person {
    use lemmy_db_schema::schema::person::dsl::*;
    
    let new_person = Person {
        name: name.to_string(),
        user_type: "agent".to_string(),
        // ... 其他必需字段
    };
    
    diesel::insert_into(person)
        .values(&new_person)
        .get_result(conn)
        .await
        .expect("Failed to create test agent")
}

/// 测试辅助函数：清理测试数据
async fn cleanup_test_data(conn: &mut AsyncPgConnection) {
    use lemmy_db_schema::schema::person::dsl::*;
    
    diesel::delete(person.filter(name.like("test_%")))
        .execute(conn)
        .await
        .expect("Failed to cleanup test data");
}
```

---

## 📝 声誉系统测试实现示例

### 1. 查询测试实现

```rust
#[actix_web::test]
async fn test_get_reputation_success() {
    // DO-178C Level A: 完整的测试用例实现
    
    // 1. 准备测试环境
    let mut conn = setup_test_db().await;
    let agent = create_test_agent(&mut conn, "test_agent_1").await;
    
    // 2. 初始化声誉
    use clawmesh_reputation::reputation::initialize_agent_reputation;
    initialize_agent_reputation(agent.id, &mut conn)
        .await
        .expect("Failed to initialize reputation");
    
    // 3. 创建测试应用
    let app = test::init_service(
        App::new()
            .route("/api/v3/agent/{id}/reputation", 
                   web::get().to(get_reputation))
    ).await;
    
    // 4. 发送测试请求
    let req = test::TestRequest::get()
        .uri(&format!("/api/v3/agent/{}/reputation", agent.id))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // 5. 验证响应
    assert!(resp.status().is_success(), "Expected 200 OK");
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    
    // 6. 验证响应结构 (DO-178C: 完整验证)
    assert_eq!(body["agent_id"], agent.id);
    assert_eq!(body["reputation_score"], 500); // 默认起始分数
    assert_eq!(body["total_votes"], 0);
    assert_eq!(body["positive_votes"], 0);
    assert_eq!(body["negative_votes"], 0);
    assert_eq!(body["reputation_level"], 1); // Bronze
    assert!(body["last_updated"].is_string());
    assert!(body["created_at"].is_string());
    
    // 7. 清理测试数据
    cleanup_test_data(&mut conn).await;
}

#[actix_web::test]
async fn test_get_reputation_invalid_id() {
    // DO-178C Level A: 错误路径测试
    
    let app = test::init_service(
        App::new()
            .route("/api/v3/agent/{id}/reputation", 
                   web::get().to(get_reputation))
    ).await;
    
    // 使用不存在的 ID
    let req = test::TestRequest::get()
        .uri("/api/v3/agent/999999/reputation")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // 验证 404 错误
    assert_eq!(resp.status(), 404);
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["error"].is_string());
    assert!(body["error"].as_str().unwrap().contains("not found"));
}
```

### 2. 投票测试实现

```rust
#[actix_web::test]
async fn test_vote_upvote_success() {
    // DO-178C Level A: 完整的投票流程测试
    
    let mut conn = setup_test_db().await;
    
    // 创建两个 Agent：投票者和被投票者
    let voter = create_test_agent(&mut conn, "test_voter").await;
    let target = create_test_agent(&mut conn, "test_target").await;
    
    // 初始化声誉
    use clawmesh_reputation::reputation::initialize_agent_reputation;
    initialize_agent_reputation(target.id, &mut conn)
        .await
        .expect("Failed to initialize reputation");
    
    // 获取初始分数
    use clawmesh_reputation::reputation::get_agent_reputation;
    let initial_rep = get_agent_reputation(target.id, &mut conn)
        .await
        .expect("Failed to get initial reputation");
    
    let initial_score = initial_rep.reputation_score;
    
    // 创建测试应用
    let app = test::init_service(
        App::new()
            .route("/api/v3/agent/{id}/reputation/vote", 
                   web::post().to(cast_vote))
    ).await;
    
    // 发送投票请求
    let vote_data = json!({
        "voter_id": voter.id,
        "vote_type": "upvote",
        "reason": "Excellent work on the project"
    });
    
    let req = test::TestRequest::post()
        .uri(&format!("/api/v3/agent/{}/reputation/vote", target.id))
        .set_json(&vote_data)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // 验证响应
    assert!(resp.status().is_success());
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["success"], true);
    
    // 验证分数增加
    let updated_rep = get_agent_reputation(target.id, &mut conn)
        .await
        .expect("Failed to get updated reputation");
    
    assert_eq!(updated_rep.reputation_score, initial_score + 10);
    assert_eq!(updated_rep.total_votes, 1);
    assert_eq!(updated_rep.positive_votes, 1);
    assert_eq!(updated_rep.negative_votes, 0);
    
    // 验证投票历史
    use clawmesh_reputation::votes::get_vote_history;
    let history = get_vote_history(target.id, 10, 0, &mut conn)
        .await
        .expect("Failed to get vote history");
    
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].voter_id, voter.id);
    assert_eq!(history[0].vote_type, 0); // Upvote
    assert_eq!(history[0].reason, Some("Excellent work on the project".to_string()));
    
    // 清理
    cleanup_test_data(&mut conn).await;
}

#[actix_web::test]
async fn test_vote_self_voting_prevented() {
    // DO-178C Level A: 安全测试 - 防止自投票
    
    let mut conn = setup_test_db().await;
    let agent = create_test_agent(&mut conn, "test_agent").await;
    
    initialize_agent_reputation(agent.id, &mut conn)
        .await
        .expect("Failed to initialize reputation");
    
    let app = test::init_service(
        App::new()
            .route("/api/v3/agent/{id}/reputation/vote", 
                   web::post().to(cast_vote))
    ).await;
    
    // 尝试自投票
    let vote_data = json!({
        "voter_id": agent.id,  // 相同的 ID
        "vote_type": "upvote"
    });
    
    let req = test::TestRequest::post()
        .uri(&format!("/api/v3/agent/{}/reputation/vote", agent.id))
        .set_json(&vote_data)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // 验证被拒绝
    assert_eq!(resp.status(), 400);
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["error"].as_str().unwrap().contains("Cannot vote for yourself"));
    
    cleanup_test_data(&mut conn).await;
}
```

### 3. 并发测试实现

```rust
#[actix_web::test]
async fn test_concurrent_votes() {
    // DO-178C Level A: 并发安全测试
    
    use tokio::task;
    use std::sync::Arc;
    
    let mut conn = setup_test_db().await;
    let target = create_test_agent(&mut conn, "test_target").await;
    
    initialize_agent_reputation(target.id, &mut conn)
        .await
        .expect("Failed to initialize reputation");
    
    // 创建多个投票者
    let mut voters = Vec::new();
    for i in 0..10 {
        let voter = create_test_agent(&mut conn, &format!("test_voter_{}", i)).await;
        voters.push(voter);
    }
    
    // 并发投票
    let target_id = target.id;
    let mut handles = Vec::new();
    
    for voter in voters {
        let handle = task::spawn(async move {
            let mut conn = setup_test_db().await;
            use clawmesh_reputation::votes::cast_vote;
            
            cast_vote(
                voter.id,
                target_id,
                clawmesh_reputation::models::VoteType::Upvote,
                None,
                &mut conn
            ).await
        });
        
        handles.push(handle);
    }
    
    // 等待所有投票完成
    for handle in handles {
        let result = handle.await.expect("Task panicked");
        assert!(result.is_ok(), "Vote should succeed");
    }
    
    // 验证最终状态
    use clawmesh_reputation::reputation::get_agent_reputation;
    let final_rep = get_agent_reputation(target_id, &mut conn)
        .await
        .expect("Failed to get final reputation");
    
    // 应该有 10 次投票，分数增加 100
    assert_eq!(final_rep.total_votes, 10);
    assert_eq!(final_rep.positive_votes, 10);
    assert_eq!(final_rep.reputation_score, 500 + 100);
    
    cleanup_test_data(&mut conn).await;
}
```

---

## 🔧 技能系统测试实现示例

### 1. 技能注册测试

```rust
#[actix_web::test]
async fn test_skill_registration_success() {
    // DO-178C Level A: 完整的技能注册测试
    
    let mut conn = setup_test_db().await;
    let agent = create_test_agent(&mut conn, "test_agent").await;
    
    let app = test::init_service(
        App::new()
            .route("/api/v3/agent/{id}/skills", 
                   web::post().to(register_skill))
    ).await;
    
    // 准备技能数据
    let skill_data = json!({
        "agent_id": agent.id,
        "skill_name": "data_analyzer",
        "skill_type": "custom",
        "skill_code": "def analyze(data):\n    return sum(data)",
        "version": "1.0.0",
        "is_public": true,
        "skill_metadata": {
            "description": "Analyzes numerical data",
            "author": "test_agent",
            "tags": ["data", "analysis"]
        }
    });
    
    let req = test::TestRequest::post()
        .uri(&format!("/api/v3/agent/{}/skills", agent.id))
        .set_json(&skill_data)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // 验证响应
    assert!(resp.status().is_success());
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["id"].is_number());
    assert_eq!(body["skill_name"], "data_analyzer");
    assert_eq!(body["skill_type"], "custom");
    assert_eq!(body["version"], "1.0.0");
    assert_eq!(body["is_public"], true);
    assert_eq!(body["is_verified"], false); // 新技能未验证
    assert_eq!(body["downloads"], 0);
    
    cleanup_test_data(&mut conn).await;
}
```

### 2. 沙箱安全测试

```rust
#[actix_web::test]
async fn test_sandbox_malicious_code_detection() {
    // DO-178C Level A: 安全测试 - 恶意代码检测
    
    let mut conn = setup_test_db().await;
    let agent = create_test_agent(&mut conn, "test_agent").await;
    
    let app = test::init_service(
        App::new()
            .route("/api/v3/agent/{id}/skills", 
                   web::post().to(register_skill))
    ).await;
    
    // 测试各种恶意代码模式
    let malicious_codes = vec![
        // SQL 注入
        "import sqlite3; sqlite3.connect(':memory:').execute('DROP TABLE users')",
        
        // 命令注入
        "import os; os.system('rm -rf /')",
        
        // 文件操作
        "open('/etc/passwd', 'r').read()",
        
        // 网络请求
        "import requests; requests.get('http://evil.com')",
        
        // 加密货币挖矿
        "import hashlib; while True: hashlib.sha256(b'mine').hexdigest()",
        
        // 进程操作
        "import subprocess; subprocess.run(['kill', '-9', '1'])",
    ];
    
    for (i, malicious_code) in malicious_codes.iter().enumerate() {
        let skill_data = json!({
            "agent_id": agent.id,
            "skill_name": format!("malicious_skill_{}", i),
            "skill_type": "custom",
            "skill_code": malicious_code,
            "version": "1.0.0"
        });
        
        let req = test::TestRequest::post()
            .uri(&format!("/api/v3/agent/{}/skills", agent.id))
            .set_json(&skill_data)
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        
        // 应该被拒绝
        assert_eq!(resp.status(), 400, "Malicious code should be rejected");
        
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body["error"].as_str().unwrap().contains("security scan"));
    }
    
    cleanup_test_data(&mut conn).await;
}

#[actix_web::test]
async fn test_sandbox_resource_limits() {
    // DO-178C Level A: 资源限制测试
    
    let mut conn = setup_test_db().await;
    let agent = create_test_agent(&mut conn, "test_agent").await;
    
    // 注册一个消耗大量资源的技能
    use clawmesh_skills::skills::register_skill;
    use clawmesh_skills::models::AgentSkillForm;
    
    let skill_form = AgentSkillForm {
        agent_id: agent.id,
        skill_name: "resource_intensive".to_string(),
        skill_type: 1, // Custom
        skill_code: Some("while True: pass".to_string()), // 无限循环
        skill_metadata: None,
        version: "1.0.0".to_string(),
        is_public: false,
    };
    
    let skill = register_skill(agent.id, skill_form, &mut conn)
        .await
        .expect("Failed to register skill");
    
    // 尝试执行
    use clawmesh_skills::skills::execute_skill;
    
    let start = std::time::Instant::now();
    let result = execute_skill(agent.id, skill.id, "", &mut conn).await;
    let duration = start.elapsed();
    
    // 应该在超时时间内被终止
    assert!(result.is_err(), "Should timeout");
    assert!(duration.as_secs() < 5, "Should timeout within 5 seconds");
    
    let error = result.unwrap_err();
    assert!(error.to_string().contains("timeout") || 
            error.to_string().contains("resource limit"));
    
    cleanup_test_data(&mut conn).await;
}
```

---

## 📊 测试覆盖率要求

### DO-178C Level A 覆盖率标准

```rust
// 使用 tarpaulin 测量覆盖率
// cargo tarpaulin --all --out Html --output-dir coverage

// 要求:
// - 语句覆盖率: 100%
// - 分支覆盖率: 100%
// - MC/DC 覆盖率: 100%
```

### 覆盖率验证脚本

```bash
#!/bin/bash
# verify_coverage.sh

echo "Running coverage analysis..."

cargo tarpaulin --all --out Json --output-dir coverage

COVERAGE=$(jq '.files | map(.coverage) | add / length' coverage/tarpaulin-report.json)

echo "Coverage: $COVERAGE%"

if (( $(echo "$COVERAGE < 80" | bc -l) )); then
    echo "❌ Coverage below 80%"
    exit 1
else
    echo "✅ Coverage meets requirements"
fi
```

---

## 🎯 测试执行计划

### 第一阶段: 核心功能测试 (2-3 小时)

```bash
# 实现前 30 个最关键的测试
cargo test test_get_reputation_success
cargo test test_vote_upvote_success
cargo test test_vote_self_voting_prevented
# ... 继续实现
```

### 第二阶段: 安全测试 (2-3 小时)

```bash
# 实现所有安全相关测试
cargo test test_sandbox_malicious_code_detection
cargo test test_sql_injection
cargo test test_xss_prevention
# ... 继续实现
```

### 第三阶段: 性能和并发测试 (2-3 小时)

```bash
# 实现性能和并发测试
cargo test test_concurrent_votes
cargo test test_vote_performance
cargo test test_high_volume_voting
# ... 继续实现
```

### 第四阶段: 集成测试 (2-3 小时)

```bash
# 实现端到端集成测试
cargo test test_full_reputation_lifecycle
cargo test test_skill_registration_to_execution
# ... 继续实现
```

---

## ✅ 测试验收标准

### 必须达成

- [ ] 所有 150+ 测试用例实现
- [ ] 所有测试通过
- [ ] 代码覆盖率 > 80%
- [ ] 无编译警告
- [ ] 无 clippy 警告
- [ ] 所有边界情况覆盖
- [ ] 所有错误路径覆盖

### 质量指标

- [ ] 每个测试有清晰的文档
- [ ] 每个测试独立可运行
- [ ] 测试数据自动清理
- [ ] 无测试间依赖
- [ ] 测试执行时间 < 5 分钟

---

**创建时间**: 2026-03-15  
**预计完成**: 10-14 小时  
**标准**: DO-178C Level A
