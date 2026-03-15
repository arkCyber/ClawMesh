/// Credit actions that affect a person's score
#[derive(Debug, Clone)]
pub enum CreditAction {
    PostUpvote,
    PostDownvote,
    CommentUpvote,
    CommentDownvote,
    DailyActive,
    CommunityCreated { members: i32 },
    Violation { severity: i32 },
}

/// Calculate credit change based on action
pub fn calculate_credit_change(action: &CreditAction) -> i32 {
    match action {
        CreditAction::PostUpvote => 2,
        CreditAction::PostDownvote => -3,
        CreditAction::CommentUpvote => 1,
        CreditAction::CommentDownvote => -2,
        CreditAction::DailyActive => 5,
        CreditAction::CommunityCreated { members } => (members / 10).min(200),
        CreditAction::Violation { severity } => -(severity * 100),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credit_calculations() {
        assert_eq!(calculate_credit_change(&CreditAction::PostUpvote), 2);
        assert_eq!(calculate_credit_change(&CreditAction::PostDownvote), -3);
        assert_eq!(calculate_credit_change(&CreditAction::CommentUpvote), 1);
        assert_eq!(calculate_credit_change(&CreditAction::CommentDownvote), -2);
        assert_eq!(calculate_credit_change(&CreditAction::DailyActive), 5);
        
        assert_eq!(
            calculate_credit_change(&CreditAction::CommunityCreated { members: 100 }),
            10
        );
        assert_eq!(
            calculate_credit_change(&CreditAction::CommunityCreated { members: 5000 }),
            200
        );
        
        assert_eq!(
            calculate_credit_change(&CreditAction::Violation { severity: 1 }),
            -100
        );
    }
}
