use super::Frame;
use crate::state::State;
use crate::ui::widgets::styling;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Style,
    text::Text,
    widgets::{Block, Borders, Paragraph},
};

pub const BANNER: &str = "
.:                                           
.: ::                                         
.:  .::     .::::    .::    .:: .::     .::    
.::   .::   .::     .::  .::  .::  .:: .::  .:: 
.:::::: .::    .::: .::   .::  .::  .::.::   .:: 
.::       .::     .::.::   .::  .::  .::.::   .:: 
.::         .::.:: .::  .:: .:::.:::  .::  .:: .:::
                                                                                                     
";

pub const CONTENT: &str = "
  why use asana when you can use the tui?

  hard core productivity, no bullshit. no fucks given. just get shit done.

  t to select a theme.
";

pub const ONBOARDING_INSTRUCTIONS: &str = r#"
Welcome to Asana TUI!

To get started, you need a Personal Access Token from Asana.

Instructions:
1. Visit https://app.asana.com/0/my-apps
2. Click "Create new token"
3. Give it a name (e.g., "TUI Access")
4. Copy the token and paste it below

Enter your Personal Access Token:
"#;

pub fn render_welcome(frame: &mut Frame, size: Rect, state: &State) {
    // Check if we have an access token (user is logged in)
    if state.has_access_token() {
        // Show logged-in welcome screen (original design)
        render_logged_in_welcome(frame, size, state);
    } else {
        // Show onboarding screen (no token)
        render_onboarding(frame, size, state);
    }
}

fn render_logged_in_welcome(frame: &mut Frame, size: Rect, _state: &State) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(6), Constraint::Length(94)].as_ref())
        .margin(2)
        .split(size);

    let theme = _state.get_theme();
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Welcome")
        .border_style(styling::active_block_border_style(theme));
    frame.render_widget(block, size);

    let banner = Text::from(BANNER);
    let banner_widget = Paragraph::new(banner.patch_style(styling::banner_style(theme)));
    frame.render_widget(banner_widget, rows[0]);

    let content = Text::from(CONTENT);
    let content_widget = Paragraph::new(content.patch_style(styling::normal_text_style(theme)));
    frame.render_widget(content_widget, rows[1]);
}

fn render_onboarding(frame: &mut Frame, size: Rect, state: &State) {
    // Full screen onboarding - no dialog, just full frame
    let theme = state.get_theme();
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Welcome to Asana TUI - Setup")
        .border_style(Style::default().fg(theme.info.to_color()));

    frame.render_widget(block, size);

    // Split screen into sections
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),  // Banner
            Constraint::Length(10), // Instructions
            Constraint::Length(3),  // Input field
            Constraint::Min(0),     // Fill remaining space
        ])
        .margin(1)
        .split(size);

    // Banner
    let banner = Text::from(BANNER);
    let banner_widget =
        Paragraph::new(banner.patch_style(Style::default().fg(theme.info.to_color())))
            .alignment(Alignment::Center);
    frame.render_widget(banner_widget, chunks[0]);

    // Instructions
    let mut instructions_text = ONBOARDING_INSTRUCTIONS.to_string();

    // Add error message if present
    if let Some(error) = state.get_auth_error() {
        instructions_text.push_str("\n\n");
        instructions_text.push_str(error);
        instructions_text.push_str("\n\nPlease check your token and try again.");
    }

    let instructions = Paragraph::new(instructions_text)
        .style(if state.get_auth_error().is_some() {
            Style::default().fg(theme.error.to_color())
        } else {
            Style::default().fg(theme.text.to_color())
        })
        .wrap(ratatui::widgets::Wrap { trim: true });
    frame.render_widget(instructions, chunks[1]);

    // Input field
    let token_input = state.get_access_token_input();
    let input_text = if token_input.is_empty() {
        "Enter token here...".to_string()
    } else {
        format!("{}", "*".repeat(token_input.len().min(50)))
    };

    let input_block = Block::default()
        .borders(Borders::ALL)
        .title(if state.get_auth_error().is_some() {
            "Access Token (Enter to resubmit, Backspace to edit)"
        } else {
            "Access Token (Enter to submit)"
        })
        .border_style(if state.get_auth_error().is_some() {
            Style::default().fg(theme.error.to_color())
        } else {
            Style::default().fg(theme.warning.to_color())
        });

    let input_para =
        Paragraph::new(input_text)
            .block(input_block)
            .style(if token_input.is_empty() {
                Style::default().fg(theme.text_muted.to_color())
            } else {
                Style::default().fg(theme.text.to_color())
            });

    frame.render_widget(input_para, chunks[2]);
}
