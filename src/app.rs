//! Application core module.
//!
//! This module contains the main `App` struct that orchestrates the application lifecycle,
//! including state management, event handling, terminal setup, and UI rendering.

use crate::asana::Asana;
use crate::config::Config;
use crate::error::{AppError, AppResult};
use crate::events::network::{Event as NetworkEvent, Handler as NetworkEventHandler};
use crate::events::terminal::Handler as TerminalEventHandler;
use crate::state::State;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use log::*;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::io::{self, stdout};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::sync::RwLock;
use tui_logger::{init_logger, set_default_level};

pub type NetworkEventSender = std::sync::mpsc::Sender<NetworkEvent>;
type NetworkEventReceiver = std::sync::mpsc::Receiver<NetworkEvent>;
pub type ConfigSaveSender = std::sync::mpsc::Sender<()>;
type ConfigSaveReceiver = std::sync::mpsc::Receiver<()>;

/// Oversees event processing, state management, and terminal output.
///
pub struct App {
    access_token: Option<String>, // Make optional to handle onboarding
    state: Arc<RwLock<State>>,
    config: Config,
}

impl App {
    /// Start a new application according to the given configuration. Returns
    /// the result of the application execution.
    ///
    pub async fn start(config: Config) -> AppResult<()> {
        // Create state first so we can capture logs to it
        let (tx, rx) = std::sync::mpsc::channel::<NetworkEvent>();
        let (config_save_tx, config_save_rx) = std::sync::mpsc::channel::<()>();
        let starred_projects = config.starred_projects.clone();
        let starred_project_names = config.starred_project_names.clone();
        let has_access_token = config.access_token.is_some();
        let access_token = config.access_token.clone();

        // Load theme from config
        let theme = crate::ui::Theme::from_name(&config.theme_name)
            .unwrap_or_else(crate::ui::Theme::default); // Theme fallback is safe

        let state = Arc::new(RwLock::new(State::new(
            tx.clone(),
            config_save_tx.clone(),
            starred_projects,
            starred_project_names,
            has_access_token,
            theme,
            config.hotkeys.clone(),
        )));

        // Set up log capture to state BEFORE initializing tui_logger
        // We'll create a custom logger that captures logs
        // Use a channel to avoid blocking on state lock
        let (log_tx, log_rx) = std::sync::mpsc::channel::<String>();
        let log_tx_for_shutdown = log_tx.clone(); // Keep a copy for shutdown

        // Spawn a task to process log entries and add them to state
        // Use spawn_blocking since log_rx is a blocking receiver
        let state_for_log_processor = Arc::clone(&state);
        tokio::task::spawn_blocking(move || {
            let handle = tokio::runtime::Handle::current();
            while let Ok(log_entry) = log_rx.recv() {
                handle.block_on(async {
                    let mut state = state_for_log_processor.write().await;
                    state.add_log_entry(log_entry);
                });
            }
        });

        let custom_logger = crate::logger::CustomLogger::new();
        custom_logger.set_log_callback(Box::new(move |log_entry: String| {
            // Send log entry through channel (non-blocking)
            let _ = log_tx.send(log_entry);
        }));

        // Set our custom logger first (before init_logger)
        // This will capture all logs to state
        log::set_logger(Box::leak(Box::new(custom_logger)))
            .map_err(|e| AppError::Logger(format!("Failed to set logger: {}", e)))?;
        log::set_max_level(LevelFilter::Trace);

        // Now try to initialize tui_logger - it will fail because we already set the logger
        // We'll catch the panic and continue. Since we're using our own list widget now,
        // we don't need tui_logger's widget, but we'll try to initialize it anyway.
        // If it fails, that's fine - our custom logger will handle everything.
        let init_result = std::panic::catch_unwind(|| init_logger(LevelFilter::Info));

        match init_result {
            Ok(Ok(_)) => {
                // init_logger succeeded (unlikely but possible)
                set_default_level(LevelFilter::Trace);
            }
            Ok(Err(_)) => {
                // init_logger returned an error (expected)
                debug!("tui_logger init skipped (using custom logger instead)");
            }
            Err(_) => {
                // init_logger panicked (also expected)
                debug!("tui_logger init panicked (using custom logger instead)");
            }
        }

        info!("Starting application...");

        let mut app = App {
            access_token,
            state,
            config,
        };

        // Create shutdown flag for graceful shutdown
        let shutdown = Arc::new(AtomicBool::new(false));

        // Always start network thread - it will handle SetAccessToken event
        // Use actual token if available, or empty string as placeholder
        let initial_token = app.access_token.clone().unwrap_or_default();
        app.start_network_with_token(rx, initial_token, shutdown.clone())?;

        app.start_config_saver(config_save_rx, shutdown.clone());

        // Store senders so we can drop them on exit to signal shutdown to background tasks
        let network_sender = tx.clone();
        let config_save_sender = config_save_tx.clone();

        let result = app.start_ui(tx).await;

        // Signal shutdown to all background tasks
        shutdown.store(true, Ordering::Relaxed);

        // Drop senders to signal background tasks to exit
        // This will cause receivers to return errors and exit their loops
        drop(network_sender);
        drop(config_save_sender);
        drop(log_tx_for_shutdown); // Also close log channel

        // Give background tasks a moment to exit gracefully
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        result?;

        // Save config on exit
        {
            let state = app.state.read().await;
            app.config.starred_projects = state.get_starred_project_gids();
            app.config.starred_project_names = state.get_starred_project_names();
            app.config.theme_name = state.get_theme().name.clone();
            app.config.hotkeys = state.get_hotkeys().clone();
            if let Err(e) = app.config.save() {
                error!("Failed to save config on exit: {}", e);
            }
        }

        info!("Exiting application...");

        std::process::exit(0);
    }

    /// Start a thread to handle config save requests.
    ///
    fn start_config_saver(&self, receiver: ConfigSaveReceiver, shutdown: Arc<AtomicBool>) {
        let state: Arc<tokio::sync::RwLock<State>> = Arc::clone(&self.state);
        let mut config = self.config.clone();
        std::thread::spawn(move || {
            loop {
                // Check for shutdown signal
                if shutdown.load(Ordering::Relaxed) {
                    debug!("Config saver received shutdown signal");
                    break;
                }

                match receiver.recv_timeout(std::time::Duration::from_millis(100)) {
                    Ok(_) => {
                        if let Ok(state_guard) = state.try_read() {
                            config.starred_projects = state_guard.get_starred_project_gids();
                            config.starred_project_names = state_guard.get_starred_project_names();
                            config.theme_name = state_guard.get_theme().name.clone();
                            config.hotkeys = state_guard.get_hotkeys().clone();
                            if let Err(e) = config.save() {
                                error!("Failed to save config: {}", e);
                            }
                        }
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        // Timeout, check shutdown again
                        continue;
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                        // Channel closed, exit gracefully
                        debug!("Config saver channel closed");
                        break;
                    }
                }
            }
            debug!("Config saver thread exiting");
        });
    }

    fn start_network_with_token(
        &self,
        net_receiver: NetworkEventReceiver,
        initial_token: String,
        shutdown: Arc<AtomicBool>,
    ) -> AppResult<()> {
        debug!("Starting asynchronous networking task...");
        let cloned_state = Arc::clone(&self.state);
        // Use spawn_blocking to bridge the blocking receiver to async context
        tokio::task::spawn_blocking(move || {
            let handle = tokio::runtime::Handle::current();
            let mut asana = Asana::new(&initial_token);
            handle.block_on(async {
                // Bridge blocking receiver to async channel
                let (async_tx, mut async_rx) = tokio::sync::mpsc::unbounded_channel();
                let async_tx_for_receiver = async_tx.clone();
                let shutdown_for_receiver = shutdown.clone();
                let receiver_handle = tokio::task::spawn_blocking(move || {
                    loop {
                        // Check for shutdown signal
                        if shutdown_for_receiver.load(Ordering::Relaxed) {
                            debug!("Network receiver received shutdown signal");
                            break;
                        }

                        match net_receiver.recv_timeout(std::time::Duration::from_millis(100)) {
                            Ok(event) => {
                                if async_tx_for_receiver.send(event).is_err() {
                                    debug!("Async channel closed, exiting network receiver");
                                    break;
                                }
                            }
                            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                                // Timeout, check shutdown again
                                continue;
                            }
                            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                                // Channel closed, exit gracefully
                                debug!("Network receiver channel closed");
                                break;
                            }
                        }
                    }
                });

                // Process events in async context
                let shutdown_for_processor = shutdown.clone();
                loop {
                    // Check for shutdown signal
                    if shutdown_for_processor.load(Ordering::Relaxed) {
                        debug!("Network processor received shutdown signal");
                        break;
                    }

                    tokio::select! {
                        event = async_rx.recv() => {
                            match event {
                                Some(network_event) => {
                                    let event_clone = network_event.clone();
                                    let mut handler = NetworkEventHandler::new(&cloned_state, &mut asana);
                                    match handler.handle(network_event).await {
                                        Ok(_) => (),
                                        Err(e) => {
                                            error!("Failed to handle network event {:?}: {}", event_clone, e);
                                            // Log the full error chain for debugging
                                            use std::error::Error;
                                            let mut source = e.source();
                                            while let Some(err) = source {
                                                error!("  Caused by: {}", err);
                                                source = err.source();
                                            }
                                        }
                                    }
                                }
                                None => {
                                    // Channel closed, exit gracefully
                                    debug!("Network event channel closed");
                                    break;
                                }
                            }
                        }
                        _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {
                            // Periodic check for shutdown
                            if shutdown_for_processor.load(Ordering::Relaxed) {
                                debug!("Network processor cancelled");
                                break;
                            }
                        }
                    }
                }
                // Close the async channel to signal receiver to exit
                drop(async_tx);
                // Wait for receiver task to finish
                let _ = receiver_handle.await;
                debug!("Network task exited cleanly");
            });
        });
        Ok(())
    }

    /// Begin the terminal event poll on a separate thread before starting the
    /// render loop on the main thread. Return the result following an exit
    /// request or unrecoverable error.
    ///
    async fn start_ui(&mut self, net_sender: NetworkEventSender) -> AppResult<()> {
        debug!("Starting user interface on main thread...");
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen)?;
        // Enable mouse capture but allow text selection with modifier keys
        // Most terminals allow Shift+Click or Alt+Click for text selection even with mouse capture enabled
        execute!(stdout, EnableMouseCapture)?;
        enable_raw_mode()?;

        let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;
        terminal.hide_cursor()?;

        // Only send Me event if we have a token (network thread is running with valid token)
        if self.access_token.is_some() {
            net_sender
                .send(NetworkEvent::Me)
                .map_err(|e| AppError::Terminal(format!("Failed to send network event: {}", e)))?;
        }

        let terminal_event_handler = TerminalEventHandler::new();
        loop {
            let mut state = self.state.write().await;
            if let Ok(size) = terminal.backend().size() {
                state.set_terminal_size(size);
            };
            terminal
                .draw(|frame| crate::ui::render(frame, &mut state))
                .map_err(|e| {
                    crate::error::AppError::Terminal(format!("Failed to draw terminal: {}", e))
                })?;
            if !terminal_event_handler
                .handle_next(&mut state)
                .map_err(|e| {
                    crate::error::AppError::Terminal(format!(
                        "Failed to handle terminal event: {}",
                        e
                    ))
                })?
            {
                debug!("Received application exit request.");
                break;
            }
        }

        disable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, LeaveAlternateScreen, DisableMouseCapture)?;

        Ok(())
    }
}
