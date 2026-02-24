/*
 * @Author         : Shang
 * @Date           : 2024-09-19
 * @LastEditors    : Shang
 * @LastEditTime   : 2024-09-20
 * @Description    :
 */
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::{Client, Method};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;
use tauri::http::method::InvalidMethod;

#[derive(Clone, Serialize, Deserialize)]
pub struct RequestConfig {
    url: String,
    method: Option<String>,
    headers: Option<HashMap<String, String>>,
    params: Option<HashMap<String, String>>,
    data: Option<String>,
    timeout: Option<u64>,
}

#[tauri::command]
pub async fn send_http_request(config: RequestConfig) -> Result<String, String> {
    let client = Client::new();

    let method_str = config
        .method
        .unwrap_or_else(|| "GET".to_string())
        .to_uppercase();
    match Method::from_str(&method_str) {
        Ok(method) => {
            // 成功处理
            println!("Valid method: {:?}", method);
        }
        Err(_) => {
            return Err("Invalid HTTP method".to_string());
        }
    }
    let method: Result<Method, InvalidMethod> = Method::from_str(&method_str);
    if let Err(_) = method {
        return Err("Invalid HTTP method".to_string());
    }

    // 将 HashMap<String, String> 转换为 HeaderMap
    let mut headers = HeaderMap::new();
    if let Some(hdrs) = config.headers {
        for (key, value) in hdrs {
            let header_name = HeaderName::from_bytes(key.as_bytes()).map_err(|e| e.to_string())?;
            let header_value = HeaderValue::from_str(&value).map_err(|e| e.to_string())?;
            headers.insert(header_name, header_value);
        }
    }

    // 构建请求
    let request = client
        .request(method.unwrap(), &config.url)
        .headers(headers)
        .timeout(Duration::from_millis(config.timeout.unwrap_or(30000)))
        .query(&config.params.unwrap_or_default())
        .body(config.data.unwrap_or_default());
    // 处理网络请求错误
    match request.send().await {
        Ok(response) => match response.text().await {
            Ok(body) => Ok(body),
            Err(e) => Err(format!("Failed to read response body: {}", e)),
        },
        Err(e) => Err(format!("HTTP request failed: {}", e)),
    }
}
