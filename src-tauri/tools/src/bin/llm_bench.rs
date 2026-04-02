use std::{
    error::Error,
    future::Future,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use tauri::{AppHandle, Listener, Wry};
use tauri_courier_ai_lib::{
    FlowArgs, ali_qwen_2_5, ali_qwen_max, ali_qwen_plus_latest, deepseek_api, doubao_lite,
    doubao_pro, doubao_seed, doubao_seed_flash, siliconflow_free_models,
    siliconflow_free_with_model, siliconflow_pro_models, siliconflow_pro_with_model,
};
use tokio::{sync::oneshot, time::timeout};

const RUNS_PER_MODEL: usize = 10;
const BENCH_PROMPT: &str = "你是一个用于延迟测试的智能助手,请直接给出简洁的中文回答,不需要反问。";
const BENCH_QUESTION: &str = "请用2句话介绍本次模型的能力。";

fn sanitize_fragment(input: &str) -> String {
    input
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-') {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn build_request_id(label: &str, iteration: usize) -> String {
    format!("bench_{}_{}", sanitize_fragment(label), iteration)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::from_filename("../.env").ok();
    dotenv::dotenv().ok();
    println!("LLM 启动耗时基准测试, 每个模型运行 {RUNS_PER_MODEL} 次");

    let app = tauri::Builder::default()
        .build(tauri::generate_context!())
        .expect("无法创建 tauri App 用于基准测试");
    let app_handle = app.handle();

    bench_models(app_handle).await;
    Ok(())
}

#[allow(dead_code)]
async fn bench_base(app_handle: &AppHandle<Wry>) {
    macro_rules! bench {
        ($label:literal, $func:path, $app:expr) => {
            if let Err(err) = run_bench_for($label, $app, $func, None).await {
                eprintln!("模型 {label} 基准测试终止: {err}", label = $label);
            }
        };
    }

    bench!("doubao_lite", doubao_lite, app_handle);
    bench!("doubao_pro", doubao_pro, app_handle);
    bench!("doubao_seed_flash", doubao_seed_flash, app_handle);
    bench!("doubao_seed", doubao_seed, app_handle);
    bench!("deepseek_api", deepseek_api, app_handle);
    bench!("ali_qwen_2_5", ali_qwen_2_5, app_handle);
    bench!("ali_qwen_plus_latest", ali_qwen_plus_latest, app_handle);
    bench!("ali_qwen_max", ali_qwen_max, app_handle);

    for &model in siliconflow_free_models() {
        let label = format!("siliconflow_free::{model}");
        if let Err(err) = run_bench_for(
            label.as_str(),
            app_handle,
            move |app, args| siliconflow_free_with_model(app, args, model),
            None,
        )
        .await
        {
            eprintln!("模型 {label} 基准测试终止: {err}");
        }
    }
}

async fn bench_models(app_handle: &AppHandle<Wry>) {
    bench_base(app_handle).await;

    for &model in siliconflow_pro_models() {
        let label = format!("siliconflow_pro::{model}");
        if let Err(err) = run_bench_for(
            label.as_str(),
            app_handle,
            move |app, args| siliconflow_pro_with_model(app, args, model),
            Some(Duration::from_secs(3)),
        )
        .await
        {
            eprintln!("模型 {label} 基准测试终止: {err}");
        }
    }
}

async fn run_bench_for<F, Fut>(
    label: &str,
    app_handle: &AppHandle<Wry>,
    func: F,
    abort_timeout: Option<Duration>,
) -> Result<(), String>
where
    F: Fn(AppHandle<Wry>, FlowArgs) -> Fut + Copy + Send + Sync + 'static,
    Fut: Future<Output = Result<String, String>> + Send + 'static,
{
    println!("\n=== {label} ===");
    let mut durations = Vec::with_capacity(RUNS_PER_MODEL);
    let mut failures = 0usize;

    for iteration in 0..RUNS_PER_MODEL {
        let single_run = execute_single_run(label, iteration, app_handle, func);
        let result = if let Some(timeout_duration) = abort_timeout {
            match timeout(timeout_duration, single_run).await {
                Ok(res) => res,
                Err(_) => {
                    println!(
                        "第 {run:02} 次超过 {secs}s 限制，跳过 {label}",
                        run = iteration + 1,
                        secs = timeout_duration.as_secs()
                    );
                    return Ok(());
                }
            }
        } else {
            single_run.await
        };

        match result {
            Ok(duration) => {
                println!(
                    "第 {run:02} 次: 首字节耗时 {secs:.2}s",
                    run = iteration + 1,
                    secs = duration.as_secs_f64()
                );
                durations.push(duration);
            }
            Err(err) => {
                failures += 1;
                eprintln!("第 {run:02} 次失败: {err}", run = iteration + 1);
            }
        }
    }

    if durations.is_empty() {
        return Err(format!("{label} 没有成功的请求, 共 {failures} 次失败"));
    }

    let avg = durations.iter().map(Duration::as_secs_f64).sum::<f64>() / durations.len() as f64;
    let fastest = durations.iter().min().cloned().unwrap();
    let slowest = durations.iter().max().cloned().unwrap();

    println!(
        "总结 {label}: 平均 {avg:.2}s | 最快 {fast:.2}s | 最慢 {slow:.2}s | 成功 {success} | 失败 {failures}",
        avg = avg,
        fast = fastest.as_secs_f64(),
        slow = slowest.as_secs_f64(),
        success = durations.len(),
        failures = failures
    );

    Ok(())
}

async fn execute_single_run<F, Fut>(
    label: &str,
    iteration: usize,
    app_handle: &AppHandle<Wry>,
    func: F,
) -> Result<Duration, String>
where
    F: Fn(AppHandle<Wry>, FlowArgs) -> Fut + Copy + Send + Sync + 'static,
    Fut: Future<Output = Result<String, String>> + Send + 'static,
{
    let request_id = build_request_id(label, iteration);
    let args = FlowArgs::new(BENCH_QUESTION, BENCH_PROMPT).set_request_id(Some(request_id.clone()));
    let start = Instant::now();
    let event_name = format!("llm_stream_{request_id}");
    let (tx, rx) = oneshot::channel();
    let sender = Arc::new(Mutex::new(Some(tx)));
    let listener_sender = Arc::clone(&sender);

    let event_id = app_handle.listen(event_name, move |event| {
        if let Some(payload) = Some(event.payload()) {
            if payload.trim().is_empty() {
                return;
            }

            if let Some(sender) = listener_sender.lock().unwrap().take() {
                let _ = sender.send(start.elapsed());
            }
        }
    });

    let result = func(app_handle.clone(), args).await;
    app_handle.unlisten(event_id);
    drop(sender);

    match result {
        Ok(_) => match rx.await {
            Ok(duration) => Ok(duration),
            Err(_) => Err("未捕获到流式文本响应".to_string()),
        },
        Err(err) => Err(err),
    }
}
