use std::fmt::Display;
use losig_core::{sense::Senses, types::Action};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Style},
    widgets::{List, ListItem},
};

use crate::{
    tui::{
        component::Component,
        state::{PageSelection, TuiState},
        utils::center,
    },
    tui_adapter::{Event, KeyCode},
};

#[derive(Debug, Clone, Copy)]
pub enum MenuOption {
    Start,
    Continue,
}

impl Display for MenuOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            MenuOption::Start => "Start Game",
            MenuOption::Continue => "Continue Game",
        };
        f.write_str(string)
    }
}

const MENU_OPTIONS: &[MenuOption] = &[MenuOption::Start, MenuOption::Continue];

pub struct MenuPage {}

impl Component for MenuPage {
    type State = TuiState;

    fn on_event(self, event: &Event, state: &mut Self::State) -> bool {
        let Event::Key(key) = event else {
            return false;
        };

        let list_state = &mut state.menu.list_state;
        match key.code {
            KeyCode::Up => list_state.select_previous(),
            KeyCode::Down => list_state.select_next(),
            KeyCode::Enter => {
                if let Some(selection) = list_state.selected().map(|i| MENU_OPTIONS[i]) {
                    match selection {
                        MenuOption::Start => {
                            state
                                .external
                                .game
                                .lock()
                                .unwrap()
                                .act(Action::Spawn, Senses::default());
                        }
                        MenuOption::Continue => {}
                    }
                    state.page = PageSelection::Game;
                }
            }
            _ => {
                return false;
            }
        }

        true
    }

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let menu_items: Vec<ListItem> = MENU_OPTIONS
            .iter()
            .map(|option| ListItem::new(option.to_string()))
            .collect();

        let center = center(
            area,
            Constraint::Percentage(50),
            Constraint::Length(menu_items.len() as u16),
        );

        let menu_list = List::new(menu_items)
            .style(Style::default().fg(Color::Gray))
            .highlight_symbol("> ");

        ratatui::widgets::StatefulWidget::render(
            menu_list,
            center,
            buf,
            &mut state.menu.list_state,
        );
    }
}
