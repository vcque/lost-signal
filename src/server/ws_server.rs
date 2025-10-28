use std::{
    collections::HashMap,
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{Arc, mpsc::Receiver},
    thread::{sleep, spawn},
    time::Duration,
};

use anyhow::{Result, bail};
use log::error;
use lost_signal::common::{
    network::{UdpCommandPacket, UdpSensesPacket},
    types::EntityId,
};
use tungstenite::{Bytes, Message, WebSocket};

use crate::{sense::SensesMessage, states::States};

type Ws = WebSocket<TcpStream>;

pub struct WsServer {
    states: Arc<States>,
    senses: Receiver<SensesMessage>,
}

#[derive(Default)]
struct Registry {
    streams: HashMap<SocketAddr, Ws>,
    addr_by_entity_id: HashMap<EntityId, SocketAddr>,
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
        let mut addr_by_entity_id = HashMap::<EntityId, SocketAddr>::new();

        loop {
            if let Ok((stream, addr)) = handle_incoming(&server) {
                ws_by_addr.insert(addr, stream);
            }

            for (addr, stream) in ws_by_addr.iter_mut() {
                if let Ok(cmd) = handle_read(stream) {
                    addr_by_entity_id.insert(cmd.entity_id, *addr);
                    states.commands.send(cmd)?;
                }
            }

            for sense in senses.try_iter() {
                let entity_id = sense.entity_id;
                let ws = addr_by_entity_id
                    .get(&entity_id)
                    .and_then(|addr| ws_by_addr.get_mut(addr));

                if let Some(ws) = ws {
                    let msg = UdpSensesPacket {
                        entity_id: entity_id,
                        senses: sense.senses,
                    };
                    let _ = handle_write(ws, msg);
                }
            }
            ws_by_addr.retain(|_, v| v.can_read());

            sleep(Duration::from_millis(20));
        }
    }
}

fn handle_incoming(server: &TcpListener) -> Result<(Ws, SocketAddr)> {
    let (stream, addr) = server.accept()?;
    let stream = tungstenite::accept(stream)?;

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
    ws.write(Message::Binary(msg))?;
    Ok(())
}
