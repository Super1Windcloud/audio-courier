use hound::WavReader;
use std::env;
use vosk::{Model, Recognizer};

fn main() {
    let model_path = concat!(env!("CARGO_MANIFEST_DIR"), "\\vosk-model-small-cn-0.22");
    let wav_path = concat!(env!("CARGO_MANIFEST_DIR"), "\\recorded_i16_mono.wav");
    let mut reader = WavReader::open(wav_path).expect("Could not create the WAV reader");
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
