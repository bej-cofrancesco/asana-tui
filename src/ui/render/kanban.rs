use super::Frame;
use crate::state::State;
use crate::ui::widgets::styling;
use tui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

/// Render kanban board view with split layout (columns + details).
///
pub fn kanban(frame: &mut Frame, size: Rect, state: &State) {
    let sections = state.get_sections();
    
    if sections.is_empty() {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Kanban Board");
        let text = Paragraph::new("No sections found. Loading...")
            .block(block)
            .alignment(Alignment::Center);
        frame.render_widget(text, size);
        return;
    }

    // Split into columns area and details area (70/30 split)
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(size);

    // Render kanban columns on the left
    render_kanban_columns(frame, chunks[0], state);
    
    // Render task details on the right
    render_kanban_details(frame, chunks[1], state);
}

fn render_kanban_columns(frame: &mut Frame, size: Rect, state: &State) {
    let sections = state.get_sections();
    let tasks = state.get_tasks();
    let current_column = state.get_kanban_column_index();
    let current_task_index = state.get_kanban_task_index();

    // Create horizontal layout for columns
    let constraints: Vec<Constraint> = (0..sections.len())
        .map(|_| Constraint::Percentage((100 / sections.len().max(1)) as u16))
        .collect();

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints.as_slice())
        .split(size);

    // Render each section as a column
    for (idx, section) in sections.iter().enumerate() {
        let section_tasks: Vec<&crate::asana::Task> = tasks
            .iter()
            .filter(|t| {
                t.section
                    .as_ref()
                    .map(|s| s.gid == section.gid)
                    .unwrap_or(false)
            })
            .collect();

        render_kanban_column(
            frame,
            chunks[idx],
            section,
            &section_tasks,
            idx == current_column,
            if idx == current_column { Some(current_task_index) } else { None },
        );
    }
}

fn render_kanban_column(
    frame: &mut Frame,
    size: Rect,
    section: &crate::asana::Section,
    tasks: &[&crate::asana::Task],
    is_selected: bool,
    selected_task_index: Option<usize>,
) {
    let title = format!("{} ({})", section.name, tasks.len());
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(if is_selected {
            styling::active_block_border_style()
        } else {
            styling::normal_block_border_style()
        });

    if tasks.is_empty() {
        let empty_text = Paragraph::new("No tasks")
            .block(block)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(empty_text, size);
        return;
    }

    let items: Vec<ListItem> = tasks
        .iter()
        .enumerate()
        .map(|(idx, task)| {
            let mut spans = vec![];
            
            // Task name (bold if selected)
            let name_style = if is_selected && selected_task_index == Some(idx) {
                styling::active_list_item_style()
            } else {
                styling::normal_text_style()
            };
            spans.push(Span::styled(&task.name, name_style));

            // Add assignee indicator
            if let Some(ref assignee) = task.assignee {
                spans.push(Span::styled(
                    format!(" (@{})", assignee.name),
                    Style::default().fg(Color::Cyan),
                ));
            }

            // Add due date if present
            if let Some(ref due_on) = task.due_on {
                spans.push(Span::styled(
                    format!(" [{}]", due_on),
                    Style::default().fg(Color::Yellow),
                ));
            }

            // Add completion indicator
            if task.completed {
                spans.push(Span::styled(
                    " ✓",
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                ));
            }

            ListItem::new(Spans::from(spans))
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .style(styling::normal_text_style())
        .highlight_style(styling::active_list_item_style());

    // Create list state with selection
    let mut list_state = tui::widgets::ListState::default();
    if is_selected {
        if let Some(idx) = selected_task_index {
            if idx < tasks.len() {
                list_state.select(Some(idx));
            } else if !tasks.is_empty() {
                list_state.select(Some(0));
            }
        } else if !tasks.is_empty() {
            list_state.select(Some(0));
        }
    }

    frame.render_stateful_widget(list, size, &mut list_state);
}

fn render_kanban_details(frame: &mut Frame, size: Rect, state: &State) {
    let sections = state.get_sections();
    let tasks = state.get_tasks();
    let current_column = state.get_kanban_column_index();
    let current_task_index = state.get_kanban_task_index();

    if current_column >= sections.len() {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Details");
        let text = Paragraph::new("Select a task to view details")
            .block(block)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(text, size);
        return;
    }

    let section = &sections[current_column];
    let section_tasks: Vec<&crate::asana::Task> = tasks
        .iter()
        .filter(|t| {
            t.section
                .as_ref()
                .map(|s| s.gid == section.gid)
                .unwrap_or(false)
        })
        .collect();

    if current_task_index >= section_tasks.len() {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Details");
        let text = Paragraph::new("No task selected")
            .block(block)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(text, size);
        return;
    }

    let task = section_tasks[current_task_index];
    
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Details")
        .border_style(styling::active_block_border_style());
    
    let mut lines = vec![];
    
    // Task name (bold)
    lines.push(Spans::from(vec![
        Span::styled(
            &task.name,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Spans::from(vec![Span::raw("")])); // Empty line
    
    // Status
    lines.push(Spans::from(vec![
        Span::styled("Status: ", Style::default().fg(Color::Yellow)),
        Span::styled(
            if task.completed { "Completed" } else { "Incomplete" },
            styling::normal_text_style(),
        ),
    ]));
    
    // Section
    lines.push(Spans::from(vec![
        Span::styled("Section: ", Style::default().fg(Color::Yellow)),
        Span::styled(&section.name, styling::normal_text_style()),
    ]));
    
    // Assignee
    if let Some(ref assignee) = task.assignee {
        lines.push(Spans::from(vec![
            Span::styled("Assignee: ", Style::default().fg(Color::Yellow)),
            Span::styled(&assignee.name, styling::normal_text_style()),
        ]));
    } else {
        lines.push(Spans::from(vec![
            Span::styled("Assignee: ", Style::default().fg(Color::Yellow)),
            Span::styled("Unassigned", Style::default().fg(Color::DarkGray)),
        ]));
    }
    
    // Due date
    if let Some(ref due_on) = task.due_on {
        lines.push(Spans::from(vec![
            Span::styled("Due: ", Style::default().fg(Color::Yellow)),
            Span::styled(due_on, styling::normal_text_style()),
        ]));
    }
    
    // Notes
    if let Some(ref notes) = task.notes {
        if !notes.trim().is_empty() {
            lines.push(Spans::from(vec![Span::raw("")])); // Empty line
            lines.push(Spans::from(vec![
                Span::styled("Notes:", Style::default().fg(Color::Yellow)),
            ]));
            // Wrap notes text
            let notes_lines: Vec<&str> = notes.lines().collect();
            for line in notes_lines.iter().take(10) { // Limit to 10 lines
                lines.push(Spans::from(vec![Span::styled(
                    *line,
                    styling::normal_text_style(),
                )]));
            }
            if notes_lines.len() > 10 {
                lines.push(Spans::from(vec![Span::styled(
                    "...",
                    Style::default().fg(Color::DarkGray),
                )]));
            }
        }
    }
    
    // Subtasks and comments count
    lines.push(Spans::from(vec![Span::raw("")])); // Empty line
    if task.num_subtasks > 0 {
        lines.push(Spans::from(vec![
            Span::styled("Subtasks: ", Style::default().fg(Color::Yellow)),
            Span::styled(task.num_subtasks.to_string(), styling::normal_text_style()),
        ]));
    }
    if task.num_comments > 0 {
        lines.push(Spans::from(vec![
            Span::styled("Comments: ", Style::default().fg(Color::Yellow)),
            Span::styled(task.num_comments.to_string(), styling::normal_text_style()),
        ]));
    }
    
    let text = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: true });
    
    frame.render_widget(text, size);
}
