//! Configuration-specific error types.

use std::path::PathBuf;

/// Errors that can occur during configuration operations.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// File path was not set
    #[error("Configuration file path not set")]
    FilePathNotSet,

    /// Access token was not set
    #[error("Access token not set")]
    AccessTokenNotSet,

    /// Failed to find home directory
    #[error("Failed to find home directory")]
    HomeDirectoryNotFound,

    /// Failed to load configuration file
    #[error("Failed to load configuration from {path}: {message}")]
    LoadFailed { path: PathBuf, message: String },

    /// Failed to save configuration file
    #[error("Failed to save configuration to {path}: {source}")]
    SaveFailed {
        path: PathBuf,
        source: std::io::Error,
    },

    /// Failed to create configuration directory
    #[error("Failed to create configuration directory {path}: {source}")]
    CreateDirectoryFailed {
        path: PathBuf,
        source: std::io::Error,
    },

    /// Failed to serialize configuration
    #[error("Failed to serialize configuration: {0}")]
    SerializationFailed(String),

    /// Failed to deserialize configuration
    #[error("Failed to deserialize configuration: {0}")]
    DeserializationFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_error_display() {
        let error = ConfigError::FilePathNotSet;
        assert!(error.to_string().contains("file path not set"));

        let error = ConfigError::AccessTokenNotSet;
        assert!(error.to_string().contains("Access token not set"));

        let error = ConfigError::HomeDirectoryNotFound;
        assert!(error.to_string().contains("home directory"));

        let error = ConfigError::SerializationFailed("test".to_string());
        assert!(error.to_string().contains("test"));

        let error = ConfigError::DeserializationFailed("test".to_string());
        assert!(error.to_string().contains("test"));
    }

    #[test]
    fn test_config_error_with_path() {
        let path = PathBuf::from("/test/path");
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "Not found");
        let error = ConfigError::SaveFailed {
            path: path.clone(),
            source: io_error,
        };
        let error_str = error.to_string();
        assert!(error_str.contains("/test/path"));
    }
}
