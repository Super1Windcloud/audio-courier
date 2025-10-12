use cpal::traits::{DeviceTrait, HostTrait};
use tauri_courier_ai_lib::{
    clear_vosk_accept_buffer, get_record_handle, start_record_audio_with_writer, stop_recording,
    RecordParams,
};

fn main() {
    let device = "default";
    let params = RecordParams {
        device: device.to_string(),
        file_name: "".to_string(),
        only_pcm: true,
        capture_interval: 2,
        pcm_callback: Some(Box::new(move |chunk: &str| println!("{:?}", chunk))),
        use_drain_chunk_buffer: true,
        use_big_model: true,
    };

    if let Ok(handle) = start_record_audio_with_writer(params) {
        let mut guard = get_record_handle().lock().unwrap();
        *guard = Some(handle);
        println!("录音识别已开始 ✅");
    } else {
        eprintln!("录音线程启动失败 ❌");
    }
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    clear_vosk_accept_buffer();

    std::io::stdin().read_line(&mut input).unwrap();
    if let Some(handle) = get_record_handle().lock().unwrap().take() {
        stop_recording(handle);
    } else {
        println!("没有正在运行的录音线程");
    }
    println!("录音识别已停止");
}



#[allow(dead_code)]
fn select_input_config() -> Result<cpal::StreamConfig, String> {
    let device = cpal::default_host()
        .default_output_device()
        .ok_or("没有可用的输出设备")?;
    let input_device = cpal::default_host()
        .default_input_device()
        .ok_or("没有可用的输入设备")?;
    let supported_configs = device
        .supported_output_configs()
        .map_err(|_| "无法获取输入设备配置".to_string())?;
    {
        println!("默认输出");
        println!("{:?}", device.default_output_config().unwrap());
        println!("默认输入");
        println!("{:?}", input_device.default_input_config().unwrap());
    }
    println!("输出设备支持的配置：");

    let desired_sample_rate = cpal::SampleRate(16000);

    let mut best_config = None;
    for range in supported_configs {
        println!("{:?}", range);
        if range.min_sample_rate() <= desired_sample_rate
            && range.max_sample_rate() >= desired_sample_rate
        {
            best_config = Some(range.with_sample_rate(desired_sample_rate).config());
            break;
        }
    }
    let device = cpal::default_host()
        .default_input_device()
        .ok_or("没有可用的输出设备")?;

    let support_input = device
        .supported_input_configs()
        .map_err(|_| "无法获取输入设备配置".to_string())?;
    println!("输入设备支持的配置：");

    for range in support_input {
        println!("{:?}", range);
    }

    if let Some(config) = best_config {
        println!("选择输出设备配置：{:?}", config);
        Ok(config)
    } else {
        let fallback = device
            .default_output_config()
            .map_err(|_| "没有可用的输入配置".to_string())?;
        Ok(fallback.config())
    }
}

#[test]
fn output_device_config() {
    select_input_config().unwrap();
}
