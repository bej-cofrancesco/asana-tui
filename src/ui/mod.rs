type Frame<'a> = ratatui::Frame<'a>;

mod render;
mod theme;
mod widgets;

pub const SPINNER_FRAME_COUNT: usize = widgets::spinner::FRAMES.len();

pub use render::render;
pub use theme::Theme;
