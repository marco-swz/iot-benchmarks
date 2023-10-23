use rustdds::*;
use mio::{Events, Interest, Poll, Token};

fn main() {
    let domain_participant = DomainParticipant::new(0).unwrap();

    let qos = QosPolicyBuilder::new()
      .reliability(policy::Reliability::Reliable { max_blocking_time: rustdds::Duration::DURATION_ZERO })
      .build();

    let subscriber = domain_participant.create_subscriber(&qos).unwrap();

    let topic_req = domain_participant
        .create_topic("dds_req".to_string(), "JustAString".to_string(), &qos, TopicKind::NoKey)
        .unwrap();

    let mut sub = subscriber
        .create_datareader_no_key::<String, CDRDeserializerAdapter<String>>(
            &topic_req, 
            None)
        .unwrap();

    let publisher = domain_participant.create_publisher(&qos).unwrap();

    let topic_rsp = domain_participant
        .create_topic("dds_rsp".to_string(), "JustAString".to_string(), &qos, TopicKind::NoKey)
        .unwrap();

    let publ = publisher
      .create_datawriter_no_key::<String, CDRSerializerAdapter<String>>(
        &topic_rsp,
        None)
      .unwrap();

    const SUB_READY: Token = Token(1);
    const SUB_STATUS_READY: Token = Token(2);

    let mut poll = Poll::new().unwrap();
    let mut events = Events::with_capacity(5);

    poll.registry()
        .register(&mut sub, SUB_READY, Interest::READABLE)
        .unwrap();

    poll.registry()
        .register(
            sub.as_status_source(),
            SUB_STATUS_READY,
            Interest::READABLE,
        )
        .unwrap();

    loop {
        
        if let Err(e) = poll.poll(&mut events, Some(std::time::Duration::from_millis(200))) {
            println!("Poll error {e}");
        }

        for event in &events {
            match event.token() {
                SUB_READY => {
                    loop {
                        println!("DataReader triggered");
                        match sub.take_next_sample() {
                            Ok(Some(sample)) => {
                                println!("recv");
                                publ.write(sample.into_value(), None).unwrap();
                            },
                            Ok(None) => break, // no more data
                            Err(e) => println!("DataReader error: {e:?}")
                        } 
                    }
                },
                SUB_STATUS_READY => {
                    while let Some(status) = sub.try_recv_status() {
                        println!("DataReader status: {status:?}");
                    }
                },
                Token(_) => (),
            } // match token
        } // for
    }
}
