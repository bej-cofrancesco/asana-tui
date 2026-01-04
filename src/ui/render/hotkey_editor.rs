//! Hotkey editor modal rendering.
//!
//! This module provides the UI for editing hotkey bindings grouped by category.

use super::Frame;
use crate::config::hotkeys::{
    format_hotkey_display, get_all_hotkeys_grouped, Hotkey, HotkeyAction, HotkeyGroup,
};
use crate::state::State;
use crate::ui::widgets::styling;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

/// Render hotkey editor modal.
///
pub fn render_hotkey_editor(frame: &mut Frame, size: Rect, state: &State) {
    // Create a centered popup dialog using ratatui pattern
    let popup_area = centered_rect(70, 75, size);

    // Clear the area first (ratatui modal pattern)
    frame.render_widget(Clear, popup_area);

    // Get all hotkeys grouped by category
    let grouped_hotkeys = get_all_hotkeys_grouped(state.get_hotkeys());

    // Flatten into a single list with group headers
    let mut all_items: Vec<(Option<&HotkeyGroup>, Option<&HotkeyAction>, Option<&Hotkey>)> =
        Vec::new();
    for (group, actions) in &grouped_hotkeys {
        // Add group header
        all_items.push((Some(group), None, None));
        // Add actions in this group
        for (action, hotkey_opt) in actions {
            all_items.push((None, Some(action), hotkey_opt.as_ref()));
        }
    }

    let selected_index = state.get_hotkey_editor_dropdown_index();

    // Split popup into title and list areas
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(10)])
        .split(popup_area);

    // Title block
    let theme = state.get_theme();
    let instructions = if state.get_hotkey_editor_selected_action().is_some() {
        // When editing, show cancel instruction
        if let Some(cancel_hotkey) = state
            .get_hotkeys()
            .welcome
            .get(&crate::config::hotkeys::HotkeyAction::Cancel)
        {
            format!(
                " Press a key to bind, {}: cancel",
                crate::config::hotkeys::format_hotkey_display(cancel_hotkey)
            )
        } else {
            " Press a key to bind, Esc: cancel".to_string()
        }
    } else {
        // When browsing, show navigation instructions
        crate::config::hotkeys::build_hotkey_editor_instructions(state.get_hotkeys())
    };

    let title_block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            "Edit Hotkeys",
            Style::default()
                .fg(theme.info.to_color())
                .add_modifier(Modifier::BOLD),
        ))
        .border_style(styling::active_block_border_style(theme));

    let title_text = Paragraph::new(instructions)
        .block(title_block)
        .alignment(Alignment::Center);
    frame.render_widget(title_text, chunks[0]);

    // Limit visible items to max 15 items (with scrolling)
    let max_visible = 15;
    let total_items = all_items.len();
    let start_index = if total_items <= max_visible {
        0
    } else {
        (selected_index as i32 - max_visible as i32 / 2)
            .max(0)
            .min((total_items - max_visible) as i32) as usize
    };
    let end_index = (start_index + max_visible).min(total_items);
    let visible_items = if all_items.is_empty() {
        vec![]
    } else {
        all_items[start_index..end_index].to_vec()
    };
    let visible_selected = selected_index.saturating_sub(start_index);

    // Create list items from visible items
    let items: Vec<ListItem> = if visible_items.is_empty() {
        vec![ListItem::new("No hotkeys available")]
    } else {
        visible_items
            .iter()
            .map(|(group_opt, action_opt, hotkey_opt)| {
                if let Some(group) = group_opt {
                    // Group header
                    ListItem::new(Line::from(vec![Span::styled(
                        format!("▶ {} ", group.name),
                        Style::default()
                            .fg(theme.info.to_color())
                            .add_modifier(Modifier::BOLD),
                    )]))
                } else if let Some(action) = action_opt {
                    // Action item
                    let action_name = format_action_name(action);
                    let hotkey_str = if let Some(hotkey) = hotkey_opt {
                        format_hotkey_display(hotkey)
                    } else {
                        "<unbound>".to_string()
                    };
                    let is_selected = state
                        .get_hotkey_editor_selected_action()
                        .as_ref()
                        .map(|a| a == action)
                        .unwrap_or(false);
                    if is_selected {
                        ListItem::new(Line::from(vec![
                            Span::styled(
                                format!("  {}: ", action_name),
                                Style::default().fg(theme.warning.to_color()),
                            ),
                            Span::styled(
                                "<press key>".to_string(),
                                Style::default()
                                    .fg(theme.warning.to_color())
                                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                            ),
                        ]))
                    } else {
                        ListItem::new(format!("  {}: {}", action_name, hotkey_str))
                    }
                } else {
                    ListItem::new("")
                }
            })
            .collect()
    };

    // Use ListState for proper selection display
    let mut list_state = ratatui::widgets::ListState::default();
    if !items.is_empty() && !all_items.is_empty() {
        let safe_index = visible_selected.min(items.len().saturating_sub(1));
        list_state.select(Some(safe_index));
    }

    // Create list block
    let list_block = Block::default()
        .borders(Borders::ALL)
        .title(format!("Hotkeys ({} total)", total_items))
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
        HotkeyAction::NavigateNext => "Navigate Next".to_string(),
        HotkeyAction::NavigatePrev => "Navigate Prev".to_string(),
        HotkeyAction::NavigateLeft => "Navigate Left".to_string(),
        HotkeyAction::NavigateRight => "Navigate Right".to_string(),
        HotkeyAction::ToggleStar => "Toggle Star".to_string(),
        HotkeyAction::EnterSearch => "Enter Search".to_string(),
        HotkeyAction::EnterDebug => "Enter Debug".to_string(),
        HotkeyAction::Select => "Select".to_string(),
        HotkeyAction::Cancel => "Cancel".to_string(),
        HotkeyAction::Quit => "Quit".to_string(),
        HotkeyAction::OpenThemeSelector => "Open Theme Selector".to_string(),
        HotkeyAction::OpenHotkeyEditor => "Open Hotkey Editor".to_string(),
        HotkeyAction::ViewTask => "View Task".to_string(),
        HotkeyAction::CreateTask => "Create Task".to_string(),
        HotkeyAction::MoveTask => "Move Task".to_string(),
        HotkeyAction::ToggleTaskComplete => "Toggle Task Complete".to_string(),
        HotkeyAction::DeleteTask => "Delete Task".to_string(),
        HotkeyAction::Back => "Back".to_string(),
        HotkeyAction::EditTask => "Edit Task".to_string(),
        HotkeyAction::AddComment => "Add Comment".to_string(),
        HotkeyAction::EditField => "Edit Field".to_string(),
        HotkeyAction::SubmitForm => "Submit Form".to_string(),
        HotkeyAction::SearchModeExit => "Search Mode Exit".to_string(),
        HotkeyAction::DebugModeCopyLog => "Debug Mode Copy Log".to_string(),
        HotkeyAction::DebugModeExit => "Debug Mode Exit".to_string(),
        HotkeyAction::DeleteConfirm => "Delete Confirm".to_string(),
        HotkeyAction::MoveTaskConfirm => "Move Task Confirm".to_string(),
        HotkeyAction::MoveTaskCancel => "Move Task Cancel".to_string(),
        HotkeyAction::ThemeSelectorSelect => "Theme Selector Select".to_string(),
        HotkeyAction::ThemeSelectorCancel => "Theme Selector Cancel".to_string(),
        HotkeyAction::FilterByAssignee => "Filter By Assignee".to_string(),
        HotkeyAction::AssigneeFilterSelect => "Assignee Filter Select".to_string(),
        HotkeyAction::AssigneeFilterCancel => "Assignee Filter Cancel".to_string(),
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
