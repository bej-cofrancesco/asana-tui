use crate::app::NetworkEventSender;
use crate::asana::{Project, Section, Story, Task, User, Workspace};
use crate::events::network::Event as NetworkEvent;
use crate::ui::SPINNER_FRAME_COUNT;
use log::*;
use std::collections::{HashMap, HashSet};
use tui::layout::Rect;
use tui::widgets::ListState;

/// Specifying the different foci.
///
#[derive(Debug, PartialEq, Eq)]
pub enum Focus {
    Menu,
    View,
}

/// Specifying the different menus.
///
#[derive(Debug, PartialEq, Eq)]
pub enum Menu {
    Status,
    Shortcuts,
    TopList,
}

/// Specifying the different views.
///
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum View {
    Welcome,
    MyTasks,
    RecentlyModified,
    RecentlyCompleted,
    ProjectTasks,
    TaskDetail,
    KanbanBoard,
    CreateTask,
    EditTask,
}

/// Specifying view mode (list or kanban).
///
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ViewMode {
    List,
    Kanban,
}

/// Specifying edit form field state.
///
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum EditFormState {
    Name,
    Notes,
    Assignee,
    DueDate,
    Section,
}

/// Specifying task filter options.
///
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TaskFilter {
    All,
    Incomplete,
    Completed,
    Assignee(Option<String>), // Filter by assignee GID (None = unassigned)
}

/// Get the base shortcuts list.
///
pub fn base_shortcuts() -> Vec<String> {
    vec![
        "My Tasks".to_string(),
        "Recently Modified".to_string(),
        "Recently Completed".to_string(),
    ]
}

/// Houses data representative of application state.
///
pub struct State {
    net_sender: Option<NetworkEventSender>,
    config_save_sender: Option<crate::app::ConfigSaveSender>,
    user: Option<User>,
    workspaces: Vec<Workspace>,
    active_workspace_gid: Option<String>,
    terminal_size: Rect,
    spinner_index: usize,
    current_focus: Focus,
    current_menu: Menu,
    current_shortcut_index: usize, // Keep for backward compatibility, but use shortcuts_list_state
    shortcuts_list_state: ListState,
    current_top_list_index: usize,
    view_stack: Vec<View>,
    tasks: Vec<Task>,
    projects: Vec<Project>,
    project: Option<Project>,
    projects_list_state: ListState,
    tasks_list_state: ListState,
    starred_projects: HashSet<String>,              // GIDs
    starred_project_names: HashMap<String, String>, // GID -> Name
    search_query: String,
    search_mode: bool,
    search_target: Option<SearchTarget>,
    filtered_projects: Vec<Project>,
    filtered_tasks: Vec<Task>,
    debug_mode: bool,
    debug_index: usize,
    debug_entries: Vec<String>, // Store log entries for navigation and copying
    task_filter: TaskFilter,
    delete_confirmation: Option<String>, // GID of task pending deletion confirmation
    current_task_detail: Option<Task>,   // Currently viewed task with full details
    sections: Vec<Section>,              // Project sections for kanban
    workspace_users: Vec<User>,          // Users for assignment dropdowns
    task_stories: Vec<Story>,            // Comments for current task
    view_mode: ViewMode,                 // List or Kanban view
    edit_mode: bool,                     // Whether in edit mode
    edit_form_state: Option<EditFormState>, // Current form field being edited
    kanban_column_index: usize,          // Current column in kanban view
    kanban_task_index: usize,            // Current task index in selected column
    comment_input_mode: bool,            // Whether in comment input mode
    comment_input_text: String,          // Current comment text being typed
    comments_scroll_offset: usize,       // Scroll offset for comments list
    details_scroll_offset: usize,        // Scroll offset for details panel
    notes_scroll_offset: usize,          // Scroll offset for notes panel
    current_task_panel: TaskDetailPanel, // Current panel in task detail view
    // Form input fields
    form_name: String,
    form_notes: String,
    form_assignee: Option<String>, // GID of selected assignee
    form_assignee_search: String,  // Search text for filtering assignees
    form_due_on: String,           // Date string
    form_section: Option<String>,  // GID of selected section
    // Original form values (for tracking changes)
    original_form_name: String,
    original_form_notes: String,
    original_form_assignee: Option<String>,
    original_form_due_on: String,
    original_form_section: Option<String>,
    // Dropdown selection indices
    assignee_dropdown_index: usize,
    section_dropdown_index: usize,
    access_token_input: String, // Input field for welcome screen
    has_access_token: bool,     // Whether access token exists (user is logged in)
    auth_error: Option<String>, // Error message if authentication fails
}

/// Specifies which panel is being searched.
///
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SearchTarget {
    Projects,
    Tasks,
}

/// Defines different panels within task detail view.
///
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TaskDetailPanel {
    Details,  // Properties, assignee, dates, etc.
    Comments, // Comments/stories
    Notes,    // Task notes/description
}

/// Defines default application state.
///
impl Default for State {
    fn default() -> State {
        State {
            net_sender: None,
            config_save_sender: None,
            user: None,
            workspaces: vec![],
            active_workspace_gid: None,
            terminal_size: Rect::default(),
            spinner_index: 0,
            current_focus: Focus::Menu,
            current_menu: Menu::Shortcuts,
            current_shortcut_index: 0,
            shortcuts_list_state: ListState::default(),
            current_top_list_index: 0,
            view_stack: vec![View::Welcome],
            tasks: vec![],
            projects: vec![],
            project: None,
            projects_list_state: ListState::default(),
            tasks_list_state: ListState::default(),
            starred_projects: HashSet::new(),
            starred_project_names: HashMap::new(),
            search_query: String::new(),
            search_mode: false,
            search_target: None,
            filtered_projects: vec![],
            filtered_tasks: vec![],
            debug_mode: false,
            debug_index: 0,
            debug_entries: vec![],
            task_filter: TaskFilter::Incomplete,
            delete_confirmation: None,
            current_task_detail: None,
            sections: vec![],
            workspace_users: vec![],
            task_stories: vec![],
            view_mode: ViewMode::List,
            edit_mode: false,
            edit_form_state: None,
            kanban_column_index: 0,
            kanban_task_index: 0,
            comment_input_mode: false,
            comment_input_text: String::new(),
            comments_scroll_offset: 0,
            details_scroll_offset: 0,
            notes_scroll_offset: 0,
            current_task_panel: TaskDetailPanel::Details,
            form_name: String::new(),
            form_notes: String::new(),
            form_assignee: None,
            form_assignee_search: String::new(),
            form_due_on: String::new(),
            form_section: None,
            original_form_name: String::new(),
            original_form_notes: String::new(),
            original_form_assignee: None,
            original_form_due_on: String::new(),
            original_form_section: None,
            assignee_dropdown_index: 0,
            section_dropdown_index: 0,
            access_token_input: String::new(),
            has_access_token: false, // Default to false, will be set when token is loaded
            auth_error: None,        // No error initially
        }
    }
}

impl State {
    pub fn new(
        net_sender: NetworkEventSender,
        config_save_sender: crate::app::ConfigSaveSender,
        starred_projects: Vec<String>,
        starred_project_names: HashMap<String, String>,
        has_access_token: bool,
    ) -> Self {
        State {
            net_sender: Some(net_sender),
            config_save_sender: Some(config_save_sender),
            starred_projects: starred_projects.into_iter().collect(),
            starred_project_names,
            debug_entries: vec![], // Initialize empty, will be populated by logger
            has_access_token,
            ..State::default()
        }
    }

    /// Returns details for current user.
    ///
    pub fn get_user(&self) -> Option<&User> {
        self.user.as_ref()
    }

    /// Sets details for current user.
    ///
    pub fn set_user(&mut self, user: User) -> &mut Self {
        self.user = Some(user);
        self
    }

    /// Returns a reference to the active workspace or None.
    ///
    pub fn get_active_workspace(&self) -> Option<&Workspace> {
        match &self.active_workspace_gid {
            Some(active_workspace_gid) => self
                .workspaces
                .iter()
                .find(|workspace| active_workspace_gid == &workspace.gid),
            None => None,
        }
    }

    /// Sets the active workspace by the given workspace GID.
    ///
    pub fn set_active_workspace(&mut self, workspace_gid: String) -> &mut Self {
        self.active_workspace_gid = Some(workspace_gid);
        self
    }

    /// Sets workspaces available to current user, initializing the active
    /// workspace GID if unset and at least one workspace is available.
    ///
    pub fn set_workspaces(&mut self, workspaces: Vec<Workspace>) -> &mut Self {
        self.workspaces = workspaces;
        self
    }

    /// Sets the terminal size.
    ///
    pub fn set_terminal_size(&mut self, size: Rect) -> &mut Self {
        self.terminal_size = size;
        self
    }

    /// Advance the spinner index.
    ///
    pub fn advance_spinner_index(&mut self) -> &mut Self {
        self.spinner_index += 1;
        if self.spinner_index >= SPINNER_FRAME_COUNT {
            self.spinner_index = 0;
        }
        self
    }

    /// Return the current spinner index.
    ///
    pub fn get_spinner_index(&self) -> &usize {
        &self.spinner_index
    }

    /// Return the current focus.
    ///
    pub fn current_focus(&self) -> &Focus {
        &self.current_focus
    }

    /// Change focus to the current menu.
    ///
    pub fn focus_menu(&mut self) -> &mut Self {
        self.current_focus = Focus::Menu;
        self
    }

    /// Change focus to the current view.
    ///
    pub fn focus_view(&mut self) -> &mut Self {
        self.current_focus = Focus::View;
        self
    }

    /// Return the current menu.
    ///
    pub fn current_menu(&self) -> &Menu {
        &self.current_menu
    }

    /// Activate the next menu.
    ///
    pub fn next_menu(&mut self) -> &mut Self {
        match self.current_menu {
            Menu::Status => self.current_menu = Menu::Shortcuts,
            Menu::Shortcuts => self.current_menu = Menu::TopList,
            Menu::TopList => self.current_menu = Menu::Status,
        }
        self
    }

    /// Activate the previous menu.
    ///
    pub fn previous_menu(&mut self) -> &mut Self {
        match self.current_menu {
            Menu::Status => self.current_menu = Menu::TopList,
            Menu::Shortcuts => self.current_menu = Menu::Status,
            Menu::TopList => self.current_menu = Menu::Shortcuts,
        }
        self
    }

    /// Activate the status menu.
    ///
    pub fn select_status_menu(&mut self) -> &mut Self {
        self.view_stack.clear();
        self.view_stack.push(View::Welcome);
        self
    }

    /// Return the current shortcut index (for backward compatibility).
    /// Used in tests.
    ///
    #[allow(dead_code)]
    pub fn current_shortcut_index(&self) -> &usize {
        &self.current_shortcut_index
    }

    /// Initialize shortcuts list state when shortcuts change.
    ///
    fn update_shortcuts_list_state(&mut self) {
        let all_shortcuts = self.get_all_shortcuts();
        if all_shortcuts.is_empty() {
            self.shortcuts_list_state.select(None);
            self.current_shortcut_index = 0;
        } else {
            // Ensure selection is valid
            let current = self.shortcuts_list_state.selected().unwrap_or(0);
            if current >= all_shortcuts.len() {
                self.shortcuts_list_state.select(Some(0));
                self.current_shortcut_index = 0;
            } else {
                self.current_shortcut_index = current;
            }
        }
    }

    /// Activate the next shortcut.
    ///
    pub fn next_shortcut_index(&mut self) -> &mut Self {
        let all_shortcuts = self.get_all_shortcuts();
        if all_shortcuts.is_empty() {
            self.shortcuts_list_state.select(None);
            self.current_shortcut_index = 0;
            return self;
        }
        let current = self.shortcuts_list_state.selected().unwrap_or(0);
        let next = if current + 1 < all_shortcuts.len() {
            current + 1
        } else {
            0
        };
        self.shortcuts_list_state.select(Some(next));
        self.current_shortcut_index = next;
        self
    }

    /// Activate the previous shortcut.
    ///
    pub fn previous_shortcut_index(&mut self) -> &mut Self {
        let all_shortcuts = self.get_all_shortcuts();
        if all_shortcuts.is_empty() {
            self.shortcuts_list_state.select(None);
            self.current_shortcut_index = 0;
            return self;
        }
        let current = self.shortcuts_list_state.selected().unwrap_or(0);
        let prev = if current > 0 {
            current - 1
        } else {
            all_shortcuts.len() - 1
        };
        self.shortcuts_list_state.select(Some(prev));
        self.current_shortcut_index = prev;
        self
    }

    /// Select the current shortcut.
    ///
    pub fn select_current_shortcut_index(&mut self) -> &mut Self {
        self.view_stack.clear();
        let all_shortcuts = self.get_all_shortcuts();
        let selected_index = self
            .shortcuts_list_state
            .selected()
            .unwrap_or(self.current_shortcut_index);

        if all_shortcuts.is_empty() || selected_index >= all_shortcuts.len() {
            return self;
        }

        let shortcut = &all_shortcuts[selected_index];

        // Check if it's a static shortcut (these are now at the end after starred projects)
        match shortcut.as_str() {
            "My Tasks" => {
                self.tasks.clear();
                self.dispatch(NetworkEvent::MyTasks);
                self.view_stack.push(View::MyTasks);
            }
            "Recently Modified" => {
                self.tasks.clear();
                self.view_stack.push(View::RecentlyModified);
            }
            "Recently Completed" => {
                self.tasks.clear();
                self.view_stack.push(View::RecentlyCompleted);
            }
            _ => {
                // It's a starred project
                if let Some(project) = self.projects.iter().find(|p| p.name == shortcut.as_str()) {
                    self.project = Some(project.to_owned());
                    self.tasks.clear();
                    self.dispatch(NetworkEvent::ProjectTasks);
                    self.view_stack.push(View::ProjectTasks);
                }
            }
        }
        self.focus_view();
        self
    }

    /// Activate the next top list item.
    ///
    pub fn next_top_list_index(&mut self) -> &mut Self {
        let filtered = self.get_filtered_projects();
        if filtered.is_empty() {
            return self;
        }
        let current = self.projects_list_state.selected().unwrap_or(0);
        let next = if current + 1 < filtered.len() {
            current + 1
        } else {
            0
        };
        self.projects_list_state.select(Some(next));
        self
    }

    /// Activate the previous top list item.
    ///
    pub fn previous_top_list_index(&mut self) -> &mut Self {
        let filtered = self.get_filtered_projects();
        if filtered.is_empty() {
            return self;
        }
        let current = self.projects_list_state.selected().unwrap_or(0);
        let prev = if current > 0 {
            current - 1
        } else {
            filtered.len() - 1
        };
        self.projects_list_state.select(Some(prev));
        self
    }

    /// Return the current top list item.
    ///
    /// Return the current top list index.
    /// Used in tests.
    ///
    #[allow(dead_code)]
    pub fn current_top_list_index(&self) -> &usize {
        &self.current_top_list_index
    }

    /// Select the current top list item.
    ///
    pub fn select_current_top_list_index(&mut self) -> &mut Self {
        let filtered = self.get_filtered_projects();
        if filtered.is_empty() {
            return self;
        }
        if let Some(selected_index) = self.projects_list_state.selected() {
            if selected_index < filtered.len() {
                self.project = Some(filtered[selected_index].to_owned());
                self.view_stack.clear();
                self.tasks.clear();
                self.dispatch(NetworkEvent::ProjectTasks);
                self.view_stack.push(View::ProjectTasks);
                self.focus_view();
                self.exit_search_mode();
            }
        }
        self
    }

    /// Return the current view.
    ///
    pub fn current_view(&self) -> &View {
        self.view_stack.last().unwrap()
    }

    /// Push a view onto the view stack.
    ///
    pub fn push_view(&mut self, view: View) -> &mut Self {
        self.view_stack.push(view);
        self
    }

    /// Pop a view from the view stack.
    ///
    pub fn pop_view(&mut self) -> Option<View> {
        // Don't pop if we're at the base view (Welcome)
        if self.view_stack.len() > 1 {
            self.view_stack.pop()
        } else {
            None
        }
    }

    /// Get the length of the view stack.
    ///
    pub fn view_stack_len(&self) -> usize {
        self.view_stack.len()
    }

    /// Return the list of tasks.
    ///
    pub fn get_tasks(&self) -> &Vec<Task> {
        &self.tasks
    }

    /// Set the list of tasks.
    ///
    pub fn set_tasks(&mut self, tasks: Vec<Task>) -> &mut Self {
        self.tasks = tasks;
        self.update_search_filters();
        if !self.tasks.is_empty() {
            self.tasks_list_state.select(Some(0));
        } else {
            self.tasks_list_state.select(None);
        }
        self
    }

    /// Return the list of projects.
    ///
    pub fn get_projects(&self) -> &Vec<Project> {
        &self.projects
    }

    /// Set the list of projects.
    ///
    pub fn set_projects(&mut self, projects: Vec<Project>) -> &mut Self {
        self.projects = projects;
        // Update starred project names when projects load
        for project in &self.projects {
            if self.starred_projects.contains(&project.gid) {
                self.starred_project_names
                    .insert(project.gid.to_owned(), project.name.to_owned());
            }
        }
        // Update shortcuts list state when projects load (so starred projects appear)
        self.update_shortcuts_list_state();
        self.update_search_filters();
        if !self.projects.is_empty() && self.current_top_list_index < self.projects.len() {
            self.projects_list_state
                .select(Some(self.current_top_list_index));
        } else if !self.projects.is_empty() {
            self.current_top_list_index = 0;
            self.projects_list_state.select(Some(0));
        } else {
            self.projects_list_state.select(None);
        }
        self
    }

    /// Return the current project.
    ///
    pub fn get_project(&self) -> Option<&Project> {
        self.project.as_ref()
    }

    /// Return the projects list state.
    ///
    pub fn get_projects_list_state(&mut self) -> &mut ListState {
        &mut self.projects_list_state
    }

    /// Return the tasks list state.
    ///
    pub fn get_tasks_list_state(&mut self) -> &mut ListState {
        &mut self.tasks_list_state
    }

    /// Get shortcuts list state.
    ///
    pub fn get_shortcuts_list_state(&mut self) -> &mut ListState {
        &mut self.shortcuts_list_state
    }

    /// Return the current task index.
    ///
    #[allow(dead_code)]
    pub fn current_task_index(&self) -> Option<usize> {
        self.tasks_list_state.selected()
    }

    /// Activate the next task.
    ///
    pub fn next_task_index(&mut self) -> &mut Self {
        let filtered = self.get_filtered_tasks();
        if filtered.is_empty() {
            self.tasks_list_state.select(None);
            return self;
        }
        let selected = self.tasks_list_state.selected();
        let next = match selected {
            Some(i) => {
                if i + 1 < filtered.len() {
                    Some(i + 1)
                } else {
                    Some(0)
                }
            }
            None => Some(0),
        };
        self.tasks_list_state.select(next);
        self
    }

    /// Activate the previous task.
    ///
    pub fn previous_task_index(&mut self) -> &mut Self {
        let filtered = self.get_filtered_tasks();
        if filtered.is_empty() {
            self.tasks_list_state.select(None);
            return self;
        }
        let selected = self.tasks_list_state.selected();
        let prev = match selected {
            Some(i) => {
                if i > 0 {
                    Some(i - 1)
                } else {
                    Some(filtered.len() - 1)
                }
            }
            None => Some(filtered.len() - 1),
        };
        self.tasks_list_state.select(prev);
        self
    }

    /// Toggle completion status of the selected task.
    ///
    pub fn toggle_task_completion(&mut self) -> &mut Self {
        let filtered = self.get_filtered_tasks();
        if let Some(selected_index) = self.tasks_list_state.selected() {
            if selected_index < filtered.len() {
                let task = &filtered[selected_index];
                // Toggle the completion status
                self.dispatch(NetworkEvent::UpdateTask {
                    gid: task.gid.to_owned(),
                    completed: Some(!task.completed),
                });
            }
        }
        self
    }

    /// Delete the selected task (shows confirmation if not already confirmed).
    ///
    pub fn delete_selected_task(&mut self) -> &mut Self {
        let filtered = self.get_filtered_tasks();
        if let Some(selected_index) = self.tasks_list_state.selected() {
            if selected_index < filtered.len() {
                let task = &filtered[selected_index];

                // If we have a pending confirmation for this task, actually delete it
                if let Some(pending_gid) = &self.delete_confirmation {
                    if pending_gid == &task.gid {
                        self.delete_confirmation = None;
                        self.dispatch(NetworkEvent::DeleteTask {
                            gid: task.gid.to_owned(),
                        });
                        return self;
                    }
                }

                // Otherwise, show confirmation
                self.delete_confirmation = Some(task.gid.to_owned());
            }
        }
        self
    }

    /// Cancel delete confirmation.
    ///
    pub fn cancel_delete_confirmation(&mut self) -> &mut Self {
        self.delete_confirmation = None;
        self
    }

    /// Check if there's a pending delete confirmation.
    ///
    pub fn has_delete_confirmation(&self) -> bool {
        self.delete_confirmation.is_some()
    }

    /// Set delete confirmation for a task GID (used from detail view).
    ///
    pub fn set_delete_confirmation(&mut self, task_gid: String) -> &mut Self {
        self.delete_confirmation = Some(task_gid);
        self
    }

    /// Get the current task filter.
    ///
    pub fn get_task_filter(&self) -> TaskFilter {
        self.task_filter.clone()
    }

    /// Set the task filter.
    ///
    pub fn set_task_filter(&mut self, filter: TaskFilter) -> &mut Self {
        self.task_filter = filter;
        // Update filtered tasks when filter changes
        self.update_search_filters();
        self
    }

    /// Cycle to the next task filter.
    ///
    pub fn next_task_filter(&mut self) -> &mut Self {
        let old_filter = self.task_filter.clone();
        self.task_filter = match &self.task_filter {
            TaskFilter::All => TaskFilter::Incomplete,
            TaskFilter::Incomplete => TaskFilter::Completed,
            TaskFilter::Completed => TaskFilter::All,
            TaskFilter::Assignee(_) => TaskFilter::All, // Reset assignee filter to All
        };

        // If switching to All or Completed, we need to refetch tasks to get completed ones
        // Only Incomplete filter can work with the current cached tasks
        if matches!(self.task_filter, TaskFilter::All | TaskFilter::Completed)
            && matches!(old_filter, TaskFilter::Incomplete)
        {
            // Refetch tasks when switching to a filter that needs completed tasks
            if matches!(self.current_view(), View::ProjectTasks) {
                self.dispatch(NetworkEvent::ProjectTasks);
            }
        }

        self.update_search_filters();

        self.update_search_filters();
        self
    }

    /// Set the current task detail.
    ///
    pub fn set_task_detail(&mut self, task: Task) -> &mut Self {
        self.current_task_detail = Some(task);
        self
    }

    /// Get the current task detail.
    ///
    pub fn get_task_detail(&self) -> Option<&Task> {
        self.current_task_detail.as_ref()
    }

    /// Clear the current task detail.
    ///
    #[allow(dead_code)]
    pub fn clear_task_detail(&mut self) -> &mut Self {
        self.current_task_detail = None;
        self.task_stories = vec![];
        self
    }

    /// Set sections for kanban board.
    ///
    pub fn set_sections(&mut self, sections: Vec<Section>) -> &mut Self {
        self.sections = sections;
        self
    }

    /// Get sections.
    ///
    pub fn get_sections(&self) -> &[Section] {
        &self.sections
    }

    /// Set workspace users.
    ///
    pub fn set_workspace_users(&mut self, users: Vec<User>) -> &mut Self {
        self.workspace_users = users;
        self
    }

    /// Get workspace users.
    ///
    pub fn get_workspace_users(&self) -> &[User] {
        &self.workspace_users
    }

    /// Set task stories/comments.
    ///
    pub fn set_task_stories(&mut self, stories: Vec<Story>) -> &mut Self {
        self.task_stories = stories;
        // Auto-scroll to bottom to show newest comments
        self.scroll_comments_to_bottom();
        self
    }

    /// Get task stories/comments.
    ///
    pub fn get_task_stories(&self) -> &[Story] {
        &self.task_stories
    }

    pub fn get_comments_scroll_offset(&self) -> usize {
        self.comments_scroll_offset
    }

    pub fn scroll_comments_down(&mut self) -> &mut Self {
        // For bottom-aligned scrolling, "down" (j key) = see newer comments = decrease index
        // Index 0 = newest (last item), higher = older
        // Filter for actual comments to get the correct count
        let comments: Vec<&Story> = self
            .task_stories
            .iter()
            .filter(|s| match &s.resource_subtype {
                Some(subtype) => subtype == "comment_added",
                None => s.created_by.is_some(),
            })
            .collect();

        let total_comments = comments.len();
        if total_comments > 0 {
            // Clamp offset to valid range first (in case it got out of bounds)
            self.comments_scroll_offset = self.comments_scroll_offset % total_comments;

            // Wrap around using modulo arithmetic (like other lists)
            // If at 0, wrap to last item; otherwise decrease
            if self.comments_scroll_offset == 0 {
                self.comments_scroll_offset = total_comments.saturating_sub(1);
            } else {
                self.comments_scroll_offset -= 1;
            }
        } else {
            // No comments, reset to 0
            self.comments_scroll_offset = 0;
        }
        self
    }

    pub fn scroll_comments_up(&mut self) -> &mut Self {
        // For bottom-aligned scrolling, "up" (k key) = see older comments = increase index
        // Filter for actual comments to get the correct count
        let comments: Vec<&Story> = self
            .task_stories
            .iter()
            .filter(|s| match &s.resource_subtype {
                Some(subtype) => subtype == "comment_added",
                None => s.created_by.is_some(),
            })
            .collect();

        let total_comments = comments.len();
        if total_comments > 0 {
            // Clamp offset to valid range first (in case it got out of bounds)
            self.comments_scroll_offset = self.comments_scroll_offset % total_comments;

            // Wrap around using modulo arithmetic (like other lists)
            // If at max, wrap to 0; otherwise increase
            let max_index = total_comments.saturating_sub(1);
            if self.comments_scroll_offset >= max_index {
                self.comments_scroll_offset = 0;
            } else {
                self.comments_scroll_offset += 1;
            }
        } else {
            // No comments, reset to 0
            self.comments_scroll_offset = 0;
        }
        self
    }

    pub fn scroll_comments_to_bottom(&mut self) -> &mut Self {
        // For bottom alignment, index 0 means newest (last item)
        self.comments_scroll_offset = 0;
        self
    }

    // Details panel scrolling

    pub fn get_details_scroll_offset(&self) -> usize {
        self.details_scroll_offset
    }

    pub fn scroll_details_up(&mut self) -> &mut Self {
        if self.details_scroll_offset > 0 {
            self.details_scroll_offset -= 1;
        }
        self
    }

    pub fn scroll_details_down(&mut self) -> &mut Self {
        // Details panel has a fixed number of properties, so we can scroll through them
        // For now, just allow scrolling up to a reasonable limit
        if self.details_scroll_offset < 20 {
            self.details_scroll_offset += 1;
        }
        self
    }

    pub fn reset_details_scroll(&mut self) -> &mut Self {
        self.details_scroll_offset = 0;
        self
    }

    // Notes panel scrolling

    pub fn get_notes_scroll_offset(&self) -> usize {
        self.notes_scroll_offset
    }

    pub fn scroll_notes_up(&mut self) -> &mut Self {
        if self.notes_scroll_offset > 0 {
            self.notes_scroll_offset -= 1;
        }
        self
    }

    pub fn scroll_notes_down(&mut self) -> &mut Self {
        // Notes can be long, allow scrolling
        if self.notes_scroll_offset < 100 {
            self.notes_scroll_offset += 1;
        }
        self
    }

    pub fn reset_notes_scroll(&mut self) -> &mut Self {
        self.notes_scroll_offset = 0;
        self
    }

    // Task detail panel navigation

    pub fn get_current_task_panel(&self) -> TaskDetailPanel {
        self.current_task_panel
    }

    pub fn next_task_panel(&mut self) -> &mut Self {
        self.current_task_panel = match self.current_task_panel {
            TaskDetailPanel::Details => TaskDetailPanel::Comments,
            TaskDetailPanel::Comments => TaskDetailPanel::Notes,
            TaskDetailPanel::Notes => TaskDetailPanel::Details, // Wrap around
        };
        self
    }

    pub fn previous_task_panel(&mut self) -> &mut Self {
        self.current_task_panel = match self.current_task_panel {
            TaskDetailPanel::Details => TaskDetailPanel::Notes, // Wrap around
            TaskDetailPanel::Comments => TaskDetailPanel::Details,
            TaskDetailPanel::Notes => TaskDetailPanel::Comments,
        };
        self
    }

    pub fn reset_task_panel(&mut self) -> &mut Self {
        self.current_task_panel = TaskDetailPanel::Details;
        self
    }

    pub fn set_current_task_panel(&mut self, panel: TaskDetailPanel) -> &mut Self {
        self.current_task_panel = panel;
        self
    }

    /// Toggle view mode (List/Kanban).
    ///
    #[allow(dead_code)]
    pub fn toggle_view_mode(&mut self) -> &mut Self {
        self.view_mode = match self.view_mode {
            ViewMode::List => ViewMode::Kanban,
            ViewMode::Kanban => ViewMode::List,
        };
        self
    }

    /// Get current view mode.
    ///
    pub fn get_view_mode(&self) -> ViewMode {
        self.view_mode
    }

    /// Set kanban column index.
    ///
    #[allow(dead_code)]
    pub fn set_kanban_column_index(&mut self, index: usize) -> &mut Self {
        self.kanban_column_index = index.min(self.sections.len().saturating_sub(1));
        self
    }

    /// Get kanban column index.
    ///
    pub fn get_kanban_column_index(&self) -> usize {
        self.kanban_column_index
    }

    /// Navigate to next kanban column.
    ///
    pub fn next_kanban_column(&mut self) -> &mut Self {
        if !self.sections.is_empty() {
            self.kanban_column_index = (self.kanban_column_index + 1) % self.sections.len();
        }
        self
    }

    /// Navigate to previous kanban column.
    ///
    pub fn previous_kanban_column(&mut self) -> &mut Self {
        if !self.sections.is_empty() {
            if self.kanban_column_index > 0 {
                self.kanban_column_index -= 1;
            } else {
                self.kanban_column_index = self.sections.len() - 1;
            }
            // Reset task index when changing columns
            self.kanban_task_index = 0;
        }
        self
    }

    /// Set kanban task index.
    ///
    #[allow(dead_code)]
    pub fn set_kanban_task_index(&mut self, index: usize) -> &mut Self {
        self.kanban_task_index = index;
        self
    }

    /// Get kanban task index.
    ///
    pub fn get_kanban_task_index(&self) -> usize {
        self.kanban_task_index
    }

    /// Navigate to next task in current kanban column.
    ///
    #[allow(dead_code)]
    pub fn next_kanban_task(&mut self) -> &mut Self {
        if !self.sections.is_empty() && self.kanban_column_index < self.sections.len() {
            let section = &self.sections[self.kanban_column_index];
            let section_tasks: Vec<&Task> = self
                .tasks
                .iter()
                .filter(|t| {
                    t.section
                        .as_ref()
                        .map(|s| s.gid == section.gid)
                        .unwrap_or(false)
                })
                .collect();

            if !section_tasks.is_empty() {
                self.kanban_task_index = (self.kanban_task_index + 1) % section_tasks.len();
            }
        }
        self
    }

    /// Navigate to previous task in current kanban column.
    ///
    #[allow(dead_code)]
    pub fn previous_kanban_task(&mut self) -> &mut Self {
        if !self.sections.is_empty() && self.kanban_column_index < self.sections.len() {
            let section = &self.sections[self.kanban_column_index];
            let section_tasks: Vec<&Task> = self
                .tasks
                .iter()
                .filter(|t| {
                    t.section
                        .as_ref()
                        .map(|s| s.gid == section.gid)
                        .unwrap_or(false)
                })
                .collect();

            if !section_tasks.is_empty() {
                if self.kanban_task_index > 0 {
                    self.kanban_task_index -= 1;
                } else {
                    self.kanban_task_index = section_tasks.len() - 1;
                }
            }
        }
        self
    }

    /// Get current selected task in kanban view.
    ///
    pub fn get_kanban_selected_task(&self) -> Option<&Task> {
        if self.sections.is_empty() || self.kanban_column_index >= self.sections.len() {
            return None;
        }

        let section = &self.sections[self.kanban_column_index];
        let section_tasks: Vec<&Task> = self
            .tasks
            .iter()
            .filter(|t| {
                t.section
                    .as_ref()
                    .map(|s| s.gid == section.gid)
                    .unwrap_or(false)
            })
            .collect();

        if self.kanban_task_index < section_tasks.len() {
            Some(section_tasks[self.kanban_task_index])
        } else {
            None
        }
    }

    /// Get edit form state.
    ///
    pub fn get_edit_form_state(&self) -> Option<EditFormState> {
        self.edit_form_state
    }

    /// Set edit form state.
    ///
    pub fn set_edit_form_state(&mut self, state: Option<EditFormState>) -> &mut Self {
        self.edit_form_state = state;
        self
    }

    /// Enter comment input mode.
    ///
    pub fn enter_comment_input_mode(&mut self) -> &mut Self {
        self.comment_input_mode = true;
        self.comment_input_text.clear();
        self
    }

    /// Exit comment input mode.
    ///
    pub fn exit_comment_input_mode(&mut self) -> &mut Self {
        self.comment_input_mode = false;
        self.comment_input_text.clear();
        self
    }

    /// Check if in comment input mode.
    ///
    pub fn is_comment_input_mode(&self) -> bool {
        self.comment_input_mode
    }

    /// Get comment input text.
    ///
    pub fn has_access_token(&self) -> bool {
        self.has_access_token
    }

    pub fn get_access_token_input(&self) -> &str {
        &self.access_token_input
    }

    pub fn set_access_token(&mut self, _token: String) -> &mut Self {
        self.has_access_token = true;
        // Token is stored in config file by network handler
        self
    }

    pub fn add_access_token_char(&mut self, c: char) -> &mut Self {
        self.access_token_input.push(c);
        self
    }

    pub fn backspace_access_token(&mut self) -> &mut Self {
        self.access_token_input.pop();
        self
    }

    pub fn clear_access_token_input(&mut self) -> &mut Self {
        self.access_token_input.clear();
        self
    }

    pub fn set_auth_error(&mut self, error: Option<String>) -> &mut Self {
        self.auth_error = error;
        self
    }

    pub fn get_auth_error(&self) -> Option<&String> {
        self.auth_error.as_ref()
    }

    pub fn clear_auth_error(&mut self) -> &mut Self {
        self.auth_error = None;
        self
    }

    /// Get comment input text.
    ///
    pub fn get_comment_input_text(&self) -> &str {
        &self.comment_input_text
    }

    /// Add character to comment input.
    ///
    pub fn add_comment_char(&mut self, c: char) -> &mut Self {
        self.comment_input_text.push(c);
        self
    }

    /// Remove last character from comment input.
    ///
    pub fn remove_comment_char(&mut self) -> &mut Self {
        self.comment_input_text.pop();
        self
    }

    /// Submit comment (returns the text and clears input).
    ///
    pub fn submit_comment(&mut self) -> String {
        let text = self.comment_input_text.clone();
        self.comment_input_text.clear();
        self.comment_input_mode = false;
        text
    }

    /// Get form name.
    ///
    pub fn get_form_name(&self) -> &str {
        &self.form_name
    }

    /// Set form name.
    ///
    pub fn set_form_name(&mut self, name: String) -> &mut Self {
        self.form_name = name;
        self
    }

    /// Add character to form name.
    ///
    pub fn add_form_name_char(&mut self, c: char) -> &mut Self {
        self.form_name.push(c);
        self
    }

    /// Remove last character from form name.
    ///
    pub fn remove_form_name_char(&mut self) -> &mut Self {
        self.form_name.pop();
        self
    }

    /// Get form notes.
    ///
    pub fn get_form_notes(&self) -> &str {
        &self.form_notes
    }

    /// Set form notes.
    ///
    pub fn set_form_notes(&mut self, notes: String) -> &mut Self {
        self.form_notes = notes;
        self
    }

    /// Add character to form notes.
    ///
    pub fn add_form_notes_char(&mut self, c: char) -> &mut Self {
        self.form_notes.push(c);
        self
    }

    /// Remove last character from form notes.
    ///
    pub fn remove_form_notes_char(&mut self) -> &mut Self {
        self.form_notes.pop();
        self
    }

    /// Get form assignee.
    ///
    pub fn get_form_assignee(&self) -> Option<&String> {
        self.form_assignee.as_ref()
    }

    /// Set form assignee.
    ///
    pub fn set_form_assignee(&mut self, assignee: Option<String>) -> &mut Self {
        self.form_assignee = assignee;
        self
    }

    /// Get form due date.
    ///
    pub fn get_form_due_on(&self) -> &str {
        &self.form_due_on
    }

    /// Set form due date.
    ///
    pub fn set_form_due_on(&mut self, due_on: String) -> &mut Self {
        self.form_due_on = due_on;
        self
    }

    /// Add character to form due date.
    ///
    pub fn add_form_due_on_char(&mut self, c: char) -> &mut Self {
        self.form_due_on.push(c);
        self
    }

    /// Remove last character from form due date.
    ///
    pub fn remove_form_due_on_char(&mut self) -> &mut Self {
        self.form_due_on.pop();
        self
    }

    /// Get form section.
    ///
    pub fn get_form_section(&self) -> Option<&String> {
        self.form_section.as_ref()
    }

    /// Get original form name (for change detection).
    ///
    pub fn get_original_form_name(&self) -> &str {
        &self.original_form_name
    }

    /// Get original form notes (for change detection).
    ///
    pub fn get_original_form_notes(&self) -> &str {
        &self.original_form_notes
    }

    /// Get original form assignee (for change detection).
    ///
    pub fn get_original_form_assignee(&self) -> &Option<String> {
        &self.original_form_assignee
    }

    /// Get original form due date (for change detection).
    ///
    pub fn get_original_form_due_on(&self) -> &str {
        &self.original_form_due_on
    }

    /// Get original form section (for change detection).
    ///
    pub fn get_original_form_section(&self) -> &Option<String> {
        &self.original_form_section
    }

    /// Set form section.
    ///
    pub fn set_form_section(&mut self, section: Option<String>) -> &mut Self {
        self.form_section = section;
        self
    }

    pub fn get_assignee_dropdown_index(&self) -> usize {
        self.assignee_dropdown_index
    }

    pub fn set_assignee_dropdown_index(&mut self, index: usize) -> &mut Self {
        self.assignee_dropdown_index = index;
        self
    }

    /// Initialize assignee dropdown index to match currently selected assignee (if any).
    /// This ensures the selected assignee is shown when entering the dropdown.
    pub fn init_assignee_dropdown_index(&mut self) -> &mut Self {
        if let Some(selected_gid) = &self.form_assignee {
            let filtered_users = self.get_filtered_assignees();
            if let Some(index) = filtered_users.iter().position(|u| &u.gid == selected_gid) {
                self.assignee_dropdown_index = index;
            } else {
                // Selected assignee not in filtered list, reset to 0
                self.assignee_dropdown_index = 0;
            }
        } else {
            // No assignee selected, start at 0
            self.assignee_dropdown_index = 0;
        }
        self
    }

    pub fn next_assignee(&mut self) -> &mut Self {
        let users = self.get_filtered_assignees();
        if !users.is_empty() {
            self.assignee_dropdown_index = (self.assignee_dropdown_index + 1) % users.len();
        }
        self
    }

    pub fn previous_assignee(&mut self) -> &mut Self {
        let users = self.get_filtered_assignees();
        if !users.is_empty() {
            if self.assignee_dropdown_index == 0 {
                self.assignee_dropdown_index = users.len() - 1;
            } else {
                self.assignee_dropdown_index -= 1;
            }
        }
        self
    }

    pub fn select_current_assignee(&mut self) -> &mut Self {
        let users = self.get_filtered_assignees();
        if !users.is_empty() && self.assignee_dropdown_index < users.len() {
            self.form_assignee = Some(users[self.assignee_dropdown_index].gid.clone());
        }
        self
    }

    /// Get filtered assignees based on search text
    pub fn get_filtered_assignees(&self) -> Vec<User> {
        let search = self.form_assignee_search.to_lowercase();
        if search.is_empty() {
            self.workspace_users.clone()
        } else {
            self.workspace_users
                .iter()
                .filter(|u| {
                    u.name.to_lowercase().contains(&search)
                        || u.email.to_lowercase().contains(&search)
                })
                .cloned()
                .collect()
        }
    }

    pub fn add_assignee_search_char(&mut self, c: char) -> &mut Self {
        self.form_assignee_search.push(c);
        // Reset dropdown index when search changes
        self.assignee_dropdown_index = 0;
        self
    }

    pub fn backspace_assignee_search(&mut self) -> &mut Self {
        self.form_assignee_search.pop();
        // Reset dropdown index when search changes
        self.assignee_dropdown_index = 0;
        self
    }

    pub fn get_assignee_search(&self) -> &str {
        &self.form_assignee_search
    }

    pub fn clear_assignee_search(&mut self) -> &mut Self {
        self.form_assignee_search.clear();
        self.assignee_dropdown_index = 0;
        self
    }

    pub fn get_section_dropdown_index(&self) -> usize {
        self.section_dropdown_index
    }

    pub fn set_section_dropdown_index(&mut self, index: usize) -> &mut Self {
        self.section_dropdown_index = index;
        self
    }

    /// Initialize section dropdown index to match currently selected section (if any).
    /// This ensures the selected section is shown when entering the dropdown.
    pub fn init_section_dropdown_index(&mut self) -> &mut Self {
        if let Some(selected_gid) = &self.form_section {
            if let Some(index) = self.sections.iter().position(|s| &s.gid == selected_gid) {
                self.section_dropdown_index = index;
            } else {
                // Selected section not in list, reset to 0
                self.section_dropdown_index = 0;
            }
        } else {
            // No section selected, start at 0
            self.section_dropdown_index = 0;
        }
        self
    }

    pub fn next_section(&mut self) -> &mut Self {
        let max = self.sections.len();
        if max > 0 {
            self.section_dropdown_index = (self.section_dropdown_index + 1) % max;
        }
        self
    }

    pub fn previous_section(&mut self) -> &mut Self {
        let max = self.sections.len();
        if max > 0 {
            self.section_dropdown_index = if self.section_dropdown_index == 0 {
                max - 1
            } else {
                self.section_dropdown_index - 1
            };
        }
        self
    }

    pub fn select_current_section(&mut self) -> &mut Self {
        if let Some(section) = self.sections.get(self.section_dropdown_index) {
            self.form_section = Some(section.gid.clone());
        } else {
            warn!("No section found at index {}", self.section_dropdown_index);
        }
        self
    }

    /// Clear form fields.
    ///
    pub fn clear_form(&mut self) -> &mut Self {
        self.form_name.clear();
        self.form_notes.clear();
        self.form_assignee = None;
        self.form_assignee_search.clear();
        self.form_due_on.clear();
        self.form_section = None;
        self.edit_form_state = None;
        self.assignee_dropdown_index = 0;
        self.section_dropdown_index = 0;
        self
    }

    /// Initialize edit form with task data.
    ///
    #[allow(dead_code)]
    pub fn init_edit_form(&mut self, task: &Task) -> &mut Self {
        // Set current form values
        self.form_name = task.name.clone();
        self.form_notes = task.notes.clone().unwrap_or_default();
        self.form_assignee = task.assignee.as_ref().map(|u| u.gid.clone());
        self.form_due_on = task.due_on.clone().unwrap_or_default();
        self.form_section = task.section.as_ref().map(|s| s.gid.clone());
        // Store original values for change detection
        self.original_form_name = task.name.clone();
        self.original_form_notes = task.notes.clone().unwrap_or_default();
        self.original_form_assignee = task.assignee.as_ref().map(|u| u.gid.clone());
        self.original_form_due_on = task.due_on.clone().unwrap_or_default();
        self.original_form_section = task.section.as_ref().map(|s| s.gid.clone());
        self.edit_form_state = Some(EditFormState::Name);
        self
    }

    /// Toggle star status of the currently selected project (from Projects list).
    ///
    pub fn toggle_star_current_project(&mut self) -> &mut Self {
        let project_info = {
            let filtered = self.get_filtered_projects();
            if let Some(selected_index) = self.projects_list_state.selected() {
                if selected_index < filtered.len() {
                    let project = &filtered[selected_index];
                    Some((project.gid.to_owned(), project.name.to_owned()))
                } else {
                    None
                }
            } else {
                None
            }
        };

        if let Some((gid, name)) = project_info {
            if self.starred_projects.contains(&gid) {
                self.starred_projects.remove(&gid);
                self.starred_project_names.remove(&gid);
            } else {
                self.starred_projects.insert(gid.clone());
                self.starred_project_names.insert(gid, name);
            }
            // Update shortcuts list state when starring/unstarring
            self.update_shortcuts_list_state();
            // Trigger config save
            if let Some(sender) = &self.config_save_sender {
                let _ = sender.send(());
            }
        }
        self
    }

    /// Unstar the currently selected shortcut (only works for starred projects, not base shortcuts).
    ///
    pub fn unstar_current_shortcut(&mut self) -> &mut Self {
        let all_shortcuts = self.get_all_shortcuts();
        let selected_index = self.shortcuts_list_state.selected();

        if let Some(index) = selected_index {
            if index < all_shortcuts.len() {
                let shortcut_name = &all_shortcuts[index];

                // Check if it's a base shortcut - these cannot be removed
                match shortcut_name.as_str() {
                    "My Tasks" | "Recently Modified" | "Recently Completed" => {
                        // Base shortcuts cannot be removed
                        return self;
                    }
                    _ => {
                        // It's a starred project - find it and unstar it
                        // First find the GID, then remove it (to avoid borrowing conflicts)
                        let gid_to_remove = self
                            .starred_project_names
                            .iter()
                            .find(|(_, name)| name.as_str() == shortcut_name.as_str())
                            .map(|(gid, _)| gid.clone());

                        if let Some(gid) = gid_to_remove {
                            self.starred_projects.remove(&gid);
                            self.starred_project_names.remove(&gid);
                            // Update shortcuts list state
                            self.update_shortcuts_list_state();
                            // Trigger config save
                            if let Some(sender) = &self.config_save_sender {
                                let _ = sender.send(());
                            }
                        }
                    }
                }
            }
        }
        self
    }

    /// Check if a project is starred.
    ///
    pub fn is_project_starred(&self, project_gid: &str) -> bool {
        self.starred_projects.contains(project_gid)
    }

    /// Get all starred projects.
    ///
    #[allow(dead_code)]
    pub fn get_starred_projects(&self) -> Vec<&Project> {
        self.projects
            .iter()
            .filter(|p| self.starred_projects.contains(&p.gid))
            .collect()
    }

    /// Get all shortcuts (starred projects first, then static shortcuts).
    ///
    pub fn get_all_shortcuts(&self) -> Vec<String> {
        // Get starred project names - prioritize stored names, then loaded projects
        let mut starred: Vec<String> = Vec::new();

        for gid in &self.starred_projects {
            // Try to get name from stored names first (most reliable)
            if let Some(name) = self.starred_project_names.get(gid) {
                starred.push(name.clone());
            } else {
                // Fallback: try to get from loaded projects
                if let Some(project) = self.projects.iter().find(|p| &p.gid == gid) {
                    starred.push(project.name.to_owned());
                }
            }
        }

        starred.sort(); // Sort for consistent ordering

        // Put starred projects first, then base shortcuts
        let mut shortcuts = starred;
        shortcuts.extend(base_shortcuts());
        shortcuts
    }

    /// Get all shortcuts and update list state.
    ///
    pub fn get_all_shortcuts_with_update(&mut self) -> Vec<String> {
        let shortcuts = self.get_all_shortcuts();
        self.update_shortcuts_list_state();
        shortcuts
    }

    /// Get starred project GIDs for saving to config.
    ///
    pub fn get_starred_project_gids(&self) -> Vec<String> {
        self.starred_projects.iter().cloned().collect()
    }

    /// Get starred project names map for saving to config.
    ///
    pub fn get_starred_project_names(&self) -> HashMap<String, String> {
        self.starred_project_names.clone()
    }

    /// Enter search mode. Only works when in Projects list or ProjectTasks view.
    ///
    pub fn enter_search_mode(&mut self) -> &mut Self {
        // If in Menu focus but not on TopList, switch to TopList first
        if *self.current_focus() == Focus::Menu && *self.current_menu() != Menu::TopList {
            self.current_menu = Menu::TopList;
        }

        // Only allow search in Projects list (TopList menu) or ProjectTasks view
        let can_search = match self.current_focus() {
            Focus::Menu => *self.current_menu() == Menu::TopList,
            Focus::View => {
                matches!(self.current_view(), View::ProjectTasks)
            }
        };

        if can_search {
            let new_target = match self.current_focus() {
                Focus::Menu => SearchTarget::Projects,
                Focus::View => SearchTarget::Tasks,
            };

            // If switching targets, clear the query
            if self.search_target != Some(new_target.clone()) {
                self.search_query.clear();
            }
            // If query is empty, we're starting fresh
            // If query is not empty and same target, we're re-entering to edit

            self.search_mode = true;
            self.search_target = Some(new_target);
            // Initialize filtered lists if needed
            if self.filtered_projects.is_empty() && !self.projects.is_empty() {
                self.filtered_projects = self.projects.clone();
            }
            if self.filtered_tasks.is_empty() && !self.tasks.is_empty() {
                self.filtered_tasks = self.tasks.clone();
            }
            self.update_search_filters();
        }
        self
    }

    /// Exit search mode. Keeps filtered results visible for navigation.
    ///
    pub fn exit_search_mode(&mut self) -> &mut Self {
        self.search_mode = false;
        // Don't clear search_query or search_target - keep filters active
        // Don't repopulate filtered lists - keep filtered results

        // Ensure selections are valid for the filtered lists
        if let Some(target) = &self.search_target {
            match target {
                SearchTarget::Projects => {
                    if !self.filtered_projects.is_empty() {
                        let current = self.projects_list_state.selected().unwrap_or(0);
                        if current >= self.filtered_projects.len() {
                            self.projects_list_state.select(Some(0));
                        }
                    } else {
                        self.projects_list_state.select(None);
                    }
                }
                SearchTarget::Tasks => {
                    if !self.filtered_tasks.is_empty() {
                        let current = self.tasks_list_state.selected().unwrap_or(0);
                        if current >= self.filtered_tasks.len() {
                            self.tasks_list_state.select(Some(0));
                        }
                    } else {
                        self.tasks_list_state.select(None);
                    }
                }
            }
        }

        self
    }

    /// Clear search completely (clears query and filters).
    ///
    #[allow(dead_code)]
    pub fn clear_search(&mut self) -> &mut Self {
        self.search_mode = false;
        self.search_query.clear();
        self.search_target = None;

        // Repopulate filtered lists with full lists
        self.filtered_projects = self.projects.clone();
        self.filtered_tasks = self.tasks.clone();

        // Reset selections to valid indices in the full lists
        if !self.projects.is_empty() {
            let current = self.projects_list_state.selected().unwrap_or(0);
            if current >= self.projects.len() {
                self.projects_list_state.select(Some(0));
            } else {
                self.projects_list_state.select(Some(current));
            }
        }

        if !self.tasks.is_empty() {
            let current = self.tasks_list_state.selected().unwrap_or(0);
            if current >= self.tasks.len() {
                self.tasks_list_state.select(Some(0));
            } else {
                self.tasks_list_state.select(Some(current));
            }
        }

        self
    }

    /// Add a character to the search query.
    ///
    pub fn add_search_char(&mut self, c: char) -> &mut Self {
        self.search_query.push(c);
        self.update_search_filters();
        self
    }

    /// Remove the last character from the search query.
    ///
    pub fn remove_search_char(&mut self) -> &mut Self {
        self.search_query.pop();
        self.update_search_filters();
        self
    }

    /// Update filtered lists based on search query. Only filters the target list.
    ///
    fn update_search_filters(&mut self) {
        if self.search_query.is_empty() {
            // If query is empty, show all items for the search target
            if let Some(target) = &self.search_target {
                match target {
                    SearchTarget::Projects => {
                        self.filtered_projects = self.projects.clone();
                    }
                    SearchTarget::Tasks => {
                        self.filtered_tasks = self.tasks.clone();
                    }
                }
            } else {
                // No target, reset both
                self.filtered_projects = self.projects.clone();
                self.filtered_tasks = self.tasks.clone();
            }
        } else {
            let query_lower = self.search_query.to_lowercase();

            // Only filter the target list
            if let Some(target) = &self.search_target {
                match target {
                    SearchTarget::Projects => {
                        self.filtered_projects = self
                            .projects
                            .iter()
                            .filter(|p| p.name.to_lowercase().contains(&query_lower))
                            .cloned()
                            .collect();
                        // Don't filter tasks when searching projects
                        self.filtered_tasks = self.tasks.clone();
                    }
                    SearchTarget::Tasks => {
                        self.filtered_tasks = self
                            .tasks
                            .iter()
                            .filter(|t| t.name.to_lowercase().contains(&query_lower))
                            .cloned()
                            .collect();
                        // Don't filter projects when searching tasks
                        self.filtered_projects = self.projects.clone();
                    }
                }
            }
        }

        // Reset selection to first item if current selection is out of bounds
        if let Some(target) = &self.search_target {
            match target {
                SearchTarget::Projects => {
                    if !self.filtered_projects.is_empty() {
                        let current = self.projects_list_state.selected().unwrap_or(0);
                        if current >= self.filtered_projects.len() {
                            self.projects_list_state.select(Some(0));
                        }
                    } else {
                        self.projects_list_state.select(None);
                    }
                }
                SearchTarget::Tasks => {
                    if !self.filtered_tasks.is_empty() {
                        let current = self.tasks_list_state.selected().unwrap_or(0);
                        if current >= self.filtered_tasks.len() {
                            self.tasks_list_state.select(Some(0));
                        }
                    } else {
                        self.tasks_list_state.select(None);
                    }
                }
            }
        }
    }

    /// Get search query.
    ///
    pub fn get_search_query(&self) -> &str {
        &self.search_query
    }

    /// Check if in search mode.
    ///
    pub fn is_search_mode(&self) -> bool {
        self.search_mode
    }

    /// Get the current search target.
    ///
    pub fn get_search_target(&self) -> Option<&SearchTarget> {
        self.search_target.as_ref()
    }

    /// Get filtered projects (or all if not searching).
    ///
    /// Enter debug mode for navigating and copying logs.
    ///
    pub fn enter_debug_mode(&mut self) -> &mut Self {
        self.debug_mode = true;
        // Set debug_index to the most recent log (last in the list)
        if !self.debug_entries.is_empty() {
            self.debug_index = self.debug_entries.len() - 1;
        } else {
            self.debug_index = 0;
        }
        self
    }

    /// Exit debug mode.
    ///
    pub fn exit_debug_mode(&mut self) -> &mut Self {
        self.debug_mode = false;
        self
    }

    /// Check if in debug mode.
    ///
    pub fn is_debug_mode(&self) -> bool {
        self.debug_mode
    }

    /// Get current debug index.
    ///
    pub fn get_debug_index(&self) -> usize {
        self.debug_index
    }

    /// Navigate to next log entry.
    ///
    pub fn next_debug(&mut self) -> &mut Self {
        if !self.debug_entries.is_empty() {
            self.debug_index = (self.debug_index + 1) % self.debug_entries.len();
        }
        self
    }

    /// Navigate to previous log entry.
    ///
    pub fn previous_debug(&mut self) -> &mut Self {
        if !self.debug_entries.is_empty() {
            if self.debug_index == 0 {
                self.debug_index = self.debug_entries.len() - 1;
            } else {
                self.debug_index -= 1;
            }
        }
        self
    }

    /// Get the currently selected log entry.
    ///
    pub fn get_current_debug(&self) -> Option<&String> {
        self.debug_entries.get(self.debug_index)
    }

    /// Add a log entry to the debug buffer.
    ///
    pub fn add_log_entry(&mut self, entry: String) {
        self.debug_entries.push(entry);
        // Keep only the last 1000 log entries to prevent memory issues
        if self.debug_entries.len() > 1000 {
            self.debug_entries.remove(0);
            // Adjust debug_index if we removed entries before it
            if self.debug_index > 0 {
                self.debug_index -= 1;
            }
        }
        // Always update index to point to the newest log so the list auto-scrolls
        // This ensures new logs are visible at the bottom
        if !self.debug_entries.is_empty() {
            self.debug_index = self.debug_entries.len() - 1;
        }
    }

    /// Get debug entries for rendering (read-only access).
    ///
    pub fn get_debug_entries(&self) -> &[String] {
        &self.debug_entries
    }

    pub fn get_filtered_projects(&self) -> &[Project] {
        // Show filtered results if we have a search query and target, even if not in search mode
        if !self.search_query.is_empty()
            && matches!(self.search_target, Some(SearchTarget::Projects))
        {
            &self.filtered_projects
        } else {
            &self.projects
        }
    }

    /// Get filtered tasks (or all if not searching tasks).
    ///
    pub fn get_filtered_tasks(&self) -> Vec<Task> {
        // Start with search-filtered tasks if applicable
        let base_tasks = if !self.search_query.is_empty()
            && matches!(self.search_target, Some(SearchTarget::Tasks))
        {
            &self.filtered_tasks
        } else {
            &self.tasks
        };

        // Apply task filter (All, Incomplete, Completed, Assignee)
        match &self.task_filter {
            TaskFilter::All => base_tasks.to_vec(),
            TaskFilter::Incomplete => base_tasks
                .iter()
                .filter(|t| !t.completed)
                .cloned()
                .collect(),
            TaskFilter::Completed => base_tasks.iter().filter(|t| t.completed).cloned().collect(),
            TaskFilter::Assignee(assignee_gid) => base_tasks
                .iter()
                .filter(|t| {
                    match (assignee_gid, &t.assignee) {
                        (None, None) => true, // Unassigned tasks
                        (Some(gid), Some(assignee)) => &assignee.gid == gid,
                        _ => false,
                    }
                })
                .cloned()
                .collect(),
        }
    }

    /// Dispatches an asynchronous network event.
    ///
    pub fn dispatch(&self, event: NetworkEvent) {
        if let Some(net_sender) = &self.net_sender {
            if let Err(err) = net_sender.send(event) {
                error!("Recieved error from network dispatch: {}", err);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::uuid::UUIDv4;
    use fake::{Fake, Faker};
    use uuid::Uuid;

    #[test]
    fn get_user() {
        let user: User = Faker.fake();
        let state = State {
            user: Some(user.to_owned()),
            ..State::default()
        };
        assert_eq!(user, *state.get_user().unwrap());
    }

    #[test]
    fn set_user() {
        let mut state = State::default();
        let user: User = Faker.fake();
        state.set_user(user.to_owned());
        assert_eq!(user, state.user.unwrap());
    }

    #[test]
    fn get_active_workspace() {
        let workspaces = vec![
            Faker.fake::<Workspace>(),
            Faker.fake::<Workspace>(),
            Faker.fake::<Workspace>(),
        ];
        let active_workspace = workspaces[0].to_owned();
        let state = State {
            active_workspace_gid: Some(active_workspace.gid.to_owned()),
            workspaces,
            ..State::default()
        };
        assert_eq!(active_workspace, *state.get_active_workspace().unwrap());
    }

    #[test]
    fn set_active_workspace() {
        let workspace_gid: Uuid = UUIDv4.fake();
        let mut state = State::default();
        state.set_active_workspace(workspace_gid.to_string());
        assert_eq!(
            workspace_gid.to_string(),
            state.active_workspace_gid.unwrap()
        );
    }

    #[test]
    fn set_workspaces() {
        let workspaces = vec![
            Faker.fake::<Workspace>(),
            Faker.fake::<Workspace>(),
            Faker.fake::<Workspace>(),
        ];
        let mut state = State::default();
        state.set_workspaces(workspaces.to_owned());
        assert_eq!(workspaces, state.workspaces);
    }

    #[test]
    fn set_terminal_size() {
        let mut state = State::default();
        let size = Rect::new(Faker.fake(), Faker.fake(), Faker.fake(), Faker.fake());
        state.set_terminal_size(size);
        assert_eq!(size, state.terminal_size);
    }

    #[test]
    fn advance_spinner_index() {
        let mut state = State::default();
        state.advance_spinner_index();
        assert_eq!(state.spinner_index, 1);
        for _ in 0..SPINNER_FRAME_COUNT {
            state.advance_spinner_index();
        }
        assert_eq!(state.spinner_index, 1);
    }

    #[test]
    fn get_spinner_index() {
        let state = State {
            spinner_index: 2,
            ..State::default()
        };
        assert_eq!(*state.get_spinner_index(), 2);
    }

    #[test]
    fn current_focus() {
        let mut state = State {
            current_focus: Focus::Menu,
            ..State::default()
        };
        assert_eq!(*state.current_focus(), Focus::Menu);
        state.current_focus = Focus::View;
        assert_eq!(*state.current_focus(), Focus::View);
    }

    #[test]
    fn focus_menu() {
        let mut state = State {
            current_focus: Focus::View,
            ..State::default()
        };
        state.focus_menu();
        assert_eq!(state.current_focus, Focus::Menu);
    }

    #[test]
    fn focus_view() {
        let mut state = State {
            current_focus: Focus::Menu,
            ..State::default()
        };
        state.focus_view();
        assert_eq!(state.current_focus, Focus::View);
    }

    #[test]
    fn current_menu() {
        let state = State {
            current_menu: Menu::Status,
            ..State::default()
        };
        assert_eq!(*state.current_menu(), Menu::Status);
    }

    #[test]
    fn next_menu() {
        let mut state = State {
            current_menu: Menu::Status,
            ..State::default()
        };
        state.next_menu();
        assert_eq!(state.current_menu, Menu::Shortcuts);
        state.next_menu();
        assert_eq!(state.current_menu, Menu::TopList);
        state.next_menu();
        assert_eq!(state.current_menu, Menu::Status);
    }

    #[test]
    fn previous_menu() {
        let mut state = State {
            current_menu: Menu::Status,
            ..State::default()
        };
        state.previous_menu();
        assert_eq!(state.current_menu, Menu::TopList);
        state.previous_menu();
        assert_eq!(state.current_menu, Menu::Shortcuts);
        state.previous_menu();
        assert_eq!(state.current_menu, Menu::Status);
    }

    #[test]
    fn select_status_menu() {
        let mut state = State {
            view_stack: vec![View::MyTasks],
            ..State::default()
        };
        state.select_status_menu();
        assert_eq!(*state.view_stack.last().unwrap(), View::Welcome);
    }

    #[test]
    fn current_shortcut_index() {
        let state = State {
            current_shortcut_index: 0,
            ..State::default()
        };
        assert_eq!(*state.current_shortcut_index(), 0);
    }

    #[test]
    fn next_shortcut_index() {
        let mut state = State {
            current_shortcut_index: 0,
            ..State::default()
        };
        state.next_shortcut_index();
        assert_eq!(state.current_shortcut_index, 1);
        state.next_shortcut_index();
        assert_eq!(state.current_shortcut_index, 2);
        state.next_shortcut_index();
        assert_eq!(state.current_shortcut_index, 0);
    }

    #[test]
    fn previous_shortcut_index() {
        let mut state = State {
            current_shortcut_index: 0,
            ..State::default()
        };
        state.previous_shortcut_index();
        assert_eq!(state.current_shortcut_index, 2);
        state.previous_shortcut_index();
        assert_eq!(state.current_shortcut_index, 1);
        state.previous_shortcut_index();
        assert_eq!(state.current_shortcut_index, 0);
    }

    #[test]
    fn select_current_shortcut_index() {
        let mut state = State {
            current_shortcut_index: 0,
            current_focus: Focus::Menu,
            ..State::default()
        };
        state.select_current_shortcut_index();
        assert_eq!(*state.view_stack.last().unwrap(), View::MyTasks);
        assert_eq!(state.current_focus, Focus::View);
        state.current_shortcut_index = 1;
        state.select_current_shortcut_index();
        assert_eq!(*state.view_stack.last().unwrap(), View::RecentlyModified);
        assert_eq!(state.current_focus, Focus::View);
    }

    #[test]
    fn current_top_list_index() {
        let state = State {
            current_top_list_index: 2,
            ..State::default()
        };
        assert_eq!(*state.current_top_list_index(), 2);
    }

    #[test]
    fn next_top_list_index_when_nonempty() {
        let mut state = State {
            current_top_list_index: 0,
            projects: vec![Faker.fake::<Project>(), Faker.fake::<Project>()],
            ..State::default()
        };
        state.next_top_list_index();
        assert_eq!(state.current_top_list_index, 1);
        state.next_top_list_index();
        assert_eq!(state.current_top_list_index, 0);
    }

    #[test]
    fn next_top_list_index_when_empty() {
        let mut state = State {
            current_top_list_index: 0,
            projects: vec![],
            ..State::default()
        };
        state.next_top_list_index();
        assert_eq!(state.current_top_list_index, 0);
    }

    #[test]
    fn previous_top_list_index_when_nonempty() {
        let mut state = State {
            current_top_list_index: 0,
            projects: vec![Faker.fake::<Project>(), Faker.fake::<Project>()],
            ..State::default()
        };
        state.previous_top_list_index();
        assert_eq!(state.current_top_list_index, 1);
        state.previous_top_list_index();
        assert_eq!(state.current_top_list_index, 0);
    }

    #[test]
    fn previous_top_list_index_when_empty() {
        let mut state = State {
            current_top_list_index: 0,
            projects: vec![],
            ..State::default()
        };
        state.previous_top_list_index();
        assert_eq!(state.current_top_list_index, 0);
    }

    #[test]
    fn current_view() {
        let mut state = State {
            view_stack: vec![View::MyTasks],
            ..State::default()
        };
        assert_eq!(*state.current_view(), View::MyTasks);
        state.view_stack = vec![View::RecentlyCompleted];
        assert_eq!(*state.current_view(), View::RecentlyCompleted);
    }

    #[test]
    fn get_tasks() {
        let tasks = vec![
            Faker.fake::<Task>(),
            Faker.fake::<Task>(),
            Faker.fake::<Task>(),
        ];
        let state = State {
            tasks: tasks.to_owned(),
            ..State::default()
        };
        assert_eq!(tasks, *state.get_tasks());
    }

    #[test]
    fn set_tasks() {
        let mut state = State::default();
        let tasks = vec![
            Faker.fake::<Task>(),
            Faker.fake::<Task>(),
            Faker.fake::<Task>(),
        ];
        state.set_tasks(tasks.to_owned());
        assert_eq!(tasks, state.tasks);
    }

    #[test]
    fn get_projects() {
        let projects = vec![
            Faker.fake::<Project>(),
            Faker.fake::<Project>(),
            Faker.fake::<Project>(),
        ];
        let state = State {
            projects: projects.to_owned(),
            ..State::default()
        };
        assert_eq!(projects, *state.get_projects());
    }

    #[test]
    fn set_projects() {
        let mut state = State::default();
        let projects = vec![
            Faker.fake::<Project>(),
            Faker.fake::<Project>(),
            Faker.fake::<Project>(),
        ];
        state.set_projects(projects.to_owned());
        assert_eq!(projects, state.projects);
    }
}
