export const MODEL_OPTIONS = [
	"siliconflow_pro",
	"siliconflow_minimax_m2_5",
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
	siliconflow_minimax_m2_5: "MiniMax M2.5",
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

export interface HotkeyHelpItem {
	label: string;
	combo: string;
	description: string;
}

export const HOTKEYS: HotkeyHelpItem[] = [
	{
		label: "显示/隐藏窗口",
		combo: "Ctrl+Shift+`",
		description: "全局快捷键",
	},
	{
		label: "切换录音",
		combo: "Alt+Space",
		description: "全局快捷键",
	},
	{
		label: "发送消息",
		combo: "Enter",
		description: "输入框聚焦时发送，Shift+Enter 换行",
	},
	{
		label: "快速发送",
		combo: "Shift+Enter",
		description: "输入框未聚焦时发送当前消息",
	},
	{
		label: "开发者工具",
		combo: "Ctrl+F12",
		description: "全局快捷键",
	},
];
