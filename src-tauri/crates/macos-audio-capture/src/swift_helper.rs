use crate::{CaptureController, CaptureSession};
use hex::encode as hex_encode;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

pub const BACKEND_NAME: &str = "swift-helper";

const AUDIO_DUMP_SWIFT_SOURCE: &str = include_str!("../../../examples/audio_dump.swift");

pub fn spawn() -> Result<CaptureSession, String> {
    let helper_binary = ensure_audio_dump_binary()?;
    let mut child = Command::new(helper_binary)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| format!("Failed to start macOS audio helper process: {err}"))?;
    let stdout = Box::new(
        child
            .stdout
            .take()
            .ok_or_else(|| "macOS audio helper stdout unavailable".to_string())?,
    ) as Box<dyn Read + Send>;
    let stderr = Box::new(
        child
            .stderr
            .take()
            .ok_or_else(|| "macOS audio helper stderr unavailable".to_string())?,
    ) as Box<dyn Read + Send>;

    Ok(CaptureSession::new(
        stdout,
        stderr,
        Box::new(SwiftHelperController { child: Some(child) }),
    ))
}

struct SwiftHelperController {
    child: Option<Child>,
}

impl CaptureController for SwiftHelperController {
    fn stop(&mut self) {
        if let Some(child) = self.child.as_mut() {
            let _ = child.kill();
        }
    }

    fn wait(&mut self) -> Result<(), String> {
        let Some(mut child) = self.child.take() else {
            return Ok(());
        };
        let status = child
            .wait()
            .map_err(|err| format!("Failed waiting for macOS audio helper: {err}"))?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("macOS audio helper exited with status {status}"))
        }
    }
}

fn ensure_audio_dump_binary() -> Result<PathBuf, String> {
    let mut hasher = Sha256::new();
    hasher.update(AUDIO_DUMP_SWIFT_SOURCE.as_bytes());
    let digest = hex_encode(hasher.finalize());
    let helper_dir = std::env::temp_dir().join("audio-courier-screen-capture");
    let source_path = helper_dir.join(format!("audio_dump_{}.swift", &digest[..12]));
    let binary_path = helper_dir.join(format!("audio_dump_{}", &digest[..12]));

    if binary_path.exists() {
        return Ok(binary_path);
    }

    fs::create_dir_all(&helper_dir)
        .map_err(|err| format!("Failed to create macOS helper directory: {err}"))?;
    fs::write(&source_path, AUDIO_DUMP_SWIFT_SOURCE)
        .map_err(|err| format!("Failed to write macOS helper source: {err}"))?;

    let output = Command::new("xcrun")
        .args([
            "swiftc",
            "-parse-as-library",
            source_path
                .to_str()
                .ok_or_else(|| "Failed to encode macOS helper source path".to_string())?,
            "-o",
            binary_path
                .to_str()
                .ok_or_else(|| "Failed to encode macOS helper binary path".to_string())?,
            "-framework",
            "ScreenCaptureKit",
            "-framework",
            "AVFoundation",
            "-framework",
            "CoreMedia",
        ])
        .output()
        .map_err(|err| format!("Failed to compile macOS audio helper with xcrun: {err}"))?;

    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(format!(
            "Failed compiling macOS audio helper (status {}): {}",
            output.status,
            if stderr.is_empty() && stdout.is_empty() {
                "unknown swiftc error".to_string()
            } else if stderr.is_empty() {
                format!("stdout: {stdout}")
            } else if stdout.is_empty() {
                stderr
            } else {
                format!("stderr: {stderr}; stdout: {stdout}")
            }
        ));
    }

    if !binary_path.exists() {
        return Err(format!(
            "macOS audio helper compile reported success but binary was not created at {}",
            binary_path.display()
        ));
    }

    Ok(binary_path)
}
