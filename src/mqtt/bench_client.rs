use std::time::{Duration, Instant};

use mqtt::Client;
use paho_mqtt as mqtt;
use anyhow::Result;

#[path="../benchmarker.rs"]
mod benchmarker;

use benchmarker::{Receiver, Sender, MsgType, index_from_message, Benchmarker};

struct WsSender {
    client: Client,
}

impl WsSender {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

impl Sender for WsSender {
    fn send(&mut self, msg: MsgType) -> Result<()> {
        let rsp = mqtt::MessageBuilder::new()
            .topic("mqtt_req")
            .payload(msg)
            .qos(1)
            .finalize();

        return Ok(self.client.publish(rsp)?)
    }
}

struct WsReceiver {
    client: Client,
    num_messages: usize,
    duration: Duration,
}

impl WsReceiver {
    pub fn new(
        client: Client,
        num_messages: usize,
        duration: Duration,
    ) -> Self {
        Self {
            client,
            num_messages,
            duration,
        }
    }
}

impl Receiver for WsReceiver {
    fn listen(&mut self) -> Result<Vec<Option<Instant>>> {

        let time_start = Instant::now();
        let mut timestamps = vec![None; self.num_messages];
        let mut num_received = 0;

        let rx = self.client.start_consuming();

        println!("Waiting for messages..");

        loop {
            let Ok(msg) = rx.try_recv() else {
                continue;
            };

            let Some(msg) = msg else {
                if self.client.is_connected() || !try_reconnect(&self.client) {
                    break;
                } 
                continue;
            };

            let Ok(idx) = index_from_message(msg.payload().to_vec()) else {
                continue;
            };

            timestamps[idx] = Some(Instant::now());

            num_received += 1;
            if num_received >= self.num_messages {
                break;
            }

            if time_start.elapsed() > self.duration + Duration::from_secs(5) {
                break;
            }
        }

        return Ok(timestamps);

    }
}


fn mqtt_init(topic: &str) -> Client {
    let host = "mqtt://localhost:1883".to_string();
    println!("Connecting to MQTT broker at {}", host);

    let opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(host)
        .client_id(topic)
        .finalize();

    let client = mqtt::Client::new(opts)
        .expect("Error creating client");


    let resp_disconnect = mqtt::MessageBuilder::new()
        .topic("mqtt_disconnect")
        .payload("Connection lost")
        .finalize();

    let conn_opts = mqtt::ConnectOptionsBuilder::new()
        .keep_alive_interval(std::time::Duration::from_secs(20))
        .clean_session(false)
        .will_message(resp_disconnect)
        .finalize();

    match client.connect(conn_opts) {
        Ok(rsp) => {
            if let Some(conn_rsp) = rsp.connect_response() {
                println!("Connected to broker");

                if conn_rsp.session_present {
                    println!("Session already present on broker");
                } else {
                    println!("Subscribing to topic {}", topic);
                    client.subscribe(topic, 1)
                        .and_then(|rsp| {
                            return rsp.subscribe_response().ok_or(mqtt::Error::General("Bad response"));
                        })
                        .and_then(|vqos| {
                            println!("QoS granted: {:?}", vqos);
                            return Ok(());
                        })
                        .unwrap_or_else(|err| {
                            client.disconnect(None).unwrap();
                            panic!("Error subscribing to topic: {:?}", err);
                        });
                }
            }
        }
        Err(e) => {
            panic!("Error connecting to broker {:?}", e);
        }
    }

    return client;
}

fn try_reconnect(client: &mqtt::Client) -> bool {
    println!("Connection lost. Reconnecting..");
    for _ in 0..60 {
        std::thread::sleep(std::time::Duration::from_secs(1));
        if client.reconnect().is_ok() {
            println!("Reconnect sucessful");
            return true;
        }
    }
    println!("Failed to reconnect");
    return false;
}

fn main() {
    let num_messages = 120000;
    let duration = Duration::from_secs(5);
    let client = mqtt_init("mqtt_bench");
    let message_size = 5 + 8;

    let send = WsSender::new(client.clone());
    let recv = WsReceiver::new(client, num_messages, duration);
    let mut bench = Benchmarker::new(num_messages, duration, message_size);

    bench.run(send, recv);
}
