use std::{
    io,
    sync::{Arc, Mutex},
};

use crate::{ratzilla_adapter::RatzillaAdapter, ws::WsServer};
use log::Level;
use losig_client::{game::GameSim, tui::GameTui};
use losig_core::{
    network::{UdpCommandPacket, UdpSensesPacket},
    types::AvatarId,
};
use wasm_bindgen::JsValue;
use web_sys::{Url, UrlSearchParams, window};

mod ratzilla_adapter;
mod ws;

pub type SenseMessage = UdpSensesPacket;
pub type CommandMessage = UdpCommandPacket;

fn main() -> io::Result<()> {
    console_log::init_with_level(Level::Debug).unwrap();

    let id = get_avatar_id().unwrap_or_else(generate_avatar_id);
    update_history(id);

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

fn generate_avatar_id() -> AvatarId {
    let crypto = window().unwrap().crypto().unwrap();
    let mut array = [0u8; 4];
    crypto.get_random_values_with_u8_array(&mut array).unwrap();
    u32::from_le_bytes(array)
}

/// Change the id in the url so that the player can bookmark its player (will need proper user
/// manager on day)
fn update_history(id: AvatarId) {
    let window = window().unwrap();
    let location = window.location();

    let current_url = location.href().unwrap();

    let url = Url::new(&current_url).unwrap();
    let search_params = url.search_params();
    search_params.set("id", &format!("{id}"));

    url.set_search(&search_params.to_string().as_string().unwrap());

    let history = window.history().unwrap();
    history
        .push_state_with_url(&JsValue::NULL, "", Some(&url.href()))
        .unwrap();
}
