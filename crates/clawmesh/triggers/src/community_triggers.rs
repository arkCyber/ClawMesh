/// 社区相关触发器

use anyhow::Result;
use diesel_async::AsyncPgConnection;
use lemmy_db_schema_file::PersonId;
use tracing::info;

/// 处理社区创建
pub async fn handle_community_created(
    creator_id: PersonId,
    initial_members: i32,
    conn: &mut AsyncPgConnection,
) -> Result<i32> {
    // 根据初始成员数计算奖励，最多200分
    let credit_change = (initial_members / 10).min(200);

    info!(
        "Triggering community creation credit update: person_id={}, members={}, credit={}",
        creator_id.0, initial_members, credit_change
    );

    clawmesh_credit::update_person_credit(
        creator_id,
        credit_change,
        &format!("Community created with {} members", initial_members),
        conn,
    )
    .await
}

/// 处理社区达到里程碑
pub async fn handle_community_milestone(
    creator_id: PersonId,
    milestone: i32,
    conn: &mut AsyncPgConnection,
) -> Result<i32> {
    let credit_change = match milestone {
        100 => 10,
        500 => 25,
        1000 => 50,
        5000 => 100,
        10000 => 200,
        _ => 0,
    };

    if credit_change > 0 {
        info!(
            "Triggering community milestone credit update: person_id={}, milestone={}, credit={}",
            creator_id.0, milestone, credit_change
        );

        clawmesh_credit::update_person_credit(
            creator_id,
            credit_change,
            &format!("Community reached {} members", milestone),
            conn,
        )
        .await
    } else {
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_milestone_rewards() {
        // 验证里程碑奖励
        let rewards = vec![
            (100, 10),
            (500, 25),
            (1000, 50),
            (5000, 100),
            (10000, 200),
        ];

        for (milestone, expected) in rewards {
            let actual = match milestone {
                100 => 10,
                500 => 25,
                1000 => 50,
                5000 => 100,
                10000 => 200,
                _ => 0,
            };
            assert_eq!(actual, expected);
        }
    }
}
