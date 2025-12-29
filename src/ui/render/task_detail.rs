use super::Frame;
use crate::state::{State, TaskDetailPanel};
use crate::ui::widgets::styling;
use chrono::DateTime;
use regex::Regex;
use tui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
};

/// Parse mentions in text - convert profile URLs to @mentions
fn parse_mentions(text: &str, users: &[crate::asana::User]) -> String {
    // Pattern: https://app.asana.com/0/profile/{gid}
    let profile_re = Regex::new(r"https://app\.asana\.com/0/profile/(\d+)").unwrap();
    
    profile_re.replace_all(text, |caps: &regex::Captures| {
        if let Some(gid) = caps.get(1) {
            // Find user by GID
            if let Some(user) = users.iter().find(|u| u.gid == gid.as_str()) {
                format!("@{}", user.name)
            } else {
                // Keep original URL if user not found
                caps.get(0).map(|m| m.as_str()).unwrap_or("").to_string()
            }
        } else {
            caps.get(0).map(|m| m.as_str()).unwrap_or("").to_string()
        }
    }).to_string()
}

/// Parse line text and highlight @mentions
fn parse_line_with_mentions(line: &str, users: &[crate::asana::User]) -> Spans<'static> {
    let mut spans = Vec::new();
    
    // Pattern to match @mentions - match @ followed by word characters and spaces
    // We'll manually check for delimiters after matching to avoid look-ahead
    let mention_re = Regex::new(r"@([\w\s]+)").unwrap();
    let mut last_end = 0;
    
    for cap in mention_re.captures_iter(line) {
        let full_match = cap.get(0).unwrap();
        let mention_name = cap.get(1).map(|m| m.as_str().trim()).unwrap_or("");
        
        // Check if the mention is followed by a valid delimiter or end of string
        // Valid delimiters: whitespace, punctuation, or end of line
        let match_end = full_match.end();
        let is_valid_mention_end = if match_end >= line.len() {
            true // End of string is always valid
        } else {
            let next_char = line[match_end..].chars().next();
            // Valid if followed by whitespace, punctuation, or nothing
            next_char.map(|c| {
                c.is_whitespace() || matches!(c, '.' | ',' | '!' | '?' | ';' | ':')
            }).unwrap_or(true)
        };
        
        // Only process if it's a valid mention boundary
        if is_valid_mention_end {
            // Add text before mention (clone to own the data)
            if full_match.start() > last_end {
                spans.push(Span::styled(
                    line[last_end..full_match.start()].to_string(),
                    styling::normal_text_style(),
                ));
            }
            
            // Check if this mention matches a known user
            let is_valid_mention = users.iter().any(|u| {
                u.name.eq_ignore_ascii_case(mention_name)
            });
            
            // Add highlighted mention (clone to own the data)
            let mention_text = full_match.as_str().to_string();
            if is_valid_mention {
                spans.push(Span::styled(
                    mention_text,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ));
            } else {
                // Not a valid mention, render as normal text
                spans.push(Span::styled(
                    mention_text,
                    styling::normal_text_style(),
                ));
            }
            
            last_end = full_match.end();
        }
    }
    
    // Add remaining text (clone to own the data)
    if last_end < line.len() {
        spans.push(Span::styled(
            line[last_end..].to_string(),
            styling::normal_text_style(),
        ));
    }
    
    if spans.is_empty() {
        Spans::from(Span::styled(line.to_string(), styling::normal_text_style()))
    } else {
        Spans::from(spans)
    }
}

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
        let is_comment_input = state.is_comment_input_mode();
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
        // If comment input mode is active, always show Comments panel
        let panel_to_show = if state.is_comment_input_mode() {
            TaskDetailPanel::Comments
        } else {
            current_panel
        };
        
        match panel_to_show {
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

        // Create all comment lines as Spans (simpler, better performance)
        // Comments are already in chronological order (oldest first), so newest is at the end
        let mut all_lines: Vec<Spans> = Vec::new();
        
        for story in comments.iter() {
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

            // Parse mentions in comment text (profile URLs like https://app.asana.com/0/profile/...)
            let parsed_text = parse_mentions(&story.text, &state.get_workspace_users());
            
            // Wrap comment text to fit available width
            let wrapped_lines = wrap_text(&parsed_text, available_width);
            
            // Create Spans for each wrapped line (with mention highlighting)
            let text_lines: Vec<Spans> = wrapped_lines
                .iter()
                .map(|line| {
                    parse_line_with_mentions(line, &state.get_workspace_users())
                })
                .collect();

            all_lines.push(header);
            all_lines.push(Spans::from("")); // Add blank line after header
            all_lines.extend(text_lines);
            all_lines.push(Spans::from("")); // Add blank line after comment
        }

        // Use Paragraph instead of List for better performance - show all comments
        // Scroll to bottom to show newest comments (which are at the end)
        let total_lines = all_lines.len();
        let visible_height = chunks[0].height.saturating_sub(2) as usize; // Account for borders
        let scroll_offset = if total_lines > visible_height {
            (total_lines - visible_height) as u16
        } else {
            0
        };

        let paragraph = Paragraph::new(Text::from(all_lines))
            .block(block)
            .wrap(Wrap { trim: true })
            .scroll((scroll_offset, 0)); // Scroll to bottom (newest)
        
        frame.render_widget(paragraph, chunks[0]);
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
