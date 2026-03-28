import { create } from "zustand";
import { persist } from "zustand/middleware";
import {
	APP_CONFIG_STATE_KEY,
	createAppConfigStorage,
	readLegacyUiConfigDefaults,
} from "@/lib/appConfig.ts";
import { logError } from "@/lib/logger.ts";
import type { LicenseStatus } from "@/types/license.ts";
import { MODEL_OPTIONS, type ModelOption } from "@/types/llm.ts";

export type TranscribeVendor =
	| "assemblyai"
	| "deepgram"
	| "gladia"
	| "revai"
	| "speechmatics";

export type UiTextTone = "light" | "dark";

interface AppStateStore {
	currentSelectedModel: ModelOption;
	updateCurrentSelectedModel: (target: ModelOption) => void;

	currentAudioChannel: string;
	updateCurrentAudioChannel: (target: string) => void;

	llmPrompt: string;
	updateLLMPrompt: (target: string) => void;
	interviewPrompt: string;
	updateInterviewPrompt: (target: string) => void;
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

	uiTextTone: UiTextTone;
	updateUiTextTone: (target: UiTextTone) => void;
}

type PersistedAppConfigState = Pick<
	AppStateStore,
	| "currentSelectedModel"
	| "currentAudioChannel"
	| "llmPrompt"
	| "interviewPrompt"
	| "isStartScrollToBottom"
	| "useRemoteModelTranscribe"
	| "captureInterval"
	| "isUsePreRecorded"
	| "uiOpacity"
	| "uiTextTone"
>;

const DEFAULT_LLM_PROMPT = import.meta.env.VITE_PROMPT || "";
const DEFAULT_INTERVIEW_PROMPT = import.meta.env.VITE_INTERVIEW_PROMPT || "";
const LEGACY_UI_DEFAULTS = readLegacyUiConfigDefaults();
const TRANSCRIBE_VENDORS: readonly TranscribeVendor[] = [
	"assemblyai",
	"deepgram",
	"gladia",
	"revai",
	"speechmatics",
];

function normalizeUiOpacity(target: number) {
	return Math.min(1, Math.max(0.3, Number(target.toFixed(2))));
}

function normalizeUiTextTone(target: unknown): UiTextTone {
	return target === "dark" ? "dark" : "light";
}

function normalizeCaptureInterval(target: unknown) {
	if (typeof target !== "number" || Number.isNaN(target)) {
		return 2;
	}

	return Math.max(1, Math.round(target));
}

function isModelOption(target: unknown): target is ModelOption {
	return (
		typeof target === "string" &&
		(MODEL_OPTIONS as readonly string[]).includes(target)
	);
}

function isTranscribeVendor(target: unknown): target is TranscribeVendor {
	return (
		typeof target === "string" &&
		(TRANSCRIBE_VENDORS as readonly string[]).includes(target)
	);
}

function createDefaultPersistedConfigState(): PersistedAppConfigState {
	return {
		currentSelectedModel: "siliconflow_pro",
		currentAudioChannel: "",
		llmPrompt: DEFAULT_LLM_PROMPT,
		interviewPrompt: DEFAULT_INTERVIEW_PROMPT,
		isStartScrollToBottom: false,
		useRemoteModelTranscribe: "deepgram",
		captureInterval: 2,
		isUsePreRecorded: false,
		uiOpacity: normalizeUiOpacity(LEGACY_UI_DEFAULTS.uiOpacity ?? 1),
		uiTextTone: normalizeUiTextTone(LEGACY_UI_DEFAULTS.uiTextTone),
	};
}

function normalizePersistedAppConfigState(
	target: unknown,
	currentState: PersistedAppConfigState,
): PersistedAppConfigState {
	if (!target || typeof target !== "object") {
		return currentState;
	}

	const persistedState = target as Partial<PersistedAppConfigState>;

	return {
		currentSelectedModel: isModelOption(persistedState.currentSelectedModel)
			? persistedState.currentSelectedModel
			: currentState.currentSelectedModel,
		currentAudioChannel:
			typeof persistedState.currentAudioChannel === "string"
				? persistedState.currentAudioChannel
				: currentState.currentAudioChannel,
		llmPrompt:
			typeof persistedState.llmPrompt === "string"
				? persistedState.llmPrompt
				: currentState.llmPrompt,
		interviewPrompt:
			typeof persistedState.interviewPrompt === "string"
				? persistedState.interviewPrompt
				: currentState.interviewPrompt,
		isStartScrollToBottom:
			typeof persistedState.isStartScrollToBottom === "boolean"
				? persistedState.isStartScrollToBottom
				: currentState.isStartScrollToBottom,
		useRemoteModelTranscribe: isTranscribeVendor(
			persistedState.useRemoteModelTranscribe,
		)
			? persistedState.useRemoteModelTranscribe
			: currentState.useRemoteModelTranscribe,
		captureInterval: normalizeCaptureInterval(
			persistedState.captureInterval ?? currentState.captureInterval,
		),
		isUsePreRecorded:
			typeof persistedState.isUsePreRecorded === "boolean"
				? persistedState.isUsePreRecorded
				: currentState.isUsePreRecorded,
		uiOpacity: normalizeUiOpacity(
			typeof persistedState.uiOpacity === "number"
				? persistedState.uiOpacity
				: currentState.uiOpacity,
		),
		uiTextTone: normalizeUiTextTone(
			persistedState.uiTextTone ?? currentState.uiTextTone,
		),
	};
}

function pickPersistedAppConfigState(
	state: AppStateStore,
): PersistedAppConfigState {
	return {
		currentSelectedModel: state.currentSelectedModel,
		currentAudioChannel: state.currentAudioChannel,
		llmPrompt: state.llmPrompt,
		interviewPrompt: state.interviewPrompt,
		isStartScrollToBottom: state.isStartScrollToBottom,
		useRemoteModelTranscribe: state.useRemoteModelTranscribe,
		captureInterval: state.captureInterval,
		isUsePreRecorded: state.isUsePreRecorded,
		uiOpacity: state.uiOpacity,
		uiTextTone: state.uiTextTone,
	};
}

const defaultPersistedConfigState = createDefaultPersistedConfigState();

const useAppStateStore = create<AppStateStore>()(
	persist(
		(set) => ({
			...defaultPersistedConfigState,
			updateUiOpacity: (target: number) => {
				set({ uiOpacity: normalizeUiOpacity(target) });
			},
			updateUiTextTone: (target: UiTextTone) => {
				set({ uiTextTone: normalizeUiTextTone(target) });
			},
			updatePreRecorded: (target: boolean) => {
				set({ isUsePreRecorded: target });
			},
			licenseStatus: null,
			updateLicenseStatus: (target: LicenseStatus | null) => {
				set({ licenseStatus: target });
			},
			updateCurrentSelectedModel: (target: ModelOption) =>
				set({ currentSelectedModel: target }),
			updateCurrentAudioChannel: (target: string) =>
				set({ currentAudioChannel: target }),
			updateLLMPrompt: (target: string) => {
				set({ llmPrompt: target });
			},
			updateInterviewPrompt: (target: string) => {
				set({ interviewPrompt: target });
			},
			currentQuestion: "",
			updateQuestion: (target: string) => {
				set({ currentQuestion: target });
			},
			updateScrollToBottom: (target: boolean) => {
				set({ isStartScrollToBottom: target });
			},
			updateRemoteModelTranscribe: (target: TranscribeVendor) => {
				set({ useRemoteModelTranscribe: target });
			},
			updateCaptureInterval: (target: number) => {
				set({ captureInterval: normalizeCaptureInterval(target) });
			},
			isRecording: false,
			recordingStartedAt: null,
			updateIsRecording: (target: boolean) => {
				set({
					isRecording: target,
					recordingStartedAt: target ? Date.now() : null,
				});
			},
		}),
		{
			name: APP_CONFIG_STATE_KEY,
			storage: createAppConfigStorage<PersistedAppConfigState>(),
			partialize: pickPersistedAppConfigState,
			skipHydration: true,
			version: 1,
			merge: (persistedState, currentState) => ({
				...currentState,
				...normalizePersistedAppConfigState(
					persistedState,
					pickPersistedAppConfigState(currentState),
				),
			}),
			onRehydrateStorage: () => (_, error) => {
				if (error) {
					logError("rehydrate-app-config failed", error);
				}
			},
		},
	),
);

export default useAppStateStore;
