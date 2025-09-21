mod llm;
use llm::*;

use serde_json::json;

#[tauri::command]
pub async fn siliconflow(
    app: tauri::AppHandle,
    question: Option<String>,
) -> Result<String, String> {
    let api_key = get_env_key("Siliconflow");
    let question = question.unwrap_or_else(|| "牛顿迭代".to_string());
    let messages = vec![
        json!({"role":"assistant","content":"你是一个代码助手,帮我解决算法问题"}),
        json!({"role":"user","content":question}),
    ];

    call_model_api(
        app,
        ModelRequest {
            model: "Qwen/Qwen2.5-Coder-32B-Instruct".to_string(),
            messages,
            base_url: "https://api.siliconflow.cn/v1".to_string(),
            api_key,
            max_tokens: 4096,
            temperature: 0.7,
        },
    )
    .await
}

#[tauri::command]
pub async fn doubao_lite(
    app: tauri::AppHandle,
    question: Option<String>,
) -> Result<String, String> {
    let api_key = get_env_key("DOUBAO");
    let question = question.unwrap_or_else(|| "牛顿迭代".to_string());
    let messages = vec![
        json!({"role":"system","content":"你是一个代码助手 帮我解决算法问题"}),
        json!({"role":"user","content":question}),
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
}

#[tauri::command]
pub async fn doubao_pro(app: tauri::AppHandle, question: Option<String>) -> Result<String, String> {
    let api_key = get_env_key("DOUBAO");
    let question = question.unwrap_or_else(|| "牛顿迭代".to_string());
    let messages = vec![
        json!({"role":"assistant","content":"你是一个代码助手 帮我解决算法问题"}),
        json!({"role":"user","content":question}),
    ];

    call_model_api(
        app,
        ModelRequest {
            model: "doubao-1.5-pro-32k-250115".to_string(),
            messages,
            base_url: "https://ark.cn-beijing.volces.com/api/v3/".to_string(),
            api_key,
            max_tokens: 4096,
            temperature: 0.7,
        },
    )
    .await
}

#[tauri::command]
pub async fn doubao_deepseek(
    app: tauri::AppHandle,
    question: Option<String>,
) -> Result<String, String> {
    let api_key = get_env_key("DOUBAO");
    let question = question.unwrap_or_else(|| "牛顿迭代".to_string());
    let messages = vec![
        json!({"role":"system","content":"你是一个代码助手,帮我解答算法代码题"}),
        json!({"role":"user","content":question}),
    ];

    call_model_api(
        app,
        ModelRequest {
            model: "ep-20250224214614-qvpgg".to_string(),
            messages,
            base_url: "https://ark.cn-beijing.volces.com/api/v3/".to_string(),
            api_key,
            max_tokens: 4096,
            temperature: 0.7,
        },
    )
    .await
}

#[tauri::command]

pub async fn kimi(app: tauri::AppHandle, question: Option<String>) -> Result<String, String> {
    let api_key = get_env_key("KIMI");
    let question = question.unwrap_or_else(|| "牛顿迭代".to_string());
    let messages = vec![
        json!({"role":"system","content":"你是一个代码助手,帮我解答算法代码题"}),
        json!({"role":"user","content":question}),
    ];

    call_model_api(
        app,
        ModelRequest {
            model: "moonshot-v1-auto".to_string(),
            messages,
            base_url: "https://api.moonshot.cn/v1".to_string(),
            api_key,
            max_tokens: 4096,
            temperature: 0.7,
        },
    )
    .await
}

#[tauri::command]

pub async fn zhipu(app: tauri::AppHandle, question: Option<String>) -> Result<String, String> {
    let api_key = get_env_key("ZHIPU");
    let question = question.unwrap_or_else(|| "牛顿迭代".to_string());
    let messages = vec![
        json!({"role":"system","content":"你是一个代码助手,帮我解答算法代码题 "}),
        json!({"role":"user","content":question}),
    ];

    call_model_api(
        app,
        ModelRequest {
            model: "codegeex-4".to_string(),
            messages,
            base_url: "https://open.bigmodel.cn/api/paas/v4/".to_string(),
            api_key,
            max_tokens: 2000,
            temperature: 0.9,
        },
    )
    .await
}

#[tauri::command]

pub async fn deepseek_api(
    app: tauri::AppHandle,
    question: Option<String>,
) -> Result<String, String> {
    let api_key = get_env_key("DEEPSEEK");
    let question = question.unwrap_or_else(|| "牛顿迭代".to_string());
    let messages = vec![
        json!({"role":"system","content":"You are a helpful Code assistant."}),
        json!({"role":"user","content":question}),
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
}

#[tauri::command]

pub async fn ali_qwen_32b(
    app: tauri::AppHandle,
    question: Option<String>,
) -> Result<String, String> {
    let api_key = get_env_key("ALI_QWEN_QWQ");
    let question = question.unwrap_or_else(|| "牛顿迭代".to_string());
    let messages = vec![
        json!({"role":"system","content":"你是一个代码助手,帮我解答算法代码题"}),
        json!({"role":"user","content":question}),
    ];

    call_model_api(
        app,
        ModelRequest {
            model: "qwq-32b".to_string(),
            messages,
            base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1".to_string(),
            api_key,
            max_tokens: 4096,
            temperature: 0.7,
        },
    )
    .await
}

#[tauri::command]

pub async fn ali_qwen_2_5(
    app: tauri::AppHandle,
    question: Option<String>,
) -> Result<String, String> {
    let api_key = get_env_key("ALI_QWEN_QWQ");
    let question = question.unwrap_or_else(|| "牛顿迭代".to_string());
    let messages = vec![
        json!({"role":"system","content":"你是一个代码助手,帮我解答算法代码题"}),
        json!({"role":"user","content":question}),
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
}

#[tauri::command]

pub async fn ali_qwen_plus(
    app: tauri::AppHandle,
    question: Option<String>,
) -> Result<String, String> {
    let api_key = get_env_key("ALI_QWEN_QWQ");
    let question = question.unwrap_or_else(|| "牛顿迭代".to_string());
    let messages = vec![
        json!({"role":"system","content":"你是一个代码助手,帮我解答算法代码题"}),
        json!({"role":"user","content":question}),
    ];

    call_model_api(
        app,
        ModelRequest {
            model: "qwq-plus".to_string(),
            messages,
            base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1".to_string(),
            api_key,
            max_tokens: 4096,
            temperature: 0.7,
        },
    )
    .await
}

#[tauri::command]
pub async fn ali_qwen_max(
    app: tauri::AppHandle,
    question: Option<String>,
) -> Result<String, String> {
    let api_key = get_env_key("ALI_QWEN_QWQ");
    let question = question.unwrap_or_else(|| "牛顿迭代".to_string());
    let messages = vec![
        json!({"role":"system","content":"你是一个代码助手,帮我解答算法代码题"}),
        json!({"role":"user","content":question}),
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
}
