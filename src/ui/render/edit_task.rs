use super::Frame;
use crate::state::{EditFormState, State};
use crate::ui::widgets::styling;
use tui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
};

/// Render task editing form.
///
pub fn edit_task(frame: &mut Frame, size: Rect, state: &State) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(1),     // Form fields
            Constraint::Length(1), // Footer
        ])
        .split(size);

    let title_block = Block::default()
        .borders(Borders::ALL)
        .title("Edit Task");
    let title = Paragraph::new("Edit Task")
        .block(title_block)
        .alignment(Alignment::Center);
    frame.render_widget(title, chunks[0]);

    let form_state = state.get_edit_form_state().unwrap_or(EditFormState::Name);
    
    // Render form fields
    let form_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Name
            Constraint::Length(5), // Notes
            Constraint::Length(3), // Assignee
            Constraint::Length(3), // Due Date
            Constraint::Length(3), // Section
            Constraint::Min(0),    // Spacer
        ])
        .split(chunks[1]);

    // Name field
    render_field(
        frame,
        form_chunks[0],
        "Name",
        state.get_form_name(),
        form_state == EditFormState::Name,
    );

    // Notes field
    render_field(
        frame,
        form_chunks[1],
        "Notes",
        state.get_form_notes(),
        form_state == EditFormState::Notes,
    );

    // Assignee field
    let assignee_text = if let Some(assignee_gid) = state.get_form_assignee() {
        state
            .get_workspace_users()
            .iter()
            .find(|u| u.gid == *assignee_gid)
            .map(|u| u.name.as_str())
            .unwrap_or("Unknown")
    } else {
        "Unassigned (press Enter to select)"
    };
    render_field(
        frame,
        form_chunks[2],
        "Assignee",
        assignee_text,
        form_state == EditFormState::Assignee,
    );

    // Due Date field
    render_field(
        frame,
        form_chunks[3],
        "Due Date (YYYY-MM-DD)",
        state.get_form_due_on(),
        form_state == EditFormState::DueDate,
    );

    // Section field
    let section_text = if let Some(section_gid) = state.get_form_section() {
        state
            .get_sections()
            .iter()
            .find(|s| s.gid == *section_gid)
            .map(|s| s.name.as_str())
            .unwrap_or("Unknown")
    } else {
        "None (press Enter to select)"
    };
    render_field(
        frame,
        form_chunks[4],
        "Section",
        section_text,
        form_state == EditFormState::Section,
    );
}

fn render_field(
    frame: &mut Frame,
    size: Rect,
    label: &str,
    value: &str,
    is_selected: bool,
) {
    let border_style = if is_selected {
        styling::active_block_border_style()
    } else {
        styling::normal_block_border_style()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(label)
        .border_style(border_style);

    let display_value = if value.is_empty() {
        "Enter value..."
    } else {
        value
    };

    let text_style = if is_selected {
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    } else {
        styling::normal_text_style()
    };

    let text = if is_selected {
        Spans::from(vec![
            Span::styled(display_value, text_style),
            Span::styled("█", Style::default().fg(Color::Cyan)), // Cursor
        ])
    } else {
        Spans::from(vec![Span::styled(display_value, text_style)])
    };

    let paragraph = Paragraph::new(text).block(block);
    frame.render_widget(paragraph, size);
}
