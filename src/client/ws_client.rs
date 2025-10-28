use std::{
    net::TcpStream,
    sync::mpsc::{Receiver, Sender},
    thread::{sleep, spawn},
    time::Duration,
};

use anyhow::{Result, bail};
use log::error;
use lost_signal::common::network::UdpSensesPacket;
use serde::{Deserialize, Serialize};
use tungstenite::{
    Bytes, ClientHandshake, HandshakeError, Message, WebSocket,
    http::Request,
};

use crate::{CommandMessage, SenseMessage};

const SERVER_ADDR: &str = "127.0.0.1:9001";

pub struct WsClient {
    commands: Receiver<CommandMessage>,
    senses: Sender<SenseMessage>,
}

type Ws = WebSocket<TcpStream>;

impl WsClient {
    pub fn new(commands: Receiver<CommandMessage>, senses: Sender<SenseMessage>) -> Self {
        Self { commands, senses }
    }
    pub fn run(self) {
        spawn(move || {
            let e = self.do_run().unwrap_err();
            error!("{e}");
        });
    }

    fn do_run(self) -> Result<()> {
        let Self { commands, senses } = self;

        let mut socket: Option<Ws> = None;

        loop {
            let Some(ref mut socket) = socket else {
                match connect() {
                    Ok(s) => socket = Some(s),
                    Err(e) => error!("{e}"),
                }
                continue;
            };

            if let Ok(sense) = handle_read::<UdpSensesPacket>(socket) {
                let _ = senses.send(sense);
            }

            for cmd in commands.try_iter() {
                let _ = handle_write(socket, cmd);
            }

            sleep(Duration::from_millis(20));
        }
    }
}

fn connect() -> Result<Ws> {
    let stream = TcpStream::connect(SERVER_ADDR)?;
    stream.set_nonblocking(true)?;

    let address = format!("ws://{SERVER_ADDR}/");

    // Create HTTP request for WebSocket handshake
    let request = Request::builder()
        .method("GET")
        .uri(address)
        .header("Host", SERVER_ADDR)
        .header("Upgrade", "websocket")
        .header(
            "Sec-WebSocket-Key",
            tungstenite::handshake::client::generate_key(),
        )
        .header("Sec-WebSocket-Version", "13")
        .header("Connection", "Upgrade")
        .body(())?;

    let mut handshake = ClientHandshake::start(stream, request, None)?;

    loop {
        match handshake.handshake() {
            Ok((socket, _)) => return Ok(socket), // Success
            Err(HandshakeError::Interrupted(mid)) => {
                handshake = mid; // Continue handshake
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(HandshakeError::Failure(e)) => return bail!(e),
        }
    }
}

fn handle_read<T: for<'a> Deserialize<'a>>(socket: &mut Ws) -> Result<T> {
    let msg = socket.read()?;

    let Message::Binary(msg) = msg else {
        bail!("Not a binary");
    };

    let msg = bincode::deserialize::<T>(&msg)?;
    Ok(msg)
}

fn handle_write<T: Serialize>(ws: &mut Ws, msg: T) -> Result<()> {
    let msg = bincode::serialize(&msg)?;
    let msg = Bytes::from_owner(msg);
    ws.send(Message::Binary(msg))?;
    Ok(())
}
