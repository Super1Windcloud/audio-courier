use serde::Serialize;
use std::str::FromStr;
use std::sync::Arc;
pub mod assemblyai;
pub mod deepgram_api;
pub mod deepgram_sdk;
pub mod gladia;
pub mod revai;
pub mod speechmatics;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TranscriptEventKind {
    Draft,
    Commit,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptEvent {
    pub vendor: String,
    pub kind: TranscriptEventKind,
    pub text: String,
}

pub type PcmCallback = Arc<dyn Fn(TranscriptEvent) + Send + Sync + 'static>;
pub type StatusCallback = Arc<dyn Fn(String) + Send + Sync + 'static>;

pub fn emit_draft(callback: &PcmCallback, vendor: &str, text: impl Into<String>) {
    emit_transcript_event(callback, vendor, TranscriptEventKind::Draft, text);
}

pub fn emit_commit(callback: &PcmCallback, vendor: &str, text: impl Into<String>) {
    emit_transcript_event(callback, vendor, TranscriptEventKind::Commit, text);
}

fn emit_transcript_event(
    callback: &PcmCallback,
    vendor: &str,
    kind: TranscriptEventKind,
    text: impl Into<String>,
) {
    let text = text.into();
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return;
    }

    callback(TranscriptEvent {
        vendor: vendor.to_string(),
        kind,
        text: trimmed.to_string(),
    });
}

pub trait StreamingTranscriber: Send + Sync {
    fn queue_chunk(&self, chunk: Vec<i16>) -> Result<(), String>;
    fn get_vendor_name(&self) -> String;
    fn force_endpoint(&self) -> Result<(), String> {
        Ok(())
    }
    #[allow(unused)]
    fn shutdown(&self);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranscriptVendors {
    DeepGram,
    RevAI,      //Normal
    AssemblyAI, //Normal
    GlaDia,     // No punctuation
    SpeechMatics,
}

impl FromStr for TranscriptVendors {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "deepgram" => Ok(TranscriptVendors::DeepGram),
            "revai" => Ok(TranscriptVendors::RevAI),
            "assemblyai" => Ok(TranscriptVendors::AssemblyAI),
            "gladia" => Ok(TranscriptVendors::GlaDia),
            "speechmatics" => Ok(TranscriptVendors::SpeechMatics),
            _ => Err(format!("Unknown vendor: {}", s)),
        }
    }
}
