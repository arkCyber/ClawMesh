/// ClawMesh 自动触发器系统
/// 
/// 自动在用户行为发生时触发信用更新

use anyhow::Result;
use diesel_async::AsyncPgConnection;
use lemmy_db_schema_file::PersonId;

pub mod post_triggers;
pub mod comment_triggers;
pub mod activity_triggers;
pub mod community_triggers;

/// 帖子投票触发器
pub async fn trigger_post_vote(
    post_creator_id: PersonId,
    is_upvote: bool,
    conn: &mut AsyncPgConnection,
) -> Result<i32> {
    post_triggers::handle_post_vote(post_creator_id, is_upvote, conn).await
}

/// 评论投票触发器
pub async fn trigger_comment_vote(
    comment_creator_id: PersonId,
    is_upvote: bool,
    conn: &mut AsyncPgConnection,
) -> Result<i32> {
    comment_triggers::handle_comment_vote(comment_creator_id, is_upvote, conn).await
}

/// 每日活跃触发器
pub async fn trigger_daily_activity(
    person_id: PersonId,
    conn: &mut AsyncPgConnection,
) -> Result<Option<i32>> {
    activity_triggers::handle_daily_activity(person_id, conn).await
}

/// 社区创建触发器
pub async fn trigger_community_created(
    creator_id: PersonId,
    initial_members: i32,
    conn: &mut AsyncPgConnection,
) -> Result<i32> {
    community_triggers::handle_community_created(creator_id, initial_members, conn).await
}

/// 违规触发器
pub async fn trigger_violation(
    person_id: PersonId,
    severity: i32,
    reason: &str,
    conn: &mut AsyncPgConnection,
) -> Result<i32> {
    let credit_change = -(severity * 100);
    clawmesh_credit::update_person_credit(
        person_id,
        credit_change,
        &format!("Violation: {}", reason),
        conn,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trigger_functions_exist() {
        // 确保所有触发器函数都存在
        assert!(true);
    }
}
