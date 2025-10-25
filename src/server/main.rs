use std::sync::{Arc, Mutex, mpsc};

use crate::{
    command::CommandQueue,
    game::Game,
    sense::{SensesMessage, SensesQueue},
    server::Server,
    states::States,
    tui::GameTui,
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

    let (sense_sender, sense_receiver) = mpsc::channel::<SensesMessage>();

    let states = States {
        world: Mutex::new(load_world()),
        commands: CommandQueue::new(),
        senses: sense_sender,
    };
    let states = Arc::new(states);

    // Start game loop in b:w
    // ackground
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
    let server = Server::new(states.clone(), sense_receiver);
    server.run();

    // Run TUI
    let mut tui = GameTui::new(states);
    tui.run().expect("Could not start TUI");
}
