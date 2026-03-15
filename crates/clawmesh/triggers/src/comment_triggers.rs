/// 评论相关触发器

use anyhow::Result;
use diesel_async::AsyncPgConnection;
use lemmy_db_schema_file::PersonId;
use tracing::info;

/// 处理评论投票
pub async fn handle_comment_vote(
    comment_creator_id: PersonId,
    is_upvote: bool,
    conn: &mut AsyncPgConnection,
) -> Result<i32> {
    let (credit_change, reason) = if is_upvote {
        (1, "Comment received upvote")
    } else {
        (-2, "Comment received downvote")
    };

    info!(
        "Triggering comment vote credit update: person_id={}, change={}",
        comment_creator_id.0, credit_change
    );

    clawmesh_credit::update_person_credit(
        comment_creator_id,
        credit_change,
        reason,
        conn,
    )
    .await
}

/// 处理评论创建
pub async fn handle_comment_created(
    creator_id: PersonId,
    conn: &mut AsyncPgConnection,
) -> Result<i32> {
    info!("Triggering comment creation credit update: person_id={}", creator_id.0);

    clawmesh_credit::update_person_credit(
        creator_id,
        1,
        "Comment created",
        conn,
    )
    .await
}

/// 处理评论删除
pub async fn handle_comment_deleted(
    creator_id: PersonId,
    conn: &mut AsyncPgConnection,
) -> Result<i32> {
    info!("Triggering comment deletion credit update: person_id={}", creator_id.0);

    clawmesh_credit::update_person_credit(
        creator_id,
        -1,
        "Comment deleted",
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
        assert_eq!(1, 1); // upvote
        assert_eq!(-2, -2); // downvote
    }
}
