//! Adapter to make the client cross platform between terminal and web

use std::sync::{Arc, Mutex};

use losig_core::{
    leaderboard::Leaderboard,
    network::{ClientMessage, ServerMessage},
    types::AvatarId,
};

use crate::{tui::GameTui, tui_adapter::TuiApp, world::WorldView};

pub struct Adapter<C, T> {
    pub avatar_id: AvatarId,
    pub client: C,
    pub tui_adapter: T,
}

impl<C: Client + 'static, T: TuiAdapter> Adapter<C, T> {
    pub fn run(mut self) {
        self.client.run();

        let world = WorldView::new(self.avatar_id);
        let world = Arc::new(Mutex::new(world));
        let leaderboard = Arc::new(Mutex::new(Leaderboard::new()));

        let callback: ServerMessageCallback;
        {
            let world = world.clone();
            let leaderboard = leaderboard.clone();
            callback = Box::new(move |msg: ServerMessage| match msg {
                ServerMessage::Senses(s) => {
                    world.lock().unwrap().update(s.turn, s.senses);
                }
                ServerMessage::Leaderboard(lb) => {
                    *leaderboard.lock().unwrap() = lb;
                }
            });
        }
        self.client.set_callback(callback);

        let game_tui = GameTui::new(Arc::new(self.client), world, leaderboard);
        self.tui_adapter.run(game_tui);
    }
}

pub type ServerMessageCallback = Box<dyn Fn(ServerMessage) + Send>;

pub trait Client {
    fn run(&mut self);
    fn set_callback(&mut self, callback: ServerMessageCallback);
    fn send(&self, message: ClientMessage);
}

pub trait TuiAdapter {
    fn run<T: TuiApp + 'static>(self, tui: T);
}
