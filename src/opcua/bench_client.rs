use std::{sync::Arc, time::{Duration, Instant}};
use anyhow::Result;

use opcua::{client::prelude::*, sync::RwLock};
use opcua::{
    crypto::SecurityPolicy,
    types::{
        DataValue, MessageSecurityMode, MonitoredItemCreateRequest, NodeId, StatusCode,
        TimestampsToReturn, UserTokenPolicy,
    },
};

#[path = "../benchmarker.rs"]
mod benchmarker;
use benchmarker::{index_from_message, Benchmarker, MsgType, Receiver, Sender};

#[path = "../config.rs"]
mod config;
use config::Config;

const DEFAULT_URL: &str = "opc.tcp://localhost:4855";

struct OpcuaSender {
}

impl OpcuaSender {
    pub fn new() -> Self {
        Self {  }
    }
}

impl Sender for OpcuaSender {
    fn send(&mut self, msg: MsgType) -> Result<()> {
        Ok(())
    }
}

struct OpcuaReceiver {
    num_messages: usize,
    duration: Duration,
    message_size: usize,
}

impl OpcuaReceiver {
    pub fn new(
        num_messages: usize,
        duration: Duration,
        message_size: usize,
    ) -> Self {
        Self {
            num_messages,
            duration,
            message_size,
        }
    }
}

impl Receiver for OpcuaReceiver {
    fn listen(&mut self) -> Result<Vec<Option<Instant>>> {
        let mut msgs = vec![None; self.num_messages];

        let mut num_received = 0;
        let time_start = Instant::now();
        loop {
            if num_received == self.num_messages {
                break;
            }

            if time_start.elapsed() > self.duration + Duration::from_secs(5) {
                break;
            }

            let mut msg = vec![0u8; self.message_size*8 + 8];

            num_received += 1;

            let Ok(idx) = index_from_message(msg) else {
                println!("parse error");
                continue;
            };
            msgs[idx] = Some(Instant::now());
        }

        return Ok(msgs);
    }
}

async fn run_bench(addr: &String, num_messages: usize, duration: Duration, message_size: usize) {

    // Make the client configuration
    let mut client = ClientBuilder::new()
        .application_name("Simple Client")
        .application_uri("urn:SimpleClient")
        .product_uri("urn:SimpleClient")
        .trust_server_certs(true)
        .create_sample_keypair(true)
        .session_retry_limit(3)
        .client()
        .unwrap();

    let endpoint: EndpointDescription = ("opc.tcp://localhost:4855/", "None", MessageSecurityMode::None, UserTokenPolicy::anonymous()).into();

    // Create the session
    let session = client.connect_to_endpoint(endpoint, IdentityToken::Anonymous).unwrap();

    if subscribe_to_values(session.clone()).is_ok() {
        let _ = Session::run(session);
    } else {
        println!("Error creating subscription");
    }

    // Increase size to add  message number
    let message_size = message_size + 8;
    println!("connected");

    let send = OpcuaSender::new();
    let recv = OpcuaReceiver::new(num_messages, duration, message_size);
    let mut bench = Benchmarker::new(num_messages, duration, message_size);

    let stats = bench.run(send, recv);
    dbg!(&stats);
    //stream.shutdown(std::net::Shutdown::Both).unwrap();
}

#[tokio::main]
async fn main() -> Result<()> {
    let config: Config = toml::from_str(
        &std::fs::read_to_string("config.toml").unwrap()
    ).unwrap();

    let schedule = config.opcua.schedule;

    let step = (schedule.stop_req_per_sec - schedule.start_req_per_sec) / schedule.steps as f64;
    for i in 0..schedule.steps {
        let duration = Duration::from_secs(schedule.secs_per_step);
        let num_messages = (schedule.start_req_per_sec * i as f64 + step) * schedule.secs_per_step as f64;

        run_bench(
            &config.opcua.address,
            num_messages.floor() as usize, 
            duration, 
            config.opcua.message_size,
        );
    }

    Ok(())
}

fn subscribe_to_values(session: Arc<RwLock<Session>>) -> Result<(), StatusCode> {
    let session = session.write();
    // Create a subscription polling every 2s with a callback
    let subscription_id = session.create_subscription(2000.0, 10, 30, 0, 0, true, DataChangeCallback::new(|changed_monitored_items| {
        println!("Data change from server:");
        changed_monitored_items.iter().for_each(|item| print_value(item));
    }))?;
    // Create some monitored items
    let items_to_create: Vec<MonitoredItemCreateRequest> = ["v1", "v2", "v3", "v4"].iter()
        .map(|v| NodeId::new(2, *v).into()).collect();
    let _ = session.create_monitored_items(subscription_id, TimestampsToReturn::Both, &items_to_create)?;
    Ok(())
}

fn print_value(item: &MonitoredItem) {
   let node_id = &item.item_to_monitor().node_id;
   let data_value = item.last_value();
   if let Some(ref value) = data_value.value {
       println!("Item \"{}\", Value = {:?}", node_id, value);
   } else {
       println!("Item \"{}\", Value not found, error: {}", node_id, data_value.status.as_ref().unwrap());
   }
}
