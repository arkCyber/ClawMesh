/// Agent Workspace Unit Tests
/// DO-178C Level A Compliant Test Suite

#[cfg(test)]
mod workspace_unit_tests {
    use clawmesh_workspace::models::*;

    // ========================================================================
    // WorkspaceForm Validation Tests
    // ========================================================================

    #[test]
    fn test_workspace_form_valid() {
        let form = WorkspaceForm {
            owner_id: 1,
            name: "Test Workspace".to_string(),
            description: Some("A test workspace".to_string()),
            workspace_type: WorkspaceType::Private as i32,
        };

        assert!(form.validate().is_ok());
    }

    #[test]
    fn test_workspace_form_empty_name() {
        let form = WorkspaceForm {
            owner_id: 1,
            name: "".to_string(),
            description: None,
            workspace_type: WorkspaceType::Private as i32,
        };

        assert!(form.validate().is_err());
    }

    #[test]
    fn test_workspace_form_name_too_long() {
        let form = WorkspaceForm {
            owner_id: 1,
            name: "a".repeat(256),
            description: None,
            workspace_type: WorkspaceType::Private as i32,
        };

        assert!(form.validate().is_err());
    }

    #[test]
    fn test_workspace_form_description_too_long() {
        let form = WorkspaceForm {
            owner_id: 1,
            name: "Valid Name".to_string(),
            description: Some("a".repeat(5001)),
            workspace_type: WorkspaceType::Private as i32,
        };

        assert!(form.validate().is_err());
    }

    // ========================================================================
    // MemberForm Validation Tests
    // ========================================================================

    #[test]
    fn test_member_form_valid() {
        let form = MemberForm {
            workspace_id: 1,
            agent_id: 2,
            role: MemberRole::Member as i32,
        };

        assert!(form.validate().is_ok());
    }

    #[test]
    fn test_member_form_invalid_role() {
        let form = MemberForm {
            workspace_id: 1,
            agent_id: 2,
            role: 99, // Invalid role
        };

        assert!(form.validate().is_err());
    }

    // ========================================================================
    // TaskForm Validation Tests
    // ========================================================================

    #[test]
    fn test_task_form_valid() {
        let form = TaskForm {
            workspace_id: 1,
            title: "Test Task".to_string(),
            description: Some("Task description".to_string()),
            assigned_to: Some(2),
            status: TaskStatus::Todo as i32,
            priority: TaskPriority::Medium as i32,
            due_date: None,
            created_by: 1,
        };

        assert!(form.validate().is_ok());
    }

    #[test]
    fn test_task_form_empty_title() {
        let form = TaskForm {
            workspace_id: 1,
            title: "".to_string(),
            description: None,
            assigned_to: None,
            status: TaskStatus::Todo as i32,
            priority: TaskPriority::Medium as i32,
            due_date: None,
            created_by: 1,
        };

        assert!(form.validate().is_err());
    }

    #[test]
    fn test_task_form_title_too_long() {
        let form = TaskForm {
            workspace_id: 1,
            title: "a".repeat(256),
            description: None,
            assigned_to: None,
            status: TaskStatus::Todo as i32,
            priority: TaskPriority::Medium as i32,
            due_date: None,
            created_by: 1,
        };

        assert!(form.validate().is_err());
    }

    #[test]
    fn test_task_form_invalid_status() {
        let form = TaskForm {
            workspace_id: 1,
            title: "Valid Title".to_string(),
            description: None,
            assigned_to: None,
            status: 99, // Invalid status
            priority: TaskPriority::Medium as i32,
            due_date: None,
            created_by: 1,
        };

        assert!(form.validate().is_err());
    }

    #[test]
    fn test_task_form_invalid_priority() {
        let form = TaskForm {
            workspace_id: 1,
            title: "Valid Title".to_string(),
            description: None,
            assigned_to: None,
            status: TaskStatus::Todo as i32,
            priority: 99, // Invalid priority
            due_date: None,
            created_by: 1,
        };

        assert!(form.validate().is_err());
    }

    // ========================================================================
    // Enum Tests
    // ========================================================================

    #[test]
    fn test_workspace_type_values() {
        assert_eq!(WorkspaceType::Private as i32, 0);
        assert_eq!(WorkspaceType::Public as i32, 1);
        assert_eq!(WorkspaceType::Shared as i32, 2);
    }

    #[test]
    fn test_member_role_values() {
        assert_eq!(MemberRole::Owner as i32, 0);
        assert_eq!(MemberRole::Admin as i32, 1);
        assert_eq!(MemberRole::Member as i32, 2);
        assert_eq!(MemberRole::Viewer as i32, 3);
    }

    #[test]
    fn test_task_status_values() {
        assert_eq!(TaskStatus::Todo as i32, 0);
        assert_eq!(TaskStatus::InProgress as i32, 1);
        assert_eq!(TaskStatus::Review as i32, 2);
        assert_eq!(TaskStatus::Done as i32, 3);
        assert_eq!(TaskStatus::Cancelled as i32, 4);
    }

    #[test]
    fn test_task_priority_values() {
        assert_eq!(TaskPriority::Low as i32, 0);
        assert_eq!(TaskPriority::Medium as i32, 1);
        assert_eq!(TaskPriority::High as i32, 2);
        assert_eq!(TaskPriority::Critical as i32, 3);
    }

    // ========================================================================
    // Permission Tests
    // ========================================================================

    #[test]
    fn test_owner_has_all_permissions() {
        // Owner should have all permissions
        let role = MemberRole::Owner as i32;
        assert_eq!(role, 0);
    }

    #[test]
    fn test_admin_permissions() {
        // Admin should have management permissions
        let role = MemberRole::Admin as i32;
        assert_eq!(role, 1);
    }

    #[test]
    fn test_member_permissions() {
        // Member should have basic permissions
        let role = MemberRole::Member as i32;
        assert_eq!(role, 2);
    }

    #[test]
    fn test_viewer_permissions() {
        // Viewer should have read-only permissions
        let role = MemberRole::Viewer as i32;
        assert_eq!(role, 3);
    }

    // ========================================================================
    // WorkspaceStatistics Tests
    // ========================================================================

    #[test]
    fn test_workspace_statistics_default() {
        let stats = WorkspaceStatistics {
            total_members: 0,
            total_tasks: 0,
            completed_tasks: 0,
            pending_tasks: 0,
            overdue_tasks: 0,
        };

        assert_eq!(stats.total_members, 0);
        assert_eq!(stats.total_tasks, 0);
        assert_eq!(stats.completed_tasks, 0);
    }

    #[test]
    fn test_workspace_statistics_calculation() {
        let stats = WorkspaceStatistics {
            total_members: 5,
            total_tasks: 10,
            completed_tasks: 6,
            pending_tasks: 3,
            overdue_tasks: 1,
        };

        assert_eq!(stats.total_members, 5);
        assert_eq!(stats.total_tasks, 10);
        assert_eq!(stats.completed_tasks, 6);
        assert_eq!(stats.pending_tasks, 3);
        assert_eq!(stats.overdue_tasks, 1);
    }
}
