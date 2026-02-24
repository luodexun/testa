/*
 * @Author         : Shang
 * @Date           : 2024-09-23
 * @LastEditors    : Shang
 * @LastEditTime   : 2024-10-14
 * @Description    :
 */
use rumqttc::{AsyncClient, ConnectReturnCode, Event, MqttOptions, Packet, QoS, TlsConfiguration, Transport};
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
    // 克隆 Arc 以便在异步任务中使用，用于在连接断开时清理 client
    let client_state = client.inner().clone();
    let mut mqtt_client = client.lock().await;

    if mqtt_client.client.is_some() {
        window.emit("tauri_mqtt_connected", true).unwrap();
        return Ok(());
    }

    // 1) 读取证书
    let ca = tokio::fs::read("/usr/lib/ness/ca.pem").await.map_err(|e| e.to_string())?;
    let cert = tokio::fs::read("/usr/lib/ness/client-cert.pem").await.map_err(|e| e.to_string())?;
    let key  = tokio::fs::read("/usr/lib/ness/client-key.key").await.map_err(|e| e.to_string())?;

    // 2) 构造 TLS 配置
    // let tls = TlsConfiguration::Native;
    let tls = TlsConfiguration::Simple {
        // ca: Vec::new(),
        ca,
        client_auth: Some((cert, key)),
        alpn: None,
    };

    // 3) 构造 MqttOptions（不再链式调用，避免任何潜在的引用）
    let mut mqtt_options = MqttOptions::new(id.clone(), host.clone(), port);
    mqtt_options.set_request_channel_capacity(50);
    mqtt_options.set_max_packet_size(157_286_400, 1_048_576);
    mqtt_options.set_keep_alive(Duration::from_secs(15));
    mqtt_options.set_clean_session(true);
    mqtt_options.set_inflight(50);
    mqtt_options.set_transport(Transport::Tls(tls));
    mqtt_options.set_credentials(username.clone(), password.clone());

    // 4) 创建客户端
    let (new_client, mut eventloop) = AsyncClient::new(mqtt_options, 50);
    mqtt_client.client = Some(new_client);

    tokio::spawn(async move {
        loop {
            match eventloop.poll().await {
                Ok(Event::Incoming(Packet::ConnAck(connack))) => {
                    let ok = connack.code == ConnectReturnCode::Success;
                    println!("[tauri] mqtt_connect success: {}", ok);
                    window.emit("tauri_mqtt_connected", ok).unwrap();
                    if !ok {
                        // 连接失败，清理 client
                        let mut c = client_state.lock().await;
                        c.client = None;
                        break;
                    }
                }
                Ok(Event::Incoming(Packet::Publish(publish))) => {
                    let payload = String::from_utf8_lossy(&publish.payload).to_string();
                    let emit_data = json!({"topic": publish.topic, "data": payload});
                    println!("[tauri] mqtt_connect incoming publish: {}", emit_data);
                    window.emit("tauri_mqtt_message_received", emit_data).unwrap();
                }
                Ok(Event::Incoming(Packet::Disconnect)) => {
                    println!("[tauri] mqtt_connect incoming disconnect");
                    window.emit("tauri_mqtt_disconnect", true).unwrap();
                }
                Ok(_) => {}
                Err(e) => {
                    println!("[tauri] Error while polling: {:?}", e);
                    window.emit("tauri_mqtt_disconnect", e.to_string()).unwrap();

                    // 发生错误退出循环前，清理 client，允许重连
                    let mut c = client_state.lock().await;
                    c.client = None;

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
    let mut client = client.lock().await;
    if let Some(ref mut mqtt_client) = client.client {
        mqtt_client.disconnect().await.map_err(|e| {
            println!("[tauri] mqtt_disconnect error{}", e);
            e.to_string()
        })?;
    }
    // 显式置空 client，允许下次连接
    client.client = None;

    println!("[tauri] mqtt_disconnect success");
    Ok(())
}
