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

    // Always show exactly 3 section columns + 1 detail column
    // Calculate widths: 3 columns get 75% total, detail gets 25%
    // Each column gets 25% of total width
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25), // Column 1
            Constraint::Percentage(25), // Column 2
            Constraint::Percentage(25), // Column 3
            Constraint::Percentage(25), // Details
        ])
        .split(size);

    // Render kanban columns (first 3 chunks)
    render_kanban_columns(frame, &chunks[0..3], state);
    
    // Render task details on the right (last chunk)
    render_kanban_details(frame, chunks[3], state);
}

fn render_kanban_columns(frame: &mut Frame, column_chunks: &[Rect], state: &State) {
    let sections = state.get_sections();
    let tasks = state.get_filtered_tasks(); // Use filtered tasks
    let current_column = state.get_kanban_column_index();
    let current_task_index = state.get_kanban_task_index();
    
    // Get visible section indices (sections with tasks after filtering)
    let visible_indices = state.get_visible_section_indices();
    
    if visible_indices.is_empty() {
        // Show empty message in first column
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Kanban Board");
        let text = Paragraph::new("No sections to display")
            .block(block)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(text, column_chunks[0]);
        return;
    }
    
    // Find current position in visible indices to determine which columns to show
    let current_pos = visible_indices.iter()
        .position(|&idx| idx == current_column)
        .unwrap_or(0);
    
    // Always show 3 columns, centered around current selection
    // Calculate which 3 sections to show
    let num_visible = visible_indices.len();
    let num_to_show = column_chunks.len().min(num_visible);
    
    // Determine start position: try to center current column, but adjust if near edges
    let start_pos = if num_visible <= num_to_show {
        0
    } else if current_pos < num_to_show / 2 {
        0
    } else if current_pos >= num_visible - num_to_show / 2 {
        num_visible - num_to_show
    } else {
        current_pos - num_to_show / 2
    };
    
    // Get the sections to display
    let sections_to_display: Vec<_> = visible_indices.iter()
        .skip(start_pos)
        .take(num_to_show)
        .enumerate()
        .filter_map(|(display_idx, &section_idx)| {
            sections.get(section_idx).map(|s| (display_idx, section_idx, s))
        })
        .collect();
    
    // Render each section column
    for ((_display_idx, section_idx, section), chunk) in sections_to_display.iter().zip(column_chunks.iter()) {
        let section_tasks: Vec<&crate::asana::Task> = tasks
            .iter()
            .filter(|t| {
                t.section
                    .as_ref()
                    .map(|s| s.gid == section.gid)
                    .unwrap_or(false)
            })
            .collect();

        // Check if this is the currently selected column (using original section index)
        let is_selected = *section_idx == current_column;

        render_kanban_column(
            frame,
            *chunk,
            section,
            &section_tasks,
            is_selected,
            if is_selected { Some(current_task_index) } else { None },
        );
    }
    
    // If we have fewer sections than columns, render empty columns
    for chunk in column_chunks.iter().skip(sections_to_display.len()) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("")
            .border_style(Style::default().fg(Color::DarkGray));
        let text = Paragraph::new("")
            .block(block)
            .alignment(Alignment::Center);
        frame.render_widget(text, *chunk);
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

    // Calculate available width for task names (accounting for borders and padding)
    // Column width is 35, minus 2 for borders, minus some padding = ~30 characters
    let available_width = size.width.saturating_sub(4) as usize; // Account for borders and padding
    
    let items: Vec<ListItem> = tasks
        .iter()
        .enumerate()
        .map(|(idx, task)| {
            // Task name (bold if selected)
            let name_style = if is_selected && selected_task_index == Some(idx) {
                styling::active_list_item_style()
            } else {
                styling::normal_text_style()
            };
            
            // Build the full text with all indicators
            let mut full_text = task.name.clone();
            
            // Add assignee indicator
            if let Some(ref assignee) = task.assignee {
                full_text.push_str(&format!(" (@{})", assignee.name));
            }

            // Add due date if present
            if let Some(ref due_on) = task.due_on {
                full_text.push_str(&format!(" [{}]", due_on));
            }

            // Add completion indicator
            if task.completed {
                full_text.push_str(" ✓");
            }
            
            // Split text into multiple lines that fit within available width
            let mut lines: Vec<Spans> = vec![];
            let words: Vec<String> = full_text.split_whitespace().map(|s| s.to_string()).collect();
            let mut current_line = vec![];
            let mut current_line_len = 0;
            
            for word in words {
                let word_len = word.chars().count();
                // Add 1 for space (except first word on line)
                let space_len = if current_line.is_empty() { 0 } else { 1 };
                
                if current_line_len + space_len + word_len <= available_width {
                    // Word fits on current line
                    if !current_line.is_empty() {
                        current_line.push(Span::raw(" "));
                    }
                    current_line.push(Span::styled(word.clone(), name_style));
                    current_line_len += space_len + word_len;
                } else {
                    // Word doesn't fit, start new line
                    if !current_line.is_empty() {
                        lines.push(Spans::from(current_line));
                    }
                    current_line = vec![Span::styled(word.clone(), name_style)];
                    current_line_len = word_len;
                }
            }
            
            // Add the last line if it's not empty
            if !current_line.is_empty() {
                lines.push(Spans::from(current_line));
            }
            
            // If no lines were created (empty task name), create at least one line
            if lines.is_empty() {
                lines.push(Spans::from(vec![Span::styled("", name_style)]));
            }
            
            ListItem::new(lines)
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
    let tasks = state.get_filtered_tasks(); // Use filtered tasks to respect search/filter
    let current_column = state.get_kanban_column_index();
    let current_task_index = state.get_kanban_task_index();

    // Validate current column index against visible sections to prevent crashes
    let visible_indices = state.get_visible_section_indices();
    if visible_indices.is_empty() || !visible_indices.contains(&current_column) {
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
