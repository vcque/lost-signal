use std::sync::{Arc, Mutex, mpsc};

use crate::{
    command::CommandMessage, game::Game, sense::SensesMessage, states::States, tui::GameTui,
    ws_server::WsServer,
};

mod command;
mod fov;
mod game;
mod sense;
mod states;
mod tiled;
mod tui;
mod world;
mod ws_server;

fn main() {
    tui_logger::init_logger(log::LevelFilter::Debug).unwrap();
    tui_logger::set_default_level(log::LevelFilter::Debug);

    let (sense_sender, sense_receiver) = mpsc::channel::<SensesMessage>();
    let (cmd_sender, cmd_receiver) = mpsc::channel::<CommandMessage>();

    let states = States {
        world: Mutex::new(tiled::load_world().unwrap()),
        commands: cmd_sender,
        senses: sense_sender,
    };
    let states = Arc::new(states);

    let game = Game::new(states.clone(), cmd_receiver);
    game.run();

    let server = WsServer::new(states.clone(), sense_receiver);
    server.run();

    let mut tui = GameTui::new(states);
    tui.run().expect("Could not start TUI");
}
