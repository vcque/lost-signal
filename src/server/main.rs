use std::sync::{Arc, Mutex};

use crate::{
    command::CommandQueue, game::Game, server::Server, states::States, tui::GameTui,
    world::load_world,
};

mod command;
mod game;
mod robot;
mod sense;
mod server;
mod states;
mod tui;
mod world;

fn main() {
    tui_logger::init_logger(log::LevelFilter::Trace).unwrap();
    tui_logger::set_default_level(log::LevelFilter::Trace);

    let states = States {
        world: Mutex::new(load_world()),
        command_queue: CommandQueue::new(),
    };
    let states = Arc::new(states);

    // Start game loop in background
    let game = Game::new(states.clone());
    game.run();

    // Start robot in background
    /*
    {
        let queue = queue.clone();
        spawn(|| {
            Robot::new(queue).run();
        });
    }
    */

    // Run server
    let server = Server::new(states.clone());
    server.run();

    // Run TUI
    let mut tui = GameTui::new(states);
    tui.run().expect("Could not start TUI");
}
