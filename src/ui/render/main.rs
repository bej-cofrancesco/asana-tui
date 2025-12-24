use super::welcome::{BANNER, CONTENT};
use super::Frame;
use crate::state::{Focus, State, View};
use crate::ui::widgets::styling;
use tui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Span, Text},
    widgets::{Block, Borders, Paragraph},
};

/// Render main widget according to state.
///
pub fn main(frame: &mut Frame, size: Rect, state: &mut State) {
    match state.current_view() {
        View::Welcome => {
            welcome(frame, size, state);
        }
        View::MyTasks => {
            my_tasks(frame, size, state);
        }
        View::RecentlyModified => {
            recently_modified(frame, size, state);
        }
        View::RecentlyCompleted => {
            recently_completed(frame, size, state);
        }
        View::ProjectTasks => {
            project_tasks(frame, size, state);
        }
    }
}

fn welcome(frame: &mut Frame, size: Rect, state: &mut State) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(6), Constraint::Length(94)].as_ref())
        .margin(2)
        .split(size);

    let block = view_block("Welcome", state);
    frame.render_widget(block, size);

    let mut banner = Text::from(BANNER);
    banner.patch_style(styling::banner_style());
    let banner_widget = Paragraph::new(banner);
    frame.render_widget(banner_widget, rows[0]);

    let mut content = Text::from(CONTENT);
    content.patch_style(styling::normal_text_style());
    let content_widget = Paragraph::new(content);
    frame.render_widget(content_widget, rows[1]);
}

fn my_tasks(frame: &mut Frame, size: Rect, state: &mut State) {
    let block = view_block("My Tasks", state);
    let tasks = state.get_tasks().clone();
    let list = task_list(&tasks).block(block);
    frame.render_stateful_widget(list, size, state.get_tasks_list_state());
}

fn recently_modified(frame: &mut Frame, size: Rect, state: &mut State) {
    let block = view_block("Recently Modified", state);
    let tasks = state.get_tasks().clone();
    let list = task_list(&tasks).block(block);
    frame.render_stateful_widget(list, size, state.get_tasks_list_state());
}

fn recently_completed(frame: &mut Frame, size: Rect, state: &mut State) {
    let block = view_block("Recently Completed", state);
    let tasks = state.get_tasks().clone();
    let list = task_list(&tasks).block(block);
    frame.render_stateful_widget(list, size, state.get_tasks_list_state());
}

fn project_tasks(frame: &mut Frame, size: Rect, state: &mut State) {
    let mut title = state.get_project()
        .map(|p| p.name.to_owned())
        .unwrap_or_else(|| "Project".to_string());
    // Show search in title if we're searching tasks (show "/" even if query is empty)
    if state.is_search_mode()
        && matches!(state.get_search_target(), Some(crate::state::SearchTarget::Tasks)) {
        title = format!("{} /{}", title, state.get_search_query());
    } else if !state.get_search_query().is_empty()
        && matches!(state.get_search_target(), Some(crate::state::SearchTarget::Tasks)) {
        // Show query even if not in search mode (after exiting search)
        title = format!("{} /{}", title, state.get_search_query());
    }
    let block = view_block(&title, state);
    let tasks = state.get_filtered_tasks().to_vec();
    let list = task_list(&tasks).block(block);
    frame.render_stateful_widget(list, size, state.get_tasks_list_state());
}

fn task_list(tasks: &[crate::asana::Task]) -> tui::widgets::List<'_> {
    if tasks.is_empty() {
        return tui::widgets::List::new(vec![tui::widgets::ListItem::new("Loading...")]);
    }
    let items: Vec<tui::widgets::ListItem> = tasks
        .iter()
        .map(|t| tui::widgets::ListItem::new(t.name.to_owned()))
        .collect();
    let list = tui::widgets::List::new(items)
        .block(tui::widgets::Block::default().borders(tui::widgets::Borders::NONE))
        .style(styling::normal_text_style())
        .highlight_style(styling::active_list_item_style());
    list
}

fn view_block<'a>(title: &'a str, state: &mut State) -> Block<'a> {
    Block::default()
        .borders(Borders::ALL)
        .border_style(match *state.current_focus() {
            Focus::View => styling::active_block_border_style(),
            _ => styling::normal_block_border_style(),
        })
        .title(Span::styled(title, styling::active_block_title_style()))
}
