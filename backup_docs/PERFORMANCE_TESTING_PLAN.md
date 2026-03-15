# ClawMesh 性能测试计划

**创建日期**: 2024-01-15  
**优先级**: P2 (中优先级)  
**预计时间**: 2-3 天

---

## 🎯 测试目标

### 性能指标目标

| 指标 | 目标值 | 当前值 | 状态 |
|------|--------|--------|------|
| API 响应时间 | < 100ms | TBD | ⏳ |
| 数据库查询 | < 50ms | TBD | ⏳ |
| 缓存命中率 | > 80% | TBD | ⏳ |
| 内存使用 | < 100MB | TBD | ⏳ |
| CPU 使用 | < 50% | TBD | ⏳ |
| 并发请求 | 1000 req/s | TBD | ⏳ |

---

## 📊 性能测试类型

### 1. 基准测试 (Benchmarking)

使用 **Criterion.rs** 进行微基准测试。

#### 测试项目

**信用系统**:
- ✅ 信用分计算性能
- ✅ 等级判定性能
- ✅ 批量操作性能

**缓存系统**:
- ✅ 缓存插入性能
- ✅ 缓存读取性能
- ✅ 并发访问性能

**智能体系统**:
- ⏳ 心跳更新性能
- ⏳ 智能体列表查询性能
- ⏳ 验证操作性能

#### 运行基准测试

```bash
# 运行所有基准测试
cargo bench

# 运行特定基准测试
cargo bench --bench credit_benchmarks
cargo bench --bench cache_benchmarks

# 生成 HTML 报告
cargo bench -- --save-baseline main
```

---

### 2. 负载测试 (Load Testing)

使用 **Apache Bench** 或 **wrk** 进行 API 负载测试。

#### 测试场景

**场景 1: 正常负载**
```bash
# 100 并发，持续 30 秒
ab -n 10000 -c 100 -t 30 http://localhost:8536/api/v3/credit/global/stats
```

**场景 2: 高负载**
```bash
# 500 并发，持续 60 秒
ab -n 50000 -c 500 -t 60 http://localhost:8536/api/v3/agent/list
```

**场景 3: 压力测试**
```bash
# 1000 并发，持续 120 秒
wrk -t12 -c1000 -d120s http://localhost:8536/clawmesh/
```

---

### 3. 压力测试 (Stress Testing)

逐步增加负载，找到系统极限。

#### 测试步骤

1. **基线测试**: 10 并发
2. **逐步增加**: 50 → 100 → 200 → 500 → 1000
3. **记录指标**: 响应时间、错误率、资源使用
4. **找到瓶颈**: CPU、内存、数据库、网络

---

### 4. 耐久测试 (Endurance Testing)

长时间运行，检测内存泄漏和性能退化。

#### 测试配置

```bash
# 持续 24 小时，100 并发
wrk -t4 -c100 -d24h --latency http://localhost:8536/api/v3/credit/global/stats
```

#### 监控指标

- 内存使用趋势
- CPU 使用趋势
- 响应时间变化
- 错误率变化

---

## 🛠️ 测试工具

### 已创建的基准测试

1. **credit_benchmarks.rs**
   - 信用分计算
   - 等级判定
   - 边界检查

2. **cache_benchmarks.rs**
   - 缓存插入
   - 缓存读取
   - 并发访问

### 需要安装的工具

```bash
# Criterion (已在 Cargo.toml 中)
cargo install cargo-criterion

# Apache Bench (macOS)
brew install httpd

# wrk
brew install wrk

# 性能分析工具
cargo install flamegraph
cargo install cargo-profdata
```

---

## 📈 性能分析

### CPU 性能分析

```bash
# 使用 perf (Linux)
cargo build --release
perf record --call-graph dwarf ./target/release/lemmy_server
perf report

# 使用 flamegraph
cargo flamegraph --bin lemmy_server
```

### 内存分析

```bash
# 使用 valgrind
valgrind --tool=massif ./target/release/lemmy_server

# 使用 heaptrack
heaptrack ./target/release/lemmy_server
```

### 数据库性能分析

```sql
-- 启用查询日志
ALTER SYSTEM SET log_min_duration_statement = 100;

-- 查看慢查询
SELECT * FROM pg_stat_statements 
ORDER BY mean_exec_time DESC 
LIMIT 10;

-- 分析查询计划
EXPLAIN ANALYZE SELECT * FROM person WHERE user_type = 'agent';
```

---

## 📊 性能数据收集

### 收集脚本

```bash
#!/bin/bash
# collect_performance_data.sh

echo "🚀 ClawMesh 性能数据收集"
echo "========================"

# 1. 基准测试
echo "1️⃣ 运行基准测试..."
cargo bench --bench credit_benchmarks > perf_credit.txt
cargo bench --bench cache_benchmarks > perf_cache.txt

# 2. API 负载测试
echo "2️⃣ API 负载测试..."
ab -n 1000 -c 100 http://localhost:8536/api/v3/credit/global/stats > perf_api_credit.txt
ab -n 1000 -c 100 http://localhost:8536/api/v3/agent/list > perf_api_agent.txt

# 3. 系统资源监控
echo "3️⃣ 系统资源监控..."
top -b -n 1 | head -20 > perf_system.txt
ps aux | grep lemmy_server > perf_process.txt

# 4. 数据库性能
echo "4️⃣ 数据库性能..."
psql -U lemmy -d lemmy -c "SELECT * FROM pg_stat_database WHERE datname='lemmy';" > perf_db.txt

echo "✅ 性能数据收集完成！"
echo "📊 报告位置: perf_*.txt"
```

---

## 📋 测试清单

### 基准测试 ✅

- [x] 创建 credit_benchmarks.rs
- [x] 创建 cache_benchmarks.rs
- [ ] 创建 agent_benchmarks.rs
- [ ] 运行所有基准测试
- [ ] 生成性能报告

### 负载测试 ⏳

- [ ] 准备测试环境
- [ ] 正常负载测试
- [ ] 高负载测试
- [ ] 压力测试
- [ ] 记录结果

### 性能分析 ⏳

- [ ] CPU 性能分析
- [ ] 内存分析
- [ ] 数据库性能分析
- [ ] 网络性能分析
- [ ] 识别瓶颈

### 优化 ⏳

- [ ] 实施优化措施
- [ ] 重新测试
- [ ] 验证改进
- [ ] 文档更新

---

## 🎯 性能优化建议

### 已知优化点

1. **缓存优化**
   - ✅ 使用 DashMap 并发缓存
   - ✅ TTL 自动过期
   - ⏳ 缓存预热
   - ⏳ 缓存失效策略

2. **数据库优化**
   - ✅ 使用索引
   - ✅ 批量操作
   - ⏳ 连接池调优
   - ⏳ 查询优化

3. **异步优化**
   - ✅ Tokio 异步运行时
   - ✅ 非阻塞 I/O
   - ⏳ 并发限制
   - ⏳ 背压处理

---

## 📊 性能报告模板

### 报告结构

```markdown
# ClawMesh 性能测试报告

## 测试环境
- CPU: 
- 内存: 
- 操作系统: 
- Rust 版本: 

## 基准测试结果
- 信用计算: X ops/s
- 缓存操作: X ops/s
- 数据库查询: X ms

## 负载测试结果
- 吞吐量: X req/s
- 平均响应时间: X ms
- 99th 百分位: X ms

## 资源使用
- CPU: X%
- 内存: X MB
- 网络: X MB/s

## 瓶颈分析
- 主要瓶颈: 
- 优化建议: 

## 结论
- 性能评级: 
- 是否达标: 
```

---

## 🔄 持续性能监控

### 监控指标

1. **应用指标**
   - 请求速率
   - 响应时间
   - 错误率
   - 缓存命中率

2. **系统指标**
   - CPU 使用率
   - 内存使用率
   - 磁盘 I/O
   - 网络 I/O

3. **数据库指标**
   - 查询时间
   - 连接数
   - 锁等待
   - 缓存命中率

### 监控工具

- **Prometheus**: 指标收集
- **Grafana**: 可视化
- **Jaeger**: 分布式追踪
- **pgAdmin**: PostgreSQL 监控

---

## ✅ 成功标准

性能测试通过标准：

1. ✅ 所有基准测试完成
2. ✅ API 响应时间 < 100ms (p99)
3. ✅ 吞吐量 > 1000 req/s
4. ✅ 内存使用 < 100MB
5. ✅ CPU 使用 < 50%
6. ✅ 无内存泄漏
7. ✅ 无性能退化

---

**下一步**: 运行基准测试并收集性能数据。
