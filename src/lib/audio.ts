import {
  startAudioLoopbackRecognition,
  stopAudioLoopbackRecognition,
} from "@/lib/cpal.ts";

declare global {
  interface Window {
    SpeechRecognition: SpeechRecognitionConstructor;
    webkitSpeechRecognition: SpeechRecognitionConstructor;
  }

  interface SpeechRecognitionConstructor {
    new (): SpeechRecognition;
  }

  interface SpeechRecognition extends EventTarget {
    lang: string;
    continuous: boolean;
    interimResults: boolean;
    start(): void;
    stop(): void;
    abort(): void;

    onaudioend: ((this: SpeechRecognition, ev: Event) => any) | null;
    onaudiostart: ((this: SpeechRecognition, ev: Event) => any) | null;
    onend: ((this: SpeechRecognition, ev: Event) => any) | null;
    onerror:
      | ((this: SpeechRecognition, ev: SpeechRecognitionErrorEvent) => any)
      | null;
    onnomatch:
      | ((this: SpeechRecognition, ev: SpeechRecognitionEvent) => any)
      | null;
    onresult:
      | ((this: SpeechRecognition, ev: SpeechRecognitionEvent) => any)
      | null;
    onsoundend: ((this: SpeechRecognition, ev: Event) => any) | null;
    onsoundstart: ((this: SpeechRecognition, ev: Event) => any) | null;
    onspeechend: ((this: SpeechRecognition, ev: Event) => any) | null;
    onspeechstart: ((this: SpeechRecognition, ev: Event) => any) | null;
    onstart: ((this: SpeechRecognition, ev: Event) => any) | null;
  }

  interface SpeechRecognitionEvent extends Event {
    resultIndex: number;
    results: SpeechRecognitionResultList;
  }

  interface SpeechRecognitionErrorEvent extends Event {
    error: string;
    message: string;
  }
}

let recognitionInstance: SpeechRecognition | null = null;
let activeCallback: ((msg: string) => void) | null = null;

function getRecognition(): SpeechRecognition {
  if (recognitionInstance) return recognitionInstance;

  const SpeechRecognitionClass =
    (window as any).SpeechRecognition ||
    (window as any).webkitSpeechRecognition;
  if (!SpeechRecognitionClass)
    throw new Error("当前浏览器不支持 SpeechRecognition API");

  const recognition = new SpeechRecognitionClass();
  recognition.lang = "zh-CN";
  recognition.continuous = true;
  recognition.interimResults = true;

  recognition.onresult = (event: SpeechRecognitionEvent) => {
    let finalTranscript = "";
    let interimTranscript = "";

    for (let i = 0; i < event.results.length; ++i) {
      const result = event.results[i];
      if (result.isFinal) {
        finalTranscript += result[0].transcript;
      } else {
        interimTranscript += result[0].transcript;
      }
    }

    if (activeCallback) activeCallback(finalTranscript + interimTranscript);
  };

  recognition.onerror = (event: SpeechRecognitionErrorEvent) => {
    console.error("SpeechRecognition error:", event.error, event.message);
  };

  recognitionInstance = recognition;
  return recognition;
}

export async function startAudioRecognition(
  onMessageCapture: (message: string) => void,
  audioDevice: string,
  captureInterval: number,
) {
  if (audioDevice.includes("输出")) {
    return await startAudioLoopbackRecognition(
      onMessageCapture,
      audioDevice,
      captureInterval,
    );
  }

  const recognition = getRecognition();
  activeCallback = onMessageCapture;
  recognition.start();

  return () => recognition.stop();
}

export async function stopAudioRecognition(device: string) {
  if (device.includes("输出")) {
    return await stopAudioLoopbackRecognition();
  }
  activeCallback = null;
  recognitionInstance?.stop();
  recognitionInstance = null;
}
