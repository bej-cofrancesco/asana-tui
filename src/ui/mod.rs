//! User interface module.
//!
//! This module handles all UI rendering using the `ratatui` library, including:
//! - Terminal rendering and layout
//! - Theme management
//! - Widget components (spinner, styling, etc.)
//! - View rendering (kanban, task detail, forms, etc.)

type Frame<'a> = ratatui::Frame<'a>;

mod render;
mod theme;
mod widgets;

pub const SPINNER_FRAME_COUNT: usize = widgets::spinner::FRAMES.len();

pub use render::render;
pub use theme::Theme;
