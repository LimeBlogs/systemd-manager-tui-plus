use ratatui::{
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Padding},
};

/// Japandi Theme: A fusion of Japanese Zen minimalism (Kanso, Ma)
/// and Scandinavian functional warmth (Lagom, Hygge).
///
/// Features muted natural earth tones, restrained glyphs, and generous breathing space.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct Theme {
    // Surface & Borders
    pub border: Color,
    pub border_focused: Color,
    pub border_subtle: Color,

    // Typography
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_muted: Color,
    pub text_dim: Color,

    // Accents & Warmth
    pub accent: Color,        // Warm Sand / Amber (暖木沙)
    pub accent_subtle: Color, // Muted Sky / Slate Blue (青瓷灰蓝)
    pub accent_warm: Color,   // Muted Ochre (暖赭)

    // Status (Nature-derived palettes)
    pub status_active: Color,     // Sage Moss (苔绿)
    pub status_activating: Color, // Warm Amber (琥珀)
    pub status_inactive: Color,   // Stone Gray (浅灰石)
    pub status_failed: Color,     // Terracotta (陶土红)

    // Selection
    pub selection_bg: Color,          // Charcoal Slate (深炭石底)
    pub selection_fg: Color,          // Off-white / Washi (和纸白)
    pub selection_disabled_bg: Color, // Muted Stone (暗石灰)
}

impl Default for Theme {
    fn default() -> Self {
        Self::japandi()
    }
}

impl Theme {
    /// Japandi Default: Warm, organic, tranquil, low eye strain.
    pub const fn japandi() -> Self {
        Self {
            // Borders: Slate rock grey, no harsh cyan
            border: Color::Rgb(74, 84, 94),
            border_focused: Color::Rgb(212, 163, 115),
            border_subtle: Color::Rgb(52, 60, 68),

            // Typography: Washi off-white down to quiet stone
            text_primary: Color::Rgb(238, 236, 230),
            text_secondary: Color::Rgb(185, 190, 196),
            text_muted: Color::Rgb(138, 146, 154),
            text_dim: Color::Rgb(95, 102, 110),

            // Accents: Warm wood, natural amber, slate blue
            accent: Color::Rgb(212, 163, 115),
            accent_subtle: Color::Rgb(138, 165, 182),
            accent_warm: Color::Rgb(224, 172, 105),

            // Status: Muted nature tones
            status_active: Color::Rgb(126, 160, 136),
            status_activating: Color::Rgb(224, 172, 105),
            status_inactive: Color::Rgb(115, 122, 132),
            status_failed: Color::Rgb(201, 106, 101),

            // Selection: Charcoal surface with gentle contrast
            selection_bg: Color::Rgb(40, 46, 54),
            selection_fg: Color::Rgb(248, 246, 240),
            selection_disabled_bg: Color::Rgb(52, 58, 66),
        }
    }

    /// Global singleton accessor
    pub const fn current() -> &'static Self {
        &THEME
    }

    // --- Style helpers ---

    pub fn primary_style(&self) -> Style {
        Style::default().fg(self.text_primary)
    }

    pub fn secondary_style(&self) -> Style {
        Style::default().fg(self.text_secondary)
    }

    pub fn muted_style(&self) -> Style {
        Style::default().fg(self.text_muted)
    }

    pub fn dim_style(&self) -> Style {
        Style::default().fg(self.text_dim)
    }

    pub fn accent_style(&self) -> Style {
        Style::default().fg(self.accent).add_modifier(Modifier::BOLD)
    }

    pub fn active_style(&self) -> Style {
        Style::default().fg(self.status_active).add_modifier(Modifier::BOLD)
    }

    pub fn activating_style(&self) -> Style {
        Style::default().fg(self.status_activating)
    }

    pub fn inactive_style(&self) -> Style {
        Style::default().fg(self.status_inactive)
    }

    pub fn failed_style(&self) -> Style {
        Style::default().fg(self.status_failed).add_modifier(Modifier::BOLD)
    }

    pub fn selection_style(&self) -> Style {
        Style::default()
            .bg(self.selection_bg)
            .fg(self.selection_fg)
            .add_modifier(Modifier::BOLD)
    }

    pub fn selection_disabled_style(&self) -> Style {
        Style::default()
            .bg(self.selection_disabled_bg)
            .fg(self.text_secondary)
            .add_modifier(Modifier::BOLD)
    }

    // --- Block & Container helpers ---

    /// Standard rounded card with calm slate borders
    #[allow(dead_code)]
    pub fn card<'a>(&self, title: impl Into<ratatui::text::Line<'a>>, focused: bool) -> Block<'a> {
        let border_color = if focused { self.border_focused } else { self.border };
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(border_color))
            .padding(Padding::horizontal(1))
            .title(title)
            .title_style(if focused {
                Style::default().fg(self.accent).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(self.text_secondary)
            })
    }

    /// Dialog card with breathing room (Ma / 间)
    pub fn dialog<'a>(&self, title: impl Into<ratatui::text::Line<'a>>) -> Block<'a> {
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(self.border_focused))
            .padding(Padding::new(2, 2, 1, 1))
            .title(title)
            .title_style(Style::default().fg(self.accent).add_modifier(Modifier::BOLD))
    }
}

pub static THEME: Theme = Theme::japandi();

/// Japandi minimalist status glyphs
#[allow(dead_code)]
pub mod glyphs {
    pub const STATUS_ACTIVE: &str = "● ";
    pub const STATUS_ACTIVATING: &str = "◐ ";
    pub const STATUS_INACTIVE: &str = "○ ";
    pub const STATUS_FAILED: &str = "✕ ";

    pub const SELECTION_RAIL: &str = "▎ ";
    pub const UNSELECTED_SPACE: &str = "  ";

    pub const CHECKED: &str = "[ ✓ 是 ]";
    pub const UNCHECKED: &str = "[ ─ 否 ]";
    pub const CHECKED_EN: &str = "[ ✓ Yes ]";
    pub const UNCHECKED_EN: &str = "[ ─ No  ]";

    pub const BULLET: &str = " · ";
    pub const SEPARATOR: &str = "  │  ";
}
