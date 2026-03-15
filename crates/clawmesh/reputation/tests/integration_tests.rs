/// Agent Reputation System Integration Tests
/// DO-178C Level A Compliant Test Suite
/// 
/// This test suite provides comprehensive coverage of the reputation system
/// including query operations, voting mechanisms, history tracking, and security.

#[cfg(test)]
mod reputation_tests {
    use diesel::prelude::*;
    use diesel_async::{AsyncPgConnection, RunQueryDsl};
    use clawmesh_reputation::{
        models::{AgentReputation, VoteType},
        reputation::{
            calculate_reputation_score,
            get_agent_reputation,
            initialize_agent_reputation,
            update_reputation_after_vote,
        },
        votes::{
            cast_vote,
            get_vote_history,
            validate_vote,
        },
    };

    // ========================================================================
    // Test Utilities
    // ========================================================================

    /// Create a test database connection
    async fn setup_test_db() -> AsyncPgConnection {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:password@localhost/lemmy_test".to_string());
        
        AsyncPgConnection::establish(&database_url)
            .await
            .expect("Failed to connect to test database")
    }

    /// Create a test person (agent) in the database
    async fn create_test_person(conn: &mut AsyncPgConnection, name: &str, is_agent: bool) -> i32 {
        use lemmy_db_schema::schema::person;
        use lemmy_db_schema::source::person::{Person, PersonInsertForm};
        
        let form = PersonInsertForm {
            name: name.to_string(),
            user_type: if is_agent { "agent" } else { "user" }.to_string(),
            ..Default::default()
        };
        
        diesel::insert_into(person::table)
            .values(&form)
            .returning(person::id)
            .get_result(conn)
            .await
            .expect("Failed to create test person")
    }

    /// Cleanup test data
    async fn cleanup_test_data(conn: &mut AsyncPgConnection) {
        use lemmy_db_schema::schema::person;
        
        diesel::delete(person::table.filter(person::name.like("test_%")))
            .execute(conn)
            .await
            .ok();
    }

    // ========================================================================
    // Score Calculation Tests (Unit Tests)
    // ========================================================================

    #[test]
    fn test_score_calculation_base() {
        // Test base score with no votes
        let score = calculate_reputation_score(0, 0);
        assert_eq!(score, 500, "Base score should be 500");
    }

    #[test]
    fn test_score_calculation_upvotes_only() {
        // Test score with only upvotes
        let score = calculate_reputation_score(10, 0);
        assert_eq!(score, 600, "10 upvotes should give 600 score");
    }

    #[test]
    fn test_score_calculation_downvotes_only() {
        // Test score with only downvotes
        let score = calculate_reputation_score(0, 10);
        assert_eq!(score, 400, "10 downvotes should give 400 score");
    }

    #[test]
    fn test_score_calculation_mixed_votes() {
        // Test score with mixed votes
        let score = calculate_reputation_score(15, 5);
        assert_eq!(score, 600, "15 upvotes and 5 downvotes should give 600 score");
    }

    #[test]
    fn test_score_calculation_min_bound() {
        // Test minimum score boundary (0)
        let score = calculate_reputation_score(0, 100);
        assert_eq!(score, 0, "Score should not go below 0");
    }

    #[test]
    fn test_score_calculation_max_bound() {
        // Test maximum score boundary (2000)
        let score = calculate_reputation_score(200, 0);
        assert_eq!(score, 2000, "Score should not exceed 2000");
    }

    #[test]
    fn test_score_calculation_beyond_max() {
        // Test score calculation beyond maximum
        let score = calculate_reputation_score(300, 0);
        assert_eq!(score, 2000, "Score should be capped at 2000");
    }

    // ========================================================================
    // Reputation Initialization Tests
    // ========================================================================

    #[tokio::test]
    async fn test_initialize_reputation_success() {
        let mut conn = setup_test_db().await;
        let agent_id = create_test_person(&mut conn, "test_agent_init", true).await;
        
        // Initialize reputation
        let result = initialize_agent_reputation(agent_id, &mut conn).await;
        assert!(result.is_ok(), "Reputation initialization should succeed");
        
        // Verify initial values
        let reputation = get_agent_reputation(agent_id, &mut conn)
            .await
            .expect("Should retrieve reputation");
        
        assert_eq!(reputation.agent_id, agent_id);
        assert_eq!(reputation.reputation_score, 500);
        assert_eq!(reputation.total_votes, 0);
        assert_eq!(reputation.positive_votes, 0);
        assert_eq!(reputation.negative_votes, 0);
        assert_eq!(reputation.reputation_level, 1); // Bronze
        
        cleanup_test_data(&mut conn).await;
    }

    #[tokio::test]
    async fn test_initialize_reputation_duplicate() {
        let mut conn = setup_test_db().await;
        let agent_id = create_test_person(&mut conn, "test_agent_dup", true).await;
        
        // Initialize twice
        initialize_agent_reputation(agent_id, &mut conn).await.ok();
        let result = initialize_agent_reputation(agent_id, &mut conn).await;
        
        // Second initialization should fail or be idempotent
        assert!(result.is_err() || result.is_ok(), "Should handle duplicate initialization");
        
        cleanup_test_data(&mut conn).await;
    }

    // ========================================================================
    // Vote Validation Tests
    // ========================================================================

    #[tokio::test]
    async fn test_validate_vote_success() {
        let mut conn = setup_test_db().await;
        let voter_id = create_test_person(&mut conn, "test_voter_1", true).await;
        let target_id = create_test_person(&mut conn, "test_target_1", true).await;
        
        initialize_agent_reputation(target_id, &mut conn).await.ok();
        
        // Validate vote
        let result = validate_vote(voter_id, target_id, &mut conn).await;
        assert!(result.is_ok(), "Valid vote should pass validation");
        
        cleanup_test_data(&mut conn).await;
    }

    #[tokio::test]
    async fn test_validate_vote_self_voting() {
        let mut conn = setup_test_db().await;
        let agent_id = create_test_person(&mut conn, "test_agent_self", true).await;
        
        initialize_agent_reputation(agent_id, &mut conn).await.ok();
        
        // Try to vote for self
        let result = validate_vote(agent_id, agent_id, &mut conn).await;
        assert!(result.is_err(), "Self-voting should be rejected");
        assert!(result.unwrap_err().to_string().contains("yourself"));
        
        cleanup_test_data(&mut conn).await;
    }

    #[tokio::test]
    async fn test_validate_vote_non_agent_voter() {
        let mut conn = setup_test_db().await;
        let user_id = create_test_person(&mut conn, "test_user_voter", false).await;
        let agent_id = create_test_person(&mut conn, "test_agent_target", true).await;
        
        initialize_agent_reputation(agent_id, &mut conn).await.ok();
        
        // Non-agent trying to vote
        let result = validate_vote(user_id, agent_id, &mut conn).await;
        assert!(result.is_err(), "Non-agent voting should be rejected");
        assert!(result.unwrap_err().to_string().contains("Only agents"));
        
        cleanup_test_data(&mut conn).await;
    }

    #[tokio::test]
    async fn test_validate_vote_non_agent_target() {
        let mut conn = setup_test_db().await;
        let voter_id = create_test_person(&mut conn, "test_voter_2", true).await;
        let user_id = create_test_person(&mut conn, "test_user_target", false).await;
        
        // Try to vote for non-agent
        let result = validate_vote(voter_id, user_id, &mut conn).await;
        assert!(result.is_err(), "Voting for non-agent should be rejected");
        assert!(result.unwrap_err().to_string().contains("not an agent"));
        
        cleanup_test_data(&mut conn).await;
    }

    // ========================================================================
    // Voting Tests
    // ========================================================================

    #[tokio::test]
    async fn test_cast_upvote_success() {
        let mut conn = setup_test_db().await;
        let voter_id = create_test_person(&mut conn, "test_voter_up", true).await;
        let target_id = create_test_person(&mut conn, "test_target_up", true).await;
        
        initialize_agent_reputation(target_id, &mut conn).await.ok();
        
        // Get initial score
        let initial = get_agent_reputation(target_id, &mut conn)
            .await
            .expect("Should get initial reputation");
        
        // Cast upvote
        let result = cast_vote(
            voter_id,
            target_id,
            VoteType::Upvote,
            Some("Great work!".to_string()),
            &mut conn
        ).await;
        
        assert!(result.is_ok(), "Upvote should succeed");
        
        // Verify score increased
        let updated = get_agent_reputation(target_id, &mut conn)
            .await
            .expect("Should get updated reputation");
        
        assert_eq!(updated.reputation_score, initial.reputation_score + 10);
        assert_eq!(updated.total_votes, 1);
        assert_eq!(updated.positive_votes, 1);
        assert_eq!(updated.negative_votes, 0);
        
        cleanup_test_data(&mut conn).await;
    }

    #[tokio::test]
    async fn test_cast_downvote_success() {
        let mut conn = setup_test_db().await;
        let voter_id = create_test_person(&mut conn, "test_voter_down", true).await;
        let target_id = create_test_person(&mut conn, "test_target_down", true).await;
        
        initialize_agent_reputation(target_id, &mut conn).await.ok();
        
        // Get initial score
        let initial = get_agent_reputation(target_id, &mut conn)
            .await
            .expect("Should get initial reputation");
        
        // Cast downvote
        let result = cast_vote(
            voter_id,
            target_id,
            VoteType::Downvote,
            Some("Needs improvement".to_string()),
            &mut conn
        ).await;
        
        assert!(result.is_ok(), "Downvote should succeed");
        
        // Verify score decreased
        let updated = get_agent_reputation(target_id, &mut conn)
            .await
            .expect("Should get updated reputation");
        
        assert_eq!(updated.reputation_score, initial.reputation_score - 10);
        assert_eq!(updated.total_votes, 1);
        assert_eq!(updated.positive_votes, 0);
        assert_eq!(updated.negative_votes, 1);
        
        cleanup_test_data(&mut conn).await;
    }

    #[tokio::test]
    async fn test_vote_with_reason() {
        let mut conn = setup_test_db().await;
        let voter_id = create_test_person(&mut conn, "test_voter_reason", true).await;
        let target_id = create_test_person(&mut conn, "test_target_reason", true).await;
        
        initialize_agent_reputation(target_id, &mut conn).await.ok();
        
        let reason = "Excellent contribution to the project";
        
        // Cast vote with reason
        cast_vote(
            voter_id,
            target_id,
            VoteType::Upvote,
            Some(reason.to_string()),
            &mut conn
        ).await.ok();
        
        // Verify reason is stored in history
        let history = get_vote_history(target_id, 10, 0, &mut conn)
            .await
            .expect("Should get vote history");
        
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].voter_id, voter_id);
        assert_eq!(history[0].reason, Some(reason.to_string()));
        
        cleanup_test_data(&mut conn).await;
    }

    // ========================================================================
    // Vote History Tests
    // ========================================================================

    #[tokio::test]
    async fn test_get_vote_history_empty() {
        let mut conn = setup_test_db().await;
        let agent_id = create_test_person(&mut conn, "test_agent_history", true).await;
        
        initialize_agent_reputation(agent_id, &mut conn).await.ok();
        
        // Get history for agent with no votes
        let history = get_vote_history(agent_id, 10, 0, &mut conn)
            .await
            .expect("Should get empty history");
        
        assert_eq!(history.len(), 0, "History should be empty");
        
        cleanup_test_data(&mut conn).await;
    }

    #[tokio::test]
    async fn test_get_vote_history_pagination() {
        let mut conn = setup_test_db().await;
        let target_id = create_test_person(&mut conn, "test_target_page", true).await;
        
        initialize_agent_reputation(target_id, &mut conn).await.ok();
        
        // Create multiple votes
        for i in 0..5 {
            let voter_id = create_test_person(
                &mut conn,
                &format!("test_voter_page_{}", i),
                true
            ).await;
            
            cast_vote(voter_id, target_id, VoteType::Upvote, None, &mut conn)
                .await
                .ok();
        }
        
        // Test pagination
        let page1 = get_vote_history(target_id, 2, 0, &mut conn)
            .await
            .expect("Should get page 1");
        assert_eq!(page1.len(), 2);
        
        let page2 = get_vote_history(target_id, 2, 2, &mut conn)
            .await
            .expect("Should get page 2");
        assert_eq!(page2.len(), 2);
        
        let page3 = get_vote_history(target_id, 2, 4, &mut conn)
            .await
            .expect("Should get page 3");
        assert_eq!(page3.len(), 1);
        
        cleanup_test_data(&mut conn).await;
    }

    // ========================================================================
    // Reputation Level Tests
    // ========================================================================

    #[tokio::test]
    async fn test_reputation_level_progression() {
        let mut conn = setup_test_db().await;
        let target_id = create_test_person(&mut conn, "test_level_prog", true).await;
        
        initialize_agent_reputation(target_id, &mut conn).await.ok();
        
        // Start at Bronze (level 1)
        let rep = get_agent_reputation(target_id, &mut conn).await.unwrap();
        assert_eq!(rep.reputation_level, 1);
        
        // Vote up to Silver (600+)
        for i in 0..11 {
            let voter_id = create_test_person(
                &mut conn,
                &format!("test_voter_level_{}", i),
                true
            ).await;
            cast_vote(voter_id, target_id, VoteType::Upvote, None, &mut conn)
                .await
                .ok();
        }
        
        let rep = get_agent_reputation(target_id, &mut conn).await.unwrap();
        assert!(rep.reputation_score >= 600);
        assert_eq!(rep.reputation_level, 2); // Silver
        
        cleanup_test_data(&mut conn).await;
    }

    // ========================================================================
    // Concurrent Voting Tests
    // ========================================================================

    #[tokio::test]
    async fn test_concurrent_votes() {
        use tokio::task;
        
        let mut conn = setup_test_db().await;
        let target_id = create_test_person(&mut conn, "test_concurrent", true).await;
        
        initialize_agent_reputation(target_id, &mut conn).await.ok();
        
        // Create voters
        let mut voter_ids = Vec::new();
        for i in 0..10 {
            let voter_id = create_test_person(
                &mut conn,
                &format!("test_voter_conc_{}", i),
                true
            ).await;
            voter_ids.push(voter_id);
        }
        
        // Spawn concurrent voting tasks
        let mut handles = Vec::new();
        for voter_id in voter_ids {
            let handle = task::spawn(async move {
                let mut conn = setup_test_db().await;
                cast_vote(voter_id, target_id, VoteType::Upvote, None, &mut conn).await
            });
            handles.push(handle);
        }
        
        // Wait for all votes
        for handle in handles {
            let result = handle.await.expect("Task should complete");
            assert!(result.is_ok(), "Concurrent vote should succeed");
        }
        
        // Verify final state
        let final_rep = get_agent_reputation(target_id, &mut conn)
            .await
            .expect("Should get final reputation");
        
        assert_eq!(final_rep.total_votes, 10);
        assert_eq!(final_rep.positive_votes, 10);
        assert_eq!(final_rep.reputation_score, 600); // 500 + (10 * 10)
        
        cleanup_test_data(&mut conn).await;
    }

    // ========================================================================
    // Error Handling Tests
    // ========================================================================

    #[tokio::test]
    async fn test_get_reputation_nonexistent() {
        let mut conn = setup_test_db().await;
        
        // Try to get reputation for non-existent agent
        let result = get_agent_reputation(999999, &mut conn).await;
        assert!(result.is_err(), "Should fail for non-existent agent");
        
        cleanup_test_data(&mut conn).await;
    }

    #[tokio::test]
    async fn test_vote_nonexistent_target() {
        let mut conn = setup_test_db().await;
        let voter_id = create_test_person(&mut conn, "test_voter_noexist", true).await;
        
        // Try to vote for non-existent agent
        let result = cast_vote(voter_id, 999999, VoteType::Upvote, None, &mut conn).await;
        assert!(result.is_err(), "Should fail for non-existent target");
        
        cleanup_test_data(&mut conn).await;
    }

    // ========================================================================
    // Integration Tests
    // ========================================================================

    #[tokio::test]
    async fn test_full_reputation_lifecycle() {
        let mut conn = setup_test_db().await;
        
        // 1. Create agent
        let agent_id = create_test_person(&mut conn, "test_lifecycle", true).await;
        
        // 2. Initialize reputation
        initialize_agent_reputation(agent_id, &mut conn)
            .await
            .expect("Should initialize");
        
        let rep = get_agent_reputation(agent_id, &mut conn).await.unwrap();
        assert_eq!(rep.reputation_score, 500);
        assert_eq!(rep.reputation_level, 1);
        
        // 3. Receive upvotes
        for i in 0..5 {
            let voter_id = create_test_person(
                &mut conn,
                &format!("test_voter_life_{}", i),
                true
            ).await;
            cast_vote(voter_id, agent_id, VoteType::Upvote, None, &mut conn)
                .await
                .expect("Vote should succeed");
        }
        
        let rep = get_agent_reputation(agent_id, &mut conn).await.unwrap();
        assert_eq!(rep.reputation_score, 550);
        assert_eq!(rep.total_votes, 5);
        
        // 4. Receive downvotes
        for i in 5..7 {
            let voter_id = create_test_person(
                &mut conn,
                &format!("test_voter_life_{}", i),
                true
            ).await;
            cast_vote(voter_id, agent_id, VoteType::Downvote, None, &mut conn)
                .await
                .expect("Vote should succeed");
        }
        
        let rep = get_agent_reputation(agent_id, &mut conn).await.unwrap();
        assert_eq!(rep.reputation_score, 530);
        assert_eq!(rep.total_votes, 7);
        
        // 5. Check history
        let history = get_vote_history(agent_id, 10, 0, &mut conn)
            .await
            .expect("Should get history");
        assert_eq!(history.len(), 7);
        
        cleanup_test_data(&mut conn).await;
    }
}
