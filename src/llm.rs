use json::JsonValue;
use std::cmp::Ordering;

pub type Handle = u32;

#[link(wasm_import_module = "blockless_llm")]
extern "C" {
    fn llm_set_model_request(model_ptr: *const u8, model_len: u32, fd: *mut u32) -> i32;
    fn llm_get_model_response(buf: *mut u8, size: u32, num: *mut u32, fd: u32) -> i32;
    fn llm_set_model_options_request(options_ptr: *const u8, options_len: u32, fd: u32) -> i32;
    fn llm_get_model_options(buf: *mut u8, size: u32, num: *mut u32, fd: u32) -> i32;
    fn llm_prompt_request(prompt_ptr: *const u8, prompt_len: u32, fd: u32) -> i32;
    fn llm_read_prompt_response(buf: *mut u8, size: u32, num: *mut u32, fd: u32) -> i32;
    fn llm_close(fd: u32) -> i32;
}

#[derive(Debug, Clone, Default)]
pub struct BlocklessLlm {
    inner: Handle,
    model_name: String,
    options: LlmOptions,
}

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

    pub fn dump(&self) -> String {
        let mut json = JsonValue::new_object();
        json["system_message"] = self.system_message.clone().into();
        if let Some(temperature) = self.temperature {
            json["temperature"] = temperature.into();
        }
        if let Some(top_p) = self.top_p {
            json["top_p"] = top_p.into();
        }
        json.dump()
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

    pub fn get_model(&self) -> Result<String, LlmErrorKind> {
        let mut buf = [0u8; 256];
        let mut num: u32 = 0;
        let rs = unsafe {
            llm_get_model_response(buf.as_mut_ptr(), buf.len() as _, &mut num, self.inner)
        };
        if rs != 0 {
            return Err(LlmErrorKind::from(rs));
        }
        let model = String::from_utf8(buf[0..num as _].to_vec()).unwrap();
        Ok(model)
    }

    pub fn set_model(&mut self, model_name: &str) -> Result<(), LlmErrorKind> {
        self.model_name = model_name.to_string();
        // handle (self.inner set from runtime)
        let rs = unsafe {
            llm_set_model_request(model_name.as_ptr(), model_name.len() as _, &mut self.inner)
        };
        if rs != 0 {
            return Err(LlmErrorKind::from(rs));
        }

        // validate model is set correctly in host/runtime
        if self.model_name != self.get_model()? {
            eprintln!(
                "Model not set correctly in host/runtime; model_name: {}, model_from_host: {}",
                self.model_name,
                self.get_model()?
            );
            return Err(LlmErrorKind::ModelNotSet);
        }
        Ok(())
    }

    pub fn get_options(&self) -> Result<LlmOptions, LlmErrorKind> {
        let mut buf = [0u8; 256];
        let mut num: u32 = 0;
        let rs = unsafe {
            llm_get_model_options(buf.as_mut_ptr(), buf.len() as _, &mut num, self.inner)
        };
        if rs != 0 {
            println!("Error getting model options: {}", rs);
            return Err(LlmErrorKind::from(rs));
        }

        // Convert buffer slice to Vec<u8> and try to parse into LlmOptions
        LlmOptions::try_from(buf[0..num as usize].to_vec())
    }

    pub fn set_options(&mut self, options: LlmOptions) -> Result<(), LlmErrorKind> {
        let options_json = options.dump();
        self.options = options;
        let rs = unsafe {
            llm_set_model_options_request(
                options_json.as_ptr(),
                options_json.len() as _,
                self.inner,
            )
        };
        if rs != 0 {
            return Err(LlmErrorKind::from(rs));
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
        let rs = unsafe { llm_prompt_request(prompt.as_ptr(), prompt.len() as _, self.inner) };
        if rs != 0 {
            return Err(LlmErrorKind::from(rs));
        }

        // Read the response
        self.get_chat_response()
    }

    fn get_chat_response(&self) -> Result<String, LlmErrorKind> {
        let mut vec = Vec::new();
        loop {
            let mut buf = [0u8; 4096]; // Larger buffer for LLM responses
            let mut num: u32 = 0;
            let rs = unsafe {
                llm_read_prompt_response(buf.as_mut_ptr(), buf.len() as _, &mut num, self.inner)
            };

            if rs != 0 {
                return Err(LlmErrorKind::from(rs));
            }

            match num.cmp(&0) {
                Ordering::Greater => vec.extend_from_slice(&buf[0..num as _]),
                _ => break,
            }
        }
        String::from_utf8(vec).map_err(|_| LlmErrorKind::Utf8Error)
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
        unsafe {
            llm_close(self.inner);
        }
    }
}

#[derive(Debug)]
pub enum LlmErrorKind {
    ModelNotSet,
    OptionsNotSet,
    Utf8Error,
    Unknown(i32),
}

impl From<i32> for LlmErrorKind {
    fn from(code: i32) -> Self {
        match code {
            1 => LlmErrorKind::ModelNotSet,
            2 => LlmErrorKind::OptionsNotSet,
            3 => LlmErrorKind::Utf8Error,
            _ => LlmErrorKind::Unknown(code),
        }
    }
}
