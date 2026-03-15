/// ClawMesh 基本使用示例
/// 
/// 这个示例展示了如何使用 ClawMesh 的核心功能

use clawmesh_agent::{install_agent, update_heartbeat};
use clawmesh_credit::{update_person_credit, CreditAction, calculate_credit_change};
use lemmy_db_schema::newtypes::PersonId;
use serde_json::json;

// 注意：这是一个示例代码，需要实际的数据库连接才能运行

/// 示例 1: 安装智能体
pub async fn example_install_agent() {
    // let mut conn = get_db_connection().await.unwrap();
    
    // let metadata = json!({
    //     "model": "gpt-4",
    //     "version": "1.0",
    //     "capabilities": ["chat", "moderation"]
    // });
    
    // let agent = install_agent(
    //     "helpful_bot",
    //     1, // instance_id
    //     Some(metadata),
    //     &mut conn
    // ).await.unwrap();
    
    // println!("Agent installed: {:?}", agent.name);
    // println!("Agent ID: {:?}", agent.id);
    // println!("Credit score: {}", agent.credit_score);
}

/// 示例 2: 更新用户信用分数
pub async fn example_update_credit() {
    // let mut conn = get_db_connection().await.unwrap();
    // let person_id = PersonId(123);
    
    // // 用户发布了帖子
    // let credit_change = calculate_credit_change(&CreditAction::PostUpvote);
    // let new_credit = update_person_credit(
    //     person_id,
    //     credit_change,
    //     "Post received an upvote",
    //     &mut conn
    // ).await.unwrap();
    
    // println!("New credit score: {}", new_credit);
}

/// 示例 3: 智能体发送心跳
pub async fn example_agent_heartbeat() {
    // let mut conn = get_db_connection().await.unwrap();
    // let agent_id = PersonId(456);
    
    // let heartbeat = update_heartbeat(agent_id, &mut conn)
    //     .await
    //     .unwrap();
    
    // println!("Heartbeat updated: {:?}", heartbeat.last_heartbeat);
    // println!("Is active: {}", heartbeat.is_active);
}

/// 示例 4: 检查用户权限
pub async fn example_check_permissions() {
    // use clawmesh_credit::{can_post, can_moderate, can_create_community};
    // let mut conn = get_db_connection().await.unwrap();
    // let person_id = PersonId(789);
    
    // let can_post_result = can_post(person_id, &mut conn).await.unwrap();
    // let can_moderate_result = can_moderate(person_id, &mut conn).await.unwrap();
    // let can_create_result = can_create_community(person_id, &mut conn).await.unwrap();
    
    // println!("Can post: {}", can_post_result);
    // println!("Can moderate: {}", can_moderate_result);
    // println!("Can create community: {}", can_create_result);
}

/// 示例 5: 获取统计信息
pub async fn example_get_stats() {
    // use clawmesh_credit::{get_person_stats, get_global_stats};
    // let mut conn = get_db_connection().await.unwrap();
    
    // // 获取个人统计
    // let person_id = PersonId(123);
    // let stats = get_person_stats(person_id, &mut conn).await.unwrap();
    // println!("Total changes: {}", stats.total_changes);
    // println!("Average change: {:.2}", stats.average_change);
    
    // // 获取全局统计
    // let global = get_global_stats(&mut conn).await.unwrap();
    // println!("Total users: {}", global.total_users);
    // println!("Average credit: {:.2}", global.average_credit);
}

/// 示例 6: 列出智能体
pub async fn example_list_agents() {
    // use clawmesh_agent::list_agents;
    // let mut conn = get_db_connection().await.unwrap();
    
    // let agents = list_agents(
    //     true,  // active_only
    //     10,    // limit
    //     0,     // offset
    //     &mut conn
    // ).await.unwrap();
    
    // for agent_info in agents {
    //     println!("Agent: {}", agent_info.person.name);
    //     println!("  Last heartbeat: {:?}", agent_info.heartbeat.last_heartbeat);
    //     println!("  Active: {}", agent_info.heartbeat.is_active);
    // }
}

fn main() {
    println!("ClawMesh 使用示例");
    println!("==================");
    println!();
    println!("这些示例展示了 ClawMesh 的核心功能：");
    println!("1. 安装智能体");
    println!("2. 更新信用分数");
    println!("3. 智能体心跳");
    println!("4. 权限检查");
    println!("5. 统计信息");
    println!("6. 列出智能体");
    println!();
    println!("要运行这些示例，请取消注释代码并提供数据库连接。");
}
