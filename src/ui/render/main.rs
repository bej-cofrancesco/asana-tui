use super::welcome;
use super::{create_task, edit_task, kanban, task_detail, Frame};
use crate::state::{Focus, State, View};
use crate::ui::widgets::styling;
use tui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Span, Spans, Text},
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
            // Check view mode - show list or kanban
            if state.get_view_mode() == crate::state::ViewMode::Kanban {
                kanban::kanban(frame, size, state);
            } else {
                project_tasks(frame, size, state);
            }
        }
        View::TaskDetail => {
            // Check if we need to show delete confirmation dialog
            if state.has_delete_confirmation() {
                let task_name = state
                    .get_task_detail()
                    .map(|t| t.name.as_str())
                    .unwrap_or("this task");
                render_delete_confirmation(frame, size, task_name);
                return;
            }
            task_detail::task_detail(frame, size, state);
        }
        View::KanbanBoard => {
            kanban::kanban(frame, size, state);
        }
        View::CreateTask => {
            create_task::create_task(frame, size, state);
        }
        View::EditTask => {
            edit_task::edit_task(frame, size, state);
        }
    }
}

fn welcome(frame: &mut Frame, size: Rect, state: &mut State) {
    welcome::render_welcome(frame, size, state);
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
    let project_name = state
        .get_project()
        .map(|p| p.name.to_owned())
        .unwrap_or_else(|| "Project".to_string());

    // Add filter indicator
    let filter_text = match state.get_task_filter() {
        crate::state::TaskFilter::All => " [All]".to_string(),
        crate::state::TaskFilter::Incomplete => " [Incomplete]".to_string(),
        crate::state::TaskFilter::Completed => " [Completed]".to_string(),
        crate::state::TaskFilter::Assignee(None) => " [Unassigned]".to_string(),
        crate::state::TaskFilter::Assignee(Some(gid)) => {
            // Find assignee name from workspace users
            let assignee_name = state
                .get_workspace_users()
                .iter()
                .find(|u| u.gid == gid)
                .map(|u| u.name.as_str())
                .unwrap_or("Unknown");
            format!(" [Assignee: {}]", assignee_name)
        }
    };
    
    let mut title = format!("{}{}", project_name, filter_text);

    // Show search in title if we're searching tasks (show "/" even if query is empty)
    if state.is_search_mode()
        && matches!(
            state.get_search_target(),
            Some(crate::state::SearchTarget::Tasks)
        )
    {
        title = format!("{} /{}", title, state.get_search_query());
    } else if !state.get_search_query().is_empty()
        && matches!(
            state.get_search_target(),
            Some(crate::state::SearchTarget::Tasks)
        )
    {
        // Show query even if not in search mode (after exiting search)
        title = format!("{} /{}", title, state.get_search_query());
    }

    // Check if we need to show delete confirmation dialog
    if state.has_delete_confirmation() {
        // Get the actual task name from filtered tasks
        let filtered = state.get_filtered_tasks();
        let task_name = if let Some(selected_index) = state.get_tasks_list_state().selected() {
            if selected_index < filtered.len() {
                filtered[selected_index].name.as_str()
            } else {
                "this task"
            }
        } else {
            "this task"
        };

        // Render confirmation dialog
        render_delete_confirmation(frame, size, task_name);
        return;
    }

    let block = view_block(&title, state);
    let tasks = state.get_filtered_tasks().to_vec();
    
    // Check if we have a search query and tasks are empty - show "No results" instead of "Loading..."
    let has_search_query = !state.get_search_query().is_empty()
        && matches!(state.get_search_target(), Some(crate::state::SearchTarget::Tasks));
    let has_loaded_tasks = !state.get_tasks().is_empty(); // We have some tasks loaded (even if filtered out)
    
    let list = if tasks.is_empty() && has_search_query && has_loaded_tasks {
        // Empty search results - show "No results found"
        tui::widgets::List::new(vec![tui::widgets::ListItem::new("No results found")])
            .block(block)
    } else {
        task_list(&tasks).block(block)
    };
    
    frame.render_stateful_widget(list, size, state.get_tasks_list_state());
}

fn render_delete_confirmation(frame: &mut Frame, size: Rect, task_name: &str) {
    use crate::ui::widgets::styling;
    use tui::{
        layout::Alignment,
        style::{Color, Modifier, Style},
        text::{Span, Spans},
        widgets::{Block, Borders, Clear, Paragraph, Wrap},
    };

    // Create a centered popup dialog
    let dialog_width = 60.min(size.width.saturating_sub(4));
    let dialog_height = 7;
    
    // Center horizontally and vertically - use proper centering formula
    let x = if size.width > dialog_width {
        (size.width - dialog_width) / 2
    } else {
        0
    };
    let y = if size.height > dialog_height {
        (size.height - dialog_height) / 2
    } else {
        0
    };

    let dialog_rect = Rect {
        x,
        y,
        width: dialog_width,
        height: dialog_height,
    };

    // Clear the area first - this removes the grey background!
    frame.render_widget(Clear, dialog_rect);

    // Format the text - truncate long task names
    let display_name = if task_name.len() > 45 {
        format!("{}...", &task_name[..45])
    } else {
        task_name.to_string()
    };

    let text = vec![
        Spans::from(""),
        Spans::from(Span::styled(
            format!("Delete task: \"{}\"?", display_name),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )),
        Spans::from(""),
        Spans::from(vec![
            Span::styled("Press ", Style::default().fg(Color::Gray)),
            Span::styled("Enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled(" to confirm, ", Style::default().fg(Color::Gray)),
            Span::styled("Esc", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" to cancel", Style::default().fg(Color::Gray)),
        ]),
    ];

    // Create a prominent popup with warning styling and solid black background
    let title = Spans::from(vec![Span::styled(
        " ⚠  Confirm Delete ",
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
    )]);
    
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
        .style(Style::default().bg(Color::Black));

    let paragraph = Paragraph::new(text)
        .block(block)
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Center);

    frame.render_widget(paragraph, dialog_rect);
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
