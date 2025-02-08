use std::fmt::{Debug, Display};
use json::{object::Object, JsonValue};
use crate::CGIErrorKind;

#[link(wasm_import_module = "blockless_cgi")]
extern "C" {
    #[link_name = "cgi_open"]
    pub(crate) fn cgi_open(opts: *const u8, opts_len: u32, cgi_handle: *mut u32) -> u32;

    #[link_name = "cgi_stdout_read"]
    pub(crate) fn cgi_stdout_read(handle: u32, buf: *mut u8, buf_len: u32, num: *mut u32) -> u32;

    #[link_name = "cgi_stderr_read"]
    pub(crate) fn cgi_stderr_read(handle: u32, buf: *mut u8, buf_len: u32, num: *mut u32) -> u32;

    #[link_name = "cgi_stdin_write"]
    #[allow(dead_code)]
    pub(crate) fn cgi_stdin_write(handle: u32, buf: *const u8, buf_len: u32, num: *mut u32) -> u32;

    #[link_name = "cgi_close"]
    pub(crate) fn cgi_close(handle: u32) -> u32;

    #[link_name = "cgi_list_exec"]
    pub(crate) fn cgi_list_exec(cgi_handle: *mut u32) -> u32;

    #[link_name = "cgi_list_read"]
    pub(crate) fn cgi_list_read(handle: u32, buf: *mut u8, buf_len: u32, num: *mut u32) -> u32;
}

#[derive(Debug)]
pub struct CGIExtensions {
    pub file_name: String,
    pub alias: String,
    pub md5: String,
    pub description: String,
}

impl Display for CGIExtensions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "fileName: {},", self.file_name)?;
        write!(f, "alias: {},", self.alias)?;
        write!(f, "md5: {},", self.md5)?;
        write!(f, "description: {},", self.description)
    }
}

pub struct CGIEnv {
    pub name: String,
    pub value: String,
}

pub struct CGICommand {
    command: String,
    args: Vec<String>,
    envs: Vec<CGIEnv>,
    handle: Option<u32>,
}

type ReadFn = unsafe extern "C" fn(u32, *mut u8, u32, *mut u32) -> u32;

impl CGICommand {
    fn new(command: String, args: Vec<String>, envs: Vec<CGIEnv>) -> Self {
        Self {
            command,
            args,
            envs,
            handle: None,
        }
    }

    pub fn exec(&mut self) -> Result<(), CGIErrorKind> {
        let mut handle = 0u32;
        let parmas = self.json_params();
        unsafe {
            let rs = cgi_open(parmas.as_ptr(), parmas.len() as _, &mut handle);
            if rs != 0 {
                return Err(CGIErrorKind::ExecError);
            }
        };
        self.handle = Some(handle);
        Ok(())
    }

    fn read_all(&mut self, read_call: ReadFn) -> Result<Vec<u8>, CGIErrorKind> {
        let mut readn = 0u32;
        let mut data: Vec<u8> = Vec::new();
        if self.handle.is_none() {
            return Ok(data);
        }
        let handle = self.handle.unwrap();
        let mut bs = [0u8; 1024];
        loop {
            unsafe {
                let rs = read_call(handle, &mut bs as _, bs.len() as _, &mut readn);
                if rs != 0 {
                    return Err(CGIErrorKind::ReadError);
                }
                if readn == 0 {
                    break;
                }
                data.extend_from_slice(&bs[..readn as _]);
            }
        }
        Ok(data)
    }

    pub fn read_all_stdin(&mut self) -> Result<Vec<u8>, CGIErrorKind> {
        self.read_all(cgi_stdout_read)
    }

    pub fn read_all_stderr(&mut self) -> Result<Vec<u8>, CGIErrorKind> {
        self.read_all(cgi_stderr_read)
    }

    pub fn exec_command(&mut self) -> Result<String, CGIErrorKind> {
        self.exec()?;
        let bs = self.read_all_stdin()?;
        String::from_utf8(bs).map_err(|_| CGIErrorKind::EncodingError)
    }

    fn json_params(&self) -> String {
        let mut obj = Object::new();
        let command = JsonValue::String(self.command.clone());
        obj.insert("command", command);
        let args = self
            .args
            .iter()
            .map(|arg| JsonValue::String(arg.to_string()))
            .collect::<Vec<_>>();
        obj.insert("args", JsonValue::Array(args));
        let envs = self
            .envs
            .iter()
            .map(|env| {
                let mut obj = Object::new();
                let name = JsonValue::String(env.name.clone());
                obj.insert("name", name);
                let value = JsonValue::String(env.value.clone());
                obj.insert("value", value);
                JsonValue::Object(obj)
            })
            .collect::<Vec<_>>();
        obj.insert("envs", JsonValue::Array(envs));
        obj.dump()
    }
}

pub struct CGIListExtensions {
    handle: u32,
}

impl Drop for CGIListExtensions {
    fn drop(&mut self) {
        unsafe {
            cgi_close(self.handle);
        }
    }
}

impl CGIListExtensions {
    pub fn new() -> Result<Self, CGIErrorKind> {
        let mut cgi_handle: u32 = 0;
        unsafe {
            let rs = cgi_list_exec(&mut cgi_handle as *mut u32);
            if rs != 0 {
                return Err(CGIErrorKind::ListError);
            }
        };
        Ok(CGIListExtensions { handle: cgi_handle })
    }

    fn list_read_all(&self) -> Result<Vec<u8>, CGIErrorKind> {
        let mut data: Vec<u8> = Vec::new();
        let mut bs = [0u8; 1024];
        let mut readn = 0u32;
        loop {
            unsafe {
                let rs = cgi_list_read(self.handle, &mut bs as _, bs.len() as _, &mut readn);
                if rs != 0 {
                    return Err(CGIErrorKind::ListError);
                }
                if readn == 0 {
                    break;
                }
                data.extend_from_slice(&bs[..readn as _]);
            }
        }
        Ok(data)
    }

    pub fn command(
        &self,
        command: &str,
        args: Vec<String>,
        envs: Vec<CGIEnv>,
    ) -> Result<CGICommand, CGIErrorKind> {
        let extensions = self.list()?;
        extensions
            .iter()
            .find(|ext| if &ext.alias == command { true } else { false })
            .map(|_| CGICommand::new(command.to_string(), args, envs))
            .ok_or(CGIErrorKind::NoCommandError)
    }

    pub fn list(&self) -> Result<Vec<CGIExtensions>, CGIErrorKind> {
        let data = self.list_read_all()?;
        let s = std::str::from_utf8(&data).map_err(|_| CGIErrorKind::EncodingError)?;
        let json = json::parse(s).map_err(|_| CGIErrorKind::JsonDecodingError)?;
        let externs = json
            .members()
            .map(|json| {
                let file_name = json["fileName"].as_str().unwrap_or("").to_string();
                let alias = json["alias"].as_str().unwrap_or("").to_string();
                let md5 = json["md5"].as_str().unwrap_or("").to_string();
                let description = json["description"].as_str().unwrap_or("").to_string();
                CGIExtensions {
                    description,
                    file_name,
                    alias,
                    md5,
                }
            })
            .collect::<Vec<_>>();
        Ok(externs)
    }
}
