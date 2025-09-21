use serde_json::json;
use std::env;
use tauri::Emitter;
use tokio_stream::StreamExt;

pub struct ModelRequest {
    pub model: String,
    pub messages: Vec<serde_json::Value>,
    pub base_url: String,
    pub api_key: String,
    pub max_tokens: u32,
    pub temperature: f32,
}

pub async fn call_model_api(app: tauri::AppHandle, req: ModelRequest) -> Result<String, String> {
    let client = reqwest::Client::new();

    let response = client
        .post(format!("{}/chat/completions", req.base_url))
        .header("Authorization", format!("Bearer {}", req.api_key))
        .json(&json!({
            "model": req.model,
            "messages": req.messages,
            "temperature": req.temperature,
            "max_tokens": req.max_tokens,
            "stream": true
        }))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let mut result = String::new();
    let mut stream = response.bytes_stream();

    while let Some(item) = stream.next().await {
        match item {
            Ok(chunk) => {
                let chunk_str = String::from_utf8_lossy(&chunk);
                // OpenAI / DashScope 的 SSE 格式里每一行可能是: data: {...}
                for line in chunk_str.lines() {
                    if line.starts_with("data: ") {
                        let data = &line[6..]; // 去掉 "data: "

                        if data == "[DONE]" {
                            break;
                        }

                        if let Ok(json_chunk) = serde_json::from_str::<serde_json::Value>(data) {
                            if let Some(content) =
                                json_chunk["choices"][0]["delta"]["content"].as_str()
                            {
                                result.push_str(content);
                                app.emit("llm_stream", content).unwrap();
                            }
                        }
                    }
                }
            }
            Err(e) => return Err(e.to_string()),
        }
    }

    Ok(result)
}

pub fn get_env_key(key_name: &str) -> String {
    env::var(key_name).unwrap_or_else(|_| panic!("{} 未在 .env 中配置", key_name))
}
