//! Terminal event handling module.
//!
//! This module handles all terminal input events, including keyboard input, mouse events,
//! and user interactions. It processes these events and updates the application state accordingly.

use crate::config::{
    get_action_for_special_mode, hotkeys::get_action_for_event, HotkeyAction, SpecialMode,
};
use crate::state::{Focus, Menu, State};
use anyhow::Result;
use clipboard::{ClipboardContext, ClipboardProvider};
use crossterm::{
    event,
    event::{Event as CrosstermEvent, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
};
use log::*;
use std::{sync::mpsc, thread, time::Duration};
use tui_textarea::Input;

/// Specify terminal event poll rate in milliseconds.
///
const TICK_RATE_IN_MS: u64 = 60;

/// Helper function to check if an event matches a hotkey action and execute it.
/// Returns true if action was handled, false otherwise.
///
fn try_execute_hotkey_action(event: &KeyEvent, state: &mut State) -> Result<Option<bool>> {
    // Don't check hotkeys in special modes - they have their own handling
    if state.is_search_mode()
        || state.is_debug_mode()
        || state.is_field_editing_mode()
        || state.is_comment_input_mode()
        || state.has_delete_confirmation()
        || state.has_theme_selector()
        || state.has_move_task()
        || state.has_assignee_filter()
    {
        return Ok(None);
    }

    if let Some(action) = get_action_for_event(event, state.current_view(), state.get_hotkeys()) {
        match action {
            HotkeyAction::Quit => {
                debug!("Processing exit terminal event (hotkey) '{:?}'...", event);
                return Ok(Some(false));
            }
            HotkeyAction::ToggleTaskComplete => {
                if state.current_focus() == &Focus::View {
                    debug!("Processing toggle task completion event '{:?}'...", event);
                    state.toggle_task_completion();
                    return Ok(Some(true));
                }
            }
            HotkeyAction::DeleteTask => {
                if state.current_focus() == &Focus::View {
                    if matches!(state.current_view(), crate::state::View::TaskDetail) {
                        // Delete task from detail view
                        if let Some(task) = state.get_task_detail() {
                            state.set_delete_confirmation(task.gid.clone());
                        }
                    } else if matches!(state.current_view(), crate::state::View::ProjectTasks) {
                        // Delete task from kanban view
                        if let Some(task) = state.get_kanban_selected_task() {
                            state.set_delete_confirmation(task.gid.clone());
                        }
                    } else {
                        debug!("Processing delete task event '{:?}'...", event);
                        state.delete_selected_task();
                    }
                    return Ok(Some(true));
                }
            }
            HotkeyAction::EditTask => {
                if matches!(state.current_focus(), Focus::View)
                    && matches!(state.current_view(), crate::state::View::TaskDetail)
                {
                    // Edit task from detail view
                    debug!("Processing edit task event '{:?}'...", event);
                    if let Some(task) = state.get_task_detail() {
                        let task_clone = task.clone();
                        state.init_edit_form(&task_clone);

                        // Load workspace users and sections for dropdowns
                        if let Some(workspace) = state.get_active_workspace() {
                            state.dispatch(crate::events::network::Event::GetWorkspaceUsers {
                                workspace_gid: workspace.gid.clone(),
                            });
                        }
                        if let Some(project) = state.get_project() {
                            state.dispatch(crate::events::network::Event::GetProjectSections {
                                project_gid: project.gid.clone(),
                            });
                        }

                        state.push_view(crate::state::View::EditTask);
                        state.focus_view();
                    }
                    return Ok(Some(true));
                }
            }
            HotkeyAction::AddComment => {
                if matches!(state.current_focus(), Focus::View)
                    && matches!(state.current_view(), crate::state::View::TaskDetail)
                {
                    // Add comment from detail view - switch to Comments panel first
                    debug!("Processing add comment event '{:?}'...", event);
                    state.set_current_task_panel(crate::state::TaskDetailPanel::Comments);
                    state.enter_comment_input_mode();
                    return Ok(Some(true));
                }
            }
            HotkeyAction::SubmitForm => {
                if matches!(
                    state.current_view(),
                    crate::state::View::CreateTask | crate::state::View::EditTask
                ) {
                    // Submit form - the actual submission logic is complex and handled
                    // by checking the hotkey action in the main event loop below.
                    // This action will be caught and processed there.
                    return Ok(Some(true));
                }
            }
            HotkeyAction::CreateTask => {
                if !state.is_debug_mode()
                    && state.current_focus() == &Focus::View
                    && matches!(
                        state.current_view(),
                        crate::state::View::ProjectTasks | crate::state::View::TaskDetail
                    )
                {
                    // Enter create task view
                    debug!("Processing create task event '{:?}'...", event);
                    state.clear_form();
                    state.set_edit_form_state(Some(crate::state::EditFormState::Name));
                    // Load workspace users and sections if needed
                    if let Some(workspace) = state.get_active_workspace() {
                        state.dispatch(crate::events::network::Event::GetWorkspaceUsers {
                            workspace_gid: workspace.gid.clone(),
                        });
                    }
                    if let Some(project) = state.get_project() {
                        state.dispatch(crate::events::network::Event::GetProjectSections {
                            project_gid: project.gid.clone(),
                        });
                        state.dispatch(crate::events::network::Event::GetProjectCustomFields {
                            project_gid: project.gid.clone(),
                        });
                    }
                    state.push_view(crate::state::View::CreateTask);
                    state.focus_view();
                    return Ok(Some(true));
                }
            }
            HotkeyAction::MoveTask => {
                if !state.is_debug_mode()
                    && state.current_focus() == &Focus::View
                    && matches!(state.current_view(), crate::state::View::ProjectTasks)
                {
                    // Open section selection modal for moving task
                    if let Some(task) = state.get_kanban_selected_task() {
                        debug!("Opening move task modal for task {}...", task.gid);
                        state.set_move_task_gid(Some(task.gid.clone()));
                    }
                    return Ok(Some(true));
                }
            }
            HotkeyAction::FilterByAssignee => {
                if !state.is_debug_mode()
                    && state.current_focus() == &Focus::View
                    && matches!(state.current_view(), crate::state::View::ProjectTasks)
                    && !state.has_assignee_filter()
                {
                    // Open assignee filter modal
                    debug!("Opening assignee filter modal...");
                    // Ensure workspace users are loaded
                    if let Some(workspace) = state.get_active_workspace() {
                        state.dispatch(crate::events::network::Event::GetWorkspaceUsers {
                            workspace_gid: workspace.gid.clone(),
                        });
                    }
                    state.open_assignee_filter();
                    return Ok(Some(true));
                }
            }
            HotkeyAction::OpenThemeSelector => {
                if !state.is_debug_mode()
                    && !state.has_theme_selector()
                    && !state.has_hotkey_editor()
                    && matches!(state.current_view(), crate::state::View::Welcome)
                {
                    // Open theme selector modal (only available on welcome screen)
                    debug!("Opening theme selector modal...");
                    state.open_theme_selector();
                    return Ok(Some(true));
                }
            }
            HotkeyAction::OpenHotkeyEditor => {
                if !state.is_debug_mode()
                    && !state.has_theme_selector()
                    && !state.has_hotkey_editor()
                    && matches!(state.current_view(), crate::state::View::Welcome)
                {
                    // Open hotkey editor modal (only available from Welcome view)
                    debug!("Opening hotkey editor modal...");
                    state.open_hotkey_editor();
                    return Ok(Some(true));
                }
            }
            HotkeyAction::EnterSearch => {
                if !state.is_search_mode()
                    && !state.is_debug_mode()
                    && !state.is_comment_input_mode()
                {
                    debug!("Processing enter search mode event '{:?}'...", event);
                    state.enter_search_mode();
                    return Ok(Some(true));
                }
            }
            HotkeyAction::EnterDebug => {
                if !state.is_debug_mode()
                    && !state.is_search_mode()
                    && !state.is_comment_input_mode()
                {
                    debug!("Processing enter debug mode event '{:?}'...", event);
                    state.enter_debug_mode();
                    return Ok(Some(true));
                }
            }
            HotkeyAction::Cancel | HotkeyAction::Back => {
                // Esc/back handling - check special states first
                if state.has_delete_confirmation() {
                    debug!(
                        "Processing cancel delete confirmation event '{:?}'...",
                        event
                    );
                    state.cancel_delete_confirmation();
                    return Ok(Some(true));
                }
                if state.has_theme_selector() {
                    debug!("Processing cancel theme selector event '{:?}'...", event);
                    state.close_theme_selector();
                    return Ok(Some(true));
                }
                if state.has_move_task() {
                    debug!("Processing cancel move task event '{:?}'...", event);
                    state.clear_move_task();
                    return Ok(Some(true));
                }
                if state.is_comment_input_mode() {
                    debug!("Processing cancel comment input event '{:?}'...", event);
                    state.exit_comment_input_mode();
                    return Ok(Some(true));
                }
                if *state.current_focus() == Focus::View {
                    debug!("Processing view navigation (Esc) event '{:?}'...", event);
                    if let Some(popped_view) = state.pop_view() {
                        debug!(
                            "Popped view: {:?}, remaining views: {}",
                            popped_view,
                            state.view_stack_len()
                        );
                        match popped_view {
                            crate::state::View::TaskDetail
                            | crate::state::View::EditTask
                            | crate::state::View::CreateTask => {
                                if matches!(state.current_view(), crate::state::View::ProjectTasks)
                                {
                                    state.dispatch(crate::events::network::Event::ProjectTasks);
                                }
                            }
                            crate::state::View::ProjectTasks => {
                                state.focus_menu();
                            }
                            _ => {}
                        }
                        if matches!(state.current_view(), crate::state::View::Welcome) {
                            state.focus_menu();
                        }
                    } else {
                        debug!("No more views to pop, focusing menu");
                        state.focus_menu();
                    }
                    return Ok(Some(true));
                }
            }
            _ => {
                // Other actions will be handled by specific key matches below
                // Return None to continue processing
            }
        }
    }
    Ok(None)
}

/// Specify different terminal event types.
///
#[derive(Debug)]
pub enum Event<I> {
    Input(I),
    Tick,
}

/// Specify struct for managing terminal events channel.
///
pub struct Handler {
    rx: mpsc::Receiver<Event<KeyEvent>>,
    _tx: mpsc::Sender<Event<KeyEvent>>,
}

impl Handler {
    /// Return new instance after spawning new input polling thread.
    ///
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        let tx_clone = tx.clone();
        thread::spawn(move || {
            loop {
                let tick_rate = Duration::from_millis(TICK_RATE_IN_MS);
                if let Ok(ready) = event::poll(tick_rate) {
                    if ready {
                        if let Ok(CrosstermEvent::Key(key)) = event::read() {
                            let _ = tx_clone.send(Event::Input(key));
                        }
                    }
                }
                // Tick events are best-effort; if channel is closed, exit thread
                if tx_clone.send(Event::Tick).is_err() {
                    break;
                }
            }
        });
        Handler { rx, _tx: tx }
    }

    /// Receive next terminal event and handle it accordingly. Returns result
    /// with value true if should continue or false if exit was requested.
    ///
    pub fn handle_next(&self, state: &mut State) -> Result<bool> {
        match self.rx.recv()? {
            Event::Input(event) => {
                // Filter out key release events - only process key press events
                // In crossterm 0.27+, we get both Press and Release events
                if event.kind != KeyEventKind::Press {
                    return Ok(true);
                }

                // Handle field editing mode - when actively editing a field
                if matches!(
                    state.current_view(),
                    crate::state::View::CreateTask | crate::state::View::EditTask
                ) && state.is_field_editing_mode()
                {
                    // In field editing mode, only Escape exits back to navigation
                    match event {
                        KeyEvent {
                            code: KeyCode::Esc, ..
                        } => {
                            state.exit_field_editing_mode();
                            return Ok(true);
                        }
                        _ => {
                            // Route all other keys to the active field
                            match state.get_edit_form_state() {
                                Some(crate::state::EditFormState::Notes) => {
                                    let input: Input = CrosstermEvent::Key(event).into();
                                    state.get_form_notes_textarea().input(input);
                                }
                                Some(crate::state::EditFormState::Name) => {
                                    if let KeyEvent {
                                        code: KeyCode::Char(c),
                                        ..
                                    } = event
                                    {
                                        state.add_form_name_char(c);
                                    } else if matches!(event.code, KeyCode::Backspace) {
                                        state.remove_form_name_char();
                                    }
                                }
                                Some(crate::state::EditFormState::DueDate) => {
                                    if let KeyEvent {
                                        code: KeyCode::Char(c),
                                        ..
                                    } = event
                                    {
                                        state.add_form_due_on_char(c);
                                    } else if matches!(event.code, KeyCode::Backspace) {
                                        state.remove_form_due_on_char();
                                    }
                                }
                                Some(crate::state::EditFormState::Assignee) => {
                                    // Handle assignee dropdown navigation and search
                                    // Support arrow keys for navigation (better UX than j/k)
                                    match event.code {
                                        KeyCode::Up => {
                                            state.previous_assignee();
                                            return Ok(true);
                                        }
                                        KeyCode::Down => {
                                            state.next_assignee();
                                            return Ok(true);
                                        }
                                        _ => {
                                            // All other keys go to text input (handled below)
                                        }
                                    }

                                    // Handle text input for search (characters and backspace)
                                    match event {
                                        KeyEvent {
                                            code: KeyCode::Char(c),
                                            ..
                                        } => {
                                            // Add character to search (unless it was a navigation key above)
                                            state.add_assignee_search_char(c);
                                            return Ok(true);
                                        }
                                        KeyEvent {
                                            code: KeyCode::Backspace,
                                            ..
                                        } => {
                                            state.backspace_assignee_search();
                                            return Ok(true);
                                        }
                                        _ => {}
                                    }
                                }
                                Some(crate::state::EditFormState::Section) => {
                                    // Handle section dropdown navigation and search
                                    // Support arrow keys for navigation (better UX than j/k)
                                    match event.code {
                                        KeyCode::Up => {
                                            state.previous_section();
                                            return Ok(true);
                                        }
                                        KeyCode::Down => {
                                            state.next_section();
                                            return Ok(true);
                                        }
                                        _ => {
                                            // All other keys go to text input (handled below)
                                        }
                                    }

                                    // Handle text input for search (characters and backspace)
                                    match event {
                                        KeyEvent {
                                            code: KeyCode::Char(c),
                                            ..
                                        } => {
                                            // Add character to search (unless it was a navigation key above)
                                            state.add_section_search_char(c);
                                            return Ok(true);
                                        }
                                        KeyEvent {
                                            code: KeyCode::Backspace,
                                            ..
                                        } => {
                                            state.backspace_section_search();
                                            return Ok(true);
                                        }
                                        _ => {}
                                    }
                                }
                                Some(crate::state::EditFormState::CustomField(_idx)) => {
                                    // Handle custom field editing based on field type
                                    // Clone custom field data to avoid borrow checker issues
                                    let (cf_gid, cf_subtype) =
                                        if let Some((_, cf)) = state.get_current_custom_field() {
                                            (cf.gid.clone(), cf.resource_subtype.clone())
                                        } else {
                                            return Ok(true);
                                        };

                                    match cf_subtype.as_str() {
                                        "text" | "number" | "date" => {
                                            // Simple text input for text, number, and date fields
                                            if let KeyEvent {
                                                code: KeyCode::Char(c),
                                                ..
                                            } = event
                                            {
                                                // For number fields, only allow digits, decimal point, and minus sign
                                                if cf_subtype == "number" {
                                                    if c.is_ascii_digit() || c == '.' || c == '-' {
                                                        state.add_custom_field_text_char(
                                                            cf_gid.clone(),
                                                            c,
                                                            &cf_subtype,
                                                        );
                                                    }
                                                } else {
                                                    state.add_custom_field_text_char(
                                                        cf_gid.clone(),
                                                        c,
                                                        &cf_subtype,
                                                    );
                                                }
                                            } else if matches!(event.code, KeyCode::Backspace) {
                                                state.remove_custom_field_text_char(
                                                    &cf_gid,
                                                    &cf_subtype,
                                                );
                                            }
                                        }
                                        "enum" => {
                                            // Handle enum dropdown navigation and search
                                            // Get custom field data first
                                            let (enum_options, search) = if let Some((_, cf)) =
                                                state.get_current_custom_field()
                                            {
                                                (
                                                    cf.enum_options.clone(),
                                                    state
                                                        .get_custom_field_search(&cf_gid)
                                                        .to_string(),
                                                )
                                            } else {
                                                return Ok(true);
                                            };

                                            // Support arrow keys for navigation (better UX than j/k)
                                            match event.code {
                                                KeyCode::Up => {
                                                    let filtered_count = enum_options
                                                        .iter()
                                                        .filter(|eo| {
                                                            eo.enabled
                                                                && (search.is_empty()
                                                                    || eo
                                                                        .name
                                                                        .to_lowercase()
                                                                        .contains(
                                                                            &search.to_lowercase(),
                                                                        ))
                                                        })
                                                        .count();
                                                    state.previous_custom_field_enum(
                                                        &cf_gid,
                                                        filtered_count,
                                                    );
                                                    return Ok(true);
                                                }
                                                KeyCode::Down => {
                                                    let filtered_count = enum_options
                                                        .iter()
                                                        .filter(|eo| {
                                                            eo.enabled
                                                                && (search.is_empty()
                                                                    || eo
                                                                        .name
                                                                        .to_lowercase()
                                                                        .contains(
                                                                            &search.to_lowercase(),
                                                                        ))
                                                        })
                                                        .count();
                                                    state.next_custom_field_enum(
                                                        &cf_gid,
                                                        filtered_count,
                                                    );
                                                    return Ok(true);
                                                }
                                                _ => {
                                                    // All other keys go to text input (handled below)
                                                }
                                            }

                                            // Handle text input for search
                                            match event {
                                                KeyEvent {
                                                    code: KeyCode::Char(c),
                                                    ..
                                                } => {
                                                    state.add_custom_field_search_char(
                                                        cf_gid.clone(),
                                                        c,
                                                    );
                                                    return Ok(true);
                                                }
                                                KeyEvent {
                                                    code: KeyCode::Backspace,
                                                    ..
                                                } => {
                                                    state.backspace_custom_field_search(&cf_gid);
                                                    return Ok(true);
                                                }
                                                _ => {}
                                            }
                                        }
                                        "multi_enum" => {
                                            // Handle multi-enum dropdown navigation and search
                                            let (enum_options, search) = if let Some((_, cf)) =
                                                state.get_current_custom_field()
                                            {
                                                (
                                                    cf.enum_options.clone(),
                                                    state
                                                        .get_custom_field_search(&cf_gid)
                                                        .to_string(),
                                                )
                                            } else {
                                                return Ok(true);
                                            };

                                            // Support arrow keys for navigation (better UX than j/k)
                                            match event.code {
                                                KeyCode::Up => {
                                                    let filtered_count = enum_options
                                                        .iter()
                                                        .filter(|eo| {
                                                            eo.enabled
                                                                && (search.is_empty()
                                                                    || eo
                                                                        .name
                                                                        .to_lowercase()
                                                                        .contains(
                                                                            &search.to_lowercase(),
                                                                        ))
                                                        })
                                                        .count();
                                                    state.previous_custom_field_enum(
                                                        &cf_gid,
                                                        filtered_count,
                                                    );
                                                    return Ok(true);
                                                }
                                                KeyCode::Down => {
                                                    let filtered_count = enum_options
                                                        .iter()
                                                        .filter(|eo| {
                                                            eo.enabled
                                                                && (search.is_empty()
                                                                    || eo
                                                                        .name
                                                                        .to_lowercase()
                                                                        .contains(
                                                                            &search.to_lowercase(),
                                                                        ))
                                                        })
                                                        .count();
                                                    state.next_custom_field_enum(
                                                        &cf_gid,
                                                        filtered_count,
                                                    );
                                                    return Ok(true);
                                                }
                                                _ => {
                                                    // Check for configured hotkeys (j/k) as fallback
                                                    if let Some(action) = get_action_for_event(
                                                        &event,
                                                        state.current_view(),
                                                        state.get_hotkeys(),
                                                    ) {
                                                        match action {
                                                            HotkeyAction::NavigateNext => {
                                                                let filtered_count = enum_options
                                                                    .iter()
                                                                    .filter(|eo| {
                                                                        eo.enabled
                                                                            && (search.is_empty()
                                                                                || eo
                                                                                    .name
                                                                                    .to_lowercase()
                                                                                    .contains(
                                                                                        &search
                                                                                            .to_lowercase(),
                                                                                    ))
                                                                    })
                                                                    .count();
                                                                state.next_custom_field_enum(
                                                                    &cf_gid,
                                                                    filtered_count,
                                                                );
                                                                return Ok(true);
                                                            }
                                                            HotkeyAction::NavigatePrev => {
                                                                let filtered_count = enum_options
                                                                    .iter()
                                                                    .filter(|eo| {
                                                                        eo.enabled
                                                                            && (search.is_empty()
                                                                                || eo
                                                                                    .name
                                                                                    .to_lowercase()
                                                                                    .contains(
                                                                                        &search
                                                                                            .to_lowercase(),
                                                                                    ))
                                                                    })
                                                                    .count();
                                                                state.previous_custom_field_enum(
                                                                    &cf_gid,
                                                                    filtered_count,
                                                                );
                                                                return Ok(true);
                                                            }
                                                            HotkeyAction::Select => {
                                                                // Toggle current enum option
                                                                let filtered: Vec<_> = enum_options
                                                                    .iter()
                                                                    .filter(|eo| {
                                                                        eo.enabled
                                                                            && (search.is_empty()
                                                                                || eo
                                                                                    .name
                                                                                    .to_lowercase()
                                                                                    .contains(
                                                                                        &search
                                                                                            .to_lowercase(),
                                                                                    ))
                                                                    })
                                                                    .collect();
                                                                let current_idx = state
                                                                    .get_custom_field_dropdown_index(
                                                                        &cf_gid,
                                                                    );
                                                                if let Some(selected) = filtered
                                                                    .get(
                                                                        current_idx.min(
                                                                            filtered
                                                                                .len()
                                                                                .saturating_sub(1),
                                                                        ),
                                                                    )
                                                                {
                                                                    state.toggle_custom_field_multi_enum(
                                                                        &cf_gid,
                                                                        selected.gid.clone(),
                                                                    );
                                                                }
                                                                return Ok(true);
                                                            }
                                                            _ => {
                                                                // Not a navigation/select action, continue to text input
                                                            }
                                                        }
                                                    }
                                                }
                                            }

                                            // Handle text input for search
                                            match event {
                                                KeyEvent {
                                                    code: KeyCode::Char(c),
                                                    ..
                                                } => {
                                                    state.add_custom_field_search_char(
                                                        cf_gid.clone(),
                                                        c,
                                                    );
                                                    return Ok(true);
                                                }
                                                KeyEvent {
                                                    code: KeyCode::Backspace,
                                                    ..
                                                } => {
                                                    state.backspace_custom_field_search(&cf_gid);
                                                    return Ok(true);
                                                }
                                                _ => {}
                                            }
                                        }
                                        "people" => {
                                            // Handle people dropdown navigation and search
                                            let (users, search) = {
                                                let users = state.get_workspace_users();
                                                (
                                                    users.to_vec(),
                                                    state
                                                        .get_custom_field_search(&cf_gid)
                                                        .to_string(),
                                                )
                                            };

                                            // Support arrow keys for navigation (better UX than j/k)
                                            match event.code {
                                                KeyCode::Up => {
                                                    let filtered_count = users
                                                        .iter()
                                                        .filter(|u| {
                                                            search.is_empty()
                                                                || u.name.to_lowercase().contains(
                                                                    &search.to_lowercase(),
                                                                )
                                                        })
                                                        .count();
                                                    state.previous_custom_field_enum(
                                                        &cf_gid,
                                                        filtered_count,
                                                    );
                                                    return Ok(true);
                                                }
                                                KeyCode::Down => {
                                                    let filtered_count = users
                                                        .iter()
                                                        .filter(|u| {
                                                            search.is_empty()
                                                                || u.name.to_lowercase().contains(
                                                                    &search.to_lowercase(),
                                                                )
                                                        })
                                                        .count();
                                                    state.next_custom_field_enum(
                                                        &cf_gid,
                                                        filtered_count,
                                                    );
                                                    return Ok(true);
                                                }
                                                _ => {
                                                    // Check for configured hotkeys (j/k) as fallback
                                                    if let Some(action) = get_action_for_event(
                                                        &event,
                                                        state.current_view(),
                                                        state.get_hotkeys(),
                                                    ) {
                                                        match action {
                                                            HotkeyAction::NavigateNext => {
                                                                let filtered_count = users
                                                                    .iter()
                                                                    .filter(|u| {
                                                                        search.is_empty()
                                                                            || u.name
                                                                                .to_lowercase()
                                                                                .contains(
                                                                                    &search.to_lowercase(),
                                                                                )
                                                                    })
                                                                    .count();
                                                                state.next_custom_field_enum(
                                                                    &cf_gid,
                                                                    filtered_count,
                                                                );
                                                                return Ok(true);
                                                            }
                                                            HotkeyAction::NavigatePrev => {
                                                                let filtered_count = users
                                                                    .iter()
                                                                    .filter(|u| {
                                                                        search.is_empty()
                                                                            || u.name
                                                                                .to_lowercase()
                                                                                .contains(
                                                                                    &search.to_lowercase(),
                                                                                )
                                                                    })
                                                                    .count();
                                                                state.previous_custom_field_enum(
                                                                    &cf_gid,
                                                                    filtered_count,
                                                                );
                                                                return Ok(true);
                                                            }
                                                            HotkeyAction::Select => {
                                                                // Toggle current person
                                                                let filtered: Vec<_> = users
                                                                    .iter()
                                                                    .filter(|u| {
                                                                        search.is_empty()
                                                                            || u.name
                                                                                .to_lowercase()
                                                                                .contains(
                                                                                    &search.to_lowercase(),
                                                                                )
                                                                    })
                                                                    .collect();
                                                                let current_idx = state
                                                                    .get_custom_field_dropdown_index(
                                                                        &cf_gid,
                                                                    );
                                                                if let Some(selected) = filtered
                                                                    .get(
                                                                        current_idx.min(
                                                                            filtered
                                                                                .len()
                                                                                .saturating_sub(1),
                                                                        ),
                                                                    )
                                                                {
                                                                    state
                                                                        .toggle_custom_field_people(
                                                                            &cf_gid,
                                                                            selected.gid.clone(),
                                                                        );
                                                                }
                                                                return Ok(true);
                                                            }
                                                            _ => {
                                                                // Not a navigation/select action, continue to text input
                                                            }
                                                        }
                                                    }
                                                }
                                            }

                                            // Handle text input for search
                                            match event {
                                                KeyEvent {
                                                    code: KeyCode::Char(c),
                                                    ..
                                                } => {
                                                    state.add_custom_field_search_char(
                                                        cf_gid.clone(),
                                                        c,
                                                    );
                                                    return Ok(true);
                                                }
                                                KeyEvent {
                                                    code: KeyCode::Backspace,
                                                    ..
                                                } => {
                                                    state.backspace_custom_field_search(&cf_gid);
                                                    return Ok(true);
                                                }
                                                _ => {}
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                                None => {}
                            }
                            return Ok(true);
                        }
                    }
                }
                // Handle hotkey editor key capture FIRST - blocks all other hotkeys
                if state.has_hotkey_editor() {
                    if let Some(action) = state.get_hotkey_editor_selected_action() {
                        // Clone the action to avoid borrow checker issues
                        let action = action.clone();
                        // Capturing a key for rebinding
                        match event {
                            KeyEvent {
                                code: KeyCode::Esc,
                                modifiers: KeyModifiers::NONE,
                                ..
                            } => {
                                // Cancel rebinding
                                state.set_hotkey_editor_selected_action(None);
                                return Ok(true);
                            }
                            _ => {
                                // Bind the key to the action using the new grouped update function
                                use crate::config::hotkeys::{
                                    format_hotkey_display, update_hotkey_for_action, Hotkey,
                                };
                                let hotkey = Hotkey {
                                    code: event.code,
                                    modifiers: event.modifiers,
                                };
                                let mut hotkeys = state.get_hotkeys().clone();
                                update_hotkey_for_action(&mut hotkeys, &action, hotkey.clone());
                                state.set_hotkeys(hotkeys);
                                state.set_hotkey_editor_selected_action(None);
                                debug!(
                                    "Updated hotkey for action {:?} to {}",
                                    action,
                                    format_hotkey_display(&hotkey)
                                );
                                // Trigger config save - config will be saved on next save cycle
                                return Ok(true);
                            }
                        }
                    } else {
                        // Navigating in hotkey editor - use configured hotkeys
                        // Check for navigation actions first
                        if let Some(action) =
                            get_action_for_event(&event, state.current_view(), state.get_hotkeys())
                        {
                            match action {
                                HotkeyAction::NavigateNext => {
                                    state.next_hotkey_action();
                                    return Ok(true);
                                }
                                HotkeyAction::NavigatePrev => {
                                    state.previous_hotkey_action();
                                    return Ok(true);
                                }
                                _ => {}
                            }
                        }
                        // Also check for Enter and Esc (these should always work)
                        match event {
                            KeyEvent {
                                code: KeyCode::Enter,
                                modifiers: KeyModifiers::NONE,
                                ..
                            } => {
                                // Start editing the selected action
                                let index = state.get_hotkey_editor_dropdown_index();
                                if let Some(action) = state.get_hotkey_action_at_index(index) {
                                    state.set_hotkey_editor_selected_action(Some(action));
                                }
                                return Ok(true);
                            }
                            KeyEvent {
                                code: KeyCode::Esc,
                                modifiers: KeyModifiers::NONE,
                                ..
                            } => {
                                // Close hotkey editor
                                state.close_hotkey_editor();
                                return Ok(true);
                            }
                            _ => {
                                // Block all other keys when in hotkey editor
                                return Ok(true);
                            }
                        }
                    }
                }

                // FIRST: Check for global navigation actions - these work everywhere
                // Check both regular views and special modes
                let navigation_action = if state.has_theme_selector() {
                    get_action_for_special_mode(
                        &event,
                        SpecialMode::ThemeSelector,
                        state.get_hotkeys(),
                    )
                } else if state.has_assignee_filter() {
                    get_action_for_special_mode(
                        &event,
                        SpecialMode::AssigneeFilter,
                        state.get_hotkeys(),
                    )
                } else if state.has_move_task() {
                    get_action_for_special_mode(&event, SpecialMode::MoveTask, state.get_hotkeys())
                } else if state.is_debug_mode() {
                    get_action_for_special_mode(&event, SpecialMode::Debug, state.get_hotkeys())
                } else {
                    get_action_for_event(&event, state.current_view(), state.get_hotkeys())
                };

                // Handle global navigation actions if found
                if let Some(action) = navigation_action {
                    match action {
                        HotkeyAction::NavigateNext | HotkeyAction::NavigatePrev => {
                            // Handle navigation - but skip if in text input mode
                            if state.is_search_mode() {
                                // In search mode, allow typing any character
                                if let KeyCode::Char(c) = event.code {
                                    state.add_search_char(c);
                                    return Ok(true);
                                }
                            } else if state.is_comment_input_mode() {
                                // In comment input mode, allow typing any character
                                if let KeyCode::Char(c) = event.code {
                                    state.add_comment_char(c);
                                    return Ok(true);
                                }
                            } else if state.has_assignee_filter() {
                                // In assignee filter mode, allow typing for search
                                if let KeyCode::Char(c) = event.code {
                                    state.add_assignee_filter_search_char(c);
                                    return Ok(true);
                                }
                            } else if matches!(
                                state.current_view(),
                                crate::state::View::CreateTask | crate::state::View::EditTask
                            ) && state.is_field_editing_mode()
                                && !matches!(
                                    state.get_edit_form_state(),
                                    Some(crate::state::EditFormState::Assignee)
                                        | Some(crate::state::EditFormState::Section)
                                )
                            {
                                // In form text fields, allow typing any character
                                if let KeyCode::Char(c) = event.code {
                                    match state.get_edit_form_state() {
                                        Some(crate::state::EditFormState::Name) => {
                                            state.add_form_name_char(c);
                                        }
                                        Some(crate::state::EditFormState::Notes) => {
                                            // Already handled above
                                        }
                                        Some(crate::state::EditFormState::DueDate) => {
                                            state.add_form_due_on_char(c);
                                        }
                                        _ => {}
                                    }
                                    return Ok(true);
                                }
                            } else {
                                // Execute navigation action
                                match action {
                                    HotkeyAction::NavigateNext => {
                                        if state.has_theme_selector() {
                                            state.next_theme();
                                        } else if state.has_assignee_filter() {
                                            state.next_assignee_filter_option();
                                        } else if state.has_move_task() {
                                            state.next_section();
                                        } else if state.is_debug_mode() {
                                            state.next_debug();
                                        } else if matches!(
                                            state.current_view(),
                                            crate::state::View::TaskDetail
                                        ) {
                                            match state.get_current_task_panel() {
                                                crate::state::TaskDetailPanel::Comments => {
                                                    state.scroll_comments_down();
                                                }
                                                crate::state::TaskDetailPanel::Details => {
                                                    state.scroll_details_down();
                                                }
                                                crate::state::TaskDetailPanel::Notes => {
                                                    state.scroll_notes_down();
                                                }
                                            }
                                        } else if matches!(
                                            state.current_view(),
                                            crate::state::View::ProjectTasks
                                        ) {
                                            state.next_kanban_task();
                                        } else if matches!(
                                            state.current_view(),
                                            crate::state::View::CreateTask
                                                | crate::state::View::EditTask
                                        ) {
                                            if !state.is_field_editing_mode() {
                                                let enabled_custom_fields =
                                                    state.get_enabled_custom_fields();
                                                let next_state = match state.get_edit_form_state() {
                                                    Some(crate::state::EditFormState::Name) => {
                                                        crate::state::EditFormState::Notes
                                                    }
                                                    Some(crate::state::EditFormState::Notes) => {
                                                        crate::state::EditFormState::Assignee
                                                    }
                                                    Some(crate::state::EditFormState::Assignee) => {
                                                        crate::state::EditFormState::DueDate
                                                    }
                                                    Some(crate::state::EditFormState::DueDate) => {
                                                        crate::state::EditFormState::Section
                                                    }
                                                    Some(crate::state::EditFormState::Section) => {
                                                        if !enabled_custom_fields.is_empty() {
                                                            crate::state::EditFormState::CustomField(
                                                                0,
                                                            )
                                                        } else {
                                                            crate::state::EditFormState::Name
                                                        }
                                                    }
                                                    Some(
                                                        crate::state::EditFormState::CustomField(
                                                            idx,
                                                        ),
                                                    ) => {
                                                        if idx + 1 < enabled_custom_fields.len() {
                                                            crate::state::EditFormState::CustomField(
                                                                idx + 1,
                                                            )
                                                        } else {
                                                            crate::state::EditFormState::Name
                                                        }
                                                    }
                                                    None => crate::state::EditFormState::Name,
                                                };
                                                state.set_edit_form_state(Some(next_state));
                                                if matches!(
                                                    next_state,
                                                    crate::state::EditFormState::Assignee
                                                ) {
                                                    state.init_assignee_dropdown_index();
                                                } else if matches!(
                                                    next_state,
                                                    crate::state::EditFormState::Section
                                                ) {
                                                    state.init_section_dropdown_index();
                                                }
                                            } else {
                                                match state.get_edit_form_state() {
                                                    Some(crate::state::EditFormState::Assignee) => {
                                                        state.next_assignee();
                                                    }
                                                    Some(crate::state::EditFormState::Section) => {
                                                        state.next_section();
                                                    }
                                                    _ => {}
                                                }
                                            }
                                        } else {
                                            match state.current_focus() {
                                                Focus::Menu => match state.current_menu() {
                                                    Menu::Status => (),
                                                    Menu::Shortcuts => {
                                                        state.next_shortcut_index();
                                                    }
                                                    Menu::TopList => {
                                                        state.next_top_list_index();
                                                    }
                                                },
                                                Focus::View => {
                                                    if !matches!(
                                                        state.current_view(),
                                                        crate::state::View::Welcome
                                                    ) {
                                                        state.next_task_index();
                                                    } else {
                                                        state.focus_menu();
                                                    }
                                                }
                                            }
                                        }
                                        return Ok(true);
                                    }
                                    HotkeyAction::NavigatePrev => {
                                        if state.has_theme_selector() {
                                            state.previous_theme();
                                        } else if state.has_assignee_filter() {
                                            state.previous_assignee_filter_option();
                                        } else if state.has_move_task() {
                                            state.previous_section();
                                        } else if state.is_debug_mode() {
                                            state.previous_debug();
                                        } else if matches!(
                                            state.current_view(),
                                            crate::state::View::TaskDetail
                                        ) {
                                            match state.get_current_task_panel() {
                                                crate::state::TaskDetailPanel::Comments => {
                                                    state.scroll_comments_up();
                                                }
                                                crate::state::TaskDetailPanel::Details => {
                                                    state.scroll_details_up();
                                                }
                                                crate::state::TaskDetailPanel::Notes => {
                                                    state.scroll_notes_up();
                                                }
                                            }
                                        } else if matches!(
                                            state.current_view(),
                                            crate::state::View::ProjectTasks
                                        ) {
                                            state.previous_kanban_task();
                                        } else if matches!(
                                            state.current_view(),
                                            crate::state::View::CreateTask
                                                | crate::state::View::EditTask
                                        ) {
                                            if !state.is_field_editing_mode() {
                                                let enabled_custom_fields =
                                                    state.get_enabled_custom_fields();
                                                let prev_state = match state.get_edit_form_state() {
                                                    Some(crate::state::EditFormState::Name) => {
                                                        if !enabled_custom_fields.is_empty() {
                                                            crate::state::EditFormState::CustomField(
                                                                enabled_custom_fields.len() - 1,
                                                            )
                                                        } else {
                                                            crate::state::EditFormState::Section
                                                        }
                                                    }
                                                    Some(crate::state::EditFormState::Notes) => {
                                                        crate::state::EditFormState::Name
                                                    }
                                                    Some(crate::state::EditFormState::Assignee) => {
                                                        crate::state::EditFormState::Notes
                                                    }
                                                    Some(crate::state::EditFormState::DueDate) => {
                                                        crate::state::EditFormState::Assignee
                                                    }
                                                    Some(crate::state::EditFormState::Section) => {
                                                        crate::state::EditFormState::DueDate
                                                    }
                                                    Some(
                                                        crate::state::EditFormState::CustomField(0),
                                                    ) => crate::state::EditFormState::Section,
                                                    Some(
                                                        crate::state::EditFormState::CustomField(
                                                            idx,
                                                        ),
                                                    ) => crate::state::EditFormState::CustomField(
                                                        idx - 1,
                                                    ),
                                                    None => crate::state::EditFormState::Name,
                                                };
                                                state.set_edit_form_state(Some(prev_state));
                                                if matches!(
                                                    prev_state,
                                                    crate::state::EditFormState::Assignee
                                                ) {
                                                    state.init_assignee_dropdown_index();
                                                } else if matches!(
                                                    prev_state,
                                                    crate::state::EditFormState::Section
                                                ) {
                                                    state.init_section_dropdown_index();
                                                }
                                            } else {
                                                match state.get_edit_form_state() {
                                                    Some(crate::state::EditFormState::Assignee) => {
                                                        state.previous_assignee();
                                                    }
                                                    Some(crate::state::EditFormState::Section) => {
                                                        state.previous_section();
                                                    }
                                                    _ => {}
                                                }
                                            }
                                        } else {
                                            match state.current_focus() {
                                                Focus::Menu => match state.current_menu() {
                                                    Menu::Status => (),
                                                    Menu::Shortcuts => {
                                                        state.previous_shortcut_index();
                                                    }
                                                    Menu::TopList => {
                                                        state.previous_top_list_index();
                                                    }
                                                },
                                                Focus::View => {
                                                    if !matches!(
                                                        state.current_view(),
                                                        crate::state::View::Welcome
                                                    ) {
                                                        state.previous_task_index();
                                                    } else {
                                                        state.focus_menu();
                                                    }
                                                }
                                            }
                                        }
                                        return Ok(true);
                                    }
                                    _ => {}
                                }
                            }
                        }
                        HotkeyAction::NavigateLeft | HotkeyAction::NavigateRight => {
                            // Handle left/right navigation - but skip if in text input mode
                            if state.is_search_mode() || state.is_comment_input_mode() {
                                // In text input modes, allow typing
                                if let KeyCode::Char(c) = event.code {
                                    if state.is_search_mode() {
                                        state.add_search_char(c);
                                    } else {
                                        state.add_comment_char(c);
                                    }
                                    return Ok(true);
                                }
                            } else if state.has_assignee_filter() {
                                // In assignee filter mode, allow typing for search
                                if let KeyCode::Char(c) = event.code {
                                    state.add_assignee_filter_search_char(c);
                                    return Ok(true);
                                }
                            } else if matches!(
                                state.current_view(),
                                crate::state::View::CreateTask | crate::state::View::EditTask
                            ) && state.is_field_editing_mode()
                                && !matches!(
                                    state.get_edit_form_state(),
                                    Some(crate::state::EditFormState::Assignee)
                                        | Some(crate::state::EditFormState::Section)
                                )
                            {
                                // In form text fields, allow typing
                                if let KeyCode::Char(c) = event.code {
                                    match state.get_edit_form_state() {
                                        Some(crate::state::EditFormState::Name) => {
                                            state.add_form_name_char(c);
                                        }
                                        Some(crate::state::EditFormState::Notes) => {
                                            // Already handled above
                                        }
                                        Some(crate::state::EditFormState::DueDate) => {
                                            state.add_form_due_on_char(c);
                                        }
                                        _ => {}
                                    }
                                    return Ok(true);
                                }
                            } else {
                                // Execute left/right navigation
                                match action {
                                    HotkeyAction::NavigateLeft => {
                                        if matches!(
                                            state.current_view(),
                                            crate::state::View::TaskDetail
                                        ) {
                                            state.previous_task_panel();
                                        } else if matches!(
                                            state.current_view(),
                                            crate::state::View::ProjectTasks
                                        ) {
                                            if *state.current_focus() == Focus::View {
                                                state.previous_kanban_column();
                                            } else {
                                                state.focus_view();
                                            }
                                        } else {
                                            // Welcome view: switch between menus when focus is on Menu
                                            if matches!(
                                                state.current_view(),
                                                crate::state::View::Welcome
                                            ) {
                                                match state.current_focus() {
                                                    Focus::Menu => {
                                                        state.previous_menu();
                                                    }
                                                    Focus::View => {
                                                        state.focus_menu();
                                                    }
                                                }
                                            } else {
                                                match state.current_focus() {
                                                    Focus::Menu => {
                                                        state.focus_view();
                                                    }
                                                    Focus::View => {
                                                        state.focus_menu();
                                                    }
                                                }
                                            }
                                        }
                                        return Ok(true);
                                    }
                                    HotkeyAction::NavigateRight => {
                                        if matches!(
                                            state.current_view(),
                                            crate::state::View::TaskDetail
                                        ) {
                                            state.next_task_panel();
                                        } else if matches!(
                                            state.current_view(),
                                            crate::state::View::ProjectTasks
                                        ) {
                                            if *state.current_focus() == Focus::View {
                                                state.next_kanban_column();
                                            } else {
                                                state.focus_view();
                                            }
                                        } else {
                                            // Welcome view: switch between menus when focus is on Menu
                                            if matches!(
                                                state.current_view(),
                                                crate::state::View::Welcome
                                            ) {
                                                match state.current_focus() {
                                                    Focus::Menu => {
                                                        state.next_menu();
                                                    }
                                                    Focus::View => {
                                                        state.focus_menu();
                                                    }
                                                }
                                            } else {
                                                match state.current_focus() {
                                                    Focus::Menu => {
                                                        state.focus_view();
                                                    }
                                                    Focus::View => {
                                                        state.focus_menu();
                                                    }
                                                }
                                            }
                                        }
                                        return Ok(true);
                                    }
                                    _ => {}
                                }
                            }
                        }
                        _ => {
                            // Not a navigation action, continue to normal processing
                        }
                    }
                }

                match event {
                    KeyEvent {
                        code: KeyCode::Char('c'),
                        modifiers: KeyModifiers::CONTROL,
                        ..
                    } => {
                        debug!("Processing exit terminal event '{:?}'...", event);
                        return Ok(false);
                    }
                    // Handle comment input mode FIRST - all character keys go to comment
                    KeyEvent {
                        code: KeyCode::Char(c),
                        modifiers: KeyModifiers::NONE,
                        ..
                    } if state.is_comment_input_mode() => {
                        state.add_comment_char(c);
                    }
                    KeyEvent {
                        code: KeyCode::Char(c),
                        modifiers: KeyModifiers::SHIFT,
                        ..
                    } if state.is_comment_input_mode() => {
                        state.add_comment_char(c);
                    }
                    // Form field navigation is now handled by NavigateFieldNext/NavigateFieldPrev hotkeys
                    KeyEvent {
                        code: KeyCode::Enter,
                        modifiers: KeyModifiers::NONE,
                        ..
                    } if matches!(
                        state.current_view(),
                        crate::state::View::CreateTask | crate::state::View::EditTask
                    ) && !state.is_field_editing_mode()
                        && state.get_edit_form_state().is_some() =>
                    {
                        // Enter field editing mode
                        state.enter_field_editing_mode();
                    }
                    // Handle token input in onboarding - all character keys go to token input
                    // Clear error when user starts typing
                    KeyEvent {
                        code: KeyCode::Char(c),
                        modifiers: KeyModifiers::NONE,
                        ..
                    } if matches!(state.current_view(), crate::state::View::Welcome)
                        && !state.has_access_token() =>
                    {
                        // Clear error when user starts typing
                        if state.get_auth_error().is_some() {
                            state.clear_auth_error();
                        }
                        state.add_access_token_char(c);
                    }
                    KeyEvent {
                        code: KeyCode::Char(c),
                        modifiers: KeyModifiers::SHIFT,
                        ..
                    } if matches!(state.current_view(), crate::state::View::Welcome)
                        && !state.has_access_token() =>
                    {
                        // Clear error when user starts typing
                        if state.get_auth_error().is_some() {
                            state.clear_auth_error();
                        }
                        state.add_access_token_char(c);
                    }
                    // Handle assignee search input - all character keys go to search (only in field editing mode)
                    KeyEvent {
                        code: KeyCode::Char(c),
                        modifiers: KeyModifiers::NONE,
                        ..
                    } if matches!(
                        state.current_view(),
                        crate::state::View::CreateTask | crate::state::View::EditTask
                    ) && state.is_field_editing_mode()
                        && matches!(
                            state.get_edit_form_state(),
                            Some(crate::state::EditFormState::Assignee)
                        ) =>
                    {
                        state.add_assignee_search_char(c);
                    }
                    KeyEvent {
                        code: KeyCode::Char(c),
                        modifiers: KeyModifiers::SHIFT,
                        ..
                    } if matches!(
                        state.current_view(),
                        crate::state::View::CreateTask | crate::state::View::EditTask
                    ) && state.is_field_editing_mode()
                        && matches!(
                            state.get_edit_form_state(),
                            Some(crate::state::EditFormState::Assignee)
                        ) =>
                    {
                        state.add_assignee_search_char(c);
                    }
                    // Handle section search input - all character keys go to search (only in field editing mode)
                    KeyEvent {
                        code: KeyCode::Char(c),
                        modifiers: KeyModifiers::NONE,
                        ..
                    } if matches!(
                        state.current_view(),
                        crate::state::View::CreateTask | crate::state::View::EditTask
                    ) && state.is_field_editing_mode()
                        && matches!(
                            state.get_edit_form_state(),
                            Some(crate::state::EditFormState::Section)
                        ) =>
                    {
                        state.add_section_search_char(c);
                    }
                    // Handle assignee filter search input - all character keys go to search
                    KeyEvent {
                        code: KeyCode::Char(c),
                        modifiers: KeyModifiers::NONE,
                        ..
                    } if state.has_assignee_filter() => {
                        state.add_assignee_filter_search_char(c);
                        return Ok(true);
                    }
                    KeyEvent {
                        code: KeyCode::Char(c),
                        modifiers: KeyModifiers::SHIFT,
                        ..
                    } if state.has_assignee_filter() => {
                        state.add_assignee_filter_search_char(c);
                        return Ok(true);
                    }
                    KeyEvent {
                        code: KeyCode::Backspace,
                        modifiers: KeyModifiers::NONE,
                        ..
                    } if state.has_assignee_filter() => {
                        state.backspace_assignee_filter_search();
                        return Ok(true);
                    }
                    // Handle arrow keys for assignee filter navigation
                    KeyEvent {
                        code: KeyCode::Up, ..
                    } if state.has_assignee_filter() => {
                        state.previous_assignee_filter_option();
                        return Ok(true);
                    }
                    KeyEvent {
                        code: KeyCode::Down,
                        ..
                    } if state.has_assignee_filter() => {
                        state.next_assignee_filter_option();
                        return Ok(true);
                    }
                    KeyEvent {
                        code: KeyCode::Char(c),
                        modifiers: KeyModifiers::SHIFT,
                        ..
                    } if matches!(
                        state.current_view(),
                        crate::state::View::CreateTask | crate::state::View::EditTask
                    ) && state.is_field_editing_mode()
                        && matches!(
                            state.get_edit_form_state(),
                            Some(crate::state::EditFormState::Section)
                        ) =>
                    {
                        state.add_section_search_char(c);
                    }
                    KeyEvent {
                        code: KeyCode::Esc,
                        modifiers: KeyModifiers::NONE,
                        ..
                    } => {
                        // Handle special modes first (they take priority)
                        if state.is_search_mode() {
                            if let Some(HotkeyAction::SearchModeExit | HotkeyAction::Cancel) =
                                get_action_for_special_mode(
                                    &event,
                                    SpecialMode::Search,
                                    state.get_hotkeys(),
                                )
                            {
                                debug!("Processing exit search mode event '{:?}'...", event);
                                state.exit_search_mode();
                                return Ok(true);
                            }
                            // Fallback to default behavior
                            debug!("Processing exit search mode event '{:?}'...", event);
                            state.exit_search_mode();
                        } else if state.is_debug_mode() {
                            if let Some(HotkeyAction::DebugModeExit | HotkeyAction::Cancel) =
                                get_action_for_special_mode(
                                    &event,
                                    SpecialMode::Debug,
                                    state.get_hotkeys(),
                                )
                            {
                                debug!("Processing exit debug mode (Esc) event '{:?}'...", event);
                                state.exit_debug_mode();
                                return Ok(true);
                            }
                            // Fallback to default behavior
                            debug!("Processing exit debug mode (Esc) event '{:?}'...", event);
                            state.exit_debug_mode();
                        } else if state.has_delete_confirmation() {
                            if let Some(HotkeyAction::Cancel) = get_action_for_special_mode(
                                &event,
                                SpecialMode::DeleteConfirmation,
                                state.get_hotkeys(),
                            ) {
                                debug!(
                                    "Processing cancel delete confirmation event '{:?}'...",
                                    event
                                );
                                state.cancel_delete_confirmation();
                                return Ok(true);
                            }
                            // Fallback to default behavior
                            debug!(
                                "Processing cancel delete confirmation event '{:?}'...",
                                event
                            );
                            state.cancel_delete_confirmation();
                        } else if state.has_theme_selector() {
                            if let Some(HotkeyAction::ThemeSelectorCancel) =
                                get_action_for_special_mode(
                                    &event,
                                    SpecialMode::ThemeSelector,
                                    state.get_hotkeys(),
                                )
                            {
                                debug!("Processing cancel theme selector event '{:?}'...", event);
                                state.close_theme_selector();
                                return Ok(true);
                            }
                            // Fallback to default behavior
                            debug!("Processing cancel theme selector event '{:?}'...", event);
                            state.close_theme_selector();
                        } else if state.has_assignee_filter() {
                            if let Some(HotkeyAction::AssigneeFilterCancel) =
                                get_action_for_special_mode(
                                    &event,
                                    SpecialMode::AssigneeFilter,
                                    state.get_hotkeys(),
                                )
                            {
                                debug!("Processing cancel assignee filter event '{:?}'...", event);
                                state.close_assignee_filter();
                                return Ok(true);
                            }
                            // Fallback to default behavior
                            debug!("Processing cancel assignee filter event '{:?}'...", event);
                            state.close_assignee_filter();
                        } else if state.has_move_task() {
                            if let Some(HotkeyAction::MoveTaskCancel) = get_action_for_special_mode(
                                &event,
                                SpecialMode::MoveTask,
                                state.get_hotkeys(),
                            ) {
                                debug!("Processing cancel move task event '{:?}'...", event);
                                state.clear_move_task();
                                return Ok(true);
                            }
                            // Fallback to default behavior
                            debug!("Processing cancel move task event '{:?}'...", event);
                            state.clear_move_task();
                        } else if state.is_comment_input_mode() {
                            debug!("Processing cancel comment input event '{:?}'...", event);
                            state.exit_comment_input_mode();
                        } else if *state.current_focus() == Focus::View {
                            debug!("Processing view navigation (Esc) event '{:?}'...", event);
                            // Pop the current view to go back
                            if let Some(popped_view) = state.pop_view() {
                                debug!(
                                    "Popped view: {:?}, remaining views: {}",
                                    popped_view,
                                    state.view_stack_len()
                                );

                                // If we're going back from a detail/edit/create view, refresh the parent view
                                match popped_view {
                                    crate::state::View::TaskDetail
                                    | crate::state::View::EditTask
                                    | crate::state::View::CreateTask => {
                                        // Refresh the parent view (usually ProjectTasks)
                                        if matches!(
                                            state.current_view(),
                                            crate::state::View::ProjectTasks
                                        ) {
                                            state.dispatch(
                                                crate::events::network::Event::ProjectTasks,
                                            );
                                        }
                                    }
                                    crate::state::View::ProjectTasks => {
                                        // When escaping from ProjectTasks, always focus menu
                                        state.focus_menu();
                                    }
                                    _ => {}
                                }

                                // If we're back at the menu, focus it
                                if matches!(state.current_view(), crate::state::View::Welcome) {
                                    state.focus_menu();
                                }
                            } else {
                                // No more views to pop, go back to menu
                                debug!("No more views to pop, focusing menu");
                                state.focus_menu();
                            }
                        }
                    }
                    // Handle space key for toggle completion (special case - not a hotkey)
                    KeyEvent {
                        code: KeyCode::Char(' '),
                        modifiers: KeyModifiers::NONE,
                        ..
                    } => {
                        if state.is_search_mode() {
                            state.add_search_char(' ');
                        } else if state.is_comment_input_mode() {
                            state.add_comment_char(' ');
                        } else if state.current_focus() == &Focus::View {
                            debug!("Processing toggle task completion event '{:?}'...", event);
                            state.toggle_task_completion();
                        }
                    }
                    KeyEvent {
                        code: KeyCode::Backspace,
                        modifiers: KeyModifiers::NONE,
                        ..
                    } => {
                        if state.is_search_mode() {
                            debug!("Processing remove search character event '{:?}'...", event);
                            state.remove_search_char();
                        } else if state.is_comment_input_mode() {
                            debug!("Processing remove comment character event '{:?}'...", event);
                            state.remove_comment_char();
                        } else if matches!(state.current_view(), crate::state::View::Welcome)
                            && !state.has_access_token()
                        {
                            debug!("Processing remove token character event '{:?}'...", event);
                            // Clear error when user edits
                            if state.get_auth_error().is_some() {
                                state.clear_auth_error();
                            }
                            state.backspace_access_token();
                        } else if matches!(
                            state.current_view(),
                            crate::state::View::CreateTask | crate::state::View::EditTask
                        ) && state.is_field_editing_mode()
                        {
                            // Handle form backspace (only in field editing mode)
                            match state.get_edit_form_state() {
                                Some(crate::state::EditFormState::Name) => {
                                    state.remove_form_name_char();
                                }
                                Some(crate::state::EditFormState::Notes) => {
                                    // Already handled above
                                }
                                Some(crate::state::EditFormState::Assignee) => {
                                    state.backspace_assignee_search();
                                }
                                Some(crate::state::EditFormState::Section) => {
                                    state.backspace_section_search();
                                }
                                Some(crate::state::EditFormState::DueDate) => {
                                    state.remove_form_due_on_char();
                                }
                                _ => {}
                            }
                        }
                    }
                    KeyEvent {
                        code: KeyCode::Tab,
                        modifiers: KeyModifiers::NONE,
                        ..
                    } => {
                        if !state.is_search_mode()
                            && !state.is_comment_input_mode()
                            && matches!(
                                state.current_view(),
                                crate::state::View::CreateTask | crate::state::View::EditTask
                            )
                        {
                            let custom_fields = state.get_project_custom_fields();
                            let next_state = match state.get_edit_form_state() {
                                Some(crate::state::EditFormState::Name) => {
                                    crate::state::EditFormState::Notes
                                }
                                Some(crate::state::EditFormState::Notes) => {
                                    crate::state::EditFormState::Assignee
                                }
                                Some(crate::state::EditFormState::Assignee) => {
                                    crate::state::EditFormState::DueDate
                                }
                                Some(crate::state::EditFormState::DueDate) => {
                                    crate::state::EditFormState::Section
                                }
                                Some(crate::state::EditFormState::Section) => {
                                    if !custom_fields.is_empty() {
                                        crate::state::EditFormState::CustomField(0)
                                    } else {
                                        crate::state::EditFormState::Name
                                    }
                                }
                                Some(crate::state::EditFormState::CustomField(idx)) => {
                                    if idx + 1 < custom_fields.len() {
                                        crate::state::EditFormState::CustomField(idx + 1)
                                    } else {
                                        crate::state::EditFormState::Name
                                    }
                                }
                                None => crate::state::EditFormState::Name,
                            };
                            state.set_edit_form_state(Some(next_state));
                            // Initialize dropdown indices when entering assignee or section fields
                            if matches!(next_state, crate::state::EditFormState::Assignee) {
                                state.init_assignee_dropdown_index();
                            } else if matches!(next_state, crate::state::EditFormState::Section) {
                                state.init_section_dropdown_index();
                            }
                        }
                    }
                    KeyEvent {
                        code: KeyCode::BackTab,
                        modifiers: KeyModifiers::SHIFT,
                        ..
                    } => {
                        if !state.is_search_mode()
                            && !state.is_comment_input_mode()
                            && matches!(
                                state.current_view(),
                                crate::state::View::CreateTask | crate::state::View::EditTask
                            )
                        {
                            let custom_fields = state.get_project_custom_fields();
                            let prev_state = match state.get_edit_form_state() {
                                Some(crate::state::EditFormState::Name) => {
                                    if !custom_fields.is_empty() {
                                        crate::state::EditFormState::CustomField(
                                            custom_fields.len() - 1,
                                        )
                                    } else {
                                        crate::state::EditFormState::Section
                                    }
                                }
                                Some(crate::state::EditFormState::Notes) => {
                                    crate::state::EditFormState::Name
                                }
                                Some(crate::state::EditFormState::Assignee) => {
                                    crate::state::EditFormState::Notes
                                }
                                Some(crate::state::EditFormState::DueDate) => {
                                    crate::state::EditFormState::Assignee
                                }
                                Some(crate::state::EditFormState::Section) => {
                                    crate::state::EditFormState::DueDate
                                }
                                Some(crate::state::EditFormState::CustomField(0)) => {
                                    crate::state::EditFormState::Section
                                }
                                Some(crate::state::EditFormState::CustomField(idx)) => {
                                    crate::state::EditFormState::CustomField(idx - 1)
                                }
                                None => crate::state::EditFormState::Name,
                            };
                            state.set_edit_form_state(Some(prev_state));
                            // Initialize dropdown indices when entering assignee or section fields
                            if matches!(prev_state, crate::state::EditFormState::Assignee) {
                                state.init_assignee_dropdown_index();
                            } else if matches!(prev_state, crate::state::EditFormState::Section) {
                                state.init_section_dropdown_index();
                            }
                        }
                    }
                    // Handle all character keys - check text input modes first, then hotkeys
                    KeyEvent {
                        code: KeyCode::Char(c),
                        modifiers: KeyModifiers::NONE,
                        ..
                    } => {
                        // First, check if we're in a text input mode
                        if state.is_search_mode() {
                            debug!("Processing search character '{}' event '{:?}'...", c, event);
                            state.add_search_char(c);
                            return Ok(true);
                        } else if state.is_comment_input_mode() {
                            debug!(
                                "Processing comment character '{}' event '{:?}'...",
                                c, event
                            );
                            state.add_comment_char(c);
                            return Ok(true);
                        } else if matches!(
                            state.current_view(),
                            crate::state::View::CreateTask | crate::state::View::EditTask
                        ) && state.is_field_editing_mode()
                            && !matches!(
                                state.get_edit_form_state(),
                                Some(crate::state::EditFormState::Assignee)
                                    | Some(crate::state::EditFormState::Section)
                            )
                        {
                            // In form text fields, allow typing any character
                            match state.get_edit_form_state() {
                                Some(crate::state::EditFormState::Name) => {
                                    state.add_form_name_char(c);
                                }
                                Some(crate::state::EditFormState::Notes) => {
                                    // Already handled above
                                }
                                Some(crate::state::EditFormState::DueDate) => {
                                    state.add_form_due_on_char(c);
                                }
                                _ => {}
                            }
                            return Ok(true);
                        }

                        // Check for special mode hotkeys (debug, search, etc.)
                        if state.is_debug_mode() {
                            if let Some(action) = get_action_for_special_mode(
                                &event,
                                SpecialMode::Debug,
                                state.get_hotkeys(),
                            ) {
                                match action {
                                    HotkeyAction::DebugModeCopyLog => {
                                        debug!("Processing copy debug log event '{:?}'...", event);
                                        if let Some(debug_entry) = state.get_current_debug() {
                                            // Copy to clipboard
                                            match ClipboardContext::new() {
                                                Ok(mut ctx) => {
                                                    match ctx.set_contents(debug_entry.to_string())
                                                    {
                                                        Ok(_) => {
                                                            info!("Debug log entry copied to clipboard");
                                                        }
                                                        Err(e) => {
                                                            warn!(
                                                                "Failed to copy to clipboard: {}",
                                                                e
                                                            );
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    warn!("Failed to initialize clipboard: {}", e);
                                                }
                                            }
                                        }
                                        return Ok(true);
                                    }
                                    HotkeyAction::DebugModeExit => {
                                        debug!("Processing exit debug mode event '{:?}'...", event);
                                        state.exit_debug_mode();
                                        return Ok(true);
                                    }
                                    _ => {}
                                }
                            }
                        } else if state.is_search_mode() {
                            if let Some(HotkeyAction::SearchModeExit) = get_action_for_special_mode(
                                &event,
                                SpecialMode::Search,
                                state.get_hotkeys(),
                            ) {
                                debug!("Processing exit search mode event '{:?}'...", event);
                                state.exit_search_mode();
                                return Ok(true);
                            }
                        }

                        // Not in text input mode or special mode - check for configured hotkeys
                        if let Ok(Some(should_continue)) = try_execute_hotkey_action(&event, state)
                        {
                            return Ok(should_continue);
                        }

                        // No handler found - skip
                        debug!("Skipping processing of terminal event '{:?}'...", event);
                    }
                    // Handle Enter key - check hotkeys first, then special cases
                    KeyEvent {
                        code: KeyCode::Enter,
                        modifiers: KeyModifiers::NONE,
                        ..
                    } => {
                        // Check for hotkey action first
                        if let Some(action) =
                            get_action_for_event(&event, state.current_view(), state.get_hotkeys())
                        {
                            match action {
                                HotkeyAction::Select => {
                                    // Handle onboarding screen - submit access token
                                    if matches!(state.current_view(), crate::state::View::Welcome)
                                        && !state.has_access_token()
                                    {
                                        let token = state.get_access_token_input().to_string();
                                        if !token.trim().is_empty() {
                                            debug!(
                                                "Submitting access token from onboarding screen..."
                                            );
                                            state.dispatch(
                                                crate::events::network::Event::SetAccessToken {
                                                    token,
                                                },
                                            );
                                        }
                                        return Ok(true);
                                    }
                                    // Select action is handled in the main Enter handler below
                                }
                                HotkeyAction::EditField => {
                                    // Enter field editing mode
                                    if matches!(
                                        state.current_view(),
                                        crate::state::View::CreateTask
                                            | crate::state::View::EditTask
                                    ) && !state.is_field_editing_mode()
                                        && state.get_edit_form_state().is_some()
                                    {
                                        state.enter_field_editing_mode();
                                        return Ok(true);
                                    }
                                }
                                _ => {}
                            }
                        }

                        // Check special modes
                        if state.is_search_mode() {
                            debug!("Processing exit search mode (Enter) event '{:?}'...", event);
                            state.exit_search_mode();
                        } else if state.is_debug_mode() {
                            debug!("Processing exit debug mode (Enter) event '{:?}'...", event);
                            state.exit_debug_mode();
                        } else if state.has_theme_selector() {
                            if let Some(HotkeyAction::ThemeSelectorSelect) =
                                get_action_for_special_mode(
                                    &event,
                                    SpecialMode::ThemeSelector,
                                    state.get_hotkeys(),
                                )
                            {
                                debug!("Processing select theme event '{:?}'...", event);
                                state.select_theme();
                                return Ok(true);
                            }
                            // Fallback to default behavior
                            debug!("Processing select theme event '{:?}'...", event);
                            state.select_theme();
                        } else if state.has_assignee_filter() {
                            if let Some(HotkeyAction::AssigneeFilterSelect) =
                                get_action_for_special_mode(
                                    &event,
                                    SpecialMode::AssigneeFilter,
                                    state.get_hotkeys(),
                                )
                            {
                                // In assignee filter modal, Enter selects the assignee and applies filter
                                debug!("Selecting assignee filter...");
                                state.select_assignee_filter();
                                return Ok(true);
                            }
                            // Fallback to default behavior
                            debug!("Selecting assignee filter...");
                            state.select_assignee_filter();
                        } else if state.has_move_task() {
                            if let Some(HotkeyAction::MoveTaskConfirm) = get_action_for_special_mode(
                                &event,
                                SpecialMode::MoveTask,
                                state.get_hotkeys(),
                            ) {
                                // In move task modal, Enter selects the section and moves the task
                                if let Some(task_gid) = state.get_move_task_gid() {
                                    let sections = state.get_sections();
                                    let selected_index = state.get_section_dropdown_index();
                                    if selected_index < sections.len() {
                                        let selected_section = &sections[selected_index];
                                        debug!(
                                            "Moving task {} to section {}...",
                                            task_gid, selected_section.gid
                                        );
                                        state.dispatch(
                                            crate::events::network::Event::MoveTaskToSection {
                                                task_gid: task_gid.clone(),
                                                section_gid: selected_section.gid.clone(),
                                            },
                                        );
                                        state.clear_move_task();
                                    }
                                }
                                return Ok(true);
                            }
                            // Fallback to default behavior
                            if let Some(task_gid) = state.get_move_task_gid() {
                                let sections = state.get_sections();
                                let selected_index = state.get_section_dropdown_index();
                                if selected_index < sections.len() {
                                    let selected_section = &sections[selected_index];
                                    debug!(
                                        "Moving task {} to section {}...",
                                        task_gid, selected_section.gid
                                    );
                                    state.dispatch(
                                        crate::events::network::Event::MoveTaskToSection {
                                            task_gid: task_gid.clone(),
                                            section_gid: selected_section.gid.clone(),
                                        },
                                    );
                                    state.clear_move_task();
                                }
                            }
                        } else if state.has_delete_confirmation() {
                            if let Some(HotkeyAction::DeleteConfirm) = get_action_for_special_mode(
                                &event,
                                SpecialMode::DeleteConfirmation,
                                state.get_hotkeys(),
                            ) {
                                debug!("Processing confirm delete event '{:?}'...", event);
                                state.confirm_delete_task();
                                return Ok(true);
                            }
                            // Fallback to default behavior
                            debug!("Processing confirm delete event '{:?}'...", event);
                            state.confirm_delete_task();
                        } else if state.is_comment_input_mode() {
                            // Submit comment
                            let task_gid = state.get_task_detail().map(|t| t.gid.clone());
                            if let Some(gid) = task_gid {
                                let comment_text = state.submit_comment();
                                if !comment_text.trim().is_empty() {
                                    state.dispatch(crate::events::network::Event::CreateStory {
                                        task_gid: gid,
                                        text: comment_text,
                                    });
                                }
                            }
                        } else if matches!(
                            state.current_view(),
                            crate::state::View::CreateTask | crate::state::View::EditTask
                        ) {
                            // Handle form Enter key
                            match state.get_edit_form_state() {
                                Some(crate::state::EditFormState::Name) => {
                                    state.set_edit_form_state(Some(
                                        crate::state::EditFormState::Notes,
                                    ));
                                }
                                Some(crate::state::EditFormState::Notes) => {
                                    // In textarea, Enter creates newline - use Tab to move to next field
                                    // Already handled above by routing to textarea
                                }
                                Some(crate::state::EditFormState::Assignee) => {
                                    // Select the current assignee and move to next field
                                    state.select_current_assignee();
                                    state.set_edit_form_state(Some(
                                        crate::state::EditFormState::DueDate,
                                    ));
                                }
                                Some(crate::state::EditFormState::DueDate) => {
                                    state.set_edit_form_state(Some(
                                        crate::state::EditFormState::Section,
                                    ));
                                    state.init_section_dropdown_index();
                                }
                                Some(crate::state::EditFormState::Section) => {
                                    // Select the current section and stay in Section field
                                    // User must press 's' to submit or Esc to cancel
                                    state.select_current_section();
                                }
                                Some(crate::state::EditFormState::CustomField(_idx)) => {
                                    // TODO: Handle custom field selection
                                }
                                None => {
                                    state.set_edit_form_state(Some(
                                        crate::state::EditFormState::Name,
                                    ));
                                }
                            }
                        } else {
                            match state.current_focus() {
                                Focus::Menu => {
                                    debug!("Processing select menu item event '{:?}'...", event);
                                    match state.current_menu() {
                                        Menu::Status => {
                                            state.select_status_menu();
                                        }
                                        Menu::Shortcuts => {
                                            state.select_current_shortcut_index();
                                        }
                                        Menu::TopList => {
                                            state.select_current_top_list_index();
                                        }
                                    }
                                }
                                Focus::View => {
                                    // Enter key in task view: view task details
                                    if matches!(
                                        state.current_view(),
                                        crate::state::View::ProjectTasks
                                    ) {
                                        if state.get_view_mode() == crate::state::ViewMode::Kanban {
                                            // Kanban view: get selected task from kanban
                                            if let Some(task) = state.get_kanban_selected_task() {
                                                state.dispatch(
                                                    crate::events::network::Event::GetTaskDetail {
                                                        gid: task.gid.clone(),
                                                    },
                                                );
                                                state.push_view(crate::state::View::TaskDetail);
                                                state.focus_view();
                                            }
                                        } else {
                                            // List view: get selected task from list
                                            let filtered = state.get_filtered_tasks();
                                            if let Some(selected_index) =
                                                state.get_tasks_list_state().selected()
                                            {
                                                if selected_index < filtered.len() {
                                                    let task = &filtered[selected_index];
                                                    state.dispatch(crate::events::network::Event::GetTaskDetail {
                                                    gid: task.gid.clone(),
                                                });
                                                    state.push_view(crate::state::View::TaskDetail);
                                                    state.focus_view();
                                                }
                                            }
                                        }
                                    } else if matches!(
                                        state.current_view(),
                                        crate::state::View::Welcome
                                    ) {
                                        // Kanban board view: view task details
                                        if let Some(task) = state.get_kanban_selected_task() {
                                            state.dispatch(
                                                crate::events::network::Event::GetTaskDetail {
                                                    gid: task.gid.clone(),
                                                },
                                            );
                                            state.push_view(crate::state::View::TaskDetail);
                                            state.focus_view();
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => {
                        if !state.is_search_mode() {
                            debug!("Skipping processing of terminal event '{:?}'...", event);
                        }
                    }
                }
            }
            Event::Tick => {
                state.advance_spinner_index();
            }
        }
        Ok(true)
    }
}
