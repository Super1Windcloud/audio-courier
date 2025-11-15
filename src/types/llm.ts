export const MODEL_OPTIONS = [
	"siliconflow_free",
	"siliconflow_pro",
	"doubao_lite",
	"doubao_pro",
	"doubao_seed_flash",
	"doubao_seed",
	"kimi",
	"zhipu",
	"deepseek_api",
	"ali_qwen_2_5",
	"ali_qwen_plus_latest",
	"ali_qwen_max",
] as const;

export type ModelOption = (typeof MODEL_OPTIONS)[number];

export const HOTKEYS = ["显示/隐藏 Ctrl+Shift+`", "切换录音 Alt+Space"];
