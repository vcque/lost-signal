use std::sync::{Arc, Mutex, mpsc};

use crate::{
    command::CommandMessage, game::Game, sense::SensesMessage, states::States, tui::GameTui,
    world::load_world, ws_server::WsServer,
};

mod command;
mod game;
mod robot;
mod sense;
mod states;
mod tui;
mod udp_server;
mod world;
mod ws_server;

fn main() {
    tui_logger::init_logger(log::LevelFilter::Debug).unwrap();
    tui_logger::set_default_level(log::LevelFilter::Debug);

    let (sense_sender, sense_receiver) = mpsc::channel::<SensesMessage>();
    let (cmd_sender, cmd_receiver) = mpsc::channel::<CommandMessage>();

    let states = States {
        world: Mutex::new(load_world()),
        commands: cmd_sender,
        senses: sense_sender,
    };
    let states = Arc::new(states);

    // Start game loop in b:w
    // ackground
    let game = Game::new(states.clone(), cmd_receiver);
    game.run();

    // Run server
    let server = WsServer::new(states.clone(), sense_receiver);
    server.run();

    // Run TUI
    let mut tui = GameTui::new(states);
    tui.run().expect("Could not start TUI");
}
