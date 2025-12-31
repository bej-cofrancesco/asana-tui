//! State management-specific error types.

/// Errors that can occur during state operations.
#[derive(Debug, thiserror::Error)]
pub enum StateError {
    /// User not set in state
    #[error("User not set in state")]
    #[allow(dead_code)]
    UserNotSet,

    /// Workspace not set in state
    #[error("Workspace not set in state")]
    #[allow(dead_code)]
    WorkspaceNotSet,

    /// Project not set in state
    #[error("Project not set in state")]
    #[allow(dead_code)]
    ProjectNotSet,

    /// Task not found in state
    #[error("Task not found: {gid}")]
    #[allow(dead_code)]
    TaskNotFound { gid: String },

    /// Invalid view transition
    #[error("Invalid view transition: {0}")]
    #[allow(dead_code)]
    InvalidViewTransition(String),

    /// State lock timeout
    #[error("State lock timeout")]
    #[allow(dead_code)]
    LockTimeout,

    /// Generic state error
    #[error("State error: {0}")]
    #[allow(dead_code)]
    Other(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_error_display() {
        let error = StateError::UserNotSet;
        assert!(error.to_string().contains("User not set"));

        let error = StateError::WorkspaceNotSet;
        assert!(error.to_string().contains("Workspace not set"));

        let error = StateError::ProjectNotSet;
        assert!(error.to_string().contains("Project not set"));

        let error = StateError::TaskNotFound {
            gid: "123456".to_string(),
        };
        assert!(error.to_string().contains("Task not found"));
        assert!(error.to_string().contains("123456"));

        let error = StateError::InvalidViewTransition("Invalid".to_string());
        assert!(error.to_string().contains("Invalid view transition"));
        assert!(error.to_string().contains("Invalid"));

        let error = StateError::LockTimeout;
        assert!(error.to_string().contains("State lock timeout"));

        let error = StateError::Other("Generic error".to_string());
        assert!(error.to_string().contains("State error"));
        assert!(error.to_string().contains("Generic error"));
    }
}
