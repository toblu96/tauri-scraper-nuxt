use rumqttc::{AsyncClient, Event, Incoming, MqttOptions, Outgoing, QoS, ConnectionError, TlsError, Transport};
use std::{sync::Arc, io::ErrorKind};
use std::time::Duration;
use tauri::{
    async_runtime::{spawn, JoinHandle, Mutex},
    plugin::{Builder, TauriPlugin},
    AppHandle, Manager, Runtime, State,
};
use rustls::{RootCertStore, Certificate, ClientConfig};
use rustls_native_certs::load_native_certs;

struct MQTTConnection {
    client: Arc<Mutex<Option<AsyncClient>>>,
    event_loop_task: Arc<Mutex<Option<JoinHandle<()>>>>,
}

#[tauri::command]
/**
 * connect to mqtt broker
 */
async fn connect<R: Runtime>(
    _app: AppHandle<R>,
    mqtt: State<'_, MQTTConnection>,
    client_id: String,
    host: String,
    port: u16,
    protocol: String,
    username: String,
    password: String
) -> Result<(), ()> {
    // check if there is already a client task running and close it
    let mut running_task = mqtt.event_loop_task.lock().await;
    println!("connect event triggered on backend1");
    if running_task.is_some() {
        println!("Task does already exist, delete and replace with new one.");
        running_task.as_ref().unwrap().abort();
        *running_task = None;
        // reset connection state in frontend store
        _app.emit_all("plugin:mqtt//connected", false).unwrap();
    }
    drop(running_task);

    // create mqtt client
    let mut mqttoptions = MqttOptions::new(&client_id, &host, port);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    // use auth if provided
    if !&username.is_empty() && !&password.is_empty() {
        println!("use auth");
        mqttoptions.set_credentials(&username, &password);
    }

    // Use rustls-native-certs to load root certificates from the operating system.
    if &protocol == "mqtts://" {
        let mut root_cert_store = RootCertStore::empty();
        for cert in load_native_certs().expect("could not load platform certs") {
            root_cert_store.add(&Certificate(cert.0)).expect("could not add cert to temporary application store.");
        }
        
        let client_config = ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_cert_store)
        .with_no_client_auth();
        
        mqttoptions.set_transport(Transport::tls_with_config(client_config.into()));
    }

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    // handle mqtt client in separate task
    let _task = spawn(async move {
        _app.emit_all("plugin:mqtt//connection-status", "Disconnected").unwrap();
        loop {
            match eventloop.poll().await {
                // set client connected status
                Ok(Event::Incoming(Incoming::PingResp))
                | Ok(Event::Incoming(Incoming::ConnAck(_))) => {
                    println!("Connection successful");
                    _app.emit_all("plugin:mqtt//connected", true).unwrap();
                }
                Ok(Event::Incoming(Incoming::Publish(p))) => {
                    println!("Topic: {}, Payload: {:?}", p.topic, p.payload);
                }
                Ok(Event::Incoming(i)) => {
                    println!("Incoming = {:?}", i);
                }
                Ok(Event::Outgoing(Outgoing::PingReq)) => {}
                Ok(Event::Outgoing(o)) => println!("Outgoing = {:?}", o),
                Err(e) => {
                    println!("Error = {:?}", e);
                    _app.emit_all("plugin:mqtt//connected", false).unwrap();

                    match e {
                        ConnectionError::MqttState(e) => {
                            println!("Pause eventloop task due to: {}", e);
                            _app.emit_all("plugin:mqtt//connection-status", format!("{:?}", e)).unwrap();
                            std::thread::sleep(std::time::Duration::from_secs(1))
                        }
                        ConnectionError::Tls(TlsError::Io(e)) => {
                            // prevent filling log with unnecessary socket errors, e.g.:
                            // Tls(Io(Os { code: 11001, kind: Uncategorized, message: "Der angegebene Host ist unbekannt." }))
                            let error: std::io::Error = e;
                            println!("End eventloop task due to: {}", error);
                            _app.emit_all("plugin:mqtt//connection-status", error.to_string()).unwrap();
                            break;
                        }
                        ConnectionError::Tls(TlsError::DNSName(e)) => {
                            // Tls(DNSName(InvalidDnsNameError))
                            println!("End eventloop task due to: {}", e);
                            _app.emit_all("plugin:mqtt//connection-status", format!("{:?}", e)).unwrap();
                            break;
                        }
                        ConnectionError::Tls(e) => {
                            // catch other Tls errors
                            println!("End eventloop task due to: {}", e);
                            _app.emit_all("plugin:mqtt//connection-status", format!("{:?}", e)).unwrap();
                            break;
                        }
                        ConnectionError::Io(e) => {
                            // prevent filling log with unnecessary socket errors, e.g.:
                            // Io(Custom { kind: ConnectionAborted, error: "connection closed by peer" })
                            // Io(Custom { kind: InvalidData, error: "Promised boundary crossed: 256" })
                            let error: std::io::Error = e;
                            if error.kind() == ErrorKind::InvalidData && error.to_string() == "Promised boundary crossed: 256" {
                                println!("End eventloop task due to: {}", error);
                                _app.emit_all("plugin:mqtt//connection-status", "Needs SSL/TLS enabled").unwrap();
                                break;
                            } else {
                                println!("Pause eventloop task due to: {}", error);
                                _app.emit_all("plugin:mqtt//connection-status", error.to_string()).unwrap();
                                std::thread::sleep(std::time::Duration::from_secs(10))
                            }
                            
                        },
                        ConnectionError::ConnectionRefused(e) => {
                            // prevent filling log with unnecessary auth errors, e.g.:
                            // ConnectionRefused(NotAuthorized)
                            println!("End eventloop task due to: {:?}", e);
                            _app.emit_all("plugin:mqtt//connection-status", format!("{:?}", e)).unwrap();
                            break;
                        },
                        ConnectionError::Timeout(_e) => {
                            _app.emit_all("plugin:mqtt//connection-status", "Timeout").unwrap();
                        }
                        _ => {
                            _app.emit_all("plugin:mqtt//connection-status", "Connection Error").unwrap();
                            break;
                        }
                    }
                }
            }
        }
    });

    // store local state
    *mqtt.client.lock().await = Some(client);
    *mqtt.event_loop_task.lock().await = Some(_task);

    Ok(())
}

#[tauri::command]
async fn publish<R: Runtime>(
    _app: AppHandle<R>,
    mqtt: State<'_, MQTTConnection>,
    topic: String,
    payload: serde_json::Value
) -> Result<(), String> {
    println!("publish");

    //TODO: Handle None in mqtt state

    println!("{}", topic);
    println!("{}", payload);

    // let client = *mqtt.client.lock().await.clone().unwrap();
    let resp = mqtt.client
        .lock()
        .await
        .as_ref()
        .unwrap()
        .publish(
            &topic,
            QoS::AtMostOnce,
            false,
            payload.to_string(),
        )
        .await;

    match resp {
        Ok(()) => { return Ok(());}
        Err(e) => {return Err(e.to_string())}
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("mqtt-client")
        .invoke_handler(tauri::generate_handler![connect, publish])
        .setup(|app_handle| {
            // setup plugin specific state here
            app_handle.manage(MQTTConnection {
                client: Default::default(),
                event_loop_task: Default::default(),
            });
            Ok(())
        })
        .build()
}
