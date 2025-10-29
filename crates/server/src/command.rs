use std::sync::mpsc::Sender;

use losig_core::network::UdpCommandPacket;

pub type CommandMessage = UdpCommandPacket;

pub type CommandQueue = Sender<CommandMessage>;
