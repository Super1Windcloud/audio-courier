import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export async function startAudioLoopbackRecognition(
  onMessageCapture: (message: string) => void,
  audioDevice: string,
  captureInterval: number,
) {
  let content = "";
  const unlistener = await listen<string>("transcription_result", (event) => {
    console.log("识别扬声器结果:", event.payload);
    content = event.payload.replace(/\s/g, "");
    onMessageCapture(content);
  });

  invoke("start_recognize_audio_stream_from_speaker_loopback", {
    deviceName: audioDevice,
    captureInterval,
  }).catch((err) => {
    console.error("invoke start output audio recognition failed", err);
    unlistener();
  });
}

export async function stopAudioLoopbackRecognition() {
  await clearVoskAcceptBuffer();
  await invoke("stop_recognize_audio_stream_from_speaker_loopback").catch(
    (err) => {
      console.error("invoke stop output audio recognition failed", err);
    },
  );
}

export async function clearVoskAcceptBuffer() {
  await invoke("clear_vosk_accept_buffer").catch((err) => {
    console.error("invoke clear vosk buffer failed", err);
  });
}
