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
    pub ui_hp: Color,

    pub foe: Color,
    pub ally_leading: Color,
    pub ally_trailing: Color,
    pub ally_discarded: Color,
    pub ally_sync: Color,
    pub ally_next_move: Color,

    pub tile_wall: Color,
    pub tile_floor: Color,
    pub tile_unseen: Color,

    pub important: Color,
    pub avatar: Color,

    pub log_minor: Color,
    pub log_info: Color,
    pub log_warn: Color,
    pub log_grave: Color,
    pub log_averted: Color,
    pub log_revision_bg: Color,
    pub log_revision_fg: Color,

    pub timeline_tail: Hsl,
    pub timeline_head: Hsl,

    pub page_info: Color,
    pub ui_bar_empty: Color,
    pub ui_focus: Color,
}

pub static THEME: LazyLock<Theme> = LazyLock::new(|| Theme {
    palette: ThemePalette {
        foe: Color::from_hsl(Hsl::new(0.0, 1.0, 0.5)),
        ally_leading: Color::from_hsl(Hsl::new(40.0, 1.0, 0.5)),
        ally_trailing: Color::from_hsl(Hsl::new(180.0, 1.0, 0.5)),
        ally_sync: Color::from_hsl(Hsl::new(75.0, 0.5, 1.0)),
        ally_discarded: Color::from_hsl(Hsl::new(40.0, 0.2, 0.2)),
        ally_next_move: Color::from_hsl(Hsl::new(40.0, 0.7, 0.2)),

        tile_wall: Color::from_hsl(Hsl::new(270.0, 1.0, 0.5)),
        tile_floor: Color::from_hsl(Hsl::new(270.0, 0.2, 0.5)),
        tile_unseen: Color::from_hsl(Hsl::new(270.0, 0.0, 0.1)),

        ui: Color::White,
        ui_disabled: Color::from_hsl(Hsl::new(0.0, 0.0, 0.5)),
        ui_text: Color::from_hsl(Hsl::new(0.0, 0.0, 0.8)),
        ui_highlight: Color::White,
        ui_selected: Color::Rgb(0, 255, 0),
        ui_hp: Color::from_hsl(Hsl::new(115.0, 0.7, 0.3)),
        ui_bar_empty: Color::from_hsl(Hsl::new(115.0, 0.0, 0.5)),
        ui_focus: Color::from_hsl(Hsl::new(220.0, 1.0, 0.5)),

        avatar: Color::from_hsl(Hsl::new(220.0, 1.0, 0.5)),

        important: Color::from_hsl(Hsl::new(40.0, 1.0, 0.5)),

        log_minor: Color::from_hsl(Hsl::new(0.0, 0.0, 0.8)),
        log_info: Color::White,
        log_warn: Color::from_hsl(Hsl::new(40.0, 1.0, 0.5)),
        log_grave: Color::from_hsl(Hsl::new(0.0, 1.0, 0.5)),
        log_averted: Color::Rgb(139, 69, 19), // Dark brown
        log_revision_bg: Color::Cyan,
        log_revision_fg: Color::Black,

        timeline_tail: Hsl::new(180.0, 1.0, 0.5),
        timeline_head: Hsl::new(40.0, 1.0, 0.5),
        page_info: Color::Gray,
    },
});
