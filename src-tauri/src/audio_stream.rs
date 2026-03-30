#![allow(clippy::needless_bool)]

use crate::loopback::{RecordParams, start_record_audio_with_writer, stop_recording};
use crate::provider_config::TranscriptRuntimeConfig;
use cpal::traits::{DeviceTrait, HostTrait};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use tauri::{AppHandle, Emitter};

pub static RECORD_HANDLE: OnceLock<Mutex<Option<std::thread::JoinHandle<()>>>> = OnceLock::new();

pub fn get_record_handle() -> &'static Mutex<Option<std::thread::JoinHandle<()>>> {
    RECORD_HANDLE.get_or_init(|| Mutex::new(None))
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SelectedAudioDevice {
    DefaultOutput,
    DefaultInput,
    NamedOutput { name: String, occurrence: usize },
    NamedInput { name: String, occurrence: usize },
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AudioChannelKind {
    Output,
    Input,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AudioChannelOption {
    pub value: String,
    pub name: String,
    pub kind: AudioChannelKind,
    pub is_default: bool,
}

fn device_display_name(device: &cpal::Device) -> Option<String> {
    device
        .description()
        .ok()
        .map(|description| description.name().to_string())
}

fn build_audio_channel_value(
    kind: AudioChannelKind,
    occurrence: Option<usize>,
    name: &str,
) -> String {
    let prefix = match kind {
        AudioChannelKind::Output => "output",
        AudioChannelKind::Input => "input",
    };

    match occurrence {
        Some(index) => format!("{prefix}:{index}:{name}"),
        None => format!("{prefix}:default:{name}"),
    }
}

fn parse_audio_channel_value(value: &str) -> Option<SelectedAudioDevice> {
    let mut parts = value.splitn(3, ':');
    let kind = parts.next()?;
    let occurrence = parts.next()?;
    let name = parts.next().unwrap_or_default().trim().to_string();

    match (kind, occurrence) {
        ("output", "default") => Some(SelectedAudioDevice::DefaultOutput),
        ("input", "default") => Some(SelectedAudioDevice::DefaultInput),
        ("output", occurrence) => occurrence
            .parse()
            .ok()
            .map(|occurrence| SelectedAudioDevice::NamedOutput { name, occurrence }),
        ("input", occurrence) => occurrence
            .parse()
            .ok()
            .map(|occurrence| SelectedAudioDevice::NamedInput { name, occurrence }),
        _ => None,
    }
}

fn parse_selected_audio_device(device_name: Option<&str>) -> SelectedAudioDevice {
    let Some(device_name) = device_name.map(str::trim).filter(|value| !value.is_empty()) else {
        return SelectedAudioDevice::DefaultOutput;
    };

    if let Some(device) = parse_audio_channel_value(device_name) {
        return device;
    }

    if device_name == "default" {
        return SelectedAudioDevice::DefaultOutput;
    }

    if device_name == "default_input" {
        return SelectedAudioDevice::DefaultInput;
    }

    if device_name.strip_suffix(" [输出] (默认)").is_some() {
        return SelectedAudioDevice::DefaultOutput;
    }

    if let Some(name) = device_name.strip_suffix(" [输出]") {
        return SelectedAudioDevice::NamedOutput {
            name: name.trim().to_string(),
            occurrence: 0,
        };
    }

    if let Some(name) = device_name.strip_suffix(" [输入]") {
        return SelectedAudioDevice::NamedInput {
            name: name.trim().to_string(),
            occurrence: 0,
        };
    }

    SelectedAudioDevice::NamedOutput {
        name: device_name.to_string(),
        occurrence: 0,
    }
}

#[tauri::command]
pub fn get_audio_stream_devices_names() -> Result<Vec<AudioChannelOption>, String> {
    let host = cpal::default_host();
    let mut channels = Vec::new();
    let mut output_occurrences = HashMap::new();
    let mut input_occurrences = HashMap::new();

    let default_output_name = host
        .default_output_device()
        .and_then(|d| device_display_name(&d));

    if let Ok(output_devices) = host.output_devices() {
        for device in output_devices {
            if let Some(name) = device_display_name(&device) {
                let occurrence = output_occurrences.entry(name.clone()).or_insert(0usize);
                let current_occurrence = *occurrence;
                *occurrence += 1;

                if Some(&name) != default_output_name.as_ref() {
                    channels.push(AudioChannelOption {
                        value: build_audio_channel_value(
                            AudioChannelKind::Output,
                            Some(current_occurrence),
                            &name,
                        ),
                        name,
                        kind: AudioChannelKind::Output,
                        is_default: false,
                    });
                }
            }
        }
    }

    if let Ok(input_devices) = host.input_devices() {
        for device in input_devices {
            if let Some(name) = device_display_name(&device) {
                let occurrence = input_occurrences.entry(name.clone()).or_insert(0usize);
                let current_occurrence = *occurrence;
                *occurrence += 1;

                channels.push(AudioChannelOption {
                    value: build_audio_channel_value(
                        AudioChannelKind::Input,
                        Some(current_occurrence),
                        &name,
                    ),
                    name,
                    kind: AudioChannelKind::Input,
                    is_default: false,
                });
            }
        }
    }

    if let Some(name) = default_output_name {
        channels.insert(
            0,
            AudioChannelOption {
                value: build_audio_channel_value(AudioChannelKind::Output, None, &name),
                name,
                kind: AudioChannelKind::Output,
                is_default: true,
            },
        );
    }

    Ok(channels)
}

#[tauri::command]
pub fn stop_recognize_audio_stream_from_speaker_loopback() {
    if let Some(handle) = get_record_handle().lock().unwrap().take() {
        stop_recording(handle);
    } else {
        println!("没有正在运行的录音线程");
    }
}

#[tauri::command]
pub fn start_recognize_audio_stream_from_speaker_loopback(
    app: AppHandle,
    device_name: Option<String>,
    selected_asr_vendor: String,
    capture_interval: u32,
    transcript_config: Option<TranscriptRuntimeConfig>,
) {
    let selected_device = parse_selected_audio_device(device_name.as_deref());
    let (device, is_input_device, device_occurrence) = match selected_device {
        SelectedAudioDevice::DefaultOutput => ("default".to_string(), false, None),
        SelectedAudioDevice::DefaultInput => ("default_input".to_string(), true, None),
        SelectedAudioDevice::NamedOutput { name, occurrence } => (name, false, Some(occurrence)),
        SelectedAudioDevice::NamedInput { name, occurrence } => (name, true, Some(occurrence)),
    };

    let last_result = Arc::new(Mutex::new(String::new()));
    let transcript_app = app.clone();
    let error_app = app.clone();
    let status_callback = Arc::new(move |message: String| {
        if let Err(err) = error_app.emit("transcription_error", message) {
            eprintln!("Failed to emit transcription error: {err}");
        }
    });
    let params = RecordParams {
        device,
        is_input_device,
        device_occurrence,
        file_name: String::new(),
        capture_interval,
        only_pcm: true,
        pcm_callback: Some(Arc::new(move |chunk: &str| {
            if !chunk.is_empty() && *last_result.lock().unwrap() != chunk {
                *last_result.lock().unwrap() = chunk.to_string();
                transcript_app.emit("transcription_result", chunk).unwrap();
            }
        })),

        use_resampled: true,
        auto_chunk_buffer: false,
        selected_asr_vendor,
        status_callback: Some(status_callback),
        transcript_config,
    };

    if let Ok(handle) = start_record_audio_with_writer(params) {
        let mut guard = get_record_handle().lock().unwrap();
        *guard = Some(handle);
        println!("录音识别已开始 ✅");
    } else {
        eprintln!("录音线程启动失败 ❌");
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AudioChannelKind, SelectedAudioDevice, build_audio_channel_value,
        parse_selected_audio_device,
    };

    #[test]
    fn parse_selected_audio_device_defaults_to_output_when_missing() {
        assert_eq!(
            parse_selected_audio_device(None),
            SelectedAudioDevice::DefaultOutput
        );
        assert_eq!(
            parse_selected_audio_device(Some("")),
            SelectedAudioDevice::DefaultOutput
        );
    }

    #[test]
    fn parse_selected_audio_device_recognizes_frontend_output_labels() {
        assert_eq!(
            parse_selected_audio_device(Some("扬声器 Realtek [输出]")),
            SelectedAudioDevice::NamedOutput {
                name: "扬声器 Realtek".to_string(),
                occurrence: 0
            }
        );
        assert_eq!(
            parse_selected_audio_device(Some("扬声器 Realtek [输出] (默认)")),
            SelectedAudioDevice::DefaultOutput
        );
    }

    #[test]
    fn parse_selected_audio_device_recognizes_frontend_input_labels() {
        assert_eq!(
            parse_selected_audio_device(Some("麦克风 USB [输入]")),
            SelectedAudioDevice::NamedInput {
                name: "麦克风 USB".to_string(),
                occurrence: 0
            }
        );
        assert_eq!(
            parse_selected_audio_device(Some("default_input")),
            SelectedAudioDevice::DefaultInput
        );
    }

    #[test]
    fn parse_selected_audio_device_recognizes_structured_output_value() {
        assert_eq!(
            parse_selected_audio_device(Some(&build_audio_channel_value(
                AudioChannelKind::Output,
                Some(2),
                "扬声器 Realtek",
            ))),
            SelectedAudioDevice::NamedOutput {
                name: "扬声器 Realtek".to_string(),
                occurrence: 2
            }
        );
        assert_eq!(
            parse_selected_audio_device(Some(&build_audio_channel_value(
                AudioChannelKind::Output,
                None,
                "扬声器 Realtek",
            ))),
            SelectedAudioDevice::DefaultOutput
        );
    }

    #[test]
    fn parse_selected_audio_device_recognizes_structured_input_value() {
        assert_eq!(
            parse_selected_audio_device(Some(&build_audio_channel_value(
                AudioChannelKind::Input,
                Some(1),
                "麦克风 USB",
            ))),
            SelectedAudioDevice::NamedInput {
                name: "麦克风 USB".to_string(),
                occurrence: 1
            }
        );
    }
}
