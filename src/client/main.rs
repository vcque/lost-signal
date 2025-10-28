#![allow(clippy::all)]

use lost_signal::common::network::{UdpCommandPacket, UdpSensesPacket};
use lost_signal::common::types::EntityId;

use std::sync::mpsc::channel;

use crate::game::GameSim;
use crate::tui::Tui;

mod game;
mod tui;
mod udp_client;
mod world;
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

    let tui = Tui::new(game);
    tui.run();
    Ok(())
}
