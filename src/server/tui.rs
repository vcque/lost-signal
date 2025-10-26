#![allow(clippy::all)]

use std::sync::Arc;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use lost_signal::common::types::{Position, Tile};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph},
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
        self.render_game_view(f, left_chunks[0]);

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

    fn render_game_view(&self, f: &mut Frame, area: Rect) {
        let world = self.states.world.lock().unwrap();

        // Calculate view center
        let center = self.get_view_center(&world);

        // Calculate visible area
        let view_width = area.width as usize;
        let view_height = area.height as usize;
        let map_size = 256; // MAP_SIZE constant

        let start_x = center.x.saturating_sub(view_width / 2);
        let start_y = center.y.saturating_sub(view_height / 2);

        // Render the view
        let mut lines = Vec::new();

        for y in 0..view_height {
            let mut line = String::new();
            for x in 0..view_width {
                let world_x = start_x + x;
                let world_y = start_y + y;

                if world_x >= map_size || world_y >= map_size {
                    line.push(' ');
                    continue;
                }

                let pos = Position {
                    x: world_x,
                    y: world_y,
                };

                // Check if there's an entity at this position
                if let Some(_entity) = world.entities.values().find(|e| e.position == pos) {
                    line.push('@');
                } else if Some(pos) == world.orb {
                    line.push('¤');
                } else {
                    // Render tile
                    let tile = world.tiles.at(pos);
                    let char = match tile {
                        Tile::Wall => '#',
                        Tile::Empty => '.',
                        Tile::Spawn => 'S',
                        Tile::Unknown => ' ',
                    };
                    line.push(char);
                }
            }
            lines.push(line);
        }

        let content = lines.join("\n");
        let game_title = format!("Game View - Turn {}", world.tick);
        let game_view =
            Paragraph::new(content).block(Block::default().borders(Borders::ALL).title(game_title));

        f.render_widget(game_view, area);
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
