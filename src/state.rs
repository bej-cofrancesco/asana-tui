use crate::app::NetworkEventSender;
use crate::asana::{Project, Task, User, Workspace};
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
}

/// Specifies which panel is being searched.
///
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SearchTarget {
    Projects,
    Tasks,
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
        }
    }
}

impl State {
    pub fn new(
        net_sender: NetworkEventSender,
        config_save_sender: crate::app::ConfigSaveSender,
        starred_projects: Vec<String>,
        starred_project_names: HashMap<String, String>,
    ) -> Self {
        State {
            net_sender: Some(net_sender),
            config_save_sender: Some(config_save_sender),
            starred_projects: starred_projects.into_iter().collect(),
            starred_project_names,
            debug_entries: vec![], // Initialize empty, will be populated by logger
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
                // For now, we'll always toggle to completed=true
                // In a full implementation, we'd check the current status
                self.dispatch(NetworkEvent::UpdateTask {
                    gid: task.gid.to_owned(),
                    completed: Some(true),
                });
            }
        }
        self
    }

    /// Delete the selected task.
    ///
    pub fn delete_selected_task(&mut self) -> &mut Self {
        let filtered = self.get_filtered_tasks();
        if let Some(selected_index) = self.tasks_list_state.selected() {
            if selected_index < filtered.len() {
                let task = &filtered[selected_index];
                self.dispatch(NetworkEvent::DeleteTask {
                    gid: task.gid.to_owned(),
                });
            }
        }
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
        // If in debug mode, update index to point to the newest log
        if self.debug_mode && !self.debug_entries.is_empty() {
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
    pub fn get_filtered_tasks(&self) -> &[Task] {
        // Show filtered results if we have a search query and target, even if not in search mode
        if !self.search_query.is_empty() && matches!(self.search_target, Some(SearchTarget::Tasks))
        {
            &self.filtered_tasks
        } else {
            &self.tasks
        }
    }

    /// Dispatches an asynchronous network event.
    ///
    fn dispatch(&self, event: NetworkEvent) {
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
