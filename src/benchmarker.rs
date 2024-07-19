use anyhow::Result;
use serde::Serialize;
use std::sync::mpsc;
use std::time::{Duration, Instant};

type MsgType = Vec<u8>;

#[derive(Debug, Serialize)]
struct BenchStats {
    pub num_sent: usize,
    pub num_received: usize,
    pub num_errors_sent: usize,
    pub num_errors_recv: usize,
    pub duration: Duration,
    pub latency: Duration,
}

//impl BenchStats {
//    fn new(sender_stats: ClientStats, listener_stats: ClientStats) -> Self {
//        return BenchStats {
//            num_sent: sender_stats.num,
//            num_received: listener_stats.num,
//            num_errors_sent: sender_stats.num_errors,
//            num_errors_recv: listener_stats.num_errors,
//            duration: sender_stats.duration,
//            latency: listener_stats.time_last_msg.duration_since(sender_stats.time_last_msg),
//        };
//    }
//}

pub trait Sender {
    fn send(&mut self, msg: MsgType) -> Result<()>;
}

pub trait Receiver {
    fn listen(&mut self, rx_stop: mpsc::Receiver<()>) -> Result<Vec<Option<Instant>>>;
}

pub struct Benchmarker {
    pub num_messages: usize,
    pub out_file: Option<String>,
    time_wait: Duration,
    stats: Option<BenchStats>,
}

impl Benchmarker {
    pub fn new(msgs_per_sec: f64, num_messages: usize) -> Self {
        Benchmarker {
            time_wait: Duration::from_secs_f64(1. / msgs_per_sec),
            num_messages,
            out_file: None,
            stats: None,
        }
    }

    pub fn run(&self, mut sender: impl Sender, mut receiver: impl Receiver + Send + 'static) {
        let msg_length = 10;
        let (tx_stop, rx_stop) = mpsc::channel();

        let listen_handle = std::thread::spawn(move || {
            let recv_times = receiver.listen(rx_stop);
            return recv_times;
        });

        let mut send_times = Vec::with_capacity(self.num_messages);
        for msg_nr in 0..self.num_messages {
            let msg = create_message(msg_nr, msg_length);

            let time_send = Instant::now();
            send_times.push(time_send);
            let res_send = sender.send(msg);

            std::thread::sleep(self.time_wait);
        }

        tx_stop.send(());
        let recv_times = listen_handle.join().unwrap().unwrap();

        //let stats = BenchStats::new(send_times, recv_times);

        for i in 0..send_times.len() {
            let send_time = send_times.get(i).unwrap();
            let recv_time = recv_times.get(i);

            let Some(Some(recv_time)) = recv_time else {
                continue;
            };

            println!("{:?}", recv_time.duration_since(*send_time).as_secs_f64());
        }

        //let mut file = std::fs::File::create(settings.out_file).unwrap();
        //serde_json::to_writer_pretty(&mut file, &stats).unwrap();
    }
}

fn create_message(msg_nr: usize, length: usize) -> Vec<u8> {
    let mut msg = Vec::from(msg_nr.to_ne_bytes());

    let string = "a".repeat(length);
    let mut padding = Vec::from(string.as_bytes());
    msg.append(&mut padding);

    return msg;
}
