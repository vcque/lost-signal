use std::{
    sync::{Arc, Mutex},
    thread::spawn,
};

use crate::{command::CommandQueue, robot::Robot, tui::GameTui, world::load_world};

mod command;
mod entity;
mod game;
mod robot;
mod server;
mod tui;
mod world;

fn main() {
    tui_logger::init_logger(log::LevelFilter::Trace).unwrap();
    tui_logger::set_default_level(log::LevelFilter::Trace);

    let world = Arc::new(Mutex::new(load_world()));
    let queue = CommandQueue::new();

    // Start game loop in background
    {
        let world = world.clone();
        let queue = queue.clone();
        spawn(|| game::gameloop(world, queue));
    }

    // Start robot in background
    {
        let queue = queue.clone();
        spawn(|| {
            Robot::new(queue).run();
        });
    }

    // Run TUI
    let mut tui = GameTui::new(world);
    if let Err(err) = tui.run() {
        eprintln!("TUI error: {}", err);
    }
}
