use std::{
    net::TcpStream,
    sync::{
        Arc, Mutex,
        mpsc::{Sender, channel},
    },
    thread::{sleep, spawn},
    time::Duration,
};

use anyhow::{Result, bail};
use log::{error, warn};
use losig_client::adapter::{Client, ConnectCallback, ServerMessageCallback};
use losig_core::network::{ClientMessage, ServerMessage};
use serde::{Deserialize, Serialize};
use tungstenite::{Bytes, ClientHandshake, HandshakeError, Message, WebSocket, http::Request};

const SERVER_ADDR: &str = "127.0.0.1:9001";

type Ws = WebSocket<TcpStream>;

pub struct WsClient {
    callback: Arc<Mutex<ServerMessageCallback>>,
    on_connect: Arc<Mutex<ConnectCallback>>,
    sender: Option<Sender<ClientMessage>>,
}

impl WsClient {
    pub fn new() -> Self {
        Self {
            callback: Arc::new(Mutex::new(Box::new(|_| {}))),
            on_connect: Arc::new(Mutex::new(Box::new(|| {}))),
            sender: None,
        }
    }
}

impl Client for WsClient {
    fn send(&self, message: losig_core::network::ClientMessage) {
        match self.sender {
            Some(ref sender) => {
                if let Err(e) = sender.send(message) {
                    error!("Error while sending client message: {e}");
                }
            }
            None => {
                warn!("Ws server not initialized!");
            }
        };
    }

    fn set_callback(&mut self, callback: ServerMessageCallback) {
        *self.callback.lock().unwrap() = callback;
    }

    fn set_on_connect(&mut self, callback: ConnectCallback) {
        *self.on_connect.lock().unwrap() = callback;
    }

    fn run(&mut self) {
        // 1. creates the necessary channels
        let (s_tx, s_rx) = channel::<ServerMessage>();
        let (c_tx, c_rx) = channel::<ClientMessage>();

        self.sender = Some(c_tx.clone());

        // 2. 1 thread for handling the WS
        let on_connect = self.on_connect.clone();
        spawn(move || {
            let mut socket: Option<Ws> = None;
            let mut connected = false;
            loop {
                let Some(ref mut socket) = socket else {
                    match connect() {
                        Ok(s) => {
                            socket = Some(s);
                            connected = false; // Reset connected flag for new socket
                        }
                        Err(e) => error!("{e}"),
                    }
                    continue;
                };

                // Call on_connect callback when first connected
                if !connected {
                    (on_connect.lock().unwrap())();
                    connected = true;
                }

                if let Ok(server_message) = handle_read::<ServerMessage>(socket) {
                    let _ = s_tx.send(server_message);
                }

                for client_message in c_rx.try_iter() {
                    let _ = handle_write(socket, client_message);
                }

                sleep(Duration::from_millis(20));
            }
        });
        // 3. 2nd thread for reading the server receiver and calling the callback
        let callback = self.callback.clone();
        spawn(move || {
            while let Ok(msg) = s_rx.recv() {
                (callback.lock().unwrap())(msg);
            }
        });
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
            Err(HandshakeError::Failure(e)) => bail!(e),
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
