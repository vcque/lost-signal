use std::sync::mpsc::Sender;

use lost_signal::common::network::UdpCommandPacket;

pub type CommandMessage = UdpCommandPacket;

pub type CommandQueue = Sender<CommandMessage>;
