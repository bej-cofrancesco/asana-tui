use crate::ui::theme::Theme;
use ratatui::style::{Modifier, Style};

/// Return the border style for active blocks.
///
pub fn active_block_border_style(theme: &Theme) -> Style {
    Style::default().fg(theme.border_active.to_color())
}

/// Return the border style for normal blocks.
///
pub fn normal_block_border_style(theme: &Theme) -> Style {
    Style::default().fg(theme.border_normal.to_color())
}

/// Return the title style for active blocks.
///
pub fn active_block_title_style() -> Style {
    Style::default().add_modifier(Modifier::BOLD)
}

/// Return the style for current list items.
///
pub fn current_list_item_style(theme: &Theme) -> Style {
    Style::default()
        .fg(theme.text.to_color())
        .add_modifier(Modifier::BOLD)
}

/// Return the style for active list items.
///
pub fn active_list_item_style(theme: &Theme) -> Style {
    current_list_item_style(theme).fg(theme.primary.to_color())
}

/// Return the style for normal text.
///
pub fn normal_text_style(theme: &Theme) -> Style {
    Style::default().fg(theme.text.to_color())
}

/// Return the style for the banner.
///
pub fn banner_style(theme: &Theme) -> Style {
    Style::default().fg(theme.banner.to_color())
}
