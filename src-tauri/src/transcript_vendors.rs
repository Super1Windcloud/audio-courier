use std::str::FromStr;
use std::sync::Arc;
pub mod assemblyai;
pub mod gladia;
pub mod revai;
pub mod speechmatics;
#[cfg(feature = "api")]
pub mod deepgram_api;
#[cfg(feature = "sdk")]
pub mod deepgram_sdk;


pub type PcmCallback = Arc<dyn Fn(&str, bool) + Send + Sync + 'static>;
pub type StatusCallback = Arc<dyn Fn(String) + Send + Sync + 'static>;

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

