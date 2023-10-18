use std::time::{Duration, Instant};
use std::sync::{Arc, Barrier};

use anyhow::Result;

#[derive(Debug)]
pub struct DirStats {
    pub num: usize,
    pub num_errors: usize,
    pub duration: Duration,
}

#[derive(Debug)]
struct BenchStats {
    pub num_sent: usize,
    pub num_received: usize,
    pub num_errors_sent: usize,
    pub num_errors_recv: usize,
    pub duration: Duration,
}

impl BenchStats {
    fn new(duration: Duration) -> Self {
        return BenchStats {
            num_sent: 0,
            num_received: 0,
            num_errors_sent: 0,
            num_errors_recv: 0,
            duration,
        };
    }
}

type FnListen<T> = fn(client: T, duration: Duration) -> Result<DirStats>;
type FnSend<T> = fn(client: &T, msg: &String) -> Result<()>;
type FnInit<T> = fn() -> T;


pub fn run_benchmark<T: Send + 'static>(fn_init_listen: FnInit<T>, fn_listen: FnListen<T>, fn_init_send: FnInit<T>, fn_send: FnSend<T>, message_len: usize, msgs_per_sec: f64, duration_sec: u64) {
    let time_wait = Duration::from_secs_f64(1. / msgs_per_sec);
    let time_start = Instant::now();
    let duration = Duration::from_secs(duration_sec);

    // Make sure all clients are initialized before starting
    let barrier = Arc::new(Barrier::new(1));
    let barrier_clone = Arc::clone(&barrier);

    let listen_handle = std::thread::spawn(move || {
        let client = fn_init_listen();
        barrier_clone.wait();
        return fn_listen(client, duration);
    });

    let client = fn_init_send();

    let mut stats = BenchStats::new(duration);

    barrier.wait();
    while time_start.elapsed() < duration {
        std::thread::sleep(time_wait);

        let msg = create_string(message_len);

        let s = fn_send(&client, &msg);
        if s.is_err() { 
            stats.num_errors_sent += 1;
            println!("send error {:?}", s.err());
            continue; 
        }
        stats.num_sent += 1;
        println!("sent: {}", msg);
    }

    let stats = listen_handle.join().unwrap().unwrap();
    println!("{:?}", stats);
}

fn create_string(length: usize) -> String {
    let string = "a".repeat(length);
    return string;
}
