mod api;
use crate::provider_config::{
    ALI_QWEN_ENV_KEYS, DEEPSEEK_ENV_KEYS, DOUBAO_ENV_KEYS, GEMINI_ENV_KEYS, KIMI_ENV_KEYS,
    LlmRuntimeConfig, OPENAI_ENV_KEYS, SILICONFLOW_ENV_KEYS, ZHIPU_ENV_KEYS,
    resolve_required_string, resolve_string_or_default,
};
use api::*;
use rand::{RngExt, rng as thread_rng};
use serde_json::json;
#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FlowArgs {
    question: String,
    llm_prompt: String,
    request_id: Option<String>,
}

impl FlowArgs {
    pub fn new(question: impl Into<String>, llm_prompt: impl Into<String>) -> Self {
        Self {
            question: question.into(),
            llm_prompt: llm_prompt.into(),
            request_id: None,
        }
    }

    pub fn set_request_id(mut self, request_id: Option<String>) -> Self {
        self.request_id = request_id;
        self
    }
}

const FREE_MODELS: [&str; 0] = [];

struct ResolvedLlmProvider {
    model: String,
    base_url: String,
    api_key: String,
    max_tokens: u32,
    temperature: f32,
    prompt_role: &'static str,
    enable_thinking: Option<bool>,
}

pub fn siliconflow_free_models() -> &'static [&'static str] {
    &FREE_MODELS
}

pub async fn siliconflow_free_with_model(
    app: tauri::AppHandle,
    flow_args: FlowArgs,
    model: &str,
) -> Result<String, String> {
    let api_key = get_env_key("Siliconflow");
    let messages = vec![
        json!({"role":"assistant","content":flow_args.llm_prompt}),
        json!({"role":"user","content":flow_args.question}),
    ];

    call_model_api(
        app,
        ModelRequest {
            model: model.to_string(),
            messages,
            base_url: "https://api.siliconflow.cn/v1".to_string(),
            api_key,
            max_tokens: 4096,
            temperature: 0.7,
            enable_thinking: None,
        },
        flow_args.request_id,
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn siliconflow_free(
    app: tauri::AppHandle,
    flow_args: FlowArgs,
) -> Result<String, String> {
    let idx = {
        let mut rng = thread_rng();
        rng.random_range(0..FREE_MODELS.len())
    };
    let random_model = FREE_MODELS[idx];
    println!("随机选择的模型: {}", random_model);
    siliconflow_free_with_model(app, flow_args, random_model).await
}

const PRO_MODELS: [&str; 23] = [
    "Pro/zai-org/GLM-5",
    "Pro/zai-org/GLM-4.7",
    "deepseek-ai/DeepSeek-V3.2",
    "Pro/deepseek-ai/DeepSeek-V3.2",
    "zai-org/GLM-4.6",
    "Qwen/Qwen3-8B",
    "Qwen/Qwen3-14B",
    "Qwen/Qwen3-32B",
    "Qwen/Qwen3-30B-A3B",
    "tencent/Hunyuan-A13B-Instruct",
    "zai-org/GLM-4.5V",
    "deepseek-ai/DeepSeek-V3.1-Terminus",
    "Pro/deepseek-ai/DeepSeek-V3.1-Terminus",
    "Qwen/Qwen3.5-397B-A17B",
    "Qwen/Qwen3.5-122B-A10B",
    "Qwen/Qwen3.5-35B-A3B",
    "Qwen/Qwen2.5-14B-Instruct",
    "Qwen/Qwen2.5-32B-Instruct",
    "inclusionAI/Ling-flash-2.0",
    "Qwen/Qwen2.5-72B-Instruct-128K",
    "zai-org/GLM-4.5-Air",
    "deepseek-ai/DeepSeek-V3",
    "baidu/ERNIE-4.5-300B-A47B",
];
const SILICONFLOW_MINIMAX_M2_5_MODEL: &str = "Pro/MiniMaxAI/MiniMax-M2.5";

pub fn siliconflow_pro_models() -> &'static [&'static str] {
    &PRO_MODELS
}

fn resolve_siliconflow_provider(
    runtime_config: &LlmRuntimeConfig,
    model: impl Into<String>,
) -> Result<ResolvedLlmProvider, String> {
    Ok(ResolvedLlmProvider {
        model: model.into(),
        base_url: "https://api.siliconflow.cn/v1".to_string(),
        api_key: resolve_required_string(
            runtime_config.siliconflow_api_key.as_deref(),
            SILICONFLOW_ENV_KEYS,
            "SILICONFLOW_API_KEY",
        )?,
        max_tokens: 4096,
        temperature: 0.7,
        prompt_role: "assistant",
        enable_thinking: Some(false),
    })
}

fn build_messages(flow_args: &FlowArgs, prompt_role: &str) -> Vec<serde_json::Value> {
    vec![
        json!({"role": prompt_role, "content": flow_args.llm_prompt}),
        json!({"role":"user","content":flow_args.question}),
    ]
}

fn resolve_provider(
    provider: &str,
    runtime_config: &LlmRuntimeConfig,
) -> Result<ResolvedLlmProvider, String> {
    let provider = provider.trim();

    match provider {
        "siliconflow_pro" => resolve_siliconflow_provider(runtime_config, {
            let idx = {
                let mut rng = thread_rng();
                rng.random_range(0..PRO_MODELS.len())
            };
            PRO_MODELS[idx].to_string()
        }),
        "siliconflow_minimax_m2_5" => {
            resolve_siliconflow_provider(runtime_config, SILICONFLOW_MINIMAX_M2_5_MODEL)
        }
        "doubao_lite" => Ok(ResolvedLlmProvider {
            model: "doubao-1.5-lite-32k-250115".to_string(),
            base_url: "https://ark.cn-beijing.volces.com/api/v3".to_string(),
            api_key: resolve_required_string(
                runtime_config.doubao_api_key.as_deref(),
                DOUBAO_ENV_KEYS,
                "DOUBAO_API_KEY",
            )?,
            max_tokens: 4096,
            temperature: 0.7,
            prompt_role: "system",
            enable_thinking: None,
        }),
        "doubao_pro" => Ok(ResolvedLlmProvider {
            model: "doubao-1.5-pro-32k-250115".to_string(),
            base_url: "https://ark.cn-beijing.volces.com/api/v3".to_string(),
            api_key: resolve_required_string(
                runtime_config.doubao_api_key.as_deref(),
                DOUBAO_ENV_KEYS,
                "DOUBAO_API_KEY",
            )?,
            max_tokens: 4096,
            temperature: 0.7,
            prompt_role: "assistant",
            enable_thinking: None,
        }),
        "kimi" => Ok(ResolvedLlmProvider {
            model: "kimi-k2-0905-preview".to_string(),
            base_url: "https://api.moonshot.cn/v1".to_string(),
            api_key: resolve_required_string(
                runtime_config.kimi_api_key.as_deref(),
                KIMI_ENV_KEYS,
                "KIMI_API_KEY",
            )?,
            max_tokens: 4096,
            temperature: 0.7,
            prompt_role: "system",
            enable_thinking: None,
        }),
        "zhipu" => Ok(ResolvedLlmProvider {
            model: "glm-4.5".to_string(),
            base_url: "https://open.bigmodel.cn/api/paas/v4".to_string(),
            api_key: resolve_required_string(
                runtime_config.zhipu_api_key.as_deref(),
                ZHIPU_ENV_KEYS,
                "ZHIPU_API_KEY",
            )?,
            max_tokens: 4096,
            temperature: 0.618,
            prompt_role: "system",
            enable_thinking: None,
        }),
        "deepseek_api" => Ok(ResolvedLlmProvider {
            model: "deepseek-chat".to_string(),
            base_url: "https://api.deepseek.com".to_string(),
            api_key: resolve_required_string(
                runtime_config.deepseek_api_key.as_deref(),
                DEEPSEEK_ENV_KEYS,
                "DEEPSEEK_API_KEY",
            )?,
            max_tokens: 4096,
            temperature: 0.7,
            prompt_role: "system",
            enable_thinking: None,
        }),
        "ali_qwen_2_5" => Ok(ResolvedLlmProvider {
            model: "qwen2.5-14b-instruct-1m".to_string(),
            base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1".to_string(),
            api_key: resolve_required_string(
                runtime_config.ali_qwen_api_key.as_deref(),
                ALI_QWEN_ENV_KEYS,
                "ALI_QWEN_QWQ_API_KEY",
            )?,
            max_tokens: 4096,
            temperature: 0.7,
            prompt_role: "system",
            enable_thinking: None,
        }),
        "ali_qwen_plus_latest" => Ok(ResolvedLlmProvider {
            model: "qwen-plus".to_string(),
            base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1".to_string(),
            api_key: resolve_required_string(
                runtime_config.ali_qwen_api_key.as_deref(),
                ALI_QWEN_ENV_KEYS,
                "ALI_QWEN_QWQ_API_KEY",
            )?,
            max_tokens: 4096,
            temperature: 0.7,
            prompt_role: "system",
            enable_thinking: None,
        }),
        "ali_qwen_max" => Ok(ResolvedLlmProvider {
            model: "qwen-max-2025-01-25".to_string(),
            base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1".to_string(),
            api_key: resolve_required_string(
                runtime_config.ali_qwen_api_key.as_deref(),
                ALI_QWEN_ENV_KEYS,
                "ALI_QWEN_QWQ_API_KEY",
            )?,
            max_tokens: 4096,
            temperature: 0.7,
            prompt_role: "system",
            enable_thinking: None,
        }),
        "openai" => Ok(ResolvedLlmProvider {
            model: resolve_string_or_default(
                runtime_config.openai_model.as_deref(),
                &["OPENAI_MODEL"],
                "gpt-4.1-mini",
            ),
            base_url: resolve_string_or_default(
                runtime_config.openai_base_url.as_deref(),
                &["OPENAI_BASE_URL"],
                "https://api.openai.com/v1",
            ),
            api_key: resolve_required_string(
                runtime_config.openai_api_key.as_deref(),
                OPENAI_ENV_KEYS,
                "OPENAI_API_KEY",
            )?,
            max_tokens: 4096,
            temperature: 0.7,
            prompt_role: "system",
            enable_thinking: None,
        }),
        "gemini" => Ok(ResolvedLlmProvider {
            model: resolve_string_or_default(
                runtime_config.gemini_model.as_deref(),
                &["GEMINI_MODEL"],
                "gemini-3-flash-preview",
            ),
            base_url: resolve_string_or_default(
                runtime_config.gemini_base_url.as_deref(),
                &["GEMINI_BASE_URL"],
                "https://generativelanguage.googleapis.com/v1beta/openai",
            ),
            api_key: resolve_required_string(
                runtime_config.gemini_api_key.as_deref(),
                GEMINI_ENV_KEYS,
                "GEMINI_API_KEY",
            )?,
            max_tokens: 4096,
            temperature: 0.7,
            prompt_role: "system",
            enable_thinking: None,
        }),
        "custom_openai" => Ok(ResolvedLlmProvider {
            model: resolve_required_string(
                runtime_config.custom_open_ai_model.as_deref(),
                &["CUSTOM_OPENAI_MODEL"],
                "自定义 OpenAI 兼容模型名",
            )?,
            base_url: resolve_required_string(
                runtime_config.custom_open_ai_base_url.as_deref(),
                &["CUSTOM_OPENAI_BASE_URL"],
                "自定义 OpenAI 兼容 Base URL",
            )?,
            api_key: resolve_required_string(
                runtime_config.custom_open_ai_api_key.as_deref(),
                &["CUSTOM_OPENAI_API_KEY"],
                "自定义 OpenAI 兼容 API Key",
            )?,
            max_tokens: 4096,
            temperature: 0.7,
            prompt_role: "system",
            enable_thinking: None,
        }),
        _ => Err(format!("不支持的大模型供应商: {provider}")),
    }
}

#[tauri::command]
pub async fn chat_with_llm_provider(
    app: tauri::AppHandle,
    provider: String,
    flow_args: FlowArgs,
    runtime_config: Option<LlmRuntimeConfig>,
) -> Result<String, String> {
    let runtime_config = runtime_config.unwrap_or_default();
    let resolved = resolve_provider(&provider, &runtime_config)?;
    let messages = build_messages(&flow_args, resolved.prompt_role);

    call_model_api(
        app,
        ModelRequest {
            model: resolved.model,
            messages,
            base_url: resolved.base_url,
            api_key: resolved.api_key,
            max_tokens: resolved.max_tokens,
            temperature: resolved.temperature,
            enable_thinking: resolved.enable_thinking,
        },
        flow_args.request_id,
    )
    .await
    .map_err(|error| error.to_string())
}

pub async fn siliconflow_pro_with_model(
    app: tauri::AppHandle,
    flow_args: FlowArgs,
    model: &str,
) -> Result<String, String> {
    let api_key = get_env_key("SILICONFLOW_API_KEY");
    let messages = vec![
        json!({"role":"assistant","content":flow_args.llm_prompt}),
        json!({"role":"user","content":flow_args.question}),
    ];

    call_model_api(
        app,
        ModelRequest {
            model: model.to_string(),
            messages,
            base_url: "https://api.siliconflow.cn/v1".to_string(),
            api_key,
            max_tokens: 4096,
            temperature: 0.7,
            enable_thinking: None,
        },
        flow_args.request_id,
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn siliconflow_pro(app: tauri::AppHandle, flow_args: FlowArgs) -> Result<String, String> {
    let idx = {
        let mut rng = thread_rng();
        rng.random_range(0..PRO_MODELS.len())
    };

    let random_model = PRO_MODELS[idx];
    println!("随机选择的模型: {}", random_model);
    siliconflow_pro_with_model(app, flow_args, random_model).await
}

#[tauri::command]
pub async fn siliconflow_minimax_m2_5(
    app: tauri::AppHandle,
    flow_args: FlowArgs,
) -> Result<String, String> {
    siliconflow_pro_with_model(app, flow_args, SILICONFLOW_MINIMAX_M2_5_MODEL).await
}

#[tauri::command]
pub async fn doubao_lite(app: tauri::AppHandle, flow_args: FlowArgs) -> Result<String, String> {
    let api_key = get_env_key("DOUBAO_API_KEY");
    let messages = vec![
        json!({"role":"system","content":flow_args.llm_prompt}),
        json!({"role":"user","content":flow_args.question}),
    ];

    call_model_api(
        app,
        ModelRequest {
            model: "doubao-1.5-lite-32k-250115".to_string(),
            messages,
            base_url: "https://ark.cn-beijing.volces.com/api/v3".to_string(),
            api_key,
            max_tokens: 4096,
            temperature: 0.7,
            enable_thinking: None,
        },
        flow_args.request_id,
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn doubao_pro(app: tauri::AppHandle, flow_args: FlowArgs) -> Result<String, String> {
    let api_key = get_env_key("DOUBAO_API_KEY");
    let messages = vec![
        json!({"role":"assistant","content":flow_args.llm_prompt}),
        json!({"role":"user","content":flow_args.question}),
    ];

    call_model_api(
        app,
        ModelRequest {
            model: "doubao-1.5-pro-32k-250115".to_string(),
            messages,
            base_url: "https://ark.cn-beijing.volces.com/api/v3".to_string(),
            api_key,
            max_tokens: 4096,
            temperature: 0.7,
            enable_thinking: None,
        },
        flow_args.request_id,
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn doubao_seed_flash(
    app: tauri::AppHandle,
    flow_args: FlowArgs,
) -> Result<String, String> {
    let api_key = get_env_key("DOUBAO_API_KEY");
    let messages = vec![
        json!({"role":"assistant","content":flow_args.llm_prompt}),
        json!({"role":"user","content":flow_args.question}),
    ];

    call_model_api(
        app,
        ModelRequest {
            model: "doubao-seed-1-6-flash-250828".to_string(),
            messages,
            base_url: "https://ark.cn-beijing.volces.com/api/v3".to_string(),
            api_key,
            max_tokens: 4096,
            temperature: 0.7,
            enable_thinking: None,
        },
        flow_args.request_id,
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn doubao_seed(app: tauri::AppHandle, flow_args: FlowArgs) -> Result<String, String> {
    let api_key = get_env_key("DOUBAO_API_KEY");
    let messages = vec![
        json!({"role":"assistant","content":flow_args.llm_prompt}),
        json!({"role":"user","content":flow_args.question}),
    ];

    call_model_api(
        app,
        ModelRequest {
            model: "doubao-seed-1-6-250615".to_string(),
            messages,
            base_url: "https://ark.cn-beijing.volces.com/api/v3".to_string(),
            api_key,
            max_tokens: 4096,
            temperature: 0.7,
            enable_thinking: None,
        },
        flow_args.request_id,
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]

pub async fn kimi(app: tauri::AppHandle, flow_args: FlowArgs) -> Result<String, String> {
    let api_key = get_env_key("KIMI_API_KEY");
    let messages = vec![
        json!({"role":"system","content":flow_args.llm_prompt}),
        json!({"role":"user","content":flow_args.question}),
    ];

    call_model_api(
        app,
        ModelRequest {
            model: "kimi-k2-0905-preview".to_string(),
            messages,
            base_url: "https://api.moonshot.cn/v1".to_string(),
            api_key,
            max_tokens: 4096,
            temperature: 0.7,
            enable_thinking: None,
        },
        flow_args.request_id,
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]

pub async fn zhipu(app: tauri::AppHandle, flow_args: FlowArgs) -> Result<String, String> {
    let api_key = get_env_key("ZHIPU_API_KEY");
    let messages = vec![
        json!({"role":"system","content":flow_args.llm_prompt}),
        json!({"role":"user","content":flow_args.question}),
    ];

    call_model_api(
        app,
        ModelRequest {
            model: "glm-4.5".to_string(),
            messages,
            base_url: "https://open.bigmodel.cn/api/paas/v4".to_string(),
            api_key,
            max_tokens: 4096,
            temperature: 0.618,
            enable_thinking: None,
        },
        flow_args.request_id,
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]

pub async fn deepseek_api(app: tauri::AppHandle, flow_args: FlowArgs) -> Result<String, String> {
    let api_key = get_env_key("DEEPSEEK_API_KEY");
    let messages = vec![
        json!({"role":"system","content":flow_args.llm_prompt}),
        json!({"role":"user","content":flow_args.question}),
    ];

    call_model_api(
        app,
        ModelRequest {
            model: "deepseek-chat".to_string(),
            messages,
            base_url: "https://api.deepseek.com".to_string(),
            api_key,
            max_tokens: 4096,
            temperature: 0.7,
            enable_thinking: None,
        },
        flow_args.request_id,
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]

pub async fn ali_qwen_2_5(app: tauri::AppHandle, flow_args: FlowArgs) -> Result<String, String> {
    let api_key = get_env_key("ALI_QWEN_QWQ_API_KEY");
    let messages = vec![
        json!({"role":"system","content":flow_args.llm_prompt}),
        json!({"role":"user","content":flow_args.question}),
    ];

    call_model_api(
        app,
        ModelRequest {
            model: "qwen2.5-14b-instruct-1m".to_string(),
            messages,
            base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1".to_string(),
            api_key,
            max_tokens: 4096,
            temperature: 0.7,
            enable_thinking: None,
        },
        flow_args.request_id,
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn ali_qwen_plus_latest(
    app: tauri::AppHandle,
    flow_args: FlowArgs,
) -> Result<String, String> {
    let api_key = get_env_key("ALI_QWEN_QWQ_API_KEY");
    let messages = vec![
        json!({"role":"system","content":flow_args.llm_prompt}),
        json!({"role":"user","content":flow_args.question}),
    ];

    call_model_api(
        app,
        ModelRequest {
            model: "qwen-plus".to_string(),
            messages,
            base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1".to_string(),
            api_key,
            max_tokens: 4096,
            temperature: 0.7,
            enable_thinking: None,
        },
        flow_args.request_id,
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn ali_qwen_max(app: tauri::AppHandle, flow_args: FlowArgs) -> Result<String, String> {
    let api_key = get_env_key("ALI_QWEN_QWQ_API_KEY");
    let messages = vec![
        json!({"role":"system","content":flow_args.llm_prompt}),
        json!({"role":"user","content":flow_args.question}),
    ];

    call_model_api(
        app,
        ModelRequest {
            model: "qwen-max-2025-01-25".to_string(),
            messages,
            base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1".to_string(),
            api_key,
            max_tokens: 4096,
            temperature: 0.7,
            enable_thinking: None,
        },
        flow_args.request_id,
    )
    .await
    .map_err(|e| e.to_string())
}
