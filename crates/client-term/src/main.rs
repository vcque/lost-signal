#![allow(clippy::all)]

use losig_client::game::GameSim;
use losig_client::tui::GameTui;
use losig_core::network::{UdpCommandPacket, UdpSensesPacket};
use losig_core::types::EntityId;

use std::sync::mpsc::channel;

mod crossterm_adapter;
mod tui;
mod udp_client;
mod ws_client;

pub type SenseMessage = UdpSensesPacket;
pub type CommandMessage = UdpCommandPacket;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tui_logger::init_logger(log::LevelFilter::Trace).unwrap();
    tui_logger::set_default_level(log::LevelFilter::Trace);

    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <entity_id>", args[0]);
        eprintln!("Example: {} 42", args[0]);
        std::process::exit(1);
    }

    let entity_id: EntityId = args[1]
        .parse()
        .map_err(|_| "Entity ID must be a valid number")?;

    println!("Starting client with entity ID: {}", entity_id);

    let (senses_send, senses_recv) = channel::<SenseMessage>();
    let (cmd_send, cmd_recv) = channel::<CommandMessage>();

    let client = ws_client::WsClient::new(cmd_recv, senses_send);
    client.run();

    let mut game = GameSim::new(entity_id, cmd_send, senses_recv);
    game.run();

    let app = GameTui::new(game);

    let adapter = crossterm_adapter::CrosstermAdapter::new(app);
    adapter.run();

    println!("Doesn't work?");
    Ok(())
}
