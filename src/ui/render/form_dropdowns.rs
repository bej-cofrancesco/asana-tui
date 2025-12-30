use super::Frame;
use crate::asana::CustomField;
use crate::state::State;
use crate::ui::widgets::styling;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

/// Generic dropdown renderer - used by assignee, section, and custom fields
fn render_dropdown_generic(
    frame: &mut Frame,
    area: Rect,
    search_text: &str,
    search_title: &str,
    items: Vec<ListItem>,
    selected_index: usize,
    dropdown_title: &str,
    state: &State,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search input (same size as original field)
            Constraint::Min(7), // Dropdown list (max 5 items + borders, but allow more if space available)
        ])
        .split(area);

    // Search input
    let theme = state.get_theme();
    let search_block = Block::default()
        .borders(Borders::ALL)
        .title(search_title)
        .border_style(styling::active_block_border_style(theme));
    let search_para = Paragraph::new(format!("> {}", search_text))
        .block(search_block)
        .style(styling::normal_text_style(theme));
    frame.render_widget(search_para, chunks[0]);

    // Calculate visible range (show max 5 items, centered around selected)
    let max_visible = 5;
    let total_items = items.len();
    let start_index = if total_items <= max_visible {
        0
    } else {
        (selected_index as i32 - max_visible as i32 / 2)
            .max(0)
            .min((total_items - max_visible) as i32) as usize
    };
    let end_index = (start_index + max_visible).min(total_items);
    let visible_items = &items[start_index..end_index];
    let visible_selected = selected_index.saturating_sub(start_index);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(dropdown_title)
        .border_style(styling::active_block_border_style(theme));

    // Use ListState for proper selection display
    let mut list_state = ratatui::widgets::ListState::default();
    if !visible_items.is_empty() {
        list_state.select(Some(
            visible_selected.min(visible_items.len().saturating_sub(1)),
        ));
    }

    let list = List::new(visible_items.iter().cloned())
        .block(block)
        .style(styling::normal_text_style(theme))
        .highlight_style(
            Style::default()
                .fg(theme.highlight_fg.to_color())
                .bg(theme.highlight_bg.to_color())
                .add_modifier(Modifier::BOLD),
        );

    frame.render_stateful_widget(list, chunks[1], &mut list_state);
}

/// Render assignee dropdown with search and filtered list.
/// Uses the same component as custom field dropdowns.
pub fn render_assignee_dropdown(frame: &mut Frame, area: Rect, state: &State) {
    let search_text = state.get_assignee_search();
    let filtered: Vec<_> = state
        .get_workspace_users()
        .iter()
        .filter(|user| {
            search_text.is_empty()
                || user
                    .name
                    .to_lowercase()
                    .contains(&search_text.to_lowercase())
                || user
                    .email
                    .to_lowercase()
                    .contains(&search_text.to_lowercase())
        })
        .collect();
    let selected_index = state.get_assignee_dropdown_index();

    let items: Vec<ListItem> = filtered
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

    render_dropdown_generic(
        frame,
        area,
        &search_text,
        "Search Assignee",
        items,
        selected_index,
        &format!(
            "Assignee ({} results, ↑ / ↓ arrow to navigate, Enter to select)",
            filtered.len()
        ),
        state,
    );
}

/// Render section dropdown with search and filtered list.
/// Uses the same component as custom field dropdowns.
pub fn render_section_dropdown(frame: &mut Frame, area: Rect, state: &State) {
    let search_text = state.get_section_search();
    let filtered: Vec<_> = state
        .get_sections()
        .iter()
        .filter(|section| {
            search_text.is_empty()
                || section
                    .name
                    .to_lowercase()
                    .contains(&search_text.to_lowercase())
        })
        .collect();
    let selected_index = state.get_section_dropdown_index();

    let items: Vec<ListItem> = filtered
        .iter()
        .map(|section| ListItem::new(section.name.clone()))
        .collect();

    render_dropdown_generic(
        frame,
        area,
        &search_text,
        "Search Section",
        items,
        selected_index,
        &format!(
            "Section ({} results, ↑ / ↓ arrow to navigate, Enter to select)",
            filtered.len()
        ),
        state,
    );
}

/// Render enum dropdown for custom fields.
/// Uses the same component as assignee and section dropdowns.
pub fn render_enum_dropdown(
    frame: &mut Frame,
    area: Rect,
    cf: &CustomField,
    cf_gid: &str,
    state: &State,
) {
    let search_text = state.get_custom_field_search(cf_gid);
    let filtered: Vec<_> = cf
        .enum_options
        .iter()
        .filter(|eo| {
            eo.enabled
                && (search_text.is_empty()
                    || eo.name.to_lowercase().contains(&search_text.to_lowercase()))
        })
        .collect();
    let selected_index = state.get_custom_field_dropdown_index(cf_gid);

    let items: Vec<ListItem> = filtered
        .iter()
        .map(|eo| ListItem::new(eo.name.clone()))
        .collect();

    render_dropdown_generic(
        frame,
        area,
        &search_text,
        &format!("Search {}", cf.name),
        items,
        selected_index,
        &format!(
            "{} ({} results, ↑ / ↓ arrow to navigate, Enter to select)",
            cf.name,
            filtered.len()
        ),
        state,
    );
}
