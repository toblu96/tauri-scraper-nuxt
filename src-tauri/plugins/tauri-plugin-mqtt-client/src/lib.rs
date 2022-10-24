use rumqttc::{AsyncClient, Event, Incoming, MqttOptions, Outgoing, QoS};
use std::sync::Arc;
use std::time::Duration;
use tauri::{
    async_runtime::{spawn, JoinHandle, Mutex},
    plugin::{Builder, TauriPlugin},
    Manager, Runtime, State,
};

struct MQTTConnection {
    client: Arc<Mutex<Option<AsyncClient>>>,
    event_loop_task: Arc<Mutex<Option<JoinHandle<()>>>>,
}

#[tauri::command]
/**
 * connect to mqtt broker
 */
async fn connect(
    mqtt: State<'_, MQTTConnection>,
    client_id: String,
    host: String,
    port: u16,
) -> Result<(), ()> {
    // check if there is already a client task running and close it
    let mut running_task = mqtt.event_loop_task.lock().await;
    println!("connect event triggered on backend1");
    if running_task.is_some() {
        println!("Task does already exist, delete and replace with new one.");
        running_task.as_ref().unwrap().abort();
        *running_task = None;
    }
    drop(running_task);

    // create mqtt client
    let mut mqttoptions = MqttOptions::new(&client_id, &host, port);
    mqttoptions.set_keep_alive(Duration::from_secs(5));
    // mqttoptions.set_credentials("admin", "public");

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    // handle mqtt client in separate task
    let _task = spawn(async move {
        loop {
            match eventloop.poll().await {
                // set client connected status
                Ok(Event::Incoming(Incoming::PingResp)) => {
                    println!("Connection successful");
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
async fn publish(mqtt: State<'_, MQTTConnection>) -> Result<(), ()> {
    println!("publish");

    //TODO: Handle None in mqtt state

    println!("{:?}", *mqtt.client);
    println!("{:?}", mqtt.client.lock().await.as_ref().unwrap());

    // let client = *mqtt.client.lock().await.clone().unwrap();
    mqtt.client
        .lock()
        .await
        .as_ref()
        .unwrap()
        .publish(
            "eh/test/tbl/hello",
            QoS::AtMostOnce,
            false,
            format!("hello from tauri publish method :)"),
        )
        .await
        .expect("Could not publish message");

    Ok(())
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
