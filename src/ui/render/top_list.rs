use super::widgets::spinner;
use super::Frame;
use crate::state::{Focus, Menu, State};
use crate::ui::widgets::styling;
use ratatui::{
    layout::Rect,
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
};

const BLOCK_TITLE: &str = "Projects";

/// Render top list widget according to state.
///
pub fn top_list(frame: &mut Frame, size: Rect, state: &mut State) {
    let filtered_projects = state.get_filtered_projects();

    // Show search in title if we're searching projects (show "/" even if query is empty)
    let title_text = if state.is_search_mode()
        && matches!(
            state.get_search_target(),
            Some(crate::state::SearchTarget::Projects)
        ) {
        format!("{} /{}", BLOCK_TITLE, state.get_search_query())
    } else if !state.get_search_query().is_empty()
        && matches!(
            state.get_search_target(),
            Some(crate::state::SearchTarget::Projects)
        )
    {
        // Show query even if not in search mode (after exiting search)
        format!("{} /{}", BLOCK_TITLE, state.get_search_query())
    } else {
        BLOCK_TITLE.to_string()
    };

    let theme = state.get_theme();
    let mut block = Block::default()
        .borders(Borders::ALL)
        .border_style(styling::normal_block_border_style(theme));

    let list_item_style;
    if *state.current_focus() == Focus::Menu && *state.current_menu() == Menu::TopList {
        list_item_style = styling::active_list_item_style(theme);
        block = block
            .border_style(styling::active_block_border_style(theme))
            .title(Span::styled(
                title_text.clone(),
                styling::active_block_title_style(),
            ));
    } else {
        list_item_style = styling::current_list_item_style(theme);
        block = block.title(title_text);
    }

    // Show spinner only if we're not searching and have no projects loaded yet
    if filtered_projects.is_empty() && !state.is_search_mode() && state.get_projects().is_empty() {
        frame.render_widget(spinner::widget(state, size.height).block(block), size);
        return;
    }

    // Check if we have a search query and projects are empty - show "No results" instead of spinner
    let has_search_query = !state.get_search_query().is_empty()
        && matches!(
            state.get_search_target(),
            Some(crate::state::SearchTarget::Projects)
        );
    let has_loaded_projects = !state.get_projects().is_empty(); // We have some projects loaded (even if filtered out)

    let items: Vec<ListItem> =
        if filtered_projects.is_empty() && has_search_query && has_loaded_projects {
            // Empty search results - show "No results found"
            vec![ListItem::new("No results found")]
        } else {
            filtered_projects
                .iter()
                .map(|p| {
                    // Make starred projects italic
                    if state.is_project_starred(&p.gid) {
                        ListItem::new(Line::from(vec![Span::styled(
                            p.name.to_owned(),
                            styling::normal_text_style(theme).add_modifier(Modifier::ITALIC),
                        )]))
                    } else {
                        ListItem::new(p.name.to_owned())
                    }
                })
                .collect()
        };

    let list = List::new(items)
        .style(styling::normal_text_style(theme))
        .highlight_style(list_item_style)
        .block(block);

    frame.render_stateful_widget(list, size, state.get_projects_list_state());
}
