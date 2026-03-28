use serde::{Deserialize, Serialize};
use std::env;

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
