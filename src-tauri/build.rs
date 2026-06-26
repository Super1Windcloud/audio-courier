use std::fs;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let env_path = manifest_dir.join(".env");

    println!("cargo:rerun-if-changed={}", env_path.display());

    if let Ok(content) = fs::read_to_string(env_path) {
        if let Some(api_key) = read_env_value(&content, "DEEPGRAM_API_KEY") {
            println!("cargo:rustc-env=BUILTIN_DEEPGRAM_API_KEY={api_key}");
        }
    }

    tauri_build::build();
}

fn read_env_value(content: &str, target_key: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let Some((key, value)) = trimmed.split_once('=') else {
            continue;
        };

        if key.trim() != target_key {
            continue;
        }

        return normalize_env_value(value);
    }

    None
}

fn normalize_env_value(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    let unquoted = if trimmed.len() >= 2 {
        let bytes = trimmed.as_bytes();
        if (bytes[0] == b'"' && bytes[trimmed.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[trimmed.len() - 1] == b'\'')
        {
            &trimmed[1..trimmed.len() - 1]
        } else {
            trimmed
        }
    } else {
        trimmed
    };

    Some(unquoted.to_string())
}
