use super::welcome;
use super::{create_task, edit_task, kanban, task_detail, Frame};
use crate::state::{Focus, State, View};
use crate::ui::widgets::styling;
use tui::{
    layout::Rect,
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
        }
        View::RecentlyModified => {
            recently_modified(frame, size, state);
        }
        View::RecentlyCompleted => {
            recently_completed(frame, size, state);
        }
        View::ProjectTasks => {
            // Always show kanban view first (so modal appears on top)
            kanban::kanban(frame, size, state);
            
            // Check if we need to show move task section selection modal (render on top)
            if state.has_move_task() {
                // Get task name for display before borrowing state mutably
                let task_name = state.get_kanban_selected_task()
                    .map(|t| t.name.clone())
                    .unwrap_or_else(|| "task".to_string());
                render_move_task_modal(frame, size, &task_name, state);
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

fn project_tasks(frame: &mut Frame, size: Rect, state: &mut State) {
    let project = state.get_project();
    let title = if let Some(project) = project {
        format!("{} - Tasks", project.name)
    } else {
        "Tasks".to_string()
    };
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

fn task_list(tasks: &[crate::asana::Task]) -> tui::widgets::List {
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
        Spans::from(Span::styled(
            "This action cannot be undone.",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Spans::from(""),
    ];

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Span::styled(
                    "⚠️  Confirm Delete",
                    Style::default()
                        .fg(Color::Red)
                        .add_modifier(Modifier::BOLD),
                ))
                .border_style(
                    Style::default()
                        .fg(Color::Red)
                        .add_modifier(Modifier::BOLD),
                ),
        )
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    frame.render_widget(Clear, dialog_rect); // Clear the area first
    frame.render_widget(paragraph, dialog_rect);
}

fn render_move_task_modal(frame: &mut Frame, size: Rect, task_name: &str, state: &State) {
    use crate::ui::widgets::styling;
    use tui::{
        layout::Alignment,
        style::{Color, Modifier, Style},
        text::{Span, Spans},
        widgets::{Block, Borders, Clear, List, ListItem},
    };

    // Create a centered popup dialog
    let dialog_width = 50.min(size.width.saturating_sub(4));
    let dialog_height = 15.min(size.height.saturating_sub(4));
    
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

    let popup = Rect {
        x,
        y,
        width: dialog_width,
        height: dialog_height,
    };

    // Get sections and selected index
    let sections = state.get_sections();
    let selected_index = state.get_section_dropdown_index();

    // Format task name - truncate if too long
    let display_name = if task_name.len() > 35 {
        format!("{}...", &task_name[..35])
    } else {
        task_name.to_string()
    };

    // Create list items
    let items: Vec<ListItem> = sections
        .iter()
        .enumerate()
        .map(|(i, section)| {
            let style = if i == selected_index {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                styling::normal_text_style()
            };
            ListItem::new(Spans::from(Span::styled(&section.name, style)))
        })
        .collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!("Move '{}' to section", display_name))
        .border_style(styling::active_block_border_style());

    // Use ListState for proper selection display
    let mut list_state = tui::widgets::ListState::default();
    if !items.is_empty() {
        list_state.select(Some(selected_index.min(items.len().saturating_sub(1))));
    }

    let list = List::new(items)
        .block(block)
        .style(styling::normal_text_style())
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_widget(Clear, popup); // Clear the area first
    frame.render_stateful_widget(list, popup, &mut list_state);
}
