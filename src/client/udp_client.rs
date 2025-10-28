use std::{
    net::UdpSocket,
    sync::mpsc::{Receiver, Sender},
    thread::{sleep, spawn},
    time::Duration,
};

use anyhow::Result;
use log::error;
use lost_signal::common::network::UdpSensesPacket;
use serde::{Deserialize, Serialize};

use crate::{CommandMessage, SenseMessage};

const SERVER_ADDR: &str = "127.0.0.1:8080";

pub struct UdpClient {
    commands: Receiver<CommandMessage>,
    senses: Sender<SenseMessage>,
}

impl UdpClient {
    pub fn new(commands: Receiver<CommandMessage>, senses: Sender<SenseMessage>) -> Self {
        UdpClient { commands, senses }
    }
    pub fn run(self) {
        spawn(move || {
            let e = self.do_run().unwrap_err();
            error!("{e}");
        });
    }

    fn do_run(self) -> Result<()> {
        let Self { commands, senses } = self;

        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.set_nonblocking(true);
        socket.connect(SERVER_ADDR);

        let buf = &mut [0; 1024];
        loop {
            if let Ok(sense) = handle_read::<UdpSensesPacket>(&socket, buf) {
                let _ = senses.send(sense);
            }

            for cmd in commands.try_iter() {
                let _ = handle_write(&socket, cmd);
            }

            sleep(Duration::from_millis(20));
        }
    }
}

fn handle_read<'a, T: Deserialize<'a>>(socket: &UdpSocket, buf: &'a mut [u8]) -> Result<T> {
    let (size, _) = socket.recv_from(buf)?;
    let command = bincode::deserialize::<T>(&buf[..size])?;
    Ok(command)
}

fn handle_write<T: Serialize>(socket: &UdpSocket, msg: T) -> Result<()> {
    let msg = bincode::serialize(&msg)?;
    socket.send(&msg)?;
    Ok(())
}
