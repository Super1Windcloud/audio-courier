use std::str::FromStr;
use std::sync::Arc;

pub type PcmCallback = Arc<dyn Fn(&str) + Send + Sync + 'static>;
pub type StatusCallback = Arc<dyn Fn(String) + Send + Sync + 'static>;

pub trait StreamingTranscriber: Send + Sync {
    fn queue_chunk(&self, chunk: Vec<i16>) -> Result<(), String>;
    fn get_vendor_name(&self) -> String; 
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranscriptVendors {
    DeepGram,
    RevAI,
    AssemblyAI,
    GlaDia,
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

pub mod assemblyai;
pub mod deepgram;
pub mod gladia;
pub mod revai;
pub mod speechmatics;
