use std::io::prelude::*;
use std::net::TcpListener;
use std::thread::spawn;

fn main() {
    let args: Vec<String> = std::env::args()
        .collect();

    let addr_default = "localhost:9001".to_string();
    let addr = args.get(1).unwrap_or(&addr_default).to_string();
    let mut message_size = args.get(2).unwrap_or(&"10".to_string()).parse().unwrap();
    message_size += 8;

    let server_recv = TcpListener::bind(addr).unwrap();
    for stream in server_recv.incoming() {
        spawn(move || {
            let mut stream = stream.unwrap();
            loop {
                let mut msg = vec![0u8; message_size];
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
