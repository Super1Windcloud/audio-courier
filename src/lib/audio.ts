import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export async function startAudioRecognition(
	onMessageCapture: (message: string) => void,
) {
	invoke("start_recognize_audio_stream");

	let content = "";
	await listen<string>("transcribed", (event) => {
		console.log("识别结果:", event.payload);
		content += event.payload;
		onMessageCapture(content);
	});
}

export async function stopAudioRecognition() {
	await invoke("stop_recognize_audio_stream");
}
