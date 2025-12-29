use super::Frame;
use crate::state::State;
use crate::ui::widgets::styling;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

/// Render assignee dropdown with search and filtered list.
/// Renders relative to the provided area - search bar same size, dropdown below.
/// This is shared between create_task and edit_task.
pub fn render_assignee_dropdown(frame: &mut Frame, area: Rect, state: &State) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search input (same size as original field)
            Constraint::Min(7), // Dropdown list (max 5 items + borders, but allow more if space available)
        ])
        .split(area);

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

    // Filtered users list - limit to max 5 visible items
    let filtered_users = state.get_filtered_assignees();
    let selected_index = state.get_assignee_dropdown_index();
    
    // Calculate visible range (show max 5 items, centered around selected)
    let max_visible = 5;
    let total_items = filtered_users.len();
    let start_index = if total_items <= max_visible {
        0
    } else {
        (selected_index as i32 - max_visible as i32 / 2)
            .max(0)
            .min((total_items - max_visible) as i32) as usize
    };
    let end_index = (start_index + max_visible).min(total_items);
    let visible_users = &filtered_users[start_index..end_index];
    let visible_selected = selected_index.saturating_sub(start_index);

    let items: Vec<ListItem> = visible_users
        .iter()
        .map(|user| {
            let display_text = if !user.email.is_empty() {
                format!("{} ({})", user.name, user.email)
            } else {
                user.name.clone()
            };
            ListItem::new(display_text)
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
    let mut list_state = ratatui::widgets::ListState::default();
    if !items.is_empty() {
        list_state.select(Some(visible_selected.min(items.len().saturating_sub(1))));
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

/// Render section dropdown with search and filtered list.
/// Renders relative to the provided area - search bar same size, dropdown below.
/// This is shared between create_task and edit_task.
pub fn render_section_dropdown(frame: &mut Frame, area: Rect, state: &State) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search input (same size as original field)
            Constraint::Min(7), // Dropdown list (max 5 items + borders, but allow more if space available)
        ])
        .split(area);

    // Search input
    let search_text = state.get_section_search();
    let search_block = Block::default()
        .borders(Borders::ALL)
        .title("Search Section")
        .border_style(styling::active_block_border_style());
    let search_para = Paragraph::new(format!("> {}", search_text))
        .block(search_block)
        .style(styling::normal_text_style());
    frame.render_widget(search_para, chunks[0]);

    // Filtered sections list - limit to max 5 visible items
    let filtered_sections = state.get_filtered_sections();
    let selected_index = state.get_section_dropdown_index();
    
    // Calculate visible range (show max 5 items, centered around selected)
    let max_visible = 5;
    let total_items = filtered_sections.len();
    let start_index = if total_items <= max_visible {
        0
    } else {
        (selected_index as i32 - max_visible as i32 / 2)
            .max(0)
            .min((total_items - max_visible) as i32) as usize
    };
    let end_index = (start_index + max_visible).min(total_items);
    let visible_sections = &filtered_sections[start_index..end_index];
    let visible_selected = selected_index.saturating_sub(start_index);

    let items: Vec<ListItem> = visible_sections
        .iter()
        .map(|section| ListItem::new(section.name.clone()))
        .collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(
            "Section ({} results, j/k to navigate, Enter to select)",
            filtered_sections.len()
        ))
        .border_style(styling::active_block_border_style());

    // Use ListState for proper selection display
    let mut list_state = ratatui::widgets::ListState::default();
    if !items.is_empty() {
        list_state.select(Some(visible_selected.min(items.len().saturating_sub(1))));
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
