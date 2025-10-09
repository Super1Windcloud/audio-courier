import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export async function startAudioLoopbackRecognition(
  onMessageCapture: (message: string) => void,
) {
  let content = "";
  const unlistener = await listen<string>("transcription_result", (event) => {
    console.log("识别扬声器结果:", event.payload);
    content += event.payload;
    onMessageCapture(content);
  });

  const errorListener = await listen<string>("transcription_error", (event) => {
    console.error("Audio Error :", event.payload);
  });

  invoke("start_recognize_audio_stream_from_speaker_loopback")
    .catch((err) => {
      console.error("invoke start output audio recognition failed", err);
    })
    .finally(() => {
      unlistener();
      errorListener();
    });
}

export async function stopAudioLoopbackRecognition() {
  await invoke("stop_recognize_audio_stream_from_speaker_loopback").catch(
    (err) => {
      console.error("invoke stop output audio recognition failed", err);
    },
  );
}
