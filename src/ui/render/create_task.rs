use super::form_dropdowns;
use super::Frame;
use crate::state::{EditFormState, State};
use crate::ui::widgets::styling;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

/// Render task creation form.
///
pub fn create_task(frame: &mut Frame, size: Rect, state: &mut State) {
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
        .title("Create New Task");
    let title = Paragraph::new("Create New Task")
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

    let is_editing = state.is_field_editing_mode();

    // Name field
    render_field(
        frame,
        form_chunks[0],
        "Name",
        state.get_form_name(),
        form_state == EditFormState::Name,
        is_editing && form_state == EditFormState::Name,
    );

    // Notes field - use textarea for multi-line editing
    render_notes_field(
        frame,
        form_chunks[1],
        state,
        form_state == EditFormState::Notes,
        is_editing && form_state == EditFormState::Notes,
    );

    // Assignee field - show dropdown if selected AND editing
    if form_state == EditFormState::Assignee && is_editing {
        form_dropdowns::render_assignee_dropdown(frame, form_chunks[2], state);
    } else {
        let assignee_text = if let Some(assignee_gid) = state.get_form_assignee() {
            state
                .get_workspace_users()
                .iter()
                .find(|u| u.gid == *assignee_gid)
                .map(|u| u.name.as_str())
                .unwrap_or("Unknown")
        } else {
            "None"
        };
        render_field(
            frame,
            form_chunks[2],
            "Assignee (dropdown)",
            assignee_text,
            form_state == EditFormState::Assignee,
            false, // Combo box doesn't show editing state when collapsed
        );
    }

    // Due Date field
    render_field(
        frame,
        form_chunks[3],
        "Due Date (YYYY-MM-DD)",
        state.get_form_due_on(),
        form_state == EditFormState::DueDate,
        is_editing && form_state == EditFormState::DueDate,
    );

    // Section field - show dropdown if selected AND editing
    if form_state == EditFormState::Section && is_editing {
        form_dropdowns::render_section_dropdown(frame, form_chunks[4], state);
    } else {
        let section_text = if let Some(section_gid) = state.get_form_section() {
            state
                .get_sections()
                .iter()
                .find(|s| s.gid == *section_gid)
                .map(|s| s.name.as_str())
                .unwrap_or("Unknown")
        } else {
            "None"
        };
        render_field(
            frame,
            form_chunks[4],
            "Section (dropdown)",
            section_text,
            form_state == EditFormState::Section,
            false, // Combo box doesn't show editing state when collapsed
        );
    }
}

fn render_field(
    frame: &mut Frame,
    size: Rect,
    label: &str,
    value: &str,
    is_selected: bool,
    is_editing: bool,
) {
    // Different styles for navigation vs editing
    let (border_style, title) = if is_editing {
        // EDITING: Yellow border and indicator
        (
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            format!("{} [EDITING]", label),
        )
    } else if is_selected {
        // SELECTED (Navigation mode): Cyan border
        (
            styling::active_block_border_style(), // Cyan
            format!("{} [Press Enter to edit]", label),
        )
    } else {
        // Not selected: Normal border
        (styling::normal_block_border_style(), label.to_string())
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(border_style);

    let display_value = if value.is_empty() {
        if is_editing {
            "Type to enter value..."
        } else {
            "Empty"
        }
    } else {
        value
    };

    let text_style = if is_editing {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else if is_selected {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        styling::normal_text_style()
    };

    let text = if is_editing {
        Line::from(vec![
            Span::styled(display_value, text_style),
            Span::styled(" █", Style::default().fg(Color::Yellow)), // Editing cursor
        ])
    } else if is_selected {
        Line::from(vec![
            Span::styled("▸ ", Style::default().fg(Color::Cyan)), // Navigation indicator
            Span::styled(display_value, text_style),
        ])
    } else {
        Line::from(vec![Span::styled(display_value, text_style)])
    };

    let paragraph = Paragraph::new(text).block(block);
    frame.render_widget(paragraph, size);
}

fn render_notes_field(
    frame: &mut Frame,
    size: Rect,
    state: &mut State,
    is_selected: bool,
    is_editing: bool,
) {
    let (border_style, title) = if is_editing {
        // EDITING: Yellow border and indicator
        (
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            "Notes [EDITING - Esc to exit]",
        )
    } else if is_selected {
        // SELECTED (Navigation mode): Cyan border
        (
            styling::active_block_border_style(), // Cyan
            "Notes [Press Enter to edit]",
        )
    } else {
        // Not selected: Normal border
        (styling::normal_block_border_style(), "Notes")
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(border_style);

    // Get mutable access to textarea from state
    let textarea = state.get_form_notes_textarea();
    
    // Apply block styling
    textarea.set_block(block);
    
    // Render the textarea
    frame.render_widget(textarea.widget(), size);
}


