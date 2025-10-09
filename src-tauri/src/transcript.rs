use vosk::CompleteResult;

#[derive(Debug)]
pub enum TranscriptionError {
    ModelNotLoaded,
    LoadModelFailed,
    ParseError(String),
}

pub struct TranscriptionManager {
    model_path: String,
    model: Option<vosk::Model>,
    recognizer: Option<vosk::Recognizer>,
    model_loaded: bool,
    buffer: Vec<i16>, // 新增
}

impl TranscriptionManager {
    pub fn new_vosk(model_path: String) -> Self {
        Self {
            model_path,
            model: None,
            recognizer: None,
            model_loaded: false,
            buffer: Vec::new(),
        }
    }

    pub fn initialize(&mut self) -> Result<(), TranscriptionError> {
        println!("正在初始化语音识别模型: {}", self.model_path);

        let model =
            vosk::Model::new(&self.model_path).ok_or(TranscriptionError::LoadModelFailed)?;

        let recognizer =
            vosk::Recognizer::new(&model, 16000.0).ok_or(TranscriptionError::LoadModelFailed)?;

        self.model = Some(model);
        self.recognizer = Some(recognizer);
        self.model_loaded = true;

        println!("语音识别模型初始化成功");
        Ok(())
    }

    pub fn process_audio_stream(
        &mut self,
        audio_data: &[f32],
    ) -> Result<String, TranscriptionError> {
        if !self.model_loaded {
            return Err(TranscriptionError::ModelNotLoaded);
        }

        if let Some(ref mut recognizer) = self.recognizer {
            // f32 -> i16
            let audio_i16: Vec<i16> = audio_data
                .iter()
                .map(|&x| (x.clamp(-1.0, 1.0) * 32767.0) as i16)
                .collect();

            // 加入缓冲区
            self.buffer.extend(audio_i16);

            // 只有当 ≥1600 帧时再送 Vosk
            if self.buffer.len() >= 1600 {
                let chunk: Vec<i16> = self.buffer.drain(..1600).collect();
                match recognizer.accept_waveform(&chunk) {
                    Ok(state) => {
                        if state == vosk::DecodingState::Finalized {
                            let res = recognizer.result();
                            return Ok(Self::extract_text_from_result(res)?);
                        } else {
                            let partial = recognizer.partial_result();
                            if !partial.partial.trim().is_empty() {
                                return Ok(partial.partial.to_string());
                            }
                        }
                    }
                    Err(e) => {
                        return Err(TranscriptionError::ParseError(format!(
                            "音频处理失败: {:?}",
                            e
                        )));
                    }
                }
            }
        }

        Ok(String::new())
    }

    fn extract_text_from_result(result: CompleteResult) -> Result<String, TranscriptionError> {
        match result {
            CompleteResult::Single(single) => {
                println!("单一结果: {:?}", single);
                Ok(single.text.to_string()) // 转换 &str 为 String
            }
            CompleteResult::Multiple(multi) => {
                println!("多重结果: {:?}", multi);
                Ok(multi
                    .alternatives
                    .first()
                    .map(|alt| alt.text.to_string()) // 转换 &str 为 String
                    .unwrap_or_default())
            }
        }
    }

    // 获取最终结果
    pub fn finalize(&mut self) -> Result<String, TranscriptionError> {
        if let Some(ref mut recognizer) = self.recognizer {
            let final_result = recognizer.final_result();
            match final_result {
                CompleteResult::Single(single) => {
                    println!("最终单一结果: '{}'", single.text);
                    Ok(single.text.to_string()) // 转换为 String
                }
                CompleteResult::Multiple(multi) => {
                    let text = multi
                        .alternatives
                        .first()
                        .map(|alt| alt.text.to_string()) // 转换为 String
                        .unwrap_or_default();
                    println!("最终多重结果: '{}'", text);
                    Ok(text)
                }
            }
        } else {
            Ok(String::new())
        }
    }

    pub fn is_initialized(&self) -> bool {
        self.model_loaded
    }

    // 重置识别器状态
    pub fn reset(&mut self) -> Result<(), TranscriptionError> {
        if let Some(ref model) = self.model {
            self.recognizer = vosk::Recognizer::new(model, 16000.0)
                .map(Some)
                .ok_or(TranscriptionError::LoadModelFailed)?;
            println!("识别器已重置");
        }
        Ok(())
    }
}
