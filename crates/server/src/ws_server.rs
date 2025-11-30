use std::{
    collections::HashMap,
    net::{SocketAddr, TcpListener, TcpStream},
    sync::mpsc::{Receiver, Sender, channel},
    thread::{sleep, spawn},
    time::Duration,
};

use anyhow::{Result, bail};
use log::{error, info, warn};
use losig_core::{
    network::{ClientMessage, ServerMessage},
    types::PlayerId,
};
use tungstenite::{Bytes, Message, WebSocket};

type Ws = WebSocket<TcpStream>;

pub enum Recipient {
    Broadcast,
    Single(PlayerId),
    Multi(Vec<PlayerId>),
}

/// Server message with recipient
pub struct ServerMessageWithRecipient {
    pub recipient: Recipient,
    pub message: ServerMessage,
}

pub struct WsServer {
    cm_tx: Sender<ClientMessage>,
    sm_rx: Receiver<ServerMessageWithRecipient>,
}

impl WsServer {
    pub fn new() -> (
        WsServer,
        Sender<ServerMessageWithRecipient>,
        Receiver<ClientMessage>,
    ) {
        let (cm_tx, cm_rx) = channel();
        let (sm_tx, sm_rx) = channel();
        (WsServer { cm_tx, sm_rx }, sm_tx, cm_rx)
    }

    pub fn run(self) {
        spawn(move || {
            let e = self.do_run().unwrap_err();
            error!("{e}");
        });
    }

    fn do_run(self) -> Result<()> {
        let Self { cm_tx, sm_rx } = self;

        let server = TcpListener::bind("127.0.0.1:9001")?;
        server.set_nonblocking(true)?;

        let mut ws_by_addr = HashMap::<SocketAddr, Ws>::new();
        let mut addr_by_avatar_id = HashMap::<PlayerId, SocketAddr>::new();

        info!("Launching server on 127.0.0.1:9001");

        loop {
            match handle_incoming(&server) {
                Ok((stream, addr)) => {
                    ws_by_addr.insert(addr, stream);
                }
                Err(e) => {
                    if !is_would_block(&e) {
                        warn!("Could not establish connection: {:?}", e);
                    }
                }
            }
            if let Ok((stream, addr)) = handle_incoming(&server) {
                ws_by_addr.insert(addr, stream);
            }

            for (addr, stream) in ws_by_addr.iter_mut() {
                match handle_read(stream) {
                    Ok(client_message) => {
                        if let Some(avatar_id) = client_message.avatar_id {
                            addr_by_avatar_id.insert(avatar_id, *addr);
                        }
                        cm_tx.send(client_message)?;
                    }
                    Err(e) => {
                        if !is_would_block(&e) {
                            warn!("Couldn't read: {e}");
                        }
                    }
                }
            }

            for server_message in sm_rx.try_iter() {
                match server_message.recipient {
                    Recipient::Single(id) => {
                        if let Some(addr) = addr_by_avatar_id.get(&id)
                            && let Some(ws) = ws_by_addr.get_mut(addr)
                        {
                            let _ = handle_write(ws, &server_message.message);
                        }
                    }
                    Recipient::Broadcast => {
                        for ws in ws_by_addr.values_mut() {
                            let _ = handle_write(ws, &server_message.message);
                        }
                    }
                    Recipient::Multi(aids) => {
                        for aid in aids {
                            if let Some(addr) = addr_by_avatar_id.get(&aid)
                                && let Some(ws) = ws_by_addr.get_mut(addr)
                            {
                                let _ = handle_write(ws, &server_message.message);
                            }
                        }
                    }
                }
            }
            ws_by_addr.retain(|_, v| v.can_read());

            sleep(Duration::from_millis(10));
        }
    }
}

fn handle_incoming(server: &TcpListener) -> Result<(Ws, SocketAddr)> {
    let (stream, addr) = server.accept()?;
    let mut stream = tungstenite::accept(stream)?;
    stream.get_mut().set_nonblocking(true)?;
    info!("Incoming connection from {addr}");
    Ok((stream, addr))
}

fn handle_read(stream: &mut Ws) -> Result<ClientMessage> {
    let msg = stream.read()?;
    let Message::Binary(msg) = msg else {
        bail!("Not a binary");
    };

    let command = bincode::deserialize::<ClientMessage>(&msg)?;
    Ok(command)
}

fn handle_write(ws: &mut Ws, msg: &ServerMessage) -> Result<()> {
    let msg = bincode::serialize(msg)?;
    let msg = Bytes::from_owner(msg);
    ws.send(Message::Binary(msg))?;
    Ok(())
}

fn is_would_block(error: &anyhow::Error) -> bool {
    error.chain().any(|cause| {
        cause
            .downcast_ref::<std::io::Error>()
            .map(|e| e.kind() == std::io::ErrorKind::WouldBlock)
            .unwrap_or(false)
    })
}
