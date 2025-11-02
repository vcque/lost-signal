#![allow(dead_code)]

use std::{
    collections::HashMap,
    net::{SocketAddr, UdpSocket},
    sync::{Arc, mpsc::Receiver},
    thread::{sleep, spawn},
    time::Duration,
};

use anyhow::Result;
use log::error;
use losig_core::{
    network::{UdpCommandPacket, UdpSensesPacket},
    types::AvatarId,
};
use serde::Serialize;

use crate::{sense::SensesMessage, states::States};

pub struct UdpServer {
    states: Arc<States>,
    senses: Receiver<SensesMessage>,
}

impl UdpServer {
    pub fn new(states: Arc<States>, senses: Receiver<SensesMessage>) -> UdpServer {
        UdpServer {
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
        let socket = UdpSocket::bind("127.0.0.1:8080")?;
        socket.set_nonblocking(true)?;

        let mut addr_by_avatar_id = HashMap::<AvatarId, SocketAddr>::new();

        let buf = &mut [0; 1024];
        loop {
            if let Ok((addr, cmd)) = handle_read(&socket, buf) {
                let avatar_id = cmd.avatar_id;
                addr_by_avatar_id.insert(avatar_id, addr);
                let _ = states.commands.send(cmd);
            }

            for sense in senses.try_iter() {
                let SensesMessage { avatar_id, senses } = sense;
                if let Some(addr) = addr_by_avatar_id.get(&sense.avatar_id) {
                    let _ = handle_write(&socket, addr, UdpSensesPacket { avatar_id, senses });
                }
            }

            sleep(Duration::from_millis(20));
        }
    }
}

fn handle_read(socket: &UdpSocket, buf: &mut [u8]) -> Result<(SocketAddr, UdpCommandPacket)> {
    let (size, addr) = socket.recv_from(buf)?;
    let command = bincode::deserialize(&buf[..size])?;
    Ok((addr, command))
}

fn handle_write<T: Serialize>(socket: &UdpSocket, addr: &SocketAddr, msg: T) -> Result<()> {
    let msg = bincode::serialize(&msg)?;
    socket.send_to(&msg, addr)?;
    Ok(())
}
