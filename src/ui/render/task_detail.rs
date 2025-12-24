use super::Frame;
use crate::state::State;
use crate::ui::widgets::styling;
use chrono::DateTime;
use tui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

/// Render task detail view (full screen).
///
pub fn task_detail(frame: &mut Frame, size: Rect, state: &State) {
    if let Some(task) = state.get_task_detail() {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(1),    // Main content
                Constraint::Length(1), // Footer
            ])
            .split(size);

        // Header with task name
        let header = Block::default()
            .borders(Borders::ALL)
            .title("Task Details")
            .border_style(styling::active_block_border_style());

        let name_text = Spans::from(vec![Span::styled(
            &task.name,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )]);
        let name_para = Paragraph::new(name_text)
            .block(header)
            .alignment(Alignment::Left);
        frame.render_widget(name_para, chunks[0]);

        // Main content area
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[1]);

        // Left column: Task properties
        render_task_properties(frame, main_chunks[0], task, state);

        // Right column: Comments
        render_comments(frame, main_chunks[1], state, task);

        // Notes area at bottom
        let notes_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(5)])
            .split(chunks[1]);

        render_notes(frame, notes_chunks[1], task);
    } else {
        // Loading or no task selected
        let block = Block::default().borders(Borders::ALL).title("Task Details");
        let text = Paragraph::new("Loading task details...")
            .block(block)
            .alignment(Alignment::Center);
        frame.render_widget(text, size);
    }
}

fn render_task_properties(frame: &mut Frame, size: Rect, task: &crate::asana::Task, _state: &State) {
    let mut lines = vec![];

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

    // Section
    if let Some(ref section) = task.section {
        lines.push(Spans::from(vec![
            Span::styled("Section: ", Style::default().fg(Color::Yellow)),
            Span::styled(&section.name, styling::normal_text_style()),
        ]));
    }

    // Tags
    if !task.tags.is_empty() {
        let tag_names: Vec<String> = task.tags.iter().map(|t| t.name.clone()).collect();
        lines.push(Spans::from(vec![
            Span::styled("Tags: ", Style::default().fg(Color::Yellow)),
            Span::styled(tag_names.join(", "), styling::normal_text_style()),
        ]));
    }

    // Status
    let status_text = if task.completed {
        "Completed"
    } else {
        "Incomplete"
    };
    lines.push(Spans::from(vec![
        Span::styled("Status: ", Style::default().fg(Color::Yellow)),
        Span::styled(
            status_text,
            if task.completed {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Red)
            },
        ),
    ]));

    // Subtasks and comments count
    lines.push(Spans::from(vec![
        Span::styled("Subtasks: ", Style::default().fg(Color::Yellow)),
        Span::styled(task.num_subtasks.to_string(), styling::normal_text_style()),
    ]));
    lines.push(Spans::from(vec![
        Span::styled("Comments: ", Style::default().fg(Color::Yellow)),
        Span::styled(task.num_comments.to_string(), styling::normal_text_style()),
    ]));

    let block = Block::default().borders(Borders::ALL).title("Properties");
    let text = Paragraph::new(Text::from(lines))
        .block(block)
        .wrap(Wrap { trim: true });
    frame.render_widget(text, size);
}

fn render_comments(frame: &mut Frame, size: Rect, state: &State, _task: &crate::asana::Task) {
    let stories = state.get_task_stories();

    // Split into comments area and input area if in comment input mode
    let chunks = if state.is_comment_input_mode() {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(3)])
            .split(size)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0)])
            .split(size)
    };

    let block = Block::default().borders(Borders::ALL).title("Comments");

    if stories.is_empty() && !state.is_comment_input_mode() {
        let text = Paragraph::new("No comments yet. Press 'c' to add a comment.")
            .block(block)
            .alignment(Alignment::Center);
        frame.render_widget(text, chunks[0]);
    } else {
        let items: Vec<ListItem> = stories
            .iter()
            .map(|story| {
                let author = story
                    .created_by
                    .as_ref()
                    .map(|u| u.name.clone())
                    .unwrap_or_else(|| "Unknown".to_string());
                let timestamp_str = story
                    .created_at
                    .as_ref()
                    .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                    .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                    .unwrap_or_else(|| "Unknown time".to_string());

                let header = Spans::from(vec![
                    Span::styled(author, Style::default().fg(Color::Cyan)),
                    Span::styled(" • ", Style::default().fg(Color::DarkGray)),
                    Span::styled(timestamp_str, Style::default().fg(Color::DarkGray)),
                ]);

                ListItem::new(vec![
                    header,
                    Spans::from(vec![Span::styled(
                        story.text.clone(),
                        styling::normal_text_style(),
                    )]),
                ])
            })
            .collect();

        let list = List::new(items)
            .block(block)
            .style(styling::normal_text_style());
        frame.render_widget(list, chunks[0]);
    }

    // Show comment input if in comment input mode
    if state.is_comment_input_mode() {
        let input_block = Block::default()
            .borders(Borders::ALL)
            .title("Add Comment (Enter: submit, Esc: cancel)");
        let input_text = format!("> {}", state.get_comment_input_text());
        let input_para = Paragraph::new(input_text)
            .block(input_block)
            .style(styling::normal_text_style());
        frame.render_widget(input_para, chunks[1]);
    }
}

fn render_notes(frame: &mut Frame, size: Rect, task: &crate::asana::Task) {
    let block = Block::default().borders(Borders::ALL).title("Notes");

    let notes_text = task.notes.as_deref().unwrap_or("No notes");
    let text = Paragraph::new(notes_text)
        .block(block)
        .wrap(Wrap { trim: true })
        .style(styling::normal_text_style());
    frame.render_widget(text, size);
}
