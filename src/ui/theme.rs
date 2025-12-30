use ratatui::style::Color;
use serde::{Deserialize, Serialize};

/// Theme color palette defining all colors used in the application.
///
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    // Primary colors
    pub primary: ColorSpec,
    pub secondary: ColorSpec,
    pub accent: ColorSpec,
    pub banner: ColorSpec,

    // Text colors
    pub text: ColorSpec,
    pub text_secondary: ColorSpec,
    pub text_muted: ColorSpec,

    // Background colors
    pub background: ColorSpec,
    pub surface: ColorSpec,

    // Status colors
    pub success: ColorSpec,
    pub warning: ColorSpec,
    pub error: ColorSpec,
    pub info: ColorSpec,

    // UI element colors
    pub border_active: ColorSpec,
    pub border_normal: ColorSpec,
    pub highlight_bg: ColorSpec,
    pub highlight_fg: ColorSpec,

    // Footer mode colors
    pub footer_search: ColorSpec,
    pub footer_debug: ColorSpec,
    pub footer_delete: ColorSpec,
    pub footer_move: ColorSpec,
    pub footer_edit: ColorSpec,
    pub footer_tasks: ColorSpec,
    pub footer_task: ColorSpec,
    pub footer_normal: ColorSpec,
}

/// Color specification that can be serialized/deserialized.
///
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ColorSpec {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl ColorSpec {
    pub fn to_color(&self) -> Color {
        Color::Rgb(self.r, self.g, self.b)
    }
}

impl Theme {
    /// Get the default theme (Rose Pine Dawn).
    ///
    pub fn default() -> Self {
        Self::rose_pine_dawn()
    }

    /// Rose Pine Dawn theme.
    ///
    pub fn rose_pine_dawn() -> Self {
        Theme {
            name: "rose-pine-dawn".to_string(),
            primary: ColorSpec {
                r: 161,
                g: 119,
                b: 255,
            }, // Purple
            secondary: ColorSpec {
                r: 59,
                g: 247,
                b: 209,
            }, // Green
            accent: ColorSpec {
                r: 255,
                g: 109,
                b: 146,
            }, // Pink
            banner: ColorSpec {
                r: 255,
                g: 109,
                b: 146,
            }, // Pink
            text: ColorSpec {
                r: 88,
                g: 82,
                b: 96,
            }, // Text
            text_secondary: ColorSpec {
                r: 121,
                g: 117,
                b: 147,
            }, // Subtext
            text_muted: ColorSpec {
                r: 152,
                g: 147,
                b: 165,
            }, // Muted
            background: ColorSpec {
                r: 250,
                g: 244,
                b: 237,
            }, // Base
            surface: ColorSpec {
                r: 255,
                g: 250,
                b: 243,
            }, // Surface
            success: ColorSpec {
                r: 59,
                g: 247,
                b: 209,
            }, // Pine
            warning: ColorSpec {
                r: 255,
                g: 210,
                b: 0,
            }, // Gold
            error: ColorSpec {
                r: 235,
                g: 111,
                b: 146,
            }, // Love
            info: ColorSpec {
                r: 61,
                g: 174,
                b: 233,
            }, // Foam
            border_active: ColorSpec {
                r: 161,
                g: 119,
                b: 255,
            }, // Purple
            border_normal: ColorSpec {
                r: 88,
                g: 82,
                b: 96,
            }, // Text
            highlight_bg: ColorSpec {
                r: 61,
                g: 174,
                b: 233,
            }, // Foam
            highlight_fg: ColorSpec { r: 0, g: 0, b: 0 }, // Black
            footer_search: ColorSpec {
                r: 61,
                g: 174,
                b: 233,
            }, // Foam
            footer_debug: ColorSpec {
                r: 59,
                g: 247,
                b: 209,
            }, // Pine
            footer_delete: ColorSpec {
                r: 235,
                g: 111,
                b: 146,
            }, // Love
            footer_move: ColorSpec {
                r: 161,
                g: 119,
                b: 255,
            }, // Purple
            footer_edit: ColorSpec {
                r: 255,
                g: 210,
                b: 0,
            }, // Gold
            footer_tasks: ColorSpec {
                r: 61,
                g: 174,
                b: 233,
            }, // Foam
            footer_task: ColorSpec {
                r: 161,
                g: 119,
                b: 255,
            }, // Purple
            footer_normal: ColorSpec { r: 0, g: 0, b: 0 }, // Black
        }
    }

    /// Rose Pine theme.
    ///
    pub fn rose_pine() -> Self {
        Theme {
            name: "rose-pine".to_string(),
            primary: ColorSpec {
                r: 196,
                g: 167,
                b: 231,
            }, // Purple
            secondary: ColorSpec {
                r: 49,
                g: 116,
                b: 143,
            }, // Pine
            accent: ColorSpec {
                r: 235,
                g: 111,
                b: 146,
            }, // Love
            banner: ColorSpec {
                r: 235,
                g: 111,
                b: 146,
            }, // Love
            text: ColorSpec {
                r: 224,
                g: 222,
                b: 244,
            }, // Text
            text_secondary: ColorSpec {
                r: 144,
                g: 140,
                b: 170,
            }, // Subtext
            text_muted: ColorSpec {
                r: 86,
                g: 82,
                b: 100,
            }, // Muted
            background: ColorSpec {
                r: 25,
                g: 23,
                b: 36,
            }, // Base
            surface: ColorSpec {
                r: 31,
                g: 29,
                b: 43,
            }, // Surface
            success: ColorSpec {
                r: 49,
                g: 116,
                b: 143,
            }, // Pine
            warning: ColorSpec {
                r: 246,
                g: 193,
                b: 119,
            }, // Gold
            error: ColorSpec {
                r: 235,
                g: 111,
                b: 146,
            }, // Love
            info: ColorSpec {
                r: 156,
                g: 207,
                b: 216,
            }, // Foam
            border_active: ColorSpec {
                r: 196,
                g: 167,
                b: 231,
            }, // Purple
            border_normal: ColorSpec {
                r: 144,
                g: 140,
                b: 170,
            }, // Subtext
            highlight_bg: ColorSpec {
                r: 156,
                g: 207,
                b: 216,
            }, // Foam
            highlight_fg: ColorSpec {
                r: 25,
                g: 23,
                b: 36,
            }, // Base
            footer_search: ColorSpec {
                r: 156,
                g: 207,
                b: 216,
            }, // Foam
            footer_debug: ColorSpec {
                r: 49,
                g: 116,
                b: 143,
            }, // Pine
            footer_delete: ColorSpec {
                r: 235,
                g: 111,
                b: 146,
            }, // Love
            footer_move: ColorSpec {
                r: 196,
                g: 167,
                b: 231,
            }, // Purple
            footer_edit: ColorSpec {
                r: 246,
                g: 193,
                b: 119,
            }, // Gold
            footer_tasks: ColorSpec {
                r: 156,
                g: 207,
                b: 216,
            }, // Foam
            footer_task: ColorSpec {
                r: 196,
                g: 167,
                b: 231,
            }, // Purple
            footer_normal: ColorSpec { r: 0, g: 0, b: 0 }, // Black
        }
    }

    /// Rose Pine Moon theme.
    ///
    pub fn rose_pine_moon() -> Self {
        Theme {
            name: "rose-pine-moon".to_string(),
            primary: ColorSpec {
                r: 196,
                g: 167,
                b: 231,
            }, // Purple
            secondary: ColorSpec {
                r: 49,
                g: 116,
                b: 143,
            }, // Pine
            accent: ColorSpec {
                r: 235,
                g: 111,
                b: 146,
            }, // Love
            banner: ColorSpec {
                r: 235,
                g: 111,
                b: 146,
            }, // Love
            text: ColorSpec {
                r: 224,
                g: 222,
                b: 244,
            }, // Text
            text_secondary: ColorSpec {
                r: 144,
                g: 140,
                b: 170,
            }, // Subtext
            text_muted: ColorSpec {
                r: 86,
                g: 82,
                b: 100,
            }, // Muted
            background: ColorSpec {
                r: 35,
                g: 33,
                b: 54,
            }, // Base
            surface: ColorSpec {
                r: 42,
                g: 39,
                b: 63,
            }, // Surface
            success: ColorSpec {
                r: 49,
                g: 116,
                b: 143,
            }, // Pine
            warning: ColorSpec {
                r: 246,
                g: 193,
                b: 119,
            }, // Gold
            error: ColorSpec {
                r: 235,
                g: 111,
                b: 146,
            }, // Love
            info: ColorSpec {
                r: 156,
                g: 207,
                b: 216,
            }, // Foam
            border_active: ColorSpec {
                r: 196,
                g: 167,
                b: 231,
            }, // Purple
            border_normal: ColorSpec {
                r: 144,
                g: 140,
                b: 170,
            }, // Subtext
            highlight_bg: ColorSpec {
                r: 156,
                g: 207,
                b: 216,
            }, // Foam
            highlight_fg: ColorSpec {
                r: 35,
                g: 33,
                b: 54,
            }, // Base
            footer_search: ColorSpec {
                r: 156,
                g: 207,
                b: 216,
            }, // Foam
            footer_debug: ColorSpec {
                r: 49,
                g: 116,
                b: 143,
            }, // Pine
            footer_delete: ColorSpec {
                r: 235,
                g: 111,
                b: 146,
            }, // Love
            footer_move: ColorSpec {
                r: 196,
                g: 167,
                b: 231,
            }, // Purple
            footer_edit: ColorSpec {
                r: 246,
                g: 193,
                b: 119,
            }, // Gold
            footer_tasks: ColorSpec {
                r: 156,
                g: 207,
                b: 216,
            }, // Foam
            footer_task: ColorSpec {
                r: 196,
                g: 167,
                b: 231,
            }, // Purple
            footer_normal: ColorSpec { r: 0, g: 0, b: 0 }, // Black
        }
    }

    /// Dracula theme.
    ///
    pub fn dracula() -> Self {
        Theme {
            name: "dracula".to_string(),
            primary: ColorSpec {
                r: 189,
                g: 147,
                b: 249,
            }, // Purple
            secondary: ColorSpec {
                r: 139,
                g: 233,
                b: 253,
            }, // Cyan
            accent: ColorSpec {
                r: 255,
                g: 121,
                b: 198,
            }, // Pink
            banner: ColorSpec {
                r: 255,
                g: 121,
                b: 198,
            }, // Pink
            text: ColorSpec {
                r: 248,
                g: 248,
                b: 242,
            }, // Foreground
            text_secondary: ColorSpec {
                r: 189,
                g: 147,
                b: 249,
            }, // Purple
            text_muted: ColorSpec {
                r: 98,
                g: 114,
                b: 164,
            }, // Comment
            background: ColorSpec {
                r: 40,
                g: 42,
                b: 54,
            }, // Background
            surface: ColorSpec {
                r: 68,
                g: 71,
                b: 90,
            }, // Selection
            success: ColorSpec {
                r: 80,
                g: 250,
                b: 123,
            }, // Green
            warning: ColorSpec {
                r: 255,
                g: 184,
                b: 108,
            }, // Orange
            error: ColorSpec {
                r: 255,
                g: 85,
                b: 85,
            }, // Red
            info: ColorSpec {
                r: 139,
                g: 233,
                b: 253,
            }, // Cyan
            border_active: ColorSpec {
                r: 189,
                g: 147,
                b: 249,
            }, // Purple
            border_normal: ColorSpec {
                r: 98,
                g: 114,
                b: 164,
            }, // Comment
            highlight_bg: ColorSpec {
                r: 139,
                g: 233,
                b: 253,
            }, // Cyan
            highlight_fg: ColorSpec {
                r: 40,
                g: 42,
                b: 54,
            }, // Background
            footer_search: ColorSpec {
                r: 139,
                g: 233,
                b: 253,
            }, // Cyan
            footer_debug: ColorSpec {
                r: 80,
                g: 250,
                b: 123,
            }, // Green
            footer_delete: ColorSpec {
                r: 255,
                g: 85,
                b: 85,
            }, // Red
            footer_move: ColorSpec {
                r: 189,
                g: 147,
                b: 249,
            }, // Purple
            footer_edit: ColorSpec {
                r: 255,
                g: 184,
                b: 108,
            }, // Orange
            footer_tasks: ColorSpec {
                r: 139,
                g: 233,
                b: 253,
            }, // Cyan
            footer_task: ColorSpec {
                r: 189,
                g: 147,
                b: 249,
            }, // Purple
            footer_normal: ColorSpec { r: 0, g: 0, b: 0 }, // Black
        }
    }

    /// Catppuccin Latte theme.
    ///
    pub fn catppuccin_latte() -> Self {
        Theme {
            name: "catppuccin-latte".to_string(),
            primary: ColorSpec {
                r: 136,
                g: 57,
                b: 239,
            }, // Mauve
            secondary: ColorSpec {
                r: 40,
                g: 205,
                b: 130,
            }, // Green
            accent: ColorSpec {
                r: 234,
                g: 118,
                b: 203,
            }, // Pink
            banner: ColorSpec {
                r: 234,
                g: 118,
                b: 203,
            }, // Pink
            text: ColorSpec {
                r: 76,
                g: 79,
                b: 105,
            }, // Text
            text_secondary: ColorSpec {
                r: 92,
                g: 95,
                b: 119,
            }, // Subtext1
            text_muted: ColorSpec {
                r: 108,
                g: 111,
                b: 133,
            }, // Subtext0
            background: ColorSpec {
                r: 239,
                g: 241,
                b: 245,
            }, // Base
            surface: ColorSpec {
                r: 230,
                g: 233,
                b: 239,
            }, // Mantle
            success: ColorSpec {
                r: 40,
                g: 205,
                b: 130,
            }, // Green
            warning: ColorSpec {
                r: 223,
                g: 142,
                b: 29,
            }, // Yellow
            error: ColorSpec {
                r: 210,
                g: 15,
                b: 57,
            }, // Red
            info: ColorSpec {
                r: 32,
                g: 159,
                b: 181,
            }, // Blue
            border_active: ColorSpec {
                r: 136,
                g: 57,
                b: 239,
            }, // Mauve
            border_normal: ColorSpec {
                r: 108,
                g: 111,
                b: 133,
            }, // Subtext0
            highlight_bg: ColorSpec {
                r: 32,
                g: 159,
                b: 181,
            }, // Blue
            highlight_fg: ColorSpec {
                r: 239,
                g: 241,
                b: 245,
            }, // Base
            footer_search: ColorSpec {
                r: 32,
                g: 159,
                b: 181,
            }, // Blue
            footer_debug: ColorSpec {
                r: 40,
                g: 205,
                b: 130,
            }, // Green
            footer_delete: ColorSpec {
                r: 210,
                g: 15,
                b: 57,
            }, // Red
            footer_move: ColorSpec {
                r: 136,
                g: 57,
                b: 239,
            }, // Mauve
            footer_edit: ColorSpec {
                r: 223,
                g: 142,
                b: 29,
            }, // Yellow
            footer_tasks: ColorSpec {
                r: 32,
                g: 159,
                b: 181,
            }, // Blue
            footer_task: ColorSpec {
                r: 136,
                g: 57,
                b: 239,
            }, // Mauve
            footer_normal: ColorSpec { r: 0, g: 0, b: 0 }, // Black
        }
    }

    /// Catppuccin Frappé theme.
    ///
    pub fn catppuccin_frappe() -> Self {
        Theme {
            name: "catppuccin-frappe".to_string(),
            primary: ColorSpec {
                r: 198,
                g: 160,
                b: 246,
            }, // Mauve
            secondary: ColorSpec {
                r: 166,
                g: 218,
                b: 149,
            }, // Green
            accent: ColorSpec {
                r: 242,
                g: 205,
                b: 205,
            }, // Pink
            banner: ColorSpec {
                r: 242,
                g: 205,
                b: 205,
            }, // Pink
            text: ColorSpec {
                r: 198,
                g: 208,
                b: 245,
            }, // Text
            text_secondary: ColorSpec {
                r: 181,
                g: 191,
                b: 226,
            }, // Subtext1
            text_muted: ColorSpec {
                r: 165,
                g: 173,
                b: 206,
            }, // Subtext0
            background: ColorSpec {
                r: 48,
                g: 52,
                b: 70,
            }, // Base
            surface: ColorSpec {
                r: 41,
                g: 44,
                b: 60,
            }, // Mantle
            success: ColorSpec {
                r: 166,
                g: 218,
                b: 149,
            }, // Green
            warning: ColorSpec {
                r: 238,
                g: 190,
                b: 138,
            }, // Yellow
            error: ColorSpec {
                r: 231,
                g: 130,
                b: 132,
            }, // Red
            info: ColorSpec {
                r: 140,
                g: 170,
                b: 238,
            }, // Blue
            border_active: ColorSpec {
                r: 198,
                g: 160,
                b: 246,
            }, // Mauve
            border_normal: ColorSpec {
                r: 165,
                g: 173,
                b: 206,
            }, // Subtext0
            highlight_bg: ColorSpec {
                r: 140,
                g: 170,
                b: 238,
            }, // Blue
            highlight_fg: ColorSpec {
                r: 48,
                g: 52,
                b: 70,
            }, // Base
            footer_search: ColorSpec {
                r: 140,
                g: 170,
                b: 238,
            }, // Blue
            footer_debug: ColorSpec {
                r: 166,
                g: 218,
                b: 149,
            }, // Green
            footer_delete: ColorSpec {
                r: 231,
                g: 130,
                b: 132,
            }, // Red
            footer_move: ColorSpec {
                r: 198,
                g: 160,
                b: 246,
            }, // Mauve
            footer_edit: ColorSpec {
                r: 238,
                g: 190,
                b: 138,
            }, // Yellow
            footer_tasks: ColorSpec {
                r: 140,
                g: 170,
                b: 238,
            }, // Blue
            footer_task: ColorSpec {
                r: 198,
                g: 160,
                b: 246,
            }, // Mauve
            footer_normal: ColorSpec { r: 0, g: 0, b: 0 }, // Black
        }
    }

    /// Catppuccin Macchiato theme.
    ///
    pub fn catppuccin_macchiato() -> Self {
        Theme {
            name: "catppuccin-macchiato".to_string(),
            primary: ColorSpec {
                r: 198,
                g: 160,
                b: 246,
            }, // Mauve
            secondary: ColorSpec {
                r: 166,
                g: 218,
                b: 149,
            }, // Green
            accent: ColorSpec {
                r: 245,
                g: 189,
                b: 230,
            }, // Pink
            banner: ColorSpec {
                r: 245,
                g: 189,
                b: 230,
            }, // Pink
            text: ColorSpec {
                r: 202,
                g: 211,
                b: 245,
            }, // Text
            text_secondary: ColorSpec {
                r: 184,
                g: 187,
                b: 241,
            }, // Subtext1
            text_muted: ColorSpec {
                r: 165,
                g: 173,
                b: 206,
            }, // Subtext0
            background: ColorSpec {
                r: 30,
                g: 32,
                b: 48,
            }, // Base
            surface: ColorSpec {
                r: 24,
                g: 25,
                b: 38,
            }, // Mantle
            success: ColorSpec {
                r: 166,
                g: 218,
                b: 149,
            }, // Green
            warning: ColorSpec {
                r: 238,
                g: 190,
                b: 138,
            }, // Yellow
            error: ColorSpec {
                r: 237,
                g: 135,
                b: 150,
            }, // Red
            info: ColorSpec {
                r: 138,
                g: 173,
                b: 244,
            }, // Blue
            border_active: ColorSpec {
                r: 198,
                g: 160,
                b: 246,
            }, // Mauve
            border_normal: ColorSpec {
                r: 165,
                g: 173,
                b: 206,
            }, // Subtext0
            highlight_bg: ColorSpec {
                r: 138,
                g: 173,
                b: 244,
            }, // Blue
            highlight_fg: ColorSpec {
                r: 30,
                g: 32,
                b: 48,
            }, // Base
            footer_search: ColorSpec {
                r: 138,
                g: 173,
                b: 244,
            }, // Blue
            footer_debug: ColorSpec {
                r: 166,
                g: 218,
                b: 149,
            }, // Green
            footer_delete: ColorSpec {
                r: 237,
                g: 135,
                b: 150,
            }, // Red
            footer_move: ColorSpec {
                r: 198,
                g: 160,
                b: 246,
            }, // Mauve
            footer_edit: ColorSpec {
                r: 238,
                g: 190,
                b: 138,
            }, // Yellow
            footer_tasks: ColorSpec {
                r: 138,
                g: 173,
                b: 244,
            }, // Blue
            footer_task: ColorSpec {
                r: 198,
                g: 160,
                b: 246,
            }, // Mauve
            footer_normal: ColorSpec { r: 0, g: 0, b: 0 }, // Black
        }
    }

    /// Catppuccin Mocha theme.
    ///
    pub fn catppuccin_mocha() -> Self {
        Theme {
            name: "catppuccin-mocha".to_string(),
            primary: ColorSpec {
                r: 203,
                g: 166,
                b: 247,
            }, // Mauve
            secondary: ColorSpec {
                r: 166,
                g: 227,
                b: 161,
            }, // Green
            accent: ColorSpec {
                r: 250,
                g: 179,
                b: 135,
            }, // Peach
            banner: ColorSpec {
                r: 245,
                g: 189,
                b: 230,
            }, // Pink
            text: ColorSpec {
                r: 205,
                g: 214,
                b: 244,
            }, // Text
            text_secondary: ColorSpec {
                r: 186,
                g: 194,
                b: 222,
            }, // Subtext1
            text_muted: ColorSpec {
                r: 166,
                g: 173,
                b: 200,
            }, // Subtext0
            background: ColorSpec {
                r: 17,
                g: 17,
                b: 27,
            }, // Base
            surface: ColorSpec {
                r: 24,
                g: 24,
                b: 37,
            }, // Mantle
            success: ColorSpec {
                r: 166,
                g: 227,
                b: 161,
            }, // Green
            warning: ColorSpec {
                r: 249,
                g: 226,
                b: 175,
            }, // Yellow
            error: ColorSpec {
                r: 243,
                g: 139,
                b: 168,
            }, // Red
            info: ColorSpec {
                r: 137,
                g: 180,
                b: 250,
            }, // Blue
            border_active: ColorSpec {
                r: 203,
                g: 166,
                b: 247,
            }, // Mauve
            border_normal: ColorSpec {
                r: 166,
                g: 173,
                b: 200,
            }, // Subtext0
            highlight_bg: ColorSpec {
                r: 137,
                g: 180,
                b: 250,
            }, // Blue
            highlight_fg: ColorSpec {
                r: 17,
                g: 17,
                b: 27,
            }, // Base
            footer_search: ColorSpec {
                r: 137,
                g: 180,
                b: 250,
            }, // Blue
            footer_debug: ColorSpec {
                r: 166,
                g: 227,
                b: 161,
            }, // Green
            footer_delete: ColorSpec {
                r: 243,
                g: 139,
                b: 168,
            }, // Red
            footer_move: ColorSpec {
                r: 203,
                g: 166,
                b: 247,
            }, // Mauve
            footer_edit: ColorSpec {
                r: 249,
                g: 226,
                b: 175,
            }, // Yellow
            footer_tasks: ColorSpec {
                r: 137,
                g: 180,
                b: 250,
            }, // Blue
            footer_task: ColorSpec {
                r: 203,
                g: 166,
                b: 247,
            }, // Mauve
            footer_normal: ColorSpec { r: 0, g: 0, b: 0 }, // Black
        }
    }

    /// Tokyo Night theme.
    ///
    pub fn tokyo_night() -> Self {
        Theme {
            name: "tokyo-night".to_string(),
            primary: ColorSpec {
                r: 125,
                g: 207,
                b: 255,
            }, // Blue
            secondary: ColorSpec {
                r: 158,
                g: 206,
                b: 106,
            }, // Green
            accent: ColorSpec {
                r: 255,
                g: 159,
                b: 196,
            }, // Magenta
            banner: ColorSpec {
                r: 255,
                g: 159,
                b: 196,
            }, // Magenta
            text: ColorSpec {
                r: 169,
                g: 177,
                b: 214,
            }, // Foreground
            text_secondary: ColorSpec {
                r: 192,
                g: 202,
                b: 245,
            }, // Foreground (brighter)
            text_muted: ColorSpec {
                r: 117,
                g: 121,
                b: 148,
            }, // Comment
            background: ColorSpec {
                r: 26,
                g: 27,
                b: 38,
            }, // Background
            surface: ColorSpec {
                r: 36,
                g: 40,
                b: 59,
            }, // Selection
            success: ColorSpec {
                r: 158,
                g: 206,
                b: 106,
            }, // Green
            warning: ColorSpec {
                r: 255,
                g: 202,
                b: 40,
            }, // Yellow
            error: ColorSpec {
                r: 247,
                g: 118,
                b: 142,
            }, // Red
            info: ColorSpec {
                r: 125,
                g: 207,
                b: 255,
            }, // Blue
            border_active: ColorSpec {
                r: 125,
                g: 207,
                b: 255,
            }, // Blue
            border_normal: ColorSpec {
                r: 117,
                g: 121,
                b: 148,
            }, // Comment
            highlight_bg: ColorSpec {
                r: 125,
                g: 207,
                b: 255,
            }, // Blue
            highlight_fg: ColorSpec {
                r: 26,
                g: 27,
                b: 38,
            }, // Background
            footer_search: ColorSpec {
                r: 125,
                g: 207,
                b: 255,
            }, // Blue
            footer_debug: ColorSpec {
                r: 158,
                g: 206,
                b: 106,
            }, // Green
            footer_delete: ColorSpec {
                r: 247,
                g: 118,
                b: 142,
            }, // Red
            footer_move: ColorSpec {
                r: 125,
                g: 207,
                b: 255,
            }, // Blue
            footer_edit: ColorSpec {
                r: 255,
                g: 202,
                b: 40,
            }, // Yellow
            footer_tasks: ColorSpec {
                r: 125,
                g: 207,
                b: 255,
            }, // Blue
            footer_task: ColorSpec {
                r: 125,
                g: 207,
                b: 255,
            }, // Blue
            footer_normal: ColorSpec { r: 0, g: 0, b: 0 }, // Black
        }
    }

    /// Tokyo Night Storm theme.
    ///
    pub fn tokyo_night_storm() -> Self {
        Theme {
            name: "tokyo-night-storm".to_string(),
            primary: ColorSpec {
                r: 125,
                g: 207,
                b: 255,
            }, // Blue
            secondary: ColorSpec {
                r: 158,
                g: 206,
                b: 106,
            }, // Green
            accent: ColorSpec {
                r: 255,
                g: 159,
                b: 196,
            }, // Magenta
            banner: ColorSpec {
                r: 255,
                g: 159,
                b: 196,
            }, // Magenta
            text: ColorSpec {
                r: 169,
                g: 177,
                b: 214,
            }, // Foreground
            text_secondary: ColorSpec {
                r: 192,
                g: 202,
                b: 245,
            }, // Foreground (brighter)
            text_muted: ColorSpec {
                r: 117,
                g: 121,
                b: 148,
            }, // Comment
            background: ColorSpec {
                r: 36,
                g: 40,
                b: 59,
            }, // Background
            surface: ColorSpec {
                r: 48,
                g: 52,
                b: 70,
            }, // Selection
            success: ColorSpec {
                r: 158,
                g: 206,
                b: 106,
            }, // Green
            warning: ColorSpec {
                r: 255,
                g: 202,
                b: 40,
            }, // Yellow
            error: ColorSpec {
                r: 247,
                g: 118,
                b: 142,
            }, // Red
            info: ColorSpec {
                r: 125,
                g: 207,
                b: 255,
            }, // Blue
            border_active: ColorSpec {
                r: 125,
                g: 207,
                b: 255,
            }, // Blue
            border_normal: ColorSpec {
                r: 117,
                g: 121,
                b: 148,
            }, // Comment
            highlight_bg: ColorSpec {
                r: 125,
                g: 207,
                b: 255,
            }, // Blue
            highlight_fg: ColorSpec {
                r: 36,
                g: 40,
                b: 59,
            }, // Background
            footer_search: ColorSpec {
                r: 125,
                g: 207,
                b: 255,
            }, // Blue
            footer_debug: ColorSpec {
                r: 158,
                g: 206,
                b: 106,
            }, // Green
            footer_delete: ColorSpec {
                r: 247,
                g: 118,
                b: 142,
            }, // Red
            footer_move: ColorSpec {
                r: 125,
                g: 207,
                b: 255,
            }, // Blue
            footer_edit: ColorSpec {
                r: 255,
                g: 202,
                b: 40,
            }, // Yellow
            footer_tasks: ColorSpec {
                r: 125,
                g: 207,
                b: 255,
            }, // Blue
            footer_task: ColorSpec {
                r: 125,
                g: 207,
                b: 255,
            }, // Blue
            footer_normal: ColorSpec { r: 0, g: 0, b: 0 }, // Black
        }
    }

    /// Tokyo Night Day theme.
    ///
    pub fn tokyo_night_day() -> Self {
        Theme {
            name: "tokyo-night-day".to_string(),
            primary: ColorSpec {
                r: 38,
                g: 139,
                b: 210,
            }, // Blue
            secondary: ColorSpec {
                r: 34,
                g: 154,
                b: 83,
            }, // Green
            accent: ColorSpec {
                r: 220,
                g: 50,
                b: 47,
            }, // Red
            banner: ColorSpec {
                r: 220,
                g: 50,
                b: 47,
            }, // Red
            text: ColorSpec {
                r: 26,
                g: 27,
                b: 38,
            }, // Foreground
            text_secondary: ColorSpec {
                r: 36,
                g: 40,
                b: 59,
            }, // Foreground (darker)
            text_muted: ColorSpec {
                r: 117,
                g: 121,
                b: 148,
            }, // Comment
            background: ColorSpec {
                r: 234,
                g: 238,
                b: 255,
            }, // Background
            surface: ColorSpec {
                r: 203,
                g: 211,
                b: 255,
            }, // Selection
            success: ColorSpec {
                r: 34,
                g: 154,
                b: 83,
            }, // Green
            warning: ColorSpec {
                r: 196,
                g: 157,
                b: 0,
            }, // Yellow
            error: ColorSpec {
                r: 220,
                g: 50,
                b: 47,
            }, // Red
            info: ColorSpec {
                r: 38,
                g: 139,
                b: 210,
            }, // Blue
            border_active: ColorSpec {
                r: 38,
                g: 139,
                b: 210,
            }, // Blue
            border_normal: ColorSpec {
                r: 117,
                g: 121,
                b: 148,
            }, // Comment
            highlight_bg: ColorSpec {
                r: 38,
                g: 139,
                b: 210,
            }, // Blue
            highlight_fg: ColorSpec {
                r: 234,
                g: 238,
                b: 255,
            }, // Background
            footer_search: ColorSpec {
                r: 38,
                g: 139,
                b: 210,
            }, // Blue
            footer_debug: ColorSpec {
                r: 34,
                g: 154,
                b: 83,
            }, // Green
            footer_delete: ColorSpec {
                r: 220,
                g: 50,
                b: 47,
            }, // Red
            footer_move: ColorSpec {
                r: 38,
                g: 139,
                b: 210,
            }, // Blue
            footer_edit: ColorSpec {
                r: 196,
                g: 157,
                b: 0,
            }, // Yellow
            footer_tasks: ColorSpec {
                r: 38,
                g: 139,
                b: 210,
            }, // Blue
            footer_task: ColorSpec {
                r: 38,
                g: 139,
                b: 210,
            }, // Blue
            footer_normal: ColorSpec { r: 0, g: 0, b: 0 }, // Black
        }
    }

    /// Get a theme by name.
    ///
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "rose-pine-dawn" => Some(Self::rose_pine_dawn()),
            "rose-pine" => Some(Self::rose_pine()),
            "rose-pine-moon" => Some(Self::rose_pine_moon()),
            "dracula" => Some(Self::dracula()),
            "catppuccin-latte" => Some(Self::catppuccin_latte()),
            "catppuccin-frappe" => Some(Self::catppuccin_frappe()),
            "catppuccin-macchiato" => Some(Self::catppuccin_macchiato()),
            "catppuccin-mocha" => Some(Self::catppuccin_mocha()),
            "tokyo-night" => Some(Self::tokyo_night()),
            "tokyo-night-storm" => Some(Self::tokyo_night_storm()),
            "tokyo-night-day" => Some(Self::tokyo_night_day()),
            _ => None,
        }
    }

    /// Get list of all available theme names.
    ///
    pub fn available_themes() -> Vec<String> {
        vec![
            "rose-pine-dawn".to_string(),
            "rose-pine".to_string(),
            "rose-pine-moon".to_string(),
            "dracula".to_string(),
            "catppuccin-latte".to_string(),
            "catppuccin-frappe".to_string(),
            "catppuccin-macchiato".to_string(),
            "catppuccin-mocha".to_string(),
            "tokyo-night".to_string(),
            "tokyo-night-storm".to_string(),
            "tokyo-night-day".to_string(),
        ]
    }
}
