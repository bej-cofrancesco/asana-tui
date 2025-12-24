use super::widgets::spinner;
use super::Frame;
use crate::state::{Focus, Menu, State};
use crate::ui::widgets::styling;
use tui::{
    layout::Rect,
    style::Modifier,
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem},
};

const BLOCK_TITLE: &str = "Projects";

/// Render top list widget according to state.
///
pub fn top_list(frame: &mut Frame, size: Rect, state: &mut State) {
    let mut block = Block::default()
        .title(BLOCK_TITLE)
        .borders(Borders::ALL)
        .border_style(styling::normal_block_border_style());

    let list_item_style;
    if *state.current_focus() == Focus::Menu && *state.current_menu() == Menu::TopList {
        list_item_style = styling::active_list_item_style();
        block = block
            .border_style(styling::active_block_border_style())
            .title(Span::styled(
                BLOCK_TITLE,
                styling::active_block_title_style(),
            ));
    } else {
        list_item_style = styling::current_list_item_style();
    }

    let filtered_projects = state.get_filtered_projects();
    if filtered_projects.is_empty() && !state.is_search_mode() {
        frame.render_widget(spinner::widget(state, size.height).block(block), size);
        return;
    }

    // Show search in title if we're searching projects (show "/" even if query is empty)
    let title = if state.is_search_mode()
        && matches!(state.get_search_target(), Some(crate::state::SearchTarget::Projects)) {
        format!("{} /{}", BLOCK_TITLE, state.get_search_query())
    } else if !state.get_search_query().is_empty()
        && matches!(state.get_search_target(), Some(crate::state::SearchTarget::Projects)) {
        // Show query even if not in search mode (after exiting search)
        format!("{} /{}", BLOCK_TITLE, state.get_search_query())
    } else {
        BLOCK_TITLE.to_string()
    };
    block = block.title(title);

    let items: Vec<ListItem> = filtered_projects
        .iter()
        .map(|p| {
            // Make starred projects italic
            if state.is_project_starred(&p.gid) {
                ListItem::new(Spans::from(vec![
                    Span::styled(
                        p.name.to_owned(),
                        styling::normal_text_style().add_modifier(Modifier::ITALIC),
                    )
                ]))
            } else {
                ListItem::new(p.name.to_owned())
            }
        })
        .collect();
    
    let list = List::new(items)
        .style(styling::normal_text_style())
        .highlight_style(list_item_style)
        .block(block);
    
    frame.render_stateful_widget(list, size, state.get_projects_list_state());
}
