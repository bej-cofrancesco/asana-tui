use super::Frame;
use crate::state::{State, TaskDetailPanel};
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
                Constraint::Min(10),   // Main content
            ])
            .split(size);

        // Header with task name and panel indicators
        let current_panel = state.get_current_task_panel();
        let panel_indicators = format!(
            "{} {} {}",
            if current_panel == TaskDetailPanel::Details {
                "[Details]"
            } else {
                " Details "
            },
            if current_panel == TaskDetailPanel::Comments {
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

        // Main content area - show only the active panel
        match current_panel {
            TaskDetailPanel::Details => {
                render_task_properties(frame, chunks[1], task, state);
            }
            TaskDetailPanel::Comments => {
                render_comments(frame, chunks[1], state, task);
            }
            TaskDetailPanel::Notes => {
                render_notes(frame, chunks[1], task, state);
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

fn render_comments(frame: &mut Frame, size: Rect, state: &State, _task: &crate::asana::Task) {
    let is_active = state.get_current_task_panel() == TaskDetailPanel::Comments;
    let stories = state.get_task_stories();

    // Filter for actual comments (resource_subtype = "comment_added")
    // Ignore system activity messages
    let comments: Vec<&crate::asana::Story> = stories
        .iter()
        .filter(|s| {
            // Include if it's a comment_added, or if subtype is missing but created_by exists
            // (for backwards compatibility)
            match &s.resource_subtype {
                Some(subtype) => subtype == "comment_added",
                None => s.created_by.is_some(), // Fallback: if no subtype, assume comment if has author
            }
        })
        .collect();

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

    let title = format!("Comments ({})", comments.len());

    // Highlight border when active OR when in comment input mode
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(if is_active || state.is_comment_input_mode() {
            styling::active_block_border_style()
        } else {
            styling::normal_block_border_style()
        });

    if comments.is_empty() && !state.is_comment_input_mode() {
        let text = Paragraph::new("No comments yet. Press 'c' to add a comment.")
            .block(block)
            .alignment(Alignment::Center);
        frame.render_widget(text, chunks[0]);
    } else {
        // Calculate available width for text (accounting for borders: 2 chars on each side)
        let available_width = chunks[0].width.saturating_sub(4).max(10) as usize;
        
        // Helper function to wrap text to fit width
        let wrap_text = |text: &str, width: usize| -> Vec<String> {
            let mut wrapped = Vec::new();
            for line in text.lines() {
                if line.is_empty() {
                    wrapped.push(String::new());
                    continue;
                }
                let mut remaining = line;
                while !remaining.is_empty() {
                    if remaining.len() <= width {
                        wrapped.push(remaining.to_string());
                        break;
                    }
                    // Find the last space before the width limit
                    let mut break_point = width;
                    if let Some(last_space) = remaining[..width].rfind(' ') {
                        break_point = last_space + 1;
                    }
                    wrapped.push(remaining[..break_point].trim_end().to_string());
                    remaining = remaining[break_point..].trim_start();
                }
            }
            wrapped
        };

        // Create nice multi-line comment items with proper formatting
        let items: Vec<ListItem> = comments
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

                // Wrap comment text to fit available width
                let wrapped_lines = wrap_text(&story.text, available_width);
                
                // Create Spans for each wrapped line
                let text_lines: Vec<Spans> = wrapped_lines
                    .iter()
                    .map(|line| {
                        Spans::from(Span::styled(
                            line.clone(),
                            styling::normal_text_style(),
                        ))
                    })
                    .collect();

                let mut all_lines = vec![header, Spans::from("")]; // Add blank line after header
                all_lines.extend(text_lines);
                all_lines.push(Spans::from("")); // Add blank line after comment

                ListItem::new(all_lines)
            })
            .collect();

        // Use ListState for proper item-by-item navigation
        // For bottom-aligned list: index 0 = newest (last item), higher = older
        let mut selected_index = state.get_comments_scroll_offset();
        let total_items = items.len();
        
        // Ensure selected_index is within bounds (wrap around if needed)
        if total_items > 0 {
            selected_index = selected_index % total_items;
        } else {
            selected_index = 0;
        }
        
        // Create list with all items - tui-rs List widget handles scrolling automatically
        let list = List::new(items.clone())
            .block(block)
            .style(styling::normal_text_style());
            // No highlight_style - make selection subtle (just cursor, no color)
        
        // Create a list state with selected item (from bottom: 0 = last item)
        let mut list_state = tui::widgets::ListState::default();
        if !items.is_empty() {
            // Convert from bottom index to top index: last item = index 0, second last = index 1, etc.
            let top_index = total_items.saturating_sub(1).saturating_sub(selected_index);
            list_state.select(Some(top_index.min(total_items.saturating_sub(1))));
        }
        
        frame.render_stateful_widget(list, chunks[0], &mut list_state);
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
