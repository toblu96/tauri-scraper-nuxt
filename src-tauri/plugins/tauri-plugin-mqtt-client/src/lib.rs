use rcgen::generate_simple_self_signed;
use rumqttc::{AsyncClient, Client, EventLoop, MqttOptions, QoS, TlsConfiguration, Transport};
use rustls::client::ResolvesClientCert;
use rustls::server::AllowAnyAnonymousOrAuthenticatedClient;
use rustls::{ClientConfig, ConfigBuilder, RootCertStore};
use std::error::Error;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::{
    async_runtime::Mutex,
    plugin::{Builder, TauriPlugin},
    AppHandle, Manager, Runtime, State,
};
use tokio::{task, time};

struct Counter(AtomicUsize);

struct MQTTAsyncClient(rumqttc::AsyncClient, rumqttc::EventLoop);

struct MQTT(Mutex<Option<AsyncClient>>);

#[tauri::command]
// this will be accessible with `invoke('plugin:awesome|initialize')`.
// where `awesome` is the plugin name.
fn initialize(counter: State<'_, Counter>) {
    let c = counter.0.fetch_add(1, Ordering::Relaxed) + 1;
    println!("New state: {}", c);
}

#[tauri::command]
/**
 * connect to mqtt broker
 */
async fn connect(_mqtt: State<'_, MQTT>) -> Result<(), ()> {
    // create cert
    let subject_alt_names = vec!["localhost".to_string()];
    let cert = generate_simple_self_signed(subject_alt_names).unwrap();
    println!("pem: {}", cert.serialize_pem().unwrap());
    println!("key: {}", cert.serialize_private_key_pem());

    // mqtt logic
    let mut mqttoptions = MqttOptions::new("rumqtt-async", "localhost", 1882);
    println!("N: {:?}", mqttoptions.broker_address());
    mqttoptions.set_keep_alive(Duration::from_secs(5));
    // mqttoptions.set_credentials("admin", "public");

    // Use rustls-native-certs to load root certificates from the operating system.
    let mut root_cert_store = rustls::RootCertStore::empty();
    let mut cert_der = vec![0];
    match cert.serialize_der() {
        Ok(cert) => cert_der = cert,
        Err(error) => println!("Problem opening the file: {:?}", error),
    }
    // let key = rustls::PrivateKey(cert.serialize_private_key_der());
    // root_cert_store.add_parsable_certificates(&[cert_der]);
    for cert in rustls_native_certs::load_native_certs().expect("could not load platform certs") {
        root_cert_store.add(&rustls::Certificate(cert.0));

        println!("cert loaded from windows.");
    }
    // root_cert_store.add_server_trust_anchors(&[webpki_roots::TLS_SERVER_ROOTS]);

    // AllowAnyAnonymousOrAuthenticatedClient::new(root_cert_store);

    let client_config = ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_cert_store)
        .with_no_client_auth();

    // mqttoptions.set_transport(Transport::tls_with_config(
    //     rumqttc::TlsConfiguration::Simple {
    //         ca: vec![0],
    //         alpn: None,
    //         client_auth: None,
    //     },
    // ));

    // mqttoptions.set_transport(Transport::tls_with_config(client_config.into()));

    // *_mqtt.0.lock().await = Some(AsyncClient::new(mqttoptions, 10).0);

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    let sub = client.subscribe("eh/test/#", QoS::AtMostOnce).await;
    match sub {
        Ok(msg) => println!("isOk: {:?}", msg),
        Err(error) => println!("Problem subscribing: {:?}", error),
    };

    // time::sleep(Duration::from_secs(1)).await;

    client
        .publish("eh/test/15", QoS::AtLeastOnce, false, "hello from tauri :)")
        // .publish("eh/test/15", QoS::AtLeastOnce, false, vec![1; 1])
        .await
        .expect("cant publish");

    // match resp {
    //     Ok(msg) => println!("isOk: {:?}", msg),
    //     Err(error) => println!("Problem opening the file: {:?}", error),
    // };

    println!("connection done. ");
    // *mqtt_client = client;
    // *mqtt_eventloop = eventloop;
    loop {
        let event = eventloop.poll().await;

        println!("{:?}", event);
        match event {
            Ok(msg) => println!("data ok: {:?}", msg),
            Err(error) => {
                println!("Problem receiving data: {:?}", error);
                break;
            }
        };
    }
    // for notification in eventloop.poll().await.iter() {
    //     println!("Notification = {:?}", notification);
    // }

    Ok(())
}

#[tauri::command]
async fn publish(_mqtt: State<'_, MQTT>) -> Result<(), ()> {
    println!("publish");
    // let mut client = _mqtt.0.lock().await.as_ref().unwrap();

    // let mut client = _mqtt.0.lock().await.unwrap().0;
    // task::spawn(async move {
    _mqtt
        .0
        .lock()
        .await
        .as_ref()
        .unwrap()
        .publish(
            "eh/test/tbl/hello",
            QoS::AtMostOnce,
            false,
            format!("hello from tauri :)"),
        )
        .await
        .err();
    // .unwrap();
    // })
    // .await;
    Ok(())
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("mqtt-client")
        .invoke_handler(tauri::generate_handler![initialize, connect, publish])
        .setup(|app_handle| {
            // setup plugin specific state here
            app_handle.manage(Counter(AtomicUsize::new(0)));
            app_handle.manage(MQTT(Default::default()));
            Ok(())
        })
        // .manage(MQTT(Default::default()))
        .build()
}
