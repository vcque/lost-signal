use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use losig_core::types::{Offset, Position, Tile};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::{services::Services, world::World};

pub struct GameTui {
    services: Services,
}

impl GameTui {
    pub fn new(services: Services) -> Self {
        Self { services }
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

            if event::poll(std::time::Duration::from_millis(50))?
                && let Event::Key(key) = event::read()?
                && let KeyCode::Char('q') = key.code
            {
                return Ok(());
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
        let world = self.services.world.lock().unwrap();
        let game_title = format!("Game View - Turn {}", world.tick);
        let borders = Block::default().borders(Borders::all()).title(game_title);

        let inner = borders.inner(area);
        borders.render(area, buf);
        let area = inner;

        let (stage_id, viewer) = self.get_view_center(&world);
        let area_offset = Offset {
            x: (area.width / 2) as isize,
            y: (area.height / 2) as isize,
        };

        let offset = viewer.as_offset() - area_offset;

        let stage = world.stages.get(stage_id).unwrap();
        let tiles = &stage.tiles;
        for x in 0..area.width as usize {
            for y in 0..area.height as usize {
                let pos = Position { x, y };

                let tile_pos = pos + offset;
                let tile = tiles.get(tile_pos);
                let char = match tile {
                    Tile::Spawn => 'S',
                    Tile::Unknown => ' ',
                    Tile::Empty => '.',
                    Tile::Wall => '#',
                    Tile::Pylon => '|',
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
        for avatar in world.avatars.values() {
            if avatar.stage != stage_id {
                continue;
            }
            let position = avatar.position;

            let Position { x, y } = position + offset;
            buf.set_string(
                area.x + x as u16,
                area.y + y as u16,
                "@",
                Style::default().green(),
            );
        }

        let Position { x, y } = stage.orb + offset;

        if (0..area.width).contains(&(x as u16)) && (0..area.height).contains(&(y as u16)) {
            buf.set_string(
                area.x + x as u16,
                area.y + y as u16,
                "¤",
                Style::default().yellow(),
            );
        }

        for foe in stage.foes.iter() {
            let position = foe.position;
            let Position { x, y } = position + offset;
            if (0..area.width).contains(&(x as u16)) && (0..area.height).contains(&(y as u16)) {
                buf.set_string(
                    area.x + x as u16,
                    area.y + y as u16,
                    "µ",
                    Style::default().red(),
                );
            }
        }
    }

    fn get_view_center(&self, world: &World) -> (usize, Position) {
        // Center on first avatar if exists, otherwise center of map
        if let Some(avatar) = world.avatars.values().next() {
            (avatar.stage, avatar.position)
        } else {
            let stage = world.stages.first().unwrap();
            (
                0,
                Position {
                    x: stage.tiles.width / 2, // MAP_SIZE / 2
                    y: stage.tiles.height / 2,
                },
            )
        }
    }
}
