// src/tui/theme.rs - Modern theme and styling for the TUI
// Provides beautiful color schemes and visual styling

use ratatui::style::{Color, Modifier, Style};

/// Modern color palette inspired by popular themes
#[allow(dead_code)]
pub struct Theme {
    // Primary colors
    pub primary: Color,
    pub primary_dark: Color,
    pub primary_light: Color,

    // Accent colors
    pub accent: Color,
    pub accent_dark: Color,

    // Background colors
    pub bg_primary: Color,
    pub bg_secondary: Color,
    pub bg_highlight: Color,

    // Text colors
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_muted: Color,

    // Status colors
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,

    // Priority colors (gradient from cool to warm)
    pub priority_lowest: Color,
    pub priority_low: Color,
    pub priority_medium: Color,
    pub priority_high: Color,
    pub priority_highest: Color,
}

impl Theme {
    /// Create a modern dark theme with vibrant colors
    pub fn modern_dark() -> Self {
        Theme {
            // Purple/Pink gradient for primary
            primary: Color::Rgb(147, 51, 234),        // Purple
            primary_dark: Color::Rgb(109, 40, 217),   // Dark purple
            primary_light: Color::Rgb(196, 181, 253), // Light purple

            // Cyan accent
            accent: Color::Rgb(34, 211, 238),      // Cyan
            accent_dark: Color::Rgb(14, 165, 233), // Dark cyan

            // Dark backgrounds with subtle differences
            bg_primary: Color::Rgb(17, 24, 39), // Very dark blue-gray
            bg_secondary: Color::Rgb(31, 41, 55), // Dark blue-gray
            bg_highlight: Color::Rgb(55, 65, 81), // Medium blue-gray

            // Text hierarchy
            text_primary: Color::Rgb(243, 244, 246), // Almost white
            text_secondary: Color::Rgb(209, 213, 219), // Light gray
            text_muted: Color::Rgb(107, 114, 128),   // Medium gray

            // Status colors
            success: Color::Rgb(34, 197, 94),  // Green
            warning: Color::Rgb(251, 191, 36), // Amber
            error: Color::Rgb(239, 68, 68),    // Red
            info: Color::Rgb(59, 130, 246),    // Blue

            // Priority gradient (cool to warm) - more vibrant colors
            priority_lowest: Color::Rgb(59, 130, 246), // Bright blue
            priority_low: Color::Rgb(34, 197, 94),     // Bright green
            priority_medium: Color::Rgb(250, 204, 21), // Bright yellow
            priority_high: Color::Rgb(251, 146, 60),   // Bright orange
            priority_highest: Color::Rgb(239, 68, 68), // Bright red
        }
    }

    /// Create a soft pastel theme
    #[allow(dead_code)]
    pub fn soft_pastel() -> Self {
        Theme {
            // Soft pink primary
            primary: Color::Rgb(236, 72, 153),        // Pink
            primary_dark: Color::Rgb(219, 39, 119),   // Dark pink
            primary_light: Color::Rgb(251, 207, 232), // Light pink

            // Soft blue accent
            accent: Color::Rgb(147, 197, 253),     // Light blue
            accent_dark: Color::Rgb(96, 165, 250), // Medium blue

            // Light backgrounds
            bg_primary: Color::Rgb(249, 250, 251), // Almost white
            bg_secondary: Color::Rgb(243, 244, 246), // Very light gray
            bg_highlight: Color::Rgb(229, 231, 235), // Light gray

            // Dark text for light theme
            text_primary: Color::Rgb(17, 24, 39),   // Very dark
            text_secondary: Color::Rgb(55, 65, 81), // Dark gray
            text_muted: Color::Rgb(107, 114, 128),  // Medium gray

            // Pastel status colors
            success: Color::Rgb(134, 239, 172), // Mint green
            warning: Color::Rgb(253, 224, 71),  // Light yellow
            error: Color::Rgb(252, 165, 165),   // Light red
            info: Color::Rgb(165, 180, 252),    // Lavender

            // Pastel priority gradient
            priority_lowest: Color::Rgb(191, 219, 254), // Baby blue
            priority_low: Color::Rgb(167, 243, 208),    // Mint
            priority_medium: Color::Rgb(253, 230, 138), // Cream
            priority_high: Color::Rgb(254, 215, 170),   // Peach
            priority_highest: Color::Rgb(254, 202, 202), // Pink
        }
    }

    /// Create a cyberpunk neon theme
    #[allow(dead_code)]
    pub fn cyberpunk() -> Self {
        Theme {
            // Neon pink primary
            primary: Color::Rgb(255, 0, 255),         // Magenta
            primary_dark: Color::Rgb(192, 38, 211),   // Purple
            primary_light: Color::Rgb(255, 182, 255), // Light magenta

            // Neon cyan accent
            accent: Color::Rgb(0, 255, 255),      // Cyan
            accent_dark: Color::Rgb(0, 184, 184), // Dark cyan

            // Dark cyberpunk backgrounds
            bg_primary: Color::Rgb(13, 2, 33),   // Very dark purple
            bg_secondary: Color::Rgb(25, 7, 51), // Dark purple
            bg_highlight: Color::Rgb(49, 10, 101), // Purple

            // Neon text
            text_primary: Color::Rgb(255, 255, 255), // White
            text_secondary: Color::Rgb(0, 255, 255), // Cyan
            text_muted: Color::Rgb(147, 51, 234),    // Purple

            // Neon status colors
            success: Color::Rgb(57, 255, 20), // Neon green
            warning: Color::Rgb(255, 255, 0), // Yellow
            error: Color::Rgb(255, 0, 0),     // Red
            info: Color::Rgb(0, 149, 255),    // Electric blue

            // Neon priority gradient
            priority_lowest: Color::Rgb(0, 255, 255), // Cyan
            priority_low: Color::Rgb(0, 255, 127),    // Spring green
            priority_medium: Color::Rgb(255, 255, 0), // Yellow
            priority_high: Color::Rgb(255, 127, 0),   // Orange
            priority_highest: Color::Rgb(255, 0, 127), // Hot pink
        }
    }

    /// Get priority color based on priority level
    pub fn priority_color(&self, priority: Option<u8>) -> Color {
        match priority {
            Some(1) => self.priority_lowest,
            Some(2) => self.priority_low,
            Some(3) => self.priority_medium,
            Some(4) => self.priority_high,
            Some(5) => self.priority_highest,
            _ => self.text_muted,
        }
    }

    /// Get style for completed todos
    pub fn completed_style(&self) -> Style {
        Style::default()
            .fg(self.text_muted)
            .add_modifier(Modifier::CROSSED_OUT | Modifier::DIM)
    }

    /// Get style for selected item
    pub fn selected_style(&self) -> Style {
        Style::default()
            .bg(self.bg_highlight)
            .fg(self.text_primary)
            .add_modifier(Modifier::BOLD)
    }

    /// Get style for borders based on focus
    pub fn border_style(&self, focused: bool) -> Style {
        if focused {
            Style::default()
                .fg(self.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(self.bg_highlight)
        }
    }

    /// Get style for title bars
    pub fn title_style(&self) -> Style {
        Style::default()
            .fg(self.primary_light)
            .add_modifier(Modifier::BOLD)
    }

    /// Get gradient bar for visual appeal
    #[allow(dead_code)]
    pub fn gradient_bar(&self) -> String {
        "▁▂▃▄▅▆▇█▇▆▅▄▃▂▁".to_string()
    }
}

/// Beautiful border sets for different UI elements
#[allow(dead_code)]
pub struct BorderSet {
    pub top_left: &'static str,
    pub top_right: &'static str,
    pub bottom_left: &'static str,
    pub bottom_right: &'static str,
    pub horizontal: &'static str,
    pub vertical: &'static str,
}

impl BorderSet {
    /// Rounded borders for a soft look
    pub fn rounded() -> Self {
        BorderSet {
            top_left: "╭",
            top_right: "╮",
            bottom_left: "╰",
            bottom_right: "╯",
            horizontal: "─",
            vertical: "│",
        }
    }

    /// Double borders for emphasis
    pub fn double() -> Self {
        BorderSet {
            top_left: "╔",
            top_right: "╗",
            bottom_left: "╚",
            bottom_right: "╝",
            horizontal: "═",
            vertical: "║",
        }
    }

    /// Thick borders for strong separation
    pub fn thick() -> Self {
        BorderSet {
            top_left: "┏",
            top_right: "┓",
            bottom_left: "┗",
            bottom_right: "┛",
            horizontal: "━",
            vertical: "┃",
        }
    }
}

/// Icons for better visual communication
pub struct Icons;

#[allow(dead_code)]
impl Icons {
    pub const CHECKBOX_EMPTY: &'static str = "□"; // White square
    pub const CHECKBOX_CHECKED: &'static str = "▣"; // Square with dot
    pub const STAR: &'static str = "★"; // Black star
    pub const STAR_EMPTY: &'static str = "☆"; // White star
    pub const ARROW_RIGHT: &'static str = "❯"; // Heavy right chevron
    pub const BULLET: &'static str = "•"; // Bullet
    pub const SPARKLE: &'static str = "◆"; // Black diamond
    pub const FIRE: &'static str = "▲"; // Black up triangle (priority high)
    pub const ROCKET: &'static str = "▶"; // Black right triangle
    pub const LIGHTNING: &'static str = "⚡"; // Lightning bolt
    pub const DIAMOND: &'static str = "◇"; // White diamond
    pub const CIRCLE: &'static str = "●"; // Black circle
    pub const TRIANGLE: &'static str = "▷"; // White right triangle

    // Additional modern icons
    pub const DOT: &'static str = "∙"; // Bullet operator
    pub const HEAVY_DOT: &'static str = "●"; // Black circle (more visible)
    pub const SQUARE: &'static str = "■"; // Black square
    pub const SQUARE_EMPTY: &'static str = "□"; // White square
    pub const PLUS: &'static str = "➕"; // Heavy plus
    pub const MINUS: &'static str = "➖"; // Heavy minus
    pub const CHECK: &'static str = "✓"; // Check mark
    pub const CROSS: &'static str = "✗"; // Ballot X
    pub const HEART: &'static str = "♥"; // Black heart
    pub const ARROW_UP: &'static str = "↑"; // Up arrow
    pub const ARROW_DOWN: &'static str = "↓"; // Down arrow
    pub const CLOCK: &'static str = "⏰"; // Alarm clock for due dates
}
