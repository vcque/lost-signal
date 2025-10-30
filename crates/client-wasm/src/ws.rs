use std::{cell::RefCell, rc::Rc};

use anyhow::{Result, bail};
use js_sys::ArrayBuffer;
use log::{debug, error};
use losig_client::game::{CommandMessage, SenseMessage};
use losig_core::network::UdpSensesPacket;
use wasm_bindgen::{JsCast, JsValue, prelude::Closure};
use web_sys::{BinaryType, MessageEvent, WebSocket};

pub struct WsServer {
    on_recv: Rc<RefCell<Box<dyn Fn(SenseMessage) + Send>>>,
    socket: RefCell<Option<WebSocket>>,
}

/// It's okay, we're targeting wasm
unsafe impl Send for WsServer {}

impl WsServer {
    pub fn new() -> Self {
        WsServer {
            on_recv: Rc::new(RefCell::new(Box::new(|_| {}))),
            socket: RefCell::new(None),
        }
    }

    pub fn set_callback(&mut self, callback: Box<dyn Fn(SenseMessage) + Send>) {
        *self.on_recv.borrow_mut() = callback;
    }

    pub fn init(&mut self) -> Result<()> {
        let socket = match WebSocket::new("/ws") {
            Ok(ws) => ws,
            Err(e) => bail!("Couldn't start ws: {e:?}"),
        };
        socket.set_binary_type(BinaryType::Arraybuffer);

        let on_open = Closure::wrap(Box::new(move |e| {
            debug!("Open: {e:?}");
        }) as Box<dyn Fn(JsValue)>);
        socket.set_onopen(Some(on_open.as_ref().unchecked_ref()));
        on_open.forget();

        let on_recv = self.on_recv.clone();
        let onmessage_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
            debug!("receiving msg: {e:?}");
            let senses = convert_response(e);
            if let Some(senses) = senses {
                (on_recv.borrow())(senses);
            }
        }) as Box<dyn FnMut(MessageEvent)>);
        socket.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget();

        let on_error = Closure::wrap(Box::new(move |e| {
            debug!("Error: {e:?}");
        }) as Box<dyn Fn(JsValue)>);
        socket.set_onerror(Some(on_error.as_ref().unchecked_ref()));
        on_error.forget();

        *self.socket.borrow_mut() = Some(socket);

        debug!("ws initialized!");
        Ok(())
    }

    pub fn send(&mut self, msg: CommandMessage) -> Result<()> {
        let data = bincode::serialize(&msg)?;
        let socket = self.socket.borrow();
        let socket = socket.as_ref().unwrap();
        match socket.send_with_u8_array(&data) {
            Ok(_) => Ok(()),
            Err(e) => bail!("{e:?}"),
        }
    }
}

fn convert_response(me: MessageEvent) -> Option<UdpSensesPacket> {
    let Ok(array) = me.data().dyn_into::<ArrayBuffer>() else {
        return None;
    };
    let uint8_array = js_sys::Uint8Array::new(&array);
    let bytes: Vec<u8> = uint8_array.to_vec();
    let slice: &[u8] = &bytes;

    match bincode::deserialize::<UdpSensesPacket>(slice) {
        Ok(msg) => Some(msg),
        Err(e) => {
            error!("Couldn't deser: {e:?}");
            None
        }
    }
}
