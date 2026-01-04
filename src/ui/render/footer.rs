use super::Frame;
use crate::config::hotkeys::{build_footer_text, format_hotkey_display, HotkeyAction};
use crate::state::State;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

/// Format hotkeys for the current view as a display string.
///
fn format_hotkeys_for_view(view: &crate::state::View, state: &State) -> String {
    let hotkeys = state.get_hotkeys();
    let view_hotkeys = match view {
        crate::state::View::Welcome => &hotkeys.welcome,
        crate::state::View::ProjectTasks => &hotkeys.project_tasks,
        crate::state::View::TaskDetail => &hotkeys.task_detail,
        crate::state::View::CreateTask => &hotkeys.create_task,
        crate::state::View::EditTask => &hotkeys.edit_task,
    };

    match view {
        crate::state::View::TaskDetail => build_footer_text(
            view_hotkeys,
            &[
                (
                    HotkeyAction::NavigateLeft,
                    "switch panel",
                    Some(HotkeyAction::NavigateRight),
                ),
                (
                    HotkeyAction::NavigateNext,
                    "scroll",
                    Some(HotkeyAction::NavigatePrev),
                ),
                (HotkeyAction::EditTask, "edit", None),
                (HotkeyAction::DeleteTask, "delete", None),
                (HotkeyAction::AddComment, "comment", None),
                (HotkeyAction::Back, "back", None),
                (HotkeyAction::Quit, "quit", None),
            ],
        ),
        crate::state::View::CreateTask | crate::state::View::EditTask => {
            if state.is_field_editing_mode() {
                " Type to edit, Esc: back to navigation, Enter: select (dropdowns)".to_string()
            } else {
                build_footer_text(
                    view_hotkeys,
                    &[
                        (
                            HotkeyAction::NavigateNext,
                            "navigate fields",
                            Some(HotkeyAction::NavigatePrev),
                        ),
                        (HotkeyAction::EditField, "edit field", None),
                        (HotkeyAction::SubmitForm, "submit", None),
                        (HotkeyAction::Cancel, "cancel", None),
                    ],
                )
            }
        }
        crate::state::View::ProjectTasks => build_footer_text(
            view_hotkeys,
            &[
                (
                    HotkeyAction::NavigateNext,
                    "navigate tasks",
                    Some(HotkeyAction::NavigatePrev),
                ),
                (
                    HotkeyAction::NavigateLeft,
                    "navigate columns (auto-scroll)",
                    Some(HotkeyAction::NavigateRight),
                ),
                (HotkeyAction::ViewTask, "view", None),
                (HotkeyAction::CreateTask, "create", None),
                (HotkeyAction::MoveTask, "move", None),
                (HotkeyAction::FilterByAssignee, "filter by assignee", None),
                (HotkeyAction::EnterSearch, "search", None),
                (HotkeyAction::Back, "back", None),
                (HotkeyAction::Quit, "quit", None),
            ],
        ),
        crate::state::View::Welcome => {
            // For Welcome view, we need special handling for the 4-key navigation display
            let mut parts = Vec::new();
            if let Some(j) = view_hotkeys.get(&HotkeyAction::NavigateNext) {
                if let Some(k) = view_hotkeys.get(&HotkeyAction::NavigatePrev) {
                    if let Some(h) = view_hotkeys.get(&HotkeyAction::NavigateLeft) {
                        if let Some(l) = view_hotkeys.get(&HotkeyAction::NavigateRight) {
                            parts.push(format!(
                                "{} {} {} {}: navigate",
                                format_hotkey_display(j),
                                format_hotkey_display(k),
                                format_hotkey_display(h),
                                format_hotkey_display(l)
                            ));
                        }
                    }
                }
            }
            // Add other actions using build_footer_text
            let other_actions = build_footer_text(
                view_hotkeys,
                &[
                    (HotkeyAction::ToggleStar, "add/remove shortcut", None),
                    (HotkeyAction::EnterSearch, "search", None),
                    (HotkeyAction::EnterDebug, "debug mode", None),
                    (HotkeyAction::OpenThemeSelector, "themes", None),
                    (HotkeyAction::OpenHotkeyEditor, "hotkeys", None),
                    (HotkeyAction::Select, "select", None),
                    (HotkeyAction::Cancel, "cancel", None),
                    (HotkeyAction::Quit, "quit", None),
                ],
            );
            if !other_actions.is_empty() {
                parts.push(other_actions);
            }
            if parts.is_empty() {
                String::new()
            } else {
                format!(" {}", parts.join(","))
            }
        }
    }
}

/// Render footer widget.
///
pub fn footer(frame: &mut Frame, size: Rect, state: &State) {
    let hotkeys = state.get_hotkeys();
    let controls_text = if state.is_search_mode() {
        format!(
            " Type to search,{}",
            build_footer_text(
                &hotkeys.search_mode,
                &[(
                    HotkeyAction::SearchModeExit,
                    "exit search",
                    Some(HotkeyAction::Cancel)
                )]
            )
        )
    } else if state.is_debug_mode() {
        build_footer_text(
            &hotkeys.debug_mode,
            &[
                (
                    HotkeyAction::NavigateNext,
                    "navigate logs",
                    Some(HotkeyAction::NavigatePrev),
                ),
                (HotkeyAction::DebugModeCopyLog, "copy log", None),
                (
                    HotkeyAction::DebugModeExit,
                    "exit debug mode",
                    Some(HotkeyAction::Cancel),
                ),
            ],
        )
    } else if state.has_delete_confirmation() {
        build_footer_text(
            &hotkeys.delete_confirmation,
            &[
                (HotkeyAction::DeleteConfirm, "confirm delete", None),
                (HotkeyAction::Cancel, "cancel", None),
            ],
        )
    } else if state.has_move_task() {
        build_footer_text(
            &hotkeys.move_task,
            &[
                (
                    HotkeyAction::NavigateNext,
                    "navigate sections",
                    Some(HotkeyAction::NavigatePrev),
                ),
                (HotkeyAction::MoveTaskConfirm, "move task", None),
                (HotkeyAction::MoveTaskCancel, "cancel", None),
            ],
        )
    } else if state.has_assignee_filter() {
        format!(
            " Type to search, ↑↓: navigate, {}",
            build_footer_text(
                &hotkeys.assignee_filter,
                &[
                    (HotkeyAction::AssigneeFilterSelect, "select", None),
                    (HotkeyAction::AssigneeFilterCancel, "cancel", None),
                ],
            )
        )
    } else if state.is_theme_mode() {
        build_footer_text(
            &hotkeys.theme_selector,
            &[
                (
                    HotkeyAction::NavigateNext,
                    "navigate themes",
                    Some(HotkeyAction::NavigatePrev),
                ),
                (HotkeyAction::ThemeSelectorSelect, "select theme", None),
                (HotkeyAction::ThemeSelectorCancel, "cancel", None),
            ],
        )
    } else if *state.current_focus() == crate::state::Focus::View {
        format_hotkeys_for_view(state.current_view(), state)
    } else {
        format_hotkeys_for_view(&crate::state::View::Welcome, state)
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
            Span::styled(
                controls_text.as_str(),
                Style::default().fg(theme.warning.to_color()),
            ),
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
            Span::styled(
                controls_text.as_str(),
                Style::default().fg(theme.warning.to_color()),
            ),
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
            Span::styled(
                controls_text.as_str(),
                Style::default().fg(theme.warning.to_color()),
            ),
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
            Span::styled(
                controls_text.as_str(),
                Style::default().fg(theme.warning.to_color()),
            ),
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
            Span::styled(
                controls_text.as_str(),
                Style::default().fg(theme.warning.to_color()),
            ),
        ])
    } else if state.has_assignee_filter() {
        Line::from(vec![
            Span::styled(
                "FILTER:",
                Style::default()
                    .fg(theme.text.to_color())
                    .bg(theme.footer_move.to_color())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                controls_text.as_str(),
                Style::default().fg(theme.warning.to_color()),
            ),
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
            Span::styled(
                controls_text.as_str(),
                Style::default().fg(theme.warning.to_color()),
            ),
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
            Span::styled(
                controls_text.as_str(),
                Style::default().fg(theme.warning.to_color()),
            ),
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
            Span::styled(
                controls_text.as_str(),
                Style::default().fg(theme.warning.to_color()),
            ),
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
            Span::styled(
                controls_text.as_str(),
                Style::default().fg(theme.warning.to_color()),
            ),
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
            Constraint::Length(right_content_width.try_into().unwrap_or(0)),
        ])
        .split(size);

    frame.render_widget(controls_widget, columns[0]);
    frame.render_widget(right_widget, columns[1]);
}
