//! Adapter to make the client cross platform between terminal and web

use std::sync::{Arc, Mutex};

use log::debug;
use losig_core::{
    leaderboard::Leaderboard,
    network::{ClientMessage, ClientMessageContent, ServerMessage},
    types::{GameOver, PlayerId},
};

use crate::{tui::GameTui, world::WorldView};

pub struct Adapter<C, T> {
    pub player_id: PlayerId,
    pub client: C,
    pub tui_adapter: T,
}

impl<C: Client, T: TuiAdapter> Adapter<C, T> {
    pub fn run(mut self) {
        let shared_state = Arc::new(Mutex::new(SharedState::new(self.player_id)));

        // Set up server message callback
        let callback: ServerMessageCallback;
        {
            let state = shared_state.clone();
            callback = Box::new(move |msg: ServerMessage| {
                let mut state = state.lock().unwrap();
                debug!("msg received: {msg:?}");
                match msg {
                    ServerMessage::Turn(tr) => {
                        state.world.update(tr);
                    }
                    ServerMessage::Leaderboard(lb) => {
                        state.leaderboard = lb;
                    }
                    ServerMessage::GameOver(gom) => {
                        debug!("Gameover received {:?}", gom);
                        state.gameover = Some(gom);
                    }
                    ServerMessage::Limbo {
                        averted,
                        senses_info,
                    } => {
                        state.limbo = Some(averted);
                        if averted && let Some(info) = senses_info {
                            state.world.update_on_averted(info);
                        }
                    }
                    ServerMessage::Timeline(stage_id, stage_turn, timeline, senses_info) => {
                        state.world.update_timeline(stage_id, timeline);
                        if let Some(info) = senses_info {
                            state.world.update_on_timeline(stage_turn, info);
                        }
                    }
                    ServerMessage::Transition(transition_message) => {
                        state.world.transition(transition_message);
                    }
                }
            });
        }
        self.client.set_callback(callback);

        let shared_client = Arc::new(Mutex::new(self.client));

        if let Ok(ref mut client) = shared_client.lock() {
            let client_connect = shared_client.clone();
            client.set_on_connect(Box::new(move || {
                let client = client_connect.lock().unwrap();
                client.send(ClientMessage {
                    player_id: Some(self.player_id),
                    content: ClientMessageContent::Leaderboard,
                });
            }));
            client.run();
        }

        let game_tui = GameTui::new(shared_client, shared_state);
        self.tui_adapter.run(game_tui);
    }
}

pub type ServerMessageCallback = Box<dyn Fn(ServerMessage) + Send>;
pub type ConnectCallback = Box<dyn Fn() + Send>;

pub trait Client: Send + 'static {
    fn run(&mut self);
    fn set_callback(&mut self, callback: ServerMessageCallback);
    fn set_on_connect(&mut self, callback: ConnectCallback);
    fn send(&self, message: ClientMessage);
}

pub trait TuiAdapter {
    fn run(self, tui: GameTui);
}

/// State manipulated by either the tui or incoming messages
pub struct SharedState {
    pub player_id: PlayerId,
    pub gameover: Option<GameOver>,
    pub limbo: Option<bool>,
    pub leaderboard: Leaderboard,
    pub world: WorldView,
}

impl SharedState {
    pub fn new(player_id: PlayerId) -> Self {
        Self {
            player_id,
            gameover: None,
            limbo: None,
            leaderboard: Leaderboard::new(),
            world: WorldView::new(),
        }
    }
}
