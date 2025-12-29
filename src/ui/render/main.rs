use super::welcome;
use super::{create_task, edit_task, kanban, task_detail, Frame};
use crate::state::{Focus, State, View};
use crate::ui::widgets::styling;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::Span,
    widgets::{Block, Borders},
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

            // Check if we need to show delete confirmation dialog (render on top)
            if state.has_delete_confirmation() {
                let task_name = state
                    .get_filtered_tasks()
                    .get(state.get_tasks_list_state().selected().unwrap_or(0))
                    .map(|t| t.name.clone())
                    .unwrap_or_else(|| "this task".to_string());
                render_delete_confirmation(frame, size, &task_name);
            }
        }
        View::RecentlyModified => {
            recently_modified(frame, size, state);

            // Check if we need to show delete confirmation dialog (render on top)
            if state.has_delete_confirmation() {
                let task_name = state
                    .get_filtered_tasks()
                    .get(state.get_tasks_list_state().selected().unwrap_or(0))
                    .map(|t| t.name.clone())
                    .unwrap_or_else(|| "this task".to_string());
                render_delete_confirmation(frame, size, &task_name);
            }
        }
        View::RecentlyCompleted => {
            recently_completed(frame, size, state);

            // Check if we need to show delete confirmation dialog (render on top)
            if state.has_delete_confirmation() {
                let task_name = state
                    .get_filtered_tasks()
                    .get(state.get_tasks_list_state().selected().unwrap_or(0))
                    .map(|t| t.name.clone())
                    .unwrap_or_else(|| "this task".to_string());
                render_delete_confirmation(frame, size, &task_name);
            }
        }
        View::ProjectTasks => {
            // Always show kanban view first (so modal appears on top)
            kanban::kanban(frame, size, state);

            // Check if we need to show move task section selection modal (render on top)
            if state.has_move_task() {
                // Get task name for display before borrowing state mutably
                let task_name = state
                    .get_kanban_selected_task()
                    .map(|t| t.name.clone())
                    .unwrap_or_else(|| "task".to_string());
                render_move_task_modal(frame, size, &task_name, state);
            }

            // Check if we need to show delete confirmation dialog (render on top of everything)
            if state.has_delete_confirmation() {
                let task_name = state
                    .get_kanban_selected_task()
                    .map(|t| t.name.clone())
                    .unwrap_or_else(|| "this task".to_string());
                render_delete_confirmation(frame, size, &task_name);
            }
        }
        View::TaskDetail => {
            task_detail::task_detail(frame, size, state);

            // Check if we need to show delete confirmation dialog (render on top)
            if state.has_delete_confirmation() {
                let task_name = state
                    .get_task_detail()
                    .map(|t| t.name.clone())
                    .unwrap_or_else(|| "this task".to_string());
                render_delete_confirmation(frame, size, &task_name);
            }
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
        View::MoveTaskSection => {
            // This view is handled as a modal overlay in ProjectTasks
            // Fallback: render kanban
            kanban::kanban(frame, size, state);
        }
    }
}

fn welcome(frame: &mut Frame, size: Rect, state: &mut State) {
    welcome::render_welcome(frame, size, state);
}

fn my_tasks(frame: &mut Frame, size: Rect, state: &mut State) {
    let block = view_block("My Tasks", state);
    let tasks = state.get_filtered_tasks().to_vec();

    // Check if we have a search query and tasks are empty - show "No results" instead of "Loading..."
    let has_search_query = !state.get_search_query().is_empty()
        && matches!(
            state.get_search_target(),
            Some(crate::state::SearchTarget::Tasks)
        );
    let has_loaded_tasks = !state.get_tasks().is_empty(); // We have some tasks loaded (even if filtered out)

    let list = if tasks.is_empty() && has_search_query && has_loaded_tasks {
        // Empty search results - show "No results found"
        ratatui::widgets::List::new(vec![ratatui::widgets::ListItem::new("No results found")])
            .block(block)
    } else {
        task_list(&tasks).block(block)
    };

    frame.render_stateful_widget(list, size, state.get_tasks_list_state());
}

fn recently_modified(frame: &mut Frame, size: Rect, state: &mut State) {
    let block = view_block("Recently Modified", state);
    let tasks = state.get_filtered_tasks().to_vec();
    let list = task_list(&tasks).block(block);
    frame.render_stateful_widget(list, size, state.get_tasks_list_state());
}

fn recently_completed(frame: &mut Frame, size: Rect, state: &mut State) {
    let block = view_block("Recently Completed", state);
    let tasks = state.get_filtered_tasks().to_vec();
    let list = task_list(&tasks).block(block);
    frame.render_stateful_widget(list, size, state.get_tasks_list_state());
}

fn task_list(tasks: &[crate::asana::Task]) -> ratatui::widgets::List {
    if tasks.is_empty() {
        return ratatui::widgets::List::new(vec![ratatui::widgets::ListItem::new("Loading...")]);
    }
    let items: Vec<ratatui::widgets::ListItem> = tasks
        .iter()
        .map(|t| ratatui::widgets::ListItem::new(t.name.to_owned()))
        .collect();
    let list = ratatui::widgets::List::new(items)
        .block(ratatui::widgets::Block::default().borders(ratatui::widgets::Borders::NONE))
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

fn render_delete_confirmation(frame: &mut Frame, size: Rect, task_name: &str) {
    use ratatui::{
        layout::{Alignment, Constraint, Direction, Layout},
        style::{Color, Modifier, Style},
        text::{Line, Span},
        widgets::{Block, Borders, Clear, Paragraph, Wrap},
    };

    // Create a centered popup dialog using ratatui pattern
    let popup_area = centered_rect(60, 25, size);

    // Clear the area first (ratatui modal pattern)
    frame.render_widget(Clear, popup_area);

    // Format the text - truncate long task names
    let display_name = if task_name.len() > 45 {
        format!("{}...", &task_name[..45])
    } else {
        task_name.to_string()
    };

    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("Delete task: \"{}\"?", display_name),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "This action cannot be undone.",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Enter: confirm, Esc: cancel",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Span::styled(
                    "⚠️  Confirm Delete",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ))
                .border_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        )
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, popup_area);
}

fn render_move_task_modal(frame: &mut Frame, size: Rect, task_name: &str, state: &State) {
    use crate::ui::widgets::styling;
    use ratatui::{
        layout::{Alignment, Constraint, Direction, Layout},
        style::{Color, Modifier, Style},
        text::Span,
        widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    };

    // Create a centered popup dialog using ratatui pattern
    // Make it taller to show 5 items comfortably
    let popup_area = centered_rect(50, 40, size);

    // Clear the area first (ratatui modal pattern)
    frame.render_widget(Clear, popup_area);

    // Get sections and selected index
    let sections = state.get_sections();
    let selected_index = state.get_section_dropdown_index();

    // Split popup into title and list areas
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(7)])
        .split(popup_area);

    // Title block - just "Move"
    let title_block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            "Move",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ))
        .border_style(styling::active_block_border_style());

    let title_text = Paragraph::new("j/k: navigate, Enter: select, Esc: cancel")
        .block(title_block)
        .alignment(Alignment::Center);
    frame.render_widget(title_text, chunks[0]);

    // Limit visible sections to max 5 items (with scrolling)
    let max_visible = 5;
    let total_items = sections.len();
    let start_index = if total_items <= max_visible {
        0
    } else {
        (selected_index as i32 - max_visible as i32 / 2)
            .max(0)
            .min((total_items - max_visible) as i32) as usize
    };
    let end_index = (start_index + max_visible).min(total_items);
    let visible_sections = if sections.is_empty() {
        vec![]
    } else {
        sections[start_index..end_index].to_vec()
    };
    let visible_selected = selected_index.saturating_sub(start_index);

    // Create list items from visible sections
    let items: Vec<ListItem> = if visible_sections.is_empty() {
        vec![ListItem::new("No sections available")]
    } else {
        visible_sections
            .iter()
            .map(|section| ListItem::new(section.name.clone()))
            .collect()
    };

    // Use ListState for proper selection display
    let mut list_state = ratatui::widgets::ListState::default();
    if !items.is_empty() && !sections.is_empty() {
        let safe_index = visible_selected.min(items.len().saturating_sub(1));
        list_state.select(Some(safe_index));
    }

    // Create list block with section count
    let list_block = Block::default()
        .borders(Borders::ALL)
        .title(format!("Sections ({})", sections.len()))
        .border_style(styling::active_block_border_style());

    let list = List::new(items)
        .block(list_block)
        .style(styling::normal_text_style())
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_stateful_widget(list, chunks[1], &mut list_state);
}

/// Helper function to create a centered rectangle (ratatui modal pattern)
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
