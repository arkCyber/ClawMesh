/// 活跃度相关触发器

use anyhow::Result;
use chrono::{NaiveDate, Utc};
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use lemmy_db_schema_file::PersonId;
use tracing::info;

/// 处理每日活跃
/// 
/// 只有当用户今天还没有获得每日活跃奖励时才会更新
pub async fn handle_daily_activity(
    person_id: PersonId,
    conn: &mut AsyncPgConnection,
) -> Result<Option<i32>> {
    use lemmy_db_schema_file::schema::credit_history;

    let today = Utc::now().date_naive();

    // 检查今天是否已经获得过每日活跃奖励
    let has_activity_today: bool = credit_history::table
        .filter(credit_history::person_id.eq(person_id))
        .filter(credit_history::reason.eq("Daily active"))
        .filter(credit_history::created_at.ge(today.and_hms_opt(0, 0, 0).unwrap()))
        .select(diesel::dsl::count(credit_history::id))
        .first::<i64>(conn)
        .await?
        > 0;

    if has_activity_today {
        info!("Person {} already received daily activity credit today", person_id.0);
        return Ok(None);
    }

    info!("Triggering daily activity credit update: person_id={}", person_id.0);

    let new_credit = clawmesh_credit::update_person_credit(
        person_id,
        5,
        "Daily active",
        conn,
    )
    .await?;

    Ok(Some(new_credit))
}

/// 处理连续活跃奖励
pub async fn handle_streak_bonus(
    person_id: PersonId,
    streak_days: i32,
    conn: &mut AsyncPgConnection,
) -> Result<i32> {
    let bonus = (streak_days / 7).min(10); // 每周额外奖励，最多10分
    
    if bonus > 0 {
        info!(
            "Triggering streak bonus: person_id={}, days={}, bonus={}",
            person_id.0, streak_days, bonus
        );

        clawmesh_credit::update_person_credit(
            person_id,
            bonus,
            &format!("{} days streak bonus", streak_days),
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
    fn test_streak_bonus_calculation() {
        // 7天 = 1分
        assert_eq!((7 / 7).min(10), 1);
        // 14天 = 2分
        assert_eq!((14 / 7).min(10), 2);
        // 100天 = 10分（上限）
        assert_eq!((100 / 7).min(10), 10);
    }
}
