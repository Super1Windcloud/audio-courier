import { create } from "zustand";
import { ModelOption } from "@/types/llm.ts";

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
	updateIsRecording: (target: boolean) => void;

	isUsePreRecorded: boolean;
	updatePreRecorded: (target: boolean) => void;
}

const useAppStateStore = create<AppStateStore>((set) => ({
	isUsePreRecorded: false,
	updatePreRecorded: (target: boolean) => {
		set({ isUsePreRecorded: target });
	},
	currentSelectedModel: "siliconflow_free",
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

	useRemoteModelTranscribe: "gladia",
	updateRemoteModelTranscribe: (target: TranscribeVendor) => {
		set({ useRemoteModelTranscribe: target });
	},

	captureInterval: 2,
	updateCaptureInterval: (target: number) => {
		set({ captureInterval: target });
	},

	isRecording: false,
	updateIsRecording: (target: boolean) => {
		set({ isRecording: target });
	},
}));

export default useAppStateStore;
