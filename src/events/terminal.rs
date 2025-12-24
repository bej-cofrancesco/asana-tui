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
                    } else if matches!(state.current_view(), crate::state::View::KanbanBoard) 
                        || (matches!(state.current_view(), crate::state::View::ProjectTasks) 
                            && state.get_view_mode() == crate::state::ViewMode::Kanban) {
                        // Navigate to previous kanban column
                        debug!("Processing previous kanban column event '{:?}'...", event);
                        state.previous_kanban_column();
                    } else {
                        match state.current_focus() {
                    Focus::Menu => {
                        debug!("Processing previous menu event '{:?}'...", event);
                        state.previous_menu();
                    }
                    Focus::View => {}
                        }
                    }
                }
                KeyEvent {
                    code: KeyCode::Char('l'),
                    modifiers: KeyModifiers::NONE,
                } => {
                    if state.is_search_mode() {
                        state.add_search_char('l');
                    } else if state.is_debug_mode() {
                        debug!("Processing next debug event '{:?}'...", event);
                        state.next_debug();
                    } else if matches!(state.current_view(), crate::state::View::KanbanBoard) 
                        || (matches!(state.current_view(), crate::state::View::ProjectTasks) 
                            && state.get_view_mode() == crate::state::ViewMode::Kanban) {
                        // Navigate to next kanban column
                        debug!("Processing next kanban column event '{:?}'...", event);
                        state.next_kanban_column();
                    } else {
                        match state.current_focus() {
                    Focus::Menu => {
                        debug!("Processing next menu event '{:?}'...", event);
                        state.next_menu();
                    }
                    Focus::View => {}
                        }
                    }
                }
                KeyEvent {
                    code: KeyCode::Char('k'),
                    modifiers: KeyModifiers::NONE,
                } => {
                    if state.is_search_mode() {
                        state.add_search_char('k');
                    } else if state.is_debug_mode() {
                        debug!("Processing previous debug event '{:?}'...", event);
                        state.previous_debug();
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
                    } else if state.is_debug_mode() {
                        debug!("Processing next debug event '{:?}'...", event);
                        state.next_debug();
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
                    if state.is_search_mode() {
                        debug!("Processing exit search mode (Enter) event '{:?}'...", event);
                        state.exit_search_mode();
                    } else if state.is_debug_mode() {
                        debug!("Processing exit debug mode (Enter) event '{:?}'...", event);
                        state.exit_debug_mode();
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
                            }
                            Some(crate::state::EditFormState::Assignee) => {
                                // Toggle through assignees or show selection
                                // For now, just move to next field
                                state.set_edit_form_state(Some(crate::state::EditFormState::DueDate));
                            }
                            Some(crate::state::EditFormState::DueDate) => {
                                state.set_edit_form_state(Some(crate::state::EditFormState::Section));
                            }
                            Some(crate::state::EditFormState::Section) | Some(crate::state::EditFormState::Tags) => {
                                // Submit form
                                if matches!(state.current_view(), crate::state::View::CreateTask) {
                                    // Create task
                                    if let Some(project) = state.get_project() {
                                        let name = state.get_form_name().to_string();
                                        if !name.trim().is_empty() {
                                            state.dispatch(crate::events::network::Event::CreateTask {
                                                project_gid: project.gid.clone(),
                                                name,
                                                notes: Some(state.get_form_notes().to_string()),
                                                assignee: state.get_form_assignee().cloned(),
                                                due_on: if state.get_form_due_on().is_empty() {
                                                    None
                                                } else {
                                                    Some(state.get_form_due_on().to_string())
                                                },
                                                section: state.get_form_section().cloned(),
                                            });
                                            state.clear_form();
                                            state.pop_view();
                                        }
                                    }
                                } else if matches!(state.current_view(), crate::state::View::EditTask) {
                                    // Update task
                                    if let Some(task) = state.get_task_detail() {
                                        let name = state.get_form_name().to_string();
                                        if !name.trim().is_empty() {
                                            state.dispatch(crate::events::network::Event::UpdateTaskFields {
                                                gid: task.gid.clone(),
                                                name: Some(name),
                                                notes: Some(state.get_form_notes().to_string()),
                                                assignee: state.get_form_assignee().cloned(),
                                                due_on: if state.get_form_due_on().is_empty() {
                                                    None
                                                } else {
                                                    Some(state.get_form_due_on().to_string())
                                                },
                                                section: state.get_form_section().cloned(),
                                                completed: None,
                                            });
                                            state.clear_form();
                                            state.pop_view();
                                        }
                                    }
                                }
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
                    } else if matches!(state.current_focus(), Focus::View) {
                        if matches!(state.current_view(), crate::state::View::TaskDetail) {
                            // Add comment from detail view
                            debug!("Processing add comment event '{:?}'...", event);
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
                    if !state.is_search_mode() && !state.is_debug_mode() {
                        match state.current_focus() {
                            Focus::View => {
                                debug!("Processing toggle task filter event '{:?}'...", event);
                                state.next_task_filter();
                            }
                            _ => {}
                        }
                    } else if state.is_search_mode() {
                        state.add_search_char('f');
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
                    } else if matches!(state.current_view(), crate::state::View::CreateTask | crate::state::View::EditTask) {
                        // Handle form backspace
                        match state.get_edit_form_state() {
                            Some(crate::state::EditFormState::Name) => {
                                state.remove_form_name_char();
                            }
                            Some(crate::state::EditFormState::Notes) => {
                                state.remove_form_notes_char();
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
                    modifiers: KeyModifiers::SHIFT,
                } => {
                    if !state.is_search_mode() && !state.is_debug_mode() {
                        // Shift+Tab navigation in forms
                        if matches!(state.current_view(), crate::state::View::CreateTask | crate::state::View::EditTask) {
                            let prev_state = match state.get_edit_form_state() {
                                Some(crate::state::EditFormState::Name) => crate::state::EditFormState::Section,
                                Some(crate::state::EditFormState::Notes) => crate::state::EditFormState::Name,
                                Some(crate::state::EditFormState::Assignee) => crate::state::EditFormState::Notes,
                                Some(crate::state::EditFormState::DueDate) => crate::state::EditFormState::Assignee,
                                Some(crate::state::EditFormState::Section) => crate::state::EditFormState::DueDate,
                                Some(crate::state::EditFormState::Tags) => crate::state::EditFormState::Section,
                                None => crate::state::EditFormState::Name,
                            };
                            state.set_edit_form_state(Some(prev_state));
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
                    } else if matches!(state.current_view(), crate::state::View::CreateTask | crate::state::View::EditTask) {
                        // Handle form input
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
                                // Assignee and Section are selected via Enter, not typed
                            }
                        }
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
