import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import useAppStateStore from "@/stores";

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
		const appState = useAppStateStore.getState();
		if (appState.isRecording) {
			appState.updateIsRecording(false);
		}
	});

	await invoke("start_recognize_audio_stream_from_speaker_loopback", {
		deviceName: audioDevice,
		selectedAsrVendor,
		captureInterval,
	}).catch((err) => {
		console.error("invoke start output audio recognition failed", err);
	});
}

export async function stopAudioLoopbackRecognition() {
	await invoke("stop_recognize_audio_stream_from_speaker_loopback").catch(
		(err) => console.error("invoke stop output audio recognition failed", err),
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
