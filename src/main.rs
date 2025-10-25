use std::sync::{Arc, Mutex};

use crate::{
    command::CommandQueue, game::Game, robot::Robot, server::Server, tui::GameTui,
    world::load_world,
};

mod command;
mod entity;
mod game;
mod robot;
mod sense;
mod server;
mod tui;
mod world;

fn main() {
    tui_logger::init_logger(log::LevelFilter::Trace).unwrap();
    tui_logger::set_default_level(log::LevelFilter::Trace);

    let world = Arc::new(Mutex::new(load_world()));
    let queue = CommandQueue::new();

    // Start game loop in background
    let game = Game::new(&world, &queue);
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
    let server = Server::new(&world, &queue);
    server.run();

    // Run TUI
    let mut tui = GameTui::new(world);
    tui.run().expect("Could not start TUI");
}
