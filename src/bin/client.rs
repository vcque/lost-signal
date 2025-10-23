#![allow(clippy::all)]

use crossterm::event::{self, Event, KeyCode, KeyEvent};
use lost_signal::{Command, Direction, server::UdpPacket};

use std::io::{self, Write};
use std::net::UdpSocket;

const SERVER_ADDR: &str = "127.0.0.1:8080";

struct Client {
    socket: UdpSocket,
    entity_id: u64,
}

impl Client {
    fn new(entity_id: u64) -> Result<Self, Box<dyn std::error::Error>> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.connect(SERVER_ADDR)?;

        Ok(Self { socket, entity_id })
    }

    fn send_command(&mut self, command: Command) -> Result<(), Box<dyn std::error::Error>> {
        let cmd = UdpPacket {
            entity_id: self.entity_id,
            content: command,
            tick: None,
        };

        let json = serde_json::to_string(&cmd)?;
        self.socket.send(json.as_bytes())?;

        Ok(())
    }

    fn handle_key(&mut self, key: KeyCode) -> Result<bool, Box<dyn std::error::Error>> {
        match key {
            KeyCode::Char('q') => return Ok(false), // Quit
            KeyCode::Char('s') | KeyCode::Char('S') => {
                self.send_command(Command::Spawn)?;
            }
            // Numpad movements
            KeyCode::Char('8') => self.send_command(Command::Move(Direction::Up))?,
            KeyCode::Char('2') => self.send_command(Command::Move(Direction::Down))?,
            KeyCode::Char('4') => self.send_command(Command::Move(Direction::Left))?,
            KeyCode::Char('6') => self.send_command(Command::Move(Direction::Right))?,
            KeyCode::Char('7') => self.send_command(Command::Move(Direction::UpLeft))?,
            KeyCode::Char('9') => self.send_command(Command::Move(Direction::UpRight))?,
            KeyCode::Char('1') => self.send_command(Command::Move(Direction::DownLeft))?,
            KeyCode::Char('3') => self.send_command(Command::Move(Direction::DownRight))?,
            // Arrow keys as backup
            KeyCode::Up => self.send_command(Command::Move(Direction::Up))?,
            KeyCode::Down => self.send_command(Command::Move(Direction::Down))?,
            KeyCode::Left => self.send_command(Command::Move(Direction::Left))?,
            KeyCode::Right => self.send_command(Command::Move(Direction::Right))?,
            _ => {
                println!("Unknown key. Use 's' to spawn, numpad (1-9) to move, 'q' to quit");
            }
        }
        Ok(true)
    }

    fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("UDP Game Client");
        println!("Commands:");
        println!("  's' - Spawn entity");
        println!("  Numpad 1-9 - Move in 8 directions");
        println!("  Arrow keys - Move in 4 directions");
        println!("  'q' - Quit");
        println!("Connected to server at {}", SERVER_ADDR);
        print!("\n> ");
        io::stdout().flush()?;

        loop {
            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                    if !self.handle_key(code)? {
                        break;
                    }
                    print!("> ");
                    io::stdout().flush()?;
                }
            }
        }

        println!("\nClient disconnected.");
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() != 2 {
        eprintln!("Usage: {} <entity_id>", args[0]);
        eprintln!("Example: {} 42", args[0]);
        std::process::exit(1);
    }
    
    let entity_id: u64 = args[1].parse()
        .map_err(|_| "Entity ID must be a valid number")?;
    
    println!("Starting client with entity ID: {}", entity_id);
    
    let mut client = Client::new(entity_id)?;
    client.run()
}
