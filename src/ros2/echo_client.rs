use futures::executor::LocalPool;
use futures::future;
use futures::stream::StreamExt;
use futures::task::LocalSpawnExt;
use r2r::QosProfile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = r2r::Context::create()?;
    let mut node = r2r::Node::create(ctx, "ros2_echo", "")?;

    let topic_req = "ros2_req";
    let topic_rsp = "ros2_rsp";

    let mut pool = LocalPool::new();
    let spawner = pool.spawner();

    let publisher = node.create_publisher::<r2r::std_msgs::msg::String>(topic_req, QosProfile::default())?;
    let subscriber = node.subscribe::<r2r::std_msgs::msg::String>(topic_rsp, QosProfile::default())?;

    spawner.spawn_local(async move {
        subscriber.for_each(|msg| {
            let _ = publisher.publish(&msg);
            future::ready(())
        })
        .await
    })?;

    loop {
        node.spin_once(std::time::Duration::from_millis(100));
        pool.run_until_stalled();
    }
}
