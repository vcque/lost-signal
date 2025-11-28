use std::sync::LazyLock;

use palette::Hsl;
use ratatui::style::Color;

pub struct Theme {
    pub palette: ThemePalette,
}

pub struct ThemePalette {
    pub ui: Color,
    pub ui_text: Color,
    pub ui_highlight: Color,
    pub ui_selected: Color,
    pub ui_disabled: Color,

    pub foe: Color,
    pub ally: Color,
    pub terrain: Color,
    pub terrain_unseen: Color,

    pub important: Color,
    pub avatar: Color,

    pub log_minor: Color,
    pub log_info: Color,
    pub log_warn: Color,
    pub log_grave: Color,
    pub log_averted: Color,
    pub log_revision_bg: Color,
    pub log_revision_fg: Color,

    pub page_info: Color,
}

pub static THEME: LazyLock<Theme> = LazyLock::new(|| Theme {
    palette: ThemePalette {
        foe: Color::from_hsl(Hsl::new(0.0, 1.0, 0.5)),
        ally: Color::from_hsl(Hsl::new(40.0, 1.0, 0.5)),
        terrain: Color::from_hsl(Hsl::new(270.0, 1.0, 0.5)),
        terrain_unseen: Color::from_hsl(Hsl::new(270.0, 0.0, 0.1)),

        ui: Color::White,
        ui_disabled: Color::from_hsl(Hsl::new(0.0, 0.0, 0.5)),
        ui_text: Color::from_hsl(Hsl::new(0.0, 0.0, 0.8)),
        ui_highlight: Color::White,
        ui_selected: Color::Rgb(0, 255, 0),
        avatar: Color::from_hsl(Hsl::new(220.0, 1.0, 0.5)),

        important: Color::from_hsl(Hsl::new(40.0, 1.0, 0.5)),

        log_minor: Color::from_hsl(Hsl::new(0.0, 0.0, 0.8)),
        log_info: Color::White,
        log_warn: Color::from_hsl(Hsl::new(40.0, 1.0, 0.5)),
        log_grave: Color::from_hsl(Hsl::new(0.0, 1.0, 0.5)),
        log_averted: Color::Rgb(139, 69, 19), // Dark brown
        log_revision_bg: Color::Cyan,
        log_revision_fg: Color::Black,

        page_info: Color::Gray,
    },
});
