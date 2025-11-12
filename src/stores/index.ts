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

  useRemoteModelTranscribe: "assemblyai",
  updateRemoteModelTranscribe: (target: TranscribeVendor) => {
    set({ useRemoteModelTranscribe: target });
  },
}));

export default useAppStateStore;
