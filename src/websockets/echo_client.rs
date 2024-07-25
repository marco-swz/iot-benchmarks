use std::net::TcpListener;
use std::thread::spawn;
use tungstenite::accept;

fn main () {
    let server_recv = TcpListener::bind("127.0.0.1:9001").unwrap();
    for stream in server_recv.incoming() {
        spawn (move || {
            let mut websocket = accept(stream.unwrap()).unwrap();
            loop {
                let Ok(msg) = websocket.read() else {
                    println!("Read error");
                    break;
                };

                // We do not want to send back ping/pong messages.
                if msg.is_binary() || msg.is_text() {
                    println!("message: {}", msg);
                    websocket.send(msg).unwrap_or_else(|e| println!("Send error: {}", e));
                }
            }
        });
    }
}
