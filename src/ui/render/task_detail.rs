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
use regex::Regex;
use std::collections::HashMap;

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

/// Create a map of user GID to user name for fast lookups
fn create_user_map(users: &[crate::asana::User]) -> HashMap<String, String> {
    users
        .iter()
        .map(|u| (u.gid.clone(), u.name.clone()))
        .collect()
}

/// Parse line text and highlight @mentions, wrapping text properly
/// Note: Profile URLs are already replaced with @username when data comes from API
fn parse_comment_text(
    line: &str,
    user_map: &HashMap<String, String>,
    width: usize,
) -> Vec<Line<'static>> {
    // Create a HashSet for O(1) lookup instead of O(n) any() check
    use std::collections::HashSet;
    let user_names: HashSet<String> = user_map.values().map(|name| name.to_lowercase()).collect();

    let mut lines = Vec::new();
    let mut current_line = Vec::new();
    let mut current_length = 0;

    // Pattern to match @mentions - handle adjacent mentions by matching word boundaries
    // This will match @username even if followed immediately by another @
    let mention_re = Regex::new(r"@(\w+)").unwrap();
    let mut last_end = 0;

    for cap in mention_re.captures_iter(line) {
        let full_match = cap.get(0).unwrap();
        let mention_name = cap.get(1).map(|m| m.as_str()).unwrap_or("");

        // Add text before mention
        if full_match.start() > last_end {
            let before_text = &line[last_end..full_match.start()];
            let words: Vec<&str> = before_text.split_whitespace().collect();
            for word in words {
                let word_with_space = if current_length > 0 {
                    format!(" {}", word)
                } else {
                    word.to_string()
                };
                if current_length + word_with_space.len() > width && !current_line.is_empty() {
                    lines.push(Line::from(current_line.clone()));
                    current_line.clear();
                    current_length = 0;
                }
                if current_length > 0 {
                    current_line.push(Span::styled(" ".to_string(), styling::normal_text_style()));
                    current_length += 1;
                }
                current_line.push(Span::styled(word.to_string(), styling::normal_text_style()));
                current_length += word.len();
            }
        }

        // Check if mention matches a known user - O(1) lookup
        let is_valid_mention = user_names.contains(&mention_name.to_lowercase());
        let mention_text = full_match.as_str();

        // Check if mention fits on current line
        if current_length + mention_text.len() > width && !current_line.is_empty() {
            lines.push(Line::from(current_line.clone()));
            current_line.clear();
            current_length = 0;
        }

        // Add space before mention if needed (but not if it's at start of line or after another mention)
        // Only add space if there's actual text before it
        if current_length > 0 && last_end < full_match.start() {
            // Check if there's non-whitespace before the mention
            let text_before = &line[last_end..full_match.start()];
            if !text_before.trim().is_empty() {
                current_line.push(Span::styled(" ".to_string(), styling::normal_text_style()));
                current_length += 1;
            }
        }

        if is_valid_mention {
            current_line.push(Span::styled(
                mention_text.to_string(),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ));
        } else {
            current_line.push(Span::styled(
                mention_text.to_string(),
                styling::normal_text_style(),
            ));
        }
        current_length += mention_text.len();

        last_end = full_match.end();
    }

    // Add remaining text
    if last_end < line.len() {
        let remaining = &line[last_end..];
        let words: Vec<&str> = remaining.split_whitespace().collect();
        for word in words {
            let word_with_space = if current_length > 0 {
                format!(" {}", word)
            } else {
                word.to_string()
            };
            if current_length + word_with_space.len() > width && !current_line.is_empty() {
                lines.push(Line::from(current_line.clone()));
                current_line.clear();
                current_length = 0;
            }
            if current_length > 0 {
                current_line.push(Span::styled(" ".to_string(), styling::normal_text_style()));
                current_length += 1;
            }
            current_line.push(Span::styled(word.to_string(), styling::normal_text_style()));
            current_length += word.len();
        }
    }

    if !current_line.is_empty() {
        lines.push(Line::from(current_line));
    }

    if lines.is_empty() {
        vec![Line::from(Span::styled(
            line.to_string(),
            styling::normal_text_style(),
        ))]
    } else {
        lines
    }
}

fn render_comments(frame: &mut Frame, size: Rect, state: &mut State, _task: &crate::asana::Task) {
    let is_active = state.get_current_task_panel() == TaskDetailPanel::Comments;
    // Clone stories to avoid borrow checker issues - limit to reasonable number for performance
    let stories: Vec<crate::asana::Story> = state.get_task_stories().to_vec();
    let users = state.get_workspace_users().to_vec();
    let user_map = create_user_map(&users);

    // Filter for actual comments (resource_subtype = "comment_added")
    let all_comments: Vec<&crate::asana::Story> = stories
        .iter()
        .filter(|s| match &s.resource_subtype {
            Some(subtype) => subtype == "comment_added",
            None => s.created_by.is_some(),
        })
        .collect();
    
    // Limit visible comments to prevent lag - only render what's visible + buffer
    // Use ListState to track which comments are visible
    let comments_list_state = state.get_comments_list_state();
    let selected_index = comments_list_state.selected().unwrap_or(0);
    
    // Calculate visible range (show ~20 comments max at a time)
    const MAX_VISIBLE: usize = 20;
    let start_index = if all_comments.len() <= MAX_VISIBLE {
        0
    } else {
        (selected_index as i32 - MAX_VISIBLE as i32 / 2)
            .max(0)
            .min((all_comments.len() - MAX_VISIBLE) as i32) as usize
    };
    let end_index = (start_index + MAX_VISIBLE).min(all_comments.len());
    let comments = &all_comments[start_index..end_index];

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

    let title = format!("Comments ({})", all_comments.len());
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
        // Calculate available width for wrapping (accounting for borders)
        let available_width = chunks[0].width.saturating_sub(2) as usize;

        // Build list items with wrapped text
        let mut items: Vec<ListItem> = Vec::new();
        for story in &comments {
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

            // Text is already processed when it came from API (URLs replaced with @username)
            // Parse and wrap comment text
            let comment_lines = parse_comment_text(&story.text, &user_map, available_width);

            // Build header line
            let header_line = Line::from(vec![
                Span::styled(author, Style::default().fg(Color::Yellow)),
                Span::styled(" • ", Style::default().fg(Color::DarkGray)),
                Span::styled(timestamp_str, Style::default().fg(Color::DarkGray)),
            ]);

            // Combine header and comment lines
            let mut all_lines = vec![header_line];
            all_lines.extend(comment_lines);

            items.push(ListItem::new(all_lines));
        }

        let list = List::new(items)
            .block(block)
            .style(styling::normal_text_style())
            .highlight_style(styling::active_list_item_style());

        frame.render_stateful_widget(list, chunks[0], state.get_comments_list_state());
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
