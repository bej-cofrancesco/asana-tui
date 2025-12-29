type Frame<'a> = ratatui::Frame<'a>;

mod render;
mod widgets;

pub const SPINNER_FRAME_COUNT: usize = widgets::spinner::FRAMES.len();

pub use render::render;
pub use widgets::color;
