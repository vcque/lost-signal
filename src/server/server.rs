use std::{
    net::UdpSocket,
    sync::{Arc, mpsc::Receiver},
    thread::spawn,
};

use log::info;
use lost_signal::common::network::{UdpCommandPacket, UdpSensesPacket};

use crate::{command::CommandMessage, sense::SensesMessage, states::States};

pub struct Server {
    states: Arc<States>,
    senses: Receiver<SensesMessage>,
}

impl Server {
    pub fn new(states: Arc<States>, senses: Receiver<SensesMessage>) -> Server {
        Server {
            states: states.clone(),
            senses,
        }
    }

    pub fn run(self) {
        let Self { states, senses } = self;

        let socket = UdpSocket::bind("127.0.0.1:8080").expect("Couldn't bind to port");

        {
            let states = states.clone();
            let socket = socket.try_clone().unwrap();
            spawn(move || {
                loop {
                    let mut buf = [0; 1024];

                    let (size, address) = socket.recv_from(&mut buf).unwrap();
                    let cmd: UdpCommandPacket = match bincode::deserialize(&buf[..size]) {
                        Ok(packet) => packet,
                        Err(e) => {
                            eprintln!("Failed to deserialize command packet: {}", e);
                            continue;
                        }
                    };

                    info!("Receiving {:?}", cmd);
                    let tick: u64 = cmd
                        .tick
                        .unwrap_or_else(|| states.world.lock().unwrap().tick + 1);

                    let msg = CommandMessage {
                        entity_id: cmd.entity_id,
                        tick,
                        action: cmd.action,
                        senses: cmd.senses,
                        address: Some(address),
                    };

                    info!("Sending {:?}", msg);

                    states.commands.send_command(msg);
                }
            });
        }

        spawn(move || {
            loop {
                if let Ok(msg) = senses.recv() {
                    let udppacket = UdpSensesPacket {
                        entity_id: msg.entity_id,
                        senses: msg.senses,
                    };

                    let binary_data = match bincode::serialize(&udppacket) {
                        Ok(data) => data,
                        Err(e) => {
                            eprintln!("Failed to serialize senses packet: {}", e);
                            continue;
                        }
                    };
                    let _ = socket.send_to(&binary_data, msg.address);
                }
            }
        });
    }
}
