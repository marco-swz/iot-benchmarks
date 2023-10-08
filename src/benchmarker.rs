use std::time::{Duration, Instant};

use anyhow::Result;

#[derive(Debug)]
struct BenchStats {
    pub num_sent: usize,
    pub num_received: usize,
    pub num_errors: usize,
    pub duration: Duration,
}

impl BenchStats {
    fn new(duration: Duration) -> Self {
        return BenchStats {
            num_sent: 0,
            num_received: 0,
            num_errors: 0,
            duration,
        };
    }
}

pub trait BenchClient {
    fn send(&self, msg: &String) -> Result<()>;
    fn wait_for_response(&mut self) -> Result<String>;
}

pub struct Benchmarker {
    client: Box<dyn BenchClient>,
}

impl Benchmarker {
    pub fn new(client: Box<dyn BenchClient>) -> Self {
        return Benchmarker{
            client
        };
    }

    pub fn run(&mut self, message_len: usize, msgs_per_sec: f64, duration_sec: u64) {
        let time_wait = Duration::from_secs_f64(1. / msgs_per_sec);
        let time_start = Instant::now();
        let duration = Duration::from_secs(duration_sec);

        let mut stats = BenchStats::new(duration);

        while time_start.elapsed() < duration {
            let msg = create_string(message_len);

            if self.client.send(&msg).is_err() { continue; }
            stats.num_sent += 1;
            println!("sent: {}", msg);

            match self.client.wait_for_response() {
                Ok(resp) => {
                    stats.num_received += 1;
                    println!("recv: {}", resp);
                    if resp != msg {
                        stats.num_errors += 1;
                    }
                },
                Err(_) => {
                    stats.num_errors += 1;
                }
            };

            std::thread::sleep(time_wait);
        }

        println!("{:?}", stats);
    }
}

fn create_string(length: usize) -> String {
    let string = "a".repeat(length);
    return string;
}
