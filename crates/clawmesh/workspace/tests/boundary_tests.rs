/// Agent Workspace Boundary Tests
/// DO-178C Level A Compliant Test Suite

#[cfg(test)]
mod workspace_boundary_tests {
    use clawmesh_workspace::models::*;

    // ========================================================================
    // Boundary Value Tests for WorkspaceForm
    // ========================================================================

    #[test]
    fn test_workspace_name_minimum_length() {
        let form = WorkspaceForm {
            owner_id: 1,
            name: "A".to_string(), // Minimum valid length
            description: None,
            is_public: true,
            max_members: 10,
        };
        assert!(form.validate().is_ok());
    }

    #[test]
    fn test_workspace_name_maximum_length() {
        let form = WorkspaceForm {
            owner_id: 1,
            name: "a".repeat(100), // Maximum valid length
            description: None,
            is_public: true,
            max_members: 10,
        };
        assert!(form.validate().is_ok());
    }

    #[test]
    fn test_workspace_name_exceeds_maximum() {
        let form = WorkspaceForm {
            owner_id: 1,
            name: "a".repeat(101), // Exceeds maximum
            description: None,
            is_public: true,
            max_members: 10,
        };
        assert!(form.validate().is_err());
    }

    #[test]
    fn test_workspace_description_maximum_length() {
        let form = WorkspaceForm {
            owner_id: 1,
            name: "Test".to_string(),
            description: Some("a".repeat(1000)), // Maximum valid length
            is_public: true,
            max_members: 10,
        };
        assert!(form.validate().is_ok());
    }

    #[test]
    fn test_workspace_description_exceeds_maximum() {
        let form = WorkspaceForm {
            owner_id: 1,
            name: "Test".to_string(),
            description: Some("a".repeat(1001)), // Exceeds maximum
            is_public: true,
            max_members: 10,
        };
        assert!(form.validate().is_err());
    }

    #[test]
    fn test_workspace_max_members_minimum() {
        let form = WorkspaceForm {
            owner_id: 1,
            name: "Test".to_string(),
            description: None,
            is_public: true,
            max_members: 1, // Minimum valid
        };
        assert!(form.validate().is_ok());
    }

    #[test]
    fn test_workspace_max_members_maximum() {
        let form = WorkspaceForm {
            owner_id: 1,
            name: "Test".to_string(),
            description: None,
            is_public: true,
            max_members: 100, // Maximum valid
        };
        assert!(form.validate().is_ok());
    }

    #[test]
    fn test_workspace_max_members_below_minimum() {
        let form = WorkspaceForm {
            owner_id: 1,
            name: "Test".to_string(),
            description: None,
            is_public: true,
            max_members: 0, // Below minimum
        };
        assert!(form.validate().is_err());
    }

    #[test]
    fn test_workspace_max_members_above_maximum() {
        let form = WorkspaceForm {
            owner_id: 1,
            name: "Test".to_string(),
            description: None,
            is_public: true,
            max_members: 101, // Above maximum
        };
        assert!(form.validate().is_err());
    }

    // ========================================================================
    // Boundary Value Tests for TaskForm
    // ========================================================================

    #[test]
    fn test_task_title_minimum_length() {
        let form = WorkspaceTaskForm {
            workspace_id: 1,
            title: "A".to_string(), // Minimum valid
            description: None,
            status: TaskStatus::Todo as i32,
            priority: TaskPriority::Medium as i32,
            assigned_to: None,
            created_by: 1,
            due_date: None,
        };
        assert!(form.validate().is_ok());
    }

    #[test]
    fn test_task_title_maximum_length() {
        let form = WorkspaceTaskForm {
            workspace_id: 1,
            title: "a".repeat(200), // Maximum valid
            description: None,
            status: TaskStatus::Todo as i32,
            priority: TaskPriority::Medium as i32,
            assigned_to: None,
            created_by: 1,
            due_date: None,
        };
        assert!(form.validate().is_ok());
    }

    #[test]
    fn test_task_title_exceeds_maximum() {
        let form = WorkspaceTaskForm {
            workspace_id: 1,
            title: "a".repeat(201), // Exceeds maximum
            description: None,
            status: TaskStatus::Todo as i32,
            priority: TaskPriority::Medium as i32,
            assigned_to: None,
            created_by: 1,
            due_date: None,
        };
        assert!(form.validate().is_err());
    }

    #[test]
    fn test_task_description_maximum_length() {
        let form = WorkspaceTaskForm {
            workspace_id: 1,
            title: "Test".to_string(),
            description: Some("a".repeat(5000)), // Maximum valid
            status: TaskStatus::Todo as i32,
            priority: TaskPriority::Medium as i32,
            assigned_to: None,
            created_by: 1,
            due_date: None,
        };
        assert!(form.validate().is_ok());
    }

    #[test]
    fn test_task_description_exceeds_maximum() {
        let form = WorkspaceTaskForm {
            workspace_id: 1,
            title: "Test".to_string(),
            description: Some("a".repeat(5001)), // Exceeds maximum
            status: TaskStatus::Todo as i32,
            priority: TaskPriority::Medium as i32,
            assigned_to: None,
            created_by: 1,
            due_date: None,
        };
        assert!(form.validate().is_err());
    }

    // ========================================================================
    // Status and Priority Boundary Tests
    // ========================================================================

    #[test]
    fn test_task_status_minimum_valid() {
        let form = WorkspaceTaskForm {
            workspace_id: 1,
            title: "Test".to_string(),
            description: None,
            status: 0, // Minimum valid (Todo)
            priority: TaskPriority::Medium as i32,
            assigned_to: None,
            created_by: 1,
            due_date: None,
        };
        assert!(form.validate().is_ok());
    }

    #[test]
    fn test_task_status_maximum_valid() {
        let form = WorkspaceTaskForm {
            workspace_id: 1,
            title: "Test".to_string(),
            description: None,
            status: 4, // Maximum valid (Cancelled)
            priority: TaskPriority::Medium as i32,
            assigned_to: None,
            created_by: 1,
            due_date: None,
        };
        assert!(form.validate().is_ok());
    }

    #[test]
    fn test_task_status_below_minimum() {
        let form = WorkspaceTaskForm {
            workspace_id: 1,
            title: "Test".to_string(),
            description: None,
            status: -1, // Below minimum
            priority: TaskPriority::Medium as i32,
            assigned_to: None,
            created_by: 1,
            due_date: None,
        };
        assert!(form.validate().is_err());
    }

    #[test]
    fn test_task_status_above_maximum() {
        let form = WorkspaceTaskForm {
            workspace_id: 1,
            title: "Test".to_string(),
            description: None,
            status: 5, // Above maximum
            priority: TaskPriority::Medium as i32,
            assigned_to: None,
            created_by: 1,
            due_date: None,
        };
        assert!(form.validate().is_err());
    }

    #[test]
    fn test_task_priority_minimum_valid() {
        let form = WorkspaceTaskForm {
            workspace_id: 1,
            title: "Test".to_string(),
            description: None,
            status: TaskStatus::Todo as i32,
            priority: 0, // Minimum valid (Low)
            assigned_to: None,
            created_by: 1,
            due_date: None,
        };
        assert!(form.validate().is_ok());
    }

    #[test]
    fn test_task_priority_maximum_valid() {
        let form = WorkspaceTaskForm {
            workspace_id: 1,
            title: "Test".to_string(),
            description: None,
            status: TaskStatus::Todo as i32,
            priority: 3, // Maximum valid (Critical)
            assigned_to: None,
            created_by: 1,
            due_date: None,
        };
        assert!(form.validate().is_ok());
    }

    #[test]
    fn test_task_priority_below_minimum() {
        let form = WorkspaceTaskForm {
            workspace_id: 1,
            title: "Test".to_string(),
            description: None,
            status: TaskStatus::Todo as i32,
            priority: -1, // Below minimum
            assigned_to: None,
            created_by: 1,
            due_date: None,
        };
        assert!(form.validate().is_err());
    }

    #[test]
    fn test_task_priority_above_maximum() {
        let form = WorkspaceTaskForm {
            workspace_id: 1,
            title: "Test".to_string(),
            description: None,
            status: TaskStatus::Todo as i32,
            priority: 4, // Above maximum
            assigned_to: None,
            created_by: 1,
            due_date: None,
        };
        assert!(form.validate().is_err());
    }

    // ========================================================================
    // Member Role Boundary Tests
    // ========================================================================

    #[test]
    fn test_member_role_minimum_valid() {
        let form = WorkspaceMemberForm {
            workspace_id: 1,
            agent_id: 2,
            role: 0, // Minimum valid (Owner)
        };
        assert!(form.validate().is_ok());
    }

    #[test]
    fn test_member_role_maximum_valid() {
        let form = WorkspaceMemberForm {
            workspace_id: 1,
            agent_id: 2,
            role: 3, // Maximum valid (Viewer)
        };
        assert!(form.validate().is_ok());
    }

    #[test]
    fn test_member_role_below_minimum() {
        let form = WorkspaceMemberForm {
            workspace_id: 1,
            agent_id: 2,
            role: -1, // Below minimum
        };
        assert!(form.validate().is_err());
    }

    #[test]
    fn test_member_role_above_maximum() {
        let form = WorkspaceMemberForm {
            workspace_id: 1,
            agent_id: 2,
            role: 4, // Above maximum
        };
        assert!(form.validate().is_err());
    }
}
