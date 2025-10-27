use std::{
    net::UdpSocket,
    sync::mpsc::{Receiver, Sender},
    thread::spawn,
};

use log::info;
use lost_signal::common::{
    action::Action,
    network::{UdpCommandPacket, UdpSensesPacket},
    sense::{SenseInfo, Senses},
    types::EntityId,
};

const SERVER_ADDR: &str = "127.0.0.1:8080";

pub struct NetworkClient {
    socket: UdpSocket,
    commands: Receiver<CommandMessage>,
    senses: Sender<SenseMessage>,
}

pub struct CommandMessage {
    pub entity_id: EntityId,
    pub action: Action,
    pub senses: Senses,
}

pub struct SenseMessage {
    pub entity_id: EntityId,
    pub info: SenseInfo,
}

impl NetworkClient {
    pub fn new(
        commands: Receiver<CommandMessage>,
        senses: Sender<SenseMessage>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.connect(SERVER_ADDR)?;

        Ok(Self {
            socket,
            commands,
            senses,
        })
    }

    pub fn run(self) {
        let Self {
            socket,
            commands,
            senses,
        } = self;

        let recv_socket = socket.try_clone().unwrap();

        spawn(move || {
            let mut buffer = [0; 1024];
            info!("client is running on {}", recv_socket.local_addr().unwrap());
            loop {
                let (size, _) = recv_socket.recv_from(&mut buffer).unwrap();
                let info: UdpSensesPacket = bincode::deserialize(&buffer[..size]).unwrap();
                senses.send(SenseMessage {
                    entity_id: info.entity_id,
                    info: info.senses,
                });
            }
        });

        spawn(move || {
            loop {
                if let Ok(msg) = commands.recv() {
                    let udppacket = UdpCommandPacket {
                        entity_id: msg.entity_id,
                        senses: msg.senses,
                        tick: None,
                        action: msg.action,
                    };

                    let binary_data = match bincode::serialize(&udppacket) {
                        Ok(data) => data,
                        Err(e) => {
                            eprintln!("Failed to serialize senses packet: {}", e);
                            continue;
                        }
                    };
                    let _ = socket.send_to(&binary_data, SERVER_ADDR);
                }
            }
        });
    }
}
