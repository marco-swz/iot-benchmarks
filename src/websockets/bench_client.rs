use std::net::TcpStream;
use tungstenite::{connect, Message, WebSocket, stream::MaybeTlsStream};
use std::time::{Duration, Instant};
use anyhow::Result;

#[path="../benchmarker.rs"]
mod benchmarker;

use benchmarker::{Benchmarker, Sender, Receiver, MsgType};

type Socket = WebSocket<MaybeTlsStream<TcpStream>>;

fn socket_init() -> Socket {

    let (socket, response) =
        connect("ws://localhost:9001/socket").expect("Can't connect");

    println!("Connected to the server");
    println!("Response HTTP code: {}", response.status());
    println!("Response contains the following headers:");
    for (ref header, _value) in response.headers() {
        println!("* {}", header);
    }

    return socket;
}

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

struct WsReceiver {
    socket: Socket,
}

impl WsReceiver {
    pub fn new() -> Self {
        let (socket, response) =
            connect("ws://localhost:9002/socket").expect("Can't connect");

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

impl Receiver for WsReceiver {
    fn listen(&mut self, rx_stop: std::sync::mpsc::Receiver<()>) -> Result<Vec<Option<Instant>>> {
        let mut num_recv = 0; let mut num_errors = 0;
        let mut time_last_msg = Instant::now();

        while time_start.elapsed() < duration {
            // TODO: Find a way to stop (read is blocking)
            let Ok(msg) = socket.read() else {
                println!("Recv error");
                num_errors += 1;
                continue;
            };
            time_last_msg = Instant::now();
            num_recv += 1;
            println!("Received: {}", msg);
        }

        socket.close(None).unwrap();
            todo!()
        }
}


fn socket_listen(mut socket: Socket, duration: Duration) -> Result<ClientStats> {
    let mut num_recv = 0; let mut num_errors = 0;
    let time_start = Instant::now();
    let mut time_last_msg = Instant::now();

    while time_start.elapsed() < duration {
        // TODO: Find a way to stop (read is blocking)
        let Ok(msg) = socket.read() else {
            println!("Recv error");
            num_errors += 1;
            continue;
        };
        time_last_msg = Instant::now();
        num_recv += 1;
        println!("Received: {}", msg);
    }

    socket.close(None).unwrap();

    return Ok(ClientStats{
        num: num_recv,
        num_errors,
        time_last_msg,
        duration: time_start.elapsed(),
    });
}


fn main() {
    let settings = BenchSettings{
        fn_init_send: socket_init,
        fn_init_listen: socket_init,
        fn_send: socket_send,
        fn_listen: socket_listen,
        duration: Duration::from_secs(5),
        msgs_per_sec: 1.,
        message_len: 10,
        out_file: "data/socket.json".to_string(),
    };

    run_benchmark(settings);
}
