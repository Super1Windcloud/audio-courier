use tauri_courier_ai_lib::{RTASRClient, ACCESS_KEY_ID, ACCESS_KEY_SECRET, APP_ID};

#[tokio::main]
async fn main() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/recorded_i16_mono_resample.wav"
    );
    let client = RTASRClient::new(APP_ID, ACCESS_KEY_ID, ACCESS_KEY_SECRET);

    client
        .connect_and_send_audio(Some(path), None)
        .await
        .unwrap();
}
