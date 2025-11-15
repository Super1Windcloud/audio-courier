import {
	startAudioLoopbackRecognition,
	stopAudioLoopbackRecognition,
} from "@/lib/cpal.ts";
import { TranscribeVendor } from "@/stores";

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

		onaudioend: ((this: SpeechRecognition, ev: Event) => void) | null;
		onaudiostart: ((this: SpeechRecognition, ev: Event) => void) | null;
		onend: ((this: SpeechRecognition, ev: Event) => void) | null;
		onerror:
			| ((this: SpeechRecognition, ev: SpeechRecognitionErrorEvent) => void)
			| null;
		onnomatch:
			| ((this: SpeechRecognition, ev: SpeechRecognitionEvent) => void)
			| null;
		onresult:
			| ((this: SpeechRecognition, ev: SpeechRecognitionEvent) => void)
			| null;
		onsoundend: ((this: SpeechRecognition, ev: Event) => void) | null;
		onsoundstart: ((this: SpeechRecognition, ev: Event) => void) | null;
		onspeechend: ((this: SpeechRecognition, ev: Event) => void) | null;
		onspeechstart: ((this: SpeechRecognition, ev: Event) => void) | null;
		onstart: ((this: SpeechRecognition, ev: Event) => void) | null;
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

	const win = window as Window & {
		SpeechRecognition?: SpeechRecognitionConstructor;
		webkitSpeechRecognition?: SpeechRecognitionConstructor;
	};
	const SpeechRecognitionClass =
		win.SpeechRecognition ?? win.webkitSpeechRecognition;
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
	selectedAsrVendor: TranscribeVendor,
	captureInterval: number,
) {
	if (audioDevice.includes("输出")) {
		return await startAudioLoopbackRecognition(
			onMessageCapture,
			audioDevice,
			selectedAsrVendor,
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
