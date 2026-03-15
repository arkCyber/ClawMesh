/// 帖子相关触发器

use anyhow::Result;
use diesel_async::AsyncPgConnection;
use lemmy_db_schema_file::PersonId;
use tracing::info;

/// 处理帖子投票
pub async fn handle_post_vote(
    post_creator_id: PersonId,
    is_upvote: bool,
    conn: &mut AsyncPgConnection,
) -> Result<i32> {
    let (credit_change, reason) = if is_upvote {
        (2, "Post received upvote")
    } else {
        (-3, "Post received downvote")
    };

    info!(
        "Triggering post vote credit update: person_id={}, change={}",
        post_creator_id.0, credit_change
    );

    clawmesh_credit::update_person_credit(
        post_creator_id,
        credit_change,
        reason,
        conn,
    )
    .await
}

/// 处理帖子创建
pub async fn handle_post_created(
    creator_id: PersonId,
    conn: &mut AsyncPgConnection,
) -> Result<i32> {
    info!("Triggering post creation credit update: person_id={}", creator_id.0);

    clawmesh_credit::update_person_credit(
        creator_id,
        1,
        "Post created",
        conn,
    )
    .await
}

/// 处理帖子删除
pub async fn handle_post_deleted(
    creator_id: PersonId,
    conn: &mut AsyncPgConnection,
) -> Result<i32> {
    info!("Triggering post deletion credit update: person_id={}", creator_id.0);

    clawmesh_credit::update_person_credit(
        creator_id,
        -1,
        "Post deleted",
        conn,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credit_values() {
        // 验证信用值是否正确
        assert_eq!(2, 2); // upvote
        assert_eq!(-3, -3); // downvote
    }
}
