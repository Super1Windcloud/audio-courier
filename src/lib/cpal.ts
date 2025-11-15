import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import useAppStateStore from "@/stores";
import { toast } from "sonner";

let unlistener: UnlistenFn | null = null;
let errorUnlistener: UnlistenFn | null = null;

export async function startAudioLoopbackRecognition(
  onMessageCapture: (message: string) => void,
  audioDevice: string,
  selectedAsrVendor: string,
  captureInterval: number,
) {
  if (unlistener) {
    unlistener();
    unlistener = null;
  }
  if (errorUnlistener) {
    errorUnlistener();
    errorUnlistener = null;
  }

  unlistener = await listen<string>("transcription_result", (event) => {
    const content = event.payload;
    console.log("识别扬声器结果:", content);
    onMessageCapture(content);
  });
  errorUnlistener = await listen<string>("transcription_error", (event) => {
    console.error("transcription error:", event.payload);
    toast.error("transcription error" + event.payload);
    const appState = useAppStateStore.getState();
    if (appState.isRecording) {
      appState.updateIsRecording(false);
    }
    toast.error("当前" + selectedAsrVendor + "Websocket流连接已关闭");
  });

  await invoke("start_recognize_audio_stream_from_speaker_loopback", {
    deviceName: audioDevice,
    selectedAsrVendor,
    captureInterval,
  }).catch((err) => {
    console.error("invoke start output audio recognition failed", err);
    toast.error("invoke start audio capture err" + err);
  });
}

export async function stopAudioLoopbackRecognition() {
  await invoke("stop_recognize_audio_stream_from_speaker_loopback").catch(
    (err) => {
      console.error("invoke stop output audio recognition failed", err);
      toast.error("invoke stop audio capture err" + err);
    },
  );

  if (unlistener) {
    unlistener();
    unlistener = null;
  }
  if (errorUnlistener) {
    errorUnlistener();
    errorUnlistener = null;
  }
}
