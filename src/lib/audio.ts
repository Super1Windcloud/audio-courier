import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Message } from "@/components/ChatContainer.tsx";
import React from "react";

export async function startAudioRecognition(
  onMessageCapture: (message: string, replyId: string) => void,
  setMessages: React.Dispatch<React.SetStateAction<Message[]>>,
) {
  await invoke("start_recognize_audio_stream");

  const replyId = (Date.now() + 1).toString();

  setMessages((prev) => [
    ...prev,
    {
      id: replyId,
      text: "",
      timestamp: new Date(),
      sender: "robot",
    },
  ]);

  let content = "";
  await listen<string>("transcribed", (event) => {
    console.log("识别结果:", event.payload);
    content += event.payload;
    onMessageCapture(content, replyId);
  });
}

export async function stopAudioRecognition() {
  await invoke("stop_recognize_audio_stream");
}
