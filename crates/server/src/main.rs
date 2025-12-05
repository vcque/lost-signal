use losig_core::leaderboard::Leaderboard;

use crate::{dispatch::Dispatch, services::Services, ws_server::WsServer};

#[cfg(feature = "tui")]
use crate::tui::GameTui;

mod action;
mod command;
mod dispatch;
mod events;
mod foes;
mod game;
mod sense;
mod sense_bounds;
mod services;
mod stage;
mod tiled;
mod world;
mod ws_server;

#[cfg(feature = "tui")]
mod tui;

fn main() {
    let (server, sm_tx, cm_rx) = WsServer::new();
    server.run();

    let world = tiled::load_arena().unwrap();
    let leaderboard = Leaderboard::default();
    let services = Services::new(world, leaderboard, sm_tx);

    let dispatch = Dispatch::new(services.clone(), cm_rx);
    dispatch.run();

    #[cfg(feature = "tui")]
    {
        tui_logger::init_logger(log::LevelFilter::Debug).unwrap();
        tui_logger::set_default_level(log::LevelFilter::Debug);
        let mut tui = GameTui::new(services);
        tui.run().expect("Could not start TUI");
    }

    #[cfg(not(feature = "tui"))]
    {
        env_logger::Builder::from_default_env()
            .target(env_logger::Target::Stdout)
            .filter_level(log::LevelFilter::Debug)
            .init();

        log::info!("Server running in headless mode. Press Ctrl+C to stop.");
        // Wait for Ctrl+C signal
        let _ = std::sync::mpsc::channel::<()>().1.recv();
    }
}
