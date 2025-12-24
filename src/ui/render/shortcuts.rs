use super::widgets::spinner;
use super::Frame;
use crate::state::{Focus, Menu, State};
use crate::ui::widgets::styling;
use tui::{
    layout::Rect,
    text::Span,
    widgets::{Block, Borders, List, ListItem},
};

const BLOCK_TITLE: &str = "Shortcuts";

/// Render shortcuts widget according to state.
///
pub fn shortcuts(frame: &mut Frame, size: Rect, state: &mut State) {
    let mut block = Block::default()
        .title(BLOCK_TITLE)
        .borders(Borders::ALL)
        .border_style(styling::normal_block_border_style());

    let mut list_item_style = styling::current_list_item_style();
    if *state.current_focus() == Focus::Menu && *state.current_menu() == Menu::Shortcuts {
        block = block
            .border_style(styling::active_block_border_style())
            .title(Span::styled(
                BLOCK_TITLE,
                styling::active_block_title_style(),
            ));
        list_item_style = styling::active_list_item_style();
    }

    // Wait for projects to load before showing starred projects in shortcuts
    // Show spinner if projects haven't loaded yet and we have starred projects from config
    let has_starred_projects = !state.get_starred_project_gids().is_empty();
    let projects_loaded = !state.get_projects().is_empty();
    
    // Only show spinner if we have starred projects from config but projects haven't loaded yet
    // Once projects load, always show shortcuts (even if empty)
    if has_starred_projects && !projects_loaded {
        // Show spinner while waiting for projects to load
        frame.render_widget(spinner::widget(state, size.height).block(block), size);
        return;
    }

    // Get all shortcuts (starred projects first, then base shortcuts)
    let all_shortcuts = state.get_all_shortcuts_with_update();
    
    let items: Vec<ListItem> = all_shortcuts
        .iter()
        .map(|s| ListItem::new(s.to_owned()))
        .collect();

    let list = List::new(items)
        .style(styling::normal_text_style())
        .highlight_style(list_item_style)
        .block(block);
    
    frame.render_stateful_widget(list, size, state.get_shortcuts_list_state());
}
