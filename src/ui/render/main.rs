use super::welcome;
use super::{Frame, create_task, edit_task, kanban, task_detail};
use crate::state::{State, View};
use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Render main widget according to state.
///
pub fn main(frame: &mut Frame, size: Rect, state: &mut State) {
    match state.current_view() {
        View::Welcome => {
            welcome(frame, size, state);
        }
        View::ProjectTasks => {
            // Always show kanban view first (so modal appears on top)
            kanban::kanban(frame, size, state);

            // Check if we need to show move task section selection modal (render on top)
            if state.has_move_task() {
                // Get task name for display before borrowing state mutably
                let task_name = state
                    .get_kanban_selected_task()
                    .map(|t| t.name.clone())
                    .unwrap_or_else(|| "task".to_string());
                render_move_task_modal(frame, size, &task_name, state);
            }

            // Check if we need to show delete confirmation dialog (render on top of everything)
            if state.has_delete_confirmation() {
                let task_name = state
                    .get_kanban_selected_task()
                    .map(|t| t.name.clone())
                    .unwrap_or_else(|| "this task".to_string());
                render_delete_confirmation(frame, size, &task_name, state);
            }
        }
        View::TaskDetail => {
            task_detail::task_detail(frame, size, state);

            // Check if we need to show delete confirmation dialog (render on top)
            if state.has_delete_confirmation() {
                let task_name = state
                    .get_task_detail()
                    .map(|t| t.name.clone())
                    .unwrap_or_else(|| "this task".to_string());
                render_delete_confirmation(frame, size, &task_name, state);
            }
        }
        View::KanbanBoard => {
            kanban::kanban(frame, size, state);
        }
        View::CreateTask => {
            create_task::create_task(frame, size, state);
        }
        View::EditTask => {
            edit_task::edit_task(frame, size, state);
        }
        View::MoveTaskSection => {
            // This view is handled as a modal overlay in ProjectTasks
            // Fallback: render kanban
            kanban::kanban(frame, size, state);
        }
    }

    // Render theme selector modal on top of everything (only on Welcome view)
    if state.has_theme_selector() && matches!(state.current_view(), View::Welcome) {
        render_theme_selector_modal(frame, size, state);
    }
}

fn welcome(frame: &mut Frame, size: Rect, state: &mut State) {
    welcome::render_welcome(frame, size, state);
}

fn render_delete_confirmation(frame: &mut Frame, size: Rect, task_name: &str, state: &State) {
    use ratatui::{
        layout::Alignment,
        style::{Modifier, Style},
        text::{Line, Span},
        widgets::{Block, Borders, Clear, Paragraph, Wrap},
    };

    // Create a centered popup dialog using ratatui pattern
    let popup_area = centered_rect(60, 25, size);

    // Clear the area first (ratatui modal pattern)
    frame.render_widget(Clear, popup_area);

    // Format the text - truncate long task names
    let display_name = if task_name.len() > 45 {
        format!("{}...", &task_name[..45])
    } else {
        task_name.to_string()
    };

    let theme = state.get_theme();
    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("Delete task: \"{}\"?", display_name),
            Style::default()
                .fg(theme.text.to_color())
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "This action cannot be undone.",
            Style::default()
                .fg(theme.warning.to_color())
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Enter: confirm, Esc: cancel",
            Style::default().fg(theme.text_muted.to_color()),
        )),
    ];

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Span::styled(
                    "⚠️  Confirm Delete",
                    Style::default()
                        .fg(theme.error.to_color())
                        .add_modifier(Modifier::BOLD),
                ))
                .border_style(
                    Style::default()
                        .fg(theme.error.to_color())
                        .add_modifier(Modifier::BOLD),
                ),
        )
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, popup_area);
}

fn render_move_task_modal(frame: &mut Frame, size: Rect, _task_name: &str, state: &State) {
    use crate::ui::widgets::styling;
    use ratatui::{
        layout::{Alignment, Constraint, Direction, Layout},
        style::{Modifier, Style},
        text::Span,
        widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    };

    // Create a centered popup dialog using ratatui pattern
    // Make it taller to show 5 items comfortably
    let popup_area = centered_rect(50, 40, size);

    // Clear the area first (ratatui modal pattern)
    frame.render_widget(Clear, popup_area);

    // Get sections and selected index
    let sections = state.get_sections();
    let selected_index = state.get_section_dropdown_index();

    // Split popup into title and list areas
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(7)])
        .split(popup_area);

    // Title block - just "Move"
    let theme = state.get_theme();
    let title_block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            "Move",
            Style::default()
                .fg(theme.info.to_color())
                .add_modifier(Modifier::BOLD),
        ))
        .border_style(styling::active_block_border_style(theme));

    let title_text = Paragraph::new("j/k: navigate, Enter: select, Esc: cancel")
        .block(title_block)
        .alignment(Alignment::Center);
    frame.render_widget(title_text, chunks[0]);

    // Limit visible sections to max 5 items (with scrolling)
    let max_visible = 5;
    let total_items = sections.len();
    let start_index = if total_items <= max_visible {
        0
    } else {
        (selected_index as i32 - max_visible as i32 / 2)
            .max(0)
            .min((total_items - max_visible) as i32) as usize
    };
    let end_index = (start_index + max_visible).min(total_items);
    let visible_sections = if sections.is_empty() {
        vec![]
    } else {
        sections[start_index..end_index].to_vec()
    };
    let visible_selected = selected_index.saturating_sub(start_index);

    // Create list items from visible sections
    let items: Vec<ListItem> = if visible_sections.is_empty() {
        vec![ListItem::new("No sections available")]
    } else {
        visible_sections
            .iter()
            .map(|section| ListItem::new(section.name.clone()))
            .collect()
    };

    // Use ListState for proper selection display
    let mut list_state = ratatui::widgets::ListState::default();
    if !items.is_empty() && !sections.is_empty() {
        let safe_index = visible_selected.min(items.len().saturating_sub(1));
        list_state.select(Some(safe_index));
    }

    // Create list block with section count
    let list_block = Block::default()
        .borders(Borders::ALL)
        .title(format!("Sections ({})", sections.len()))
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

fn render_theme_selector_modal(frame: &mut Frame, size: Rect, state: &State) {
    use crate::ui::widgets::styling;
    use ratatui::{
        layout::{Alignment, Constraint, Direction, Layout},
        style::{Modifier, Style},
        text::Span,
        widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    };

    // Create a centered popup dialog using ratatui pattern
    let popup_area = centered_rect(50, 50, size);

    // Clear the area first (ratatui modal pattern)
    frame.render_widget(Clear, popup_area);

    // Get available themes and selected index
    let available_themes = crate::ui::Theme::available_themes();
    let selected_index = state.get_theme_dropdown_index();

    // Split popup into title and list areas
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(7)])
        .split(popup_area);

    // Title block
    let theme = state.get_theme();
    let title_block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            "Select Theme",
            Style::default()
                .fg(theme.info.to_color())
                .add_modifier(Modifier::BOLD),
        ))
        .border_style(styling::active_block_border_style(theme));

    let title_text = Paragraph::new("j/k: navigate, Enter: select, Esc: cancel")
        .block(title_block)
        .alignment(Alignment::Center);
    frame.render_widget(title_text, chunks[0]);

    // Limit visible themes to max 8 items (with scrolling)
    let max_visible = 8;
    let total_items = available_themes.len();
    let start_index = if total_items <= max_visible {
        0
    } else {
        (selected_index as i32 - max_visible as i32 / 2)
            .max(0)
            .min((total_items - max_visible) as i32) as usize
    };
    let end_index = (start_index + max_visible).min(total_items);
    let visible_themes = if available_themes.is_empty() {
        vec![]
    } else {
        available_themes[start_index..end_index].to_vec()
    };
    let visible_selected = selected_index.saturating_sub(start_index);

    // Create list items from visible themes
    let items: Vec<ListItem> = if visible_themes.is_empty() {
        vec![ListItem::new("No themes available")]
    } else {
        visible_themes
            .iter()
            .map(|theme_name| {
                // Format theme name nicely (e.g., "tokyo-night" -> "Tokyo Night")
                let display_name = theme_name
                    .split('-')
                    .map(|word| {
                        let mut chars = word.chars();
                        match chars.next() {
                            None => String::new(),
                            Some(first) => {
                                first.to_uppercase().collect::<String>() + chars.as_str()
                            }
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ");

                // Show indicator if this is the current theme
                let current_indicator = if theme_name == &state.get_theme().name {
                    " (current)"
                } else {
                    ""
                };

                ListItem::new(format!("{}{}", display_name, current_indicator))
            })
            .collect()
    };

    // Use ListState for proper selection display
    let mut list_state = ratatui::widgets::ListState::default();
    if !items.is_empty() && !available_themes.is_empty() {
        let safe_index = visible_selected.min(items.len().saturating_sub(1));
        list_state.select(Some(safe_index));
    }

    // Create list block with theme count
    let list_block = Block::default()
        .borders(Borders::ALL)
        .title(format!("Themes ({})", available_themes.len()))
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

/// Helper function to create a centered rectangle (ratatui modal pattern)
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
