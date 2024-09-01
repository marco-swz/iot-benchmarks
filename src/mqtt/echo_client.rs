use paho_mqtt as mqtt;
use anyhow::Result;

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

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let addr_default = "localhost:1883".to_string();
    let addr = args.get(1).unwrap_or(&addr_default);

    let host = format!("mqtt://{addr}");
    println!("Connecting to MQTT broker at {}", host);

    let opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(host)
        .client_id("mqtt_echo")
        .finalize();

    let client = mqtt::Client::new(opts)
        .expect("Error creating client");

    let rx = client.start_consuming();

    let resp_disconnect = mqtt::MessageBuilder::new()
        .topic("mqtt_echo")
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
                    println!("Subscribing to topic 'mqtt_req'");
                    client.subscribe("mqtt_req", 1)
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

    let exit_client = client.clone();
    ctrlc::set_handler(move || {
        exit_client.stop_consuming();
    }).expect("Error setting up exit client");

    println!("Waiting for messages..");
    for msg in rx.iter() {
        if let Some(req) = msg {

            let rsp = mqtt::MessageBuilder::new()
                .topic("mqtt_rsp")
                .payload(req.payload())
                .qos(1)
                .finalize();

            if let Err(_) = client.publish(rsp) {
                println!("Error sending response");
            }

        } else if client.is_connected() || !try_reconnect(&client) {
            break;
        }
    }

    return Ok(());
}
