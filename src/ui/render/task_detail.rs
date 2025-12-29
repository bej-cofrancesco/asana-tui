use super::Frame;
use crate::state::{State, TaskDetailPanel};
use crate::ui::widgets::styling;
use chrono::DateTime;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

/// Render task detail view (full screen).
///
pub fn task_detail(frame: &mut Frame, size: Rect, state: &mut State) {
    // Get task and panel info before borrowing state mutably
    let task_opt = state.get_task_detail().cloned();
    let current_panel = state.get_current_task_panel();
    let is_comment_input = state.is_comment_input_mode();

    if let Some(task) = task_opt {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(10),   // Main content
            ])
            .split(size);

        let panel_indicators = format!(
            "{} {} {}",
            if current_panel == TaskDetailPanel::Details {
                "[Details]"
            } else {
                " Details "
            },
            if current_panel == TaskDetailPanel::Comments || is_comment_input {
                "[Comments]"
            } else {
                " Comments "
            },
            if current_panel == TaskDetailPanel::Notes {
                "[Notes]"
            } else {
                " Notes "
            },
        );

        let header = Block::default()
            .borders(Borders::ALL)
            .title(format!(
                "Task Details - h/l: switch panel | {}",
                panel_indicators
            ))
            .border_style(styling::active_block_border_style());

        let name_text = Line::from(vec![Span::styled(
            &task.name,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )]);
        let name_para = Paragraph::new(name_text)
            .block(header)
            .alignment(Alignment::Left);
        frame.render_widget(name_para, chunks[0]);

        // Main content area - show only the active panel
        // If comment input mode is active, always show Comments panel
        let panel_to_show = if is_comment_input {
            TaskDetailPanel::Comments
        } else {
            current_panel
        };

        match panel_to_show {
            TaskDetailPanel::Details => {
                render_task_properties(frame, chunks[1], &task, state);
            }
            TaskDetailPanel::Comments => {
                render_comments(frame, chunks[1], state, &task);
            }
            TaskDetailPanel::Notes => {
                render_notes(frame, chunks[1], &task, state);
            }
        }
    } else {
        // Loading or no task selected
        let block = Block::default().borders(Borders::ALL).title("Task Details");
        let text = Paragraph::new("Loading task details...")
            .block(block)
            .alignment(Alignment::Center);
        frame.render_widget(text, size);
    }
}

fn render_task_properties(frame: &mut Frame, size: Rect, task: &crate::asana::Task, state: &State) {
    let is_active = state.get_current_task_panel() == TaskDetailPanel::Details;

    let mut lines = vec![];

    // Assignee
    if let Some(ref assignee) = task.assignee {
        lines.push(Line::from(vec![
            Span::styled("Assignee: ", Style::default().fg(Color::Yellow)),
            Span::styled(&assignee.name, styling::normal_text_style()),
        ]));
    } else {
        lines.push(Line::from(vec![
            Span::styled("Assignee: ", Style::default().fg(Color::Yellow)),
            Span::styled("Unassigned", Style::default().fg(Color::DarkGray)),
        ]));
    }

    // Due date
    if let Some(ref due_on) = task.due_on {
        lines.push(Line::from(vec![
            Span::styled("Due: ", Style::default().fg(Color::Yellow)),
            Span::styled(due_on, styling::normal_text_style()),
        ]));
    }

    // Section
    if let Some(ref section) = task.section {
        lines.push(Line::from(vec![
            Span::styled("Section: ", Style::default().fg(Color::Yellow)),
            Span::styled(&section.name, styling::normal_text_style()),
        ]));
    }

    // Tags
    if !task.tags.is_empty() {
        let tag_names: Vec<String> = task.tags.iter().map(|t| t.name.clone()).collect();
        lines.push(Line::from(vec![
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
    lines.push(Line::from(vec![
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
    lines.push(Line::from(vec![
        Span::styled("Subtasks: ", Style::default().fg(Color::Yellow)),
        Span::styled(task.num_subtasks.to_string(), styling::normal_text_style()),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Comments: ", Style::default().fg(Color::Yellow)),
        Span::styled(task.num_comments.to_string(), styling::normal_text_style()),
    ]));

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Properties")
        .border_style(if is_active {
            styling::active_block_border_style()
        } else {
            styling::normal_block_border_style()
        });
    let text = Paragraph::new(Text::from(lines))
        .block(block)
        .wrap(Wrap { trim: true });
    frame.render_widget(text, size);
}

fn render_comments(frame: &mut Frame, size: Rect, state: &mut State, _task: &crate::asana::Task) {
    let is_active = state.get_current_task_panel() == TaskDetailPanel::Comments;
    let is_comment_input = state.is_comment_input_mode();
    let mut comments_list_state = state.get_comments_list_state().clone();

    // Get all comments
    let stories: Vec<crate::asana::Story> = state.get_task_stories().to_vec();
    let comments: Vec<&crate::asana::Story> = stories
        .iter()
        .filter(|s| match &s.resource_subtype {
            Some(subtype) => subtype == "comment_added",
            None => s.created_by.is_some(),
        })
        .collect();

    // Split into comments area and input area if in comment input mode
    let chunks = if is_comment_input {
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

    let title = format!("Comments ({})", comments.len());
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(if is_active || is_comment_input {
            styling::active_block_border_style()
        } else {
            styling::normal_block_border_style()
        });

    if comments.is_empty() && !is_comment_input {
        let text = Paragraph::new("No comments yet. Press 'c' to add a comment.")
            .block(block)
            .alignment(Alignment::Center);
        frame.render_widget(text, chunks[0]);
    } else {
        // Build simple list items
        let items: Vec<ListItem> = comments
            .iter()
            .map(|story| {
                let author = story
                    .created_by
                    .as_ref()
                    .map(|u| u.name.as_str())
                    .unwrap_or("Unknown");
                let timestamp_str = story
                    .created_at
                    .as_ref()
                    .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                    .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                    .unwrap_or_else(|| "Unknown time".to_string());

                // Simple format: header line with author and timestamp, then comment text
                vec![
                    Line::from(vec![
                        Span::styled(
                            author,
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(" • ", Style::default().fg(Color::DarkGray)),
                        Span::styled(timestamp_str, Style::default().fg(Color::DarkGray)),
                    ]),
                    Line::from(vec![Span::styled(
                        &story.text,
                        Style::default().fg(Color::White),
                    )]),
                ]
            })
            .map(ListItem::new)
            .collect();

        let list = List::new(items)
            .block(block)
            .style(styling::normal_text_style())
            .highlight_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            );

        frame.render_stateful_widget(list, chunks[0], &mut comments_list_state);
    }

    // Show comment input if in comment input mode
    if is_comment_input {
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

fn render_notes(frame: &mut Frame, size: Rect, task: &crate::asana::Task, state: &State) {
    let is_active = state.get_current_task_panel() == TaskDetailPanel::Notes;

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Notes")
        .border_style(if is_active {
            styling::active_block_border_style()
        } else {
            styling::normal_block_border_style()
        });

    let notes_text = task.notes.as_deref().unwrap_or("No notes");
    let text = Paragraph::new(notes_text)
        .block(block)
        .wrap(Wrap { trim: true })
        .style(styling::normal_text_style());
    frame.render_widget(text, size);
}
