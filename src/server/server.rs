use std::{net::UdpSocket, sync::Arc, thread::spawn};

use log::info;
use lost_signal::common::network::UdpPacket;

use crate::{command::CommandMessage, states::States};

pub struct Server {
    states: Arc<States>,
}

impl Server {
    pub fn new(states: Arc<States>) -> Server {
        Server {
            states: states.clone(),
        }
    }

    pub fn run(self) {
        spawn(move || {
            loop {
                let socket = UdpSocket::bind("127.0.0.1:8080").expect("Couldn't bind to port");
                let mut buf = [0; 1024];

                let (size, _) = socket.recv_from(&mut buf).unwrap();
                let data = str::from_utf8(&buf[..size]).unwrap();
                let cmd: UdpPacket = serde_json::from_str(data).unwrap();

                info!("Receiving {:?}", cmd);
                let tick: u64 = cmd
                    .tick
                    .unwrap_or_else(|| self.states.world.lock().unwrap().tick + 1);

                let msg = CommandMessage {
                    entity_id: cmd.entity_id,
                    tick,
                    content: cmd.command,
                    senses: cmd.senses,
                };

                info!("Sending {:?}", msg);

                self.states.command_queue.send_command(msg);
            }
        });
    }
}
