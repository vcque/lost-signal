#![allow(clippy::all)]

use losig_client::game::GameSim;
use losig_client::tui::GameTui;
use losig_core::network::{UdpCommandPacket, UdpSensesPacket};
use losig_core::types::EntityId;

use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread::spawn;

mod crossterm_adapter;
mod tui;
mod udp_client;
mod ws_client;

pub type SenseMessage = UdpSensesPacket;
pub type CommandMessage = UdpCommandPacket;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tui_logger::init_logger(log::LevelFilter::Debug).unwrap();
    tui_logger::set_default_level(log::LevelFilter::Debug);

    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <entity_id>", args[0]);
        eprintln!("Example: {} 42", args[0]);
        std::process::exit(1);
    }

    let entity_id: EntityId = args[1]
        .parse()
        .map_err(|_| "Entity ID must be a valid number")?;

    let (senses_tx, senses_rx) = channel::<SenseMessage>();
    let (cmd_tx, cmd_rx) = channel::<CommandMessage>();

    let client = ws_client::WsClient::new(cmd_rx, senses_tx);
    client.run();

    let mut game = GameSim::new(entity_id);
    game.set_callback(Box::new(move |msg| {
        cmd_tx.send(msg).unwrap();
    }));

    let game = Arc::new(Mutex::new(game));
    {
        let game = game.clone();
        spawn(move || {
            loop {
                let sense = senses_rx.recv().unwrap();
                game.lock().unwrap().update(sense.senses);
            }
        });
    }
    let app = GameTui::new(game);

    let adapter = crossterm_adapter::CrosstermAdapter::new(app);
    adapter.run();

    Ok(())
}
