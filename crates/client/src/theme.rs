//! Tui theme

use std::sync::LazyLock;

use ratatui::style::{Color, Style, Stylize};

pub struct Theme {
    pub styles: ThemeStyles,
}

pub struct ThemeStyles {
    pub active: Style,
    pub inactive: Style,
    pub selection: Style,
    pub danger: Style,
    pub signal: Style,
}

pub static THEME: LazyLock<Theme> = LazyLock::new(|| Theme {
    styles: ThemeStyles {
        active: Style::default(),
        inactive: Style::default().fg(Color::Gray),
        selection: Style::default().fg(Color::LightGreen).bold(),
        danger: Style::default().fg(Color::Red),
        signal: Style::default().fg(Color::LightYellow),
    },
});

