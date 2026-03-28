use serde::{Deserialize, Serialize};
use std::env;

pub const SILICONFLOW_ENV_KEYS: &[&str] = &["SILICONFLOW_API_KEY", "Siliconflow"];
pub const DOUBAO_ENV_KEYS: &[&str] = &["DOUBAO_API_KEY", "DOUBAO"];
pub const KIMI_ENV_KEYS: &[&str] = &["KIMI_API_KEY", "KIMI"];
pub const ZHIPU_ENV_KEYS: &[&str] = &["ZHIPU_API_KEY", "ZHIPU"];
pub const DEEPSEEK_ENV_KEYS: &[&str] = &["DEEPSEEK_API_KEY", "DEEPSEEK"];
pub const ALI_QWEN_ENV_KEYS: &[&str] = &["ALI_QWEN_QWQ_API_KEY", "ALI_QWEN_QWQ"];
pub const OPENAI_ENV_KEYS: &[&str] = &["OPENAI_API_KEY", "OPENAI"];
pub const GEMINI_ENV_KEYS: &[&str] = &["GEMINI_API_KEY", "GOOGLE_GENAI_API_KEY"];
pub const ASSEMBLY_ENV_KEYS: &[&str] = &["ASSEMBLY_API_KEY"];
pub const DEEPGRAM_ENV_KEYS: &[&str] = &["DEEPGRAM_API_KEY"];
pub const GLADIA_ENV_KEYS: &[&str] = &["GLADIA_API_KEY"];
pub const SPEECHMATICS_ENV_KEYS: &[&str] = &["SPEECHMATICS_API_KEY"];
pub const REVAI_ENV_KEYS: &[&str] = &["REVAI_API_KEY"];

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmRuntimeConfig {
    pub siliconflow_api_key: Option<String>,
    pub doubao_api_key: Option<String>,
    pub kimi_api_key: Option<String>,
    pub zhipu_api_key: Option<String>,
    pub deepseek_api_key: Option<String>,
    pub ali_qwen_api_key: Option<String>,
    pub openai_api_key: Option<String>,
    pub openai_base_url: Option<String>,
    pub openai_model: Option<String>,
    pub gemini_api_key: Option<String>,
    pub gemini_base_url: Option<String>,
    pub gemini_model: Option<String>,
    pub custom_open_ai_name: Option<String>,
    pub custom_open_ai_api_key: Option<String>,
    pub custom_open_ai_base_url: Option<String>,
    pub custom_open_ai_model: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptRuntimeConfig {
    pub deepgram_api_key: Option<String>,
    pub deepgram_language: Option<String>,
    pub assembly_api_key: Option<String>,
    pub gladia_api_key: Option<String>,
    pub gladia_language: Option<String>,
    pub gladia_model: Option<String>,
    pub speechmatics_api_key: Option<String>,
    pub speechmatics_language: Option<String>,
    pub speechmatics_rt_url: Option<String>,
    pub revai_api_key: Option<String>,
    pub revai_language: Option<String>,
    pub revai_metadata: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderEnvPresets {
    pub llm: LlmRuntimeConfig,
    pub transcript: TranscriptRuntimeConfig,
}

pub fn normalize_optional_string(value: Option<&str>) -> Option<String> {
    let trimmed = value?.trim();
    if trimmed.is_empty() {
        return None;
    }

    Some(trimmed.to_string())
}

pub fn resolve_optional_string(override_value: Option<&str>, env_keys: &[&str]) -> Option<String> {
    if let Some(value) = normalize_optional_string(override_value) {
        return Some(value);
    }

    env_keys.iter().find_map(|key| {
        env::var(key)
            .ok()
            .and_then(|value| normalize_optional_string(Some(value.as_str())))
    })
}

pub fn resolve_required_string(
    override_value: Option<&str>,
    env_keys: &[&str],
    label: &str,
) -> Result<String, String> {
    resolve_optional_string(override_value, env_keys)
        .ok_or_else(|| format!("缺少 {label} 配置，请在前端填写或在环境变量中提供"))
}

pub fn resolve_string_or_default(
    override_value: Option<&str>,
    env_keys: &[&str],
    default_value: &str,
) -> String {
    resolve_optional_string(override_value, env_keys).unwrap_or_else(|| default_value.to_string())
}

pub fn llm_runtime_config_from_env() -> LlmRuntimeConfig {
    LlmRuntimeConfig {
        siliconflow_api_key: resolve_optional_string(None, SILICONFLOW_ENV_KEYS),
        doubao_api_key: resolve_optional_string(None, DOUBAO_ENV_KEYS),
        kimi_api_key: resolve_optional_string(None, KIMI_ENV_KEYS),
        zhipu_api_key: resolve_optional_string(None, ZHIPU_ENV_KEYS),
        deepseek_api_key: resolve_optional_string(None, DEEPSEEK_ENV_KEYS),
        ali_qwen_api_key: resolve_optional_string(None, ALI_QWEN_ENV_KEYS),
        openai_api_key: resolve_optional_string(None, OPENAI_ENV_KEYS),
        openai_base_url: resolve_optional_string(None, &["OPENAI_BASE_URL"]),
        openai_model: resolve_optional_string(None, &["OPENAI_MODEL"]),
        gemini_api_key: resolve_optional_string(None, GEMINI_ENV_KEYS),
        gemini_base_url: resolve_optional_string(None, &["GEMINI_BASE_URL"]),
        gemini_model: resolve_optional_string(None, &["GEMINI_MODEL"]),
        custom_open_ai_name: resolve_optional_string(None, &["CUSTOM_OPENAI_NAME"]),
        custom_open_ai_api_key: resolve_optional_string(None, &["CUSTOM_OPENAI_API_KEY"]),
        custom_open_ai_base_url: resolve_optional_string(None, &["CUSTOM_OPENAI_BASE_URL"]),
        custom_open_ai_model: resolve_optional_string(None, &["CUSTOM_OPENAI_MODEL"]),
    }
}

pub fn transcript_runtime_config_from_env() -> TranscriptRuntimeConfig {
    TranscriptRuntimeConfig {
        deepgram_api_key: resolve_optional_string(None, DEEPGRAM_ENV_KEYS),
        deepgram_language: resolve_optional_string(None, &["DEEPGRAM_LANGUAGE"]),
        assembly_api_key: resolve_optional_string(None, ASSEMBLY_ENV_KEYS),
        gladia_api_key: resolve_optional_string(None, GLADIA_ENV_KEYS),
        gladia_language: resolve_optional_string(None, &["GLADIA_LANGUAGE"]),
        gladia_model: resolve_optional_string(None, &["GLADIA_MODEL"]),
        speechmatics_api_key: resolve_optional_string(None, SPEECHMATICS_ENV_KEYS),
        speechmatics_language: resolve_optional_string(None, &["SPEECHMATICS_LANGUAGE"]),
        speechmatics_rt_url: resolve_optional_string(None, &["SPEECHMATICS_RT_URL"]),
        revai_api_key: resolve_optional_string(None, REVAI_ENV_KEYS),
        revai_language: resolve_optional_string(None, &["REVAI_LANGUAGE"]),
        revai_metadata: resolve_optional_string(None, &["REVAI_METADATA"]),
    }
}

pub fn provider_env_presets_from_env() -> ProviderEnvPresets {
    ProviderEnvPresets {
        llm: llm_runtime_config_from_env(),
        transcript: transcript_runtime_config_from_env(),
    }
}
