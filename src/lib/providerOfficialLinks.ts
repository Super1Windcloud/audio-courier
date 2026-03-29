import type {
	LlmProviderSettings,
	TranscriptProviderSettings,
} from "@/types/provider.ts";

export interface ProviderOfficialLink {
	label?: string;
	url: string;
}

const OPENAI_REFERENCE_LINK: ProviderOfficialLink = {
	label: "官网",
	url: "https://developers.openai.com/api/reference/overview",
};

const OPENAI_COMPATIBILITY_LINK: ProviderOfficialLink = {
	label: "兼容规范",
	url: "https://developers.openai.com/api/reference/overview",
};

const GEMINI_REFERENCE_LINK: ProviderOfficialLink = {
	label: "官网",
	url: "https://ai.google.dev/gemini-api/docs/openai",
};

export const llmProviderOfficialLinks: Partial<
	Record<keyof LlmProviderSettings, ProviderOfficialLink>
> = {
	siliconflowApiKey: {
		label: "官网",
		url: "https://docs.siliconflow.com/en/userguide/introduction",
	},
	doubaoApiKey: {
		label: "官网",
		url: "https://www.volcengine.com/docs/82379?lang=zh",
	},
	kimiApiKey: {
		label: "官网",
		url: "https://platform.moonshot.ai/docs/overview",
	},
	zhipuApiKey: {
		label: "官网",
		url: "https://docs.bigmodel.cn/cn/guide/start/introduction",
	},
	deepseekApiKey: {
		label: "官网",
		url: "https://api-docs.deepseek.com/",
	},
	aliQwenApiKey: {
		label: "官网",
		url: "https://help.aliyun.com/zh/model-studio/first-api-call-to-qwen",
	},
	openaiApiKey: OPENAI_REFERENCE_LINK,
	openaiBaseUrl: OPENAI_REFERENCE_LINK,
	openaiModel: OPENAI_REFERENCE_LINK,
	geminiApiKey: GEMINI_REFERENCE_LINK,
	geminiBaseUrl: GEMINI_REFERENCE_LINK,
	geminiModel: GEMINI_REFERENCE_LINK,
	customOpenAiName: OPENAI_COMPATIBILITY_LINK,
	customOpenAiApiKey: OPENAI_COMPATIBILITY_LINK,
	customOpenAiBaseUrl: OPENAI_COMPATIBILITY_LINK,
	customOpenAiModel: OPENAI_COMPATIBILITY_LINK,
};

export const transcriptProviderOfficialLinks: Partial<
	Record<keyof TranscriptProviderSettings, ProviderOfficialLink>
> = {
	deepgramApiKey: {
		label: "官网",
		url: "https://developers.deepgram.com/docs/stt/getting-started",
	},
	deepgramLanguage: {
		label: "官网",
		url: "https://developers.deepgram.com/docs/stt/getting-started",
	},
	assemblyApiKey: {
		label: "官网",
		url: "https://www.assemblyai.com/docs/",
	},
	gladiaApiKey: {
		label: "官网",
		url: "https://docs.gladia.io/api-reference/live-flow",
	},
	gladiaLanguage: {
		label: "官网",
		url: "https://docs.gladia.io/api-reference/live-flow",
	},
	gladiaModel: {
		label: "官网",
		url: "https://docs.gladia.io/api-reference/live-flow",
	},
	speechmaticsApiKey: {
		label: "官网",
		url: "https://docs.speechmatics.com/",
	},
	speechmaticsLanguage: {
		label: "官网",
		url: "https://docs.speechmatics.com/",
	},
	speechmaticsRtUrl: {
		label: "官网",
		url: "https://docs.speechmatics.com/",
	},
	revaiApiKey: {
		label: "官网",
		url: "https://docs.rev.ai/api/streaming/get-started/",
	},
	revaiLanguage: {
		label: "官网",
		url: "https://docs.rev.ai/api/streaming/get-started/",
	},
	revaiMetadata: {
		label: "官网",
		url: "https://docs.rev.ai/api/streaming/get-started/",
	},
};
