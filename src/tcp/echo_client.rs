use std::io::prelude::*;
use std::net::TcpListener;
use std::thread::spawn;

fn main() {
    let server_recv = TcpListener::bind("127.0.0.1:9001").unwrap();
    for stream in server_recv.incoming() {
        spawn(move || {
            let mut stream = stream.unwrap();
            loop {
                let mut msg = vec![0u8; 5 + 8];
                let Ok(msg_size) = stream.read(&mut msg) else {
                    break;
                };

                if msg_size == 0 {
                    // The connection is closed
                    break;
                }

                stream.write_all(&msg).unwrap();
            }

            println!("disconnected");
        });
    }
}
