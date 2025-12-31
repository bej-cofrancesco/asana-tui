//! Application-wide error types.
//!
//! This module defines the main error type hierarchy for the application,
//! allowing for type-safe error handling throughout the codebase.

pub use crate::asana::AsanaError;
pub use crate::config::ConfigError;
pub use crate::state::StateError;

/// Main application error type.
///
/// This is the top-level error type that encompasses all error types
/// in the application. It uses `thiserror` for automatic error derivation
/// and conversion.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// Configuration-related errors
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    /// Asana API-related errors
    #[error("Asana API error: {0}")]
    Asana(#[from] AsanaError),

    /// State management errors
    #[error("State error: {0}")]
    State(#[from] StateError),

    /// I/O errors
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Terminal/UI errors
    #[error("Terminal error: {0}")]
    Terminal(String),

    /// Logger initialization errors
    #[error("Logger error: {0}")]
    Logger(String),

    /// Runtime creation errors
    #[error("Failed to create runtime: {0}")]
    #[allow(dead_code)]
    RuntimeCreation(String),

    /// Generic error with context
    #[error("{0}")]
    #[allow(dead_code)]
    Other(String),
}

/// Convenience type alias for Result with AppError
pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_error_from_config_error() {
        let config_error = ConfigError::FilePathNotSet;
        let app_error: AppError = config_error.into();
        assert!(matches!(app_error, AppError::Config(_)));
        assert!(app_error.to_string().contains("Configuration error"));
    }

    #[test]
    fn test_app_error_from_asana_error() {
        let asana_error = AsanaError::Other("Test error".to_string());
        let app_error: AppError = asana_error.into();
        assert!(matches!(app_error, AppError::Asana(_)));
        assert!(app_error.to_string().contains("Asana API error"));
    }

    #[test]
    fn test_app_error_from_state_error() {
        let state_error = StateError::UserNotSet;
        let app_error: AppError = state_error.into();
        assert!(matches!(app_error, AppError::State(_)));
        assert!(app_error.to_string().contains("State error"));
    }

    #[test]
    fn test_app_error_from_io_error() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let app_error: AppError = io_error.into();
        assert!(matches!(app_error, AppError::Io(_)));
        assert!(app_error.to_string().contains("I/O error"));
    }

    #[test]
    fn test_app_error_terminal() {
        let error = AppError::Terminal("Terminal error".to_string());
        assert!(error.to_string().contains("Terminal error"));
    }

    #[test]
    fn test_app_error_other() {
        let error = AppError::Other("Generic error".to_string());
        assert_eq!(error.to_string(), "Generic error");
    }
}
