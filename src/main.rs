/*
 * @Author         : Shang
 * @Date           : 2024-09-19
 * @LastEditors    : Shang
 * @LastEditTime   : 2024-10-09
 * @Description    :
 */
// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
use tokio::sync::Mutex;
mod services;
use services::{http_service, mqtt_service};

fn main() {
    let mqtt_client = Arc::new(Mutex::new(mqtt_service::MqttClient::default()));
    tauri::Builder::default()
        .manage(mqtt_client)
        .invoke_handler(tauri::generate_handler![
            http_service::send_http_request,
            mqtt_service::mqtt_connect,
            mqtt_service::mqtt_publish_message,
            mqtt_service::mqtt_subscribe_topic,
            mqtt_service::mqtt_unsubscribe_topic,
            mqtt_service::mqtt_disconnect
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
