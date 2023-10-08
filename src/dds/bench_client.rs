use rustdds::*;
use rustdds::no_key::{DataWriter, DataReader};
#[path="../benchmarker.rs"]
mod benchmarker;

use benchmarker::{Benchmarker, BenchClient};

struct DdsClient {
    publisher: DataWriter<String>,
    subscriber: DataReader<String>,
}

impl DdsClient {
    fn new() -> Self {
        let domain_participant = DomainParticipant::new(0).unwrap();

        let qos = QosPolicyBuilder::new()
          .reliability(policy::Reliability::Reliable { max_blocking_time: rustdds::Duration::DURATION_ZERO })
          .build();

        let subscriber = domain_participant.create_subscriber(&qos).unwrap();

        let topic_req = domain_participant
            .create_topic("dds_req".to_string(), "JustAString".to_string(), &qos, TopicKind::NoKey)
            .unwrap();

        let subscriber = subscriber
            .create_datareader_no_key::<String, CDRDeserializerAdapter<String>>(
                &topic_req, 
                None)
            .unwrap();

        let publisher = domain_participant.create_publisher(&qos).unwrap();

        let topic_rsp = domain_participant
            .create_topic("dds_rsp".to_string(), "JustAString".to_string(), &qos, TopicKind::NoKey)
            .unwrap();

        let publisher = publisher
          .create_datawriter_no_key::<String, CDRSerializerAdapter<String>>(
            &topic_rsp,
            None)
          .unwrap();

        return DdsClient{
            publisher,
            subscriber,
        }
    }
}

impl BenchClient for DdsClient {
    fn send(&self, msg: &String) -> anyhow::Result<()> {
        return Ok(self.publisher.write(msg.to_string(), None)?);
    }

    fn wait_for_response(&mut self) -> anyhow::Result<String> {
        loop {
            match self.subscriber.take_next_sample() {
                Ok(Some(msg)) => return Ok(msg.value().to_string()),
                Err(e) => return Err(e.into()),
                _ => (),
            };
        } 

    }
}

fn main() {
    let mut bench = Benchmarker::new(
        Box::new(DdsClient::new())
    );

    bench.run(4, 5., 5);

}
