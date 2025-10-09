use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample};
use std::fs::File;
use std::io::BufWriter;
use std::sync::{Arc, Mutex};
pub struct RecordParams {
    pub device: String,
    pub duration: u64,
}

pub fn record_audio(params: RecordParams) -> Result<(), String> {
    let host = cpal::default_host();

    let device = match params.device.as_str() {
        "default" => host.default_output_device(),
        "default-input" => host.default_input_device(),
        name => host
            .input_devices()
            .unwrap()
            .find(|x| x.name().map(|y| y == name).unwrap_or(false)),
    }
    .ok_or_else(|| "failed to find input device".to_string())?;

    println!("Input device: {}", device.name().unwrap());

    let config = if device.supports_input() {
        device.default_input_config()
    } else {
        device.default_output_config()
    }
    .map_err(|_| "Failed to get default input/output config".to_string())?;
    println!("Default input/output config: {config:?}");

    const PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/recorded.wav");
    let spec = wav_spec_from_config(&config);
    let writer = hound::WavWriter::create(PATH, spec)
        .map_err(|e| format!("Failed to create WAV writer: {e}"))?;
    let writer = Arc::new(Mutex::new(Some(writer)));

    println!("Begin recording...");

    let writer_2 = writer.clone();

    let err_fn = move |err| {
        eprintln!("An error occurred on stream: {err}");
    };

    let stream = match config.sample_format() {
        cpal::SampleFormat::I8 => device
            .build_input_stream(
                &config.into(),
                move |data, _: &_| write_input_data::<i8, i8>(data, &writer_2),
                err_fn,
                None,
            )
            .unwrap(),
        cpal::SampleFormat::I16 => device
            .build_input_stream(
                &config.into(),
                move |data, _: &_| write_input_data::<i16, i16>(data, &writer_2),
                err_fn,
                None,
            )
            .unwrap(),
        cpal::SampleFormat::I32 => device
            .build_input_stream(
                &config.into(),
                move |data, _: &_| write_input_data::<i32, i32>(data, &writer_2),
                err_fn,
                None,
            )
            .unwrap(),
        cpal::SampleFormat::F32 => device
            .build_input_stream(
                &config.into(),
                move |data, _: &_| write_input_data::<f32, f32>(data, &writer_2),
                err_fn,
                None,
            )
            .unwrap(),
        sample_format => return Err(format!("Unsupported sample format '{sample_format}'")),
    };

    stream
        .play()
        .map_err(|e| format!("Failed to play stream: {e}"))?;

    std::thread::sleep(std::time::Duration::from_secs(params.duration));
    drop(stream);
    writer
        .lock()
        .unwrap()
        .take()
        .unwrap()
        .finalize()
        .map_err(|e| format!("Failed to finalize WAV file: {e}"))?;
    println!("Recording complete! Saved to {PATH}");

    Ok(())
}

fn sample_format(format: cpal::SampleFormat) -> hound::SampleFormat {
    if format.is_float() {
        hound::SampleFormat::Float
    } else {
        hound::SampleFormat::Int
    }
}

fn wav_spec_from_config(config: &cpal::SupportedStreamConfig) -> hound::WavSpec {
    hound::WavSpec {
        channels: config.channels() as _,
        sample_rate: config.sample_rate().0 as _,
        bits_per_sample: (config.sample_format().sample_size() * 8) as _,
        sample_format: sample_format(config.sample_format()),
    }
}

type WavWriterHandle = Arc<Mutex<Option<hound::WavWriter<BufWriter<File>>>>>;

fn write_input_data<T, U>(input: &[T], writer: &WavWriterHandle)
where
    T: Sample,
    U: Sample + hound::Sample + FromSample<T>,
{
    if let Ok(mut guard) = writer.try_lock() {
        if let Some(writer) = guard.as_mut() {
            for &sample in input.iter() {
                let sample: U = U::from_sample(sample);
                writer.write_sample(sample).ok();
            }
        }
    }
}

fn main() {
    let params = RecordParams {
        device: String::from("default"),
        duration: 10,
    };

    if let Err(e) = record_audio(params) {
        eprintln!("Error: {e}");
    }
}
