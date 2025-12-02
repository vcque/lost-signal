use losig_core::{
    leaderboard::Leaderboard,
    sense::Senses,
    types::{ClientAction, GameOverStatus},
};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    widgets::{Block, Borders, Cell, List, ListItem, Row, Table, Widget},
};
use std::fmt::Display;

use crate::{
    tui::{
        InputServices, MenuState, RenderServices, THEME,
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

impl MenuPage {
    pub fn on_event(
        self,
        event: &Event,
        state: &mut TuiState,
        mut services: InputServices,
    ) -> bool {
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
                            services.new_game();
                            services.act(ClientAction::Wait, Default::default());
                        }
                        MenuOption::Continue => {
                            services.act(ClientAction::Wait, Default::default());
                        }
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

    pub fn render(
        self,
        area: Rect,
        buf: &mut Buffer,
        state: &mut MenuState,
        services: RenderServices,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        // Menu on the left
        let menu_items: Vec<ListItem> = MENU_OPTIONS
            .iter()
            .map(|option| ListItem::new(option.to_string()))
            .collect();

        let menu_center = center(
            chunks[0],
            Constraint::Percentage(50),
            Constraint::Length(menu_items.len() as u16),
        );

        let menu_list = List::new(menu_items)
            .style(Style::default().fg(THEME.palette.ui_text))
            .highlight_style(Style::default().bold())
            .highlight_symbol("> ");

        ratatui::widgets::StatefulWidget::render(
            menu_list,
            menu_center,
            buf,
            &mut state.list_state,
        );

        // Leaderboard on the right
        let leaderboard_widget = LeaderboardWidget::new(&services.state.leaderboard);
        leaderboard_widget.render(chunks[1], buf);
    }
}

struct LeaderboardWidget<'a> {
    leaderboard: &'a Leaderboard,
    max_entries: usize,
}

impl<'a> LeaderboardWidget<'a> {
    fn new(leaderboard: &'a Leaderboard) -> Self {
        Self {
            leaderboard,
            max_entries: 10,
        }
    }
}

impl<'a> Widget for LeaderboardWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let area = center(area, Constraint::Length(36), Constraint::Length(12));
        let top_entries = self.leaderboard.top_entries(self.max_entries);

        let header = Row::new(vec![
            Cell::from("Rank").style(Style::default().bold()),
            Cell::from("Name").style(Style::default().bold()),
            Cell::from("Stage").style(Style::default().bold()),
            Cell::from("Turns").style(Style::default().bold()),
            Cell::from("Score").style(Style::default().bold()),
        ]);

        let mut rows = Vec::new();
        let actual_entries = top_entries.iter().rev().collect::<Vec<_>>();

        for i in 0..self.max_entries {
            let rank = i + 1;
            if let Some(entry) = actual_entries.get(i) {
                let row = Row::new(vec![
                    Cell::from(rank.to_string()),
                    Cell::from(entry.name.clone()),
                    Cell::from(if entry.gameover.status == GameOverStatus::Win {
                        "WIN".to_owned()
                    } else {
                        entry.gameover.stage.to_string()
                    }),
                    Cell::from(entry.gameover.turns.to_string()),
                    Cell::from(entry.gameover.score.to_string()),
                ]);
                rows.push(row);
            } else {
                let row = Row::new(vec![
                    Cell::from(rank.to_string()),
                    Cell::from("-"),
                    Cell::from("-"),
                    Cell::from("-"),
                    Cell::from("-"),
                ])
                .style(THEME.palette.ui_disabled);
                rows.push(row);
            }
        }

        let leaderboard_table = Table::new(
            rows,
            [
                Constraint::Length(4), // Rank
                Constraint::Length(9), // Name
                Constraint::Length(6), // Deaths
                Constraint::Length(6), // Turns
                Constraint::Length(6), // Score
            ],
        )
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("🏆 Leaderboard"),
        )
        .style(THEME.palette.important);

        leaderboard_table.render(area, buf);
    }
}
