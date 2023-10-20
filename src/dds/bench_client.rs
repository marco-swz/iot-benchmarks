use rustdds::{DomainParticipant, CDRSerializerAdapter, CDRDeserializerAdapter, QosPolicyBuilder, TopicKind};
use rustdds::no_key::{DataWriter, DataReader};
use rustdds::policy;
use anyhow::Result;
use std::time::{Duration, Instant};

#[path="../benchmarker.rs"]
mod benchmarker;

use benchmarker::{ClientStats, BenchSettings};
use benchmarker::run_benchmark;

fn dds_init_pub() -> DataWriter<String> {
    let domain_participant = DomainParticipant::new(0).unwrap();

    let qos = QosPolicyBuilder::new()
      .reliability(policy::Reliability::Reliable { max_blocking_time: rustdds::Duration::DURATION_ZERO })
      .build();

    let publisher = domain_participant.create_publisher(&qos).unwrap();

    let topic = domain_participant
        .create_topic("dds_rsp".to_string(), "JustAString".to_string(), &qos, TopicKind::NoKey)
        .unwrap();

    let publisher = publisher
      .create_datawriter_no_key::<String, CDRSerializerAdapter<String>>(
        &topic,
        None)
      .unwrap();

    return publisher;
}

fn dds_init_sub() -> DataReader<String> {
    let domain_participant = DomainParticipant::new(0).unwrap();

    let qos = QosPolicyBuilder::new()
      .reliability(policy::Reliability::Reliable { max_blocking_time: rustdds::Duration::DURATION_ZERO })
      .build();

    let subscriber = domain_participant.create_subscriber(&qos).unwrap();

    let topic = domain_participant
        .create_topic("dds_req".to_string(), "JustAString".to_string(), &qos, TopicKind::NoKey)
        .unwrap();

    let subscriber = subscriber
        .create_datareader_no_key::<String, CDRDeserializerAdapter<String>>(
            &topic, 
            None)
        .unwrap();

    return subscriber;
}

fn dds_send(publisher: &DataWriter<String>, msg: &String) -> Result<()> {
        return Ok(publisher.write(msg.to_string(), None)?);
}

fn dds_listen(mut subscriber: DataReader<String>, duration: Duration) -> Result<ClientStats> {
    let (mut num_recv, mut num_errors) = (0, 0);

    let mut time_last_msg = Instant::now();
    let time_start = Instant::now();
    while time_start.elapsed() < duration {
        match subscriber.take_next_sample() {
            Ok(Some(_msg)) => {
                num_recv += 1;
                time_last_msg = Instant::now();
            },
            Err(_) => num_errors += 1,
            _ => (),
        };
    } 

    return Ok(ClientStats{
        num: num_recv,
        num_errors,
        time_last_msg,
        duration: time_start.elapsed(),
    });
}

fn main() {
    let settings = BenchSettings{
        fn_init_send: dds_init_pub,
        fn_init_listen: dds_init_sub,
        fn_send: dds_send,
        fn_listen: dds_listen,
        duration: Duration::from_secs(5),
        msgs_per_sec: 1.,
        message_len: 10,
        out_file: "data/dds.json".to_string(),
    };

    run_benchmark(settings);
}
