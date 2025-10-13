#[allow(dead_code)]
#[derive(Debug)]
pub enum TranscriptionError {
    ModelNotLoaded,
    LoadModelFailed,
    ParseError(String),
}
#[allow(dead_code)]
pub struct TranscriptionManager {
    model_path: String,
    model: Option<vosk::Model>,
    recognizer: Option<vosk::Recognizer>,
    model_loaded: bool,
    buffer: Vec<i16>,
}

#[allow(dead_code)]
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
}
