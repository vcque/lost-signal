#![allow(clippy::all)]

use crossterm::event::{self, Event, KeyCode, KeyEvent};
use lost_signal::common::action::Action;
use lost_signal::common::network::{UdpCommandPacket, UdpSensesPacket};
use lost_signal::common::sense::{SenseInfo, Senses, TerrainInfo, TerrainSense, WorldSense};
use lost_signal::common::types::{Direction, Tile};

use std::io;
use std::net::UdpSocket;

use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction as LayoutDirection, Layout},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

const SERVER_ADDR: &str = "127.0.0.1:8080";

struct NetworkClient {
    socket: UdpSocket,
    entity_id: u64,
}

impl NetworkClient {
    fn new(entity_id: u64) -> Result<Self, Box<dyn std::error::Error>> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.connect(SERVER_ADDR)?;
        socket.set_nonblocking(true)?;

        Ok(Self { socket, entity_id })
    }

    fn send_action(&mut self, action: Action) -> Result<(), Box<dyn std::error::Error>> {
        let cmd = UdpCommandPacket {
            entity_id: self.entity_id,
            action,
            tick: None,
            senses: Senses {
                world: Some(WorldSense {}),
                terrain: Some(TerrainSense { radius: 5 }),
            },
        };

        let binary_data = bincode::serialize(&cmd)?;
        self.socket.send(&binary_data)?;

        Ok(())
    }

    fn check_messages(&mut self) -> Result<Vec<UdpSensesPacket>, Box<dyn std::error::Error>> {
        let mut packets = Vec::new();
        let mut buffer = [0; 1024];
        
        loop {
            match self.socket.recv(&mut buffer) {
                Ok(size) => {
                    match bincode::deserialize::<UdpSensesPacket>(&buffer[..size]) {
                        Ok(packet) => packets.push(packet),
                        Err(e) => eprintln!("Failed to deserialize packet: {}", e),
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    break;
                }
                Err(e) => return Err(e.into()),
            }
        }
        Ok(packets)
    }
}

struct GameTUI {
    network_client: NetworkClient,
    terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
    events: Vec<String>,
    world_state: WorldView,
    senses_data: Option<SenseInfo>,
}

impl GameTUI {
    fn new(entity_id: u64) -> Result<Self, Box<dyn std::error::Error>> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        let network_client = NetworkClient::new(entity_id)?;
        Ok(Self {
            network_client,
            terminal,
            events: Vec::new(),
            world_state: WorldView::new(),
            senses_data: None,
        })
    }

    fn cleanup(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        disable_raw_mode()?;
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen)?;
        self.terminal.show_cursor()?;
        Ok(())
    }

    fn add_event(&mut self, event: String) {
        self.events.push(event);
        if self.events.len() > 100 {
            self.events.remove(0);
        }
    }

    fn process_messages(&mut self, packets: Vec<UdpSensesPacket>) {
        for packet in packets {
            self.add_event("Received binary packet".to_string());
            
            if let Some(ref terrain_info) = packet.senses.terrain {
                self.world_state.apply(&terrain_info);
            }
            self.senses_data = Some(packet.senses);
            self.add_event("Updated senses data".to_string());
        }
    }

    fn handle_key(&mut self, key: KeyCode) -> Result<bool, Box<dyn std::error::Error>> {
        match key {
            KeyCode::Char('q') => return Ok(false),
            KeyCode::Char('s') | KeyCode::Char('S') => {
                self.network_client.send_action(Action::Spawn)?;
                self.add_event("Sent: Spawn action".to_string());
            }
            // Numpad movements
            KeyCode::Char('8') => {
                self.network_client.send_action(Action::Move(Direction::Up))?;
                self.add_event("Sent: Move Up".to_string());
            }
            KeyCode::Char('2') => {
                self.network_client.send_action(Action::Move(Direction::Down))?;
                self.add_event("Sent: Move Down".to_string());
            }
            KeyCode::Char('4') => {
                self.network_client.send_action(Action::Move(Direction::Left))?;
                self.add_event("Sent: Move Left".to_string());
            }
            KeyCode::Char('6') => {
                self.network_client.send_action(Action::Move(Direction::Right))?;
                self.add_event("Sent: Move Right".to_string());
            }
            KeyCode::Char('7') => {
                self.network_client.send_action(Action::Move(Direction::UpLeft))?;
                self.add_event("Sent: Move Up-Left".to_string());
            }
            KeyCode::Char('9') => {
                self.network_client.send_action(Action::Move(Direction::UpRight))?;
                self.add_event("Sent: Move Up-Right".to_string());
            }
            KeyCode::Char('1') => {
                self.network_client.send_action(Action::Move(Direction::DownLeft))?;
                self.add_event("Sent: Move Down-Left".to_string());
            }
            KeyCode::Char('3') => {
                self.network_client.send_action(Action::Move(Direction::DownRight))?;
                self.add_event("Sent: Move Down-Right".to_string());
            }
            // Arrow keys as backup
            KeyCode::Up => {
                self.network_client.send_action(Action::Move(Direction::Up))?;
                self.add_event("Sent: Move Up".to_string());
            }
            KeyCode::Down => {
                self.network_client.send_action(Action::Move(Direction::Down))?;
                self.add_event("Sent: Move Down".to_string());
            }
            KeyCode::Left => {
                self.network_client.send_action(Action::Move(Direction::Left))?;
                self.add_event("Sent: Move Left".to_string());
            }
            KeyCode::Right => {
                self.network_client.send_action(Action::Move(Direction::Right))?;
                self.add_event("Sent: Move Right".to_string());
            }
            _ => {
                self.add_event("Unknown key. Use 's' to spawn, numpad (1-9) to move, 'q' to quit".to_string());
            }
        }
        Ok(true)
    }

    fn draw(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.terminal.draw(|f| {
            let size = f.area();
            
            // Create main layout: left side for world, right side for senses
            let main_chunks = Layout::default()
                .direction(LayoutDirection::Horizontal)
                .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
                .split(size);
            
            // Split left side: world on top, events on bottom
            let left_chunks = Layout::default()
                .direction(LayoutDirection::Vertical)
                .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
                .split(main_chunks[0]);
            
            // World pane
            let world_block = Block::default()
                .title("World")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White));
            let world_content = self.world_state.render_styled(
                left_chunks[0].width as usize, 
                left_chunks[0].height as usize
            );
            let world_paragraph = Paragraph::new(world_content)
                .block(world_block);
            f.render_widget(world_paragraph, left_chunks[0]);
            
            // Events pane
            let events_block = Block::default()
                .title("Events")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White));
            let events_items: Vec<ListItem> = self.events
                .iter()
                .rev()
                .take(left_chunks[1].height.saturating_sub(2) as usize)
                .map(|event| ListItem::new(Line::from(Span::raw(event))))
                .collect();
            let events_list = List::new(events_items)
                .block(events_block)
                .style(Style::default().fg(Color::Yellow));
            f.render_widget(events_list, left_chunks[1]);
            
            // Senses pane
            let senses_block = Block::default()
                .title("Senses")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White));
            
            let senses_content = if let Some(ref senses) = self.senses_data {
                let mut content = String::new();
                if let Some(ref world_info) = senses.world {
                    content.push_str(&format!("Tick: {}\n", world_info.tick));
                    content.push_str(&format!("Tick Duration: {:?}\n\n", world_info.tick_duration));
                } else {
                    content.push_str("No world info\n\n");
                }
                
                if let Some(ref terrain_info) = senses.terrain {
                    content.push_str(&format!("Terrain Radius: {}\n", terrain_info.radius));
                    content.push_str(&format!("Tiles Received: {}\n\n", terrain_info.tiles.len()));
                } else {
                    content.push_str("No terrain info\n\n");
                }
                
                content.push_str("Controls:\n's' - Spawn\nNumpad/Arrows - Move\n'q' - Quit");
                content
            } else {
                "No senses data available\n\nControls:\n's' - Spawn\nNumpad/Arrows - Move\n'q' - Quit".to_string()
            };
            
            let senses_paragraph = Paragraph::new(senses_content)
                .block(senses_block)
                .style(Style::default().fg(Color::Cyan));
            f.render_widget(senses_paragraph, main_chunks[1]);
        })?;
        Ok(())
    }
    
    fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.add_event(format!("Connected to server at {}", SERVER_ADDR));
        self.add_event("Controls: 's' spawn, numpad/arrows move, 'q' quit".to_string());
        
        loop {
            let packets = self.network_client.check_messages()?;
            if !packets.is_empty() {
                self.process_messages(packets);
            }
            
            self.draw()?;
            
            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                    if !self.handle_key(code)? {
                        break;
                    }
                }
            }
        }
        
        self.cleanup()?;
        Ok(())
    }
}


const VIEW_SIZE: usize = 32;
struct WorldView {
    tiles: [Tile; VIEW_SIZE * VIEW_SIZE],
}

impl WorldView {
    fn new() -> WorldView {
      WorldView {
            tiles: [Tile::Unknown; VIEW_SIZE * VIEW_SIZE]
        }
    }

    /// Add new info from the server
    fn apply(&mut self, terrain: &TerrainInfo) {
        // view is always centered
        let center = VIEW_SIZE / 2;
        let radius = terrain.radius;
        for x in 0..(2 * radius + 1) {
            for y in 0..(2 * radius + 1) {
                let tile = terrain.tiles[x + (2 * radius + 1) * y];
                if !matches!(tile, Tile::Unknown) {
                    let x_view = center - radius + x;
                    let y_view = center - radius + y;
                    self.tiles[x_view + VIEW_SIZE * y_view] = tile;
                }
            }
        }
    }
    
    fn render_styled(&self, width: usize, height: usize) -> Text<'_> {
        let mut lines = Vec::new();
        
        // Calculate visible area based on available space
        let visible_width = (width.saturating_sub(2)).min(VIEW_SIZE); // -2 for borders
        let visible_height = (height.saturating_sub(2)).min(VIEW_SIZE); // -2 for borders
        
        // Center the view
        let start_x = (VIEW_SIZE - visible_width) / 2;
        let start_y = (VIEW_SIZE - visible_height) / 2;
        
        for y in start_y..(start_y + visible_height) {
            let mut spans = Vec::new();
            for x in start_x..(start_x + visible_width) {
                let (tile, style) = if x == VIEW_SIZE / 2 &&  y == VIEW_SIZE / 2 {
                    ('@', Color::White)
                } else {
                    let tile = self.tiles[x + VIEW_SIZE * y];
                    (Self::tile_to_char(tile), Self::tile_to_color(tile))
                };

                spans.push(Span::styled(
                    tile.to_string(),
                    Style::default().fg(style)
                ));
            }
            lines.push(Line::from(spans));
        }
        
        Text::from(lines)
    }
    
    fn tile_to_char(tile: Tile) -> char {
        match tile {
            Tile::Wall => '#',
            Tile::Empty => '.',
            Tile::Spawn => 'S',
            Tile::Orb => 'O',
            Tile::Unknown => ' ',
        }
    }
    
    fn tile_to_color(tile: Tile) -> Color {
        match tile {
            Tile::Wall => Color::Gray,
            Tile::Empty => Color::White,
            Tile::Spawn => Color::Blue,
            Tile::Orb => Color::Yellow,
            Tile::Unknown => Color::Black,
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <entity_id>", args[0]);
        eprintln!("Example: {} 42", args[0]);
        std::process::exit(1);
    }

    let entity_id: u64 = args[1]
        .parse()
        .map_err(|_| "Entity ID must be a valid number")?;

    println!("Starting client with entity ID: {}", entity_id);

    let mut tui = GameTUI::new(entity_id)?;
    let result = tui.run();
    
    // Ensure cleanup happens even if run() fails
    if let Err(cleanup_err) = tui.cleanup() {
        eprintln!("Cleanup error: {}", cleanup_err);
    }
    
    result
}
