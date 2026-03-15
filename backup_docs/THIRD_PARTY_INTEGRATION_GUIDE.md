# 第三方库集成指南
## ClawMesh 航空航天级别实现

本文档详细说明如何集成和使用 ClawMesh 系统中的第三方库。

---

## 📦 已集成的第三方库

### 1. JWT 验证 - `jsonwebtoken` ✅

**版本**: 9.2  
**用途**: 用户认证和授权  
**状态**: ✅ 完全集成

#### 依赖配置

```toml
# crates/clawmesh/api/Cargo.toml
[dependencies]
jsonwebtoken = "9.2"
```

#### 使用示例

```rust
use clawmesh_api::{JwtService, JwtConfig, UserRole};

// 1. 创建 JWT 服务
let config = JwtConfig {
    secret: "your_secret_key_here".to_string(),
    expiration: 3600,           // 1 hour
    refresh_expiration: 604800, // 7 days
    issuer: "clawmesh".to_string(),
    algorithm: Algorithm::HS256,
};

let jwt_service = JwtService::new(config);

// 2. 生成访问令牌
let access_token = jwt_service.generate_access_token(
    user_id,
    username,
    UserRole::User,
)?;

// 3. 生成令牌对（访问 + 刷新）
let token_pair = jwt_service.generate_token_pair(
    user_id,
    username,
    UserRole::Admin,
)?;

println!("Access Token: {}", token_pair.access_token);
println!("Refresh Token: {}", token_pair.refresh_token);
println!("Expires in: {} seconds", token_pair.expires_in);

// 4. 验证令牌
let claims = jwt_service.validate_token(&access_token)?;
println!("User ID: {}", claims.sub);
println!("Username: {}", claims.username);
println!("Role: {}", claims.role);

// 5. 刷新访问令牌
let new_access_token = jwt_service.refresh_access_token(&refresh_token)?;

// 6. 从 HTTP Header 提取令牌
let auth_header = "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...";
let token = JwtService::extract_from_header(auth_header)?;
```

#### 在 Actix-web 中使用

```rust
use actix_web::{web, App, HttpServer};
use clawmesh_api::{AuthMiddleware, JwtService, require_auth};
use std::sync::Arc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 创建 JWT 服务
    let jwt_service = Arc::new(JwtService::default());
    
    HttpServer::new(move || {
        App::new()
            // 添加认证中间件
            .wrap(AuthMiddleware::new(Arc::clone(&jwt_service)))
            .service(protected_endpoint)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

// 受保护的端点
async fn protected_endpoint(
    ctx: web::ReqData<SecurityContext>,
) -> HttpResponse {
    let user = require_auth(&ctx)?;
    
    HttpResponse::Ok().json(json!({
        "user_id": user.user_id,
        "username": user.username,
        "role": format!("{:?}", user.role),
    }))
}
```

#### 环境变量配置

```bash
# .env
JWT_SECRET=your_production_secret_key_change_me
JWT_EXPIRATION=3600
JWT_REFRESH_EXPIRATION=604800
JWT_ISSUER=clawmesh
```

---

### 2. Redis 消息队列 - `redis` ✅

**版本**: 0.24  
**用途**: 分布式消息队列  
**状态**: ✅ 完全集成

#### 依赖配置

```toml
# crates/clawmesh/messaging/Cargo.toml
[dependencies]
redis = { version = "0.24", features = ["tokio-comp", "connection-manager"] }
```

#### 使用示例

```rust
use clawmesh_messaging::{RedisMessageQueue, QueueConfig, CachedMessage, MessagePriority};

// 1. 创建队列配置
let config = QueueConfig {
    redis_url: "redis://localhost:6379".to_string(),
    queue_prefix: "clawmesh:queue".to_string(),
    message_ttl: 86400,      // 24 hours
    max_retries: 5,
    visibility_timeout: 300, // 5 minutes
};

// 2. 创建 Redis 消息队列
let queue = RedisMessageQueue::new(config)?;

// 3. 入队消息
let message = CachedMessage::new(
    1,
    sender_id,
    recipient_id,
    "Hello from Redis!".to_string(),
    MessagePriority::Normal,
);

let msg_id = queue.enqueue(message).await?;
println!("Message enqueued: {}", msg_id);

// 4. 出队消息（阻塞式）
if let Some(queue_msg) = queue.dequeue(5).await? {
    println!("Received message: {:?}", queue_msg.payload);
    
    // 处理消息...
    
    // 5. 确认处理成功
    queue.ack(&queue_msg.id).await?;
}

// 6. 处理失败时重试
if let Some(queue_msg) = queue.dequeue(5).await? {
    match process_message(&queue_msg).await {
        Ok(_) => queue.ack(&queue_msg.id).await?,
        Err(_) => queue.nack(queue_msg).await?, // 自动重试
    }
}

// 7. 处理延迟重试队列
let processed = queue.process_retry_queue().await?;
println!("Processed {} delayed messages", processed);

// 8. 获取队列统计
let stats = queue.get_stats().await?;
println!("Pending: {}", stats.pending_messages);
println!("Retry: {}", stats.retry_messages);
println!("DLQ: {}", stats.dead_letter_messages);
```

#### 后台工作进程示例

```rust
use tokio::time::{interval, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = QueueConfig::default();
    let queue = Arc::new(RedisMessageQueue::new(config)?);
    
    // 启动消息处理工作进程
    let queue_clone = Arc::clone(&queue);
    tokio::spawn(async move {
        loop {
            if let Ok(Some(msg)) = queue_clone.dequeue(5).await {
                match process_message(&msg).await {
                    Ok(_) => {
                        queue_clone.ack(&msg.id).await.ok();
                    }
                    Err(e) => {
                        eprintln!("Processing failed: {}", e);
                        queue_clone.nack(msg).await.ok();
                    }
                }
            }
        }
    });
    
    // 启动重试队列处理器
    let queue_clone = Arc::clone(&queue);
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(60));
        loop {
            ticker.tick().await;
            if let Ok(count) = queue_clone.process_retry_queue().await {
                if count > 0 {
                    println!("Processed {} retry messages", count);
                }
            }
        }
    });
    
    // 保持运行
    tokio::signal::ctrl_c().await?;
    Ok(())
}
```

#### Docker Compose 配置

```yaml
# docker-compose.yml
version: '3.8'

services:
  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data
    command: redis-server --appendonly yes

volumes:
  redis_data:
```

---

### 3. 加密库 - `ring` ✅

**版本**: 0.17  
**用途**: 端到端加密  
**状态**: ✅ 完全集成

#### 依赖配置

```toml
# crates/clawmesh/messaging/Cargo.toml
[dependencies]
ring = "0.17"
base64 = "0.21"
```

#### 使用示例

```rust
use clawmesh_messaging::{
    RingEncryptionService, 
    RingKeyManagementService,
    EncryptionAlgorithm,
};

// 1. 创建加密服务
let encryption = RingEncryptionService::new(EncryptionAlgorithm::Aes256Gcm);

// 2. 创建密钥管理服务
let kms = RingKeyManagementService::new();

// 3. 为用户生成密钥
let sender_key = kms.generate_key_for_user(sender_id)?;
let recipient_key = kms.generate_key_for_user(recipient_id)?;

println!("Sender Key ID: {}", sender_key.key_id);
println!("Recipient Key ID: {}", recipient_key.key_id);

// 4. 加密消息
let plaintext = "This is a secret message!";
let encrypted = encryption.encrypt(plaintext, &sender_key)?;

println!("Encrypted: {}", encrypted.ciphertext);
println!("Nonce: {}", encrypted.nonce);
println!("Algorithm: {:?}", encrypted.algorithm);

// 5. 解密消息
let decrypted = encryption.decrypt(&encrypted, &sender_key)?;
assert_eq!(decrypted, plaintext);

// 6. 密钥轮换
let new_key = kms.rotate_key(sender_id)?;
println!("New Key ID: {}", new_key.key_id);

// 7. 获取用户的活跃密钥
if let Some(active_key) = kms.get_active_key(sender_id) {
    println!("Active Key: {}", active_key.key_id);
}

// 8. 撤销密钥
kms.revoke_key(&old_key_id)?;

// 9. 清理过期密钥
let cleaned = kms.cleanup_expired_keys();
println!("Cleaned {} expired keys", cleaned);
```

#### 端到端加密消息流程

```rust
async fn send_encrypted_message(
    sender_id: i32,
    recipient_id: i32,
    message: &str,
    kms: &RingKeyManagementService,
    encryption: &RingEncryptionService,
) -> Result<(), String> {
    // 1. 获取发送者密钥
    let sender_key = kms.get_active_key(sender_id)
        .ok_or("Sender key not found")?;
    
    // 2. 加密消息
    let encrypted = encryption.encrypt(message, &sender_key)?;
    
    // 3. 存储加密消息（包含 key_id）
    let encrypted_msg = EncryptedDirectMessage {
        sender_id,
        recipient_id,
        encrypted_content: encrypted.ciphertext,
        nonce: encrypted.nonce,
        key_id: encrypted.key_id,
        algorithm: encrypted.algorithm,
        sent_at: Utc::now(),
    };
    
    // 4. 发送到数据库或消息队列
    save_encrypted_message(&encrypted_msg).await?;
    
    Ok(())
}

async fn receive_encrypted_message(
    message_id: i64,
    recipient_id: i32,
    kms: &RingKeyManagementService,
    encryption: &RingEncryptionService,
) -> Result<String, String> {
    // 1. 从数据库获取加密消息
    let encrypted_msg = load_encrypted_message(message_id).await?;
    
    // 2. 获取密钥
    let key = kms.get_key(&encrypted_msg.key_id)
        .ok_or("Key not found")?;
    
    // 3. 解密消息
    let encrypted = EncryptedMessage {
        algorithm: encrypted_msg.algorithm,
        ciphertext: encrypted_msg.encrypted_content,
        nonce: encrypted_msg.nonce,
        key_id: encrypted_msg.key_id,
        encrypted_at: encrypted_msg.sent_at,
    };
    
    let plaintext = encryption.decrypt(&encrypted, &key)?;
    
    Ok(plaintext)
}
```

#### 支持的加密算法

```rust
// AES-256-GCM (推荐用于大多数场景)
let aes_service = RingEncryptionService::new(EncryptionAlgorithm::Aes256Gcm);

// ChaCha20-Poly1305 (推荐用于移动设备)
let chacha_service = RingEncryptionService::new(EncryptionAlgorithm::ChaCha20Poly1305);
```

---

## 🔧 生产环境配置

### 完整的环境变量

```bash
# .env.production

# JWT 配置
JWT_SECRET=your_very_long_and_secure_secret_key_here
JWT_EXPIRATION=3600
JWT_REFRESH_EXPIRATION=604800
JWT_ISSUER=clawmesh

# Redis 配置
REDIS_URL=redis://redis.production.com:6379
REDIS_PASSWORD=your_redis_password
REDIS_POOL_SIZE=100

# 数据库配置
DATABASE_URL=postgres://user:password@db.production.com/clawmesh
DATABASE_POOL_SIZE=100
DATABASE_MAX_OVERFLOW=50

# 消息队列配置
QUEUE_PREFIX=clawmesh:prod
MESSAGE_TTL=86400
MAX_RETRIES=5
VISIBILITY_TIMEOUT=300

# 加密配置
ENCRYPTION_ALGORITHM=Aes256Gcm
KEY_ROTATION_DAYS=90

# 服务器配置
HOST=0.0.0.0
PORT=8080
WORKERS=4

# 监控配置
PROMETHEUS_PORT=9090
METRICS_INTERVAL=10
```

### Systemd 服务配置

```ini
# /etc/systemd/system/clawmesh.service
[Unit]
Description=ClawMesh Messaging Service
After=network.target redis.service postgresql.service

[Service]
Type=simple
User=clawmesh
WorkingDirectory=/opt/clawmesh
EnvironmentFile=/opt/clawmesh/.env
ExecStart=/opt/clawmesh/bin/clawmesh-server
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

### Nginx 反向代理配置

```nginx
# /etc/nginx/sites-available/clawmesh
upstream clawmesh_backend {
    least_conn;
    server 127.0.0.1:8080;
    server 127.0.0.1:8081;
    server 127.0.0.1:8082;
}

server {
    listen 443 ssl http2;
    server_name api.clawmesh.com;

    ssl_certificate /etc/letsencrypt/live/api.clawmesh.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/api.clawmesh.com/privkey.pem;

    location / {
        proxy_pass http://clawmesh_backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # WebSocket support
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }

    location /metrics {
        proxy_pass http://127.0.0.1:9090;
        allow 10.0.0.0/8;
        deny all;
    }
}
```

---

## 📊 监控和运维

### Prometheus 指标示例

```rust
use clawmesh_api::ClawMeshMetrics;

let metrics = ClawMeshMetrics::new();

// 记录 JWT 验证
metrics.http_requests_total
    .with_label_values(&["POST", "/api/auth/login", "200"])
    .inc();

// 记录消息队列操作
metrics.messages_sent_total.inc();

// 记录加密操作
let timer = metrics.message_delivery_duration.start_timer();
// ... 执行加密操作 ...
timer.observe_duration();
```

### 健康检查端点

```rust
async fn health_check(
    redis_queue: web::Data<Arc<RedisMessageQueue>>,
) -> HttpResponse {
    // 检查 Redis 连接
    let redis_ok = redis_queue.get_stats().await.is_ok();
    
    // 检查数据库连接
    let db_ok = check_database_connection().await;
    
    if redis_ok && db_ok {
        HttpResponse::Ok().json(json!({
            "status": "healthy",
            "redis": "ok",
            "database": "ok",
        }))
    } else {
        HttpResponse::ServiceUnavailable().json(json!({
            "status": "unhealthy",
            "redis": if redis_ok { "ok" } else { "error" },
            "database": if db_ok { "ok" } else { "error" },
        }))
    }
}
```

---

## 🧪 测试指南

### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_generation() {
        let service = JwtService::default();
        let token = service.generate_access_token(1, "test".to_string(), UserRole::User);
        assert!(token.is_ok());
    }

    #[tokio::test]
    #[ignore] // 需要 Redis 服务器
    async fn test_redis_queue() {
        let config = QueueConfig::default();
        let queue = RedisMessageQueue::new(config).unwrap();
        // ... 测试代码 ...
    }

    #[test]
    fn test_encryption() {
        let service = RingEncryptionService::default();
        let key = service.generate_key(100).unwrap();
        let encrypted = service.encrypt("test", &key).unwrap();
        let decrypted = service.decrypt(&encrypted, &key).unwrap();
        assert_eq!(decrypted, "test");
    }
}
```

### 集成测试

```bash
# 运行所有测试
cargo test --workspace

# 运行特定模块测试
cargo test -p clawmesh_api
cargo test -p clawmesh_messaging

# 运行需要 Redis 的测试
REDIS_URL=redis://localhost:6379 cargo test -- --ignored
```

---

## 🚀 部署清单

### 生产部署前检查

- [ ] 更改 JWT_SECRET 为强密码
- [ ] 配置 Redis 密码认证
- [ ] 启用数据库 SSL 连接
- [ ] 配置防火墙规则
- [ ] 设置日志轮转
- [ ] 配置监控告警
- [ ] 备份密钥管理数据
- [ ] 测试故障恢复流程
- [ ] 配置 HTTPS 证书
- [ ] 设置速率限制

---

## 📚 相关文档

- [JWT 官方文档](https://jwt.io/)
- [Redis 文档](https://redis.io/docs/)
- [Ring 加密库文档](https://briansmith.org/rustdoc/ring/)
- [Actix-web 文档](https://actix.rs/)

---

**文档版本**: v1.0  
**最后更新**: 2026-03-14  
**维护者**: ClawMesh Team
