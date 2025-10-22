use std::{
    sync::{Arc, Mutex},
    thread::spawn,
};

use crate::{command::CommandQueue, world::load_world};

mod command;
mod entity;
mod game;
mod server;
mod world;

fn main() {
    let world = Arc::new(Mutex::new(load_world()));
    let queue = CommandQueue::new();

    let handle = spawn(|| game::gameloop(world, queue));

    let _ = handle.join();
}
