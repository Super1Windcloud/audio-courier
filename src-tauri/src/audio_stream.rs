use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::Mutex;
use std::time::Duration;
use tauri::{Emitter, State};


pub struct AudioState {
    pub stream: Mutex<Option<cpal::Stream>>,
}

#[tauri::command]
pub async fn stop_recognize_audio_stream(state: State<'_, AudioState>) -> Result<(), String> {
    let mut guard = state.stream.lock().unwrap();
    if let Some(stream) = guard.take() {
        drop(stream); // 直接丢弃，音频流就会自动停止
        Ok(())
    } else {
        Err("没有正在运行的音频流".into())
    }
}

#[tauri::command]
pub async fn start_recognize_audio_stream(
    app: tauri::AppHandle,
    state: State<'_, AudioState>,
) -> Result<(), String> {
    let host = cpal::default_host();

    let device = host.default_output_device().ok_or("没有找到默认输出设备")?;
    println!("捕获设备: {}", device.name().unwrap());

    let config = device.default_output_config().unwrap();

    let stream = device
        .build_input_stream(
            &config.into(),
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                // 这里就是系统音频的 PCM 数据 (f32 格式)
                // 下一步：送到识别引擎
                let fake_text = "模拟识别: hello world";
                let _ = app.emit("transcribed", fake_text);
            },
            move |err| {
                eprintln!("音频流错误: {:?}", err);
            },
            Some(Duration::from_millis(10000)),
        )
        .unwrap();

    stream.play().unwrap();
    *state.stream.lock().unwrap() = Some(stream);

    Ok(())
}

#[tauri::command]
pub fn get_audio_stream_devices_name() -> Vec<String> {
    let host = cpal::default_host();

    let devices = host.output_devices().unwrap();

    let default = host
        .default_output_device()
        .ok_or("没有找到默认输出设备")
        .unwrap();

    let mut names = vec![];
    for device in devices {
        names.push(device.name().unwrap().to_string());
    }

    names
        .iter()
        .position(|x| *x == default.name().unwrap())
        .map(|x| names.remove(x));

    names.insert(0, default.name().unwrap().to_string());

    names
}

#[test]
fn test_get_audio_stream_channel() {
    let host = cpal::default_host();

    // 获取默认输出设备
    if let Some(default_out) = host.default_output_device() {
        println!("默认输出设备: {}", default_out.name().unwrap());
    } else {
        println!("没有找到默认输出设备");
    }
    println!("所有输出设备:");
    for device in host.output_devices().unwrap() {
        println!("  - {}", device.name().unwrap());
    }

    let devices = host.output_devices().unwrap();

    for device in devices {
        println!("设备: {}", device.name().unwrap());

        if let Ok(formats) = device.supported_output_configs() {
            for format in formats {
                println!(
                    "  min: {:?}, max: {:?}, channels: {}, sample_format: {:?}",
                    format.min_sample_rate(),
                    format.max_sample_rate(),
                    format.channels(),
                    format.sample_format()
                );
            }
        }
    }
}
