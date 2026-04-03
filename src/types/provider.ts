export type TranscribeVendor =
	| "assemblyai"
	| "deepgram"
	| "gladia"
	| "revai"
	| "speechmatics";

export type MacosSystemAudioBackend = "swift-helper" | "rust-native";

export const MACOS_SYSTEM_AUDIO_BACKENDS: readonly MacosSystemAudioBackend[] = [
	"swift-helper",
	"rust-native",
];

export const MACOS_SYSTEM_AUDIO_BACKEND_LABELS: Record<
	MacosSystemAudioBackend,
	string
> = {
	"swift-helper": "Swift Helper",
	"rust-native": "Rust Native",
};

export const TRANSCRIBE_VENDORS: readonly TranscribeVendor[] = [
	"assemblyai",
	"deepgram",
	"gladia",
	"revai",
	"speechmatics",
];

export const TRANSCRIBE_VENDOR_LABELS: Record<TranscribeVendor, string> = {
	assemblyai: "AssemblyAI",
	deepgram: "DeepGram",
	gladia: "Gladia",
	revai: "RevAI",
	speechmatics: "Speechmatics",
};

export interface LlmProviderSettings {
	siliconflowApiKey: string;
	doubaoApiKey: string;
	kimiApiKey: string;
	zhipuApiKey: string;
	deepseekApiKey: string;
	aliQwenApiKey: string;
	openaiApiKey: string;
	openaiBaseUrl: string;
	openaiModel: string;
	geminiApiKey: string;
	geminiBaseUrl: string;
	geminiModel: string;
	customOpenAiName: string;
	customOpenAiApiKey: string;
	customOpenAiBaseUrl: string;
	customOpenAiModel: string;
}

export interface TranscriptProviderSettings {
	deepgramApiKey: string;
	deepgramLanguage: string;
	assemblyApiKey: string;
	gladiaApiKey: string;
	gladiaLanguage: string;
	gladiaModel: string;
	speechmaticsApiKey: string;
	speechmaticsLanguage: string;
	speechmaticsRtUrl: string;
	revaiApiKey: string;
	revaiLanguage: string;
	revaiMetadata: string;
	macosSystemAudioBackend: MacosSystemAudioBackend;
}

export interface ProviderEnvPresets {
	llm: LlmProviderSettings;
	transcript: TranscriptProviderSettings;
}

const LLM_API_KEY_FIELDS = [
	"siliconflowApiKey",
	"doubaoApiKey",
	"kimiApiKey",
	"zhipuApiKey",
	"deepseekApiKey",
	"aliQwenApiKey",
	"openaiApiKey",
	"geminiApiKey",
	"customOpenAiApiKey",
] as const satisfies readonly (keyof LlmProviderSettings)[];

function readString(value: unknown, fallback = "") {
	return typeof value === "string" ? value : fallback;
}

export function createDefaultLlmProviderSettings(): LlmProviderSettings {
	return {
		siliconflowApiKey: "",
		doubaoApiKey: "",
		kimiApiKey: "",
		zhipuApiKey: "",
		deepseekApiKey: "",
		aliQwenApiKey: "",
		openaiApiKey: "",
		openaiBaseUrl: "https://api.openai.com/v1",
		openaiModel: "gpt-4.1-mini",
		geminiApiKey: "",
		geminiBaseUrl: "https://generativelanguage.googleapis.com/v1beta/openai",
		geminiModel: "gemini-3-flash-preview",
		customOpenAiName: "自定义 OpenAI 兼容供应商",
		customOpenAiApiKey: "",
		customOpenAiBaseUrl: "",
		customOpenAiModel: "",
	};
}

export function createDefaultTranscriptProviderSettings(): TranscriptProviderSettings {
	return {
		deepgramApiKey: "",
		deepgramLanguage: "zh",
		assemblyApiKey: "",
		gladiaApiKey: "",
		gladiaLanguage: "zh",
		gladiaModel: "solaria-1",
		speechmaticsApiKey: "",
		speechmaticsLanguage: "cmn",
		speechmaticsRtUrl: "wss://eu2.rt.speechmatics.com/v2/",
		revaiApiKey: "",
		revaiLanguage: "cmn",
		revaiMetadata: "",
		macosSystemAudioBackend: "swift-helper",
	};
}

export function normalizeLlmProviderSettings(
	value: unknown,
): LlmProviderSettings {
	const defaults = createDefaultLlmProviderSettings();
	if (!value || typeof value !== "object") {
		return defaults;
	}

	const raw = value as Partial<LlmProviderSettings>;

	return {
		siliconflowApiKey: readString(raw.siliconflowApiKey),
		doubaoApiKey: readString(raw.doubaoApiKey),
		kimiApiKey: readString(raw.kimiApiKey),
		zhipuApiKey: readString(raw.zhipuApiKey),
		deepseekApiKey: readString(raw.deepseekApiKey),
		aliQwenApiKey: readString(raw.aliQwenApiKey),
		openaiApiKey: readString(raw.openaiApiKey),
		openaiBaseUrl: readString(raw.openaiBaseUrl, defaults.openaiBaseUrl),
		openaiModel: readString(raw.openaiModel, defaults.openaiModel),
		geminiApiKey: readString(raw.geminiApiKey),
		geminiBaseUrl: readString(raw.geminiBaseUrl, defaults.geminiBaseUrl),
		geminiModel: readString(raw.geminiModel, defaults.geminiModel),
		customOpenAiName: readString(
			raw.customOpenAiName,
			defaults.customOpenAiName,
		),
		customOpenAiApiKey: readString(raw.customOpenAiApiKey),
		customOpenAiBaseUrl: readString(raw.customOpenAiBaseUrl),
		customOpenAiModel: readString(raw.customOpenAiModel),
	};
}

export function normalizeTranscriptProviderSettings(
	value: unknown,
): TranscriptProviderSettings {
	const defaults = createDefaultTranscriptProviderSettings();
	if (!value || typeof value !== "object") {
		return defaults;
	}

	const raw = value as Partial<TranscriptProviderSettings>;

	return {
		deepgramApiKey: readString(raw.deepgramApiKey),
		deepgramLanguage: readString(
			raw.deepgramLanguage,
			defaults.deepgramLanguage,
		),
		assemblyApiKey: readString(raw.assemblyApiKey),
		gladiaApiKey: readString(raw.gladiaApiKey),
		gladiaLanguage: readString(raw.gladiaLanguage, defaults.gladiaLanguage),
		gladiaModel: readString(raw.gladiaModel, defaults.gladiaModel),
		speechmaticsApiKey: readString(raw.speechmaticsApiKey),
		speechmaticsLanguage: readString(
			raw.speechmaticsLanguage,
			defaults.speechmaticsLanguage,
		),
		speechmaticsRtUrl: readString(
			raw.speechmaticsRtUrl,
			defaults.speechmaticsRtUrl,
		),
		revaiApiKey: readString(raw.revaiApiKey),
		revaiLanguage: readString(raw.revaiLanguage, defaults.revaiLanguage),
		revaiMetadata: readString(raw.revaiMetadata),
		macosSystemAudioBackend:
			raw.macosSystemAudioBackend === "rust-native"
				? "rust-native"
				: defaults.macosSystemAudioBackend,
	};
}

export function createDefaultProviderEnvPresets(): ProviderEnvPresets {
	return {
		llm: createDefaultLlmProviderSettings(),
		transcript: createDefaultTranscriptProviderSettings(),
	};
}

export function mergeLlmApiKeyPresetsIntoSettings(
	currentSettings: LlmProviderSettings,
	presetSettings: LlmProviderSettings,
) {
	const nextSettings = {
		...normalizeLlmProviderSettings(currentSettings),
	};
	const normalizedPresetSettings = normalizeLlmProviderSettings(presetSettings);

	for (const field of LLM_API_KEY_FIELDS) {
		if (hasConfiguredValue(normalizedPresetSettings[field])) {
			nextSettings[field] = normalizedPresetSettings[field];
		}
	}

	return nextSettings;
}

export function stripLlmApiKeyPresetsFromSettings(
	currentSettings: LlmProviderSettings,
	presetSettings: LlmProviderSettings,
) {
	const nextSettings = {
		...normalizeLlmProviderSettings(currentSettings),
	};
	const normalizedPresetSettings = normalizeLlmProviderSettings(presetSettings);

	for (const field of LLM_API_KEY_FIELDS) {
		if (
			hasConfiguredValue(normalizedPresetSettings[field]) &&
			nextSettings[field] === normalizedPresetSettings[field]
		) {
			nextSettings[field] = "";
		}
	}

	return nextSettings;
}

export function normalizeProviderEnvPresets(
	value: unknown,
): ProviderEnvPresets {
	const defaults = createDefaultProviderEnvPresets();
	if (!value || typeof value !== "object") {
		return defaults;
	}

	const raw = value as Partial<ProviderEnvPresets>;

	return {
		llm: normalizeLlmProviderSettings(raw.llm),
		transcript: normalizeTranscriptProviderSettings(raw.transcript),
	};
}

export function hasConfiguredValue(value: unknown) {
	if (typeof value !== "string") {
		return false;
	}

	return value.trim().length > 0;
}

export function hasAnyTranscriptApiKeyConfigured(
	settings: TranscriptProviderSettings,
) {
	return [
		settings.deepgramApiKey,
		settings.assemblyApiKey,
		settings.gladiaApiKey,
		settings.speechmaticsApiKey,
		settings.revaiApiKey,
	].some(hasConfiguredValue);
}

export function getTranscriptProviderStatus(apiKey: string) {
	return hasConfiguredValue(apiKey) ? "已激活" : "未激活";
}

export function getLlmProviderStatus(
	localApiKey: string,
	presetApiKey: string,
) {
	if (hasConfiguredValue(localApiKey)) {
		return "已激活";
	}

	if (hasConfiguredValue(presetApiKey)) {
		return "免费额度";
	}

	return "未激活";
}
