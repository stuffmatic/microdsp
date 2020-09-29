use crossbeam_channel::unbounded;
use std::cell::Cell;
use std::env;
use std::rc::Rc;
use std::thread;
use std::time::Duration;
use ws::{connect, listen, WebSocket, CloseCode, Factory, Handler, Handshake, Message, Result, Sender};

type MessageType = String;

struct Server {
    out: Sender,
    tx: crossbeam_channel::Sender<MessageType>
}

// https://www.jan-prochazka.eu/ws-rs/guide.html
// https://github.com/housleyjk/ws-rs/issues/131

impl Handler for Server {
    fn on_message(&mut self, msg: Message) -> Result<()> {
        println!("Incoming ws message '{}'. ", msg);
        match self.tx.send(msg.to_string()) {
            Ok(_) => {
                println!("Relayed ws message")
            },
            Err(e) => {
                println!("Failed to relay ws message {}", e)
            }
        }
        Ok(())
        // self.out.send(msg)
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

fn ws_main(tx: crossbeam_channel::Sender<MessageType>, rx: crossbeam_channel::Receiver<MessageType>) {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());
    listen(addr, |out| Server {
        out,
        tx: tx.clone()
    })
    .unwrap();
    /*let listen_result = listen(addr, |out| {
        // The handler needs to take ownership of out, so we use move
        move |msg| {
            // Handle messages received on this connection
            println!("Server got message '{}'. ", msg);

            // Use the out channel to send messages back
            out.send(msg)
        }
    });*/
}

fn main() {
    // r receives data from (a number of) connected clients. Gets cloned per client
    // s sends data to all clients
    let (tx_send, rx_send) = unbounded::<MessageType>();
    let (tx_recv, rx_recv) = unbounded::<MessageType>();

    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());

    let socket = WebSocket::new(move |out| Server {
        out,
        tx: tx_recv.clone()
    }).unwrap();
    let broadcaster = socket.broadcaster();

    let b_join_handle = thread::spawn(move || {
        loop {
            if let Ok(x) = rx_send.recv() {
                println!("Broadcasting {}", x);
                broadcaster.send(x).expect("Unable to send WebSocket message.")
            } else {
                println!("Shutting down broadcaster thread.");
                break
            }
        }
    });

    let s_join_handle = thread::spawn(move || {
        socket.listen(addr).expect("Unable to listen on websocket");
    });

    let poll_interval_ms = 500;
    let mut i = 0;
    println!("Entering event loop, polling every {} ms", poll_interval_ms);
    loop {
        i += 1;
        let x = tx_send.send(i.to_string());
        loop {
            match rx_recv.try_recv() {
                Ok(value) => {
                    println!("Received value {}", value)
                },
                Err(error) => {
                    println!("Failed to received value {}", error);
                    break
                }
            }
        }

        if i % 2 == 0 {
            let _ = tx_send.send(format!("Outgoing message #{}", i));
        }

        thread::sleep(Duration::from_millis(poll_interval_ms));
        if i > 100 {
            break;
        }
    }

    s_join_handle.join().expect("Websocket thread failed");
    b_join_handle.join().expect("Broadcaster thread failed");

    /*let join_handle = std::thread::spawn(|| ws_main(s, r));

    loop {
      let x = r_clone.recv().unwrap();
      println!("Got recv {}", x);
    }

    let poll_interval_ms = 30;
    println!("Entering event loop, polling every {} ms", poll_interval_ms);
    loop {
        thread::sleep(Duration::from_millis(poll_interval_ms));
    }*/
}
