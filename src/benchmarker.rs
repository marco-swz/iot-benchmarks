use anyhow::Result;
use serde::Serialize;
use std::sync::mpsc;
use std::time::{Duration, Instant};
use anyhow::anyhow;
use hdrhistogram::Histogram;

pub type MsgType = Vec<u8>;

#[derive(Debug, Serialize)]
struct BenchStats {
    pub num_sent: usize,
    pub num_received: usize,
    pub latency_avg: f64,
    pub latency_median: u64,
    pub latency_std: f64,
}

impl BenchStats {
    fn new(send_times: Vec<Instant>, recv_times: Vec<Option<Instant>>) -> Self {

        let latencies: Vec<u128> = send_times.iter()
            .zip(recv_times)
            .filter(|(_s, r)| r.is_some())
            .map(|(s, r)| r.unwrap().duration_since(*s).as_micros())
            .collect();

        let mut hist = Histogram::<u64>::new(2).unwrap();

        for time in &latencies {
            hist += *time as u64;
        }

        return BenchStats {
            num_sent: send_times.len(),
            num_received: latencies.len(),
            latency_avg: hist.mean(),
            latency_median: hist.value_at_quantile(0.5),
            latency_std: hist.stdev(),
        };
    }
}

pub trait Sender {
    fn send(&mut self, msg: MsgType) -> Result<()>;
}

pub trait Receiver {
    fn listen(&mut self) -> Result<Vec<Option<Instant>>>;
}

pub struct Benchmarker {
    pub num_messages: usize,
    pub out_file: Option<String>,
    time_wait: Duration,
    stats: Option<BenchStats>,
    message_size: usize,
}

impl Benchmarker {
    pub fn new(num_messages: usize, duration: Duration, message_size: usize) -> Self {
        Benchmarker {
            time_wait: Duration::from_secs_f64(duration.as_secs_f64() / num_messages as f64),
            num_messages,
            out_file: None,
            stats: None,
            message_size,
        }
    }

    pub fn run(&mut self, mut sender: impl Sender, mut receiver: impl Receiver + Send + 'static) {
        let listen_handle = std::thread::spawn(move || {
            let recv_times = receiver.listen();
            return recv_times;
        });

        let mut send_times = Vec::with_capacity(self.num_messages);
        for msg_nr in 0..self.num_messages {
            let msg = create_message(msg_nr, self.message_size);

            let time_send = Instant::now();
            send_times.push(time_send);
            let _ = sender.send(msg);

            std::thread::sleep(self.time_wait);
        }

        drop(sender);

        let recv_times = listen_handle.join().unwrap().unwrap();

        self.stats = Some(BenchStats::new(send_times, recv_times));
        
        dbg!(&self.stats);

        //let mut file = std::fs::File::create(settings.out_file).unwrap();
        //serde_json::to_writer_pretty(&mut file, &stats).unwrap();
        //
    }
}

fn create_message(msg_nr: usize, length: usize) -> MsgType {
    assert!(length > 8);

    let mut msg = Vec::from(msg_nr.to_ne_bytes());

    let string = "a".repeat(length-8);
    let mut padding = Vec::from(string.as_bytes());
    msg.append(&mut padding);

    return msg;
}

pub fn index_from_message(msg: MsgType) -> Result<usize> {
    if msg.len() < 8 {
        return Err(anyhow!("Message too short"));
    }
    let idx: [u8; 8] = msg[..8].try_into()?;
    return Ok(usize::from_ne_bytes(idx));
}

