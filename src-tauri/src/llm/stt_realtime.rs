use crate::PcmCallback;
use base64::{engine::general_purpose, Engine as _};
use chrono::offset::FixedOffset;
use chrono::Local;
use futures_util::{SinkExt, StreamExt};
use hmac::{Hmac, KeyInit, Mac};
use serde_json::json;
use serde_json::Value;
use sha1::Sha1;
use std::time::{Duration, Instant};
use std::{fs::File, io::Read, path::Path};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tungstenite::client::IntoClientRequest;
use uuid::Uuid;

type HmacSha1 = Hmac<Sha1>;

const SAMPLE_RATE: u64 = 16000 ;
const AUDIO_FRAME_SIZE: usize = (SAMPLE_RATE * FRAME_INTERVAL_MS / 1000) as usize;
const FRAME_INTERVAL_MS: u64 = 40;

struct FixedParams {
    audio_encode: String,
    lang: String,
    samplerate: String,
}

pub struct RTASRClient {
    app_id: String,
    access_key_id: String,
    access_key_secret: String,
    pub(crate) base_ws_url: String,
    fixed_params: FixedParams,
}

impl RTASRClient {
    pub fn new(app_id: &str, access_key_id: &str, access_key_secret: &str) -> Self {
        rustls::crypto::ring::default_provider()
            .install_default()
            .expect("Failed to install default rustls crypto provider");

        Self {
            app_id: app_id.to_string(),
            access_key_id: access_key_id.to_string(),
            access_key_secret: access_key_secret.to_string(),
            base_ws_url: "wss://office-api-ast-dx.iflyaisol.com/ast/communicate/v1".into(),
            fixed_params: FixedParams {
                audio_encode: "pcm_s16le".into(),
                lang: "autodialect".into(),
                samplerate: "16000".into(),
            },
        }
    }

    fn utc_time_str(&self) -> String {
        let offset = FixedOffset::east_opt(8 * 3600).unwrap();
        Local::now()
            .with_timezone(&offset)
            .format("%Y-%m-%dT%H:%M:%S%z")
            .to_string()
    }

    pub(crate) fn generate_auth_params(&self) -> Vec<(String, String)> {
        let mut params = vec![
            ("accessKeyId".to_string(), self.access_key_id.clone()),
            ("appId".to_string(), self.app_id.clone()),
            ("uuid".to_string(), Uuid::new_v4().to_string()),
            ("utc".to_string(), self.utc_time_str()),
            (
                "audio_encode".to_string(),
                self.fixed_params.audio_encode.clone(),
            ),
            ("lang".to_string(), self.fixed_params.lang.clone()),
            (
                "samplerate".to_string(),
                self.fixed_params.samplerate.clone(),
            ),
        ];

        params.sort_by(|a, b| a.0.cmp(&b.0));

        let base_str = params
            .iter()
            .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        let mut mac = HmacSha1::new_from_slice(self.access_key_secret.as_bytes()).unwrap();
        mac.update(base_str.as_bytes());
        let signature = general_purpose::STANDARD.encode(mac.finalize().into_bytes());

        params.push(("signature".to_string(), signature));
        params
    }

    pub fn extract_text_from_asr_result_fixed(content: &str) -> Result<String, serde_json::Error> {
        let json_value: Value = serde_json::from_str(content)?;

        let mut text = String::new();

        // 2. 尝试将 Value 转换为数组 Array，如果不是数组则跳过或返回错误
        if let Some(segments) = json_value.as_array() {
            for segment in segments {
                if let Some(words_array) = segment.get("ws").and_then(Value::as_array) {
                    for word in words_array {
                        if let Some(word_text) = word.get("w").and_then(Value::as_str) {
                            text.push_str(word_text);
                        }
                    }
                }
            }
        }

        Ok(text)
    }

    pub async fn run_ws_loop(
        &self,
        mut rx: tokio::sync::mpsc::Receiver<Vec<i16>>,
        pcm_callback: Option<PcmCallback>,
    ) {
        let auth_params = self.generate_auth_params();
        let url = format!(
            "{}?{}",
            self.base_ws_url,
            serde_urlencoded::to_string(auth_params).unwrap()
        );
        let (ws_stream, _) = connect_async(url).await.unwrap();
        println!("✅ WebSocket 连接成功");

        let (mut write, mut read) = ws_stream.split();

        // 启动接收线程
        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                if let Ok(Message::Text(txt)) = msg {
                    if let Ok(text) = RTASRClient::extract_text_from_asr_result_fixed(&txt) {
                        if let Some(cb) = &pcm_callback {
                            cb(&text);
                        }
                    }
                }
            }
        });

        // 循环接收 PCM 数据发送给 WebSocket
        while let Some(chunk) = rx.recv().await {
            let bytes = chunk
                .iter()
                .flat_map(|&x| x.to_le_bytes())
                .collect::<Vec<u8>>();

            if let Err(e) = write.send(Message::Binary(bytes.into())).await {
                eprintln!("发送错误: {:?}", e);
                break;
            }
        }

        // 发送结束标志
        let end_msg = json!({"end": true});
        let _ = write.send(Message::Text(end_msg.to_string().into())).await;
    }

    pub async fn connect_and_send_audio<P: AsRef<Path>>(
        &self,
        audio_path: Option<P>,
        buffer: Option<Vec<i16>>,
    ) -> anyhow::Result<()> {
        let auth_params = self.generate_auth_params();
        let url = format!(
            "{}?{}",
            self.base_ws_url,
            serde_urlencoded::to_string(auth_params)?
        );

        let url = url.into_client_request()?;
        let (ws_stream, _) = connect_async(url).await?;
        println!("✅ WebSocket 连接成功");

        let (mut write, mut read) = ws_stream.split();

        let mut text = String::new();
        // ✅ 启动接收线程（异步并行打印识别结果）
        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(txt)) => {
                        if let Ok(json) = serde_json::from_str::<Value>(&txt) {
                            if let Some(rt_array) = json
                                .get("data")
                                .and_then(|d| d.get("cn"))
                                .and_then(|cn| cn.get("st"))
                                .and_then(|st| st.get("rt"))
                                .and_then(|rt| rt.as_array())
                            {
                                for rt_item in rt_array {
                                    if let Some(ws_array) =
                                        rt_item.get("ws").and_then(|ws| ws.as_array())
                                    {
                                        for ws_item in ws_array {
                                            if let Some(cw_array) =
                                                ws_item.get("cw").and_then(|cw| cw.as_array())
                                            {
                                                for cw in cw_array {
                                                    if let Some(word) =
                                                        cw.get("w").and_then(|w| w.as_str())
                                                    {
                                                        text.push_str(word);
                                                        println!("🟢 实时结果: {}", text);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            } else {
                                println!("No 'cn' field in this message: {:?}", json);
                            }
                        } else {
                            println!("⚠️ 无法解析文本: {}", txt);
                        }
                    }
                    Ok(Message::Binary(_)) => {}
                    Err(e) => {
                        eprintln!("接收错误: {:.?}", e);
                        break;
                    }
                    _ => {}
                }
            }
            println!("📴 接收线程结束");
        });

        // ✅ 读取音频
        let audio_bytes: Vec<u8> = if let Some(path) = audio_path {
            let mut f = File::open(path)?;
            let mut buf = Vec::new();
            f.read_to_end(&mut buf)?;
            buf
        } else if let Some(buf_i16) = buffer {
            buf_i16.iter().flat_map(|&x| x.to_le_bytes()).collect()
        } else {
            anyhow::bail!("必须提供音频路径或缓冲区");
        };

        let total_frames = audio_bytes.len().div_ceil(AUDIO_FRAME_SIZE);
        println!("🔊 开始发送音频，共 {} 帧", total_frames);

        // ✅ 实时发送音频帧
        let start_time = Instant::now();
        for (i, chunk) in audio_bytes.chunks(AUDIO_FRAME_SIZE).enumerate() {
            let expected = Duration::from_millis(i as u64 * FRAME_INTERVAL_MS);
            let elapsed = start_time.elapsed();
            if expected > elapsed {
                tokio::time::sleep(expected - elapsed).await;
            }

            if let Err(e) = write.send(Message::Binary(chunk.to_vec().into())).await {
                eprintln!("发送错误: {:.?}", e);
                break;
            }

            if i % 10 == 0 {
                println!("📤 已发送第 {} 帧", i);
            }
        }

        // ✅ 发送结束标志
        let end_msg = json!({"end": true});
        write
            .send(Message::Text(end_msg.to_string().into()))
            .await?;
        println!("✅ 已发送结束标记");

        Ok(())
    }
}
