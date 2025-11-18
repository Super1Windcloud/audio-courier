use deepgram::{
    Deepgram, DeepgramError,
    common::{
        audio_source::AudioSource,
        options::{Language, Options},
    },
};
use dotenv::dotenv;
use std::time::Instant;
use std::{env, fs};

static PATH_TO_FILE: &str = "assets/recorded_i16_mono_resample.wav";

#[tokio::main]
async fn main() -> Result<(), DeepgramError> {
    dotenv().ok();
    let start = Instant::now();
    let deepgram_api_key =
        env::var("DEEPGRAM_API_KEY_GITHUB").expect("DEEPGRAM_API_KEY environmental variable");

    let dg_client = Deepgram::new(&deepgram_api_key)?;

    let bytes = fs::read(PATH_TO_FILE)?;
    let source = AudioSource::from_buffer_with_mime_type(bytes, "audio/wav");

    let options = Options::builder()
        .punctuate(true)
        .language(Language::zh_CN)
        .build();

    let response = dg_client
        .transcription()
        .prerecorded(source, &options)
        .await?;

    let transcript = &response.results.channels[0].alternatives[0].transcript;
    println!("{transcript}");
    println!("Cost Time :{:?}", start.elapsed());
    Ok(())
}
