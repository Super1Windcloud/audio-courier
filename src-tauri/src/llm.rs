mod api;
mod stt;
mod stt_realtime;

pub use stt_realtime::*; 
use api::*;
use rand::{rng as thread_rng, Rng};
use serde_json::json;
#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FlowArgs {
    question: String,
    llm_prompt: String,
}

const FREE_MODELS: [&str; 8] = [
    "Qwen/Qwen2.5-Coder-32B-Instruct",
    "Qwen/Qwen2.5-7B-Instruct",
    "Qwen/Qwen2-7B-Instruct",
    "tencent/Hunyuan-MT-7B",
    "THUDM/GLM-Z1-9B-0414",
    "THUDM/GLM-4-9B-0414",
    "internlm/internlm2_5-7b-chat",
    "THUDM/glm-4-9b-chat",
];

#[tauri::command]
pub async fn siliconflow_free(
    app: tauri::AppHandle,
    flow_args: FlowArgs,
) -> Result<String, String> {
    let api_key = get_env_key("Siliconflow");
    let messages = vec![
        json!({"role":"assistant","content":flow_args.llm_prompt}),
        json!({"role":"user","content":flow_args.question}),
    ];
    let idx = {
        let mut rng = thread_rng();
        rng.random_range(0..FREE_MODELS.len())
    };
    let random_model = FREE_MODELS[idx];
    // let random_model = "internlm/internlm2_5-7b-chat";
    println!("随机选择的模型: {}", random_model);
    call_model_api(
        app,
        ModelRequest {
            model: random_model.to_string(),
            messages,
            base_url: "https://api.siliconflow.cn/v1".to_string(),
            api_key,
            max_tokens: 4096,
            temperature: 0.7,
        },
    )
    .await
    .map_err(|e| e.to_string())
}

const PRO_MODELS: [&str; 43] = [
    "Pro/deepseek-ai/DeepSeek-R1-Distill-Qwen-7B",
    "Pro/Qwen/Qwen2.5-Coder-7B-Instruct",
    "Pro/Qwen/Qwen2.5-VL-7B-Instruct",
    "Pro/Qwen/Qwen2.5-7B-Instruct",
    "Pro/Qwen/Qwen2-7B-Instruct",
    "Pro/THUDM/glm-4-9b-chat",
    "deepseek-ai/DeepSeek-R1-Distill-Qwen-14B",
    "Qwen/Qwen2.5-14B-Instruct",
    "deepseek-ai/deepseek-vl2",
    "Qwen/Qwen2.5-Coder-32B-Instruct",
    "deepseek-ai/DeepSeek-R1-Distill-Qwen-32B",
    "Qwen/Qwen2.5-32B-Instruct",
    "deepseek-ai/DeepSeek-V2.5",
    "Qwen/Qwen2.5-VL-32B-Instruct",
    "THUDM/GLM-4-32B-0414",
    "Qwen/Qwen3-14B",
    "Qwen/Qwen3-30B-A3B-Instruct-2507",
    "Qwen/Qwen3-30B-A3B",
    "Qwen/Qwen3-Coder-30B-A3B-Instruct",
    "Qwen/Qwen3-Next-80B-A3B-Instruct",
    "inclusionAI/Ling-flash-2.0",
    "tencent/Hunyuan-A13B-Instruct",
    "Qwen/Qwen3-32B",
    "Tongyi-Zhiwen/QwenLong-L1-32B",
    "ByteDance-Seed/Seed-OSS-36B-Instruct",
    "ascend-tribe/pangu-pro-moe",
    "THUDM/GLM-Z1-Rumination-32B-0414",
    "THUDM/GLM-Z1-32B-0414",
    "Qwen/QwQ-32B",
    "Qwen/Qwen2.5-72B-Instruct",
    "Qwen/Qwen2.5-VL-72B-Instruct",
    "Qwen/Qwen2.5-72B-Instruct-128K",
    "Qwen/Qwen2-VL-72B-Instruct",
    "zai-org/GLM-4.5V",
    "zai-org/GLM-4.5-Air",
    "Pro/deepseek-ai/DeepSeek-V3",
    "deepseek-ai/DeepSeek-V3",
    "moonshotai/Kimi-Dev-72B",
    "baidu/ERNIE-4.5-300B-A47B",
    "Qwen/Qwen3-235B-A22B",
    "Pro/deepseek-ai/DeepSeek-V3.1",
    "deepseek-ai/DeepSeek-V3.1",
    "zai-org/GLM-4.5",
];

#[tauri::command]
pub async fn siliconflow_pro(app: tauri::AppHandle, flow_args: FlowArgs) -> Result<String, String> {
    let api_key = get_env_key("Siliconflow");
    let messages = vec![
        json!({"role":"assistant","content":flow_args.llm_prompt}),
        json!({"role":"user","content":flow_args.question}),
    ];
    let idx = {
        let mut rng = thread_rng();
        rng.random_range(0..PRO_MODELS.len())
    };

    let random_model = PRO_MODELS[idx];
    println!("随机选择的模型: {}", random_model);
    call_model_api(
        app,
        ModelRequest {
            model: random_model.to_string(),
            messages,
            base_url: "https://api.siliconflow.cn/v1".to_string(),
            api_key,
            max_tokens: 4096,
            temperature: 0.7,
        },
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn doubao_lite(app: tauri::AppHandle, flow_args: FlowArgs) -> Result<String, String> {
    let api_key = get_env_key("DOUBAO");
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
        },
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn doubao_pro(app: tauri::AppHandle, flow_args: FlowArgs) -> Result<String, String> {
    let api_key = get_env_key("DOUBAO");
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
        },
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn doubao_seed_flash(
    app: tauri::AppHandle,
    flow_args: FlowArgs,
) -> Result<String, String> {
    let api_key = get_env_key("DOUBAO");
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
        },
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn doubao_seed(app: tauri::AppHandle, flow_args: FlowArgs) -> Result<String, String> {
    let api_key = get_env_key("DOUBAO");
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
        },
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]

pub async fn kimi(app: tauri::AppHandle, flow_args: FlowArgs) -> Result<String, String> {
    let api_key = get_env_key("KIMI");
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
        },
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]

pub async fn zhipu(app: tauri::AppHandle, flow_args: FlowArgs) -> Result<String, String> {
    let api_key = get_env_key("ZHIPU");
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
        },
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]

pub async fn deepseek_api(app: tauri::AppHandle, flow_args: FlowArgs) -> Result<String, String> {
    let api_key = get_env_key("DEEPSEEK");
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
        },
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]

pub async fn ali_qwen_2_5(app: tauri::AppHandle, flow_args: FlowArgs) -> Result<String, String> {
    let api_key = get_env_key("ALI_QWEN_QWQ");
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
        },
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn ali_qwen_plus_latest(
    app: tauri::AppHandle,
    flow_args: FlowArgs,
) -> Result<String, String> {
    let api_key = get_env_key("ALI_QWEN_QWQ");
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
        },
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn ali_qwen_max(app: tauri::AppHandle, flow_args: FlowArgs) -> Result<String, String> {
    let api_key = get_env_key("ALI_QWEN_QWQ");
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
        },
    )
    .await
    .map_err(|e| e.to_string())
}
