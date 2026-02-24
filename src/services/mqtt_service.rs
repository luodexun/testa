/*
 * @Author         : Shang
 * @Date           : 2024-09-23
 * @LastEditors    : Shang
 * @LastEditTime   : 2024-10-14
 * @Description    :
 */
use rumqttc::{AsyncClient, ConnectReturnCode, Event, MqttOptions, Packet, QoS, Transport};
use serde_json::json;
use std::{sync::Arc, time::Duration};
use tauri::{command, State, Window};
use tokio::sync::Mutex;

#[derive(Default)]
pub struct MqttClient {
    client: Option<AsyncClient>,
}

#[command]
pub async fn mqtt_connect(
    window: Window,
    client: State<'_, Arc<Mutex<MqttClient>>>,
    id: String,
    host: String,
    port: u16,
    username: String,
    password: String,
) -> Result<(), String> {
    let mut mqtt_client = client.lock().await;

    if mqtt_client.client.is_some() {
        window.emit("tauri_mqtt_connected", true).unwrap();
        return Ok(());
    }

    let mut mqtt_options = MqttOptions::new(id.clone(), host.clone(), port.clone());
    mqtt_options
        .set_request_channel_capacity(50)
        .set_max_packet_size(157286400, 1048576)
        .set_keep_alive(Duration::from_secs(15))
        .set_clean_session(true)
        .set_inflight(50);

    if !username.is_empty() {
        mqtt_options.set_credentials(username.clone(), password.clone());
    }

    // let ws_config = WebsocketConfig::new(format!("wss://{}:{}/mqtt", host, port));
    // mqtt_options.set_transport(Transport::Wss(ws_config));
    
    let (new_client, mut connection) = AsyncClient::new(mqtt_options, 50);
    mqtt_client.client = Some(new_client);

    // 克隆 Arc<Mutex<MqttClient>> 以便在后台任务中可以安全地修改全局状态
    let client_state_clone = client.inner().clone();

    tokio::spawn(async move {
        loop {
            match connection.poll().await {
                Ok(event) => match event {
                    Event::Incoming(Packet::ConnAck(connack)) => {
                        if connack.code == ConnectReturnCode::Success {
                            println!(
                                "[tauri] mqtt_connect success (id: {}, host: {}, port: {})",
                                id, host, port
                            );
                            window.emit("tauri_mqtt_connected", true).unwrap();
                        } else {
                            println!(
                                "[tauri] mqtt_connect failed (id: {}), code: {:?}",
                                id, connack.code
                            );
                            // 连接失败，清理状态
                            let mut guard = client_state_clone.lock().await;
                            guard.client = None;
                            window.emit("tauri_mqtt_connected", false).unwrap();
                        }
                    }
                    Event::Incoming(Packet::Publish(publish)) => {
                        let payload = String::from_utf8_lossy(&publish.payload).to_string();
                        let emit_data = json!({"topic":&publish.topic,"data":payload});
                        println!(
                            "[tauri] mqtt_connect incoming publish:{}",
                            emit_data.clone()
                        );
                        window
                            .emit("tauri_mqtt_message_received", emit_data)
                            .unwrap();
                    }
                    Event::Incoming(Packet::Disconnect) => {
                        println!("[tauri] mqtt_connect incoming disconnect");
                        window.emit("tauri_mqtt_disconnect", true).unwrap();
                        // 服务器主动断开，清理状态并退出循环
                        let mut guard = client_state_clone.lock().await;
                        guard.client = None;
                        break;
                    }
                    _ => {
                        println!("[tauri] mqtt_connect Incoming OTHER:{:?}", event);
                    }
                },
                Err(e) => {
                    println!("[tauri] Error while polling: {:?}", e);
                    window.emit("tauri_mqtt_disconnect", e.to_string()).unwrap();
                    // 发生错误，清理状态并退出循环
                    let mut guard = client_state_clone.lock().await;
                    guard.client = None;
                    break;
                }
            }
        }
    });

    Ok(())
}

#[command]
pub async fn mqtt_subscribe_topic(
    client: State<'_, Arc<Mutex<MqttClient>>>,
    topic: String,
) -> Result<(), String> {
    let mut client = client.lock().await;
    if client.client.is_none() {
        return Err("MQTT client is not connected".to_string());
    }
    if let Some(ref mut mqtt_client) = client.client {
        mqtt_client
            .subscribe(topic.clone(), QoS::AtMostOnce)
            .await
            .map_err(|e| {
                println!("[tauri] mqtt_subscribe_topic error{}", e);
                e.to_string()
            })?;
    }
    println!("[tauri] mqtt_subscribe_topic success ,topic:{}", topic);
    Ok(())
}

#[command]
pub async fn mqtt_publish_message(
    client: State<'_, Arc<Mutex<MqttClient>>>,
    topic: String,
    message: String,
) -> Result<(), String> {
    let client = client.lock().await;
    if client.client.is_none() {
        return Err("MQTT client is not connected".to_string());
    }
    if let Some(ref mqtt_client) = client.client {
        mqtt_client
            .publish(
                topic.clone(),
                rumqttc::QoS::AtMostOnce,
                false,
                message.clone(),
            )
            .await
            .map_err(|e| {
                println!("[tauri] mqtt_publish_message error{}", e);
                e.to_string()
            })?;
    }
    println!(
        "[tauri] mqtt_publish_message success ,topic:{},message:{}",
        topic, message
    );
    Ok(())
}

#[command]
pub async fn mqtt_unsubscribe_topic(
    client: State<'_, Arc<Mutex<MqttClient>>>,
    topic: String,
) -> Result<(), String> {
    let mut client = client.lock().await;
    if let Some(ref mut mqtt_client) = client.client {
        mqtt_client.unsubscribe(topic.clone()).await.map_err(|e| {
            println!("[tauri] mqtt_unsubscribe_topic error{}", e);
            e.to_string()
        })?;
    }
    println!("[tauri] mqtt_unsubscribe_topic success ,topic:{}", topic);
    Ok(())
}

#[command]
pub async fn mqtt_disconnect(client: State<'_, Arc<Mutex<MqttClient>>>) -> Result<(), String> {
    let mut client_guard = client.lock().await;
    // 使用 .take() 可以安全地取出 Option 中的值，并将原位置为 None，完美解决状态重置问题
    if let Some(mqtt_client) = client_guard.client.take() {
        mqtt_client.disconnect().await.map_err(|e| {
            println!("[tauri] mqtt_disconnect error{}", e);
            e.to_string()
        })?;
    }
    println!("[tauri] mqtt_disconnect success");
    Ok(())
}
