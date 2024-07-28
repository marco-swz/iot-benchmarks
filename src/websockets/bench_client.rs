use std::{net::TcpStream, time::Duration};
use tungstenite::{connect, Message, WebSocket, stream::MaybeTlsStream};
use std::time::Instant;
use anyhow::Result;

#[path="../benchmarker.rs"]
mod benchmarker;

use benchmarker::{Benchmarker, Sender, Receiver, MsgType};

type Socket = WebSocket<MaybeTlsStream<TcpStream>>;

struct WsSender {
    socket: Socket,
}

impl WsSender {
    pub fn new() -> Self {
        let (socket, response) =
            connect("ws://localhost:9001/socket").expect("Can't connect");

        println!("Connected to the server");
        println!("Response HTTP code: {}", response.status());
        println!("Response contains the following headers:");
        for (ref header, _value) in response.headers() {
            println!("* {}", header);
        }

        Self{
            socket
        }
    }
}

impl Sender for WsSender {
    fn send(&mut self, msg: MsgType) -> Result<()> {
        self.socket.send(Message::Binary(msg))?;
        Ok(())
    }
}

impl Drop for WsSender {
    fn drop(&mut self) {
        let _ = self.socket.close(None);
        let _ = self.socket.flush();
    }
}

struct WsReceiver {
    socket: Socket,
    num_messages: usize,
    duration: Duration,
}

impl WsReceiver {
    pub fn new(num_messages: usize, duration: Duration) -> Self {
        let (socket, response) =
            connect("ws://localhost:9001/socket").expect("Can't connect");

        println!("Connected to the server");
        println!("Response HTTP code: {}", response.status());
        println!("Response contains the following headers:");
        for (ref header, _value) in response.headers() {
            println!("* {}", header);
        }

        Self{
            socket,
            num_messages,
            duration
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

            // `read` will return if connection is closed
            let Ok(msg) = self.socket.read() else {
                dbg!("err");
                continue;
            };

            num_received += 1;

            dbg!(&msg);

            let Ok(idx) = index_from_message(msg.into_data()) else {
                continue;
            };
            msgs[idx] = Some(Instant::now());
        }

        dbg!(&msgs);
        _ = self.socket.close(None);

        return Ok(msgs);
    }
}

fn index_from_message(msg: MsgType) -> Result<usize> {
    let idx: [u8; 8] = msg[..8].try_into()?;
    return Ok(usize::from_ne_bytes(idx));
}

fn main() {
    let num_messages = 10;
    let duration = Duration::from_secs(5);
    let send = WsSender::new();
    let recv = WsReceiver::new(num_messages, duration);
    let bench = Benchmarker::new(num_messages, duration);

    bench.run(send, recv);
}
