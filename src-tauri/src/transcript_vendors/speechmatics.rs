use crate::transcript_vendors::{PcmCallback, StatusCallback, StreamingTranscriber};
use speechmatics::realtime::models::ConversationConfig;
use speechmatics::realtime::{ReadMessage, RealtimeSession, SessionConfig, models};
use std::env;
use std::io;
use std::pin::Pin;
use std::sync::Mutex;
use std::task::{Context, Poll};
use std::thread::{self, JoinHandle};
use tokio::io::{AsyncRead, ReadBuf};
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

pub struct SpeechmaticsTranscriber {
    sender: Mutex<Option<mpsc::Sender<Vec<u8>>>>,
    handle: Mutex<Option<JoinHandle<()>>>,
}

impl SpeechmaticsTranscriber {
    pub fn start(
        sample_rate: u32,
        callback: PcmCallback,
        status_callback: Option<StatusCallback>,
    ) -> Result<Self, String> {
        let api_key = env::var("SPEECHMATICS_API_KEY")
            .map_err(|e| format!("Missing SPEECHMATICS_API_KEY environment variable: {e}"))?;
        let language = env::var("SPEECHMATICS_LANGUAGE")
            .ok()
            .filter(|value| !value.is_empty());
        let url = Some("wss://eu2.rt.speechmatics.com/v2/".to_string());

        let (sender, receiver) = mpsc::channel::<Vec<u8>>(64);
        let callback_clone = callback.clone();
        let status_callback_clone = status_callback.clone();

        let handle = thread::spawn(move || {
            let runtime = Runtime::new().expect("Failed to build Tokio runtime");
            if let Err(err) = runtime.block_on(run_session(
                api_key,
                url,
                language,
                sample_rate,
                callback_clone,
                receiver,
            )) {
                if let Some(cb) = status_callback_clone.as_ref() {
                    cb(format!("speechmatics: {err}"));
                }
                eprintln!("Speechmatics streaming error: {err}");
            }
        });

        Ok(Self {
            sender: Mutex::new(Some(sender)),
            handle: Mutex::new(Some(handle)),
        })
    }

    pub fn enqueue_chunk(&self, chunk: Vec<i16>) -> Result<(), String> {
        let sender = self
            .sender
            .lock()
            .unwrap()
            .as_ref()
            .cloned()
            .ok_or_else(|| "Speechmatics transcriber is not running".to_string())?;

        let mut bytes = Vec::with_capacity(chunk.len() * 2);
        for sample in chunk {
            bytes.extend_from_slice(&sample.to_le_bytes());
        }

        sender
            .blocking_send(bytes)
            .map_err(|e| format!("Failed to queue PCM chunk for Speechmatics: {e}"))
    }

    pub fn stop(&self) {
        self.sender.lock().unwrap().take();

        if let Some(handle) = self.handle.lock().unwrap().take() {
            let _ = handle.join();
        }
    }
}

impl Drop for SpeechmaticsTranscriber {
    fn drop(&mut self) {
        self.stop();
    }
}

impl StreamingTranscriber for SpeechmaticsTranscriber {
    fn queue_chunk(&self, chunk: Vec<i16>) -> Result<(), String> {
        self.enqueue_chunk(chunk)
    }

    fn get_vendor_name(&self) -> String {
        "SpeechMatics".to_string()
    }

    fn shutdown(&self) {
        self.stop();
        println!("Speechmatics websocket shutdown invoked");
    }
}

async fn run_session(
    api_key: String,
    rt_url: Option<String>,
    language: Option<String>,
    sample_rate: u32,
    callback: PcmCallback,
    audio_rx: mpsc::Receiver<Vec<u8>>,
) -> Result<(), String> {
    let (mut session, mut message_rx) = RealtimeSession::new(api_key, rt_url)
        .map_err(|e| format!("Failed to init Speechmatics session: {e}"))?;

    let mut config = SessionConfig::default();
    if let Some(lang) = language {
        config.transcription_config.language = lang;
    }
    config.translation_config = None;
    let mut audio_format = models::AudioFormat::new(models::audio_format::Type::Raw);
    audio_format.encoding = Some(models::audio_format::Encoding::PcmS16le);
    audio_format.sample_rate = Some(sample_rate as i32);
    config.audio_format = Some(audio_format);
    config.transcription_config.conversation_config = Some(Box::new(ConversationConfig {
        end_of_utterance_silence_trigger: Some(0.5),
    }));
    config.transcription_config.max_delay = Some(2.0);
    config.transcription_config.enable_partials = Some(false);

    let reader = ChannelAudioReader::new(audio_rx);

    let message_task = tokio::spawn(async move {
        while let Some(message) = message_rx.recv().await {
            match message {
                ReadMessage::AddTranscript(transcript) => {
                    let text = transcript.metadata.transcript.trim().to_string();
                    if !text.is_empty() {
                        callback(&text);
                    }
                }
                ReadMessage::Error(err) => {
                    eprintln!("Speechmatics returned error: {}", err.reason);
                    break;
                }
                ReadMessage::EndOfTranscript(_) => break,
                _ => {}
            }
        }
    });

    if let Err(err) = session.run(config, reader).await {
        let _ = message_task.await;
        return Err(format!("Speechmatics session run failed: {err}"));
    }

    let _ = message_task.await;
    println!("Speechmatics websocket closed");
    Ok(())
}

struct ChannelAudioReader {
    receiver: mpsc::Receiver<Vec<u8>>,
    buffer: Vec<u8>,
    position: usize,
    finished: bool,
}

impl ChannelAudioReader {
    fn new(receiver: mpsc::Receiver<Vec<u8>>) -> Self {
        Self {
            receiver,
            buffer: Vec::new(),
            position: 0,
            finished: false,
        }
    }
}

impl AsyncRead for ChannelAudioReader {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        if self.finished {
            return Poll::Ready(Ok(()));
        }

        if self.position >= self.buffer.len() {
            match Pin::new(&mut self.receiver).poll_recv(cx) {
                Poll::Ready(Some(chunk)) => {
                    self.buffer = chunk;
                    self.position = 0;
                }
                Poll::Ready(None) => {
                    self.finished = true;
                    return Poll::Ready(Ok(()));
                }
                Poll::Pending => return Poll::Pending,
            }
        }

        if self.position >= self.buffer.len() {
            return Poll::Ready(Ok(()));
        }

        let available = self.buffer.len() - self.position;
        let to_copy = available.min(buf.remaining());
        if to_copy == 0 {
            return Poll::Ready(Ok(()));
        }

        buf.put_slice(&self.buffer[self.position..self.position + to_copy]);
        self.position += to_copy;
        Poll::Ready(Ok(()))
    }
}
