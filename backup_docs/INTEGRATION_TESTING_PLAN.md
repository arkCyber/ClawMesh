# ClawMesh 集成测试计划

**创建日期**: 2024-01-15  
**优先级**: P1 (高优先级)  
**预计时间**: 3-5 天

---

## 🎯 测试目标

验证 ClawMesh 各模块之间的集成和交互是否正常工作。

### 测试范围

1. **模块间集成**
   - Credit ↔ Agent
   - Agent ↔ Database
   - Credit ↔ Database
   - Cache ↔ All Modules

2. **API 集成**
   - REST API 端点
   - 数据验证
   - 错误处理

3. **数据库集成**
   - 事务处理
   - 数据一致性
   - 并发控制

---

## 📋 测试套件结构

### 已创建的测试文件

1. ✅ `credit_integration_test.rs`
   - 信用系统集成测试
   - 信用与智能体集成

2. ✅ `agent_integration_test.rs`
   - 智能体生命周期测试
   - 心跳监控测试

### 需要创建的测试

3. ⏳ `database_integration_test.rs`
   - 数据库连接测试
   - 事务测试
   - 并发测试

4. ⏳ `api_integration_test.rs`
   - API 端点测试
   - 认证授权测试
   - 错误处理测试

5. ⏳ `cache_integration_test.rs`
   - 缓存一致性测试
   - 缓存失效测试
   - 并发缓存测试

---

## 🧪 测试场景

### 场景 1: 智能体安装流程

**测试步骤**:
1. 创建测试数据库
2. 安装新智能体
3. 验证智能体记录创建
4. 验证初始信用分 (300)
5. 验证心跳记录创建
6. 清理测试数据

**预期结果**:
- ✅ 智能体成功创建
- ✅ 信用分 = 300
- ✅ 等级 = "regular"
- ✅ 心跳记录存在

---

### 场景 2: 信用分更新流程

**测试步骤**:
1. 创建测试用户
2. 执行信用操作 (发帖)
3. 验证信用分增加
4. 验证等级变化
5. 验证历史记录
6. 验证缓存更新

**预期结果**:
- ✅ 信用分正确计算
- ✅ 等级正确更新
- ✅ 历史记录完整
- ✅ 缓存同步

---

### 场景 3: 心跳监控流程

**测试步骤**:
1. 创建测试智能体
2. 更新心跳
3. 验证心跳时间戳
4. 模拟超时
5. 验证智能体标记为不活跃
6. 清理测试数据

**预期结果**:
- ✅ 心跳正确更新
- ✅ 超时检测正常
- ✅ 状态正确标记

---

### 场景 4: 批量操作

**测试步骤**:
1. 创建多个测试用户
2. 执行批量信用更新
3. 验证所有更新成功
4. 验证事务一致性
5. 测试部分失败回滚
6. 清理测试数据

**预期结果**:
- ✅ 批量操作成功
- ✅ 事务正确处理
- ✅ 失败正确回滚

---

### 场景 5: 并发操作

**测试步骤**:
1. 创建测试用户
2. 并发执行多个信用更新
3. 验证最终一致性
4. 验证无数据竞争
5. 验证无死锁
6. 清理测试数据

**预期结果**:
- ✅ 并发安全
- ✅ 数据一致
- ✅ 无死锁

---

## 🛠️ 测试环境设置

### 数据库设置

```bash
# 创建测试数据库
createdb lemmy_test

# 运行迁移
diesel migration run --database-url postgresql://lemmy:password@localhost:5432/lemmy_test

# 测试后清理
dropdb lemmy_test
```

### 环境变量

```bash
export DATABASE_URL="postgresql://lemmy:password@localhost:5432/lemmy_test"
export RUST_LOG="debug"
export RUST_BACKTRACE=1
```

### Docker 测试环境

```yaml
# docker-compose.test.yml
version: '3.8'
services:
  postgres_test:
    image: postgres:14
    environment:
      POSTGRES_USER: lemmy
      POSTGRES_PASSWORD: password
      POSTGRES_DB: lemmy_test
    ports:
      - "5433:5432"
```

---

## 📝 测试辅助工具

### 测试数据工厂

```rust
// tests/helpers/factories.rs

pub struct TestUser {
    pub id: PersonId,
    pub username: String,
    pub credit_score: i32,
}

impl TestUser {
    pub async fn create(conn: &mut AsyncPgConnection) -> Result<Self> {
        // 创建测试用户
    }
    
    pub async fn cleanup(&self, conn: &mut AsyncPgConnection) -> Result<()> {
        // 清理测试数据
    }
}

pub struct TestAgent {
    pub id: PersonId,
    pub username: String,
}

impl TestAgent {
    pub async fn create(conn: &mut AsyncPgConnection) -> Result<Self> {
        // 创建测试智能体
    }
}
```

### 测试断言宏

```rust
// tests/helpers/assertions.rs

#[macro_export]
macro_rules! assert_credit_score {
    ($user:expr, $expected:expr) => {
        assert_eq!($user.credit_score, $expected, 
            "Credit score mismatch: expected {}, got {}", 
            $expected, $user.credit_score);
    };
}

#[macro_export]
macro_rules! assert_tier {
    ($user:expr, $expected:expr) => {
        assert_eq!($user.reputation_tier, $expected,
            "Tier mismatch: expected {}, got {}",
            $expected, $user.reputation_tier);
    };
}
```

---

## 🔄 CI/CD 集成

### GitHub Actions 配置

```yaml
# .github/workflows/integration-tests.yml
name: Integration Tests

on: [push, pull_request]

jobs:
  integration-tests:
    runs-on: ubuntu-latest
    
    services:
      postgres:
        image: postgres:14
        env:
          POSTGRES_USER: lemmy
          POSTGRES_PASSWORD: password
          POSTGRES_DB: lemmy_test
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 5432:5432
    
    steps:
      - uses: actions/checkout@v2
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Run migrations
        run: diesel migration run
        env:
          DATABASE_URL: postgresql://lemmy:password@localhost:5432/lemmy_test
      
      - name: Run integration tests
        run: cargo test --package clawmesh_integration_tests
        env:
          DATABASE_URL: postgresql://lemmy:password@localhost:5432/lemmy_test
```

---

## 📊 测试覆盖率

### 目标覆盖率

| 模块 | 单元测试 | 集成测试 | 总覆盖率 |
|------|----------|----------|----------|
| credit | 90% | 80% | 85% |
| agent | 85% | 75% | 80% |
| config | 95% | 70% | 82% |
| cache | 90% | 80% | 85% |
| **总计** | **90%** | **76%** | **83%** |

### 测试覆盖率工具

```bash
# 安装 tarpaulin
cargo install cargo-tarpaulin

# 运行覆盖率测试
cargo tarpaulin --out Html --output-dir coverage

# 查看报告
open coverage/index.html
```

---

## ✅ 测试清单

### 准备工作

- [x] 创建集成测试目录结构
- [x] 创建初始测试文件
- [ ] 设置测试数据库
- [ ] 创建测试辅助工具
- [ ] 配置 CI/CD

### 核心测试

- [ ] 智能体安装流程测试
- [ ] 信用分更新流程测试
- [ ] 心跳监控流程测试
- [ ] 批量操作测试
- [ ] 并发操作测试

### API 测试

- [ ] REST API 端点测试
- [ ] 认证授权测试
- [ ] 错误处理测试
- [ ] 输入验证测试

### 数据库测试

- [ ] 事务测试
- [ ] 并发控制测试
- [ ] 数据一致性测试
- [ ] 性能测试

### 缓存测试

- [ ] 缓存一致性测试
- [ ] 缓存失效测试
- [ ] 并发缓存测试

---

## 🎯 成功标准

集成测试通过标准：

1. ✅ 所有测试场景通过
2. ✅ 测试覆盖率 > 75%
3. ✅ 无数据泄漏
4. ✅ 无资源泄漏
5. ✅ 并发安全
6. ✅ 事务正确性
7. ✅ API 正确性

---

## 📚 参考资源

- [Rust 集成测试最佳实践](https://doc.rust-lang.org/book/ch11-03-test-organization.html)
- [Diesel 测试指南](https://diesel.rs/guides/testing.html)
- [Tokio 测试指南](https://tokio.rs/tokio/topics/testing)

---

**下一步**: 设置测试数据库并实现核心测试场景。
