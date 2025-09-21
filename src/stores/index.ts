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
}

const useAppStateStore = create<AppStateStore>((set) => ({
  currentSelectedModel: "siliconflow", // 默认模型
  updateCurrentSelectedModel: (target: ModelOption) =>
    set({ currentSelectedModel: target }),

  currentAudioChannel: "",
  updateCurrentAudioChannel: (target: string) =>
    set({ currentAudioChannel: target }),
  llmPrompt: import.meta.env.VITE_PROMPT as string,

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
}));

export default useAppStateStore;
