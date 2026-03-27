import { create } from "zustand";
import type { LicenseStatus } from "@/types/license.ts";
import type { ModelOption } from "@/types/llm.ts";

export type TranscribeVendor =
	| "assemblyai"
	| "deepgram"
	| "gladia"
	| "revai"
	| "speechmatics";

interface AppStateStore {
	currentSelectedModel: ModelOption;
	updateCurrentSelectedModel: (target: ModelOption) => void;

	currentAudioChannel: string;
	updateCurrentAudioChannel: (target: string) => void;

	llmPrompt: string;
	updateLLMPrompt: (target: string) => void;
	currentQuestion: string;
	updateQuestion: (target: string) => void;

	isStartScrollToBottom: boolean;
	updateScrollToBottom: (target: boolean) => void;

	useRemoteModelTranscribe: TranscribeVendor;
	updateRemoteModelTranscribe: (target: TranscribeVendor) => void;

	captureInterval: number;
	updateCaptureInterval: (target: number) => void;

	isRecording: boolean;
	recordingStartedAt: number | null;
	updateIsRecording: (target: boolean) => void;

	isUsePreRecorded: boolean;
	updatePreRecorded: (target: boolean) => void;

	licenseStatus: LicenseStatus | null;
	updateLicenseStatus: (target: LicenseStatus | null) => void;

	uiOpacity: number;
	updateUiOpacity: (target: number) => void;
}

const UI_OPACITY_STORAGE_KEY = "audio-courier-ui-opacity";

function normalizeUiOpacity(target: number) {
	return Math.min(1, Math.max(0.45, Number(target.toFixed(2))));
}

function readInitialUiOpacity() {
	if (typeof window === "undefined") {
		return 1;
	}

	const rawValue = window.localStorage.getItem(UI_OPACITY_STORAGE_KEY);
	if (!rawValue) {
		return 1;
	}

	const parsedValue = Number(rawValue);
	if (Number.isNaN(parsedValue)) {
		return 1;
	}

	return normalizeUiOpacity(parsedValue);
}

const useAppStateStore = create<AppStateStore>((set) => ({
	uiOpacity: readInitialUiOpacity(),
	updateUiOpacity: (target: number) => {
		const nextOpacity = normalizeUiOpacity(target);
		if (typeof window !== "undefined") {
			window.localStorage.setItem(UI_OPACITY_STORAGE_KEY, String(nextOpacity));
		}
		set({ uiOpacity: nextOpacity });
	},
	isUsePreRecorded: false,
	updatePreRecorded: (target: boolean) => {
		set({ isUsePreRecorded: target });
	},
	licenseStatus: null,
	updateLicenseStatus: (target: LicenseStatus | null) => {
		set({ licenseStatus: target });
	},
	currentSelectedModel: "siliconflow_pro",
	updateCurrentSelectedModel: (target: ModelOption) =>
		set({ currentSelectedModel: target }),

	currentAudioChannel: "",
	updateCurrentAudioChannel: (target: string) =>
		set({ currentAudioChannel: target }),
	llmPrompt: import.meta.env.VITE_PROMPT || "",
	updateLLMPrompt: (target: string) => {
		set({ llmPrompt: target });
	},
	currentQuestion: "",
	updateQuestion: (target: string) => {
		set({ currentQuestion: target });
	},
	isStartScrollToBottom: false,
	updateScrollToBottom: (target: boolean) => {
		set({ isStartScrollToBottom: target });
	},

	useRemoteModelTranscribe: "deepgram",
	updateRemoteModelTranscribe: (target: TranscribeVendor) => {
		set({ useRemoteModelTranscribe: target });
	},

	captureInterval: 2,
	updateCaptureInterval: (target: number) => {
		set({ captureInterval: target });
	},

	isRecording: false,
	recordingStartedAt: null,
	updateIsRecording: (target: boolean) => {
		set({
			isRecording: target,
			recordingStartedAt: target ? Date.now() : null,
		});
	},
}));

export default useAppStateStore;
