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
    } else if *state.current_focus() == crate::state::Focus::View 
        && matches!(state.current_view(), crate::state::View::ProjectTasks) {
        "j/k: navigate, space/x: toggle complete, d: delete, f: filter, /: search, esc: refresh & back, q: quit"
    } else {
        "j k h l: navigate, s: add/remove shortcut, /: search, d: debug mode, enter: select, esc: cancel, q: quit"
    };

    let controls_content = if state.is_search_mode() {
        // Show search mode indicator with different styling
        Spans::from(vec![
            Span::styled(
                "SEARCH MODE:",
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
                "DEBUG MODE:",
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(controls_text, Style::default().fg(YELLOW)),
        ])
    } else {
        Spans::from(vec![Span::styled(
            controls_text,
            Style::default().fg(YELLOW),
        )])
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
