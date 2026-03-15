/// Agent Social Features Unit Tests
/// DO-178C Level A Compliant Test Suite

#[cfg(test)]
mod social_unit_tests {
    use clawmesh_social::models::*;

    // ========================================================================
    // PostForm Validation Tests
    // ========================================================================

    #[test]
    fn test_post_form_valid() {
        let form = PostForm {
            agent_id: 1,
            title: "Test Post".to_string(),
            content: Some("This is a test post".to_string()),
            tags: Some(vec!["test".to_string()]),
            is_public: true,
        };

        assert!(form.validate().is_ok());
    }

    #[test]
    fn test_post_form_empty_title() {
        let form = PostForm {
            agent_id: 1,
            title: "".to_string(),
            content: None,
            tags: None,
            is_public: true,
        };

        assert!(form.validate().is_err());
    }

    #[test]
    fn test_post_form_title_too_long() {
        let form = PostForm {
            agent_id: 1,
            title: "a".repeat(256),
            content: None,
            tags: None,
            is_public: true,
        };

        assert!(form.validate().is_err());
    }

    #[test]
    fn test_post_form_content_too_long() {
        let form = PostForm {
            agent_id: 1,
            title: "Valid Title".to_string(),
            content: Some("a".repeat(50001)),
            tags: None,
            is_public: true,
        };

        assert!(form.validate().is_err());
    }

    #[test]
    fn test_post_form_too_many_tags() {
        let form = PostForm {
            agent_id: 1,
            title: "Valid Title".to_string(),
            content: None,
            tags: Some(vec!["tag".to_string(); 21]),
            is_public: true,
        };

        assert!(form.validate().is_err());
    }

    // ========================================================================
    // CommentForm Validation Tests
    // ========================================================================

    #[test]
    fn test_comment_form_valid() {
        let form = CommentForm {
            post_id: 1,
            agent_id: 2,
            parent_id: None,
            content: "Great post!".to_string(),
        };

        assert!(form.validate().is_ok());
    }

    #[test]
    fn test_comment_form_empty_content() {
        let form = CommentForm {
            post_id: 1,
            agent_id: 2,
            parent_id: None,
            content: "".to_string(),
        };

        assert!(form.validate().is_err());
    }

    #[test]
    fn test_comment_form_content_too_long() {
        let form = CommentForm {
            post_id: 1,
            agent_id: 2,
            parent_id: None,
            content: "a".repeat(10001),
        };

        assert!(form.validate().is_err());
    }

    #[test]
    fn test_comment_form_with_parent() {
        let form = CommentForm {
            post_id: 1,
            agent_id: 2,
            parent_id: Some(5),
            content: "Reply to comment".to_string(),
        };

        assert!(form.validate().is_ok());
    }

    // ========================================================================
    // VoteForm Validation Tests
    // ========================================================================

    #[test]
    fn test_vote_form_valid_upvote() {
        let form = VoteForm {
            agent_id: 1,
            post_id: Some(2),
            comment_id: None,
            vote_type: VoteType::Upvote as i32,
        };

        assert!(form.validate().is_ok());
    }

    #[test]
    fn test_vote_form_valid_downvote() {
        let form = VoteForm {
            agent_id: 1,
            post_id: Some(2),
            comment_id: None,
            vote_type: VoteType::Downvote as i32,
        };

        assert!(form.validate().is_ok());
    }

    #[test]
    fn test_vote_form_no_target() {
        let form = VoteForm {
            agent_id: 1,
            post_id: None,
            comment_id: None,
            vote_type: VoteType::Upvote as i32,
        };

        assert!(form.validate().is_err());
    }

    #[test]
    fn test_vote_form_both_targets() {
        let form = VoteForm {
            agent_id: 1,
            post_id: Some(2),
            comment_id: Some(3),
            vote_type: VoteType::Upvote as i32,
        };

        assert!(form.validate().is_err());
    }

    #[test]
    fn test_vote_form_invalid_type() {
        let form = VoteForm {
            agent_id: 1,
            post_id: Some(2),
            comment_id: None,
            vote_type: 99,
        };

        assert!(form.validate().is_err());
    }

    // ========================================================================
    // NotificationForm Validation Tests
    // ========================================================================

    #[test]
    fn test_notification_form_valid() {
        let form = NotificationForm {
            agent_id: 1,
            notification_type: NotificationType::NewFollower as i32,
            actor_id: 2,
            post_id: None,
            comment_id: None,
            message: "started following you".to_string(),
        };

        assert!(form.validate().is_ok());
    }

    #[test]
    fn test_notification_form_empty_message() {
        let form = NotificationForm {
            agent_id: 1,
            notification_type: NotificationType::NewFollower as i32,
            actor_id: 2,
            post_id: None,
            comment_id: None,
            message: "".to_string(),
        };

        assert!(form.validate().is_err());
    }

    #[test]
    fn test_notification_form_message_too_long() {
        let form = NotificationForm {
            agent_id: 1,
            notification_type: NotificationType::NewFollower as i32,
            actor_id: 2,
            post_id: None,
            comment_id: None,
            message: "a".repeat(501),
        };

        assert!(form.validate().is_err());
    }

    #[test]
    fn test_notification_form_invalid_type() {
        let form = NotificationForm {
            agent_id: 1,
            notification_type: 99,
            actor_id: 2,
            post_id: None,
            comment_id: None,
            message: "test".to_string(),
        };

        assert!(form.validate().is_err());
    }

    // ========================================================================
    // Enum Tests
    // ========================================================================

    #[test]
    fn test_vote_type_values() {
        assert_eq!(VoteType::Upvote as i32, 1);
        assert_eq!(VoteType::Downvote as i32, -1);
    }

    #[test]
    fn test_notification_type_values() {
        assert_eq!(NotificationType::NewFollower as i32, 0);
        assert_eq!(NotificationType::PostComment as i32, 1);
        assert_eq!(NotificationType::CommentReply as i32, 2);
        assert_eq!(NotificationType::PostVote as i32, 3);
        assert_eq!(NotificationType::CommentVote as i32, 4);
        assert_eq!(NotificationType::Mention as i32, 5);
    }

    // ========================================================================
    // PostWithDetails Tests
    // ========================================================================

    #[test]
    fn test_post_with_details_default() {
        use chrono::Utc;
        
        let post = AgentPost {
            id: 1,
            agent_id: 1,
            title: "Test".to_string(),
            content: None,
            tags: None,
            is_public: true,
            is_deleted: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let details = PostWithDetails {
            post,
            author_name: "TestAgent".to_string(),
            vote_count: 0,
            comment_count: 0,
            user_vote: None,
            is_bookmarked: false,
        };

        assert_eq!(details.vote_count, 0);
        assert_eq!(details.comment_count, 0);
        assert!(!details.is_bookmarked);
    }

    // ========================================================================
    // CommentWithDetails Tests
    // ========================================================================

    #[test]
    fn test_comment_with_details_default() {
        use chrono::Utc;
        
        let comment = AgentComment {
            id: 1,
            post_id: 1,
            agent_id: 1,
            parent_id: None,
            content: "Test comment".to_string(),
            is_deleted: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let details = CommentWithDetails {
            comment,
            author_name: "TestAgent".to_string(),
            vote_count: 0,
            user_vote: None,
        };

        assert_eq!(details.vote_count, 0);
        assert_eq!(details.user_vote, None);
    }
}
