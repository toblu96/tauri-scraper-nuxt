use log::{debug, error, info, warn};
use microkv::MicroKV;
use rumqttc::{
    AsyncClient, ConnectionError, Event, EventLoop, Incoming, MqttOptions, Outgoing, QoS, TlsError,
    Transport,
};
use rustls::{Certificate, ClientConfig, RootCertStore};
use rustls_native_certs::load_native_certs;
use std::io::ErrorKind;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::task::JoinHandle;

use crate::server::router::settings::Broker;
use crate::server::store::AppState;

static DB_KEY: &str = "broker";

#[derive(Clone)]
pub struct MqttClient {
    pub client: Arc<RwLock<AsyncClient>>,
    store: Arc<RwLock<MicroKV>>,
    event_loop_task: Arc<RwLock<JoinHandle<()>>>,
    pub current_client_config: Arc<RwLock<Broker>>,
}

impl MqttClient {
    /// Init mqtt client plugin
    pub fn init(app_state: &Arc<AppState>) -> Self {
        // load initial broker config from local db
        let store = app_state.db.clone();

        // create client
        let (client, eventloop_task, current_client_config) = create_mqtt_client(&store);

        MqttClient {
            client: Arc::new(RwLock::new(client)),
            event_loop_task: Arc::new(RwLock::new(eventloop_task)),
            current_client_config: Arc::new(RwLock::new(current_client_config)),
            store,
        }
    }

    /// Refresh broker connection to latest config values
    pub fn refresh(&mut self) {
        let current = self.current_client_config.read().unwrap().clone();
        let new = self
            .store
            .read()
            .unwrap()
            .get_unwrap::<Broker>(DB_KEY)
            .unwrap();

        // check if something has changed in config
        if current != new {
            info!("Update broker connection with new settings.");
            // stop current eventloop task
            self.event_loop_task.write().unwrap().abort();

            // recreate client and event loop
            let (client, eventloop_task, current_client_config) = create_mqtt_client(&self.store);

            // store updated data to local state
            *self.client.write().unwrap() = client;
            *self.event_loop_task.write().unwrap() = eventloop_task;
            *self.current_client_config.write().unwrap() = current_client_config;
        }
    }

    /// Publish new mqtt message
    pub fn publish(&mut self, topic: &str, payload: serde_json::Value) {
        let client = self.client.read().unwrap().clone();
        let topic = topic.to_string();

        tokio::spawn(async move {
            // let topic = topic.clone();
            // let payload = payload.clone();
            if let Err(err) = client
                .publish(&topic, QoS::AtMostOnce, false, payload.to_string())
                .await
            {
                warn!("Could not publish mqtt message {payload:?} to topic {topic} due to: {err:?}")
            }
        });
    }
}

/// creates a new mqtt client
fn create_mqtt_client(
    store: &Arc<RwLock<MicroKV>>,
) -> (rumqttc::AsyncClient, tokio::task::JoinHandle<()>, Broker) {
    // default broker values
    let mut username = "".to_string();
    let mut password = "".to_string();
    let mut client_id = "test-client-01".to_string();
    let mut host = "localhost".to_string();
    let mut port = 1883;
    let mut protocol = "mqtt://".to_string();
    let mut device_group = "autogroup_Monitor".to_string();
    let mut device_id = "FC_0103".to_string();

    // update default values
    let broker_data = store.read().unwrap().get_unwrap::<Broker>(DB_KEY);
    match broker_data {
        Ok(broker) => {
            username = broker.username;
            password = broker.password;
            client_id = broker.client_id;
            host = broker.host;
            port = broker.port;
            protocol = broker.protocol;
            device_group = broker.device_group;
            device_id = broker.device_id;
        }
        Err(err) => {
            error!("Could not override default broker settings from local file db: {err:?}")
        }
    }

    // create mqtt client
    let mut mqttoptions = MqttOptions::new(&client_id, &host, port);
    mqttoptions.set_keep_alive(Duration::from_secs(30));

    // use auth if provided
    if !&username.is_empty() && !&password.is_empty() {
        mqttoptions.set_credentials(&username, &password);
    }

    // Use rustls-native-certs to load root certificates from the operating system.
    if &protocol == "mqtts://" {
        let mut root_cert_store = RootCertStore::empty();
        for cert in load_native_certs().expect("could not load platform certs") {
            root_cert_store
                .add(&Certificate(cert.0))
                .expect("could not add cert to temporary application store.");
        }

        let client_config = ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root_cert_store)
            .with_no_client_auth();

        mqttoptions.set_transport(Transport::tls_with_config(client_config.into()));
    }

    let (client, eventloop) = AsyncClient::new(mqttoptions, 10);

    // spawn new eventloop task
    let eventloop_task = spawn_eventloop_task(&store, eventloop);

    // store current config
    let current_client_config = Broker {
        client_id,
        device_group,
        device_id,
        host,
        password,
        port,
        protocol,
        username,
        state: "Server started".to_string(),
        connected: false,
    };

    (client, eventloop_task, current_client_config)
}

/// Save new broker connection state to local file db
fn update_broker_state(store: &Arc<RwLock<MicroKV>>, connected: bool, state: &str) {
    let lock = store.write().unwrap();
    let mut broker = lock.get_unwrap::<Broker>(DB_KEY);
    match broker {
        Ok(ref mut broker) => {
            broker.state = state.to_string();
            broker.connected = connected;

            // write to file db
            if let Err(err) = lock.put(DB_KEY, &broker.clone()) {
                error!("Could not update broker state on local file db: {err:?}")
            }
        }
        Err(err) => {
            error!("Could not read broker state from local file db: {err:?}");
            return;
        }
    }
}

// handle mqtt client in separate task
pub fn spawn_eventloop_task(
    store1: &Arc<RwLock<MicroKV>>,
    mut eventloop: EventLoop,
) -> tokio::task::JoinHandle<()> {
    let store = store1.clone();
    tokio::spawn(async move {
        loop {
            match eventloop.poll().await {
                // set client connected status
                Ok(Event::Incoming(Incoming::PingResp))
                | Ok(Event::Incoming(Incoming::ConnAck(_))) => {
                    debug!("[MQTT] Connection successful");
                    update_broker_state(&store, true, "Connected");
                }
                Ok(Event::Incoming(Incoming::Publish(p))) => {
                    info!("[MQTT] Topic: {}, Payload: {:?}", p.topic, p.payload);
                }
                Ok(Event::Incoming(i)) => {
                    info!("[MQTT] Incoming = {:?}", i);
                }
                Ok(Event::Outgoing(Outgoing::PingReq)) => {}
                Ok(Event::Outgoing(o)) => debug!("Outgoing = {:?}", o),
                Err(e) => {
                    match e {
                        ConnectionError::MqttState(e) => {
                            warn!("[MQTT] Pause eventloop task due to: {}", e);
                            update_broker_state(&store, false, &e.to_string());
                            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                        }
                        ConnectionError::Tls(TlsError::Io(e)) => {
                            // prevent filling log with unnecessary socket errors, e.g.:
                            // Tls(Io(Os { code: 11001, kind: Uncategorized, message: "Der angegebene Host ist unbekannt." }))
                            let error: std::io::Error = e;
                            warn!("[MQTT] End eventloop task due to: {}", error);
                            update_broker_state(&store, false, &error.to_string());
                            break;
                        }
                        ConnectionError::Tls(TlsError::DNSName(e)) => {
                            // Tls(DNSName(InvalidDnsNameError))
                            warn!("[MQTT] End eventloop task due to: {}", e);
                            update_broker_state(&store, false, &e.to_string());
                            break;
                        }
                        ConnectionError::Tls(e) => {
                            // catch other Tls errors
                            warn!("[MQTT] End eventloop task due to: {}", e);
                            update_broker_state(&store, false, &e.to_string());
                            break;
                        }
                        ConnectionError::Io(e) => {
                            // prevent filling log with unnecessary socket errors, e.g.:
                            // Io(Custom { kind: ConnectionAborted, error: "connection closed by peer" })
                            // Io(Custom { kind: InvalidData, error: "Promised boundary crossed: 256" })
                            let error: std::io::Error = e;
                            if error.kind() == ErrorKind::InvalidData
                                && error.to_string() == "Promised boundary crossed: 256"
                            {
                                warn!("[MQTT] End eventloop task due to: {}", error);
                                update_broker_state(&store, false, "Needs SSL/TLS enabled");
                                break;
                            } else {
                                warn!("[MQTT] Pause eventloop task due to: {}", error);
                                update_broker_state(&store, false, &error.to_string());
                                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                            }
                        }
                        ConnectionError::ConnectionRefused(e) => {
                            // prevent filling log with unnecessary auth errors, e.g.:
                            // ConnectionRefused(NotAuthorized)
                            warn!("[MQTT] End eventloop task due to: {:?}", e);
                            update_broker_state(&store, false, &format!("{:?}", e));
                            break;
                        }
                        ConnectionError::Timeout(e) => {
                            warn!("[MQTT] Timeout: {}", e);
                            update_broker_state(&store, false, "Timeout");
                        }
                        _ => {
                            error!("[MQTT] Unspecified connection error");
                            update_broker_state(&store, false, "Connection Error");
                            break;
                        }
                    }
                }
            }
        }
    })
}
