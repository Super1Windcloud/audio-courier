#![allow(clippy::collapsible_if)]

use crate::utils::write_some_log;
use serde_json::json;
use std::env;
use std::time::Duration;
use tauri::Emitter;
use tokio::time;
use tokio_stream::StreamExt;

const REQUEST_TIMEOUT_SECONDS: u64 = 3;

pub struct ModelRequest {
    pub model: String,
    pub messages: Vec<serde_json::Value>,
    pub base_url: String,
    pub api_key: String,
    pub max_tokens: u32,
    pub temperature: f32,
    pub enable_thinking: Option<bool>,
}

#[derive(Debug)]
pub enum ModelError {
    NetworkError(String),
    InvalidResponse(String),
    Timeout,
    RateLimited,
    Unauthorized(String),
    InternalServerError,
    StreamingError(String),
    JsonParseError(String),
}

impl std::fmt::Display for ModelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelError::NetworkError(msg) => write!(f, "网络连接错误: {}", msg),
            ModelError::InvalidResponse(msg) => write!(f, "服务器响应无效: {}", msg),
            ModelError::Timeout => write!(f, "请求或首包超时(3秒)"),
            ModelError::RateLimited => write!(f, "请求频率限制，请稍后重试"),
            ModelError::Unauthorized(msg) => write!(f, "API密钥无效或未授权: {}", msg),
            ModelError::InternalServerError => write!(f, "服务器内部错误"),
            ModelError::StreamingError(msg) => write!(f, "流式传输错误: {}", msg),
            ModelError::JsonParseError(msg) => write!(f, "JSON解析错误: {}", msg),
        }
    }
}

pub async fn call_model_api(
    app: tauri::AppHandle,
    req: ModelRequest,
    request_id: Option<String>,
) -> Result<String, ModelError> {
    let model_name = req.model.clone();
    let client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(REQUEST_TIMEOUT_SECONDS))
        .build()
        .map_err(|e| ModelError::NetworkError(format!("客户端创建失败: {}", e)))?;

    let mut request_body = json!({
        "model": req.model,
        "messages": req.messages,
        "temperature": req.temperature,
        "max_tokens": req.max_tokens,
        "stream": true
    });

    if let Some(enable_thinking) = req.enable_thinking {
        request_body["enable_thinking"] = json!(enable_thinking);
    }

    // 发送请求并处理基本网络错误
    let request = client
        .post(format!(
            "{}/chat/completions",
            req.base_url.trim_end_matches('/')
        ))
        .header("Authorization", format!("Bearer {}", req.api_key))
        .json(&request_body);

    let response = time::timeout(Duration::from_secs(REQUEST_TIMEOUT_SECONDS), request.send())
        .await
        .map_err(|_| ModelError::Timeout)?
        .map_err(|e| {
            if e.is_timeout() {
                ModelError::Timeout
            } else if e.is_connect() {
                ModelError::NetworkError(format!("连接失败: {}", e))
            } else if e.is_request() {
                ModelError::NetworkError(format!("请求发送失败: {}", e))
            } else {
                ModelError::NetworkError(format!("网络请求失败: {}", e))
            }
        })?;

    // 检查HTTP状态码
    let status = response.status();
    if !status.is_success() {
        let error_body = response
            .text()
            .await
            .unwrap_or_else(|_| "无法读取错误响应体".to_string());
        return Err(match status.as_u16() {
            401 => ModelError::Unauthorized(format!(
                "model={model_name}, HTTP状态码: {status}, 响应体: {error_body}"
            )),
            429 => ModelError::RateLimited,
            500..=599 => ModelError::InternalServerError,
            _ => ModelError::InvalidResponse(format!(
                "model={model_name}, HTTP状态码: {status}, 响应体: {error_body}"
            )),
        });
    }

    // 检查响应内容类型
    if let Some(content_type) = response.headers().get("content-type") {
        if !content_type.to_str().unwrap_or("").contains("text/plain")
            && !content_type
                .to_str()
                .unwrap_or("")
                .contains("text/event-stream")
        {
            return Err(ModelError::InvalidResponse(format!(
                "model={model_name}, 响应不是流式格式"
            )));
        }
    }

    let mut result = String::new();
    let mut stream = response.bytes_stream();
    let mut consecutive_errors = 0;
    const MAX_CONSECUTIVE_ERRORS: usize = 5;

    // 确定事件名称 - 如果有请求ID则使用带ID的事件名
    let event_name = if let Some(id) = &request_id {
        format!("llm_stream_{}", id)
    } else {
        "llm_stream".to_string()
    };

    loop {
        let item = if result.is_empty() {
            match time::timeout(Duration::from_secs(REQUEST_TIMEOUT_SECONDS), stream.next()).await {
                Ok(item) => item,
                Err(_) => return Err(ModelError::Timeout),
            }
        } else {
            stream.next().await
        };

        let Some(item) = item else {
            break;
        };

        match item {
            Ok(chunk) => {
                consecutive_errors = 0; // 重置错误计数

                let chunk_str = String::from_utf8_lossy(&chunk);

                // 处理空数据块
                if chunk_str.trim().is_empty() {
                    continue;
                }

                // 解析SSE格式数据
                for line in chunk_str.lines() {
                    let line = line.trim();

                    if line.is_empty() || line.starts_with(':') {
                        continue; // 跳过空行和注释行
                    }

                    if let Some(data) = line.strip_prefix("data: ") {
                        if data == "[DONE]" {
                            return Ok(result);
                        }

                        // JSON解析错误处理
                        match serde_json::from_str::<serde_json::Value>(data) {
                            Ok(json_chunk) => {
                                // 检查是否有错误信息
                                if let Some(error) = json_chunk.get("error") {
                                    return Err(ModelError::InvalidResponse(format!(
                                        "model={model_name}, API错误: {}",
                                        error
                                    )));
                                }

                                // 提取内容
                                if let Some(content) =
                                    json_chunk["choices"][0]["delta"]["content"].as_str()
                                {
                                    result.push_str(content);

                                    // 发送流式数据到前端，处理发送错误
                                    if let Err(e) = app.emit(&event_name, content) {
                                        eprintln!("警告: 无法发送流式数据到前端: {}", e);
                                        write_some_log(
                                            format!(" 无法发送流式数据到前端: {}", e).as_str(),
                                        )
                                    }
                                }
                            }
                            Err(e) => {
                                // JSON解析失败，可能是部分数据，记录但继续
                                eprintln!("JSON解析警告: {} (数据: {})", e, data);
                                consecutive_errors += 1;

                                if consecutive_errors >= MAX_CONSECUTIVE_ERRORS {
                                    return Err(ModelError::JsonParseError(format!(
                                        "连续JSON解析失败次数过多: {}",
                                        e
                                    )));
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                consecutive_errors += 1;
                eprintln!("流数据接收错误: {}", e);

                if consecutive_errors >= MAX_CONSECUTIVE_ERRORS {
                    return Err(ModelError::StreamingError(format!(
                        "连续流错误次数过多: {}",
                        e
                    )));
                }
            }
        }
    }

    // 流意外结束
    if result.is_empty() {
        Err(ModelError::StreamingError("流数据为空".to_string()))
    } else {
        Ok(result)
    }
}

pub fn get_env_key(key_name: &str) -> String {
    env::var(key_name).unwrap_or_else(|_| {
        eprintln!("环境变量 {} 未设置，请设置后重试", key_name);
        std::process::exit(1);
    })
}
