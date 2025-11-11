import { create } from "zustand";
import { ModelOption } from "@/types/llm.ts";

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
	captureInterval: number;
	updateCaptureInterval: (target: number) => void;
	useBigModel: boolean;
	updateUseBigModel: (target: boolean) => void;
	useRemoteModelTranscribe: boolean;
	updateUseRemoteModelTranscribe: (target: boolean) => void;

	useResamplePCMBuffer: boolean;
	updateUseResamplePCMBuffer: (target: boolean) => void;

	useAutoChunkBuffer: boolean;
	updateUseAutoChunkBuffer: (target: boolean) => void;
}

const useAppStateStore = create<AppStateStore>((set) => ({
	currentSelectedModel: "siliconflow_free", // 默认模型
	updateCurrentSelectedModel: (target: ModelOption) =>
		set({ currentSelectedModel: target }),

	currentAudioChannel: "",
	updateCurrentAudioChannel: (target: string) =>
		set({ currentAudioChannel: target }),
	llmPrompt: import.meta.env.VITE_INTERVIEW_PROMPT || "",
	updateLLMPrompt: (target: string) => {
		set({ llmPrompt: target });
	},
	currentQuestion: "",
	updateQuestion: (target: string) => {
		set({ currentQuestion: target });
	},
	isStartScrollToBottom: true,
	updateScrollToBottom: (target: boolean) => {
		set({ isStartScrollToBottom: target });
	},

	captureInterval: 1,
	updateCaptureInterval: (target: number) => {
		set({ captureInterval: target });
	},
	useBigModel: true,
	updateUseBigModel: (target: boolean) => {
		set({ useBigModel: target });
	},
	useRemoteModelTranscribe: false,
	updateUseRemoteModelTranscribe: (target: boolean) => {
		set({ useRemoteModelTranscribe: target });
	},

	useResamplePCMBuffer: false,
	updateUseResamplePCMBuffer: (target: boolean) => {
		set({ useResamplePCMBuffer: target });
	},
	useAutoChunkBuffer: true,
	updateUseAutoChunkBuffer: (target: boolean) => {
		set({ useAutoChunkBuffer: target });
	},
}));

export default useAppStateStore;
