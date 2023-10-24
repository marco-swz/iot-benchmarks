use std::time::{Duration, Instant};
use std::sync::{Arc, Barrier};
use serde::Serialize;

use anyhow::Result;

pub struct BenchSettings<T, U: Send + 'static> {
    pub fn_init_send: FnInit<T>,
    pub fn_init_listen: FnInit<U>,
    pub fn_send: FnSend<T>,
    pub fn_listen: FnListen<U>,
    pub message_len: usize,
    pub duration: Duration,
    pub msgs_per_sec: f64,
    pub out_file: String,
}

#[derive(Debug)]
pub struct ClientStats {
    pub num: usize,
    pub num_errors: usize,
    pub duration: Duration,
    pub time_last_msg: Instant,
}

impl ClientStats {
    fn new() -> Self {
        return Self{
            num: 0,
            num_errors: 0,
            duration: Duration::from_secs(0),
            time_last_msg: Instant::now(),
        };
    }
}

#[derive(Debug, Serialize)]
struct BenchStats {
    pub num_sent: usize,
    pub num_received: usize,
    pub num_errors_sent: usize,
    pub num_errors_recv: usize,
    pub duration: Duration,
    pub latency: Duration,
}

impl BenchStats {
    fn new(sender_stats: ClientStats, listener_stats: ClientStats) -> Self {
        return BenchStats {
            num_sent: sender_stats.num,
            num_received: listener_stats.num,
            num_errors_sent: sender_stats.num_errors,
            num_errors_recv: listener_stats.num_errors,
            duration: sender_stats.duration,
            latency: listener_stats.time_last_msg.duration_since(sender_stats.time_last_msg),
        };
    }
}

type FnListen<T> = fn(client: T, duration: Duration) -> Result<ClientStats>;
type FnSend<T> = fn(client: &mut T, msg: String) -> Result<()>;
type FnInit<T> = fn() -> T;

pub fn run_benchmark<T, U: Send + 'static>(settings: BenchSettings<T, U>) {
    let time_wait = Duration::from_secs_f64(1. / settings.msgs_per_sec);
    let time_start = Instant::now();
    let duration = settings.duration;

    // Make sure all clients are initialized before starting
    let barrier = Arc::new(Barrier::new(1));
    let barrier_clone = Arc::clone(&barrier);

    let listen_handle = std::thread::spawn(move || {
        let client = (settings.fn_init_listen)();
        barrier_clone.wait();
        return (settings.fn_listen)(client, duration+Duration::from_secs(5));
    });

    let mut client = (settings.fn_init_send)();

    let mut send_stats = ClientStats::new();
    barrier.wait();
    send_stats.time_last_msg = Instant::now();

    while time_start.elapsed() < duration {
        std::thread::sleep(time_wait);

        let msg = create_string(settings.message_len);

        let s = (settings.fn_send)(&mut client, msg);
        send_stats.time_last_msg = Instant::now();
        if s.is_err() { 
            send_stats.num_errors += 1;
            println!("send error {:?}", s.err());
            continue; 
        }
        send_stats.num += 1;
    }

    send_stats.duration = time_start.elapsed();

    let listen_stats = listen_handle.join().unwrap().unwrap();

    let stats = BenchStats::new(send_stats, listen_stats);
    println!("{:?}", stats);

    let mut file = std::fs::File::create(settings.out_file).unwrap();
    serde_json::to_writer_pretty(&mut file, &stats).unwrap();
}

fn create_string(length: usize) -> String {
    let string = "a".repeat(length);
    return string;
}
