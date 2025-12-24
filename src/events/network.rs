use crate::asana::Asana;
use crate::state::State;
use anyhow::Result;
use log::*;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Specify different network event types.
///
#[derive(Debug, Clone)]
pub enum Event {
    Me,
    ProjectTasks,
    MyTasks,
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
    },
    UpdateTaskFields {
        gid: String,
        name: Option<String>,
        notes: Option<String>,
        assignee: Option<String>,
        due_on: Option<String>,
        section: Option<String>,
        completed: Option<bool>,
    },
}

/// Specify struct for managing state with network events.
///
pub struct Handler<'a> {
    state: &'a Arc<Mutex<State>>,
    asana: &'a mut Asana,
}

impl<'a> Handler<'a> {
    /// Return new instance with reference to state.
    ///
    pub fn new(state: &'a Arc<Mutex<State>>, asana: &'a mut Asana) -> Self {
        Handler { state, asana }
    }

    /// Handle network events by type.
    ///
    pub async fn handle(&mut self, event: Event) -> Result<()> {
        debug!("Processing network event '{:?}'...", event);
        match event {
            Event::Me => self.me().await?,
            Event::ProjectTasks => self.project_tasks().await?,
            Event::MyTasks => self.my_tasks().await?,
            Event::UpdateTask { gid, completed } => self.update_task(gid, completed).await?,
            Event::DeleteTask { gid } => self.delete_task(gid).await?,
            Event::RefreshTasks => self.refresh_tasks().await?,
            Event::GetTaskDetail { gid } => self.get_task_detail(gid).await?,
            Event::GetProjectSections { project_gid } => self.get_project_sections(project_gid).await?,
            Event::CreateStory { task_gid, text } => self.create_story(task_gid, text).await?,
            Event::GetWorkspaceUsers { workspace_gid } => self.get_workspace_users(workspace_gid).await?,
            Event::CreateTask { project_gid, name, notes, assignee, due_on, section } => {
                self.create_task(project_gid, name, notes, assignee, due_on, section).await?
            },
            Event::UpdateTaskFields { gid, name, notes, assignee, due_on, section, completed } => {
                self.update_task_fields(gid, name, notes, assignee, due_on, section, completed).await?
            }
        }
        Ok(())
    }

    /// Update state with user details and projects for active workspace.
    ///
    async fn me(&mut self) -> Result<()> {
        info!("Preparing initial application data...");
        info!("Fetching user details and available workspaces...");
        let (user, workspaces) = self.asana.me().await?;
        {
            let mut state = self.state.lock().await;
            state.set_user(user);
            if !workspaces.is_empty() {
                state.set_workspaces(workspaces.to_owned());
                state.set_active_workspace(workspaces[0].gid.to_owned());
            }
        }
        if !workspaces.is_empty() {
            info!("Fetching projects for active workspace...");
            let projects = self.asana.projects(&workspaces[0].gid).await?;
            let mut state = self.state.lock().await;
            state.set_projects(projects);
        }
        info!("Loaded initial application data.");
        Ok(())
    }

    /// Update state with tasks for project.
    ///
    async fn project_tasks(&mut self) -> Result<()> {
        let project;
        let workspace_gid;
        {
            let state = self.state.lock().await;
            if state.get_project().is_none() {
                warn!("Skipping tasks request for unset project.");
                return Ok(());
            }
            project = state.get_project().unwrap().to_owned();
            workspace_gid = state.get_active_workspace().map(|w| w.gid.to_owned());
        }
        info!(
            "Fetching tasks for project '{}' (GID: {})...",
            &project.name, &project.gid
        );
        // Determine if we should include completed tasks based on current filter
        let include_completed;
        let view_mode;
        {
            let state = self.state.lock().await;
            include_completed = matches!(state.get_task_filter(), crate::state::TaskFilter::All | crate::state::TaskFilter::Completed);
            view_mode = state.get_view_mode();
        }
        
        match self
            .asana
            .tasks(&project.gid, workspace_gid.as_deref(), include_completed)
            .await
        {
            Ok(tasks) => {
                info!(
                    "Received {} tasks for project '{}'.",
                    tasks.len(),
                    &project.name
                );
                let mut state = self.state.lock().await;
                state.set_tasks(tasks);
                // If in kanban mode, also load sections
                if view_mode == crate::state::ViewMode::Kanban {
                    drop(state);
                    self.get_project_sections(project.gid.clone()).await?;
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
    async fn my_tasks(&mut self) -> Result<()> {
        info!("Fetching incomplete tasks assigned to user...");
        let user_gid;
        let workspace_gid;
        {
            let state = self.state.lock().await;
            user_gid = state.get_user().unwrap().gid.to_owned();
            workspace_gid = state.get_active_workspace().unwrap().gid.to_owned();
        }
        let my_tasks = self.asana.my_tasks(&user_gid, &workspace_gid).await?;
        let mut state = self.state.lock().await;
        state.set_tasks(my_tasks);
        info!("Received incomplete tasks assigned to user.");
        Ok(())
    }

    /// Update a task (e.g., mark as complete/incomplete).
    ///
    async fn update_task(&mut self, task_gid: String, completed: Option<bool>) -> Result<()> {
        info!("Updating task {}...", task_gid);
        self.asana.update_task(&task_gid, completed).await?;
        info!("Task {} updated successfully.", task_gid);
        // Refresh the current task list
        let view;
        {
            let state = self.state.lock().await;
            view = state.current_view().clone();
        }
        match view {
            crate::state::View::MyTasks => {
                self.my_tasks().await?;
            }
            crate::state::View::ProjectTasks => {
                self.project_tasks().await?;
            }
            _ => {}
        }
        Ok(())
    }

    /// Delete a task.
    ///
    async fn delete_task(&mut self, task_gid: String) -> Result<()> {
        info!("Deleting task {}...", task_gid);
        self.asana.delete_task(&task_gid).await?;
        info!("Task {} deleted successfully.", task_gid);
        // Refresh the current task list
        let view;
        {
            let state = self.state.lock().await;
            view = state.current_view().clone();
        }
        match view {
            crate::state::View::MyTasks => {
                self.my_tasks().await?;
            }
            crate::state::View::ProjectTasks => {
                self.project_tasks().await?;
            }
            _ => {}
        }
        Ok(())
    }

    /// Refresh the current task list.
    ///
    async fn refresh_tasks(&mut self) -> Result<()> {
        let view;
        {
            let state = self.state.lock().await;
            view = state.current_view().clone();
        }
        match view {
            crate::state::View::MyTasks => {
                self.my_tasks().await?;
            }
            crate::state::View::ProjectTasks => {
                self.project_tasks().await?;
            }
            _ => {}
        }
        Ok(())
    }
    
    /// Get full task details.
    ///
    async fn get_task_detail(&mut self, task_gid: String) -> Result<()> {
        info!("Fetching full details for task {}...", task_gid);
        let task = self.asana.get_task(&task_gid).await?;
        let task_gid_clone = task_gid.clone();
        let mut state = self.state.lock().await;
        state.set_task_detail(task);
        drop(state);
        // Also load stories/comments
        self.get_task_stories(task_gid_clone).await?;
        info!("Task details loaded successfully.");
        Ok(())
    }
    
    /// Get project sections for kanban board.
    ///
    async fn get_project_sections(&mut self, project_gid: String) -> Result<()> {
        info!("Fetching sections for project {}...", project_gid);
        let sections = self.asana.get_project_sections(&project_gid).await?;
        let mut state = self.state.lock().await;
        state.set_sections(sections);
        info!("Sections loaded successfully.");
        Ok(())
    }
    
    /// Get task stories/comments.
    ///
    async fn get_task_stories(&mut self, task_gid: String) -> Result<()> {
        info!("Fetching stories/comments for task {}...", task_gid);
        let stories = self.asana.get_task_stories(&task_gid).await?;
        let mut state = self.state.lock().await;
        state.set_task_stories(stories);
        info!("Stories/comments loaded successfully.");
        Ok(())
    }
    
    /// Create a story/comment on a task.
    ///
    async fn create_story(&mut self, task_gid: String, text: String) -> Result<()> {
        info!("Creating comment on task {}...", task_gid);
        self.asana.create_story(&task_gid, &text).await?;
        // Refresh stories after creating
        self.get_task_stories(task_gid).await?;
        info!("Comment created successfully.");
        Ok(())
    }
    
    /// Get workspace users.
    ///
    async fn get_workspace_users(&mut self, workspace_gid: String) -> Result<()> {
        info!("Fetching users for workspace {}...", workspace_gid);
        let users = self.asana.get_workspace_users(&workspace_gid).await?;
        let mut state = self.state.lock().await;
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
    ) -> Result<()> {
        info!("Creating new task '{}' in project {}...", name, project_gid);
        let task = self.asana.create_task(
            &project_gid,
            &name,
            notes.as_deref(),
            assignee.as_deref(),
            due_on.as_deref(),
            section.as_deref(),
        ).await?;
        
        info!("Task '{}' created successfully with GID {}", task.name, task.gid);
        
        // Refresh tasks after creating - need to get project from state
        let project_gid_for_refresh;
        {
            let state = self.state.lock().await;
            if let Some(project) = state.get_project() {
                project_gid_for_refresh = project.gid.clone();
            } else {
                // If no project in state, use the one from the create request
                project_gid_for_refresh = project_gid.clone();
            }
        }
        
        // Refresh the project tasks
        self.project_tasks().await?;
        
        // If in kanban mode, also refresh sections
        {
            let state = self.state.lock().await;
            if state.get_view_mode() == crate::state::ViewMode::Kanban {
                drop(state);
                self.get_project_sections(project_gid_for_refresh).await?;
            }
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
    ) -> Result<()> {
        info!("Updating task fields for {}...", gid);
        self.asana.update_task_fields(
            &gid,
            name.as_deref(),
            notes.as_deref(),
            assignee.as_deref(),
            due_on.as_deref(),
            section.as_deref(),
            completed,
        ).await?;
        // Refresh task detail if we're viewing it
        let view;
        let task_gid_to_refresh;
        {
            let state = self.state.lock().await;
            view = state.current_view().clone();
            if matches!(view, crate::state::View::TaskDetail) {
                if let Some(task) = state.get_task_detail() {
                    task_gid_to_refresh = Some(task.gid.clone());
                } else {
                    task_gid_to_refresh = None;
                }
            } else {
                task_gid_to_refresh = None;
            }
        }
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
    async fn move_task_to_section(&mut self, task_gid: String, section_gid: String) -> Result<()> {
        info!("Moving task {} to section {}...", task_gid, section_gid);
        self.asana.add_task_to_section(&task_gid, &section_gid).await?;
        // Refresh tasks after moving
        self.project_tasks().await?;
        info!("Task moved successfully.");
        Ok(())
    }
}
