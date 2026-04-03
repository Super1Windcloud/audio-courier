#![cfg(target_os = "macos")]

use std::io::Read;

#[cfg(all(feature = "swift-helper", feature = "rust-native"))]
compile_error!("Enable only one of `swift-helper` or `rust-native` for macos-audio-capture.");

#[cfg(not(any(feature = "swift-helper", feature = "rust-native")))]
compile_error!("Enable one of `swift-helper` or `rust-native` for macos-audio-capture.");

#[cfg(feature = "rust-native")]
mod rust_native;
#[cfg(feature = "swift-helper")]
mod swift_helper;

#[cfg(feature = "rust-native")]
use rust_native as selected_backend;
#[cfg(feature = "swift-helper")]
use swift_helper as selected_backend;

pub struct CaptureSession {
    stdout: Option<Box<dyn Read + Send>>,
    stderr: Option<Box<dyn Read + Send>>,
    controller: Box<dyn CaptureController>,
}

trait CaptureController: Send {
    fn stop(&mut self);
    fn wait(&mut self) -> Result<(), String>;
}

impl CaptureSession {
    fn new(
        stdout: Box<dyn Read + Send>,
        stderr: Box<dyn Read + Send>,
        controller: Box<dyn CaptureController>,
    ) -> Self {
        Self {
            stdout: Some(stdout),
            stderr: Some(stderr),
            controller,
        }
    }

    pub fn take_stdout(&mut self) -> Result<Box<dyn Read + Send>, String> {
        self.stdout
            .take()
            .ok_or_else(|| "macOS audio capture stdout unavailable".to_string())
    }

    pub fn take_stderr(&mut self) -> Result<Box<dyn Read + Send>, String> {
        self.stderr
            .take()
            .ok_or_else(|| "macOS audio capture stderr unavailable".to_string())
    }

    pub fn stop(&mut self) {
        self.controller.stop();
    }

    pub fn wait(&mut self) -> Result<(), String> {
        self.controller.wait()
    }
}

pub fn selected_backend_name() -> &'static str {
    selected_backend::BACKEND_NAME
}

pub fn spawn_system_audio_capture() -> Result<CaptureSession, String> {
    selected_backend::spawn()
}
