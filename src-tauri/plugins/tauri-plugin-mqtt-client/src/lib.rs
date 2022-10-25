use rumqttc::{AsyncClient, Event, Incoming, MqttOptions, Outgoing, QoS};
use std::sync::Arc;
use std::time::Duration;
use tauri::{
    async_runtime::{spawn, JoinHandle, Mutex},
    plugin::{Builder, TauriPlugin},
    AppHandle, Manager, Runtime, State,
};

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
    // mqttoptions.set_credentials("admin", "public");

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    // handle mqtt client in separate task
    let _task = spawn(async move {
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
