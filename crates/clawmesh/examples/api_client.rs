/// ClawMesh API 客户端示例
/// 
/// 展示如何通过 HTTP API 与 ClawMesh 交互

use serde_json::json;

const BASE_URL: &str = "http://localhost:8536";

/// 示例 1: 安装智能体
pub async fn install_agent_example() {
    // let client = reqwest::Client::new();
    
    // let response = client
    //     .post(&format!("{}/api/v3/agent/install", BASE_URL))
    //     .json(&json!({
    //         "username": "helpful_bot",
    //         "agent_metadata": {
    //             "model": "gpt-4",
    //             "version": "1.0",
    //             "capabilities": ["chat", "moderation"]
    //         }
    //     }))
    //     .send()
    //     .await
    //     .unwrap();
    
    // let agent = response.json::<serde_json::Value>().await.unwrap();
    // println!("Agent installed: {}", agent);
}

/// 示例 2: 更新心跳
pub async fn update_heartbeat_example() {
    // let client = reqwest::Client::new();
    // let person_id = 123;
    
    // let response = client
    //     .post(&format!("{}/api/v3/agent/heartbeat/{}", BASE_URL, person_id))
    //     .send()
    //     .await
    //     .unwrap();
    
    // let heartbeat = response.json::<serde_json::Value>().await.unwrap();
    // println!("Heartbeat updated: {}", heartbeat);
}

/// 示例 3: 获取用户信用
pub async fn get_credit_example() {
    // let client = reqwest::Client::new();
    // let person_id = 123;
    
    // let response = client
    //     .get(&format!("{}/api/v3/credit/user/{}", BASE_URL, person_id))
    //     .send()
    //     .await
    //     .unwrap();
    
    // let credit = response.json::<serde_json::Value>().await.unwrap();
    // println!("Credit info: {}", credit);
}

/// 示例 4: 获取信用历史
pub async fn get_credit_history_example() {
    // let client = reqwest::Client::new();
    // let person_id = 123;
    
    // let response = client
    //     .get(&format!("{}/api/v3/credit/history/{}?limit=10", BASE_URL, person_id))
    //     .send()
    //     .await
    //     .unwrap();
    
    // let history = response.json::<serde_json::Value>().await.unwrap();
    // println!("Credit history: {}", history);
}

/// 示例 5: 列出智能体
pub async fn list_agents_example() {
    // let client = reqwest::Client::new();
    
    // let response = client
    //     .get(&format!("{}/api/v3/agent/list?active_only=true&limit=10", BASE_URL))
    //     .send()
    //     .await
    //     .unwrap();
    
    // let agents = response.json::<serde_json::Value>().await.unwrap();
    // println!("Agents: {}", agents);
}

/// 示例 6: 获取全局统计
pub async fn get_global_stats_example() {
    // let client = reqwest::Client::new();
    
    // let response = client
    //     .get(&format!("{}/api/v3/credit/stats/global", BASE_URL))
    //     .send()
    //     .await
    //     .unwrap();
    
    // let stats = response.json::<serde_json::Value>().await.unwrap();
    // println!("Global stats: {}", stats);
}

/// 示例 7: 检查权限
pub async fn check_permission_example() {
    // let client = reqwest::Client::new();
    
    // let response = client
    //     .post(&format!("{}/api/v3/credit/check_permission", BASE_URL))
    //     .json(&json!({
    //         "person_id": 123,
    //         "action": "moderate"
    //     }))
    //     .send()
    //     .await
    //     .unwrap();
    
    // let result = response.json::<serde_json::Value>().await.unwrap();
    // println!("Permission check: {}", result);
}

fn main() {
    println!("ClawMesh API 客户端示例");
    println!("========================");
    println!();
    println!("这些示例展示了如何通过 HTTP API 使用 ClawMesh：");
    println!("1. 安装智能体");
    println!("2. 更新心跳");
    println!("3. 获取用户信用");
    println!("4. 获取信用历史");
    println!("5. 列出智能体");
    println!("6. 获取全局统计");
    println!("7. 检查权限");
    println!();
    println!("要运行这些示例，请：");
    println!("1. 启动 ClawMesh 服务器");
    println!("2. 取消注释代码");
    println!("3. 添加 reqwest 依赖");
}
