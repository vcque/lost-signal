use std::{
    collections::HashMap,
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{Arc, mpsc::Receiver},
    thread::{sleep, spawn},
    time::Duration,
};

use anyhow::{Result, bail};
use log::{error, info, warn};
use losig_core::{
    network::{UdpCommandPacket, UdpSensesPacket},
    types::AvatarId,
};
use tungstenite::{Bytes, Message, WebSocket};

use crate::{sense::SensesMessage, states::States};

type Ws = WebSocket<TcpStream>;

pub struct WsServer {
    states: Arc<States>,
    senses: Receiver<SensesMessage>,
}

impl WsServer {
    pub fn new(states: Arc<States>, senses: Receiver<SensesMessage>) -> WsServer {
        WsServer {
            states: states.clone(),
            senses,
        }
    }

    pub fn run(self) {
        spawn(move || {
            let e = self.do_run().unwrap_err();
            error!("{e}");
        });
    }

    fn do_run(self) -> Result<()> {
        let Self { states, senses } = self;

        let server = TcpListener::bind("127.0.0.1:9001")?;
        server.set_nonblocking(true)?;

        let mut ws_by_addr = HashMap::<SocketAddr, Ws>::new();
        let mut addr_by_avatar_id = HashMap::<AvatarId, SocketAddr>::new();

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
                    Ok(cmd) => {
                        addr_by_avatar_id.insert(cmd.avatar_id, *addr);
                        states.commands.send(cmd)?;
                    }
                    Err(e) => {
                        if !is_would_block(&e) {
                            warn!("Couldn't read: {e}");
                        }
                    }
                }
            }

            for sense in senses.try_iter() {
                let avatar_id = sense.avatar_id;
                let ws = addr_by_avatar_id
                    .get(&avatar_id)
                    .and_then(|addr| ws_by_addr.get_mut(addr));

                if let Some(ws) = ws {
                    let msg = UdpSensesPacket {
                        avatar_id,
                        senses: sense.senses,
                    };
                    let _ = handle_write(ws, msg);
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

fn handle_read(stream: &mut Ws) -> Result<UdpCommandPacket> {
    let msg = stream.read()?;
    let Message::Binary(msg) = msg else {
        bail!("Not a binary");
    };

    let command = bincode::deserialize::<UdpCommandPacket>(&msg)?;
    Ok(command)
}

fn handle_write(ws: &mut Ws, msg: UdpSensesPacket) -> Result<()> {
    let msg = bincode::serialize(&msg)?;
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
