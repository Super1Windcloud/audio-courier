export const MODEL_OPTIONS = [
	"siliconflow_pro",
	"doubao_lite",
	"doubao_pro",
	"kimi",
	"zhipu",
	"deepseek_api",
	"ali_qwen_2_5",
	"ali_qwen_plus_latest",
	"ali_qwen_max",
	"openai",
	"gemini",
	"custom_openai",
] as const;

export type ModelOption = (typeof MODEL_OPTIONS)[number];

export const MODEL_LABELS: Record<ModelOption, string> = {
	siliconflow_pro: "SiliconFlow Pro",
	doubao_lite: "Doubao Lite",
	doubao_pro: "Doubao Pro",
	kimi: "Kimi",
	zhipu: "Zhipu",
	deepseek_api: "DeepSeek",
	ali_qwen_2_5: "Qwen 2.5",
	ali_qwen_plus_latest: "Qwen Plus",
	ali_qwen_max: "Qwen Max",
	openai: "OpenAI",
	gemini: "Gemini",
	custom_openai: "自定义 OpenAI 兼容",
};

export const HOTKEYS = ["显示/隐藏 Ctrl+Shift+`", "切换录音 Alt+Space"];
