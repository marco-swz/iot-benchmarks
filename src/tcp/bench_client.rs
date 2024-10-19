use anyhow::Result;
use std::io::prelude::*;
use std::time::Instant;
use std::{net::TcpStream, time::Duration};

#[path = "../benchmarker.rs"]
mod benchmarker;
use benchmarker::{index_from_message, Benchmarker, MsgType, Receiver, Sender};

#[path = "../config.rs"]
mod config;
use config::Config;

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

            let mut msg = vec![0u8; self.message_size*8 + 8];
            let Ok(msg_size) = self.stream.read(&mut msg) else {
                continue;
            };

            if msg_size == 0 {
                println!("disconnected");
                break;
            }

            num_received += 1;

            let Ok(idx) = index_from_message(msg) else {
                println!("parse error");
                continue;
            };
            msgs[idx] = Some(Instant::now());
        }

        return Ok(msgs);
    }
}

fn run_bench(addr: &String, num_messages: usize, duration: Duration, message_size: usize) {
    let stream = TcpStream::connect(addr).unwrap();
    // Increase size to add  message number
    let message_size = message_size + 8;
    stream.set_nonblocking(true).unwrap();
    println!("connected");

    let send = WsSender::new(stream.try_clone().unwrap());
    let recv = WsReceiver::new(stream.try_clone().unwrap(), num_messages, duration, message_size);
    let mut bench = Benchmarker::new(num_messages, duration, message_size);

    let stats = bench.run(send, recv);
    dbg!(&stats);
    //stream.shutdown(std::net::Shutdown::Both).unwrap();
}

fn main() {
    let config: Config = toml::from_str(
        &std::fs::read_to_string("config.toml").unwrap()
    ).unwrap();

    let schedule = config.tcp.schedule;

    let step = (schedule.stop_req_per_sec - schedule.start_req_per_sec) / schedule.steps as f64;
    for i in 0..schedule.steps {
        let duration = Duration::from_secs(schedule.secs_per_step);
        let num_messages = (schedule.start_req_per_sec * i as f64 + step) * schedule.secs_per_step as f64;

        run_bench(
            &config.tcp.address,
            num_messages.floor() as usize, 
            duration, 
            config.tcp.message_size,
        );
    }
}
