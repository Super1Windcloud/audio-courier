use hound::WavReader;
use std::env;
use std::path::PathBuf;
use vosk::{Model, Recognizer};

fn main() {
    let model_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("vosk-model-small-cn-0.22");
    let model_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("vosk-model-cn-0.22");
    let model_path = model_path.to_str().unwrap();
    //Resample Time elapsed: 9.335443s
    // Normal Time elapsed 11.8

    let wav_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("recorded_i16_mono.wav");
    let wav_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("recorded_i16_mono_resample.wav");

    let wav_path = wav_path.to_str().unwrap();
    let mut reader =
        WavReader::open(wav_path).expect(format!("Could not open wav file: {}", wav_path).as_str());
    let samples = reader
        .samples()
        .collect::<hound::Result<Vec<_>>>()
        .unwrap_or_else(|_| panic!("Could not read WAV file samples: {}", wav_path));

    let sample_rate = reader.spec();
    println!("Sample rate: {:?}", sample_rate);

    let model = Model::new(model_path).expect("Could not create the model");
    let mut recognizer = Recognizer::new(&model, reader.spec().sample_rate as f32)
        .expect("Could not create the recognizer");

    recognizer.set_max_alternatives(1);
    recognizer.set_partial_words(false);
    recognizer.set_words(false);

    let start = std::time::Instant::now();
    for sample in samples.chunks((sample_rate.sample_rate) as usize) {
        recognizer.accept_waveform(sample).unwrap();
        let recognition = recognizer.partial_result();
        let content = recognition.partial;
        println!("Partial result: {}", content);
    }
    println!(
        "{:#?}",
        recognizer.final_result().multiple().unwrap().alternatives[0].text
    );
    println!("Time elapsed: {}s", start.elapsed().as_secs_f32());
}
