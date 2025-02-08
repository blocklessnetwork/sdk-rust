use json::JsonValue;

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

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Default)]
pub struct BlocklessLlm {
    inner: Handle,
    model_name: String,
    options: LlmOptions,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct LlmOptions {
    pub system_message: String,
    // pub max_tokens: u32,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    // pub frequency_penalty: f32,
    // pub presence_penalty: f32,
}

impl Default for LlmOptions {
    fn default() -> Self {
        LlmOptions {
            system_message: String::new(),
            temperature: None,
            top_p: None,
            // frequency_penalty: 0.0,
            // presence_penalty: 0.0,
        }
    }
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
        let json = json::parse(&json_str).map_err(|_| LlmErrorKind::OptionsNotSet)?;

        // Extract system_message
        let system_message = json["system_message"]
            .as_str()
            .ok_or(LlmErrorKind::OptionsNotSet)?
            .to_string();

        Ok(LlmOptions {
            system_message,
            temperature: json["temperature"].as_f32(),
            top_p: json["top_p"].as_f32(),
        })
    }
}

impl BlocklessLlm {
    pub fn new(model_name: &str) -> Result<Self, LlmErrorKind> {
        let mut llm = Self::default();
        llm.set_model(model_name)?;
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
            return Err(LlmErrorKind::OptionsNotSet);
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

    // TODO: response streaming - not yet supported
    // - read next available chunks
    // - block until chunk is read, repeat until no more chunks
    // pub fn read_response_chunk(&self, buf: &mut [u8]) -> Result<u32, LlmErrorKind> {
    //     let mut num: u32 = 0;
    //     let rs = unsafe {
    //         llm_read_prompt_response(self.inner, buf.as_mut_ptr(), buf.len() as _, &mut num)
    //     };

    //     if rs != 0 {
    //         return Err(LlmErrorKind::from(rs));
    //     }
    //     Ok(num)
    // }
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
    ModelNotSet,
    OptionsNotSet,
    Utf8Error,
    Unknown(u8),
}

impl From<u8> for LlmErrorKind {
    fn from(code: u8) -> Self {
        match code {
            1 => LlmErrorKind::ModelNotSet,
            2 => LlmErrorKind::OptionsNotSet,
            3 => LlmErrorKind::Utf8Error,
            _ => LlmErrorKind::Unknown(code),
        }
    }
}
