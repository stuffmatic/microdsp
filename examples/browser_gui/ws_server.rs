use std::env;
use std::thread;
use crossbeam_channel::unbounded;
use ws::{CloseCode, Handler, Handshake, Message, Result, WebSocket};

type MessageType = String;

struct WebSocketHandler {
    tx: crossbeam_channel::Sender<MessageType>,
}

impl Handler for WebSocketHandler {
    fn on_message(&mut self, msg: Message) -> Result<()> {
        println!("Incoming ws message '{}'. ", msg);
        match self.tx.send(msg.to_string()) {
            Ok(_) => println!("Relayed ws message"),
            Err(e) => println!("Failed to relay ws message {}", e),
        }
        Ok(())
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        println!("WebSocket closing for ({:?}) {}", code, reason);
    }

    fn on_open(&mut self, shake: Handshake) -> Result<()> {
        if let Some(addr) = shake.remote_addr()? {
            println!("Connection with {} now open", addr);
        }
        Ok(())
    }
}

pub struct WebsocketServer {
  pub tx_send: crossbeam_channel::Sender<MessageType>,
  pub rx_recv: crossbeam_channel::Receiver<MessageType>,
  pub broadcaster_join_handle: std::thread::JoinHandle<()>,
  pub socket_join_handle: std::thread::JoinHandle<()>
}

pub fn start_ws_server() -> WebsocketServer {
    let addr = "127.0.0.1:9876".to_string();

    // A channel for pushing data from the main thread to the websocket for sending
    let (tx_send, rx_send) = unbounded::<MessageType>();
    // A channel for pushing incoming data from the websocket to the main thread
    let (tx_recv, rx_recv) = unbounded::<MessageType>();

    // Create a websocket
    let socket = WebSocket::new(move |_| WebSocketHandler {
        tx: tx_recv.clone(),
    })
    .unwrap();

    // For sending messages to all connected clients
    let broadcaster = socket.broadcaster();

    // Spawn a thread for receiving and broadcasting messages to all connected clients
    let broadcaster_join_handle = thread::spawn(move || loop {
        if let Ok(x) = rx_send.recv() {
            broadcaster
                .send(x)
                .expect("Unable to send WebSocket message.")
        } else {
            println!("Shutting down broadcaster thread.");
            break;
        }
    });

    // Spawn a thread for accepting websocket connections
    let socket_join_handle = thread::spawn(move || {
        socket.listen(addr).expect("Unable to listen on websocket");
    });

    WebsocketServer {
      socket_join_handle,
      broadcaster_join_handle,
      tx_send,
      rx_recv
    }
}
