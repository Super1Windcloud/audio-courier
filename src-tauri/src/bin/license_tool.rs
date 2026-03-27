use base64::{Engine as _, engine::general_purpose::STANDARD};
use chrono::{DateTime, Utc};
use ed25519_dalek::SigningKey;
use std::env;
use std::path::PathBuf;
use tauri_courier_ai_lib::license::{read_activation_request, sign_license, write_signed_license};

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    let Some(command) = args.get(1).map(String::as_str) else {
        return Err(help_text());
    };

    match command {
        "generate-keypair" => generate_keypair(),
        "sign" => sign_command(&args[2..]),
        _ => Err(help_text()),
    }
}

fn generate_keypair() -> Result<(), String> {
    let signing_key = SigningKey::from_bytes(&rand::random::<[u8; 32]>());
    let verify_key = signing_key.verifying_key();

    println!(
        "LICENSE_PRIVATE_KEY={}",
        STANDARD.encode(signing_key.to_bytes())
    );
    println!(
        "LICENSE_PUBLIC_KEY={}",
        STANDARD.encode(verify_key.to_bytes())
    );
    Ok(())
}

fn sign_command(args: &[String]) -> Result<(), String> {
    let mut request_path: Option<PathBuf> = None;
    let mut output_path: Option<PathBuf> = None;
    let mut user_id: Option<String> = None;
    let mut expires_at: Option<DateTime<Utc>> = None;
    let mut max_version: Option<String> = None;
    let mut features: Vec<String> = Vec::new();
    let mut private_key: Option<String> = env::var("LICENSE_PRIVATE_KEY").ok();

    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--request" => {
                index += 1;
                request_path = args.get(index).map(PathBuf::from);
            }
            "--output" => {
                index += 1;
                output_path = args.get(index).map(PathBuf::from);
            }
            "--user-id" => {
                index += 1;
                user_id = args.get(index).cloned();
            }
            "--expires-at" => {
                index += 1;
                expires_at = args
                    .get(index)
                    .ok_or_else(help_text)?
                    .parse::<DateTime<Utc>>()
                    .map(Some)
                    .map_err(|err| format!("expires-at 解析失败: {}", err))?;
            }
            "--max-version" => {
                index += 1;
                max_version = args.get(index).cloned();
            }
            "--feature" => {
                index += 1;
                if let Some(feature) = args.get(index) {
                    features.push(feature.clone());
                }
            }
            "--private-key" => {
                index += 1;
                private_key = args.get(index).cloned();
            }
            flag => return Err(format!("未知参数: {flag}\n\n{}", help_text())),
        }
        index += 1;
    }

    let request_path = request_path.ok_or_else(help_text)?;
    let user_id = user_id.ok_or_else(help_text)?;
    let expires_at = expires_at.ok_or_else(help_text)?;
    let max_version = max_version.ok_or_else(help_text)?;
    let private_key = private_key.ok_or_else(|| {
        "缺少私钥。请传 --private-key 或设置 LICENSE_PRIVATE_KEY 环境变量".to_string()
    })?;

    let request = read_activation_request(&request_path)?;
    let license = sign_license(
        request,
        user_id,
        expires_at,
        max_version,
        if features.is_empty() {
            vec!["pro".to_string()]
        } else {
            features
        },
        &private_key,
    )?;

    if let Some(output_path) = output_path {
        write_signed_license(&output_path, &license)?;
        println!("许可证已写入 {}", output_path.display());
    } else {
        println!(
            "{}",
            serde_json::to_string_pretty(&license).map_err(|err| err.to_string())?
        );
    }

    Ok(())
}

fn help_text() -> String {
    [
        "用法:",
        "  cargo run --manifest-path src-tauri/Cargo.toml --bin license_tool -- generate-keypair",
        "  cargo run --manifest-path src-tauri/Cargo.toml --bin license_tool -- sign --request activation_request.json --user-id customer_001 --expires-at 2027-03-27T23:59:59Z --max-version 1.9.99 --feature pro --output license.json",
    ]
    .join("\n")
}
