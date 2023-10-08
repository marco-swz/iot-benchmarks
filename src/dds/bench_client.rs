#[path="../benchmarker.rs"]
mod benchmarker;

use benchmarker::{Benchmarker, BenchClient};

struct DdsClient {}

impl BenchClient for DdsClient {
    fn send(&self, _msg: &String) -> anyhow::Result<()> {
        return Ok(());
    }

    fn wait_for_response(&self) -> anyhow::Result<String> {
        return Ok("aaaa".to_string());
    }
}


fn main() {
    let bench = Benchmarker::new(
        Box::new(DdsClient{})
    );

    bench.run(4, 5., 5);
}
