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
                    } else if *state.current_focus() == Focus::View {
                        debug!("Processing view cancel/refresh terminal event '{:?}'...", event);
                        // Refetch tasks when escaping from task view
                        if matches!(state.current_view(), crate::state::View::ProjectTasks) {
                            state.dispatch(crate::events::network::Event::ProjectTasks);
                        }
                        state.focus_menu();
                    }
                }
                KeyEvent {
                    code: KeyCode::Char('h'),
                    modifiers: KeyModifiers::NONE,
                } => {
                    if state.is_search_mode() {
                        state.add_search_char('h');
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
                            Focus::View => {}
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
                                debug!("Processing delete task event '{:?}'...", event);
                                state.delete_selected_task();
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
                    code: KeyCode::Backspace,
                    modifiers: KeyModifiers::NONE,
                } => {
                    if state.is_search_mode() {
                        debug!("Processing remove search character event '{:?}'...", event);
                        state.remove_search_char();
                    }
                }
                KeyEvent {
                    code: KeyCode::Char(c),
                    modifiers: KeyModifiers::NONE,
                } => {
                    if state.is_search_mode() {
                        debug!("Processing search character '{}' event '{:?}'...", c, event);
                        state.add_search_char(c);
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
