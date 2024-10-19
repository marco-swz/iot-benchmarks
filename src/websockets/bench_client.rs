use std::{net::TcpStream, time::Duration};
use tungstenite::{connect, Message, WebSocket, stream::MaybeTlsStream};
use std::time::Instant;
use anyhow::Result;

#[path="../benchmarker.rs"]
mod benchmarker;
use benchmarker::{Benchmarker, Sender, Receiver, MsgType, index_from_message};

#[path="../config.rs"]
mod config;
use config::Config;

type Socket = WebSocket<MaybeTlsStream<TcpStream>>;

struct WsSender {
    socket: Socket,
}

impl WsSender {
    pub fn new(addr: &String) -> Self {
        let (socket, response) =
            connect(addr).expect("Can't connect");

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
    pub fn new(addr: &String, num_messages: usize, duration: Duration) -> Self {
        let (socket, response) =
            connect(addr).expect("Can't connect");

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

fn run_bench(addr: &String, num_messages: usize, duration: Duration, message_size: usize) {
    let send = WsSender::new(addr);
    let recv = WsReceiver::new(addr, num_messages, duration);
    let mut bench = Benchmarker::new(num_messages, duration, message_size+8);

    bench.run(send, recv);
}

fn main() {
    let config: Config = toml::from_str(
        &std::fs::read_to_string("config.toml").unwrap()
    ).unwrap();

    let schedule = config.websocket.schedule;

    let step = (schedule.stop_req_per_sec - schedule.start_req_per_sec) / schedule.steps as f64;
    for i in 0..schedule.steps {
        let duration = Duration::from_secs(schedule.secs_per_step);
        let num_messages = (schedule.start_req_per_sec * i as f64 + step) * schedule.secs_per_step as f64;

        run_bench(
            &config.websocket.address,
            num_messages.floor() as usize, 
            duration, 
            config.websocket.message_size,
        );
    }
}
