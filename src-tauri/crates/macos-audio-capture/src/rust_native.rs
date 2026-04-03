use crate::{CaptureController, CaptureSession};
use block2::RcBlock;
use dispatch2::{DispatchQueue, DispatchQueueAttr};
use objc2::{
    AllocAnyThread, DefinedClass, define_class, msg_send, rc::Retained, runtime::ProtocolObject,
};
use objc2_core_audio_types::{
    AudioBuffer, AudioBufferList, AudioStreamBasicDescription, kAudioFormatFlagIsFloat,
    kAudioFormatFlagIsNonInterleaved, kAudioFormatFlagIsSignedInteger, kAudioFormatLinearPCM,
};
use objc2_core_graphics::{CGPreflightScreenCaptureAccess, CGRequestScreenCaptureAccess};
use objc2_core_media::{
    CMAudioFormatDescription, CMAudioFormatDescriptionGetStreamBasicDescription, CMBlockBuffer,
    CMSampleBuffer, kCMSampleBufferFlag_AudioBufferList_Assure16ByteAlignment,
};
use objc2_foundation::{NSArray, NSError, NSObject, NSObjectProtocol};
use objc2_screen_capture_kit::{
    SCContentFilter, SCDisplay, SCRunningApplication, SCShareableContent, SCStream,
    SCStreamConfiguration, SCStreamDelegate, SCStreamOutput, SCStreamOutputType, SCWindow,
};
use std::io::{Read, Write};
use std::mem::size_of;
use std::os::unix::net::UnixStream;
use std::panic::{self, AssertUnwindSafe};
use std::slice;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
    mpsc,
};
use std::thread::{self, JoinHandle};
use std::time::Duration;

pub const BACKEND_NAME: &str = "rust-native";

const TARGET_SAMPLE_RATE: isize = 16_000;
const TARGET_CHANNELS: isize = 1;
const STARTUP_TIMEOUT: Duration = Duration::from_secs(10);

pub fn spawn() -> Result<CaptureSession, String> {
    let (stdout_read, stdout_write) =
        UnixStream::pair().map_err(|err| format!("Failed creating macOS audio pipe: {err}"))?;
    let (stderr_read, stderr_write) =
        UnixStream::pair().map_err(|err| format!("Failed creating macOS log pipe: {err}"))?;

    let stop = Arc::new(AtomicBool::new(false));
    let (ready_tx, ready_rx) = mpsc::channel();
    let stop_for_thread = stop.clone();

    let handle = thread::Builder::new()
        .name("macos-audio-capture-rust-native".to_string())
        .spawn(move || {
            match panic::catch_unwind(AssertUnwindSafe(|| {
                run_capture_thread(
                    stdout_write,
                    stderr_write,
                    stop_for_thread,
                    ready_tx.clone(),
                )
            })) {
                Ok(result) => result,
                Err(payload) => {
                    let panic_message = if let Some(message) = payload.downcast_ref::<&str>() {
                        (*message).to_string()
                    } else if let Some(message) = payload.downcast_ref::<String>() {
                        message.clone()
                    } else {
                        "unknown panic payload".to_string()
                    };
                    let error = format!("macOS native audio capture panicked: {panic_message}");
                    let _ = ready_tx.send(Err(error.clone()));
                    Err(error)
                }
            }
        })
        .map_err(|err| format!("Failed spawning macOS native audio capture thread: {err}"))?;

    match ready_rx.recv_timeout(STARTUP_TIMEOUT) {
        Ok(Ok(())) => Ok(CaptureSession::new(
            Box::new(stdout_read) as Box<dyn Read + Send>,
            Box::new(stderr_read) as Box<dyn Read + Send>,
            Box::new(RustNativeController {
                stop,
                handle: Some(handle),
            }),
        )),
        Ok(Err(err)) => {
            let _ = handle.join();
            Err(err)
        }
        Err(mpsc::RecvTimeoutError::Timeout) => {
            stop.store(true, Ordering::SeqCst);
            let _ = handle.join();
            Err(format!(
                "macOS native audio capture did not become ready within {}s",
                STARTUP_TIMEOUT.as_secs()
            ))
        }
        Err(err) => {
            let _ = handle.join();
            Err(format!(
                "macOS native audio capture thread exited before startup completed: {err}"
            ))
        }
    }
}

struct RustNativeController {
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<Result<(), String>>>,
}

impl CaptureController for RustNativeController {
    fn stop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
    }

    fn wait(&mut self) -> Result<(), String> {
        let Some(handle) = self.handle.take() else {
            return Ok(());
        };
        handle
            .join()
            .map_err(|_| "macOS native audio capture thread panicked".to_string())?
    }
}

#[derive(Debug)]
struct StreamOutputState {
    stdout: Mutex<UnixStream>,
    stderr: Mutex<UnixStream>,
    stop: Arc<AtomicBool>,
    error: Mutex<Option<String>>,
}

impl StreamOutputState {
    fn write_stderr(&self, message: &str) {
        let mut stderr = self.stderr.lock().unwrap();
        let _ = stderr.write_all(message.as_bytes());
        let _ = stderr.write_all(b"\n");
        let _ = stderr.flush();
    }

    fn set_error(&self, message: String) {
        let mut error = self.error.lock().unwrap();
        if error.is_none() {
            self.write_stderr(&message);
            *error = Some(message);
        }
        self.stop.store(true, Ordering::SeqCst);
    }

    fn set_error_with_context(&self, context: &str, err: impl std::fmt::Display) {
        self.set_error(format!("{context}: {err}"));
    }

    fn current_error(&self) -> Option<String> {
        self.error.lock().unwrap().clone()
    }

    fn handle_sample_buffer(&self, sample_buffer: &CMSampleBuffer) {
        match convert_sample_buffer(sample_buffer) {
            Ok(bytes) => {
                if bytes.is_empty() {
                    return;
                }

                let mut stdout = self.stdout.lock().unwrap();
                if let Err(err) = stdout.write_all(&bytes) {
                    self.set_error_with_context("Failed writing macOS native audio samples", err);
                }
            }
            Err(err) => self.set_error(err),
        }
    }
}

define_class!(
    #[unsafe(super(NSObject))]
    #[name = "AudioCourierStreamOutput"]
    #[ivars = Arc<StreamOutputState>]
    #[derive(Debug)]
    struct AudioCourierStreamOutput;

    #[allow(non_snake_case)]
    unsafe impl SCStreamOutput for AudioCourierStreamOutput {
        #[unsafe(method(stream:didOutputSampleBuffer:ofType:))]
        unsafe fn stream_didOutputSampleBuffer_ofType(
            &self,
            _stream: &SCStream,
            sample_buffer: &CMSampleBuffer,
            output_type: SCStreamOutputType,
        ) {
            if output_type == SCStreamOutputType::Audio {
                self.ivars().handle_sample_buffer(sample_buffer);
            }
        }
    }

    #[allow(non_snake_case)]
    unsafe impl SCStreamDelegate for AudioCourierStreamOutput {
        #[unsafe(method(stream:didStopWithError:))]
        unsafe fn stream_didStopWithError(&self, _stream: &SCStream, error: &NSError) {
            self.ivars()
                .set_error(format!("macOS native capture stopped: {error}"));
        }
    }
);

unsafe impl NSObjectProtocol for AudioCourierStreamOutput {}

impl AudioCourierStreamOutput {
    fn new(state: Arc<StreamOutputState>) -> Retained<Self> {
        let this = Self::alloc().set_ivars(state);
        unsafe { msg_send![super(this), init] }
    }
}

fn run_capture_thread(
    stdout: UnixStream,
    stderr: UnixStream,
    stop: Arc<AtomicBool>,
    ready_tx: mpsc::Sender<Result<(), String>>,
) -> Result<(), String> {
    if !CGPreflightScreenCaptureAccess() && !CGRequestScreenCaptureAccess() {
        let err = "Screen recording permission required for macOS system audio capture".to_string();
        let _ = ready_tx.send(Err(err.clone()));
        return Err(err);
    }

    let state = Arc::new(StreamOutputState {
        stdout: Mutex::new(stdout),
        stderr: Mutex::new(stderr),
        stop: stop.clone(),
        error: Mutex::new(None),
    });

    unsafe {
        let content = match get_shareable_content() {
            Ok(content) => content,
            Err(err) => {
                state.write_stderr(&err);
                let _ = ready_tx.send(Err(err.clone()));
                return Err(err);
            }
        };
        let display = match first_display(&content) {
            Ok(display) => display,
            Err(err) => {
                state.write_stderr(&err);
                let _ = ready_tx.send(Err(err.clone()));
                return Err(err);
            }
        };

        let excluded_applications = empty_array::<SCRunningApplication>();
        let excepting_windows = empty_array::<SCWindow>();
        let filter = SCContentFilter::initWithDisplay_excludingApplications_exceptingWindows(
            SCContentFilter::alloc(),
            &display,
            &excluded_applications,
            &excepting_windows,
        );
        let configuration = SCStreamConfiguration::new();
        configuration.setCapturesAudio(true);
        configuration.setCaptureMicrophone(false);
        configuration.setExcludesCurrentProcessAudio(true);
        configuration.setSampleRate(TARGET_SAMPLE_RATE);
        configuration.setChannelCount(TARGET_CHANNELS);

        let delegate = AudioCourierStreamOutput::new(state.clone());
        let delegate_protocol: &ProtocolObject<dyn SCStreamDelegate> =
            ProtocolObject::from_ref(&*delegate);
        let output_protocol: &ProtocolObject<dyn SCStreamOutput> =
            ProtocolObject::from_ref(&*delegate);
        let stream = SCStream::initWithFilter_configuration_delegate(
            SCStream::alloc(),
            &filter,
            &configuration,
            Some(delegate_protocol),
        );
        let callback_queue =
            DispatchQueue::new("com.audio-courier.system-audio", DispatchQueueAttr::SERIAL);

        stream
            .addStreamOutput_type_sampleHandlerQueue_error(
                output_protocol,
                SCStreamOutputType::Audio,
                Some(&callback_queue),
            )
            .map_err(|err| {
                let message = format!("Failed adding macOS native audio output: {err}");
                state.write_stderr(&message);
                let _ = ready_tx.send(Err(message.clone()));
                message
            })?;

        if let Err(err) = start_capture(&stream) {
            state.write_stderr(&err);
            let _ = ready_tx.send(Err(err.clone()));
            return Err(err);
        }
        let _ = ready_tx.send(Ok(()));
        state.write_stderr("macOS native audio capture started");

        while !stop.load(Ordering::SeqCst) && state.current_error().is_none() {
            thread::sleep(Duration::from_millis(100));
        }

        let stop_result = stop_capture(&stream);
        drop(callback_queue);
        drop(delegate);

        if let Some(err) = state.current_error() {
            match stop_result {
                Ok(()) => Err(err),
                Err(stop_err) => Err(format!("{err}; stop error: {stop_err}")),
            }
        } else {
            stop_result
        }
    }
}

unsafe fn get_shareable_content() -> Result<Retained<SCShareableContent>, String> {
    let (tx, rx) = mpsc::channel();
    let block = RcBlock::new(
        move |content: *mut SCShareableContent, error: *mut NSError| {
            let result = if !error.is_null() {
                Err(format!(
                    "Failed retrieving macOS shareable content: {}",
                    unsafe { &*error }
                ))
            } else {
                unsafe { Retained::retain(content) }
                    .ok_or_else(|| "macOS shareable content callback returned null".to_string())
            };
            let _ = tx.send(result);
        },
    );

    unsafe {
        SCShareableContent::getShareableContentExcludingDesktopWindows_onScreenWindowsOnly_completionHandler(
            false,
            true,
            &block,
        );
    }

    rx.recv()
        .map_err(|err| format!("Shareable content callback channel failed: {err}"))?
}

unsafe fn first_display(content: &SCShareableContent) -> Result<Retained<SCDisplay>, String> {
    unsafe { content.displays() }
        .firstObject()
        .ok_or_else(|| "No shareable macOS display found for system audio capture".to_string())
}

unsafe fn empty_array<T: objc2::Message>() -> Retained<NSArray<T>> {
    NSArray::new()
}

unsafe fn start_capture(stream: &SCStream) -> Result<(), String> {
    let (tx, rx) = mpsc::channel();
    let block = RcBlock::new(move |error: *mut NSError| {
        let result = if error.is_null() {
            Ok(())
        } else {
            Err(format!(
                "Failed starting macOS native audio capture: {}",
                unsafe { &*error }
            ))
        };
        let _ = tx.send(result);
    });

    unsafe {
        stream.startCaptureWithCompletionHandler(Some(&block));
    }
    rx.recv()
        .map_err(|err| format!("Start capture callback channel failed: {err}"))?
}

unsafe fn stop_capture(stream: &SCStream) -> Result<(), String> {
    let (tx, rx) = mpsc::channel();
    let block = RcBlock::new(move |error: *mut NSError| {
        let result = if error.is_null() {
            Ok(())
        } else {
            Err(format!(
                "Failed stopping macOS native audio capture: {}",
                unsafe { &*error }
            ))
        };
        let _ = tx.send(result);
    });

    unsafe {
        stream.stopCaptureWithCompletionHandler(Some(&block));
    }
    rx.recv()
        .map_err(|err| format!("Stop capture callback channel failed: {err}"))?
}

fn convert_sample_buffer(sample_buffer: &CMSampleBuffer) -> Result<Vec<u8>, String> {
    let format_description = unsafe {
        sample_buffer
            .format_description()
            .ok_or_else(|| "macOS audio sample buffer missing format description".to_string())?
    };
    let stream_description_ptr = unsafe {
        CMAudioFormatDescriptionGetStreamBasicDescription(
            &format_description as &CMAudioFormatDescription,
        )
    };
    if stream_description_ptr.is_null() {
        return Err("macOS audio sample buffer missing stream description".to_string());
    }

    let stream_description = unsafe { &*stream_description_ptr };
    let frames = unsafe { sample_buffer.num_samples() as usize };
    if frames == 0 {
        return Ok(Vec::new());
    }

    let audio_buffers = sample_audio_buffers(sample_buffer)?;
    pcm_to_i16_mono_bytes(stream_description, frames, &audio_buffers).map_err(|err| {
        format!(
            "{err}; {}",
            sample_buffer_context(sample_buffer, stream_description, audio_buffers.len())
        )
    })
}

fn sample_buffer_context(
    sample_buffer: &CMSampleBuffer,
    description: &AudioStreamBasicDescription,
    buffer_count: usize,
) -> String {
    let frames = unsafe { sample_buffer.num_samples() };
    format!(
        "format_id={}, sample_rate={}, channels={}, bits_per_channel={}, flags={}, frames={}, buffers={}",
        description.mFormatID,
        description.mSampleRate,
        description.mChannelsPerFrame,
        description.mBitsPerChannel,
        description.mFormatFlags,
        frames,
        buffer_count
    )
}

fn sample_audio_buffers(sample_buffer: &CMSampleBuffer) -> Result<Vec<Vec<u8>>, String> {
    let mut needed_size = 0usize;
    let status = unsafe {
        sample_buffer.audio_buffer_list_with_retained_block_buffer(
            &mut needed_size,
            std::ptr::null_mut(),
            0,
            None,
            None,
            kCMSampleBufferFlag_AudioBufferList_Assure16ByteAlignment,
            std::ptr::null_mut(),
        )
    };

    if status != 0 || needed_size < size_of::<AudioBufferList>() {
        return Err(format!(
            "Failed querying macOS audio buffer list size (status {status}, size {needed_size})"
        ));
    }

    let mut storage = vec![0u8; needed_size];
    let audio_buffer_list = storage.as_mut_ptr().cast::<AudioBufferList>();
    let mut block_buffer: *mut CMBlockBuffer = std::ptr::null_mut();
    let status = unsafe {
        sample_buffer.audio_buffer_list_with_retained_block_buffer(
            std::ptr::null_mut(),
            audio_buffer_list,
            needed_size,
            None,
            None,
            kCMSampleBufferFlag_AudioBufferList_Assure16ByteAlignment,
            &mut block_buffer,
        )
    };
    let _block_buffer = unsafe { Retained::from_raw(block_buffer) };

    if status != 0 {
        return Err(format!(
            "Failed retrieving macOS audio buffer list (status {status}, size {needed_size})"
        ));
    }

    let audio_buffer_list = unsafe { &*audio_buffer_list };
    let buffer_count = audio_buffer_list.mNumberBuffers as usize;
    if buffer_count == 0 {
        return Err("macOS audio sample buffer returned zero audio buffers".to_string());
    }
    let buffers = unsafe {
        slice::from_raw_parts(
            audio_buffer_list.mBuffers.as_ptr() as *const AudioBuffer,
            buffer_count,
        )
    };

    let mut result = Vec::with_capacity(buffer_count);
    for (index, buffer) in buffers.iter().enumerate() {
        let data = if buffer.mData.is_null() || buffer.mDataByteSize == 0 {
            Vec::new()
        } else {
            unsafe {
                slice::from_raw_parts(buffer.mData.cast::<u8>(), buffer.mDataByteSize as usize)
                    .to_vec()
            }
        };
        if data.is_empty() {
            return Err(format!(
                "macOS audio buffer #{index} was empty (byte_size={})",
                buffer.mDataByteSize
            ));
        }
        result.push(data);
    }

    Ok(result)
}

fn pcm_to_i16_mono_bytes(
    description: &AudioStreamBasicDescription,
    frames: usize,
    buffers: &[Vec<u8>],
) -> Result<Vec<u8>, String> {
    if description.mFormatID != kAudioFormatLinearPCM {
        return Err(format!(
            "Unsupported macOS system audio format id: {}",
            description.mFormatID
        ));
    }

    let channels = description.mChannelsPerFrame.max(1) as usize;
    let is_float = description.mFormatFlags & kAudioFormatFlagIsFloat != 0;
    let is_signed_integer = description.mFormatFlags & kAudioFormatFlagIsSignedInteger != 0;
    let is_non_interleaved = description.mFormatFlags & kAudioFormatFlagIsNonInterleaved != 0;

    let samples = if is_float && description.mBitsPerChannel == 32 {
        if is_non_interleaved {
            read_f32_non_interleaved(frames, channels, buffers)?
        } else {
            read_f32_interleaved(frames, channels, buffers)?
        }
    } else if is_signed_integer && description.mBitsPerChannel == 16 {
        if is_non_interleaved {
            read_i16_non_interleaved(frames, channels, buffers)?
        } else {
            read_i16_interleaved(frames, channels, buffers)?
        }
    } else {
        return Err(format!(
            "Unsupported macOS PCM layout: bits={}, flags={}",
            description.mBitsPerChannel, description.mFormatFlags
        ));
    };

    let mut bytes = Vec::with_capacity(samples.len() * 2);
    for sample in samples {
        bytes.extend_from_slice(&sample.to_le_bytes());
    }
    Ok(bytes)
}

fn read_f32_non_interleaved(
    frames: usize,
    channels: usize,
    buffers: &[Vec<u8>],
) -> Result<Vec<i16>, String> {
    if buffers.len() < channels {
        return Err(format!(
            "Expected {channels} non-interleaved macOS audio buffers, got {}",
            buffers.len()
        ));
    }

    let channel_slices = buffers
        .iter()
        .take(channels)
        .map(|buffer| bytes_as_f32(buffer, frames))
        .collect::<Result<Vec<_>, _>>()?;

    let mut output = Vec::with_capacity(frames);
    for frame_index in 0..frames {
        let mut mixed = 0.0f32;
        for channel in &channel_slices {
            mixed += channel[frame_index];
        }
        output.push(float_to_i16(mixed / channels as f32));
    }
    Ok(output)
}

fn read_f32_interleaved(
    frames: usize,
    channels: usize,
    buffers: &[Vec<u8>],
) -> Result<Vec<i16>, String> {
    let Some(buffer) = buffers.first() else {
        return Ok(Vec::new());
    };
    let samples = bytes_as_f32(buffer, frames * channels)?;
    let mut output = Vec::with_capacity(frames);

    for frame in samples.chunks_exact(channels) {
        let mixed = frame.iter().copied().sum::<f32>() / channels as f32;
        output.push(float_to_i16(mixed));
    }

    Ok(output)
}

fn read_i16_non_interleaved(
    frames: usize,
    channels: usize,
    buffers: &[Vec<u8>],
) -> Result<Vec<i16>, String> {
    if buffers.len() < channels {
        return Err(format!(
            "Expected {channels} non-interleaved macOS audio buffers, got {}",
            buffers.len()
        ));
    }

    let channel_slices = buffers
        .iter()
        .take(channels)
        .map(|buffer| bytes_as_i16(buffer, frames))
        .collect::<Result<Vec<_>, _>>()?;

    let mut output = Vec::with_capacity(frames);
    for frame_index in 0..frames {
        let mut mixed = 0i32;
        for channel in &channel_slices {
            mixed += i32::from(channel[frame_index]);
        }
        output.push((mixed / channels as i32) as i16);
    }
    Ok(output)
}

fn read_i16_interleaved(
    frames: usize,
    channels: usize,
    buffers: &[Vec<u8>],
) -> Result<Vec<i16>, String> {
    let Some(buffer) = buffers.first() else {
        return Ok(Vec::new());
    };
    let samples = bytes_as_i16(buffer, frames * channels)?;
    let mut output = Vec::with_capacity(frames);

    for frame in samples.chunks_exact(channels) {
        let mixed = frame.iter().map(|sample| i32::from(*sample)).sum::<i32>() / channels as i32;
        output.push(mixed as i16);
    }

    Ok(output)
}

fn bytes_as_f32(buffer: &[u8], expected_samples: usize) -> Result<Vec<f32>, String> {
    let expected_bytes = expected_samples * size_of::<f32>();
    if buffer.len() < expected_bytes {
        return Err(format!(
            "macOS audio buffer shorter than expected: need {expected_bytes} bytes, got {}",
            buffer.len()
        ));
    }

    Ok(buffer[..expected_bytes]
        .chunks_exact(4)
        .map(|chunk| f32::from_ne_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect())
}

fn bytes_as_i16(buffer: &[u8], expected_samples: usize) -> Result<Vec<i16>, String> {
    let expected_bytes = expected_samples * size_of::<i16>();
    if buffer.len() < expected_bytes {
        return Err(format!(
            "macOS audio buffer shorter than expected: need {expected_bytes} bytes, got {}",
            buffer.len()
        ));
    }

    Ok(buffer[..expected_bytes]
        .chunks_exact(2)
        .map(|chunk| i16::from_ne_bytes([chunk[0], chunk[1]]))
        .collect())
}

fn float_to_i16(sample: f32) -> i16 {
    (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16
}
