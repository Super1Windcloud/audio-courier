import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { toast } from "sonner";
import { logError, logInfo } from "@/lib/logger.ts";
import { setRecordingStateImmediately } from "@/lib/recordingState.ts";
import useAppStateStore from "@/stores";
import {
	TRANSCRIBE_VENDOR_LABELS,
	type TranscribeVendor,
} from "@/types/provider.ts";

let traditionalChineseConverter: ((content: string) => string) | null = null;

interface TranscriptEvent {
	vendor: string;
	kind: "draft" | "commit";
	text: string;
}

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
	onFinalMessageCapture: (message: string) => void,
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

	const vendorLabel =
		TRANSCRIBE_VENDOR_LABELS[selectedAsrVendor as TranscribeVendor] ??
		selectedAsrVendor;
	toast.message("开始转录", {
		description: `当前使用 ${vendorLabel}`,
	});

	const transcriptProviderSettings =
		useAppStateStore.getState().transcriptProviderSettings;

	const normalizeTranscript = async (payload: string) => {
		if (selectedAsrVendor.toLowerCase() === "gladia") {
			return await convertTraditionalChinese(payload);
		}

		return payload;
	};

	unlistener = await listen<TranscriptEvent>(
		"transcription_event",
		async (event) => {
			const { kind, text, vendor } = event.payload;
			if (!text.trim()) {
				return;
			}

			logInfo(
				`transcription_event received vendor=${vendor} kind=${kind} length=${text.length}`,
			);
			const normalized = await normalizeTranscript(text);
			onMessageCapture(normalized);
			if (kind === "commit") {
				onFinalMessageCapture(normalized);
			}
		},
	);
	errorUnlistener = await listen<string>("transcription_error", (event) => {
		console.error("transcription error:", event.payload);
		logError("transcription_error received", event.payload);
		const appState = useAppStateStore.getState();
		if (appState.isRecording) {
			setRecordingStateImmediately(false);
		}
		toast.error(`当前 ${selectedAsrVendor} 转录连接异常关闭: ${event.payload}`);
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
