//! Navigation-related state types.
//!
//! This module contains enums and types related to navigation, views, menus, and focus.

/// Specifying the different foci.
///
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Focus {
    Menu,
    View,
}

/// Specifying the different menus.
///
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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
    ProjectTasks,
    TaskDetail,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_focus() {
        assert_eq!(Focus::Menu, Focus::Menu);
        assert_eq!(Focus::View, Focus::View);
        assert_ne!(Focus::Menu, Focus::View);
    }

    #[test]
    fn test_menu() {
        assert_eq!(Menu::Status, Menu::Status);
        assert_eq!(Menu::Shortcuts, Menu::Shortcuts);
        assert_eq!(Menu::TopList, Menu::TopList);
    }

    #[test]
    fn test_view() {
        assert_eq!(View::Welcome, View::Welcome);
        assert_eq!(View::ProjectTasks, View::ProjectTasks);
        assert_eq!(View::TaskDetail, View::TaskDetail);
        assert_eq!(View::CreateTask, View::CreateTask);
        assert_eq!(View::EditTask, View::EditTask);
    }

    #[test]
    fn test_view_mode() {
        assert_eq!(ViewMode::List, ViewMode::List);
        assert_eq!(ViewMode::Kanban, ViewMode::Kanban);
        assert_ne!(ViewMode::List, ViewMode::Kanban);
    }

    #[test]
    fn test_search_target() {
        assert_eq!(SearchTarget::Projects, SearchTarget::Projects);
        assert_eq!(SearchTarget::Tasks, SearchTarget::Tasks);
        assert_ne!(SearchTarget::Projects, SearchTarget::Tasks);
    }

    #[test]
    fn test_task_detail_panel() {
        assert_eq!(TaskDetailPanel::Details, TaskDetailPanel::Details);
        assert_eq!(TaskDetailPanel::Comments, TaskDetailPanel::Comments);
        assert_eq!(TaskDetailPanel::Notes, TaskDetailPanel::Notes);
    }
}
