//! Adapter to make the client cross platform between terminal and web

use std::sync::{Arc, Mutex};

use log::debug;
use losig_core::{
    leaderboard::Leaderboard,
    network::{ClientMessage, ClientMessageContent, ServerMessage},
    types::AvatarId,
};

use crate::{tui::GameTui, tui_adapter::TuiApp, world::WorldView};

pub struct Adapter<C, T> {
    pub avatar_id: AvatarId,
    pub client: C,
    pub tui_adapter: T,
}

impl<C: Client + Send + 'static, T: TuiAdapter> Adapter<C, T> {
    pub fn run(mut self) {
        let world = WorldView::new(self.avatar_id);
        let world = Arc::new(Mutex::new(world));
        let leaderboard = Arc::new(Mutex::new(Leaderboard::new()));

        // Set up server message callback
        let callback: ServerMessageCallback;
        {
            let world = world.clone();
            let leaderboard = leaderboard.clone();
            callback = Box::new(move |msg: ServerMessage| match msg {
                ServerMessage::Turn(tr) => {
                    world.lock().unwrap().update(tr);
                }
                ServerMessage::Leaderboard(lb) => {
                    *leaderboard.lock().unwrap() = lb;
                }
            });
        }
        self.client.set_callback(callback);

        let client_share = Arc::new(Mutex::new(self.client));

        if let Ok(ref mut client) = client_share.lock() {
            let client_connect = client_share.clone();
            client.set_on_connect(Box::new(move || {
                let client = client_connect.lock().unwrap();
                debug!("sending req");
                client.send(ClientMessage {
                    avatar_id: Some(self.avatar_id),
                    content: ClientMessageContent::Leaderboard,
                });
            }));
            client.run();
        }

        let game_tui = GameTui::new(client_share, world, leaderboard);
        self.tui_adapter.run(game_tui);
    }
}

pub type ServerMessageCallback = Box<dyn Fn(ServerMessage) + Send>;
pub type ConnectCallback = Box<dyn Fn() + Send>;

pub trait Client {
    fn run(&mut self);
    fn set_callback(&mut self, callback: ServerMessageCallback);
    fn set_on_connect(&mut self, callback: ConnectCallback);
    fn send(&self, message: ClientMessage);
}

pub trait TuiAdapter {
    fn run<T: TuiApp + 'static>(self, tui: T);
}
