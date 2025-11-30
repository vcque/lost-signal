use std::io;

use crate::{ratzilla_adapter::RatzillaAdapter, ws::WsClient};
use log::Level;
use losig_client::adapter::Adapter;
use losig_core::types::PlayerId;
use wasm_bindgen::JsValue;
use web_sys::{Url, UrlSearchParams, window};

mod ratzilla_adapter;
mod ws;

fn main() -> io::Result<()> {
    console_log::init_with_level(Level::Debug).unwrap();

    let player_id = get_player_id().unwrap_or_else(generate_player_id);
    update_history(player_id);

    let client = WsClient::new();
    let tui_adapter = RatzillaAdapter::new();
    Adapter {
        player_id,
        client,
        tui_adapter,
    }
    .run();
    Ok(())
}

fn get_player_id() -> Option<PlayerId> {
    let window = window()?;
    let location = window.location();
    let params = location.search().ok()?;
    let params = UrlSearchParams::new_with_str(&params).ok()?;

    params.get("id").and_then(|s| s.parse::<PlayerId>().ok())
}

fn generate_player_id() -> PlayerId {
    let crypto = window().unwrap().crypto().unwrap();
    let mut array = [0u8; 4];
    crypto.get_random_values_with_u8_array(&mut array).unwrap();
    u32::from_le_bytes(array)
}

/// Change the id in the url so that the player can bookmark its player (will need proper user
/// manager on day)
fn update_history(id: PlayerId) {
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
