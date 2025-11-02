use std::{
    io,
    sync::{Arc, Mutex},
};

use log::Level;
use losig_client::{game::GameSim, tui::GameTui};
use losig_core::{
    network::{UdpCommandPacket, UdpSensesPacket},
    types::AvatarId,
};
use web_sys::{UrlSearchParams, window};

use crate::{ratzilla_adapter::RatzillaAdapter, ws::WsServer};

mod ratzilla_adapter;
mod ws;

pub type SenseMessage = UdpSensesPacket;
pub type CommandMessage = UdpCommandPacket;

fn main() -> io::Result<()> {
    console_log::init_with_level(Level::Debug).unwrap();

    let id = get_avatar_id().unwrap_or(1);

    let game = GameSim::new(id);
    let game = Arc::new(Mutex::new(game));

    let mut server = WsServer::new();
    {
        let game = game.clone();
        server.set_callback(Box::new(move |msg| {
            game.lock().unwrap().update(msg.senses);
        }));
    }

    server.init();
    let server = Arc::new(Mutex::new(server));
    {
        let mut game = game.lock().unwrap();
        game.set_callback(Box::new(move |cmd| {
            server
                .lock()
                .unwrap()
                .send(cmd)
                .expect("Cannot send message");
        }));
    }

    let tui = GameTui::new(game);
    let adapter = RatzillaAdapter::new(tui);
    adapter.run()?;
    Ok(())
}

fn get_avatar_id() -> Option<AvatarId> {
    let window = window()?;
    let location = window.location();
    let params = location.search().ok()?;
    let params = UrlSearchParams::new_with_str(&params).ok()?;

    params.get("id").and_then(|s| s.parse::<AvatarId>().ok())
}
