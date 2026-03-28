mod api;
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

const FREE_MODELS: [&str; 8] = [
    "Qwen/Qwen2.5-Coder-32B-Instruct", // 0.15S
    "Qwen/Qwen2.5-7B-Instruct",        //0.22S
    "Qwen/Qwen2-7B-Instruct",          // 0.1S
    "tencent/Hunyuan-MT-7B",           //0.17S
    "THUDM/GLM-Z1-9B-0414",            // 0.22S
    "THUDM/GLM-4-9B-0414",             //0.5S
    "internlm/internlm2_5-7b-chat",    //0.11S
    "THUDM/glm-4-9b-chat",             //0.35S
];

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

const PRO_MODELS: [&str; 12] = [
    "Pro/Qwen/Qwen2.5-7B-Instruct",     //0.17S
    "Pro/THUDM/glm-4-9b-chat",          //0.27S
    "Qwen/Qwen2.5-14B-Instruct",        // 0.21S
    "Qwen/Qwen2.5-Coder-32B-Instruct",  //0.14S
    "Qwen/Qwen2.5-32B-Instruct",        // 0.23S
    "THUDM/GLM-4-32B-0414",             //0.29S
    "Qwen/Qwen3-Next-80B-A3B-Instruct", //0.36S
    "inclusionAI/Ling-flash-2.0",       // 0.4S
    "Qwen/Qwen2.5-72B-Instruct-128K",   //0.53S
    "zai-org/GLM-4.5-Air",              //0.41S
    "deepseek-ai/DeepSeek-V3",          //0.68S
    "baidu/ERNIE-4.5-300B-A47B",        // 0.16S
];

pub fn siliconflow_pro_models() -> &'static [&'static str] {
    &PRO_MODELS
}

pub async fn siliconflow_pro_with_model(
    app: tauri::AppHandle,
    flow_args: FlowArgs,
    model: &str,
) -> Result<String, String> {
    let api_key = get_env_key("SiliconflowVLM");
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
        flow_args.request_id,
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
        flow_args.request_id,
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
        flow_args.request_id,
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
        flow_args.request_id,
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
        flow_args.request_id,
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
        flow_args.request_id,
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
        flow_args.request_id,
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
        flow_args.request_id,
    )
    .await
    .map_err(|e| e.to_string())
}
