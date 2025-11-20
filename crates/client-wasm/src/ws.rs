use std::{cell::RefCell, rc::Rc};

use anyhow::{Result, bail};
use gloo_timers::callback::Interval;
use js_sys::ArrayBuffer;
use log::{debug, error};
use losig_client::adapter::{Client, ConnectCallback, ServerMessageCallback};
use losig_core::network::{ClientMessage, ServerMessage};
use wasm_bindgen::{JsCast, JsValue, prelude::Closure};
use web_sys::{BinaryType, MessageEvent, WebSocket, console};

#[derive(Clone)]
pub struct WsClient {
    on_recv: Rc<RefCell<ServerMessageCallback>>,
    on_connect: Rc<RefCell<ConnectCallback>>,
    socket: Rc<RefCell<Option<WebSocket>>>,
    timer: Rc<RefCell<Option<Interval>>>,
}

/// It's okay, we're targeting wasm
unsafe impl Send for WsClient {}

impl WsClient {
    pub fn new() -> Self {
        WsClient {
            on_recv: Rc::new(RefCell::new(Box::new(|_| {}))),
            on_connect: Rc::new(RefCell::new(Box::new(|| {}))),
            socket: Rc::new(RefCell::new(None)),
            timer: Rc::new(RefCell::new(None)),
        }
    }

    pub fn init(&self) {
        let _ = self.connect();
        let ws = self.clone();
        let timer = Interval::new(5000, move || {
            if ws.socket.borrow().is_none() {
                let _ = ws.connect();
            }
        });
        *self.timer.borrow_mut() = Some(timer);
    }

    pub fn connect(&self) -> Result<()> {
        let socket = match WebSocket::new("/ws") {
            Ok(ws) => ws,
            Err(e) => bail!("Couldn't start ws: {e:?}"),
        };
        socket.set_binary_type(BinaryType::Arraybuffer);
        let socket_ref = Rc::new(RefCell::new(Some(socket)));
        let mut socket = socket_ref.borrow_mut();
        let socket = socket.as_mut().unwrap();

        let s_ref = socket_ref.clone();
        let ws = self.clone();
        let on_open = Closure::wrap(Box::new(move |e| {
            debug!("Ws open");
            console::log_1(&e);
            let socket = s_ref.borrow_mut().take();
            match socket {
                Some(socket) => *ws.socket.borrow_mut() = Some(socket),
                None => debug!("There is no socket!"),
            }
            (ws.on_connect.borrow())();
        }) as Box<dyn Fn(JsValue)>);
        socket.set_onopen(Some(on_open.as_ref().unchecked_ref()));
        on_open.forget();

        let ws = self.clone();
        let on_close = Closure::wrap(Box::new(move |e| {
            debug!("Ws close");
            console::log_1(&e);
            *ws.socket.borrow_mut() = None;
        }) as Box<dyn Fn(JsValue)>);
        socket.set_onclose(Some(on_close.as_ref().unchecked_ref()));
        on_close.forget();

        let ws = self.clone();
        let onmessage_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
            let server_message = convert_response(e);
            if let Some(server_message) = server_message {
                (ws.on_recv.borrow())(server_message);
            }
        }) as Box<dyn FnMut(MessageEvent)>);
        socket.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget();

        let on_error = Closure::wrap(Box::new(move |e| {
            debug!("Ws error");
            console::log_1(&e);
        }) as Box<dyn Fn(JsValue)>);
        socket.set_onerror(Some(on_error.as_ref().unchecked_ref()));
        on_error.forget();

        debug!("ws initialized!");
        Ok(())
    }

    pub fn send_inner(&self, msg: ClientMessage) -> Result<()> {
        let data = bincode::serialize(&msg)?;
        let socket = self.socket.borrow();
        let socket = socket.as_ref().unwrap();
        match socket.send_with_u8_array(&data) {
            Ok(_) => Ok(()),
            Err(e) => bail!("{e:?}"),
        }
    }
}

impl Client for WsClient {
    fn run(&mut self) {
        self.init();
    }

    fn set_callback(&mut self, callback: ServerMessageCallback) {
        *self.on_recv.borrow_mut() = callback;
    }

    fn set_on_connect(&mut self, callback: ConnectCallback) {
        *self.on_connect.borrow_mut() = callback;
    }

    fn send(&self, message: ClientMessage) {
        if let Err(e) = self.send_inner(message) {
            error!("Could not send message: {e}");
        }
    }
}

fn convert_response(me: MessageEvent) -> Option<ServerMessage> {
    let Ok(array) = me.data().dyn_into::<ArrayBuffer>() else {
        return None;
    };
    let uint8_array = js_sys::Uint8Array::new(&array);
    let bytes: Vec<u8> = uint8_array.to_vec();
    let slice: &[u8] = &bytes;

    match bincode::deserialize::<ServerMessage>(slice) {
        Ok(msg) => Some(msg),
        Err(e) => {
            error!("Couldn't deser: {e:?}");
            None
        }
    }
}
