use losig_core::leaderboard::Leaderboard;

use crate::{dispatch::Dispatch, services::Services, tui::GameTui, ws_server::WsServer};

mod action;
mod command;
mod dispatch;
mod foes;
mod fov;
mod game;
mod sense;
mod services;
mod stage;
mod tiled;
mod tui;
mod world;
mod ws_server;

fn main() {
    tui_logger::init_logger(log::LevelFilter::Debug).unwrap();
    tui_logger::set_default_level(log::LevelFilter::Debug);

    let (server, sm_tx, cm_rx) = WsServer::new();
    server.run();

    let world = tiled::load_tutorial().unwrap();
    let leaderboard = Leaderboard::default();
    let services = Services::new(world, leaderboard, sm_tx);

    let dispatch = Dispatch::new(services.clone(), cm_rx);
    dispatch.run();

    let mut tui = GameTui::new(services);
    tui.run().expect("Could not start TUI");
}
