use std::time::{Duration, Instant};

use mqtt::Client;
use paho_mqtt as mqtt;
use anyhow::Result;

#[path="../benchmarker.rs"]
mod benchmarker;

use benchmarker::DirStats;
use benchmarker::run_benchmark;

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

fn mqtt_listen(client: Client, duration: Duration) -> Result<DirStats> {
    // Allow exiting using ctrl-c
    let exit_client = client.clone();
    ctrlc::set_handler(move || {
        exit_client.stop_consuming();
    }).expect("Error setting up exit client");

    // Automatically exit after duration + 5sec
    let exit_client = client.clone();
    let _ = std::thread::spawn(move || {
        std::thread::sleep(duration + Duration::from_secs(5));
        exit_client.stop_consuming();
    });

    let mut num_recv = 0; let mut num_errors = 0;
    let rx = client.start_consuming();

    println!("Waiting for messages..");

    let time_start = Instant::now();
    for msg in rx.iter() {
        if let Some(_req) = msg {
            num_recv += 1;
        } else if client.is_connected() || !try_reconnect(&client) {
            break;
        } else {
            num_errors += 1;
        }
    }

    let duration = time_start.elapsed();

    return Ok(DirStats{
        num: num_recv,
        num_errors,
        duration,
    });
}

fn mqtt_send(client: &Client, msg: &String) -> Result<()> {
    let rsp = mqtt::MessageBuilder::new()
        .topic("mqtt_req")
        .payload(msg.as_str())
        .qos(1)
        .finalize();

    return Ok(client.publish(rsp)?)
}

fn main() {
    run_benchmark(
        || mqtt_init("mqtt_rsp"),
        mqtt_listen,
        || mqtt_init("mqtt_req"),
        mqtt_send,
        4, 5., 5
    );
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

