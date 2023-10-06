use rustdds::*;
use rustdds::no_key::{DataReader, DataWriter, DataSample, Sample}; // We use a NO_KEY topic here
use serde::{Serialize, Deserialize};
use mio::{Events, Interest, Poll, Token};

fn main() {
    let domain_participant = DomainParticipant::new(0).unwrap();

    let qos = QosPolicyBuilder::new()
      .reliability(policy::Reliability::Reliable { max_blocking_time: rustdds::Duration::DURATION_ZERO })
      .build();

    let subscriber = domain_participant.create_subscriber(&qos).unwrap();
    let publisher = domain_participant.create_publisher(&qos).unwrap();

    let some_topic = domain_participant
        .create_topic("some_topic".to_string(), "SomeType".to_string(), &qos, TopicKind::NoKey)
        .unwrap();


    let mut reader = subscriber
        .create_datareader_no_key::<String, CDRDeserializerAdapter<String>>(
            &some_topic, 
            None)
        .unwrap();

    let writer = publisher
      .create_datawriter_no_key::<String, CDRSerializerAdapter<String>>(
        &some_topic,
        None)
      .unwrap();

    let mut poll = Poll::new().unwrap();
    let mut events = Events::with_capacity(5);

    let mut reader_opt = poll
        .registry()
        .register(
            reader.as_status_source(),
            WRITER_STATUS_READY,
            Interest::READABLE,
        )
        .unwrap();

    loop {
        
        if let Err(e) = poll.poll(&mut events, Some(std::time::Duration::from_millis(200))) {
            println!("Poll error {e}");
        }

        for event in &events {
            match event.token() {
                READER_READY => {
                    match reader_opt {
                        Some(ref mut reader) => {
                            loop {
                                println!("DataReader triggered");
                                match reader.take_next_sample() {
                                    Ok(Some(sample)) => match sample.into_value() {
                                        Sample::Value(sample) => {
                                            writer.write(sample, None).unwrap();
                                        }
                                        Sample::Dispose(key) => println!("Disposed key"),
                                    },
                                    Ok(None) => break, // no more data
                                    Err(e) => println!("DataReader error: {e:?}")
                                } // match next sample
                            }
                        }
                    } // match reader_opt
                },
                READER_STATUS_READY => match reader_opt {
                    Some(ref mut reader) => {
                        while let Some(status) = reader.try_recv_status() {
                            println!("DataReader status: {status:?}");
                        }
                    }
                    None => {
                        error!("Where is my reader?");
                    }
                },
            } // match token
        }
    }

}
