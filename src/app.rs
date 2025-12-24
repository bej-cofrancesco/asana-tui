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
    access_token: String,
    state: Arc<Mutex<State>>,
    config: Config,
}

impl App {
    /// Start a new application according to the given configuration. Returns
    /// the result of the application execution.
    ///
    pub async fn start(config: Config) -> Result<()> {
        init_logger(LevelFilter::Info).unwrap();
        set_default_level(LevelFilter::Trace);

        info!("Starting application...");
        let (tx, rx) = std::sync::mpsc::channel::<NetworkEvent>();
        let (config_save_tx, config_save_rx) = std::sync::mpsc::channel::<()>();
        let starred_projects = config.starred_projects.clone();
        let starred_project_names = config.starred_project_names.clone();
        let access_token = config.access_token.clone().ok_or(anyhow!("Failed to retrieve access token"))?;
        let mut app = App {
            access_token,
            state: Arc::new(Mutex::new(State::new(tx.clone(), config_save_tx.clone(), starred_projects, starred_project_names))),
            config,
        };
        app.start_network(rx)?;
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
        debug!("Creating new thread for asynchronous networking...");
        let cloned_state = Arc::clone(&self.state);
        let access_token = self.access_token.to_owned();
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
                        match network_event_handler.handle(network_event).await {
                            Ok(_) => (),
                            Err(e) => error!("Failed to handle network event: {}", e),
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
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        enable_raw_mode()?;

        let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;
        terminal.hide_cursor()?;

        net_sender.send(NetworkEvent::Me)?;

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
