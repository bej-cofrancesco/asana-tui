use crate::app::NetworkEventSender;
use crate::asana::{CustomField, Project, Section, Story, Task, User, Workspace};
use crate::config::{HotkeyAction, ViewHotkeys};
use crate::events::network::Event as NetworkEvent;
use crate::ui::SPINNER_FRAME_COUNT;
use log::*;
use ratatui::layout::Rect;
use ratatui::widgets::ListState;
use std::collections::{HashMap, HashSet};
use tui_textarea::TextArea;

// Import types from new modules - enums are now in separate modules
use super::form::{base_shortcuts, CustomFieldValue, EditFormState, TaskFilter};
use super::navigation::{Focus, Menu, SearchTarget, TaskDetailPanel, View, ViewMode};

/// Houses data representative of application state.
///
/// Note: The State struct is kept here for now since all methods reference it.
/// In a future refactoring, we can move methods to separate modules.
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
    comments_list_state: ListState,
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
    move_task_gid: Option<String>,       // GID of task being moved (for section selection modal)
    theme_selector_open: bool,           // Whether theme selector modal is open
    theme_dropdown_index: usize,         // Selected index in theme selector
    current_task_detail: Option<Task>,   // Currently viewed task with full details
    sections: Vec<Section>,              // Project sections for kanban
    workspace_users: Vec<User>,          // Users for assignment dropdowns
    task_stories: Vec<Story>,            // Comments for current task
    view_mode: ViewMode,                 // List or Kanban view
    #[allow(dead_code)]
    edit_mode: bool, // Whether in edit mode
    edit_form_state: Option<EditFormState>, // Current form field being edited
    field_editing_mode: bool,            // Whether actively editing a field (vs navigating)
    kanban_column_index: usize,          // Current column in kanban view
    kanban_task_index: usize,            // Current task index in selected column
    #[allow(dead_code)]
    kanban_horizontal_scroll: usize, // Horizontal scroll offset for kanban columns
    comment_input_mode: bool,            // Whether in comment input mode
    comment_input_text: String,          // Current comment text being typed
    #[allow(dead_code)]
    comments_scroll_offset: usize, // Scroll offset for comments list
    details_scroll_offset: usize,        // Scroll offset for details panel
    notes_scroll_offset: usize,          // Scroll offset for notes panel
    current_task_panel: TaskDetailPanel, // Current panel in task detail view
    // Form input fields
    form_name: String,
    form_notes_textarea: TextArea<'static>, // TextArea for multi-line notes editing
    form_assignee: Option<String>,          // GID of selected assignee
    form_assignee_search: String,           // Search text for filtering assignees
    form_due_on: String,                    // Date string
    form_section: Option<String>,           // GID of selected section
    form_section_search: String,            // Search text for filtering sections
    // Original form values (for tracking changes)
    original_form_name: String,
    original_form_notes: String,
    original_form_assignee: Option<String>,
    original_form_due_on: String,
    original_form_section: Option<String>,
    // Dropdown selection indices
    assignee_dropdown_index: usize,
    section_dropdown_index: usize,
    // Custom fields
    project_custom_fields: Vec<CustomField>, // Custom fields available for the current project
    form_custom_field_values: HashMap<String, CustomFieldValue>, // GID -> value for form
    custom_field_search: HashMap<String, String>, // GID -> search text for enum/people fields
    custom_field_dropdown_index: HashMap<String, usize>, // GID -> dropdown index
    form_scroll_offset: usize, // Scroll offset for form fields (how many fields to skip)
    access_token_input: String, // Input field for welcome screen
    has_access_token: bool,    // Whether access token exists (user is logged in)
    auth_error: Option<String>, // Error message if authentication fails
    theme: crate::ui::Theme,   // Current theme
    hotkeys: ViewHotkeys,      // Hotkey bindings per view
    hotkey_editor_open: bool,  // Whether hotkey editor modal is open
    hotkey_editor_view: Option<View>, // Which view is being edited
    hotkey_editor_selected_action: Option<HotkeyAction>, // Action being edited
    hotkey_editor_dropdown_index: usize, // Selected index in hotkey editor
}

// SearchTarget and TaskDetailPanel are now in navigation.rs

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
            comments_list_state: ListState::default(),
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
            task_filter: TaskFilter::All,
            delete_confirmation: None,
            move_task_gid: None,
            theme_selector_open: false,
            theme_dropdown_index: 0,
            current_task_detail: None,
            sections: vec![],
            workspace_users: vec![],
            task_stories: vec![],
            view_mode: ViewMode::Kanban,
            edit_mode: false,
            edit_form_state: None,
            field_editing_mode: false,
            kanban_column_index: 0,
            kanban_task_index: 0,
            kanban_horizontal_scroll: 0,
            comment_input_mode: false,
            comment_input_text: String::new(),
            comments_scroll_offset: 0,
            details_scroll_offset: 0,
            notes_scroll_offset: 0,
            current_task_panel: TaskDetailPanel::Details,
            form_name: String::new(),
            form_notes_textarea: TextArea::default(),
            form_assignee: None,
            form_assignee_search: String::new(),
            form_due_on: String::new(),
            form_section: None,
            form_section_search: String::new(),
            original_form_name: String::new(),
            original_form_notes: String::new(),
            original_form_assignee: None,
            original_form_due_on: String::new(),
            original_form_section: None,
            assignee_dropdown_index: 0,
            section_dropdown_index: 0,
            project_custom_fields: vec![],
            form_custom_field_values: HashMap::new(),
            custom_field_search: HashMap::new(),
            custom_field_dropdown_index: HashMap::new(),
            form_scroll_offset: 0,
            access_token_input: String::new(),
            has_access_token: false, // Default to false, will be set when token is loaded
            auth_error: None,        // No error initially
            theme: crate::ui::Theme::default(),
            hotkeys: ViewHotkeys::default(),
            hotkey_editor_open: false,
            hotkey_editor_view: None,
            hotkey_editor_selected_action: None,
            hotkey_editor_dropdown_index: 0,
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
        theme: crate::ui::Theme,
        hotkeys: ViewHotkeys,
    ) -> Self {
        State {
            net_sender: Some(net_sender),
            config_save_sender: Some(config_save_sender),
            starred_projects: starred_projects.into_iter().collect(),
            starred_project_names,
            debug_entries: vec![], // Initialize empty, will be populated by logger
            has_access_token,
            theme,
            hotkeys,
            ..State::default()
        }
    }

    /// Get the current theme.
    ///
    pub fn get_theme(&self) -> &crate::ui::Theme {
        &self.theme
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

        // It's a starred project
        if let Some(project) = self.projects.iter().find(|p| p.name == shortcut.as_str()) {
            self.project = Some(project.to_owned());
            self.tasks.clear();
            self.dispatch(NetworkEvent::ProjectTasks);
            self.view_stack.push(View::ProjectTasks);
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
        self.current_top_list_index = next; // Keep for backward compatibility
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
        self.current_top_list_index = prev; // Keep for backward compatibility
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
        // view_stack is always initialized with at least one view in State::new
        // This should never be None, but we provide a fallback for safety
        self.view_stack
            .last()
            .expect("view_stack should never be empty")
    }

    /// Push a view onto the view stack.
    ///
    pub fn push_view(&mut self, view: View) -> &mut Self {
        // Close theme selector if pushing a non-Welcome view
        if !matches!(view, View::Welcome) {
            self.close_theme_selector();
        }
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

    /// Get comments list state.
    ///
    pub fn get_comments_list_state(&mut self) -> &mut ListState {
        &mut self.comments_list_state
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

    /// Set delete confirmation for a task GID (used from detail view and kanban view).
    ///
    pub fn set_delete_confirmation(&mut self, task_gid: String) -> &mut Self {
        self.delete_confirmation = Some(task_gid);
        self
    }

    /// Confirm and delete the task with pending confirmation (works for all views).
    ///
    pub fn confirm_delete_task(&mut self) -> &mut Self {
        if let Some(task_gid) = &self.delete_confirmation {
            let gid = task_gid.clone();
            self.delete_confirmation = None;
            self.dispatch(NetworkEvent::DeleteTask { gid });
        }
        self
    }

    /// Get the current task filter.
    ///
    #[allow(dead_code)]
    pub fn get_task_filter(&self) -> TaskFilter {
        self.task_filter.clone()
    }

    /// Set the task filter.
    ///
    #[allow(dead_code)]
    pub fn set_task_filter(&mut self, filter: TaskFilter) -> &mut Self {
        self.task_filter = filter;
        // Update filtered tasks when filter changes
        self.update_search_filters();
        self
    }

    /// Cycle to the next task filter.
    ///
    #[allow(dead_code)]
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
        // Validate and adjust column index if needed
        let visible_indices = self.get_visible_section_indices();
        if !visible_indices.is_empty() {
            // If current column index is not in visible indices, reset to first visible
            if !visible_indices.contains(&self.kanban_column_index) {
                self.kanban_column_index = visible_indices[0];
                self.kanban_task_index = 0;
            }
        } else if !self.sections.is_empty() {
            // If no visible sections but we have sections, reset to first section
            self.kanban_column_index = 0;
            self.kanban_task_index = 0;
        }
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
        // Initialize comments list state - select first comment (oldest)
        let comments: Vec<&Story> = self
            .task_stories
            .iter()
            .filter(|s| match &s.resource_subtype {
                Some(subtype) => subtype == "comment_added",
                None => s.created_by.is_some(),
            })
            .collect();
        if !comments.is_empty() {
            self.comments_list_state.select(Some(0));
        } else {
            self.comments_list_state.select(None);
        }
        self
    }

    /// Get task stories/comments.
    ///
    pub fn get_task_stories(&self) -> &[Story] {
        &self.task_stories
    }

    #[allow(dead_code)]
    pub fn get_comments_scroll_offset(&self) -> usize {
        self.comments_scroll_offset
    }

    pub fn scroll_comments_down(&mut self) -> &mut Self {
        // Use ListState for proper navigation
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
            let current = self.comments_list_state.selected().unwrap_or(0);
            let next = if current >= total_comments.saturating_sub(1) {
                0 // Wrap to top
            } else {
                current + 1
            };
            self.comments_list_state.select(Some(next));
        } else {
            self.comments_list_state.select(None);
        }
        self
    }

    pub fn scroll_comments_up(&mut self) -> &mut Self {
        // Use ListState for proper navigation
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
            let current = self.comments_list_state.selected().unwrap_or(0);
            let prev = if current == 0 {
                total_comments.saturating_sub(1) // Wrap to bottom
            } else {
                current - 1
            };
            self.comments_list_state.select(Some(prev));
        } else {
            self.comments_list_state.select(None);
        }
        self
    }

    // Details panel scrolling

    #[allow(dead_code)]
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

    #[allow(dead_code)]
    pub fn reset_details_scroll(&mut self) -> &mut Self {
        self.details_scroll_offset = 0;
        self
    }

    // Notes panel scrolling

    #[allow(dead_code)]
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

    #[allow(dead_code)]
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

    #[allow(dead_code)]
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
        // Validate against visible sections to prevent crashes when filtering
        let visible_indices = self.get_visible_section_indices();
        if !visible_indices.is_empty() {
            // If index is in visible sections, use it; otherwise use first visible
            if visible_indices.contains(&index) {
                self.kanban_column_index = index;
            } else {
                self.kanban_column_index = visible_indices[0];
            }
        } else {
            // No visible sections, clamp to sections length
            self.kanban_column_index = index.min(self.sections.len().saturating_sub(1));
        }
        self
    }

    /// Get kanban column index.
    ///
    pub fn get_kanban_column_index(&self) -> usize {
        self.kanban_column_index
    }

    /// Get visible sections (sections that have tasks after filtering).
    /// Returns a vector of section indices that should be shown.
    ///
    pub fn get_visible_section_indices(&self) -> Vec<usize> {
        let sections = &self.sections;
        let tasks = self.get_filtered_tasks();

        // Get all sections that have tasks after filtering
        let sections_with_tasks: Vec<usize> = sections
            .iter()
            .enumerate()
            .filter_map(|(idx, section)| {
                // Check if this section has any filtered tasks
                if tasks.iter().any(|t| {
                    t.section
                        .as_ref()
                        .map(|s| s.gid == section.gid)
                        .unwrap_or(false)
                }) {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect();

        // If we have a search/filter active and some sections have no tasks, only show sections with tasks
        if !self.search_query.is_empty() && sections_with_tasks.len() < sections.len() {
            // Filtering is active - only show sections with tasks
            sections_with_tasks
        } else {
            // No filtering or all sections have tasks - show all sections
            (0..sections.len()).collect::<Vec<_>>()
        }
    }

    /// Navigate to next kanban column.
    ///
    pub fn next_kanban_column(&mut self) -> &mut Self {
        let visible_indices = self.get_visible_section_indices();

        if visible_indices.is_empty() {
            return self;
        }

        // Find current position in visible indices
        let current_pos = visible_indices
            .iter()
            .position(|&idx| idx == self.kanban_column_index)
            .unwrap_or(0);

        // Move to next position, wrapping around
        let next_pos = (current_pos + 1) % visible_indices.len();
        self.kanban_column_index = visible_indices[next_pos];

        // Reset task index when changing columns
        self.kanban_task_index = 0;
        // Auto-scroll to keep selected column visible
        self.auto_scroll_to_column();

        self
    }

    /// Navigate to previous kanban column.
    ///
    pub fn previous_kanban_column(&mut self) -> &mut Self {
        let visible_indices = self.get_visible_section_indices();

        if visible_indices.is_empty() {
            return self;
        }

        // Find current position in visible indices
        let current_pos = visible_indices
            .iter()
            .position(|&idx| idx == self.kanban_column_index)
            .unwrap_or(0);

        // Move to previous position, wrapping around
        let prev_pos = if current_pos > 0 {
            current_pos - 1
        } else {
            visible_indices.len() - 1
        };
        self.kanban_column_index = visible_indices[prev_pos];

        // Reset task index when changing columns
        self.kanban_task_index = 0;
        // Auto-scroll to keep selected column visible
        self.auto_scroll_to_column();

        self
    }

    /// Auto-scroll to keep the selected column visible.
    /// With fixed 3-column layout, this is now a no-op but kept for compatibility.
    ///
    fn auto_scroll_to_column(&mut self) {
        // With fixed 3-column layout, we always show 3 columns centered around selection
        // No horizontal scrolling needed, but we keep this function for compatibility
        // The rendering code handles centering the current column in the 3 visible columns
    }

    /// Set task GID for moving (opens section selection modal).
    ///
    pub fn set_move_task_gid(&mut self, task_gid: Option<String>) -> &mut Self {
        let should_init = task_gid.is_some();
        self.move_task_gid = task_gid;
        // Initialize section dropdown index when opening modal
        if should_init {
            self.init_section_dropdown_index();
        }
        self
    }

    /// Get task GID being moved.
    ///
    pub fn get_move_task_gid(&self) -> Option<&String> {
        self.move_task_gid.as_ref()
    }

    /// Check if move task modal is open.
    ///
    pub fn has_move_task(&self) -> bool {
        self.move_task_gid.is_some()
    }

    /// Clear move task state.
    ///
    pub fn clear_move_task(&mut self) -> &mut Self {
        self.move_task_gid = None;
        self
    }

    /// Open theme selector modal.
    ///
    pub fn open_theme_selector(&mut self) -> &mut Self {
        self.theme_selector_open = true;
        // Initialize dropdown index to current theme
        let available_themes = crate::ui::Theme::available_themes();
        if let Some(current_index) = available_themes
            .iter()
            .position(|name| name == &self.theme.name)
        {
            self.theme_dropdown_index = current_index;
        } else {
            self.theme_dropdown_index = 0;
        }
        self
    }

    /// Close theme selector modal.
    ///
    pub fn close_theme_selector(&mut self) -> &mut Self {
        self.theme_selector_open = false;
        self
    }

    /// Check if theme selector modal is open.
    ///
    pub fn has_theme_selector(&self) -> bool {
        self.theme_selector_open
    }

    /// Check if in theme selector mode.
    ///
    pub fn is_theme_mode(&self) -> bool {
        self.theme_selector_open
    }

    /// Get theme dropdown index.
    ///
    pub fn get_theme_dropdown_index(&self) -> usize {
        self.theme_dropdown_index
    }

    /// Set theme dropdown index.
    ///
    #[allow(dead_code)]
    pub fn set_theme_dropdown_index(&mut self, index: usize) -> &mut Self {
        self.theme_dropdown_index = index;
        self
    }

    /// Navigate to next theme in selector.
    ///
    pub fn next_theme(&mut self) -> &mut Self {
        let available_themes = crate::ui::Theme::available_themes();
        if !available_themes.is_empty() {
            self.theme_dropdown_index = (self.theme_dropdown_index + 1) % available_themes.len();
        }
        self
    }

    /// Navigate to previous theme in selector.
    ///
    pub fn previous_theme(&mut self) -> &mut Self {
        let available_themes = crate::ui::Theme::available_themes();
        if !available_themes.is_empty() {
            if self.theme_dropdown_index == 0 {
                self.theme_dropdown_index = available_themes.len() - 1;
            } else {
                self.theme_dropdown_index -= 1;
            }
        }
        self
    }

    /// Get hotkeys.
    ///
    pub fn get_hotkeys(&self) -> &ViewHotkeys {
        &self.hotkeys
    }

    /// Set hotkeys.
    ///
    pub fn set_hotkeys(&mut self, hotkeys: ViewHotkeys) -> &mut Self {
        self.hotkeys = hotkeys;
        self
    }

    /// Open hotkey editor modal.
    ///
    pub fn open_hotkey_editor(&mut self) -> &mut Self {
        self.hotkey_editor_open = true;
        self.hotkey_editor_view = Some(self.current_view().clone());
        self.hotkey_editor_dropdown_index = 0;
        self
    }

    /// Close hotkey editor modal.
    ///
    pub fn close_hotkey_editor(&mut self) -> &mut Self {
        self.hotkey_editor_open = false;
        self.hotkey_editor_view = None;
        self.hotkey_editor_selected_action = None;
        self
    }

    /// Check if hotkey editor modal is open.
    ///
    pub fn has_hotkey_editor(&self) -> bool {
        self.hotkey_editor_open
    }

    /// Get hotkey editor view.
    ///
    pub fn get_hotkey_editor_view(&self) -> Option<&View> {
        self.hotkey_editor_view.as_ref()
    }

    /// Set hotkey editor view.
    ///
    #[allow(dead_code)]
    pub fn set_hotkey_editor_view(&mut self, view: Option<View>) -> &mut Self {
        self.hotkey_editor_view = view;
        self
    }

    /// Get hotkey editor dropdown index.
    ///
    pub fn get_hotkey_editor_dropdown_index(&self) -> usize {
        self.hotkey_editor_dropdown_index
    }

    /// Get hotkey editor selected action.
    ///
    pub fn get_hotkey_editor_selected_action(&self) -> Option<&HotkeyAction> {
        self.hotkey_editor_selected_action.as_ref()
    }

    /// Set hotkey editor selected action.
    ///
    pub fn set_hotkey_editor_selected_action(&mut self, action: Option<HotkeyAction>) -> &mut Self {
        self.hotkey_editor_selected_action = action;
        self
    }

    /// Navigate to next hotkey action in editor.
    ///
    pub fn next_hotkey_action(&mut self) -> &mut Self {
        // Get actions for current view
        let actions = if let Some(view) = &self.hotkey_editor_view {
            match view {
                View::Welcome => &self.hotkeys.welcome,
                View::ProjectTasks => &self.hotkeys.project_tasks,
                View::TaskDetail => &self.hotkeys.task_detail,
                View::CreateTask => &self.hotkeys.create_task,
                View::EditTask => &self.hotkeys.edit_task,
            }
        } else {
            return self;
        };

        if !actions.is_empty() {
            self.hotkey_editor_dropdown_index =
                (self.hotkey_editor_dropdown_index + 1) % actions.len();
        }
        self
    }

    /// Navigate to previous hotkey action in editor.
    ///
    pub fn previous_hotkey_action(&mut self) -> &mut Self {
        // Get actions for current view
        let actions = if let Some(view) = &self.hotkey_editor_view {
            match view {
                View::Welcome => &self.hotkeys.welcome,
                View::ProjectTasks => &self.hotkeys.project_tasks,
                View::TaskDetail => &self.hotkeys.task_detail,
                View::CreateTask => &self.hotkeys.create_task,
                View::EditTask => &self.hotkeys.edit_task,
            }
        } else {
            return self;
        };

        if !actions.is_empty() {
            if self.hotkey_editor_dropdown_index == 0 {
                self.hotkey_editor_dropdown_index = actions.len() - 1;
            } else {
                self.hotkey_editor_dropdown_index -= 1;
            }
        }
        self
    }

    /// Select current theme and apply it.
    ///
    pub fn select_theme(&mut self) -> &mut Self {
        let available_themes = crate::ui::Theme::available_themes();
        if let Some(theme_name) = available_themes.get(self.theme_dropdown_index) {
            if let Some(new_theme) = crate::ui::Theme::from_name(theme_name) {
                self.theme = new_theme;
                // Trigger config save
                if let Some(sender) = &self.config_save_sender {
                    let _ = sender.send(());
                }
            }
        }
        self.close_theme_selector()
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
            // Use filtered tasks to respect search/filter
            let filtered_tasks = self.get_filtered_tasks();
            let section_tasks: Vec<&Task> = filtered_tasks
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
            } else {
                // No tasks in this section after filtering, reset index
                self.kanban_task_index = 0;
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
            // Use filtered tasks to respect search/filter
            let filtered_tasks = self.get_filtered_tasks();
            let section_tasks: Vec<&Task> = filtered_tasks
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
            } else {
                // No tasks in this section after filtering, reset index
                self.kanban_task_index = 0;
            }
        }
        self
    }

    /// Get current selected task in kanban view.
    ///
    pub fn get_kanban_selected_task(&self) -> Option<Task> {
        if self.sections.is_empty() || self.kanban_column_index >= self.sections.len() {
            return None;
        }

        let section = &self.sections[self.kanban_column_index];
        // Use filtered tasks to respect search/filter
        let filtered_tasks = self.get_filtered_tasks();
        let section_tasks: Vec<&Task> = filtered_tasks
            .iter()
            .filter(|t| {
                t.section
                    .as_ref()
                    .map(|s| s.gid == section.gid)
                    .unwrap_or(false)
            })
            .collect();

        if self.kanban_task_index < section_tasks.len() {
            Some(section_tasks[self.kanban_task_index].clone())
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

    /// Check if actively editing a field (vs navigating between fields).
    ///
    pub fn is_field_editing_mode(&self) -> bool {
        self.field_editing_mode
    }

    /// Enter field editing mode (start editing the current field).
    ///
    pub fn enter_field_editing_mode(&mut self) -> &mut Self {
        self.field_editing_mode = true;
        self
    }

    /// Exit field editing mode (return to navigation mode).
    ///
    pub fn exit_field_editing_mode(&mut self) -> &mut Self {
        self.field_editing_mode = false;
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
    #[allow(dead_code)]
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
    /// Get form notes as string.
    ///
    pub fn get_form_notes(&self) -> String {
        self.form_notes_textarea
            .lines()
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Get form notes textarea (mutable).
    ///
    pub fn get_form_notes_textarea(&mut self) -> &mut TextArea<'static> {
        &mut self.form_notes_textarea
    }

    /// Set form notes from string.
    ///
    pub fn set_form_notes(&mut self, notes: String) -> &mut Self {
        self.form_notes_textarea = TextArea::from(notes.lines().collect::<Vec<_>>());
        self
    }

    /// Get form assignee.
    ///
    pub fn get_form_assignee(&self) -> Option<&String> {
        self.form_assignee.as_ref()
    }

    /// Set form assignee.
    ///
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    pub fn set_form_section(&mut self, section: Option<String>) -> &mut Self {
        self.form_section = section;
        self
    }

    pub fn get_assignee_dropdown_index(&self) -> usize {
        self.assignee_dropdown_index
    }

    #[allow(dead_code)]
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

    #[allow(dead_code)]
    pub fn clear_assignee_search(&mut self) -> &mut Self {
        self.form_assignee_search.clear();
        self.assignee_dropdown_index = 0;
        self
    }

    pub fn get_section_dropdown_index(&self) -> usize {
        self.section_dropdown_index
    }

    #[allow(dead_code)]
    pub fn set_section_dropdown_index(&mut self, index: usize) -> &mut Self {
        self.section_dropdown_index = index;
        self
    }

    /// Get filtered sections based on search text
    pub fn get_filtered_sections(&self) -> Vec<Section> {
        let search = self.form_section_search.to_lowercase();
        if search.is_empty() {
            self.sections.clone()
        } else {
            self.sections
                .iter()
                .filter(|s| s.name.to_lowercase().contains(&search))
                .cloned()
                .collect()
        }
    }

    pub fn add_section_search_char(&mut self, c: char) -> &mut Self {
        self.form_section_search.push(c);
        // Reset dropdown index when search changes
        self.section_dropdown_index = 0;
        self
    }

    pub fn backspace_section_search(&mut self) -> &mut Self {
        self.form_section_search.pop();
        // Reset dropdown index when search changes
        self.section_dropdown_index = 0;
        self
    }

    pub fn get_section_search(&self) -> &str {
        &self.form_section_search
    }

    #[allow(dead_code)]
    pub fn clear_section_search(&mut self) -> &mut Self {
        self.form_section_search.clear();
        self.section_dropdown_index = 0;
        self
    }

    /// Initialize section dropdown index to match currently selected section (if any).
    /// This ensures the selected section is shown when entering the dropdown.
    pub fn init_section_dropdown_index(&mut self) -> &mut Self {
        if let Some(selected_gid) = &self.form_section {
            let filtered_sections = self.get_filtered_sections();
            if let Some(index) = filtered_sections
                .iter()
                .position(|s| &s.gid == selected_gid)
            {
                self.section_dropdown_index = index;
            } else {
                // Selected section not in filtered list, reset to 0
                self.section_dropdown_index = 0;
            }
        } else {
            // No section selected, start at 0
            self.section_dropdown_index = 0;
        }
        self
    }

    pub fn next_section(&mut self) -> &mut Self {
        let sections = self.get_filtered_sections();
        if !sections.is_empty() {
            self.section_dropdown_index = (self.section_dropdown_index + 1) % sections.len();
        }
        self
    }

    pub fn previous_section(&mut self) -> &mut Self {
        let sections = self.get_filtered_sections();
        if !sections.is_empty() {
            if self.section_dropdown_index == 0 {
                self.section_dropdown_index = sections.len() - 1;
            } else {
                self.section_dropdown_index -= 1;
            }
        }
        self
    }

    pub fn select_current_section(&mut self) -> &mut Self {
        let sections = self.get_filtered_sections();
        if !sections.is_empty() && self.section_dropdown_index < sections.len() {
            self.form_section = Some(sections[self.section_dropdown_index].gid.clone());
        } else {
            warn!("No section found at index {}", self.section_dropdown_index);
        }
        self
    }

    /// Clear form fields.
    ///
    pub fn clear_form(&mut self) -> &mut Self {
        self.form_name.clear();
        self.form_notes_textarea = TextArea::default();
        self.form_assignee = None;
        self.form_assignee_search.clear();
        self.form_due_on.clear();
        self.form_section = None;
        self.form_section_search.clear();
        self.form_custom_field_values.clear();
        self.custom_field_search.clear();
        self.custom_field_dropdown_index.clear();
        self.edit_form_state = None;
        self.field_editing_mode = false;
        self.assignee_dropdown_index = 0;
        self.section_dropdown_index = 0;
        self.form_scroll_offset = 0;
        self
    }

    /// Get available custom fields for the current project.
    ///
    pub fn get_project_custom_fields(&self) -> &[CustomField] {
        &self.project_custom_fields
    }

    /// Get enabled custom fields (excluding custom_id and disabled fields).
    ///
    pub fn get_enabled_custom_fields(&self) -> Vec<&CustomField> {
        self.project_custom_fields
            .iter()
            .filter(|cf| {
                // Skip custom_id fields
                let is_custom_id = cf
                    .representation_type
                    .as_ref()
                    .map(|s| s == "custom_id")
                    .unwrap_or(false)
                    || cf.id_prefix.is_some();
                // Only include enabled fields
                !is_custom_id && cf.enabled
            })
            .collect()
    }

    /// Set project custom fields.
    ///
    pub fn set_project_custom_fields(&mut self, fields: Vec<CustomField>) -> &mut Self {
        self.project_custom_fields = fields;
        self
    }

    /// Get custom field value for a given GID.
    ///
    pub fn get_custom_field_value(&self, gid: &str) -> Option<&CustomFieldValue> {
        self.form_custom_field_values.get(gid)
    }

    /// Get all custom field values (for API calls).
    ///
    pub fn get_form_custom_field_values(&self) -> &HashMap<String, CustomFieldValue> {
        &self.form_custom_field_values
    }

    /// Set custom field value for a given GID.
    ///
    #[allow(dead_code)]
    pub fn set_custom_field_value(&mut self, gid: String, value: CustomFieldValue) -> &mut Self {
        self.form_custom_field_values.insert(gid, value);
        self
    }

    /// Get custom field search text.
    ///
    pub fn get_custom_field_search(&self, gid: &str) -> &str {
        self.custom_field_search
            .get(gid)
            .map(|s| s.as_str())
            .unwrap_or("")
    }

    /// Add character to custom field search.
    ///
    pub fn add_custom_field_search_char(&mut self, gid: String, c: char) -> &mut Self {
        self.custom_field_search.entry(gid).or_default().push(c);
        self
    }

    /// Backspace custom field search.
    ///
    pub fn backspace_custom_field_search(&mut self, gid: &str) -> &mut Self {
        if let Some(search) = self.custom_field_search.get_mut(gid) {
            search.pop();
        }
        self
    }

    /// Get custom field dropdown index.
    ///
    pub fn get_custom_field_dropdown_index(&self, gid: &str) -> usize {
        *self.custom_field_dropdown_index.get(gid).unwrap_or(&0)
    }

    /// Set custom field dropdown index.
    ///
    pub fn set_custom_field_dropdown_index(&mut self, gid: String, index: usize) -> &mut Self {
        self.custom_field_dropdown_index.insert(gid, index);
        self
    }

    /// Add character to custom field text/number/date value.
    ///
    pub fn add_custom_field_text_char(
        &mut self,
        gid: String,
        c: char,
        field_type: &str,
    ) -> &mut Self {
        match field_type {
            "number" => {
                // For number fields, parse the current value and append the character
                let current_text = self
                    .form_custom_field_values
                    .get(&gid)
                    .and_then(|v| match v {
                        CustomFieldValue::Number(Some(n)) => Some(n.to_string()),
                        CustomFieldValue::Text(s) => Some(s.clone()),
                        _ => None,
                    })
                    .unwrap_or_default();

                let new_text = current_text + &c.to_string();
                // Try to parse as number
                if let Ok(num) = new_text.parse::<f64>() {
                    self.form_custom_field_values
                        .insert(gid, CustomFieldValue::Number(Some(num)));
                } else {
                    // If parsing fails, store as text (will be ignored when sending)
                    self.form_custom_field_values
                        .insert(gid, CustomFieldValue::Text(new_text));
                }
            }
            _ => {
                // For text and date fields, store as text
                let current = self
                    .form_custom_field_values
                    .get(&gid)
                    .and_then(|v| match v {
                        CustomFieldValue::Text(s) => Some(s.clone()),
                        CustomFieldValue::Date(Some(d)) => Some(d.clone()),
                        _ => None,
                    })
                    .unwrap_or_default();

                let new_value = current + &c.to_string();
                if field_type == "date" {
                    self.form_custom_field_values
                        .insert(gid, CustomFieldValue::Date(Some(new_value)));
                } else {
                    self.form_custom_field_values
                        .insert(gid, CustomFieldValue::Text(new_value));
                }
            }
        }
        self
    }

    /// Remove character from custom field text/number/date value.
    ///
    pub fn remove_custom_field_text_char(&mut self, gid: &str, field_type: &str) -> &mut Self {
        match field_type {
            "number" => {
                // For number fields, get current value and remove last character
                let current_text = self
                    .form_custom_field_values
                    .get(gid)
                    .and_then(|v| match v {
                        CustomFieldValue::Number(Some(n)) => Some(n.to_string()),
                        CustomFieldValue::Text(s) => Some(s.clone()),
                        _ => None,
                    })
                    .unwrap_or_default();

                let mut new_text = current_text;
                new_text.pop();

                if new_text.is_empty() {
                    self.form_custom_field_values.remove(gid);
                } else if let Ok(num) = new_text.parse::<f64>() {
                    self.form_custom_field_values
                        .insert(gid.to_string(), CustomFieldValue::Number(Some(num)));
                } else {
                    self.form_custom_field_values
                        .insert(gid.to_string(), CustomFieldValue::Text(new_text));
                }
            }
            _ => {
                // For text and date fields
                if let Some(value) = self.form_custom_field_values.get(gid) {
                    let mut new_value = match value {
                        CustomFieldValue::Text(s) => s.clone(),
                        CustomFieldValue::Date(Some(d)) => d.clone(),
                        _ => return self,
                    };
                    new_value.pop();

                    if new_value.is_empty() {
                        self.form_custom_field_values.remove(gid);
                    } else if field_type == "date" {
                        self.form_custom_field_values
                            .insert(gid.to_string(), CustomFieldValue::Date(Some(new_value)));
                    } else {
                        self.form_custom_field_values
                            .insert(gid.to_string(), CustomFieldValue::Text(new_value));
                    }
                }
            }
        }
        self
    }

    /// Get current custom field being edited.
    ///
    pub fn get_current_custom_field(&self) -> Option<(usize, &CustomField)> {
        if let Some(EditFormState::CustomField(idx)) = self.edit_form_state {
            self.project_custom_fields.get(idx).map(|cf| (idx, cf))
        } else {
            None
        }
    }

    /// Navigate to next enum option in custom field dropdown.
    ///
    pub fn next_custom_field_enum(&mut self, gid: &str, count: usize) -> &mut Self {
        let current = self.get_custom_field_dropdown_index(gid);
        let new_idx = if current + 1 >= count { 0 } else { current + 1 };
        self.set_custom_field_dropdown_index(gid.to_string(), new_idx);
        self
    }

    /// Navigate to previous enum option in custom field dropdown.
    ///
    pub fn previous_custom_field_enum(&mut self, gid: &str, count: usize) -> &mut Self {
        let current = self.get_custom_field_dropdown_index(gid);
        let new_idx = if current == 0 {
            count.saturating_sub(1)
        } else {
            current - 1
        };
        self.set_custom_field_dropdown_index(gid.to_string(), new_idx);
        self
    }

    /// Select current enum option for custom field.
    ///
    pub fn select_custom_field_enum(&mut self, gid: String, enum_gid: String) -> &mut Self {
        self.form_custom_field_values
            .insert(gid, CustomFieldValue::Enum(Some(enum_gid)));
        self
    }

    /// Toggle multi-enum option for custom field.
    ///
    pub fn toggle_custom_field_multi_enum(&mut self, gid: &str, enum_gid: String) -> &mut Self {
        let current = self
            .form_custom_field_values
            .get(gid)
            .and_then(|v| match v {
                CustomFieldValue::MultiEnum(gids) => Some(gids.clone()),
                _ => None,
            })
            .unwrap_or_default();

        let mut new_gids = current;
        if new_gids.contains(&enum_gid) {
            new_gids.retain(|g| g != &enum_gid);
        } else {
            new_gids.push(enum_gid);
        }
        self.form_custom_field_values
            .insert(gid.to_string(), CustomFieldValue::MultiEnum(new_gids));
        self
    }

    /// Toggle people option for custom field.
    ///
    pub fn toggle_custom_field_people(&mut self, gid: &str, user_gid: String) -> &mut Self {
        let current = self
            .form_custom_field_values
            .get(gid)
            .and_then(|v| match v {
                CustomFieldValue::People(gids) => Some(gids.clone()),
                _ => None,
            })
            .unwrap_or_default();

        let mut new_gids = current;
        if new_gids.contains(&user_gid) {
            new_gids.retain(|g| g != &user_gid);
        } else {
            new_gids.push(user_gid);
        }
        self.form_custom_field_values
            .insert(gid.to_string(), CustomFieldValue::People(new_gids));
        self
    }

    /// Get form scroll offset.
    ///
    #[allow(dead_code)]
    pub fn get_form_scroll_offset(&self) -> usize {
        self.form_scroll_offset
    }

    /// Scroll form up (show earlier fields).
    ///
    pub fn scroll_form_up(&mut self) -> &mut Self {
        if self.form_scroll_offset > 0 {
            self.form_scroll_offset -= 1;
        }
        self
    }

    /// Scroll form down (show later fields).
    ///
    #[allow(dead_code)]
    pub fn scroll_form_down(&mut self) -> &mut Self {
        // Don't scroll past the last field - limit is handled in rendering
        self.form_scroll_offset += 1;
        self
    }

    /// Get total number of form fields (standard + custom).
    ///
    #[allow(dead_code)]
    pub fn get_total_form_field_count(&self) -> usize {
        // Standard fields: Name, Notes, Assignee, DueDate, Section = 5
        5 + self.project_custom_fields.len()
    }

    /// Initialize edit form with task data.
    ///
    #[allow(dead_code)]
    pub fn init_edit_form(&mut self, task: &Task) -> &mut Self {
        // Set current form values
        self.form_name = task.name.clone();
        self.set_form_notes(task.notes.clone().unwrap_or_default());
        self.form_assignee = task.assignee.as_ref().map(|u| u.gid.clone());
        self.form_due_on = task.due_on.clone().unwrap_or_default();
        self.form_section = task.section.as_ref().map(|s| s.gid.clone());
        // Store original values for change detection
        self.original_form_name = task.name.clone();
        self.original_form_notes = task.notes.clone().unwrap_or_default();
        self.original_form_assignee = task.assignee.as_ref().map(|u| u.gid.clone());
        self.original_form_due_on = task.due_on.clone().unwrap_or_default();
        self.original_form_section = task.section.as_ref().map(|s| s.gid.clone());
        // Initialize custom field values from task
        self.form_custom_field_values.clear();
        self.custom_field_dropdown_index.clear();
        for cf in &task.custom_fields {
            let value = match cf.resource_subtype.as_str() {
                "text" => CustomFieldValue::Text(cf.text_value.clone().unwrap_or_default()),
                "number" => CustomFieldValue::Number(cf.number_value),
                "date" => CustomFieldValue::Date(cf.date_value.clone()),
                "enum" => CustomFieldValue::Enum(cf.enum_value.as_ref().map(|e| e.gid.clone())),
                "multi_enum" => CustomFieldValue::MultiEnum(
                    cf.multi_enum_values.iter().map(|e| e.gid.clone()).collect(),
                ),
                "people" => CustomFieldValue::People(
                    cf.people_value.iter().map(|u| u.gid.clone()).collect(),
                ),
                _ => continue,
            };
            self.form_custom_field_values.insert(cf.gid.clone(), value);

            // Initialize dropdown index for enum fields with selected values
            if cf.resource_subtype == "enum" {
                if let Some(selected_enum_gid) = cf.enum_value.as_ref().map(|e| &e.gid) {
                    // Find the project custom field definition to get enum options
                    if let Some(project_cf) = self
                        .project_custom_fields
                        .iter()
                        .find(|pcf| pcf.gid == cf.gid)
                    {
                        if let Some(index) = project_cf
                            .enum_options
                            .iter()
                            .position(|eo| eo.gid == *selected_enum_gid)
                        {
                            self.custom_field_dropdown_index
                                .insert(cf.gid.clone(), index);
                        }
                    }
                }
            }
        }
        // Initialize assignee and section dropdown indices
        self.init_assignee_dropdown_index();
        self.init_section_dropdown_index();
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
                    // Also validate kanban column index when filtering tasks
                    // This prevents crashes when sections become hidden due to filtering
                    let visible_indices = self.get_visible_section_indices();
                    if !visible_indices.is_empty()
                        && !visible_indices.contains(&self.kanban_column_index)
                    {
                        // Current column is not visible, move to first visible
                        self.kanban_column_index = visible_indices[0];
                        self.kanban_task_index = 0;
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
        let workspace_gid: Uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440100")
            .expect("Hardcoded test UUID should be valid");
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
            view_stack: vec![View::Welcome],
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
        // Add starred projects to make shortcuts list non-empty
        state.starred_projects.insert("proj1".to_string());
        state.starred_projects.insert("proj2".to_string());
        state.starred_projects.insert("proj3".to_string());
        state
            .starred_project_names
            .insert("proj1".to_string(), "Project 1".to_string());
        state
            .starred_project_names
            .insert("proj2".to_string(), "Project 2".to_string());
        state
            .starred_project_names
            .insert("proj3".to_string(), "Project 3".to_string());
        state.shortcuts_list_state.select(Some(0));
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
        // Add starred projects to make shortcuts list non-empty
        state.starred_projects.insert("proj1".to_string());
        state.starred_projects.insert("proj2".to_string());
        state.starred_projects.insert("proj3".to_string());
        state
            .starred_project_names
            .insert("proj1".to_string(), "Project 1".to_string());
        state
            .starred_project_names
            .insert("proj2".to_string(), "Project 2".to_string());
        state
            .starred_project_names
            .insert("proj3".to_string(), "Project 3".to_string());
        state.shortcuts_list_state.select(Some(0));
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
        // Add starred projects to make shortcuts list non-empty
        let project1 = Faker.fake::<Project>();
        state.starred_projects.insert(project1.gid.clone());
        state
            .starred_project_names
            .insert(project1.gid.clone(), project1.name.clone());
        state.projects.push(project1);
        state.shortcuts_list_state.select(Some(0));
        state.select_current_shortcut_index();
        // select_current_shortcut_index calls focus_view() which switches to View focus
        assert_eq!(*state.current_focus(), Focus::View);
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
            ..State::default()
        };
        let projects = vec![Faker.fake::<Project>(), Faker.fake::<Project>()];
        state.set_projects(projects);
        state.projects_list_state.select(Some(0));
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
            ..State::default()
        };
        let projects = vec![Faker.fake::<Project>(), Faker.fake::<Project>()];
        state.set_projects(projects);
        state.projects_list_state.select(Some(0));
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
        let state = State {
            view_stack: vec![View::Welcome],
            ..State::default()
        };
        assert_eq!(*state.current_view(), View::Welcome);
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
