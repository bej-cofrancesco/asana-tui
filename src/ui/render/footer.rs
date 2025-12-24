use super::Frame;
use crate::state::State;
use crate::ui::color::*;
use tui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
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
    } else if *state.current_focus() == crate::state::Focus::View {
        match state.current_view() {
            crate::state::View::TaskDetail => {
                " h/l: switch panel, j/k: scroll, e: edit, d: delete, c: comment, Esc: back, q: quit"
            }
            crate::state::View::CreateTask | crate::state::View::EditTask => {
                " Tab/Shift+Tab: navigate fields, Enter: save, Esc: cancel"
            }
            crate::state::View::ProjectTasks => {
                if state.get_view_mode() == crate::state::ViewMode::Kanban {
                    " j/k: navigate tasks, h/l: navigate columns, Enter: view, n: create, m: move, v: list view, Esc: back, q: quit"
                } else {
                    " j/k: navigate, Enter: view, n: create, space/x: toggle, d: delete, f: filter, v: kanban, /: search, Esc: back, q: quit"
                }
            }
            _ => {
                "j k h l: navigate, s: add/remove shortcut, /: search, d: debug mode, enter: select, esc: cancel, q: quit"
            }
        }
    } else {
        " j k h l: navigate, s: add/remove shortcut, /: search, d: debug mode, enter: select, esc: cancel, q: quit"
    };

    let controls_content = if state.is_search_mode() {
        // Show search mode indicator with different styling
        Spans::from(vec![
            Span::styled(
                "SEARCH:",
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(controls_text, Style::default().fg(YELLOW)),
        ])
    } else if state.is_debug_mode() {
        // Show debug mode indicator with different styling
        Spans::from(vec![
            Span::styled(
                "DEBUG:",
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(controls_text, Style::default().fg(YELLOW)),
        ])
    } else if state.has_delete_confirmation() {
        Spans::from(vec![
            Span::styled(
                "DELETE:",
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Red)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(controls_text, Style::default().fg(YELLOW)),
        ])
    } else if *state.current_focus() == crate::state::Focus::View
        && matches!(state.current_view(), crate::state::View::ProjectTasks)
    {
        Spans::from(vec![
            Span::styled(
                "TASKS:",
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(controls_text, Style::default().fg(YELLOW)),
        ])
    } else if matches!(state.current_view(), crate::state::View::TaskDetail) {
        Spans::from(vec![
            Span::styled(
                "TASK:",
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(controls_text, Style::default().fg(YELLOW)),
        ])
    } else {
        Spans::from(vec![
            Span::styled(
                "NORMAL:",
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(controls_text, Style::default().fg(YELLOW)),
        ])
    };

    let controls_widget = Paragraph::new(controls_content).alignment(Alignment::Left);

    let version_content = Spans::from(vec![Span::styled(
        format!(" {}", env!("CARGO_PKG_VERSION")),
        Style::default().fg(GREEN),
    )]);
    let version_content_width = version_content.width();
    let version_widget = Paragraph::new(version_content).alignment(Alignment::Right);

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(version_content_width.try_into().unwrap()),
        ])
        .split(size);

    frame.render_widget(controls_widget, columns[0]);
    frame.render_widget(version_widget, columns[1]);
}
