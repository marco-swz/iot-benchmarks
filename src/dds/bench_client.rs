use rustdds::{DomainParticipant, CDRSerializerAdapter, CDRDeserializerAdapter, QosPolicyBuilder, TopicKind};
use rustdds::no_key::{DataWriter, DataReader};
use rustdds::policy;
use anyhow::Result;
use std::time::{Duration, Instant};
use mio::{Events, Interest, Poll, Token};

#[path="../benchmarker.rs"]
mod benchmarker;

use benchmarker::{ClientStats, BenchSettings};
use benchmarker::run_benchmark;

fn dds_init_pub() -> (DataWriter<String>, DomainParticipant) {
    let domain_participant = DomainParticipant::new(1).unwrap();

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

    return (publisher, domain_participant);
}

fn dds_init_sub() -> (DataReader<String>, DomainParticipant) {
    let domain_participant = DomainParticipant::new(2).unwrap();

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

    return (subscriber, domain_participant);
}

fn dds_send(pub_part: &mut (DataWriter<String>, DomainParticipant), msg: String) -> Result<()> {
    let (publisher, _) = pub_part;
    println!("publ");
    return Ok(publisher.write(msg.to_string(), None)?);
}

fn dds_listen(sub_part: (DataReader<String>, DomainParticipant), duration: Duration) -> Result<ClientStats> {
    let (mut subscriber, _) = sub_part;
    let (mut num_recv, mut num_errors) = (0, 0);

    let mut time_last_msg = Instant::now();
    let time_start = Instant::now();
    //while time_start.elapsed() < duration {
    //    match subscriber.take_next_sample() {
    //        Ok(Some(_msg)) => {
    //            println!("listen");
    //            num_recv += 1;
    //            time_last_msg = Instant::now();
    //        },
    //        Err(_) => num_errors += 1,
    //        _ => (),
    //    };
    //} 
    const SUB_READY: Token = Token(1);
    const SUB_STATUS_READY: Token = Token(2);

    let mut poll = Poll::new().unwrap();
    let mut events = Events::with_capacity(5);

    poll.registry()
        .register(&mut subscriber, SUB_READY, Interest::READABLE)
        .unwrap();

    //poll.registry()
    //    .register(
    //        sub.as_status_source(),
    //        SUB_STATUS_READY,
    //        Interest::READABLE,
    //    )
    //    .unwrap();

    loop {
        
        if let Err(e) = poll.poll(&mut events, Some(std::time::Duration::from_millis(200))) {
            println!("Poll error {e}");
        }

        for event in &events {
            match event.token() {
                SUB_READY => {
                    loop {
                        println!("DataReader triggered");
                        match subscriber.take_next_sample() {
                            Ok(Some(sample)) => {
                                println!("recv");
                                num_recv += 1;
                            },
                            Ok(None) => break, // no more data
                            Err(e) => println!("DataReader error: {e:?}")
                        } 
                    }
                },
                //SUB_STATUS_READY => {
                //    while let Some(status) = subscriber.try_recv_status() {
                //        println!("DataReader status: {status:?}");
                //    }
                //},
                Token(_) => (),
            } // match token
        } // for
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
