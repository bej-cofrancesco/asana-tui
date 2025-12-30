use super::Frame;
use crate::state::State;
use crate::ui::widgets::styling;
use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
};

/// Render log widget according to state.
///
pub fn log(frame: &mut Frame, size: Rect, state: &mut State) {
    let title = if state.is_debug_mode() {
        "Log (DEBUG MODE: j/k: navigate, y: copy, / or Esc: exit)"
    } else {
        "Logs"
    };

    let block = Block::default().title(title).borders(Borders::ALL);

    // If in debug mode, show list with selection
    if state.is_debug_mode() {
        let theme = state.get_theme();
        let debug_entries = state.get_debug_entries();
        let items: Vec<ListItem> = debug_entries
            .iter()
            .enumerate()
            .map(|(i, entry)| {
                let style = if i == state.get_debug_index() {
                    styling::active_list_item_style(theme)
                } else {
                    styling::normal_text_style(theme)
                };
                ListItem::new(Line::from(vec![Span::styled(entry.clone(), style)]))
            })
            .collect();

        let list = List::new(items)
            .style(styling::normal_text_style(theme))
            .highlight_style(styling::active_list_item_style(theme))
            .block(block);

        // Create a dummy ListState for rendering
        let mut list_state = ratatui::widgets::ListState::default();
        list_state.select(Some(state.get_debug_index()));
        frame.render_stateful_widget(list, size, &mut list_state);
    } else {
        // Normal mode: show logs from state with auto-scroll to bottom
        // Use stateful widget to control scroll position
        let theme = state.get_theme();
        let debug_entries = state.get_debug_entries();
        let items: Vec<ListItem> = debug_entries
            .iter()
            .map(|entry| {
                ListItem::new(Line::from(vec![Span::styled(
                    entry.clone(),
                    styling::normal_text_style(theme),
                )]))
            })
            .collect();

        let list = List::new(items)
            .style(styling::normal_text_style(theme))
            .block(block);

        // Use stateful widget to control scroll position
        // Set selection to the last item (bottom) to auto-scroll
        let mut list_state = ratatui::widgets::ListState::default();
        if !debug_entries.is_empty() {
            list_state.select(Some(debug_entries.len() - 1));
        }
        frame.render_stateful_widget(list, size, &mut list_state);
    }
}
