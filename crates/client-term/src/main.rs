#![allow(clippy::all)]

use losig_client::adapter::Adapter;
use losig_core::types::AvatarId;

use crate::crossterm_adapter::CrosstermAdapter;
use crate::ws_client::WsClient;

mod crossterm_adapter;
mod tui;
mod ws_client;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tui_logger::init_logger(log::LevelFilter::Debug).unwrap();
    tui_logger::set_default_level(log::LevelFilter::Debug);

    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <avatar_id>", args[0]);
        eprintln!("Example: {} 42", args[0]);
        std::process::exit(1);
    }

    let avatar_id: AvatarId = args[1]
        .parse()
        .map_err(|_| "Avatar ID must be a valid number")?;

    let client = WsClient::new();
    let tui_adapter = CrosstermAdapter::new();
    Adapter {
        avatar_id,
        client,
        tui_adapter,
    }
    .run();
    Ok(())
}
