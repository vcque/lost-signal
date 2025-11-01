#![allow(clippy::all)]

use std::sync::Arc;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use log::debug;
use losig_core::types::{MAP_SIZE, Offset, Position, Tile};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::{states::States, world::World};

pub struct GameTui {
    states: Arc<States>,
}

impl GameTui {
    pub fn new(states: Arc<States>) -> Self {
        Self { states }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let result = self.run_app(&mut terminal);

        // Cleanup
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        result
    }

    fn run_app(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            terminal.draw(|f| self.ui(f))?;

            if event::poll(std::time::Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    if let KeyCode::Char('q') = key.code {
                        return Ok(());
                    }
                }
            }
        }
    }

    fn ui(&self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(f.area());

        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(chunks[0]);

        // Game view
        self.render_game_view(left_chunks[0], f.buffer_mut());

        // Status bar
        let status = Paragraph::new("Press 'q' to quit")
            .block(Block::default().borders(Borders::ALL).title("Status"));
        f.render_widget(status, left_chunks[1]);

        // Logs panel - using tui-logger
        let logger_widget = tui_logger::TuiLoggerWidget::default().block(
            Block::default()
                .title("Game Logs")
                .borders(ratatui::widgets::Borders::ALL),
        );
        f.render_widget(logger_widget, chunks[1]);
    }

    fn render_game_view(&self, area: Rect, buf: &mut Buffer) {
        let world = self.states.world.lock().unwrap();
        let game_title = format!("Game View - Turn {}", world.tick);
        let borders = Block::default().borders(Borders::all()).title(game_title);

        let inner = borders.inner(area);
        borders.render(area, buf);
        let area = inner;

        let viewer = self.get_view_center(&world);
        let area_offset = Offset {
            x: (area.width / 2) as isize,
            y: (area.height / 2) as isize,
        };

        let offset = viewer.as_offset() - area_offset;

        for x in 0..area.width as usize {
            for y in 0..area.height as usize {
                let pos = Position { x, y };

                if pos.is_oob(MAP_SIZE, MAP_SIZE, offset) {
                    continue;
                }

                let tile_pos = pos + offset;
                let tile = world.tiles.at(tile_pos);
                let char = match tile {
                    Tile::Spawn => 'S',
                    Tile::Unknown => ' ',
                    Tile::Empty => '.',
                    Tile::Wall => '#',
                };

                buf.set_string(
                    area.x + x as u16,
                    area.y + y as u16,
                    char.to_string(),
                    Style::default(),
                );
            }
        }

        // Convert from world ref to view ref
        let offset = -offset;
        for entity in world.entities.values() {
            let position = entity.position;

            // Needs to check also x
            if !position.is_oob(area.width as usize, area.height as usize, offset) {
                let Position { x, y } = position + offset;
                buf.set_string(
                    area.x + x as u16,
                    area.y + y as u16,
                    "@",
                    Style::default().green(),
                );
            }
        }

        if let Some(position) = world.orb {
            // Needs to check also x
            if !position.is_oob(area.width as usize, area.height as usize, offset) {
                let Position { x, y } = position + offset;
                buf.set_string(
                    area.x + x as u16,
                    area.y + y as u16,
                    "¤",
                    Style::default().yellow(),
                );
            }
        }

        for foe in world.foes.iter() {
            let position = foe.position;

            // Needs to check also x
            if !position.is_oob(area.width as usize, area.height as usize, offset) {
                let Position { x, y } = position + offset;
                buf.set_string(
                    area.x + x as u16,
                    area.y + y as u16,
                    "µ",
                    Style::default().red(),
                );
            }
        }
    }

    fn get_view_center(&self, world: &World) -> Position {
        // Center on first entity if exists, otherwise center of map
        if let Some(entity) = world.entities.values().next() {
            entity.position
        } else {
            Position {
                x: 256 / 2, // MAP_SIZE / 2
                y: 256 / 2,
            }
        }
    }
}
