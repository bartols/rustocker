use ratatui::style::{Color, Style};
use std::sync::OnceLock;

#[derive(Debug, Clone)]
pub struct Theme {
    // Base colors
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,

    // Text colors
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_muted: Color,
    pub text_disabled: Color,

    // UI colors
    pub background: Color,
    pub surface: Color,
    pub border: Color,
    pub selected_bg: Color,
    pub selected_fg: Color,
    pub hover_bg: Color,

    // Status colors
    pub running: Color,
    pub stopped: Color,
    pub loading: Color,
}

impl Theme {
    /// Default Docker TUI theme (dark)
    pub fn default() -> Self {
        Self {
            // Base colors
            primary: Color::Cyan,
            secondary: Color::Blue,
            accent: Color::Yellow,
            success: Color::Green,
            warning: Color::Rgb(255, 165, 0), // Orange
            error: Color::Red,
            info: Color::LightBlue,

            // Text colors
            text_primary: Color::White,
            text_secondary: Color::LightBlue,
            text_muted: Color::DarkGray,
            text_disabled: Color::Gray,

            // UI colors
            background: Color::Black,
            surface: Color::Rgb(40, 40, 40),
            border: Color::Gray,
            selected_bg: Color::DarkGray,
            selected_fg: Color::LightYellow,
            hover_bg: Color::Rgb(60, 60, 60),

            // Status colors
            running: Color::Green,
            stopped: Color::Red,
            loading: Color::Yellow,
        }
    }

    /// Alternative blue theme
    pub fn blue() -> Self {
        Self {
            primary: Color::Blue,
            secondary: Color::LightBlue,
            accent: Color::Cyan,
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            info: Color::LightCyan,

            text_primary: Color::White,
            text_secondary: Color::LightBlue,
            text_muted: Color::Gray,
            text_disabled: Color::DarkGray,

            background: Color::Black,
            surface: Color::Rgb(20, 30, 50),
            border: Color::Blue,
            selected_bg: Color::Rgb(30, 50, 80),
            selected_fg: Color::LightCyan,
            hover_bg: Color::Rgb(40, 60, 90),

            running: Color::LightGreen,
            stopped: Color::LightRed,
            loading: Color::LightYellow,
        }
    }

    /// Light theme
    /*
    pub fn light() -> Self {
        Self {
            primary: Color::Blue,
            secondary: Color::DarkBlue,
            accent: Color::Rgb(255, 140, 0), // Dark Orange
            success: Color::DarkGreen,
            warning: Color::Rgb(255, 165, 0), // Orange
            error: Color::DarkRed,
            info: Color::Blue,

            text_primary: Color::Black,
            text_secondary: Color::DarkBlue,
            text_muted: Color::Gray,
            text_disabled: Color::LightGray,

            background: Color::White,
            surface: Color::Rgb(250, 250, 250),
            border: Color::Gray,
            selected_bg: Color::LightBlue,
            selected_fg: Color::Black,
            hover_bg: Color::Rgb(240, 240, 240),

            running: Color::DarkGreen,
            stopped: Color::DarkRed,
            loading: Color::Rgb(255, 140, 0),
        }
    }
    */

    /// Dracula inspired theme
    pub fn dracula() -> Self {
        Self {
            primary: Color::Rgb(139, 233, 253),  // Cyan
            secondary: Color::Rgb(98, 114, 164), // Purple
            accent: Color::Rgb(255, 184, 108),   // Orange
            success: Color::Rgb(80, 250, 123),   // Green
            warning: Color::Rgb(255, 255, 135),  // Yellow
            error: Color::Rgb(255, 85, 85),      // Red
            info: Color::Rgb(189, 147, 249),     // Purple

            text_primary: Color::Rgb(248, 248, 242), // Foreground
            text_secondary: Color::Rgb(139, 233, 253), // Cyan
            text_muted: Color::Rgb(98, 114, 164),    // Comment
            text_disabled: Color::Rgb(68, 71, 90),   // Current line

            background: Color::Rgb(40, 42, 54),     // Background
            surface: Color::Rgb(68, 71, 90),        // Current line
            border: Color::Rgb(98, 114, 164),       // Comment
            selected_bg: Color::Rgb(68, 71, 90),    // Current line
            selected_fg: Color::Rgb(255, 184, 108), // Orange
            hover_bg: Color::Rgb(98, 114, 164),     // Comment

            running: Color::Rgb(80, 250, 123),  // Green
            stopped: Color::Rgb(255, 85, 85),   // Red
            loading: Color::Rgb(255, 255, 135), // Yellow
        }
    }

    /// Gruvbox theme
    pub fn gruvbox() -> Self {
        Self {
            primary: Color::Rgb(142, 192, 124),   // Bright green
            secondary: Color::Rgb(131, 165, 152), // Aqua
            accent: Color::Rgb(250, 189, 47),     // Yellow
            success: Color::Rgb(142, 192, 124),   // Green
            warning: Color::Rgb(250, 189, 47),    // Yellow
            error: Color::Rgb(251, 73, 52),       // Red
            info: Color::Rgb(131, 165, 152),      // Aqua

            text_primary: Color::Rgb(235, 219, 178),   // fg1
            text_secondary: Color::Rgb(189, 174, 147), // fg2
            text_muted: Color::Rgb(146, 131, 116),     // fg4
            text_disabled: Color::Rgb(102, 92, 84),    // gray

            background: Color::Rgb(40, 40, 40),    // bg0
            surface: Color::Rgb(60, 56, 54),       // bg1
            border: Color::Rgb(102, 92, 84),       // gray
            selected_bg: Color::Rgb(80, 73, 69),   // bg2
            selected_fg: Color::Rgb(250, 189, 47), // Yellow
            hover_bg: Color::Rgb(102, 92, 84),     // gray

            running: Color::Rgb(142, 192, 124), // Green
            stopped: Color::Rgb(251, 73, 52),   // Red
            loading: Color::Rgb(250, 189, 47),  // Yellow
        }
    }
}

impl Theme {
    // Convenience methods for commonly used styles

    pub fn header_style(&self) -> Style {
        Style::default().fg(self.accent)
    }

    pub fn border_style(&self) -> Style {
        Style::default().fg(self.border)
    }

    pub fn selected_style(&self) -> Style {
        Style::default().fg(self.selected_fg).bg(self.selected_bg)
    }

    pub fn normal_style(&self) -> Style {
        Style::default().fg(self.text_primary)
    }

    pub fn muted_style(&self) -> Style {
        Style::default().fg(self.text_muted)
    }

    pub fn error_style(&self) -> Style {
        Style::default().fg(self.error)
    }

    pub fn success_style(&self) -> Style {
        Style::default().fg(self.success)
    }

    pub fn warning_style(&self) -> Style {
        Style::default().fg(self.warning)
    }

    pub fn info_style(&self) -> Style {
        Style::default().fg(self.info)
    }

    pub fn modal_border_style(&self) -> Style {
        Style::default().fg(self.primary)
    }

    pub fn highlight_style(&self) -> Style {
        Style::default().fg(self.secondary)
    }

    pub fn running_status_style(&self) -> Style {
        Style::default().fg(self.running)
    }

    pub fn stopped_status_style(&self) -> Style {
        Style::default().fg(self.stopped)
    }

    pub fn loading_style(&self) -> Style {
        Style::default().fg(self.loading)
    }
}

// Global theme instance (you could make this configurable later)
static CURRENT_THEME: OnceLock<Theme> = OnceLock::new();

pub fn init_theme(theme: Theme) {
    let _ = CURRENT_THEME.set(theme);
}

pub fn current_theme() -> &'static Theme {
    CURRENT_THEME.get_or_init(|| Theme::default())
}
