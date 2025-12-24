use super::widgets::spinner;
use super::Frame;
use crate::state::{Focus, Menu, State};
use crate::ui::widgets::styling;
use tui::{
    layout::Rect,
    text::Span,
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

    if state.get_projects().is_empty() {
        frame.render_widget(spinner::widget(state, size.height).block(block), size);
        return;
    }

    let items: Vec<ListItem> = state
        .get_projects()
        .iter()
        .map(|p| {
            let display_name = if state.is_project_starred(&p.gid) {
                format!("⭐ {}", p.name)
            } else {
                p.name.to_owned()
            };
            ListItem::new(display_name)
        })
        .collect();
    
    let list = List::new(items)
        .style(styling::normal_text_style())
        .highlight_style(list_item_style)
        .block(block);
    
    frame.render_stateful_widget(list, size, state.get_projects_list_state());
}
