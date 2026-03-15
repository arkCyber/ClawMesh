# ClawMesh Agent 系统 - 下一步行动指南
## 航空航天级别标准实施

**创建时间**: 2026-03-15 12:05  
**当前状态**: 代码完成 70%，等待数据库迁移和测试实现  
**标准**: DO-178C Level A

---

## 🎯 当前状态总结

### ✅ 已完成的工作

**1. 代码实现 (100%)**
- ✅ 声誉系统 - 4 个文件, ~900 行代码
- ✅ 技能系统 - 6 个文件, ~2,100 行代码
- ✅ API 接口 - 2 个文件, ~650 行代码
- ✅ 数据库迁移脚本 - 4 个文件

**2. Schema 集成 (100%)**
- ✅ 在 `db_schema_file/src/schema.rs` 中添加 5 个新表
- ✅ 配置所有索引和外键
- ✅ 删除重复的 table! 定义

**3. 依赖修复 (100%)**
- ✅ 修复所有路径错误
- ✅ 统一 diesel 版本 (workspace)
- ✅ 统一 diesel-async 版本 (0.7.4)
- ✅ 添加模块到 workspace

**4. 文档 (100%)**
- ✅ 数据库迁移指南
- ✅ 航空航天级别实施计划
- ✅ 准确完成度报告
- ✅ Moltbook 对比分析

### ⏳ 待完成的工作 (30%)

**1. 数据库迁移 (0%)**
- ⏳ 运行迁移脚本
- ⏳ 验证表创建
- ⏳ 初始化数据

**2. 测试实现 (30%)**
- ⏳ 声誉系统测试 (60+ 用例)
- ⏳ 技能系统测试 (90+ 用例)
- ⏳ 集成测试

**3. 功能验证 (0%)**
- ⏳ 编译验证
- ⏳ 端到端测试
- ⏳ 性能测试

---

## 📋 立即执行清单 (本周)

### 步骤 1: 运行数据库迁移 ⏰ 30 分钟

```bash
# 1. 备份数据库
cd /Users/arksong/ClawMeet-Lemmy
pg_dump -U postgres lemmy > backup_$(date +%Y%m%d_%H%M%S).sql

# 2. 运行迁移
diesel migration run

# 3. 验证表创建
psql -U postgres -d lemmy -c "
SELECT tablename 
FROM pg_tables 
WHERE tablename LIKE 'agent_%'
ORDER BY tablename;
"

# 4. 初始化现有 Agent 的声誉
psql -U postgres -d lemmy << 'EOF'
INSERT INTO agent_reputation (agent_id, reputation_score, total_votes, positive_votes, negative_votes, reputation_level)
SELECT id, 500, 0, 0, 0, 1
FROM person
WHERE user_type = 'agent'
ON CONFLICT (agent_id) DO NOTHING;
EOF

# 5. 验证数据
psql -U postgres -d lemmy -c "SELECT COUNT(*) FROM agent_reputation;"
```

**验证清单**:
- [ ] 5 个新表已创建
- [ ] 10+ 个索引已创建
- [ ] 触发器正常工作
- [ ] 现有 Agent 已初始化声誉

---

### 步骤 2: 解决编译问题 ⏰ 1-2 小时

```bash
# 1. 清理构建缓存
cargo clean

# 2. 更新依赖
cargo update

# 3. 检查 reputation 模块
cargo check --package clawmesh_reputation

# 4. 检查 skills 模块
cargo check --package clawmesh_skills

# 5. 检查 API 模块
cargo check --package clawmesh_api

# 6. 编译所有模块
cargo build --all

# 7. 检查警告
cargo clippy --all
```

**预期结果**:
- [ ] 所有模块编译通过
- [ ] 无编译错误
- [ ] 无 clippy 警告

---

### 步骤 3: 实现声誉系统测试 ⏰ 4-6 小时

**文件位置**: `crates/clawmesh/api/tests/agent_reputation_tests.rs`

**测试框架已建立，需要实现具体测试代码**

#### 优先实现的测试 (前 20 个)

```rust
// 1. 基础查询测试 (5 个)
#[actix_web::test]
async fn test_get_reputation_success() {
    // 实现: 创建测试 Agent，获取声誉，验证响应
}

#[actix_web::test]
async fn test_get_reputation_invalid_id() {
    // 实现: 使用不存在的 ID，验证 404 错误
}

#[actix_web::test]
async fn test_get_reputation_non_agent() {
    // 实现: 使用普通用户 ID，验证 400 错误
}

#[actix_web::test]
async fn test_reputation_percentage_calculation() {
    // 实现: 验证声誉百分比计算正确
}

#[actix_web::test]
async fn test_reputation_level_display() {
    // 实现: 验证等级显示正确
}

// 2. 投票测试 (10 个)
#[actix_web::test]
async fn test_vote_upvote_success() {
    // 实现: 成功 upvote，验证分数增加 10
}

#[actix_web::test]
async fn test_vote_downvote_success() {
    // 实现: 成功 downvote，验证分数减少 10
}

#[actix_web::test]
async fn test_vote_self_voting_prevented() {
    // 实现: 尝试自投，验证被阻止
}

#[actix_web::test]
async fn test_vote_duplicate_within_24h() {
    // 实现: 24小时内重复投票，验证被阻止
}

#[actix_web::test]
async fn test_vote_reputation_level_change() {
    // 实现: 投票导致等级变化，验证等级更新
}

// ... 继续实现其他测试
```

**实现模板**:

```rust
use actix_web::{test, web, App};
use lemmy_api_utils::context::LemmyContext;
use serde_json::json;

async fn setup_test_context() -> LemmyContext {
    // 创建测试数据库连接
    // 创建测试 Agent
    // 返回 context
}

#[actix_web::test]
async fn test_get_reputation_success() {
    let context = setup_test_context().await;
    
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(context.clone()))
            .route("/api/v3/agent/{id}/reputation", 
                   web::get().to(get_reputation))
    ).await;
    
    let req = test::TestRequest::get()
        .uri("/api/v3/agent/1/reputation")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    assert!(resp.status().is_success());
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["agent_id"], 1);
    assert!(body["reputation_score"].is_number());
}
```

---

### 步骤 4: 实现技能系统测试 ⏰ 6-8 小时

**文件位置**: `crates/clawmesh/api/tests/agent_skills_tests.rs`

**测试框架已建立，需要实现具体测试代码**

#### 优先实现的测试 (前 30 个)

**A. 技能注册测试 (5 个)**
**B. 技能查询测试 (5 个)**
**C. 技能安装测试 (5 个)**
**D. 技能执行测试 (5 个)**
**E. 沙箱安全测试 (10 个)**

---

### 步骤 5: 运行所有测试 ⏰ 1 小时

```bash
# 1. 运行所有测试
cargo test --all

# 2. 运行声誉系统测试
cargo test --package clawmesh_api -- agent_reputation

# 3. 运行技能系统测试
cargo test --package clawmesh_api -- agent_skills

# 4. 生成测试报告
cargo test --all -- --nocapture > test_report.txt

# 5. 代码覆盖率
cargo tarpaulin --all --out Html --output-dir coverage

# 6. 查看覆盖率报告
open coverage/index.html
```

**验证清单**:
- [ ] 所有测试通过 (150+ 个)
- [ ] 代码覆盖率 > 80%
- [ ] 无测试失败
- [ ] 无测试超时

---

### 步骤 6: 端到端验证 ⏰ 2 小时

```bash
# 1. 启动服务器
cargo run --bin lemmy_server

# 2. 测试声誉 API
curl http://localhost:8080/api/v3/agent/1/reputation

# 3. 测试投票 API
curl -X POST http://localhost:8080/api/v3/agent/1/reputation/vote \
  -H "Content-Type: application/json" \
  -d '{"vote_type": "upvote", "reason": "Great work!"}'

# 4. 测试技能注册 API
curl -X POST http://localhost:8080/api/v3/agent/1/skills \
  -H "Content-Type: application/json" \
  -d '{
    "skill_name": "test_skill",
    "skill_type": "custom",
    "version": "1.0.0",
    "is_public": true
  }'

# 5. 测试技能市场 API
curl http://localhost:8080/api/v3/agent/skills/marketplace

# 6. 性能测试
ab -n 1000 -c 10 http://localhost:8080/api/v3/agent/1/reputation
```

**验证清单**:
- [ ] 服务器正常启动
- [ ] 所有 API 端点可访问
- [ ] 响应时间 < 100ms
- [ ] 无内存泄漏
- [ ] 无错误日志

---

## 📅 时间规划

### 本周 (2026-03-15 至 2026-03-18)

**周五 (3/15)**:
- ✅ 完成代码审计和规划
- ⏳ 运行数据库迁移
- ⏳ 解决编译问题

**周六 (3/16)**:
- ⏳ 实现声誉系统测试 (60+ 个)
- ⏳ 运行测试验证

**周日 (3/17)**:
- ⏳ 实现技能系统测试 (90+ 个)
- ⏳ 运行测试验证

**周一 (3/18)**:
- ⏳ 端到端测试
- ⏳ 性能优化
- ⏳ 生成完成报告

---

## 🔧 故障排查指南

### 问题 1: 数据库迁移失败

**症状**: `diesel migration run` 报错

**解决方案**:
```bash
# 1. 检查数据库连接
psql -U postgres -d lemmy -c "SELECT version();"

# 2. 检查权限
psql -U postgres -d lemmy -c "SELECT current_user;"

# 3. 手动运行迁移
psql -U postgres -d lemmy -f migrations/2026-03-15-000001_create_agent_reputation/up.sql
```

### 问题 2: 编译失败

**症状**: `cargo build` 报错

**解决方案**:
```bash
# 1. 清理缓存
cargo clean
rm -rf target/

# 2. 更新依赖
cargo update

# 3. 检查 Cargo.lock
git checkout Cargo.lock

# 4. 重新编译
cargo build --all
```

### 问题 3: 测试失败

**症状**: `cargo test` 失败

**解决方案**:
```bash
# 1. 查看详细错误
cargo test -- --nocapture

# 2. 单独运行失败的测试
cargo test test_name -- --nocapture

# 3. 检查数据库状态
psql -U postgres -d lemmy -c "\dt agent_*"

# 4. 重置测试数据库
diesel database reset
diesel migration run
```

---

## 📊 成功标准

### 本周完成标准

**必须达成 (P0)**:
- [ ] 数据库迁移成功
- [ ] 所有代码编译通过
- [ ] 150+ 测试用例实现
- [ ] 测试通过率 100%
- [ ] 代码覆盖率 > 80%

**建议达成 (P1)**:
- [ ] 性能测试通过
- [ ] 无内存泄漏
- [ ] API 文档完整
- [ ] 部署指南完整

**可选达成 (P2)**:
- [ ] 压力测试通过
- [ ] 安全审计通过
- [ ] 用户手册完整

---

## 🎓 DO-178C Level A 检查清单

### 代码质量

- [ ] 所有函数有详细注释
- [ ] 所有错误有处理
- [ ] 所有 Result 有检查
- [ ] 无 unwrap() 和 expect()
- [ ] 所有输入有验证
- [ ] 所有边界有检查

### 测试覆盖

- [ ] 语句覆盖率 100%
- [ ] 分支覆盖率 100%
- [ ] MC/DC 覆盖率 100%
- [ ] 所有边界测试
- [ ] 所有错误路径测试

### 文档完整性

- [ ] API 文档完整
- [ ] 数据库文档完整
- [ ] 部署文档完整
- [ ] 测试文档完整
- [ ] 安全文档完整

### 审计追踪

- [ ] 所有关键操作记录日志
- [ ] 所有错误记录日志
- [ ] 所有安全事件记录
- [ ] 日志可追溯
- [ ] 日志不可篡改

---

## 📞 需要帮助？

### 遇到问题时

1. **查看文档**:
   - `DATABASE_MIGRATION_GUIDE.md`
   - `AEROSPACE_GRADE_IMPLEMENTATION_PLAN.md`
   - `FINAL_ACCURATE_COMPLETION_REPORT.md`

2. **检查日志**:
   ```bash
   tail -f logs/lemmy.log
   ```

3. **运行诊断**:
   ```bash
   cargo check --all
   cargo clippy --all
   cargo test --all
   ```

4. **查看测试输出**:
   ```bash
   cargo test -- --nocapture
   ```

---

## ✅ 完成后的下一步

### 下月任务 (P1)

1. **实现协作工作空间** (12-16 小时)
   - 数据模型设计
   - API 实现
   - 测试实现

2. **实现社交功能** (16-20 小时)
   - 帖子系统
   - 评论系统
   - 投票系统
   - 关注系统

### 本季度任务 (P2)

3. **实现交易市场** (20-30 小时)
   - 市场设计
   - 支付集成
   - 交易流程

---

**创建时间**: 2026-03-15 12:05  
**预计完成**: 2026-03-18  
**当前进度**: 70%  
**目标进度**: 100%

**立即开始**: 运行数据库迁移！

```bash
cd /Users/arksong/ClawMeet-Lemmy
diesel migration run
```
