use super::Frame;
use crate::state::State;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

/// Render footer widget.
///
pub fn footer(frame: &mut Frame, size: Rect, state: &State) {
    let controls_text = if state.is_search_mode() {
        " Type to search, / or Esc: exit search"
    } else if state.is_debug_mode() {
        " j/k: navigate logs, y: copy log, / or Esc: exit debug mode"
    } else if state.has_delete_confirmation() {
        " Enter: confirm delete, Esc: cancel"
    } else if state.has_move_task() {
        " j/k: navigate sections, Enter: move task, Esc: cancel"
    } else if state.is_theme_mode() {
        " j/k: navigate themes, Enter: select theme, Esc: cancel"
    } else if *state.current_focus() == crate::state::Focus::View {
        match state.current_view() {
            crate::state::View::TaskDetail => {
                " h/l: switch panel, j/k: scroll, e: edit, d: delete, c: comment, Esc: back, q: quit"
            }
            crate::state::View::CreateTask | crate::state::View::EditTask => {
                if state.is_field_editing_mode() {
                    // When actively editing a field
                    " Type to edit, Esc: back to navigation, Enter: select (dropdowns)"
                } else {
                    // When navigating between fields
                    " j/k: navigate fields, Enter: edit field, s: submit, Esc: cancel"
                }
            }
            crate::state::View::ProjectTasks => {
                " j/k: navigate tasks, h/l: navigate columns (auto-scroll), Enter: view, n: create, m: move, /: search, Esc: back, q: quit"
            }
            _ => {
                "j k h l: navigate, s: add/remove shortcut, /: search, d: debug mode, enter: select, esc: cancel, q: quit"
            }
        }
    } else {
        " j k h l: navigate, s: add/remove shortcut, /: search, d: debug mode, enter: select, esc: cancel, q: quit"
    };

    let theme = state.get_theme();
    let controls_content = if state.is_search_mode() {
        // Show search mode indicator with different styling
        Line::from(vec![
            Span::styled(
                "SEARCH:",
                Style::default()
                    .fg(theme.text.to_color())
                    .bg(theme.footer_search.to_color())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(controls_text, Style::default().fg(theme.warning.to_color())),
        ])
    } else if state.is_theme_mode() {
        // Show theme mode indicator with different styling
        Line::from(vec![
            Span::styled(
                "THEME:",
                Style::default()
                    .fg(theme.text.to_color())
                    .bg(theme.footer_edit.to_color()) // Use edit color for theme mode
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(controls_text, Style::default().fg(theme.warning.to_color())),
        ])
    } else if state.is_debug_mode() {
        // Show debug mode indicator with different styling
        Line::from(vec![
            Span::styled(
                "DEBUG:",
                Style::default()
                    .fg(theme.text.to_color())
                    .bg(theme.footer_debug.to_color())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(controls_text, Style::default().fg(theme.warning.to_color())),
        ])
    } else if state.has_delete_confirmation() {
        Line::from(vec![
            Span::styled(
                "DELETE:",
                Style::default()
                    .fg(theme.text.to_color())
                    .bg(theme.footer_delete.to_color())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(controls_text, Style::default().fg(theme.warning.to_color())),
        ])
    } else if state.has_move_task() {
        Line::from(vec![
            Span::styled(
                "MOVE:",
                Style::default()
                    .fg(theme.text.to_color())
                    .bg(theme.footer_move.to_color())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(controls_text, Style::default().fg(theme.warning.to_color())),
        ])
    } else if matches!(
        state.current_view(),
        crate::state::View::CreateTask | crate::state::View::EditTask
    ) && state.is_field_editing_mode()
    {
        // Show EDIT mode indicator
        Line::from(vec![
            Span::styled(
                "EDIT:",
                Style::default()
                    .fg(theme.text.to_color())
                    .bg(theme.footer_edit.to_color())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(controls_text, Style::default().fg(theme.warning.to_color())),
        ])
    } else if *state.current_focus() == crate::state::Focus::View
        && matches!(state.current_view(), crate::state::View::ProjectTasks)
    {
        Line::from(vec![
            Span::styled(
                "TASKS:",
                Style::default()
                    .fg(theme.text.to_color())
                    .bg(theme.footer_tasks.to_color())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(controls_text, Style::default().fg(theme.warning.to_color())),
        ])
    } else if matches!(state.current_view(), crate::state::View::TaskDetail) {
        Line::from(vec![
            Span::styled(
                "TASK:",
                Style::default()
                    .fg(theme.text.to_color())
                    .bg(theme.footer_task.to_color())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(controls_text, Style::default().fg(theme.warning.to_color())),
        ])
    } else {
        Line::from(vec![
            Span::styled(
                "NORMAL:",
                Style::default()
                    .fg(theme.text.to_color())
                    .bg(theme.footer_normal.to_color())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(controls_text, Style::default().fg(theme.warning.to_color())),
        ])
    };

    let controls_widget = Paragraph::new(controls_content).alignment(Alignment::Left);

    // Show search query in footer when searching tasks, otherwise show version
    let right_content = if state.is_search_mode()
        && matches!(
            state.get_search_target(),
            Some(crate::state::SearchTarget::Tasks)
        ) {
        // Show search query
        let search_text = if state.get_search_query().is_empty() {
            "/".to_string()
        } else {
            format!("/{}", state.get_search_query())
        };
        Line::from(vec![Span::styled(
            search_text,
            Style::default()
                .fg(theme.text.to_color())
                .bg(theme.footer_search.to_color())
                .add_modifier(Modifier::BOLD),
        )])
    } else if !state.get_search_query().is_empty()
        && matches!(
            state.get_search_target(),
            Some(crate::state::SearchTarget::Tasks)
        )
    {
        // Show query even if not in search mode (after exiting search)
        Line::from(vec![Span::styled(
            format!("/{}", state.get_search_query()),
            Style::default().fg(theme.text_muted.to_color()),
        )])
    } else {
        // Show version number
        Line::from(vec![Span::styled(
            format!(" {}", env!("CARGO_PKG_VERSION")),
            Style::default().fg(theme.secondary.to_color()),
        )])
    };

    let right_content_width = right_content.width();
    let right_widget = Paragraph::new(right_content).alignment(Alignment::Right);

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(right_content_width.try_into().unwrap()),
        ])
        .split(size);

    frame.render_widget(controls_widget, columns[0]);
    frame.render_widget(right_widget, columns[1]);
}
