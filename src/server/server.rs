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
                    let data = str::from_utf8(&buf[..size]).unwrap();
                    let cmd: UdpCommandPacket = serde_json::from_str(data).unwrap();

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

                    let json = serde_json::to_string(&udppacket).unwrap();
                    socket.send_to(json.as_bytes(), msg.address);
                }
            }
        });
    }
}
