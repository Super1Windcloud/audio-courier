import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { restoreFakePunctuation } from "@/utils";

let unlistener: UnlistenFn | null = null;

export async function startAudioLoopbackRecognition(
	onMessageCapture: (message: string) => void,
	audioDevice: string,
	captureInterval: number,
	useBigModel: boolean,
	useRemoteModelTranscribe: boolean,
	useResamplePCMBuffer: boolean,
	useAutoChunkBuffer: boolean,
) {
	if (unlistener) {
		unlistener();
		unlistener = null;
	}

	unlistener = await listen<string>("transcription_result", (event) => {
		console.log("识别扬声器结果:", event.payload);
		let content = event.payload.replace(/\s/g, "");
		content = restoreFakePunctuation(content);
		onMessageCapture(content);
	});

	await invoke("start_recognize_audio_stream_from_speaker_loopback", {
		deviceName: audioDevice,
		captureInterval,
		useBigModel,
		useRemoteModel: useRemoteModelTranscribe,
		useResampled: useResamplePCMBuffer,
		autoChunkBuffer: useAutoChunkBuffer,
	}).catch((err) => {
		console.error("invoke start output audio recognition failed", err);
	});
}

export async function stopAudioLoopbackRecognition() {
	await clearVoskAcceptBuffer();
	await invoke("stop_recognize_audio_stream_from_speaker_loopback").catch(
		(err) => console.error("invoke stop output audio recognition failed", err),
	);

	if (unlistener) {
		unlistener();
		unlistener = null;
	}
}

export async function clearVoskAcceptBuffer() {
	await invoke("clear_vosk_accept_buffer").catch((err) => {
		console.error("invoke clear vosk buffer failed", err);
	});
}
