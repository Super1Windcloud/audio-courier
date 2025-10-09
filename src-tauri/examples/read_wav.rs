use std::env;

use hound::WavReader;
use vosk::{Model, Recognizer};

fn main() {
    let model_path = concat!(env!("CARGO_MANIFEST_DIR"), "/vosk-model-small-cn-0.22");
    let wav_path = concat!(env!("CARGO_MANIFEST_DIR"), "/recorded.wav");

    let mut reader = WavReader::open(wav_path).expect("Could not create the WAV reader");
    let samples = reader
        .samples()
        .collect::<hound::Result<Vec<i16>>>()
        .expect("Could not read WAV file");

    let model = Model::new(model_path).expect("Could not create the model");
    let mut recognizer = Recognizer::new(&model, reader.spec().sample_rate as f32)
        .expect("Could not create the recognizer");

    recognizer.set_max_alternatives(10);
    recognizer.set_words(true);
    recognizer.set_partial_words(true);

    for sample in samples.chunks(100) {
        recognizer.accept_waveform(sample).unwrap();
        println!("{:#?}", recognizer.partial_result());
    }

    println!("{:#?}", recognizer.final_result().multiple().unwrap());
}
