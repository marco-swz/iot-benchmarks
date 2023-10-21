use futures::executor::LocalPool;
use futures::{future, Stream};
use futures::stream::StreamExt;
use futures::task::LocalSpawnExt;
use r2r::{QosProfile, Node, Publisher};
use r2r::std_msgs::msg;

use std::time::{Duration, Instant};

use anyhow::Result;

#[path="../benchmarker.rs"]
mod benchmarker;

use benchmarker::{ClientStats, BenchSettings};
use benchmarker::run_benchmark;

fn ros2_init_sub() -> (Node, impl Stream<Item = msg::String> + 'static) {
    let ctx = r2r::Context::create().unwrap();
    let mut node = r2r::Node::create(ctx, "ros2_sub", "").unwrap();

    let topic_rsp = "ros2_rsp";
    let subscriber = node.subscribe::<r2r::std_msgs::msg::String>(topic_rsp, QosProfile::default()).unwrap();

    return (node, subscriber);
}

fn ros2_listen(node_sub: (Node, impl Stream<Item = msg::String> + 'static), duration: Duration) -> Result<ClientStats> {
    let (mut node, subscriber) = node_sub;

    let mut pool = LocalPool::new();
    let spawner = pool.spawner();

    let mut num_recv = 0; let num_errors = 0;
    let time_start = Instant::now();
    let mut time_last_msg = Instant::now();

    println!("Waiting for messages..");

    spawner.spawn_local(async move {
        subscriber.for_each(|_msg| {
            num_recv += 1;
            time_last_msg = Instant::now();
            future::ready(())
        })
        .await
    })?;

    while time_start.elapsed() < duration {
        node.spin_once(std::time::Duration::from_millis(100));
        pool.run_until_stalled();
    }

    let duration = time_start.elapsed();

    return Ok(ClientStats{
        num: num_recv,
        num_errors,
        duration,
        time_last_msg,
    });
}

fn ros2_init_pub() -> Publisher<msg::String> {
    let ctx = r2r::Context::create().unwrap();
    let mut node = r2r::Node::create(ctx, "ros2_echo", "").unwrap();

    let topic_req = "ros2_req";

    let publisher = node.create_publisher::<r2r::std_msgs::msg::String>(topic_req, QosProfile::default()).unwrap();
    return publisher;
}

fn ros2_send(publisher: &Publisher<msg::String>, message: String) -> Result<()> {
    let message = msg::String{ data: message };
    return Ok(publisher.publish(&message)?);
}

fn main() -> Result<()> {
    let settings = BenchSettings{
        fn_init_send: ros2_init_pub,
        fn_init_listen: ros2_init_sub,
        fn_send: ros2_send,
        fn_listen: ros2_listen,
        duration: Duration::from_secs(5),
        msgs_per_sec: 10.,
        message_len: 10,
        out_file: "data/ros2.json".to_string(),
    };

    run_benchmark(settings);
    return Ok(());
}
