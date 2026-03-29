import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { toast } from "sonner";
import { logError, logInfo } from "@/lib/logger.ts";
import { setRecordingStateImmediately } from "@/lib/recordingState.ts";
import useAppStateStore from "@/stores";

let traditionalChineseConverter: ((content: string) => string) | null = null;

async function convertTraditionalChinese(content: string) {
	if (!traditionalChineseConverter) {
		const { convertTraditionalChinese: convert } = await import(
			"../../scripts/opencc.ts"
		);
		traditionalChineseConverter = convert;
	}

	return traditionalChineseConverter(content);
}

let unlistener: UnlistenFn | null = null;
let errorUnlistener: UnlistenFn | null = null;

export async function startAudioLoopbackRecognition(
	onMessageCapture: (message: string) => void,
	audioDevice: string,
	selectedAsrVendor: string,
	captureInterval: number,
	isUsePreRecorded: boolean,
) {
	void isUsePreRecorded;
	if (unlistener) {
		unlistener();
		unlistener = null;
	}
	if (errorUnlistener) {
		errorUnlistener();
		errorUnlistener = null;
	}

	let content: string = "";
	const transcriptProviderSettings =
		useAppStateStore.getState().transcriptProviderSettings;

	unlistener = await listen<string>("transcription_result", async (event) => {
		logInfo(`transcription_result received length=${event.payload.length}`);
		if (
			selectedAsrVendor.toLowerCase() === "assemblyai" ||
			selectedAsrVendor.toLowerCase() === "revai" ||
			selectedAsrVendor.toLowerCase() === "deepgram"
		) {
			content = event.payload;
		} else if (selectedAsrVendor.toLowerCase() === "gladia") {
			content = await convertTraditionalChinese(event.payload);
		} else {
			content += event.payload;
		}
		onMessageCapture(content);
	});
	errorUnlistener = await listen<string>("transcription_error", (event) => {
		console.error("transcription error:", event.payload);
		logError("transcription_error received", event.payload);
		toast.error(`transcription error${event.payload}`);
		const appState = useAppStateStore.getState();
		if (appState.isRecording) {
			setRecordingStateImmediately(false);
		}
		toast.error(`当前${selectedAsrVendor}Websocket流连接已关闭`);
	});

	await invoke("start_recognize_audio_stream_from_speaker_loopback", {
		deviceName: audioDevice,
		selectedAsrVendor,
		captureInterval,
		transcriptConfig: transcriptProviderSettings,
	}).catch((err) => {
		console.error("invoke start output audio recognition failed", err);
		logError("invoke start output audio recognition failed", err);
		toast.error(`invoke start audio capture err${err}`);
		const appState = useAppStateStore.getState();
		if (appState.isRecording) {
			setRecordingStateImmediately(false);
		}
	});
}

export async function stopAudioLoopbackRecognition() {
	await invoke("stop_recognize_audio_stream_from_speaker_loopback").catch(
		(err) => {
			console.error("invoke stop output audio recognition failed", err);
			logError("invoke stop output audio recognition failed", err);
			toast.error(`invoke stop audio capture err${err}`);
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
