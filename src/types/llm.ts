export const MODEL_OPTIONS = [
  "siliconflow",
  "doubao_lite",
  "doubao_pro",
  "kimi",
  "zhipu",
  "deepseek_api",
  "ali_qwen_32b",
  "ali_qwen_2_5",
  "ali_qwen_plus",
  "ali_qwen_max",
  "doubao_deepseek",
] as const;

export type ModelOption = typeof MODEL_OPTIONS[number];
