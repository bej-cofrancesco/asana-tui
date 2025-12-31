//! Asana API-specific error types.

/// Errors that can occur during Asana API operations.
#[derive(Debug, thiserror::Error)]
pub enum AsanaError {
    /// HTTP request failed
    #[error("HTTP request failed: {0}")]
    HttpRequest(#[from] reqwest::Error),

    /// API returned an error response
    #[error("API error (status {status}): {message}")]
    #[allow(dead_code)]
    ApiError { status: u16, message: String },

    /// Failed to deserialize API response
    #[error("Failed to deserialize API response: {0}")]
    Deserialization(#[from] serde_json::Error),

    /// Invalid custom field value
    #[error("Invalid custom field value: {0}")]
    #[allow(dead_code)]
    InvalidCustomField(String),

    /// Custom field validation failed
    #[error("Custom field validation failed: {0}")]
    #[allow(dead_code)]
    CustomFieldValidation(String),

    /// Task not found
    #[error("Task not found: {gid}")]
    #[allow(dead_code)]
    TaskNotFound { gid: String },

    /// Project not found
    #[error("Project not found: {gid}")]
    #[allow(dead_code)]
    ProjectNotFound { gid: String },

    /// Section not found
    #[error("Section not found: {gid}")]
    #[allow(dead_code)]
    SectionNotFound { gid: String },

    /// Generic API error
    #[error("Asana API error: {0}")]
    Other(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asana_error_display() {
        let error = AsanaError::Other("Test error".to_string());
        assert!(error.to_string().contains("Asana API error"));
        assert!(error.to_string().contains("Test error"));

        let error = AsanaError::InvalidCustomField("Invalid field".to_string());
        assert!(error.to_string().contains("Invalid custom field value"));

        let error = AsanaError::TaskNotFound {
            gid: "123456".to_string(),
        };
        assert!(error.to_string().contains("Task not found"));
        assert!(error.to_string().contains("123456"));

        let error = AsanaError::ProjectNotFound {
            gid: "789012".to_string(),
        };
        assert!(error.to_string().contains("Project not found"));
        assert!(error.to_string().contains("789012"));
    }

    #[test]
    fn test_asana_error_api_error() {
        let error = AsanaError::ApiError {
            status: 404,
            message: "Not found".to_string(),
        };
        let error_str = error.to_string();
        assert!(error_str.contains("404"));
        assert!(error_str.contains("Not found"));
    }
}
