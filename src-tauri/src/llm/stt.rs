use crate::llm::api::get_env_key;
use anyhow::{anyhow, Context, Result};
use reqwest::multipart::{Form, Part};
use reqwest::Client;
use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Deserialize, Debug)]
struct TranscriptionResponse {
    text: String,
}

#[allow(dead_code)]
async fn realtime_audio_transcription(i: usize) -> Result<()> {
    let api_key = get_env_key("Siliconflow");

    let audio_path = concat!(env!("CARGO_MANIFEST_DIR"), "/record.wav");
    if !Path::new(audio_path).exists() {
        return Err(anyhow!("音频文件不存在: {}", audio_path));
    }
    let audio_models = ["TeleAI/TeleSpeechASR", "FunAudioLLM/SenseVoiceSmall"];
    // let random_model = audio_models[rng().random_range(..audio_models.len())];
    let random_model = audio_models[i];

    let resp = transcribe_file(api_key.as_str(), audio_path, random_model).await?;
    println!("模型：{}", random_model);
    println!("识别结果：\n{}", resp.text);

    Ok(())
}

#[allow(dead_code)]
async fn transcribe_file(
    api_key: &str,
    audio_path: &str,
    model: &str,
) -> Result<TranscriptionResponse> {
    let url = "https://api.siliconflow.cn/v1/audio/transcriptions";

    // 读取音频文件到内存（对于大文件你可能要改成流式上传）
    let path = Path::new(audio_path);
    let file_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow!("无法从路径获取文件名"))?;

    let mut file = File::open(path).with_context(|| format!("打开文件失败: {}", audio_path))?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).with_context(|| "读取文件失败")?;

    // 尝试猜测 mime type，简单处理：以扩展名判断，必要时可用 mime_guess crate
    let mime = if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        match ext.to_lowercase().as_str() {
            "wav" => "audio/wav",
            "mp3" => "audio/mpeg",
            "m4a" => "audio/mp4",
            "flac" => "audio/flac",
            "ogg" => "audio/ogg",
            _ => "application/octet-stream",
        }
    } else {
        "application/octet-stream"
    };

    let part = Part::bytes(buf)
        .file_name(file_name.to_string())
        .mime_str(mime)
        .with_context(|| "构建 multipart file part 失败")?;

    // model 字段为普通文本 part
    let form = Form::new()
        .part("file", part)
        .text("model", model.to_string());

    let client = Client::new();
    let res = client
        .post(url)
        .bearer_auth(api_key)
        .multipart(form)
        .send()
        .await
        .with_context(|| "HTTP 请求发送失败")?;

    let status = res.status();
    let text = res.text().await.with_context(|| "读取响应体失败")?;

    if !status.is_success() {
        return Err(anyhow!(
            "API 返回错误: status={} body={}",
            status.as_u16(),
            text
        ));
    }

    // 解析 JSON
    let parsed: TranscriptionResponse =
        serde_json::from_str(&text).with_context(|| format!("解析 JSON 失败: {}", text))?;

    Ok(parsed)
}

///cargo   test --package audio-courier --lib llm::stt::test_transcribe_file -- --exact  --nocapture
#[tokio::test]
async fn test_transcribe_file() {
    use dotenv::dotenv;
    dotenv().ok();
    for i in 0..2 {
        let start = std::time::Instant::now();
        realtime_audio_transcription(i as usize).await.unwrap();
        let end = std::time::Instant::now();
        println!("第{}次耗时：{:?}", i + 1, end.duration_since(start));
    }
}
