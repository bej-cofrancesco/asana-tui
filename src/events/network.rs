//! Network event handling module.
//!
//! This module handles all network-related events, including Asana API interactions,
//! task management, project operations, and data synchronization with the application state.

use crate::asana::Asana;
use crate::error::{AppError, AppResult};
use crate::state::State;
use crate::utils::text_processing::replace_profile_urls;
use log::*;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Specify different network event types.
///
#[derive(Debug, Clone)]
pub enum Event {
    SetAccessToken {
        token: String,
    },
    Me,
    ProjectTasks,
    UpdateTask {
        gid: String,
        completed: Option<bool>,
    },
    DeleteTask {
        gid: String,
    },
    #[allow(dead_code)]
    RefreshTasks,
    GetTaskDetail {
        gid: String,
    },
    GetProjectSections {
        project_gid: String,
    },
    GetProjectCustomFields {
        project_gid: String,
    },
    CreateStory {
        task_gid: String,
        text: String,
    },
    GetWorkspaceUsers {
        workspace_gid: String,
    },
    CreateTask {
        project_gid: String,
        name: String,
        notes: Option<String>,
        assignee: Option<String>,
        due_on: Option<String>,
        section: Option<String>,
        custom_fields: HashMap<String, crate::state::CustomFieldValue>,
    },
    UpdateTaskFields {
        gid: String,
        name: Option<String>,
        notes: Option<String>,
        assignee: Option<String>,
        due_on: Option<String>,
        section: Option<String>,
        completed: Option<bool>,
        custom_fields: HashMap<String, crate::state::CustomFieldValue>,
    },
    MoveTaskToSection {
        task_gid: String,
        section_gid: String,
    },
}

/// Specify struct for managing state with network events.
///
pub struct Handler<'a> {
    state: &'a Arc<RwLock<State>>,
    asana: &'a mut Asana,
}

impl<'a> Handler<'a> {
    /// Return new instance with reference to state.
    ///
    pub fn new(state: &'a Arc<RwLock<State>>, asana: &'a mut Asana) -> Self {
        Handler { state, asana }
    }

    /// Handle network events by type.
    ///
    pub async fn handle(&mut self, event: Event) -> AppResult<()> {
        debug!("Processing network event '{:?}'...", event);
        match event {
            Event::SetAccessToken { token } => {
                // Clear any previous error
                {
                    let mut state = self.state.write().await;
                    state.clear_auth_error();
                }

                // Try to save token and authenticate
                let result = async {
                    // Save token to config file - use the same pattern as original load()
                    // 1. Create new config and load it to set up file path
                    // 2. Save the token which will create the file
                    let mut config = crate::config::Config::new();

                    // Load config first to set up the file path (creates directory if needed)
                    // This is the same logic that was in the original load() method
                    config.load(None)?;

                    // Now save the token - this will create the file using create_file()
                    // This matches the original pattern where after getting the token,
                    // the file would be created immediately
                    config.save_token(token.clone())?;

                    info!("Access token saved to config file successfully.");

                    // Update Asana client with new token
                    *self.asana = crate::asana::Asana::new(&token);

                    // Fetch user data - this may fail if token is invalid
                    self.me().await?;

                    // If we get here, authentication succeeded
                    // Update state to mark as authenticated
                    {
                        let mut state = self.state.write().await;
                        state.set_access_token(token);
                        state.clear_access_token_input();
                        state.clear_auth_error();
                    }

                    info!("Access token saved and user authenticated.");
                    Ok::<(), anyhow::Error>(())
                }
                .await;

                // Handle errors gracefully
                if let Err(e) = result {
                    let error_msg = format!("Authentication failed: {}", e);
                    error!("{}", error_msg);

                    // Set error in state but keep has_access_token as false
                    // This allows user to see the error and resubmit
                    {
                        let mut state = self.state.write().await;
                        state.set_auth_error(Some(error_msg));
                        // Don't set has_access_token to true on error
                        // Don't clear the input so user can edit and resubmit
                    }

                    // Return Ok so the event handler doesn't crash
                    // The error is now stored in state and will be displayed in UI
                    return Ok(());
                }
            }
            Event::Me => self.me().await?,
            Event::ProjectTasks => self.project_tasks().await?,
            Event::UpdateTask { gid, completed } => self.update_task(gid, completed).await?,
            Event::DeleteTask { gid } => self.delete_task(gid).await?,
            Event::RefreshTasks => self.refresh_tasks().await?,
            Event::GetTaskDetail { gid } => self.get_task_detail(gid).await?,
            Event::GetProjectSections { project_gid } => {
                self.get_project_sections(project_gid).await?
            }
            Event::GetProjectCustomFields { project_gid } => {
                self.get_project_custom_fields(project_gid).await?
            }
            Event::CreateStory { task_gid, text } => self.create_story(task_gid, text).await?,
            Event::GetWorkspaceUsers { workspace_gid } => {
                self.get_workspace_users(workspace_gid).await?
            }
            Event::CreateTask {
                project_gid,
                name,
                notes,
                assignee,
                due_on,
                section,
                custom_fields,
            } => {
                self.create_task(
                    project_gid,
                    name,
                    notes,
                    assignee,
                    due_on,
                    section,
                    custom_fields,
                )
                .await?
            }
            Event::UpdateTaskFields {
                gid,
                name,
                notes,
                assignee,
                due_on,
                section,
                completed,
                custom_fields,
            } => {
                self.update_task_fields(
                    gid,
                    name,
                    notes,
                    assignee,
                    due_on,
                    section,
                    completed,
                    custom_fields,
                )
                .await?
            }
            Event::MoveTaskToSection {
                task_gid,
                section_gid,
            } => self.move_task_to_section(task_gid, section_gid).await?,
        }
        Ok(())
    }

    /// Update state with user details and projects for active workspace.
    ///
    async fn me(&mut self) -> AppResult<()> {
        info!("Preparing initial application data...");
        info!("Fetching user details and available workspaces...");
        let (user, workspaces) = self
            .asana
            .me()
            .await
            .map_err(|e| AppError::Asana(crate::asana::AsanaError::Other(e.to_string())))?;
        {
            let mut state = self.state.write().await;
            state.set_user(user);
            if !workspaces.is_empty() {
                state.set_workspaces(workspaces.clone());
                state.set_active_workspace(workspaces[0].gid.clone());
            }
        }
        if !workspaces.is_empty() {
            info!("Fetching projects for active workspace...");
            let projects = self
                .asana
                .projects(&workspaces[0].gid)
                .await
                .map_err(|e| AppError::Asana(crate::asana::AsanaError::Other(e.to_string())))?;
            let mut state = self.state.write().await;
            state.set_projects(projects);
        }
        info!("Loaded initial application data.");
        Ok(())
    }

    /// Update state with tasks for project.
    ///
    async fn project_tasks(&mut self) -> AppResult<()> {
        let project;
        let workspace_gid;
        {
            let state = self.state.read().await;
            project = match state.get_project() {
                Some(p) => p.clone(),
                None => {
                    warn!("Skipping tasks request for unset project.");
                    return Ok(());
                }
            };
            workspace_gid = state.get_active_workspace().map(|w| w.gid.clone());
        }
        info!(
            "Fetching tasks for project '{}' (GID: {})...",
            &project.name, &project.gid
        );
        // Always include completed tasks since we're using kanban view
        let include_completed = true;
        let view_mode;
        {
            let state = self.state.read().await;
            view_mode = state.get_view_mode();
        }

        let tasks_result = self
            .asana
            .tasks(&project.gid, workspace_gid.as_deref(), include_completed)
            .await
            .map_err(|e| AppError::Asana(crate::asana::AsanaError::Other(e.to_string())));

        match tasks_result {
            Ok(tasks) => {
                info!(
                    "Received {} tasks for project '{}'.",
                    tasks.len(),
                    &project.name
                );
                let mut state = self.state.write().await;
                state.set_tasks(tasks);
                // If in kanban mode, also load sections
                if view_mode == crate::state::ViewMode::Kanban {
                    let project_gid = project.gid.clone();
                    drop(state);
                    self.get_project_sections(project_gid).await?;
                }
                Ok(())
            }
            Err(e) => {
                error!(
                    "Failed to fetch tasks for project '{}' (GID: {}): {}",
                    &project.name, &project.gid, e
                );
                // Log the full error chain
                let mut source = e.source();
                while let Some(err) = source {
                    error!("  Project tasks error chain - Caused by: {}", err);
                    source = err.source();
                }
                Err(e)
            }
        }
    }

    /// Update state with tasks assigned to the user.
    ///
    async fn update_task(&mut self, task_gid: String, completed: Option<bool>) -> AppResult<()> {
        self.asana
            .update_task(&task_gid, completed)
            .await
            .map_err(|e| AppError::Asana(crate::asana::AsanaError::Other(e.to_string())))?;
        info!("Task {} updated successfully.", task_gid);
        // Refresh the current task list
        self.refresh_current_view().await?;
        Ok(())
    }

    /// Delete a task.
    ///
    async fn delete_task(&mut self, task_gid: String) -> AppResult<()> {
        info!("Deleting task {}...", task_gid);
        self.asana
            .delete_task(&task_gid)
            .await
            .map_err(|e| AppError::Asana(crate::asana::AsanaError::Other(e.to_string())))?;
        info!("Task {} deleted successfully.", task_gid);
        // Refresh the current task list
        self.refresh_current_view().await?;
        Ok(())
    }

    /// Refresh the current task list.
    ///
    async fn refresh_tasks(&mut self) -> AppResult<()> {
        self.refresh_current_view().await?;
        Ok(())
    }

    /// Refresh the current view if it's ProjectTasks.
    ///
    async fn refresh_current_view(&mut self) -> AppResult<()> {
        let view;
        {
            let state = self.state.read().await;
            view = state.current_view().clone();
        }
        match view {
            crate::state::View::ProjectTasks => {
                self.project_tasks().await?;
            }
            _ => {}
        }
        Ok(())
    }

    /// Get full task details.
    ///
    async fn get_task_detail(&mut self, task_gid: String) -> AppResult<()> {
        info!("Fetching full details for task {}...", task_gid);
        let mut task = self
            .asana
            .get_task(&task_gid)
            .await
            .map_err(|e| AppError::Asana(crate::asana::AsanaError::Other(e.to_string())))?;

        // Process task notes - replace profile URLs with @username when data comes from API
        let user_map: HashMap<String, String> = {
            let state = self.state.read().await;
            state
                .get_workspace_users()
                .iter()
                .map(|u| (u.gid.clone(), u.name.clone()))
                .collect()
        };

        // Process notes if present
        if let Some(ref mut notes) = task.notes {
            *notes = replace_profile_urls(notes, &user_map);
        }

        let (workspace_gid, project_gid) = {
            let state = self.state.read().await;
            (
                state.get_active_workspace().map(|w| w.gid.clone()),
                state.get_project().map(|p| p.gid.clone()),
            )
        };
        {
            let mut state = self.state.write().await;
            state.set_task_detail(task);
        }

        // Fetch assignees, sections, and custom fields on the fly
        if let Some(workspace_gid) = workspace_gid {
            self.get_workspace_users(workspace_gid).await?;
        }
        if let Some(project_gid) = project_gid {
            let project_gid_clone = project_gid.clone();
            self.get_project_sections(project_gid_clone).await?;
            self.get_project_custom_fields(project_gid).await?;
        }

        // Also load stories/comments (which will also be processed)
        self.get_task_stories(task_gid).await?;
        info!("Task details loaded successfully.");
        Ok(())
    }

    /// Get project sections for kanban board.
    ///
    async fn get_project_sections(&mut self, project_gid: String) -> AppResult<()> {
        info!("Fetching sections for project {}...", project_gid);
        let sections = self
            .asana
            .get_project_sections(&project_gid)
            .await
            .map_err(|e| AppError::Asana(crate::asana::AsanaError::Other(e.to_string())))?;
        let mut state = self.state.write().await;
        state.set_sections(sections);
        info!("Sections loaded successfully.");
        Ok(())
    }

    /// Get custom fields for a project.
    ///
    async fn get_project_custom_fields(&mut self, project_gid: String) -> AppResult<()> {
        info!("Fetching custom fields for project {}...", project_gid);
        let custom_fields = self
            .asana
            .get_project_custom_fields(&project_gid)
            .await
            .map_err(|e| AppError::Asana(crate::asana::AsanaError::Other(e.to_string())))?;

        {
            let mut state = self.state.write().await;
            state.set_project_custom_fields(custom_fields);
        }

        info!("Custom fields loaded successfully.");
        Ok(())
    }

    // replace_profile_urls is now in utils::text_processing

    /// Get task stories/comments.
    ///
    async fn get_task_stories(&mut self, task_gid: String) -> AppResult<()> {
        info!("Fetching stories/comments for task {}...", task_gid);
        let mut stories = self
            .asana
            .get_task_stories(&task_gid)
            .await
            .map_err(|e| AppError::Asana(crate::asana::AsanaError::Other(e.to_string())))?;

        // Process URLs when data comes from API - replace profile URLs with @username
        let user_map: HashMap<String, String> = {
            let state = self.state.read().await;
            state
                .get_workspace_users()
                .iter()
                .map(|u| (u.gid.clone(), u.name.clone()))
                .collect()
        };

        // Process each story's text to replace profile URLs
        for story in &mut stories {
            story.text = replace_profile_urls(&story.text, &user_map);
        }

        {
            let mut state = self.state.write().await;
            state.set_task_stories(stories);
        }
        info!("Stories/comments loaded successfully.");
        Ok(())
    }

    /// Create a story/comment on a task.
    ///
    async fn create_story(&mut self, task_gid: String, text: String) -> AppResult<()> {
        info!("Creating comment on task {}...", task_gid);
        self.asana
            .create_story(&task_gid, &text)
            .await
            .map_err(|e| AppError::Asana(crate::asana::AsanaError::Other(e.to_string())))?;
        // Refresh stories after creating
        self.get_task_stories(task_gid).await?;
        info!("Comment created successfully.");
        Ok(())
    }

    /// Get workspace users.
    ///
    async fn get_workspace_users(&mut self, workspace_gid: String) -> AppResult<()> {
        info!("Fetching users for workspace {}...", workspace_gid);
        let users = self
            .asana
            .get_workspace_users(&workspace_gid)
            .await
            .map_err(|e| AppError::Asana(crate::asana::AsanaError::Other(e.to_string())))?;
        let mut state = self.state.write().await;
        state.set_workspace_users(users);
        info!("Users loaded successfully.");
        Ok(())
    }

    /// Create a new task.
    ///
    async fn create_task(
        &mut self,
        project_gid: String,
        name: String,
        notes: Option<String>,
        assignee: Option<String>,
        due_on: Option<String>,
        section: Option<String>,
        custom_fields: HashMap<String, crate::state::CustomFieldValue>,
    ) -> AppResult<()> {
        info!("Creating new task '{}' in project {}...", name, project_gid);
        let task = self
            .asana
            .create_task(
                &project_gid,
                &name,
                notes.as_deref(),
                assignee.as_deref(),
                due_on.as_deref(),
                section.as_deref(),
                &custom_fields,
            )
            .await
            .map_err(|e| AppError::Asana(crate::asana::AsanaError::Other(e.to_string())))?;

        info!(
            "Task '{}' created successfully with GID {}",
            task.name, task.gid
        );

        // Refresh tasks after creating - need to get project from state
        let project_gid_for_refresh = {
            let state = self.state.read().await;
            state
                .get_project()
                .map(|p| p.gid.clone())
                .unwrap_or_else(|| project_gid.clone())
        };

        // Refresh the project tasks
        self.project_tasks().await?;

        // If in kanban mode, also refresh sections
        let should_refresh_sections = {
            let state = self.state.read().await;
            state.get_view_mode() == crate::state::ViewMode::Kanban
        };
        if should_refresh_sections {
            self.get_project_sections(project_gid_for_refresh).await?;
        }

        Ok(())
    }

    /// Update task fields.
    ///
    async fn update_task_fields(
        &mut self,
        gid: String,
        name: Option<String>,
        notes: Option<String>,
        assignee: Option<String>,
        due_on: Option<String>,
        section: Option<String>,
        completed: Option<bool>,
        custom_fields: HashMap<String, crate::state::CustomFieldValue>,
    ) -> AppResult<()> {
        self.asana
            .update_task_fields(
                &gid,
                name.as_deref(),
                notes.as_deref(),
                assignee.as_deref(),
                due_on.as_deref(),
                section.as_deref(),
                completed,
                &custom_fields,
            )
            .await
            .map_err(|e| AppError::Asana(crate::asana::AsanaError::Other(e.to_string())))?;
        // Refresh task detail if we're viewing it
        let (view, task_gid_to_refresh) = {
            let state = self.state.read().await;
            let view = state.current_view().clone();
            let task_gid_to_refresh = if matches!(view, crate::state::View::TaskDetail) {
                state.get_task_detail().map(|t| t.gid.clone())
            } else {
                None
            };
            (view, task_gid_to_refresh)
        };
        if let Some(gid) = task_gid_to_refresh {
            self.get_task_detail(gid).await?;
            return Ok(());
        }
        // Otherwise refresh task list
        if matches!(view, crate::state::View::ProjectTasks) {
            self.project_tasks().await?;
        }
        info!("Task updated successfully.");
        Ok(())
    }

    /// Move task to a different section.
    ///
    async fn move_task_to_section(
        &mut self,
        task_gid: String,
        section_gid: String,
    ) -> AppResult<()> {
        self.asana
            .add_task_to_section(&task_gid, &section_gid)
            .await
            .map_err(|e| AppError::Asana(crate::asana::AsanaError::Other(e.to_string())))?;
        // Refresh tasks after moving
        self.project_tasks().await?;
        info!("Task moved successfully.");
        Ok(())
    }
}
