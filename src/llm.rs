use json::JsonValue;
use std::{str::FromStr, string::ToString};

type Handle = u32;
type ExitCode = u8;

#[link(wasm_import_module = "blockless_llm")]
extern "C" {
    fn llm_set_model_request(h: *mut Handle, model_ptr: *const u8, model_len: u8) -> ExitCode;
    fn llm_get_model_response(
        h: Handle,
        buf: *mut u8,
        buf_len: u8,
        bytes_written: *mut u8,
    ) -> ExitCode;
    fn llm_set_model_options_request(
        h: Handle,
        options_ptr: *const u8,
        options_len: u16,
    ) -> ExitCode;
    fn llm_get_model_options(
        h: Handle,
        buf: *mut u8,
        buf_len: u16,
        bytes_written: *mut u16,
    ) -> ExitCode;
    fn llm_prompt_request(h: Handle, prompt_ptr: *const u8, prompt_len: u16) -> ExitCode;
    fn llm_read_prompt_response(
        h: Handle,
        buf: *mut u8,
        buf_len: u16,
        bytes_written: *mut u16,
    ) -> ExitCode;
    fn llm_close(h: Handle) -> ExitCode;
}

#[derive(Debug, Clone)]
pub enum SupportedModels {
    Llama321BInstruct(Option<String>),
    Llama323BInstruct(Option<String>),
    Mistral7BInstructV03(Option<String>),
    Mixtral8x7BInstructV01(Option<String>),
    Gemma22BInstruct(Option<String>),
    Gemma27BInstruct(Option<String>),
    Gemma29BInstruct(Option<String>),
}

impl FromStr for SupportedModels {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            // Llama 3.2 1B
            "Llama-3.2-1B-Instruct" => Ok(SupportedModels::Llama321BInstruct(None)),
            "Llama-3.2-1B-Instruct-Q6_K"
            | "Llama-3.2-1B-Instruct_Q6_K"
            | "Llama-3.2-1B-Instruct.Q6_K" => {
                Ok(SupportedModels::Llama321BInstruct(Some("Q6_K".to_string())))
            }
            "Llama-3.2-1B-Instruct-q4f16_1" | "Llama-3.2-1B-Instruct.q4f16_1" => Ok(
                SupportedModels::Llama321BInstruct(Some("q4f16_1".to_string())),
            ),

            // Llama 3.2 3B
            "Llama-3.2-3B-Instruct" => Ok(SupportedModels::Llama323BInstruct(None)),
            "Llama-3.2-3B-Instruct-Q6_K"
            | "Llama-3.2-3B-Instruct_Q6_K"
            | "Llama-3.2-3B-Instruct.Q6_K" => {
                Ok(SupportedModels::Llama323BInstruct(Some("Q6_K".to_string())))
            }
            "Llama-3.2-3B-Instruct-q4f16_1" | "Llama-3.2-3B-Instruct.q4f16_1" => Ok(
                SupportedModels::Llama323BInstruct(Some("q4f16_1".to_string())),
            ),

            // Mistral 7B
            "Mistral-7B-Instruct-v0.3" => Ok(SupportedModels::Mistral7BInstructV03(None)),
            "Mistral-7B-Instruct-v0.3-q4f16_1" | "Mistral-7B-Instruct-v0.3.q4f16_1" => Ok(
                SupportedModels::Mistral7BInstructV03(Some("q4f16_1".to_string())),
            ),

            // Mixtral 8x7B
            "Mixtral-8x7B-Instruct-v0.1" => Ok(SupportedModels::Mixtral8x7BInstructV01(None)),
            "Mixtral-8x7B-Instruct-v0.1-q4f16_1" | "Mixtral-8x7B-Instruct-v0.1.q4f16_1" => Ok(
                SupportedModels::Mixtral8x7BInstructV01(Some("q4f16_1".to_string())),
            ),

            // Gemma models
            "gemma-2-2b-it" => Ok(SupportedModels::Gemma22BInstruct(None)),
            "gemma-2-2b-it-q4f16_1" | "gemma-2-2b-it.q4f16_1" => Ok(
                SupportedModels::Gemma22BInstruct(Some("q4f16_1".to_string())),
            ),

            "gemma-2-27b-it" => Ok(SupportedModels::Gemma27BInstruct(None)),
            "gemma-2-27b-it-q4f16_1" | "gemma-2-27b-it.q4f16_1" => Ok(
                SupportedModels::Gemma27BInstruct(Some("q4f16_1".to_string())),
            ),

            "gemma-2-9b-it" => Ok(SupportedModels::Gemma29BInstruct(None)),
            "gemma-2-9b-it-q4f16_1" | "gemma-2-9b-it.q4f16_1" => Ok(
                SupportedModels::Gemma29BInstruct(Some("q4f16_1".to_string())),
            ),

            _ => Err(format!("Unsupported model: {}", s)),
        }
    }
}

impl ToString for SupportedModels {
    fn to_string(&self) -> String {
        match self {
            SupportedModels::Llama321BInstruct(_) => "Llama-3.2-1B-Instruct".to_string(),
            SupportedModels::Llama323BInstruct(_) => "Llama-3.2-3B-Instruct".to_string(),
            SupportedModels::Mistral7BInstructV03(_) => "Mistral-7B-Instruct-v0.3".to_string(),
            SupportedModels::Mixtral8x7BInstructV01(_) => "Mixtral-8x7B-Instruct-v0.1".to_string(),
            SupportedModels::Gemma22BInstruct(_) => "gemma-2-2b-it".to_string(),
            SupportedModels::Gemma27BInstruct(_) => "gemma-2-27b-it".to_string(),
            SupportedModels::Gemma29BInstruct(_) => "gemma-2-9b-it".to_string(),
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Default)]
pub struct BlocklessLlm {
    inner: Handle,
    model_name: String,
    options: LlmOptions,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct LlmOptions {
    pub system_message: String,
    // pub max_tokens: u32,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    // pub frequency_penalty: f32,
    // pub presence_penalty: f32,
}

impl LlmOptions {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn dump(&self) -> Vec<u8> {
        let mut json = JsonValue::new_object();
        json["system_message"] = self.system_message.clone().into();
        if let Some(temperature) = self.temperature {
            json["temperature"] = temperature.into();
        }
        if let Some(top_p) = self.top_p {
            json["top_p"] = top_p.into();
        }
        json.dump().into_bytes()
    }
}

impl std::fmt::Display for LlmOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bytes = self.dump();
        match String::from_utf8(bytes) {
            Ok(s) => write!(f, "{}", s),
            Err(_) => write!(f, "<invalid utf8>"),
        }
    }
}

impl TryFrom<Vec<u8>> for LlmOptions {
    type Error = LlmErrorKind;

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        // Convert bytes to UTF-8 string
        let json_str = String::from_utf8(bytes).map_err(|_| LlmErrorKind::Utf8Error)?;

        // Parse the JSON string
        let json = json::parse(&json_str).map_err(|_| LlmErrorKind::ModelOptionsNotSet)?;

        // Extract system_message
        let system_message = json["system_message"]
            .as_str()
            .ok_or(LlmErrorKind::ModelOptionsNotSet)?
            .to_string();

        Ok(LlmOptions {
            system_message,
            temperature: json["temperature"].as_f32(),
            top_p: json["top_p"].as_f32(),
        })
    }
}

impl BlocklessLlm {
    pub fn new(model: SupportedModels) -> Result<Self, LlmErrorKind> {
        let model_name = model.to_string();
        let mut llm: BlocklessLlm = Default::default();
        llm.set_model(&model_name)?;
        Ok(llm)
    }

    pub fn handle(&self) -> Handle {
        self.inner
    }

    pub fn get_model(&self) -> Result<String, LlmErrorKind> {
        let mut buf = [0u8; u8::MAX as usize];
        let mut num_bytes: u8 = 0;
        let code = unsafe {
            llm_get_model_response(self.inner, buf.as_mut_ptr(), buf.len() as _, &mut num_bytes)
        };
        if code != 0 {
            return Err(code.into());
        }
        let model = String::from_utf8(buf[0..num_bytes as _].to_vec()).unwrap();
        Ok(model)
    }

    pub fn set_model(&mut self, model_name: &str) -> Result<(), LlmErrorKind> {
        self.model_name = model_name.to_string();
        let code = unsafe {
            llm_set_model_request(&mut self.inner, model_name.as_ptr(), model_name.len() as _)
        };
        if code != 0 {
            return Err(code.into());
        }

        // validate model is set correctly in host/runtime
        let host_model = self.get_model()?;
        if self.model_name != host_model {
            eprintln!(
                "Model not set correctly in host/runtime; model_name: {}, model_from_host: {}",
                self.model_name, host_model
            );
            return Err(LlmErrorKind::ModelNotSet);
        }
        Ok(())
    }

    pub fn get_options(&self) -> Result<LlmOptions, LlmErrorKind> {
        let mut buf = [0u8; u16::MAX as usize];
        let mut num_bytes: u16 = 0;
        let code = unsafe {
            llm_get_model_options(self.inner, buf.as_mut_ptr(), buf.len() as _, &mut num_bytes)
        };
        if code != 0 {
            return Err(code.into());
        }

        // Convert buffer slice to Vec<u8> and try to parse into LlmOptions
        LlmOptions::try_from(buf[0..num_bytes as usize].to_vec())
    }

    pub fn set_options(&mut self, options: LlmOptions) -> Result<(), LlmErrorKind> {
        let options_json = options.dump();
        self.options = options;
        let code = unsafe {
            llm_set_model_options_request(
                self.inner,
                options_json.as_ptr(),
                options_json.len() as _,
            )
        };
        if code != 0 {
            return Err(code.into());
        }

        // Verify options were set correctly
        let host_options = self.get_options()?;
        if self.options != host_options {
            println!(
                "Options not set correctly in host/runtime; options: {:?}, options_from_host: {:?}",
                self.options, host_options
            );
            return Err(LlmErrorKind::ModelOptionsNotSet);
        }
        Ok(())
    }

    pub fn chat_request(&self, prompt: &str) -> Result<String, LlmErrorKind> {
        // Perform the prompt request
        let code = unsafe { llm_prompt_request(self.inner, prompt.as_ptr(), prompt.len() as _) };
        if code != 0 {
            return Err(code.into());
        }

        // Read the response
        self.get_chat_response()
    }

    fn get_chat_response(&self) -> Result<String, LlmErrorKind> {
        let mut buf = [0u8; u16::MAX as usize];
        let mut num_bytes: u16 = 0;
        let code = unsafe {
            llm_read_prompt_response(self.inner, buf.as_mut_ptr(), buf.len() as _, &mut num_bytes)
        };
        if code != 0 {
            return Err(code.into());
        }

        let response_vec = buf[0..num_bytes as usize].to_vec();
        String::from_utf8(response_vec).map_err(|_| LlmErrorKind::Utf8Error)
    }
}

impl Drop for BlocklessLlm {
    fn drop(&mut self) {
        let code = unsafe { llm_close(self.inner) };
        if code != 0 {
            eprintln!("Error closing LLM: {}", code);
        }
    }
}

#[derive(Debug)]
pub enum LlmErrorKind {
    ModelNotSet,               // 1
    ModelNotSupported,         // 2
    ModelInitializationFailed, // 3
    ModelCompletionFailed,     // 4
    ModelOptionsNotSet,        // 5
    ModelShutdownFailed,       // 6
    Utf8Error,                 // 7
    RuntimeError,              // 8
}

impl From<u8> for LlmErrorKind {
    fn from(code: u8) -> Self {
        match code {
            1 => LlmErrorKind::ModelNotSet,
            2 => LlmErrorKind::ModelNotSupported,
            3 => LlmErrorKind::ModelInitializationFailed,
            4 => LlmErrorKind::ModelCompletionFailed,
            5 => LlmErrorKind::ModelOptionsNotSet,
            6 => LlmErrorKind::ModelShutdownFailed,
            7 => LlmErrorKind::Utf8Error,
            _ => LlmErrorKind::RuntimeError,
        }
    }
}
