use super::Frame;
use crate::state::{State, TaskDetailPanel};
use crate::ui::widgets::styling;
use chrono::DateTime;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
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

        let theme = state.get_theme();
        let header = Block::default()
            .borders(Borders::ALL)
            .title(format!(
                "Task Details - h/l: switch panel | {}",
                panel_indicators
            ))
            .border_style(styling::active_block_border_style(theme));

        let name_text = Line::from(vec![Span::styled(
            &task.name,
            Style::default()
                .fg(theme.text.to_color())
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
        use super::widgets::spinner;
        frame.render_widget(spinner::widget(state, size.height).block(block), size);
    }
}

fn render_task_properties(frame: &mut Frame, size: Rect, task: &crate::asana::Task, state: &State) {
    let theme = state.get_theme();
    let is_active = state.get_current_task_panel() == TaskDetailPanel::Details;

    let mut lines = vec![];

    // Assignee
    if let Some(ref assignee) = task.assignee {
        lines.push(Line::from(vec![
            Span::styled("Assignee: ", Style::default().fg(theme.warning.to_color())),
            Span::styled(&assignee.name, styling::normal_text_style(theme)),
        ]));
    } else {
        lines.push(Line::from(vec![
            Span::styled("Assignee: ", Style::default().fg(theme.warning.to_color())),
            Span::styled(
                "Unassigned",
                Style::default().fg(theme.text_muted.to_color()),
            ),
        ]));
    }

    // Due date
    if let Some(ref due_on) = task.due_on {
        lines.push(Line::from(vec![
            Span::styled("Due: ", Style::default().fg(theme.warning.to_color())),
            Span::styled(due_on, styling::normal_text_style(theme)),
        ]));
    }

    // Section
    if let Some(ref section) = task.section {
        lines.push(Line::from(vec![
            Span::styled("Section: ", Style::default().fg(theme.warning.to_color())),
            Span::styled(&section.name, styling::normal_text_style(theme)),
        ]));
    }

    // Tags
    if !task.tags.is_empty() {
        let tag_names: Vec<String> = task.tags.iter().map(|t| t.name.clone()).collect();
        lines.push(Line::from(vec![
            Span::styled("Tags: ", Style::default().fg(theme.warning.to_color())),
            Span::styled(tag_names.join(", "), styling::normal_text_style(theme)),
        ]));
    }

    // Status
    let status_text = if task.completed {
        "Completed"
    } else {
        "Incomplete"
    };
    lines.push(Line::from(vec![
        Span::styled("Status: ", Style::default().fg(theme.warning.to_color())),
        Span::styled(
            status_text,
            if task.completed {
                Style::default().fg(theme.success.to_color())
            } else {
                Style::default().fg(theme.error.to_color())
            },
        ),
    ]));

    // Subtasks and comments count
    lines.push(Line::from(vec![
        Span::styled("Subtasks: ", Style::default().fg(theme.warning.to_color())),
        Span::styled(
            task.num_subtasks.to_string(),
            styling::normal_text_style(theme),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Comments: ", Style::default().fg(theme.warning.to_color())),
        Span::styled(
            task.num_comments.to_string(),
            styling::normal_text_style(theme),
        ),
    ]));

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Properties")
        .border_style(if is_active {
            styling::active_block_border_style(theme)
        } else {
            styling::normal_block_border_style(theme)
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
    // Calculate dynamic height for comment input based on content
    let input_height = if is_comment_input {
        let input_text = state.get_comment_input_text();
        // Calculate available width for wrapping (account for borders: 4 chars total, 2 on each side)
        let available_width = size.width.saturating_sub(4) as usize;

        // Count lines needed for wrapped text (matching the actual wrapping logic)
        let words: Vec<&str> = input_text.split_whitespace().collect();
        if words.is_empty() {
            3 // Minimum height for empty input
        } else {
            let mut lines = 1u16; // Start with 1 line (for "> " prefix)
            let mut current_line_len = 2usize; // "> " is 2 chars on first line

            for word in words {
                let word_len = word.chars().count();
                let space_len = if current_line_len > 2 { 1 } else { 0 }; // Don't add space after "> "

                if current_line_len + space_len + word_len <= available_width {
                    current_line_len += space_len + word_len;
                } else {
                    lines += 1;
                    current_line_len = 2 + word_len; // 2 for indent "  " on continuation lines
                }
            }

            // Minimum 3 lines, maximum 10 lines for input box
            lines.max(3).min(10)
        }
    } else {
        0
    };

    let chunks = if is_comment_input {
        // Constraint::Length includes borders, so we need input_height + 2
        // (1 for top border with title, input_height for content, 1 for bottom border)
        let total_input_height = input_height + 2;
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(total_input_height)])
            .split(size)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0)])
            .split(size)
    };

    let theme = state.get_theme();
    let title = format!("Comments ({})", comments.len());
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(if is_active || is_comment_input {
            styling::active_block_border_style(theme)
        } else {
            styling::normal_block_border_style(theme)
        });

    if comments.is_empty() && !is_comment_input {
        let text = Paragraph::new("No comments yet. Press 'c' to add a comment.")
            .block(block)
            .alignment(Alignment::Center);
        frame.render_widget(text, chunks[0]);
    } else {
        // Build simple list items with text wrapping
        // Calculate available width for wrapping (account for borders: 2 chars on each side)
        let available_width = chunks[0].width.saturating_sub(4) as usize;

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

                // Header line with author and timestamp
                let mut lines = vec![Line::from(vec![
                    Span::styled(
                        author,
                        Style::default()
                            .fg(theme.warning.to_color())
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(" • ", Style::default().fg(theme.text_muted.to_color())),
                    Span::styled(
                        timestamp_str,
                        Style::default().fg(theme.text_muted.to_color()),
                    ),
                ])];

                // Wrap comment text into multiple lines
                let comment_text = &story.text;
                let words: Vec<&str> = comment_text.split_whitespace().collect();
                let mut current_line = Vec::new();
                let mut current_line_len = 0;

                for word in words {
                    let word_len = word.chars().count();
                    let space_len = if current_line.is_empty() { 0 } else { 1 };

                    if current_line_len + space_len + word_len <= available_width {
                        // Word fits on current line
                        if !current_line.is_empty() {
                            current_line.push(Span::raw(" "));
                        }
                        current_line.push(Span::styled(
                            word,
                            Style::default().fg(theme.text.to_color()),
                        ));
                        current_line_len += space_len + word_len;
                    } else {
                        // Word doesn't fit, start new line
                        if !current_line.is_empty() {
                            lines.push(Line::from(current_line));
                        }
                        current_line = vec![Span::styled(
                            word,
                            Style::default().fg(theme.text.to_color()),
                        )];
                        current_line_len = word_len;
                    }
                }

                // Add the last line if it's not empty
                if !current_line.is_empty() {
                    lines.push(Line::from(current_line));
                }

                // If comment text is empty, add an empty line for the comment body
                if comment_text.trim().is_empty() {
                    lines.push(Line::from(vec![Span::styled(
                        "(empty comment)",
                        Style::default().fg(theme.text_muted.to_color()),
                    )]));
                } else if lines.len() == 1 {
                    // If only header line exists (shouldn't happen with non-empty text, but safety check)
                    lines.push(Line::from(vec![Span::styled(
                        "",
                        Style::default().fg(theme.text.to_color()),
                    )]));
                }

                ListItem::new(lines)
            })
            .collect();

        let list = List::new(items)
            .block(block)
            .style(styling::normal_text_style(theme))
            .highlight_style(
                Style::default()
                    .fg(theme.info.to_color())
                    .add_modifier(Modifier::BOLD),
            );

        frame.render_stateful_widget(list, chunks[0], &mut comments_list_state);
    }

    // Show comment input if in comment input mode
    if is_comment_input {
        let input_block = Block::default()
            .borders(Borders::ALL)
            .title("Add Comment (Enter: submit, Esc: cancel)");

        // Get input text and wrap it manually to ensure proper wrapping
        let input_text = state.get_comment_input_text();
        let available_width = chunks[1].width.saturating_sub(4) as usize; // Account for borders

        // Build wrapped lines with "> " prefix only on first line
        let mut lines = Vec::new();
        if input_text.is_empty() {
            lines.push(Line::from(vec![Span::styled(
                "> ",
                styling::normal_text_style(theme),
            )]));
        } else {
            // Split text into words and wrap
            let words: Vec<&str> = input_text.split_whitespace().collect();
            let mut current_line = vec![Span::styled("> ", styling::normal_text_style(theme))];
            let mut current_line_len = 2; // "> " is 2 chars

            for word in words {
                let word_len = word.chars().count();
                let space_len = if current_line_len > 2 { 1 } else { 0 }; // Don't add space after "> "

                if current_line_len + space_len + word_len <= available_width {
                    if space_len > 0 {
                        current_line.push(Span::raw(" "));
                    }
                    current_line.push(Span::styled(word, styling::normal_text_style(theme)));
                    current_line_len += space_len + word_len;
                } else {
                    // Start new line
                    lines.push(Line::from(current_line));
                    current_line = vec![
                        Span::styled("  ", styling::normal_text_style(theme)), // Indent continuation lines
                        Span::styled(word, styling::normal_text_style(theme)),
                    ];
                    current_line_len = 2 + word_len; // 2 for indent
                }
            }

            // Add the last line
            if !current_line.is_empty() {
                lines.push(Line::from(current_line));
            }
        }

        let input_para = Paragraph::new(lines)
            .block(input_block)
            .style(styling::normal_text_style(theme));
        frame.render_widget(input_para, chunks[1]);
    }
}

fn render_notes(frame: &mut Frame, size: Rect, task: &crate::asana::Task, state: &State) {
    let theme = state.get_theme();
    let is_active = state.get_current_task_panel() == TaskDetailPanel::Notes;

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Notes")
        .border_style(if is_active {
            styling::active_block_border_style(theme)
        } else {
            styling::normal_block_border_style(theme)
        });

    let notes_text = task.notes.as_deref().unwrap_or("No notes");
    let text = Paragraph::new(notes_text)
        .block(block)
        .wrap(Wrap { trim: true })
        .style(styling::normal_text_style(theme));
    frame.render_widget(text, size);
}
