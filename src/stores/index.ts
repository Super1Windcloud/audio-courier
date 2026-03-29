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
import {
	createDefaultLlmProviderSettings,
	createDefaultProviderEnvPresets,
	createDefaultTranscriptProviderSettings,
	type LlmProviderSettings,
	normalizeLlmProviderSettings,
	normalizeProviderEnvPresets,
	normalizeTranscriptProviderSettings,
	type ProviderEnvPresets,
	TRANSCRIBE_VENDORS,
	type TranscribeVendor,
	type TranscriptProviderSettings,
} from "@/types/provider.ts";

export type UiTextTone = "light" | "dark";
export type { TranscribeVendor } from "@/types/provider.ts";

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

	llmProviderSettings: LlmProviderSettings;
	updateLlmProviderSettings: (target: LlmProviderSettings) => void;

	transcriptProviderSettings: TranscriptProviderSettings;
	updateTranscriptProviderSettings: (
		target: TranscriptProviderSettings,
	) => void;

	envProviderPresets: ProviderEnvPresets;
	updateEnvProviderPresets: (target: ProviderEnvPresets) => void;
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
	| "llmProviderSettings"
	| "transcriptProviderSettings"
>;

const DEFAULT_LLM_PROMPT =
	(import.meta.env.DEV ? import.meta.env.VITE_PROMPT : "") || "";
const DEFAULT_INTERVIEW_PROMPT =
	(import.meta.env.DEV ? import.meta.env.VITE_INTERVIEW_PROMPT : "") || "";
const LEGACY_UI_DEFAULTS = readLegacyUiConfigDefaults();

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
		llmProviderSettings: createDefaultLlmProviderSettings(),
		transcriptProviderSettings: createDefaultTranscriptProviderSettings(),
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
		llmProviderSettings: normalizeLlmProviderSettings(
			persistedState.llmProviderSettings,
		),
		transcriptProviderSettings: normalizeTranscriptProviderSettings(
			persistedState.transcriptProviderSettings,
		),
	};
}

function pickPersistedAppConfigState(
	state: AppStateStore,
): PersistedAppConfigState {
	const result: PersistedAppConfigState = {
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
		llmProviderSettings: state.llmProviderSettings,
		transcriptProviderSettings: state.transcriptProviderSettings,
	};

	if (import.meta.env.DEV) {
		// In development, do not write these to storage if they come from the env
		if (state.llmPrompt === DEFAULT_LLM_PROMPT) {
			result.llmPrompt = "";
		}
		if (state.interviewPrompt === DEFAULT_INTERVIEW_PROMPT) {
			result.interviewPrompt = "";
		}
	}

	return result;
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
			llmProviderSettings: defaultPersistedConfigState.llmProviderSettings,
			updateLlmProviderSettings: (target: LlmProviderSettings) => {
				set({ llmProviderSettings: normalizeLlmProviderSettings(target) });
			},
			transcriptProviderSettings:
				defaultPersistedConfigState.transcriptProviderSettings,
			updateTranscriptProviderSettings: (
				target: TranscriptProviderSettings,
			) => {
				set({
					transcriptProviderSettings:
						normalizeTranscriptProviderSettings(target),
				});
			},
			envProviderPresets: createDefaultProviderEnvPresets(),
			updateEnvProviderPresets: (target: ProviderEnvPresets) => {
				set({ envProviderPresets: normalizeProviderEnvPresets(target) });
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
			version: 2,
			migrate: (persistedState) => {
				if (!persistedState || typeof persistedState !== "object") {
					return persistedState as PersistedAppConfigState;
				}

				const { envProviderPresets: _ignored, ...rest } =
					persistedState as PersistedAppConfigState & {
						envProviderPresets?: ProviderEnvPresets;
					};

				return rest;
			},
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
