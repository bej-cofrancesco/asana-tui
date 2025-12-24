use super::Frame;
use crate::state::State;
use crate::ui::widgets::styling;
use tui::{
    layout::Rect,
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem},
};

/// Render log widget according to state.
///
pub fn log(frame: &mut Frame, size: Rect, state: &mut State) {
    let title = if state.is_debug_mode() {
        "Log (DEBUG MODE: j/k: navigate, y: copy, / or Esc: exit)"
    } else {
        "Log (Press d to enter debug mode)"
    };

    let block = Block::default().title(title).borders(Borders::ALL);

    // If in debug mode, show list with selection
    if state.is_debug_mode() {
        let debug_entries = state.get_debug_entries();
        let items: Vec<ListItem> = debug_entries
            .iter()
            .enumerate()
            .map(|(i, entry)| {
                let style = if i == state.get_debug_index() {
                    styling::active_list_item_style()
                } else {
                    styling::normal_text_style()
                };
                ListItem::new(Spans::from(vec![Span::styled(entry.clone(), style)]))
            })
            .collect();

        let list = List::new(items)
            .style(styling::normal_text_style())
            .highlight_style(styling::active_list_item_style())
            .block(block);

        // Create a dummy ListState for rendering
        let mut list_state = tui::widgets::ListState::default();
        list_state.select(Some(state.get_debug_index()));
        frame.render_stateful_widget(list, size, &mut list_state);
    } else {
        // Normal mode: show logs from state (same as debug mode but without selection)
        // This way logs are always available in both modes
        let debug_entries = state.get_debug_entries();
        let items: Vec<ListItem> = debug_entries
            .iter()
            .map(|entry| {
                ListItem::new(Spans::from(vec![Span::styled(
                    entry.clone(),
                    styling::normal_text_style(),
                )]))
            })
            .collect();

        let list = List::new(items)
            .style(styling::normal_text_style())
            .block(block);

        frame.render_widget(list, size);
    }
}
