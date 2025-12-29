use crate::state::{Focus, Menu, State};
use anyhow::Result;
use clipboard::{ClipboardContext, ClipboardProvider};
use crossterm::{
    event,
    event::{Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers},
};
use log::*;
use std::{sync::mpsc, thread, time::Duration};

/// Specify terminal event poll rate in milliseconds.
///
const TICK_RATE_IN_MS: u64 = 60;

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
        thread::spawn(move || loop {
            let tick_rate = Duration::from_millis(TICK_RATE_IN_MS);
            if event::poll(tick_rate).unwrap() {
                if let CrosstermEvent::Key(key) = event::read().unwrap() {
                    tx_clone.send(Event::Input(key)).unwrap();
                }
            }
            tx_clone.send(Event::Tick).unwrap();
        });
        Handler { rx, _tx: tx }
    }

    /// Receive next terminal event and handle it accordingly. Returns result
    /// with value true if should continue or false if exit was requested.
    ///
    pub fn handle_next(&self, state: &mut State) -> Result<bool> {
        match self.rx.recv()? {
            Event::Input(event) => match event {
                KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::CONTROL,
                } => {
                    debug!("Processing exit terminal event '{:?}'...", event);
                    return Ok(false);
                }
                // Handle comment input mode FIRST - all character keys go to comment
                KeyEvent {
                    code: KeyCode::Char(c),
                    modifiers: KeyModifiers::NONE,
                } if state.is_comment_input_mode() => {
                    state.add_comment_char(c);
                }
                KeyEvent {
                    code: KeyCode::Char(c),
                    modifiers: KeyModifiers::SHIFT,
                } if state.is_comment_input_mode() => {
                    state.add_comment_char(c);
                }
                // Handle form input mode - all character keys go to form fields (except h/j/k/l when in dropdowns)
                KeyEvent {
                    code: KeyCode::Char(c),
                    modifiers: KeyModifiers::NONE,
                } if matches!(state.current_view(), crate::state::View::CreateTask | crate::state::View::EditTask)
                    && !matches!(state.get_edit_form_state(), Some(crate::state::EditFormState::Assignee) | Some(crate::state::EditFormState::Section))
                    && !matches!(c, 'h' | 'j' | 'k' | 'l') => {
                    // Route to appropriate form field
                    match state.get_edit_form_state() {
                        Some(crate::state::EditFormState::Name) => {
                            state.add_form_name_char(c);
                        }
                        Some(crate::state::EditFormState::Notes) => {
                            state.add_form_notes_char(c);
                        }
                        Some(crate::state::EditFormState::DueDate) => {
                            state.add_form_due_on_char(c);
                        }
                        _ => {
                            // Default to Name field if no field is selected
                            state.set_edit_form_state(Some(crate::state::EditFormState::Name));
                            state.add_form_name_char(c);
                        }
                    }
                }
                KeyEvent {
                    code: KeyCode::Char(c),
                    modifiers: KeyModifiers::SHIFT,
                } if matches!(state.current_view(), crate::state::View::CreateTask | crate::state::View::EditTask)
                    && !matches!(state.get_edit_form_state(), Some(crate::state::EditFormState::Assignee) | Some(crate::state::EditFormState::Section))
                    && !matches!(c, 'H' | 'J' | 'K' | 'L') => {
                    // Route to appropriate form field
                    match state.get_edit_form_state() {
                        Some(crate::state::EditFormState::Name) => {
                            state.add_form_name_char(c);
                        }
                        Some(crate::state::EditFormState::Notes) => {
                            state.add_form_notes_char(c);
                        }
                        Some(crate::state::EditFormState::DueDate) => {
                            state.add_form_due_on_char(c);
                        }
                        _ => {
                            // Default to Name field if no field is selected
                            state.set_edit_form_state(Some(crate::state::EditFormState::Name));
                            state.add_form_name_char(c);
                        }
                    }
                }
                // Handle token input in onboarding - all character keys go to token input
                // Clear error when user starts typing
                KeyEvent {
                    code: KeyCode::Char(c),
                    modifiers: KeyModifiers::NONE,
                } if matches!(state.current_view(), crate::state::View::Welcome) 
                    && !state.has_access_token() => {
                    // Clear error when user starts typing
                    if state.get_auth_error().is_some() {
                        state.clear_auth_error();
                    }
                    state.add_access_token_char(c);
                }
                KeyEvent {
                    code: KeyCode::Char(c),
                    modifiers: KeyModifiers::SHIFT,
                } if matches!(state.current_view(), crate::state::View::Welcome) 
                    && !state.has_access_token() => {
                    // Clear error when user starts typing
                    if state.get_auth_error().is_some() {
                        state.clear_auth_error();
                    }
                    state.add_access_token_char(c);
                }
                // Handle assignee search input - all character keys go to search
                KeyEvent {
                    code: KeyCode::Char(c),
                    modifiers: KeyModifiers::NONE,
                } if matches!(state.current_view(), crate::state::View::CreateTask | crate::state::View::EditTask) 
                    && matches!(state.get_edit_form_state(), Some(crate::state::EditFormState::Assignee)) => {
                    state.add_assignee_search_char(c);
                }
                KeyEvent {
                    code: KeyCode::Char(c),
                    modifiers: KeyModifiers::SHIFT,
                } if matches!(state.current_view(), crate::state::View::CreateTask | crate::state::View::EditTask) 
                    && matches!(state.get_edit_form_state(), Some(crate::state::EditFormState::Assignee)) => {
                    state.add_assignee_search_char(c);
                }
                KeyEvent {
                    code: KeyCode::Char('q'),
                    modifiers: KeyModifiers::NONE,
                } => {
                    if state.is_search_mode() {
                        state.add_search_char('q');
                    } else {
                        debug!("Processing exit terminal event '{:?}'...", event);
                        return Ok(false);
                    }
                }
                KeyEvent {
                    code: KeyCode::Esc,
                    modifiers: KeyModifiers::NONE,
                } => {
                    if state.is_search_mode() {
                        debug!("Processing exit search mode event '{:?}'...", event);
                        state.exit_search_mode();
                    } else if state.is_debug_mode() {
                        debug!("Processing exit debug mode (Esc) event '{:?}'...", event);
                        state.exit_debug_mode();
                    } else if state.has_delete_confirmation() {
                        debug!("Processing cancel delete confirmation event '{:?}'...", event);
                        state.cancel_delete_confirmation();
                    } else if state.has_move_task() {
                        debug!("Processing cancel move task event '{:?}'...", event);
                        state.clear_move_task();
                    } else if state.is_comment_input_mode() {
                        debug!("Processing cancel comment input event '{:?}'...", event);
                        state.exit_comment_input_mode();
                    } else if *state.current_focus() == Focus::View {
                        debug!("Processing view navigation (Esc) event '{:?}'...", event);
                        // Pop the current view to go back
                        if let Some(popped_view) = state.pop_view() {
                            debug!("Popped view: {:?}, remaining views: {}", popped_view, state.view_stack_len());
                            
                            // If we're going back from a detail/edit/create view, refresh the parent view
                            match popped_view {
                                crate::state::View::TaskDetail | 
                                crate::state::View::EditTask | 
                                crate::state::View::CreateTask | 
                                crate::state::View::KanbanBoard => {
                                    // Refresh the parent view (usually ProjectTasks)
                                    if matches!(state.current_view(), crate::state::View::ProjectTasks) {
                                        state.dispatch(crate::events::network::Event::ProjectTasks);
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
                KeyEvent {
                    code: KeyCode::Char('h'),
                    modifiers: KeyModifiers::NONE,
                } => {
                    if state.is_search_mode() {
                        state.add_search_char('h');
                    } else if matches!(state.current_view(), crate::state::View::CreateTask | crate::state::View::EditTask)
                        && !matches!(state.get_edit_form_state(), Some(crate::state::EditFormState::Assignee) | Some(crate::state::EditFormState::Section)) {
                        // In form text fields, allow typing 'h'
                        match state.get_edit_form_state() {
                            Some(crate::state::EditFormState::Name) => {
                                state.add_form_name_char('h');
                            }
                            Some(crate::state::EditFormState::Notes) => {
                                state.add_form_notes_char('h');
                            }
                            Some(crate::state::EditFormState::DueDate) => {
                                state.add_form_due_on_char('h');
                            }
                            _ => {}
                        }
                    } else if matches!(state.current_view(), crate::state::View::TaskDetail) {
                        // In task detail view, navigate to previous panel
                        debug!("Processing previous task panel event '{:?}'...", event);
                        state.previous_task_panel();
                    } else if matches!(state.current_view(), crate::state::View::ProjectTasks)
                        && *state.current_focus() == Focus::View {
                        // In kanban view with View focus, navigate to previous column
                        debug!("Processing previous kanban column event '{:?}'...", event);
                        state.previous_kanban_column();
                    } else {
                        // In normal mode (Welcome view) or Menu focus, navigate menu
                        match state.current_focus() {
                    Focus::Menu => {
                        debug!("Processing previous menu event '{:?}'...", event);
                        state.previous_menu();
                    }
                    Focus::View => {
                        // If we're in Welcome view but focus is View, switch to Menu focus
                        if matches!(state.current_view(), crate::state::View::Welcome) {
                            state.focus_menu();
                            state.previous_menu();
                        }
                    }
                        }
                    }
                }
                KeyEvent {
                    code: KeyCode::Char('l'),
                    modifiers: KeyModifiers::NONE,
                } => {
                    if state.is_search_mode() {
                        state.add_search_char('l');
                    } else if matches!(state.current_view(), crate::state::View::CreateTask | crate::state::View::EditTask)
                        && !matches!(state.get_edit_form_state(), Some(crate::state::EditFormState::Assignee) | Some(crate::state::EditFormState::Section)) {
                        // In form text fields, allow typing 'l'
                        match state.get_edit_form_state() {
                            Some(crate::state::EditFormState::Name) => {
                                state.add_form_name_char('l');
                            }
                            Some(crate::state::EditFormState::Notes) => {
                                state.add_form_notes_char('l');
                            }
                            Some(crate::state::EditFormState::DueDate) => {
                                state.add_form_due_on_char('l');
                            }
                            _ => {}
                        }
                    } else if matches!(state.current_view(), crate::state::View::TaskDetail) {
                        // In task detail view, navigate to next panel
                        debug!("Processing next task panel event '{:?}'...", event);
                        state.next_task_panel();
                    } else if matches!(state.current_view(), crate::state::View::ProjectTasks)
                        && *state.current_focus() == Focus::View {
                        // In kanban view with View focus, navigate to next column
                        debug!("Processing next kanban column event '{:?}'...", event);
                        state.next_kanban_column();
                    } else {
                        // In normal mode (Welcome view) or Menu focus, navigate menu
                        match state.current_focus() {
                    Focus::Menu => {
                        debug!("Processing next menu event '{:?}'...", event);
                        state.next_menu();
                    }
                    Focus::View => {
                        // If we're in Welcome view but focus is View, switch to Menu focus
                        if matches!(state.current_view(), crate::state::View::Welcome) {
                            state.focus_menu();
                            state.next_menu();
                        }
                    }
                        }
                    }
                }
                KeyEvent {
                    code: KeyCode::Char('k'),
                    modifiers: KeyModifiers::NONE,
                } => {
                    if state.is_search_mode() {
                        state.add_search_char('k');
                    } else if matches!(state.current_view(), crate::state::View::CreateTask | crate::state::View::EditTask)
                        && !matches!(state.get_edit_form_state(), Some(crate::state::EditFormState::Assignee) | Some(crate::state::EditFormState::Section)) {
                        // In form text fields, allow typing 'k'
                        match state.get_edit_form_state() {
                            Some(crate::state::EditFormState::Name) => {
                                state.add_form_name_char('k');
                            }
                            Some(crate::state::EditFormState::Notes) => {
                                state.add_form_notes_char('k');
                            }
                            Some(crate::state::EditFormState::DueDate) => {
                                state.add_form_due_on_char('k');
                            }
                            _ => {}
                        }
                    } else if state.is_debug_mode() {
                        debug!("Processing previous debug event '{:?}'...", event);
                        state.previous_debug();
                    } else if state.is_comment_input_mode() {
                        // In comment input mode, 'k' should be typed, not scroll
                        state.add_comment_char('k');
                    } else if matches!(state.current_view(), crate::state::View::TaskDetail) {
                        // In task detail view, handle based on active panel
                        if state.get_current_task_panel() == crate::state::TaskDetailPanel::Comments {
                            // Scroll comments up
                            debug!("Processing scroll comments up event '{:?}'...", event);
                            state.scroll_comments_up();
                        } else {
                            // Navigate to previous task in filtered list
                            let filtered = state.get_filtered_tasks();
                            if let Some(current_task) = state.get_task_detail() {
                                // Find current task index in filtered list
                                if let Some(current_idx) = filtered.iter().position(|t| t.gid == current_task.gid) {
                                    let prev_idx = if current_idx > 0 {
                                        current_idx - 1
                                    } else {
                                        filtered.len().saturating_sub(1)
                                    };
                                    if prev_idx < filtered.len() && !filtered.is_empty() {
                                        let task = &filtered[prev_idx];
                                        state.dispatch(crate::events::network::Event::GetTaskDetail {
                                            gid: task.gid.clone(),
                                        });
                                    }
                                }
                            }
                        }
                    } else if matches!(state.current_view(), crate::state::View::ProjectTasks) {
                        // In kanban view, navigate to previous task
                        debug!("Processing previous kanban task event '{:?}'...", event);
                        state.previous_kanban_task();
                    } else if matches!(state.current_view(), crate::state::View::CreateTask | crate::state::View::EditTask) {
                        // Handle dropdown navigation in forms
                        match state.get_edit_form_state() {
                            Some(crate::state::EditFormState::Assignee) => {
                                debug!("Processing previous assignee event '{:?}'...", event);
                                state.previous_assignee();
                            }
                            Some(crate::state::EditFormState::Section) => {
                                debug!("Processing previous section event '{:?}'...", event);
                                state.previous_section();
                            }
                            _ => {}
                        }
                    } else if state.has_move_task() {
                        // In move task modal, navigate sections
                        debug!("Processing previous section in move modal event '{:?}'...", event);
                        state.previous_section();
                    } else {
                        match state.current_focus() {
                    Focus::Menu => {
                        debug!("Processing previous menu item event '{:?}'...", event);
                        match state.current_menu() {
                            Menu::Status => (),
                            Menu::Shortcuts => {
                                state.previous_shortcut_index();
                            }
                            Menu::TopList => {
                                state.previous_top_list_index();
                            }
                        }
                    }
                            Focus::View => {
                                debug!("Processing previous task event '{:?}'...", event);
                                state.previous_task_index();
                            }
                        }
                    }
                }
                KeyEvent {
                    code: KeyCode::Char('j'),
                    modifiers: KeyModifiers::NONE,
                } => {
                    if state.is_search_mode() {
                        state.add_search_char('j');
                    } else if matches!(state.current_view(), crate::state::View::CreateTask | crate::state::View::EditTask)
                        && !matches!(state.get_edit_form_state(), Some(crate::state::EditFormState::Assignee) | Some(crate::state::EditFormState::Section)) {
                        // In form text fields, allow typing 'j'
                        match state.get_edit_form_state() {
                            Some(crate::state::EditFormState::Name) => {
                                state.add_form_name_char('j');
                            }
                            Some(crate::state::EditFormState::Notes) => {
                                state.add_form_notes_char('j');
                            }
                            Some(crate::state::EditFormState::DueDate) => {
                                state.add_form_due_on_char('j');
                            }
                            _ => {}
                        }
                    } else if state.is_debug_mode() {
                        debug!("Processing next debug event '{:?}'...", event);
                        state.next_debug();
                    } else if state.is_comment_input_mode() {
                        // In comment input mode, 'j' should be typed, not scroll
                        state.add_comment_char('j');
                    } else if matches!(state.current_view(), crate::state::View::TaskDetail) {
                        // In task detail view, handle based on active panel
                        if state.get_current_task_panel() == crate::state::TaskDetailPanel::Comments {
                            // Scroll comments down
                            debug!("Processing scroll comments down event '{:?}'...", event);
                            state.scroll_comments_down();
                        } else {
                            // Navigate to next task in filtered list
                            let filtered = state.get_filtered_tasks();
                            if let Some(current_task) = state.get_task_detail() {
                                // Find current task index in filtered list
                                if let Some(current_idx) = filtered.iter().position(|t| t.gid == current_task.gid) {
                                    let next_idx = if filtered.is_empty() {
                                        0
                                    } else {
                                        (current_idx + 1) % filtered.len()
                                    };
                                    if next_idx < filtered.len() && !filtered.is_empty() {
                                        let task = &filtered[next_idx];
                                        state.dispatch(crate::events::network::Event::GetTaskDetail {
                                            gid: task.gid.clone(),
                                        });
                                    }
                                }
                            }
                        }
                    } else if matches!(state.current_view(), crate::state::View::ProjectTasks) {
                        // In kanban view, navigate to next task
                        debug!("Processing next kanban task event '{:?}'...", event);
                        state.next_kanban_task();
                    } else if matches!(state.current_view(), crate::state::View::CreateTask | crate::state::View::EditTask) {
                        // Handle dropdown navigation in forms
                        match state.get_edit_form_state() {
                            Some(crate::state::EditFormState::Assignee) => {
                                debug!("Processing next assignee event '{:?}'...", event);
                                state.next_assignee();
                            }
                            Some(crate::state::EditFormState::Section) => {
                                debug!("Processing next section event '{:?}'...", event);
                                state.next_section();
                            }
                            _ => {}
                        }
                    } else if state.has_move_task() {
                        // In move task modal, navigate sections
                        debug!("Processing next section in move modal event '{:?}'...", event);
                        state.next_section();
                    } else {
                        match state.current_focus() {
                    Focus::Menu => {
                        debug!("Processing next menu item event '{:?}'...", event);
                        match state.current_menu() {
                            Menu::Status => (),
                            Menu::Shortcuts => {
                                state.next_shortcut_index();
                            }
                            Menu::TopList => {
                                state.next_top_list_index();
                            }
                        }
                    }
                            Focus::View => {
                                debug!("Processing next task event '{:?}'...", event);
                                state.next_task_index();
                            }
                        }
                    }
                }
                KeyEvent {
                    code: KeyCode::Enter,
                    modifiers: KeyModifiers::NONE,
                } => {
                    // Handle onboarding screen - submit access token
                    if matches!(state.current_view(), crate::state::View::Welcome) && !state.has_access_token() {
                        let token = state.get_access_token_input().to_string();
                        if !token.trim().is_empty() {
                            debug!("Submitting access token from onboarding screen...");
                            state.dispatch(crate::events::network::Event::SetAccessToken { token });
                        }
                    } else if state.is_search_mode() {
                        debug!("Processing exit search mode (Enter) event '{:?}'...", event);
                        state.exit_search_mode();
                    } else if state.is_debug_mode() {
                        debug!("Processing exit debug mode (Enter) event '{:?}'...", event);
                        state.exit_debug_mode();
                    } else if state.has_move_task() {
                        // In move task modal, Enter selects the section and moves the task
                        if let Some(task_gid) = state.get_move_task_gid() {
                            let sections = state.get_sections();
                            let selected_index = state.get_section_dropdown_index();
                            if selected_index < sections.len() {
                                let selected_section = &sections[selected_index];
                                debug!("Moving task {} to section {}...", task_gid, selected_section.gid);
                                state.dispatch(crate::events::network::Event::MoveTaskToSection {
                                    task_gid: task_gid.clone(),
                                    section_gid: selected_section.gid.clone(),
                                });
                                state.clear_move_task();
                            }
                        }
                    } else if state.has_delete_confirmation() {
                        // Confirm delete - call delete_selected_task again to actually delete
                        debug!("Processing confirm delete event '{:?}'...", event);
                        state.delete_selected_task();
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
                    } else if matches!(state.current_view(), crate::state::View::CreateTask | crate::state::View::EditTask) {
                        // Handle form Enter key
                        match state.get_edit_form_state() {
                            Some(crate::state::EditFormState::Name) => {
                                state.set_edit_form_state(Some(crate::state::EditFormState::Notes));
                            }
                            Some(crate::state::EditFormState::Notes) => {
                                state.set_edit_form_state(Some(crate::state::EditFormState::Assignee));
                                state.init_assignee_dropdown_index();
                            }
                            Some(crate::state::EditFormState::Assignee) => {
                                // Select the current assignee and move to next field
                                state.select_current_assignee();
                                state.set_edit_form_state(Some(crate::state::EditFormState::DueDate));
                            }
                            Some(crate::state::EditFormState::DueDate) => {
                                state.set_edit_form_state(Some(crate::state::EditFormState::Section));
                                state.init_section_dropdown_index();
                            }
                            Some(crate::state::EditFormState::Section) => {
                                // Select the current section and stay in Section field
                                // User must press 's' to submit or Esc to cancel
                                state.select_current_section();
                            }
                            None => {
                                state.set_edit_form_state(Some(crate::state::EditFormState::Name));
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
                                if matches!(state.current_view(), crate::state::View::ProjectTasks) {
                                    if state.get_view_mode() == crate::state::ViewMode::Kanban {
                                        // Kanban view: get selected task from kanban
                                        if let Some(task) = state.get_kanban_selected_task() {
                                            state.dispatch(crate::events::network::Event::GetTaskDetail {
                                                gid: task.gid.clone(),
                                            });
                                            state.push_view(crate::state::View::TaskDetail);
                                            state.focus_view();
                                        }
                                    } else {
                                        // List view: get selected task from list
                                        let filtered = state.get_filtered_tasks();
                                        if let Some(selected_index) = state.get_tasks_list_state().selected() {
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
                                } else if matches!(state.current_view(), crate::state::View::KanbanBoard) {
                                    // Kanban board view: view task details
                                    if let Some(task) = state.get_kanban_selected_task() {
                                        state.dispatch(crate::events::network::Event::GetTaskDetail {
                                            gid: task.gid.clone(),
                                        });
                                        state.push_view(crate::state::View::TaskDetail);
                                        state.focus_view();
                                    }
                                }
                            }
                        }
                    }
                }
                KeyEvent {
                    code: KeyCode::Char(' '),
                    modifiers: KeyModifiers::NONE,
                } => {
                    if !state.is_search_mode() {
                        match state.current_focus() {
                            Focus::View => {
                                debug!("Processing toggle task completion event '{:?}'...", event);
                                state.toggle_task_completion();
                            }
                            _ => {}
                        }
                    } else {
                        state.add_search_char(' ');
                    }
                }
                KeyEvent {
                    code: KeyCode::Char('x'),
                    modifiers: KeyModifiers::NONE,
                } => {
                    if !state.is_search_mode() {
                        match state.current_focus() {
                            Focus::View => {
                                debug!("Processing toggle task completion event '{:?}'...", event);
                                state.toggle_task_completion();
                            }
                            _ => {}
                        }
                    } else {
                        state.add_search_char('x');
                    }
                }
                KeyEvent {
                    code: KeyCode::Char('d'),
                    modifiers: KeyModifiers::NONE,
                } => {
                    if state.is_search_mode() {
                        // In search mode, add to search query
                        state.add_search_char('d');
                    } else if state.is_comment_input_mode() {
                        state.add_comment_char('d');
                    } else if state.is_debug_mode() {
                        debug!("Processing exit debug mode (d) event '{:?}'...", event);
                        state.exit_debug_mode();
                    } else {
                        // Check if we should enter debug mode or delete task
                        match state.current_focus() {
                            Focus::View => {
                                if matches!(state.current_view(), crate::state::View::TaskDetail) {
                                    // Delete task from detail view
                                    if let Some(task) = state.get_task_detail() {
                                        state.set_delete_confirmation(task.gid.clone());
                                    }
                                } else {
                                    debug!("Processing delete task event '{:?}'...", event);
                                    state.delete_selected_task();
                                }
                            }
                            _ => {
                                // Enter debug mode when not in View focus
                                debug!("Processing enter debug mode (d) event '{:?}'...", event);
                                state.enter_debug_mode();
                            }
                        }
                    }
                }
                KeyEvent {
                    code: KeyCode::Char('e'),
                    modifiers: KeyModifiers::NONE,
                } => {
                    if state.is_search_mode() {
                        state.add_search_char('e');
                    } else if state.is_comment_input_mode() {
                        state.add_comment_char('e');
                    } else if matches!(state.current_focus(), Focus::View) {
                        if matches!(state.current_view(), crate::state::View::TaskDetail) {
                            // Edit task from detail view
                            debug!("Processing edit task event '{:?}'...", event);
                            // Clone task data to avoid borrow checker issues
                            let task_data = state.get_task_detail().map(|t| (
                                t.name.clone(),
                                t.notes.clone(),
                                t.assignee.as_ref().map(|a| a.gid.clone()),
                                t.due_on.clone(),
                                t.section.as_ref().map(|s| s.gid.clone()),
                            ));
                            
                            if let Some((name, notes, assignee_gid, due_on, section_gid)) = task_data {
                                // Pre-populate form with task data
                                state.set_form_name(name);
                                state.set_form_notes(notes.unwrap_or_default());
                                state.set_form_assignee(assignee_gid);
                                state.set_form_due_on(due_on.unwrap_or_default());
                                state.set_form_section(section_gid);
                                state.set_edit_form_state(Some(crate::state::EditFormState::Name));
                                
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
                        }
                    }
                }
                KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::NONE,
                } => {
                    if state.is_search_mode() {
                        state.add_search_char('c');
                    } else if state.is_comment_input_mode() {
                        state.add_comment_char('c');
                    } else if matches!(state.current_focus(), Focus::View) {
                        if matches!(state.current_view(), crate::state::View::TaskDetail) {
                            // Add comment from detail view - switch to Comments panel first
                            debug!("Processing add comment event '{:?}'...", event);
                            state.set_current_task_panel(crate::state::TaskDetailPanel::Comments);
                            state.enter_comment_input_mode();
                        }
                    }
                }
                KeyEvent {
                    code: KeyCode::Char('s'),
                    modifiers: KeyModifiers::NONE,
                } => {
                    if state.is_search_mode() {
                        debug!("Processing search character 's' event '{:?}'...", event);
                        state.add_search_char('s');
                    } else if matches!(state.current_view(), crate::state::View::CreateTask | crate::state::View::EditTask) {
                        // Submit form
                        // Make sure section is selected if we're in the section field - do this first
                        let is_section_field = matches!(state.get_edit_form_state(), Some(crate::state::EditFormState::Section));
                        if is_section_field {
                            state.select_current_section();
                        }
                        // Now get all the values we need
                        if matches!(state.current_view(), crate::state::View::CreateTask) {
                            // Create task
                            let project_gid_opt = state.get_project().map(|p| p.gid.clone());
                            let name = state.get_form_name().to_string();
                            if !name.trim().is_empty() {
                                if let Some(project_gid) = project_gid_opt {
                                    let notes = state.get_form_notes().to_string();
                                    let assignee = state.get_form_assignee().cloned();
                                    let due_on = if state.get_form_due_on().is_empty() {
                                        None
                                    } else {
                                        Some(state.get_form_due_on().to_string())
                                    };
                                    let section = state.get_form_section().cloned();
                                    state.dispatch(crate::events::network::Event::CreateTask {
                                        project_gid,
                                        name,
                                        notes: Some(notes),
                                        assignee,
                                        due_on,
                                        section,
                                    });
                                    state.clear_form();
                                    state.pop_view();
                                }
                            }
                        } else if matches!(state.current_view(), crate::state::View::EditTask) {
                            // Update task - only send fields that have changed
                            let task_gid_opt = state.get_task_detail().map(|t| t.gid.clone());
                            if let Some(task_gid) = task_gid_opt {
                                // Compare current values with original values - clone all values first
                                let original_name = state.get_original_form_name().to_string();
                                let original_notes = state.get_original_form_notes().to_string();
                                let original_assignee = state.get_original_form_assignee().clone();
                                let original_due_on = state.get_original_form_due_on().to_string();
                                let original_section = state.get_original_form_section().clone();
                                
                                let current_name = state.get_form_name().to_string();
                                let current_notes = state.get_form_notes().to_string();
                                let current_assignee = state.get_form_assignee().cloned();
                                let current_due_on = state.get_form_due_on().to_string();
                                let current_section = state.get_form_section().cloned();
                                
                                // Build update fields, only including changed non-empty values
                                let mut name_val = None;
                                if current_name != original_name && !current_name.trim().is_empty() {
                                    name_val = Some(current_name);
                                }
                                
                                let mut notes_val = None;
                                if current_notes != original_notes && !current_notes.trim().is_empty() {
                                    notes_val = Some(current_notes);
                                }
                                
                                let mut assignee_val = None;
                                {
                                    let current = current_assignee.as_ref();
                                    let original = original_assignee.as_ref();
                                    match (current, original) {
                                        (Some(a), Some(b)) if a.as_str() == b.as_str() => {},
                                        (None, None) => {},
                                        (Some(gid), _) if !gid.trim().is_empty() => {
                                            assignee_val = Some(gid.clone());
                                        },
                                        _ => {},
                                    }
                                }
                                
                                let mut due_on_val = None;
                                if current_due_on != original_due_on && !current_due_on.trim().is_empty() {
                                    due_on_val = Some(current_due_on);
                                }
                                
                                let mut section_val = None;
                                {
                                    let current = current_section.as_ref();
                                    let original = original_section.as_ref();
                                    match (current, original) {
                                        (Some(a), Some(b)) if a.as_str() == b.as_str() => {},
                                        (None, None) => {},
                                        (Some(gid), _) if !gid.trim().is_empty() => {
                                            section_val = Some(gid.clone());
                                        },
                                        _ => {},
                                    }
                                }
                                
                                // Check if any field has changed
                                let has_other_changes = name_val.is_some() 
                                    || notes_val.is_some() 
                                    || assignee_val.is_some() 
                                    || due_on_val.is_some();
                                
                                // Only dispatch if there are actual changes
                                // If only section changed, UpdateTaskFields will handle it via add_task_to_section
                                // without sending a PUT request (handled in asana/mod.rs)
                                if has_other_changes || section_val.is_some() {
                                    state.dispatch(crate::events::network::Event::UpdateTaskFields {
                                        gid: task_gid,
                                        name: name_val,
                                        notes: notes_val,
                                        assignee: assignee_val,
                                        due_on: due_on_val,
                                        section: section_val,
                                        completed: None,
                                    });
                                }
                                
                                state.clear_form();
                                state.pop_view();
                            }
                        }
                    } else {
                        match state.current_focus() {
                            Focus::Menu => {
                                debug!("Processing star/unstar event '{:?}'...", event);
                                match state.current_menu() {
                                    Menu::TopList => {
                                        state.toggle_star_current_project();
                                    }
                                    Menu::Shortcuts => {
                                        // Unstar from shortcuts list (only works for starred projects)
                                        state.unstar_current_shortcut();
                                    }
                                    _ => {}
                                }
                            }
                            _ => {}
                        }
                    }
                }
                KeyEvent {
                    code: KeyCode::Char('/'),
                    modifiers: KeyModifiers::NONE,
                } => {
                    if state.is_search_mode() {
                        debug!("Processing exit search mode (/) event '{:?}'...", event);
                        state.exit_search_mode();
                    } else if state.is_debug_mode() {
                        debug!("Processing exit debug mode (/) event '{:?}'...", event);
                        state.exit_debug_mode();
                    } else {
                        // Check if we should enter search mode or log mode
                        // If focus is on logs area, enter log mode, otherwise search mode
                        // For now, we'll use a different key for log mode - let's use 'l' when not in search
                        // Actually, let's make '/' toggle log mode when not in a searchable area
                        debug!("Processing enter search mode event '{:?}'...", event);
                        state.enter_search_mode();
                    }
                }
                KeyEvent {
                    code: KeyCode::Char('y'),
                    modifiers: KeyModifiers::NONE,
                } => {
                    if state.is_debug_mode() {
                        debug!("Processing copy debug log event '{:?}'...", event);
                        if let Some(debug_entry) = state.get_current_debug() {
                            // Copy to clipboard
                            match ClipboardContext::new() {
                                Ok(mut ctx) => match ctx.set_contents(debug_entry.to_string()) {
                                    Ok(_) => {
                                        info!("Debug log entry copied to clipboard");
                                    }
                                    Err(e) => {
                                        warn!("Failed to copy to clipboard: {}", e);
                                    }
                                },
                                Err(e) => {
                                    warn!("Failed to initialize clipboard: {}", e);
                                }
                            }
                        }
                    } else if !state.is_search_mode() {
                        debug!("Skipping 'y' key when not in debug mode");
                    }
                }
                KeyEvent {
                    code: KeyCode::Char('f'),
                    modifiers: KeyModifiers::NONE,
                } => {
                    if state.is_search_mode() {
                        state.add_search_char('f');
                    }
                }
                KeyEvent {
                    code: KeyCode::Char('m'),
                    modifiers: KeyModifiers::NONE,
                } => {
                    if state.is_search_mode() {
                        state.add_search_char('m');
                    } else if !state.is_debug_mode() {
                        match state.current_focus() {
                            Focus::View => {
                                if matches!(state.current_view(), crate::state::View::ProjectTasks) {
                                    // Open section selection modal for moving task
                                    if let Some(task) = state.get_kanban_selected_task() {
                                        debug!("Opening move task modal for task {}...", task.gid);
                                        state.set_move_task_gid(Some(task.gid.clone()));
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                KeyEvent {
                    code: KeyCode::Char('n'),
                    modifiers: KeyModifiers::NONE,
                } => {
                    if state.is_search_mode() {
                        state.add_search_char('n');
                    } else if !state.is_debug_mode() {
                        match state.current_focus() {
                            Focus::View => {
                                if matches!(state.current_view(), crate::state::View::ProjectTasks | crate::state::View::TaskDetail) {
                                    // Enter create task view
                                    debug!("Processing create task event '{:?}'...", event);
                                    // Initialize form state
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
                                    }
                                    state.push_view(crate::state::View::CreateTask);
                                    state.focus_view();
                                }
                            }
                            _ => {}
                        }
                    }
                }
                KeyEvent {
                    code: KeyCode::Backspace,
                    modifiers: KeyModifiers::NONE,
                } => {
                    if state.is_search_mode() {
                        debug!("Processing remove search character event '{:?}'...", event);
                        state.remove_search_char();
                    } else if state.is_comment_input_mode() {
                        debug!("Processing remove comment character event '{:?}'...", event);
                        state.remove_comment_char();
                    } else if matches!(state.current_view(), crate::state::View::Welcome) 
                        && !state.has_access_token() {
                        debug!("Processing remove token character event '{:?}'...", event);
                        // Clear error when user edits
                        if state.get_auth_error().is_some() {
                            state.clear_auth_error();
                        }
                        state.backspace_access_token();
                    } else if matches!(state.current_view(), crate::state::View::CreateTask | crate::state::View::EditTask) {
                        // Handle form backspace
                        match state.get_edit_form_state() {
                            Some(crate::state::EditFormState::Name) => {
                                state.remove_form_name_char();
                            }
                            Some(crate::state::EditFormState::Notes) => {
                                state.remove_form_notes_char();
                            }
                            Some(crate::state::EditFormState::Assignee) => {
                                state.backspace_assignee_search();
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
                } => {
                    if !state.is_search_mode() && !state.is_comment_input_mode() {
                        if matches!(state.current_view(), crate::state::View::CreateTask | crate::state::View::EditTask) {
                            let next_state = match state.get_edit_form_state() {
                                Some(crate::state::EditFormState::Name) => crate::state::EditFormState::Notes,
                                Some(crate::state::EditFormState::Notes) => crate::state::EditFormState::Assignee,
                                Some(crate::state::EditFormState::Assignee) => crate::state::EditFormState::DueDate,
                                Some(crate::state::EditFormState::DueDate) => crate::state::EditFormState::Section,
                                Some(crate::state::EditFormState::Section) => crate::state::EditFormState::Name,
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
                }
                KeyEvent {
                    code: KeyCode::BackTab,
                    modifiers: KeyModifiers::SHIFT,
                } => {
                    if !state.is_search_mode() && !state.is_comment_input_mode() {
                        if matches!(state.current_view(), crate::state::View::CreateTask | crate::state::View::EditTask) {
                            let prev_state = match state.get_edit_form_state() {
                                Some(crate::state::EditFormState::Name) => crate::state::EditFormState::Section,
                                Some(crate::state::EditFormState::Notes) => crate::state::EditFormState::Name,
                                Some(crate::state::EditFormState::Assignee) => crate::state::EditFormState::Notes,
                                Some(crate::state::EditFormState::DueDate) => crate::state::EditFormState::Assignee,
                                Some(crate::state::EditFormState::Section) => crate::state::EditFormState::DueDate,
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
                }
                KeyEvent {
                    code: KeyCode::Char(c),
                    modifiers: KeyModifiers::NONE,
                } => {
                    if state.is_search_mode() {
                        debug!("Processing search character '{}' event '{:?}'...", c, event);
                        state.add_search_char(c);
                    } else if state.is_comment_input_mode() {
                        debug!("Processing comment character '{}' event '{:?}'...", c, event);
                        state.add_comment_char(c);
                    } else {
                        debug!("Skipping processing of terminal event '{:?}'...", event);
                    }
                }
                _ => {
                    if !state.is_search_mode() {
                    debug!("Skipping processing of terminal event '{:?}'...", event);
                    }
                }
            },
            Event::Tick => {
                state.advance_spinner_index();
            }
        }
        Ok(true)
    }
}
