use anyhow::Result;
use std::io::prelude::*;
use std::net::Shutdown;
use std::time::Instant;
use std::{net::TcpStream, time::Duration};

#[path = "../benchmarker.rs"]
mod benchmarker;

use benchmarker::{index_from_message, Benchmarker, MsgType, Receiver, Sender};

struct WsSender {
    stream: TcpStream,
}

impl WsSender {
    pub fn new(stream: TcpStream) -> Self {
        Self { stream }
    }
}

impl Sender for WsSender {
    fn send(&mut self, msg: MsgType) -> Result<()> {
        self.stream.write_all(&msg)?;
        Ok(())
    }
}

struct WsReceiver {
    stream: TcpStream,
    num_messages: usize,
    duration: Duration,
    message_size: usize,
}

impl WsReceiver {
    pub fn new(
        stream: TcpStream,
        num_messages: usize,
        duration: Duration,
        message_size: usize,
    ) -> Self {
        Self {
            stream,
            num_messages,
            duration,
            message_size,
        }
    }
}

impl Receiver for WsReceiver {
    fn listen(&mut self) -> Result<Vec<Option<Instant>>> {
        let mut msgs = vec![None; self.num_messages];

        let mut num_received = 0;
        let time_start = Instant::now();
        loop {
            if num_received == self.num_messages {
                break;
            }

            if time_start.elapsed() > self.duration + Duration::from_secs(5) {
                break;
            }

            let mut msg = vec![0u8; self.message_size];
            let Ok(msg_size) = self.stream.read(&mut msg) else {
                continue;
            };

            if msg_size == 0 {
                println!("disconnected");
                break;
            }

            num_received += 1;

            let Ok(idx) = index_from_message(msg) else {
                continue;
            };
            msgs[idx] = Some(Instant::now());
        }

        return Ok(msgs);
    }
}

fn main() {
    let num_messages = 120000;
    let duration = Duration::from_secs(5);
    let stream = TcpStream::connect("127.0.0.1:9001").unwrap();
    let message_size = 5 + 8;
    stream.set_nonblocking(true).unwrap();
    println!("connected");

    let send = WsSender::new(stream.try_clone().unwrap());
    let recv = WsReceiver::new(stream, num_messages, duration, message_size);
    let mut bench = Benchmarker::new(num_messages, duration, message_size);

    bench.run(send, recv);
}
