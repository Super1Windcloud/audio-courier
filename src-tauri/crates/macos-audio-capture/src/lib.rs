#![cfg(target_os = "macos")]

use std::io::Read;

#[cfg(not(any(feature = "swift-helper", feature = "rust-native")))]
compile_error!("Enable one of `swift-helper` or `rust-native` for macos-audio-capture.");

#[cfg(feature = "rust-native")]
mod rust_native;
#[cfg(feature = "swift-helper")]
mod swift_helper;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureBackend {
    SwiftHelper,
    RustNative,
}

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

pub fn selected_backend_name(backend: CaptureBackend) -> &'static str {
    match backend {
        CaptureBackend::SwiftHelper => {
            #[cfg(feature = "swift-helper")]
            {
                swift_helper::BACKEND_NAME
            }
            #[cfg(not(feature = "swift-helper"))]
            {
                "swift-helper"
            }
        }
        CaptureBackend::RustNative => {
            #[cfg(feature = "rust-native")]
            {
                rust_native::BACKEND_NAME
            }
            #[cfg(not(feature = "rust-native"))]
            {
                "rust-native"
            }
        }
    }
}

pub fn spawn_system_audio_capture(backend: CaptureBackend) -> Result<CaptureSession, String> {
    match backend {
        CaptureBackend::SwiftHelper => {
            #[cfg(feature = "swift-helper")]
            {
                swift_helper::spawn()
            }
            #[cfg(not(feature = "swift-helper"))]
            {
                Err("macOS audio capture backend `swift-helper` is not compiled in".to_string())
            }
        }
        CaptureBackend::RustNative => {
            #[cfg(feature = "rust-native")]
            {
                rust_native::spawn()
            }
            #[cfg(not(feature = "rust-native"))]
            {
                Err("macOS audio capture backend `rust-native` is not compiled in".to_string())
            }
        }
    }
}
