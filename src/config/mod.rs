//! Configuration management module.
//!
//! This module handles loading, saving, and managing application configuration,
//! including API tokens, starred projects, and theme preferences.

mod error;

pub use error::ConfigError;

use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

const FILE_NAME: &str = "config.yml";
const DEFAULT_DIRECTORY_PATH: &str = ".config/asana-tui";

/// Oversees management of configuration file.
///
#[derive(Clone)]
pub struct Config {
    pub access_token: Option<String>,
    pub starred_projects: Vec<String>, // GIDs
    pub starred_project_names: std::collections::HashMap<String, String>, // GID -> Name
    pub theme_name: String,
    file_path: Option<PathBuf>,
}

/// Define specification for configuration file.
///
#[derive(Serialize, Deserialize)]
struct FileSpec {
    pub access_token: String,
    #[serde(default)]
    pub starred_projects: Vec<String>, // GIDs
    #[serde(default)]
    pub starred_project_names: std::collections::HashMap<String, String>, // GID -> Name
    #[serde(default = "default_theme_name")]
    pub theme_name: String,
}

fn default_theme_name() -> String {
    "tokyo-night".to_string()
}

impl Config {
    /// Return a new empty instance.
    ///
    pub fn new() -> Config {
        Config {
            file_path: None,
            access_token: None,
            starred_projects: vec![],
            starred_project_names: std::collections::HashMap::new(),
            theme_name: default_theme_name(),
        }
    }

    /// Try to load an existing configuration from the disk using the custom
    /// path if provided. If the file cannot be loaded, authorize with the
    /// user and initialize the configuration file with the new token at the
    /// default file path or the custom path if provided.
    ///
    pub fn load(&mut self, custom_path: Option<&str>) -> Result<(), AppError> {
        // Use default path unless custom path provided
        let dir_path = match custom_path {
            Some(path) => Path::new(&path).to_path_buf(),
            None => Config::default_path()?,
        };

        // Try to create dir path if it doesn't exist
        if !dir_path.exists() {
            fs::create_dir_all(&dir_path).map_err(|e| ConfigError::CreateDirectoryFailed {
                path: dir_path.clone(),
                source: e,
            })?;
        }

        // Specify config file path
        self.file_path = Some(dir_path.join(Path::new(FILE_NAME)));
        let file_path = self.file_path.as_ref().ok_or(ConfigError::FilePathNotSet)?;

        // If file exists, try to extract token and starred projects
        if file_path.exists() {
            let contents = fs::read_to_string(file_path).map_err(|e| ConfigError::LoadFailed {
                path: file_path.clone(),
                message: format!("IO error: {}", e),
            })?;
            let data: FileSpec = serde_yaml::from_str(&contents)
                .map_err(|e| ConfigError::DeserializationFailed(e.to_string()))?;
            self.access_token = Some(data.access_token);
            self.starred_projects = data.starred_projects;
            self.starred_project_names = data.starred_project_names;
            self.theme_name = data.theme_name;
        }
        // Otherwise, leave access_token as None - will be handled in TUI onboarding
        // Don't prompt via stdin, let the TUI handle it

        Ok(())
    }

    /// Attempt to serialize the configuration data and write it to the disk,
    /// returning any unrecoverable errors.
    ///
    fn create_file(&self) -> Result<(), AppError> {
        let file_path = self.file_path.as_ref().ok_or(ConfigError::FilePathNotSet)?;
        let access_token = self
            .access_token
            .as_ref()
            .ok_or(ConfigError::AccessTokenNotSet)?;

        let data = FileSpec {
            access_token: access_token.clone(),
            starred_projects: self.starred_projects.clone(),
            starred_project_names: self.starred_project_names.clone(),
            theme_name: self.theme_name.clone(),
        };
        let content = serde_yaml::to_string(&data)
            .map_err(|e| ConfigError::SerializationFailed(e.to_string()))?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = file_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(|e| ConfigError::CreateDirectoryFailed {
                    path: parent.to_path_buf(),
                    source: e,
                })?;
            }
        }

        let mut file = fs::File::create(file_path).map_err(|e| ConfigError::SaveFailed {
            path: file_path.clone(),
            source: e,
        })?;
        write!(file, "{}", content).map_err(|e| ConfigError::SaveFailed {
            path: file_path.clone(),
            source: e,
        })?;
        file.flush().map_err(|e| ConfigError::SaveFailed {
            path: file_path.clone(),
            source: e,
        })?; // Ensure data is written to disk
        Ok(())
    }

    /// Save the current configuration to disk.
    ///
    pub fn save(&self) -> Result<(), AppError> {
        if self.file_path.is_none() {
            return Err(ConfigError::FilePathNotSet.into());
        }
        let data = FileSpec {
            access_token: self
                .access_token
                .clone()
                .ok_or(ConfigError::AccessTokenNotSet)?,
            starred_projects: self.starred_projects.clone(),
            starred_project_names: self.starred_project_names.clone(),
            theme_name: self.theme_name.clone(),
        };
        let content = serde_yaml::to_string(&data)
            .map_err(|e| ConfigError::SerializationFailed(e.to_string()))?;
        let file_path = self.file_path.as_ref().ok_or(ConfigError::FilePathNotSet)?;
        let mut file = fs::File::create(file_path).map_err(|e| ConfigError::SaveFailed {
            path: file_path.clone(),
            source: e,
        })?;
        write!(file, "{}", content).map_err(|e| ConfigError::SaveFailed {
            path: file_path.clone(),
            source: e,
        })?;
        Ok(())
    }

    /// Save access token to config file.
    ///
    pub fn save_token(&mut self, token: String) -> Result<(), AppError> {
        self.access_token = Some(token.clone());
        // Ensure file path is set
        if self.file_path.is_none() {
            let dir_path = Config::default_path()?;
            if !dir_path.exists() {
                fs::create_dir_all(&dir_path).map_err(|e| ConfigError::CreateDirectoryFailed {
                    path: dir_path.clone(),
                    source: e,
                })?;
            }
            self.file_path = Some(dir_path.join(Path::new(FILE_NAME)));
        }

        // Ensure directory exists (in case it was deleted)
        if let Some(file_path) = &self.file_path {
            if let Some(parent) = file_path.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent).map_err(|e| ConfigError::CreateDirectoryFailed {
                        path: parent.to_path_buf(),
                        source: e,
                    })?;
                }
            }
        }

        self.create_file()?;
        Ok(())
    }

    /// Returns the path buffer for the default path to the configuration file
    /// or an error if the home directory could not be found.
    ///
    fn default_path() -> Result<PathBuf, AppError> {
        match dirs::home_dir() {
            Some(home) => {
                let home_path = Path::new(&home);
                let default_config_path = Path::new(DEFAULT_DIRECTORY_PATH);
                Ok(home_path.join(default_config_path))
            }
            None => Err(ConfigError::HomeDirectoryNotFound.into()),
        }
    }
}
