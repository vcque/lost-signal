use std::sync::LazyLock;

use ratatui::style::{Color, Style, Stylize};

pub struct Theme {
    pub styles: ThemeStyles,
    pub palette: ThemePalette,
}

pub struct ThemeStyles {
    pub active: Style,
    pub inactive: Style,
    pub selection: Style,
    pub danger: Style,
    pub focus: Style,
}

pub struct ThemePalette {
    pub background_primary: Color,
    pub foreground_primary: Color,
    pub foreground_secondary: Color,
    pub foreground_muted: Color,
    pub foreground_dark: Color,
    pub accent_success: Color,
    pub accent_warning: Color,
    pub accent_danger: Color,
    pub accent_info: Color,
    pub accent_focus: Color,
    pub game_spawn: Color,
    pub game_pylon_bg: Color,
    pub game_pylon_fg: Color,
    pub game_wall_bg: Color,
    pub popup_bg: Color,
    pub popup_fg: Color,
    pub page_info: Color,
}

pub static THEME: LazyLock<Theme> = LazyLock::new(|| Theme {
    palette: ThemePalette {
        background_primary: Color::Black,
        foreground_primary: Color::White,
        foreground_secondary: Color::Yellow,
        foreground_muted: Color::Gray,
        foreground_dark: Color::DarkGray,
        accent_success: Color::Green,
        accent_warning: Color::Yellow,
        accent_danger: Color::Red,
        accent_info: Color::Cyan,
        accent_focus: Color::LightYellow,
        game_spawn: Color::LightYellow,
        game_pylon_bg: Color::Gray,
        game_pylon_fg: Color::LightBlue,
        game_wall_bg: Color::Gray,
        popup_bg: Color::Black,
        popup_fg: Color::White,
        page_info: Color::Gray,
    },
    styles: ThemeStyles {
        active: Style::default(),
        inactive: Style::default().fg(Color::Gray),
        selection: Style::default().fg(Color::LightGreen).bold(),
        danger: Style::default().fg(Color::Red),
        focus: Style::default().fg(Color::LightYellow),
    },
});
