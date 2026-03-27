import {
	startAudioLoopbackRecognition,
	stopAudioLoopbackRecognition,
} from "@/lib/cpal.ts";
import { logError } from "@/lib/logger.ts";
import type { TranscribeVendor } from "@/stores";

interface BrowserSpeechRecognition extends EventTarget {
	lang: string;
	continuous: boolean;
	interimResults: boolean;
	start(): void;
	stop(): void;
	abort(): void;
	onresult: ((event: BrowserSpeechRecognitionEvent) => void) | null;
	onerror: ((event: BrowserSpeechRecognitionErrorEvent) => void) | null;
}

interface BrowserSpeechRecognitionEvent extends Event {
	results: SpeechRecognitionResultList;
}

interface BrowserSpeechRecognitionErrorEvent extends Event {
	error: string;
	message: string;
}

type SpeechRecognitionClass = new () => BrowserSpeechRecognition;

declare global {
	interface Window {
		SpeechRecognition?: SpeechRecognitionClass;
		webkitSpeechRecognition?: SpeechRecognitionClass;
	}
}

let activeCallback: ((msg: string) => void) | null = null;
let recognitionInstance: BrowserSpeechRecognition | null = null;

function getRecognition(): BrowserSpeechRecognition {
	if (recognitionInstance) return recognitionInstance;

	const win = window as Window & {
		SpeechRecognition?: SpeechRecognitionClass;
		webkitSpeechRecognition?: SpeechRecognitionClass;
	};
	const SpeechRecognitionClass =
		win.SpeechRecognition ?? win.webkitSpeechRecognition;
	if (!SpeechRecognitionClass)
		throw new Error("当前浏览器不支持 SpeechRecognition API");

	const recognition = new SpeechRecognitionClass();
	recognition.lang = "zh-CN";
	recognition.continuous = true;
	recognition.interimResults = true;

	recognition.onresult = (event: BrowserSpeechRecognitionEvent) => {
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

	recognition.onerror = (event: BrowserSpeechRecognitionErrorEvent) => {
		console.error("SpeechRecognition error:", event.error, event.message);
		logError("SpeechRecognition error", `${event.error}: ${event.message}`);
	};

	recognitionInstance = recognition;
	return recognition;
}

export async function startAudioRecognition(
	onMessageCapture: (message: string) => void,
	audioDevice: string,
	selectedAsrVendor: TranscribeVendor,
	captureInterval: number,
	isUsePreRecorded: boolean,
) {
	if (audioDevice.includes("输出")) {
		return await startAudioLoopbackRecognition(
			onMessageCapture,
			audioDevice,
			selectedAsrVendor,
			captureInterval,
			isUsePreRecorded,
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
