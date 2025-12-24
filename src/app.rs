use crate::asana::Asana;
use crate::config::Config;
use crate::events::network::{Event as NetworkEvent, Handler as NetworkEventHandler};
use crate::events::terminal::Handler as TerminalEventHandler;
use crate::state::State;
use anyhow::{anyhow, Result};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use log::*;
use std::io::{self, stdout};
use std::sync::Arc;
use tokio::sync::Mutex;
use tui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use tui_logger::{init_logger, set_default_level};

pub type NetworkEventSender = std::sync::mpsc::Sender<NetworkEvent>;
type NetworkEventReceiver = std::sync::mpsc::Receiver<NetworkEvent>;
pub type ConfigSaveSender = std::sync::mpsc::Sender<()>;
type ConfigSaveReceiver = std::sync::mpsc::Receiver<()>;

/// Oversees event processing, state management, and terminal output.
///
pub struct App {
    access_token: Option<String>, // Make optional to handle onboarding
    state: Arc<Mutex<State>>,
    config: Config,
}

impl App {
    /// Start a new application according to the given configuration. Returns
    /// the result of the application execution.
    ///
    pub async fn start(config: Config) -> Result<()> {
        // Create state first so we can capture logs to it
        let (tx, rx) = std::sync::mpsc::channel::<NetworkEvent>();
        let (config_save_tx, config_save_rx) = std::sync::mpsc::channel::<()>();
        let starred_projects = config.starred_projects.clone();
        let starred_project_names = config.starred_project_names.clone();
        let has_access_token = config.access_token.is_some();
        let access_token = config.access_token.clone();

        let state = Arc::new(Mutex::new(State::new(
            tx.clone(),
            config_save_tx.clone(),
            starred_projects,
            starred_project_names,
            has_access_token,
        )));

        // Set up log capture to state BEFORE initializing tui_logger
        // We'll create a custom logger that captures logs
        // Use a channel to avoid blocking on state lock
        let (log_tx, log_rx) = std::sync::mpsc::channel::<String>();

        // Spawn a thread to process log entries and add them to state
        let state_for_log_processor = Arc::clone(&state);
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            while let Ok(log_entry) = log_rx.recv() {
                // Use async lock since state is a tokio::sync::Mutex
                rt.block_on(async {
                    let mut state = state_for_log_processor.lock().await;
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
            .map_err(|e| anyhow!("Failed to set logger: {}", e))?;
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

        // Always start network thread - it will handle SetAccessToken event
        // Use actual token if available, or empty string as placeholder
        let initial_token = app.access_token.clone().unwrap_or_default();
        app.start_network_with_token(rx, initial_token)?;

        app.start_config_saver(config_save_rx);
        app.start_ui(tx).await?;

        // Save config on exit
        {
            let state = app.state.lock().await;
            app.config.starred_projects = state.get_starred_project_gids();
            app.config.starred_project_names = state.get_starred_project_names();
            if let Err(e) = app.config.save() {
                error!("Failed to save config on exit: {}", e);
            }
        }

        info!("Exiting application...");
        Ok(())
    }

    /// Start a thread to handle config save requests.
    ///
    fn start_config_saver(&self, receiver: ConfigSaveReceiver) {
        let state = Arc::clone(&self.state);
        let mut config = self.config.clone();
        std::thread::spawn(move || {
            while receiver.recv().is_ok() {
                if let Ok(state_guard) = state.try_lock() {
                    config.starred_projects = state_guard.get_starred_project_gids();
                    config.starred_project_names = state_guard.get_starred_project_names();
                    if let Err(e) = config.save() {
                        error!("Failed to save config: {}", e);
                    }
                }
            }
        });
    }

    /// Start a separate thread for asynchronous state mutations.
    ///
    fn start_network(&self, net_receiver: NetworkEventReceiver) -> Result<()> {
        let access_token = self
            .access_token
            .clone()
            .ok_or(anyhow!("No access token"))?;
        self.start_network_with_token(net_receiver, access_token)
    }

    fn start_network_with_token(
        &self,
        net_receiver: NetworkEventReceiver,
        initial_token: String,
    ) -> Result<()> {
        debug!("Creating new thread for asynchronous networking...");
        let cloned_state = Arc::clone(&self.state);
        let access_token = initial_token.to_owned();
        std::thread::spawn(move || {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async {
                    let mut asana = Asana::new(&access_token);
                    let mut network_event_handler =
                        NetworkEventHandler::new(&cloned_state, &mut asana);
                    while let Ok(network_event) = net_receiver.recv() {
                        let event_clone = network_event.clone();
                        match network_event_handler.handle(network_event).await {
                            Ok(_) => (),
                            Err(e) => {
                                error!("Failed to handle network event {:?}: {}", event_clone, e);
                                // Log the full error chain for debugging
                                let mut source = e.source();
                                while let Some(err) = source {
                                    error!("  Caused by: {}", err);
                                    source = err.source();
                                }
                            }
                        }
                    }
                })
        });
        Ok(())
    }

    /// Begin the terminal event poll on a separate thread before starting the
    /// render loop on the main thread. Return the result following an exit
    /// request or unrecoverable error.
    ///
    async fn start_ui(&mut self, net_sender: NetworkEventSender) -> Result<()> {
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
            net_sender.send(NetworkEvent::Me)?;
        }

        let terminal_event_handler = TerminalEventHandler::new();
        loop {
            let mut state = self.state.lock().await;
            if let Ok(size) = terminal.backend().size() {
                state.set_terminal_size(size);
            };
            terminal.draw(|frame| crate::ui::render(frame, &mut state))?;
            if !terminal_event_handler.handle_next(&mut state)? {
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
