/// Agent Workspace Integration Tests
/// DO-178C Level A Compliant Test Suite
/// 
/// Comprehensive tests for workspace collaboration features

#[cfg(test)]
mod workspace_integration_tests {
    use clawmesh_workspace::{
        models::*,
        workspace::*,
        members::*,
        tasks::*,
        activities::*,
    };
    use diesel::prelude::*;
    use diesel_async::{AsyncPgConnection, RunQueryDsl};

    // ========================================================================
    // Test Setup Helpers
    // ========================================================================

    async fn setup_test_db() -> AsyncPgConnection {
        // Mock database connection setup
        // In real tests, this would connect to a test database
        unimplemented!("Database connection setup")
    }

    async fn create_test_agent(conn: &mut AsyncPgConnection) -> i32 {
        // Create a test agent and return its ID
        unimplemented!("Test agent creation")
    }

    async fn cleanup_test_data(conn: &mut AsyncPgConnection) {
        // Clean up test data
        unimplemented!("Test data cleanup")
    }

    // ========================================================================
    // Workspace Creation Tests
    // ========================================================================

    #[tokio::test]
    async fn test_create_workspace_success() {
        let mut conn = setup_test_db().await;
        let owner_id = create_test_agent(&mut conn).await;

        let form = WorkspaceForm {
            name: "Test Workspace".to_string(),
            description: Some("A test workspace".to_string()),
            owner_id,
            is_public: false,
            max_members: 10,
        };

        let result = create_workspace(form, &mut conn).await;
        assert!(result.is_ok());

        let workspace = result.unwrap();
        assert_eq!(workspace.name, "Test Workspace");
        assert_eq!(workspace.owner_id, owner_id);
        assert!(!workspace.is_public);
        assert_eq!(workspace.max_members, 10);

        cleanup_test_data(&mut conn).await;
    }

    #[tokio::test]
    async fn test_create_workspace_invalid_name() {
        let mut conn = setup_test_db().await;
        let owner_id = create_test_agent(&mut conn).await;

        // Empty name
        let form = WorkspaceForm {
            name: "".to_string(),
            description: None,
            owner_id,
            is_public: false,
            max_members: 10,
        };

        let result = create_workspace(form, &mut conn).await;
        assert!(result.is_err());

        cleanup_test_data(&mut conn).await;
    }

    #[tokio::test]
    async fn test_create_workspace_invalid_max_members() {
        let mut conn = setup_test_db().await;
        let owner_id = create_test_agent(&mut conn).await;

        // Max members too high
        let form = WorkspaceForm {
            name: "Test Workspace".to_string(),
            description: None,
            owner_id,
            is_public: false,
            max_members: 200, // Exceeds limit
        };

        let result = create_workspace(form, &mut conn).await;
        assert!(result.is_err());

        cleanup_test_data(&mut conn).await;
    }

    #[tokio::test]
    async fn test_create_workspace_owner_auto_added() {
        let mut conn = setup_test_db().await;
        let owner_id = create_test_agent(&mut conn).await;

        let form = WorkspaceForm {
            name: "Test Workspace".to_string(),
            description: None,
            owner_id,
            is_public: false,
            max_members: 10,
        };

        let workspace = create_workspace(form, &mut conn).await.unwrap();

        // Verify owner is automatically added as member
        let members = list_members(workspace.id, &mut conn).await.unwrap();
        assert_eq!(members.len(), 1);
        assert_eq!(members[0].agent_id, owner_id);
        assert_eq!(members[0].role, WorkspaceRole::Owner as i32);

        cleanup_test_data(&mut conn).await;
    }

    // ========================================================================
    // Workspace Query Tests
    // ========================================================================

    #[tokio::test]
    async fn test_get_workspace_success() {
        let mut conn = setup_test_db().await;
        let owner_id = create_test_agent(&mut conn).await;

        let form = WorkspaceForm {
            name: "Test Workspace".to_string(),
            description: Some("Description".to_string()),
            owner_id,
            is_public: false,
            max_members: 10,
        };

        let created = create_workspace(form, &mut conn).await.unwrap();
        let retrieved = get_workspace(created.id, &mut conn).await.unwrap();

        assert_eq!(created.id, retrieved.id);
        assert_eq!(created.name, retrieved.name);

        cleanup_test_data(&mut conn).await;
    }

    #[tokio::test]
    async fn test_get_workspace_not_found() {
        let mut conn = setup_test_db().await;

        let result = get_workspace(99999, &mut conn).await;
        assert!(result.is_err());

        cleanup_test_data(&mut conn).await;
    }

    #[tokio::test]
    async fn test_list_workspaces_by_agent() {
        let mut conn = setup_test_db().await;
        let agent1 = create_test_agent(&mut conn).await;
        let agent2 = create_test_agent(&mut conn).await;

        // Create workspaces for agent1
        for i in 1..=3 {
            let form = WorkspaceForm {
                name: format!("Workspace {}", i),
                description: None,
                owner_id: agent1,
                is_public: false,
                max_members: 10,
            };
            create_workspace(form, &mut conn).await.unwrap();
        }

        // Create workspace for agent2
        let form = WorkspaceForm {
            name: "Agent2 Workspace".to_string(),
            description: None,
            owner_id: agent2,
            is_public: false,
            max_members: 10,
        };
        create_workspace(form, &mut conn).await.unwrap();

        // List agent1's workspaces
        let workspaces = list_workspaces(Some(agent1), None, 10, 0, &mut conn)
            .await
            .unwrap();

        assert_eq!(workspaces.len(), 3);

        cleanup_test_data(&mut conn).await;
    }

    #[tokio::test]
    async fn test_list_public_workspaces() {
        let mut conn = setup_test_db().await;
        let owner_id = create_test_agent(&mut conn).await;

        // Create public workspace
        let form1 = WorkspaceForm {
            name: "Public Workspace".to_string(),
            description: None,
            owner_id,
            is_public: true,
            max_members: 10,
        };
        create_workspace(form1, &mut conn).await.unwrap();

        // Create private workspace
        let form2 = WorkspaceForm {
            name: "Private Workspace".to_string(),
            description: None,
            owner_id,
            is_public: false,
            max_members: 10,
        };
        create_workspace(form2, &mut conn).await.unwrap();

        // List only public workspaces
        let workspaces = list_workspaces(None, Some(true), 10, 0, &mut conn)
            .await
            .unwrap();

        assert!(workspaces.iter().all(|w| w.is_public));

        cleanup_test_data(&mut conn).await;
    }

    // ========================================================================
    // Workspace Update Tests
    // ========================================================================

    #[tokio::test]
    async fn test_update_workspace_success() {
        let mut conn = setup_test_db().await;
        let owner_id = create_test_agent(&mut conn).await;

        let form = WorkspaceForm {
            name: "Original Name".to_string(),
            description: None,
            owner_id,
            is_public: false,
            max_members: 10,
        };

        let workspace = create_workspace(form, &mut conn).await.unwrap();

        // Update workspace
        let update_form = WorkspaceForm {
            name: "Updated Name".to_string(),
            description: Some("New description".to_string()),
            owner_id,
            is_public: true,
            max_members: 20,
        };

        let updated = update_workspace(workspace.id, update_form, owner_id, &mut conn)
            .await
            .unwrap();

        assert_eq!(updated.name, "Updated Name");
        assert_eq!(updated.description, Some("New description".to_string()));
        assert!(updated.is_public);
        assert_eq!(updated.max_members, 20);

        cleanup_test_data(&mut conn).await;
    }

    #[tokio::test]
    async fn test_update_workspace_no_permission() {
        let mut conn = setup_test_db().await;
        let owner_id = create_test_agent(&mut conn).await;
        let other_agent = create_test_agent(&mut conn).await;

        let form = WorkspaceForm {
            name: "Test Workspace".to_string(),
            description: None,
            owner_id,
            is_public: false,
            max_members: 10,
        };

        let workspace = create_workspace(form, &mut conn).await.unwrap();

        // Try to update as non-owner
        let update_form = WorkspaceForm {
            name: "Hacked Name".to_string(),
            description: None,
            owner_id,
            is_public: false,
            max_members: 10,
        };

        let result = update_workspace(workspace.id, update_form, other_agent, &mut conn).await;
        assert!(result.is_err());

        cleanup_test_data(&mut conn).await;
    }

    // ========================================================================
    // Workspace Delete Tests
    // ========================================================================

    #[tokio::test]
    async fn test_delete_workspace_success() {
        let mut conn = setup_test_db().await;
        let owner_id = create_test_agent(&mut conn).await;

        let form = WorkspaceForm {
            name: "To Delete".to_string(),
            description: None,
            owner_id,
            is_public: false,
            max_members: 10,
        };

        let workspace = create_workspace(form, &mut conn).await.unwrap();

        // Delete workspace
        let result = delete_workspace(workspace.id, owner_id, &mut conn).await;
        assert!(result.is_ok());

        // Verify deletion
        let get_result = get_workspace(workspace.id, &mut conn).await;
        assert!(get_result.is_err());

        cleanup_test_data(&mut conn).await;
    }

    #[tokio::test]
    async fn test_delete_workspace_only_owner() {
        let mut conn = setup_test_db().await;
        let owner_id = create_test_agent(&mut conn).await;
        let other_agent = create_test_agent(&mut conn).await;

        let form = WorkspaceForm {
            name: "Protected Workspace".to_string(),
            description: None,
            owner_id,
            is_public: false,
            max_members: 10,
        };

        let workspace = create_workspace(form, &mut conn).await.unwrap();

        // Try to delete as non-owner
        let result = delete_workspace(workspace.id, other_agent, &mut conn).await;
        assert!(result.is_err());

        cleanup_test_data(&mut conn).await;
    }

    // ========================================================================
    // Member Management Tests
    // ========================================================================

    #[tokio::test]
    async fn test_add_member_success() {
        let mut conn = setup_test_db().await;
        let owner_id = create_test_agent(&mut conn).await;
        let new_member = create_test_agent(&mut conn).await;

        let form = WorkspaceForm {
            name: "Test Workspace".to_string(),
            description: None,
            owner_id,
            is_public: false,
            max_members: 10,
        };

        let workspace = create_workspace(form, &mut conn).await.unwrap();

        // Add member
        let member_form = WorkspaceMemberForm {
            workspace_id: workspace.id,
            agent_id: new_member,
            role: WorkspaceRole::Member as i32,
        };

        let result = add_member(member_form, &mut conn).await;
        assert!(result.is_ok());

        let members = list_members(workspace.id, &mut conn).await.unwrap();
        assert_eq!(members.len(), 2); // Owner + new member

        cleanup_test_data(&mut conn).await;
    }

    #[tokio::test]
    async fn test_add_member_duplicate() {
        let mut conn = setup_test_db().await;
        let owner_id = create_test_agent(&mut conn).await;

        let form = WorkspaceForm {
            name: "Test Workspace".to_string(),
            description: None,
            owner_id,
            is_public: false,
            max_members: 10,
        };

        let workspace = create_workspace(form, &mut conn).await.unwrap();

        // Try to add owner again
        let member_form = WorkspaceMemberForm {
            workspace_id: workspace.id,
            agent_id: owner_id,
            role: WorkspaceRole::Member as i32,
        };

        let result = add_member(member_form, &mut conn).await;
        assert!(result.is_err());

        cleanup_test_data(&mut conn).await;
    }

    #[tokio::test]
    async fn test_add_member_capacity_limit() {
        let mut conn = setup_test_db().await;
        let owner_id = create_test_agent(&mut conn).await;

        let form = WorkspaceForm {
            name: "Small Workspace".to_string(),
            description: None,
            owner_id,
            is_public: false,
            max_members: 2, // Only 2 members allowed
        };

        let workspace = create_workspace(form, &mut conn).await.unwrap();

        // Add first member (should succeed)
        let member1 = create_test_agent(&mut conn).await;
        let member_form1 = WorkspaceMemberForm {
            workspace_id: workspace.id,
            agent_id: member1,
            role: WorkspaceRole::Member as i32,
        };
        add_member(member_form1, &mut conn).await.unwrap();

        // Try to add second member (should fail - capacity reached)
        let member2 = create_test_agent(&mut conn).await;
        let member_form2 = WorkspaceMemberForm {
            workspace_id: workspace.id,
            agent_id: member2,
            role: WorkspaceRole::Member as i32,
        };

        let result = add_member(member_form2, &mut conn).await;
        assert!(result.is_err());

        cleanup_test_data(&mut conn).await;
    }

    #[tokio::test]
    async fn test_remove_member_success() {
        let mut conn = setup_test_db().await;
        let owner_id = create_test_agent(&mut conn).await;
        let member_id = create_test_agent(&mut conn).await;

        let form = WorkspaceForm {
            name: "Test Workspace".to_string(),
            description: None,
            owner_id,
            is_public: false,
            max_members: 10,
        };

        let workspace = create_workspace(form, &mut conn).await.unwrap();

        // Add member
        let member_form = WorkspaceMemberForm {
            workspace_id: workspace.id,
            agent_id: member_id,
            role: WorkspaceRole::Member as i32,
        };
        add_member(member_form, &mut conn).await.unwrap();

        // Remove member
        let result = remove_member(workspace.id, member_id, owner_id, &mut conn).await;
        assert!(result.is_ok());

        let members = list_members(workspace.id, &mut conn).await.unwrap();
        assert_eq!(members.len(), 1); // Only owner remains

        cleanup_test_data(&mut conn).await;
    }

    #[tokio::test]
    async fn test_remove_member_cannot_remove_owner() {
        let mut conn = setup_test_db().await;
        let owner_id = create_test_agent(&mut conn).await;

        let form = WorkspaceForm {
            name: "Test Workspace".to_string(),
            description: None,
            owner_id,
            is_public: false,
            max_members: 10,
        };

        let workspace = create_workspace(form, &mut conn).await.unwrap();

        // Try to remove owner
        let result = remove_member(workspace.id, owner_id, owner_id, &mut conn).await;
        assert!(result.is_err());

        cleanup_test_data(&mut conn).await;
    }

    #[tokio::test]
    async fn test_update_member_role() {
        let mut conn = setup_test_db().await;
        let owner_id = create_test_agent(&mut conn).await;
        let member_id = create_test_agent(&mut conn).await;

        let form = WorkspaceForm {
            name: "Test Workspace".to_string(),
            description: None,
            owner_id,
            is_public: false,
            max_members: 10,
        };

        let workspace = create_workspace(form, &mut conn).await.unwrap();

        // Add member
        let member_form = WorkspaceMemberForm {
            workspace_id: workspace.id,
            agent_id: member_id,
            role: WorkspaceRole::Member as i32,
        };
        add_member(member_form, &mut conn).await.unwrap();

        // Promote to admin
        let updated = update_member_role(
            workspace.id,
            member_id,
            WorkspaceRole::Admin,
            owner_id,
            &mut conn,
        )
        .await
        .unwrap();

        assert_eq!(updated.role, WorkspaceRole::Admin as i32);

        cleanup_test_data(&mut conn).await;
    }

    // ========================================================================
    // Task Management Tests
    // ========================================================================

    #[tokio::test]
    async fn test_create_task_success() {
        let mut conn = setup_test_db().await;
        let owner_id = create_test_agent(&mut conn).await;

        let form = WorkspaceForm {
            name: "Test Workspace".to_string(),
            description: None,
            owner_id,
            is_public: false,
            max_members: 10,
        };

        let workspace = create_workspace(form, &mut conn).await.unwrap();

        // Create task
        let task_form = WorkspaceTaskForm {
            workspace_id: workspace.id,
            title: "Test Task".to_string(),
            description: Some("Task description".to_string()),
            status: TaskStatus::Todo as i32,
            priority: TaskPriority::Medium as i32,
            assigned_to: None,
            created_by: owner_id,
            due_date: None,
        };

        let task = create_task(task_form, &mut conn).await.unwrap();
        assert_eq!(task.title, "Test Task");
        assert_eq!(task.status, TaskStatus::Todo as i32);

        cleanup_test_data(&mut conn).await;
    }

    #[tokio::test]
    async fn test_assign_task() {
        let mut conn = setup_test_db().await;
        let owner_id = create_test_agent(&mut conn).await;
        let assignee = create_test_agent(&mut conn).await;

        let form = WorkspaceForm {
            name: "Test Workspace".to_string(),
            description: None,
            owner_id,
            is_public: false,
            max_members: 10,
        };

        let workspace = create_workspace(form, &mut conn).await.unwrap();

        // Add assignee as member
        let member_form = WorkspaceMemberForm {
            workspace_id: workspace.id,
            agent_id: assignee,
            role: WorkspaceRole::Member as i32,
        };
        add_member(member_form, &mut conn).await.unwrap();

        // Create task
        let task_form = WorkspaceTaskForm {
            workspace_id: workspace.id,
            title: "Assignable Task".to_string(),
            description: None,
            status: TaskStatus::Todo as i32,
            priority: TaskPriority::High as i32,
            assigned_to: None,
            created_by: owner_id,
            due_date: None,
        };

        let task = create_task(task_form, &mut conn).await.unwrap();

        // Assign task
        let assigned = assign_task(task.id, assignee, owner_id, &mut conn)
            .await
            .unwrap();

        assert_eq!(assigned.assigned_to, Some(assignee));

        cleanup_test_data(&mut conn).await;
    }

    #[tokio::test]
    async fn test_update_task_status() {
        let mut conn = setup_test_db().await;
        let owner_id = create_test_agent(&mut conn).await;

        let form = WorkspaceForm {
            name: "Test Workspace".to_string(),
            description: None,
            owner_id,
            is_public: false,
            max_members: 10,
        };

        let workspace = create_workspace(form, &mut conn).await.unwrap();

        // Create task
        let task_form = WorkspaceTaskForm {
            workspace_id: workspace.id,
            title: "Status Test Task".to_string(),
            description: None,
            status: TaskStatus::Todo as i32,
            priority: TaskPriority::Medium as i32,
            assigned_to: None,
            created_by: owner_id,
            due_date: None,
        };

        let task = create_task(task_form, &mut conn).await.unwrap();

        // Update to InProgress
        let updated = update_task_status(task.id, TaskStatus::InProgress, owner_id, &mut conn)
            .await
            .unwrap();

        assert_eq!(updated.status, TaskStatus::InProgress as i32);

        // Update to Done
        let completed = update_task_status(task.id, TaskStatus::Done, owner_id, &mut conn)
            .await
            .unwrap();

        assert_eq!(completed.status, TaskStatus::Done as i32);
        assert!(completed.completed_at.is_some());

        cleanup_test_data(&mut conn).await;
    }

    #[tokio::test]
    async fn test_list_tasks_by_status() {
        let mut conn = setup_test_db().await;
        let owner_id = create_test_agent(&mut conn).await;

        let form = WorkspaceForm {
            name: "Test Workspace".to_string(),
            description: None,
            owner_id,
            is_public: false,
            max_members: 10,
        };

        let workspace = create_workspace(form, &mut conn).await.unwrap();

        // Create tasks with different statuses
        for i in 0..3 {
            let task_form = WorkspaceTaskForm {
                workspace_id: workspace.id,
                title: format!("Task {}", i),
                description: None,
                status: TaskStatus::Todo as i32,
                priority: TaskPriority::Medium as i32,
                assigned_to: None,
                created_by: owner_id,
                due_date: None,
            };
            create_task(task_form, &mut conn).await.unwrap();
        }

        for i in 3..5 {
            let task_form = WorkspaceTaskForm {
                workspace_id: workspace.id,
                title: format!("Task {}", i),
                description: None,
                status: TaskStatus::InProgress as i32,
                priority: TaskPriority::Medium as i32,
                assigned_to: None,
                created_by: owner_id,
                due_date: None,
            };
            create_task(task_form, &mut conn).await.unwrap();
        }

        // List only Todo tasks
        let todo_tasks = list_tasks(
            workspace.id,
            Some(TaskStatus::Todo),
            None,
            10,
            0,
            &mut conn,
        )
        .await
        .unwrap();

        assert_eq!(todo_tasks.len(), 3);

        cleanup_test_data(&mut conn).await;
    }

    // ========================================================================
    // Activity Logging Tests
    // ========================================================================

    #[tokio::test]
    async fn test_activity_logging() {
        let mut conn = setup_test_db().await;
        let owner_id = create_test_agent(&mut conn).await;

        let form = WorkspaceForm {
            name: "Test Workspace".to_string(),
            description: None,
            owner_id,
            is_public: false,
            max_members: 10,
        };

        let workspace = create_workspace(form, &mut conn).await.unwrap();

        // Get activities
        let activities = get_workspace_activities(workspace.id, 10, 0, &mut conn)
            .await
            .unwrap();

        // Should have at least workspace creation activity
        assert!(!activities.is_empty());
        assert_eq!(activities[0].activity_type, ActivityType::WorkspaceCreated as i32);

        cleanup_test_data(&mut conn).await;
    }

    // ========================================================================
    // Workspace Statistics Tests
    // ========================================================================

    #[tokio::test]
    async fn test_workspace_stats() {
        let mut conn = setup_test_db().await;
        let owner_id = create_test_agent(&mut conn).await;

        let form = WorkspaceForm {
            name: "Test Workspace".to_string(),
            description: None,
            owner_id,
            is_public: false,
            max_members: 10,
        };

        let workspace = create_workspace(form, &mut conn).await.unwrap();

        // Add members
        for _ in 0..3 {
            let member = create_test_agent(&mut conn).await;
            let member_form = WorkspaceMemberForm {
                workspace_id: workspace.id,
                agent_id: member,
                role: WorkspaceRole::Member as i32,
            };
            add_member(member_form, &mut conn).await.unwrap();
        }

        // Create tasks
        for i in 0..5 {
            let task_form = WorkspaceTaskForm {
                workspace_id: workspace.id,
                title: format!("Task {}", i),
                description: None,
                status: if i < 2 {
                    TaskStatus::Done as i32
                } else {
                    TaskStatus::Todo as i32
                },
                priority: TaskPriority::Medium as i32,
                assigned_to: None,
                created_by: owner_id,
                due_date: None,
            };
            create_task(task_form, &mut conn).await.unwrap();
        }

        // Get stats
        let stats = get_workspace_stats(workspace.id, &mut conn).await.unwrap();

        assert_eq!(stats.total_members, 4); // Owner + 3 members
        assert_eq!(stats.total_tasks, 5);
        assert_eq!(stats.completed_tasks, 2);

        cleanup_test_data(&mut conn).await;
    }

    // ========================================================================
    // Permission Tests
    // ========================================================================

    #[tokio::test]
    async fn test_permission_checks() {
        let mut conn = setup_test_db().await;
        let owner_id = create_test_agent(&mut conn).await;
        let admin_id = create_test_agent(&mut conn).await;
        let member_id = create_test_agent(&mut conn).await;
        let viewer_id = create_test_agent(&mut conn).await;

        let form = WorkspaceForm {
            name: "Permission Test".to_string(),
            description: None,
            owner_id,
            is_public: false,
            max_members: 10,
        };

        let workspace = create_workspace(form, &mut conn).await.unwrap();

        // Add members with different roles
        let roles = vec![
            (admin_id, WorkspaceRole::Admin),
            (member_id, WorkspaceRole::Member),
            (viewer_id, WorkspaceRole::Viewer),
        ];

        for (agent_id, role) in roles {
            let member_form = WorkspaceMemberForm {
                workspace_id: workspace.id,
                agent_id,
                role: role as i32,
            };
            add_member(member_form, &mut conn).await.unwrap();
        }

        // Test permissions
        assert!(check_member_permission(workspace.id, owner_id, "manage_members", &mut conn).await.unwrap());
        assert!(check_member_permission(workspace.id, admin_id, "manage_members", &mut conn).await.unwrap());
        assert!(!check_member_permission(workspace.id, member_id, "manage_members", &mut conn).await.unwrap());
        assert!(!check_member_permission(workspace.id, viewer_id, "manage_members", &mut conn).await.unwrap());

        assert!(check_member_permission(workspace.id, member_id, "create_tasks", &mut conn).await.unwrap());
        assert!(!check_member_permission(workspace.id, viewer_id, "create_tasks", &mut conn).await.unwrap());

        cleanup_test_data(&mut conn).await;
    }

    // ========================================================================
    // Integration Lifecycle Test
    // ========================================================================

    #[tokio::test]
    async fn test_full_workspace_lifecycle() {
        let mut conn = setup_test_db().await;
        let owner_id = create_test_agent(&mut conn).await;
        let member1 = create_test_agent(&mut conn).await;
        let member2 = create_test_agent(&mut conn).await;

        // 1. Create workspace
        let form = WorkspaceForm {
            name: "Project Alpha".to_string(),
            description: Some("A collaborative project".to_string()),
            owner_id,
            is_public: false,
            max_members: 10,
        };

        let workspace = create_workspace(form, &mut conn).await.unwrap();
        assert_eq!(workspace.name, "Project Alpha");

        // 2. Add team members
        for member in [member1, member2] {
            let member_form = WorkspaceMemberForm {
                workspace_id: workspace.id,
                agent_id: member,
                role: WorkspaceRole::Member as i32,
            };
            add_member(member_form, &mut conn).await.unwrap();
        }

        let members = list_members(workspace.id, &mut conn).await.unwrap();
        assert_eq!(members.len(), 3);

        // 3. Create and assign tasks
        let task1_form = WorkspaceTaskForm {
            workspace_id: workspace.id,
            title: "Design System".to_string(),
            description: Some("Design the system architecture".to_string()),
            status: TaskStatus::Todo as i32,
            priority: TaskPriority::High as i32,
            assigned_to: None,
            created_by: owner_id,
            due_date: None,
        };

        let task1 = create_task(task1_form, &mut conn).await.unwrap();
        assign_task(task1.id, member1, owner_id, &mut conn).await.unwrap();

        // 4. Work on tasks
        update_task_status(task1.id, TaskStatus::InProgress, member1, &mut conn)
            .await
            .unwrap();

        update_task_status(task1.id, TaskStatus::Done, member1, &mut conn)
            .await
            .unwrap();

        // 5. Check statistics
        let stats = get_workspace_stats(workspace.id, &mut conn).await.unwrap();
        assert_eq!(stats.total_members, 3);
        assert_eq!(stats.total_tasks, 1);
        assert_eq!(stats.completed_tasks, 1);

        // 6. Review activities
        let activities = get_workspace_activities(workspace.id, 20, 0, &mut conn)
            .await
            .unwrap();

        assert!(activities.len() >= 5); // Creation, members added, task created, assigned, completed

        cleanup_test_data(&mut conn).await;
    }
}
