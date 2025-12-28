use super::Frame;
use crate::state::State;
use crate::ui::widgets::styling;
use tui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

/// Render assignee dropdown with search and filtered list.
/// This is shared between create_task and edit_task.
pub fn render_assignee_dropdown(frame: &mut Frame, size: Rect, state: &State) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search input
            Constraint::Min(1),    // List
        ])
        .split(size);

    // Search input
    let search_text = state.get_assignee_search();
    let search_block = Block::default()
        .borders(Borders::ALL)
        .title("Search Assignee")
        .border_style(styling::active_block_border_style());
    let search_para = Paragraph::new(format!("> {}", search_text))
        .block(search_block)
        .style(styling::normal_text_style());
    frame.render_widget(search_para, chunks[0]);

    // Filtered users list
    let filtered_users = state.get_filtered_assignees();
    let selected_index = state.get_assignee_dropdown_index();

    let items: Vec<ListItem> = filtered_users
        .iter()
        .enumerate()
        .map(|(i, user)| {
            let style = if i == selected_index {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                styling::normal_text_style()
            };
            let display_text = if !user.email.is_empty() {
                format!("{} ({})", user.name, user.email)
            } else {
                user.name.clone()
            };
            ListItem::new(Spans::from(Span::styled(display_text, style)))
        })
        .collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(
            "Assignee ({} results, j/k to navigate, Enter to select)",
            filtered_users.len()
        ))
        .border_style(styling::active_block_border_style());

    // Use ListState for proper selection display
    let mut list_state = tui::widgets::ListState::default();
    if !items.is_empty() {
        list_state.select(Some(selected_index.min(items.len().saturating_sub(1))));
    }

    let list = List::new(items)
        .block(block)
        .style(styling::normal_text_style())
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_stateful_widget(list, chunks[1], &mut list_state);
}

/// Render section dropdown with list.
/// This is shared between create_task and edit_task.
pub fn render_section_dropdown(frame: &mut Frame, size: Rect, state: &State) {
    let sections = state.get_sections();
    let selected_index = state.get_section_dropdown_index();

    let items: Vec<ListItem> = sections
        .iter()
        .enumerate()
        .map(|(i, section)| {
            let style = if i == selected_index {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                styling::normal_text_style()
            };
            ListItem::new(Spans::from(Span::styled(&section.name, style)))
        })
        .collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(
            "Section ({} results, j/k to navigate, Enter to select)",
            sections.len()
        ))
        .border_style(styling::active_block_border_style());

    // Use ListState for proper selection display
    let mut list_state = tui::widgets::ListState::default();
    if !items.is_empty() {
        list_state.select(Some(selected_index.min(items.len().saturating_sub(1))));
    }

    let list = List::new(items)
        .block(block)
        .style(styling::normal_text_style())
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_stateful_widget(list, size, &mut list_state);
}
