//! Hotkey editor modal rendering.
//!
//! This module provides the UI for editing hotkey bindings per view.

use super::Frame;
use crate::config::{Hotkey, HotkeyAction};
use crate::state::State;
use crate::ui::widgets::styling;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

/// Render hotkey editor modal.
///
pub fn render_hotkey_editor(frame: &mut Frame, size: Rect, state: &State) {
    // Create a centered popup dialog using ratatui pattern
    let popup_area = centered_rect(60, 60, size);

    // Clear the area first (ratatui modal pattern)
    frame.render_widget(Clear, popup_area);

    // Get current view being edited
    let current_view = state
        .get_hotkey_editor_view()
        .cloned()
        .unwrap_or_else(|| state.current_view().clone());

    // Get actions for current view
    let actions: Vec<(&HotkeyAction, &Hotkey)> = match current_view {
        crate::state::View::Welcome => state.get_hotkeys().welcome.iter().collect::<Vec<_>>(),
        crate::state::View::ProjectTasks => {
            state.get_hotkeys().project_tasks.iter().collect::<Vec<_>>()
        }
        crate::state::View::TaskDetail => {
            state.get_hotkeys().task_detail.iter().collect::<Vec<_>>()
        }
        crate::state::View::CreateTask => {
            state.get_hotkeys().create_task.iter().collect::<Vec<_>>()
        }
        crate::state::View::EditTask => state.get_hotkeys().edit_task.iter().collect::<Vec<_>>(),
    };

    let selected_index = state.get_hotkey_editor_dropdown_index();

    // Split popup into title and list areas
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(7)])
        .split(popup_area);

    // Title block
    let theme = state.get_theme();
    let view_name = match current_view {
        crate::state::View::Welcome => "Welcome",
        crate::state::View::ProjectTasks => "Project Tasks",
        crate::state::View::TaskDetail => "Task Detail",
        crate::state::View::CreateTask => "Create Task",
        crate::state::View::EditTask => "Edit Task",
    };

    let title_block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            format!("Edit Hotkeys - {}", view_name),
            Style::default()
                .fg(theme.info.to_color())
                .add_modifier(Modifier::BOLD),
        ))
        .border_style(styling::active_block_border_style(theme));

    let instructions = if state.get_hotkey_editor_selected_action().is_some() {
        " Press a key to bind, Esc: cancel"
    } else {
        " j/k: navigate, Enter: edit, Esc: cancel"
    };

    let title_text = Paragraph::new(instructions)
        .block(title_block)
        .alignment(Alignment::Center);
    frame.render_widget(title_text, chunks[0]);

    // Limit visible actions to max 10 items (with scrolling)
    let max_visible = 10;
    let total_items = actions.len();
    let start_index = if total_items <= max_visible {
        0
    } else {
        (selected_index as i32 - max_visible as i32 / 2)
            .max(0)
            .min((total_items - max_visible) as i32) as usize
    };
    let end_index = (start_index + max_visible).min(total_items);
    let visible_actions = if actions.is_empty() {
        vec![]
    } else {
        actions[start_index..end_index].to_vec()
    };
    let visible_selected = selected_index.saturating_sub(start_index);

    // Create list items from visible actions
    let items: Vec<ListItem> = if visible_actions.is_empty() {
        vec![ListItem::new("No hotkeys available")]
    } else {
        visible_actions
            .iter()
            .map(|(action, hotkey)| {
                let action_name = format_action_name(action);
                let hotkey_str = format_hotkey(hotkey);
                ListItem::new(format!("{}: {}", action_name, hotkey_str))
            })
            .collect()
    };

    // Use ListState for proper selection display
    let mut list_state = ratatui::widgets::ListState::default();
    if !items.is_empty() && !actions.is_empty() {
        let safe_index = visible_selected.min(items.len().saturating_sub(1));
        list_state.select(Some(safe_index));
    }

    // Create list block with action count
    let list_block = Block::default()
        .borders(Borders::ALL)
        .title(format!("Actions ({})", actions.len()))
        .border_style(styling::active_block_border_style(theme));

    let list = List::new(items)
        .block(list_block)
        .style(styling::normal_text_style(theme))
        .highlight_style(
            Style::default()
                .fg(theme.highlight_fg.to_color())
                .bg(theme.highlight_bg.to_color())
                .add_modifier(Modifier::BOLD),
        );

    frame.render_stateful_widget(list, chunks[1], &mut list_state);
}

/// Helper function to format action name for display.
///
fn format_action_name(action: &HotkeyAction) -> String {
    match action {
        HotkeyAction::NavigateMenuNext => "Navigate Menu Next".to_string(),
        HotkeyAction::NavigateMenuPrev => "Navigate Menu Prev".to_string(),
        HotkeyAction::NavigateMenuLeft => "Navigate Menu Left".to_string(),
        HotkeyAction::NavigateMenuRight => "Navigate Menu Right".to_string(),
        HotkeyAction::ToggleStar => "Toggle Star".to_string(),
        HotkeyAction::EnterSearch => "Enter Search".to_string(),
        HotkeyAction::EnterDebug => "Enter Debug".to_string(),
        HotkeyAction::Select => "Select".to_string(),
        HotkeyAction::Cancel => "Cancel".to_string(),
        HotkeyAction::Quit => "Quit".to_string(),
        HotkeyAction::OpenThemeSelector => "Open Theme Selector".to_string(),
        HotkeyAction::OpenHotkeyEditor => "Open Hotkey Editor".to_string(),
        HotkeyAction::NavigateTaskNext => "Navigate Task Next".to_string(),
        HotkeyAction::NavigateTaskPrev => "Navigate Task Prev".to_string(),
        HotkeyAction::NavigateColumnNext => "Navigate Column Next".to_string(),
        HotkeyAction::NavigateColumnPrev => "Navigate Column Prev".to_string(),
        HotkeyAction::ViewTask => "View Task".to_string(),
        HotkeyAction::CreateTask => "Create Task".to_string(),
        HotkeyAction::MoveTask => "Move Task".to_string(),
        HotkeyAction::ToggleTaskComplete => "Toggle Task Complete".to_string(),
        HotkeyAction::DeleteTask => "Delete Task".to_string(),
        HotkeyAction::Back => "Back".to_string(),
        HotkeyAction::SwitchPanelNext => "Switch Panel Next".to_string(),
        HotkeyAction::SwitchPanelPrev => "Switch Panel Prev".to_string(),
        HotkeyAction::ScrollDown => "Scroll Down".to_string(),
        HotkeyAction::ScrollUp => "Scroll Up".to_string(),
        HotkeyAction::EditTask => "Edit Task".to_string(),
        HotkeyAction::AddComment => "Add Comment".to_string(),
        HotkeyAction::NavigateFieldNext => "Navigate Field Next".to_string(),
        HotkeyAction::NavigateFieldPrev => "Navigate Field Prev".to_string(),
        HotkeyAction::EditField => "Edit Field".to_string(),
        HotkeyAction::SubmitForm => "Submit Form".to_string(),
        HotkeyAction::SearchModeExit => "Search Mode Exit".to_string(),
        HotkeyAction::DebugModeNavigateNext => "Debug Mode Navigate Next".to_string(),
        HotkeyAction::DebugModeNavigatePrev => "Debug Mode Navigate Prev".to_string(),
        HotkeyAction::DebugModeCopyLog => "Debug Mode Copy Log".to_string(),
        HotkeyAction::DebugModeExit => "Debug Mode Exit".to_string(),
        HotkeyAction::DeleteConfirm => "Delete Confirm".to_string(),
        HotkeyAction::MoveTaskNavigateNext => "Move Task Navigate Next".to_string(),
        HotkeyAction::MoveTaskNavigatePrev => "Move Task Navigate Prev".to_string(),
        HotkeyAction::MoveTaskConfirm => "Move Task Confirm".to_string(),
        HotkeyAction::MoveTaskCancel => "Move Task Cancel".to_string(),
        HotkeyAction::ThemeSelectorNavigateNext => "Theme Selector Navigate Next".to_string(),
        HotkeyAction::ThemeSelectorNavigatePrev => "Theme Selector Navigate Prev".to_string(),
        HotkeyAction::ThemeSelectorSelect => "Theme Selector Select".to_string(),
        HotkeyAction::ThemeSelectorCancel => "Theme Selector Cancel".to_string(),
    }
}

/// Helper function to format hotkey for display.
///
fn format_hotkey(hotkey: &Hotkey) -> String {
    let mut parts = Vec::new();
    if hotkey
        .modifiers
        .contains(crossterm::event::KeyModifiers::CONTROL)
    {
        parts.push("Ctrl");
    }
    if hotkey
        .modifiers
        .contains(crossterm::event::KeyModifiers::SHIFT)
    {
        parts.push("Shift");
    }
    if hotkey
        .modifiers
        .contains(crossterm::event::KeyModifiers::ALT)
    {
        parts.push("Alt");
    }

    let key_str = match &hotkey.code {
        crossterm::event::KeyCode::Char(c) => {
            if *c == ' ' {
                "Space".to_string()
            } else {
                c.to_string()
            }
        }
        crossterm::event::KeyCode::Esc => "Esc".to_string(),
        crossterm::event::KeyCode::Enter => "Enter".to_string(),
        crossterm::event::KeyCode::Backspace => "Backspace".to_string(),
        crossterm::event::KeyCode::Up => "Up".to_string(),
        crossterm::event::KeyCode::Down => "Down".to_string(),
        crossterm::event::KeyCode::Left => "Left".to_string(),
        crossterm::event::KeyCode::Right => "Right".to_string(),
        _ => "Unknown".to_string(),
    };

    if parts.is_empty() {
        key_str
    } else {
        format!("{}+{}", parts.join("+"), key_str)
    }
}

/// Helper function to create a centered rectangle (ratatui modal pattern).
///
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
