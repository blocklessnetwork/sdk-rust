use std::fmt::{Display, Debug};

use json::{JsonValue, object::Object, Array};

use crate::cgi_host::{cgi_close, cgi_list_exec, cgi_list_read, cgi_open, cgi_stderr_read};

pub struct CGIExtensions {
    pub file_name: String,
    pub alias: String,
    pub md5: String,
    pub description: String,
}

impl Display for CGIExtensions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let _ = f.write_fmt(format_args!("fileName: {}", self.file_name));
        let _ = f.write_fmt(format_args!("alias: {}", self.alias));
        let _ = f.write_fmt(format_args!("md5: {}", self.md5));
        f.write_fmt(format_args!("description: {}", self.description))
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
    handle: Option<u32>
}

impl CGICommand {
    fn new(command: String, args: Vec<String>, envs: Vec<CGIEnv>) -> Self  {
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

    pub fn read_all_stdin(&mut self) -> Result<Vec<u8>, CGIErrorKind> {
        let mut readn = 0u32; 
        let mut data: Vec<u8> = Vec::new(); 
        if self.handle.is_none() {
            return Ok(data);
        }
        let handle = self.handle.unwrap();
        let mut bs = [0u8; 1024];
        loop {
            unsafe {
                let rs = cgi_stderr_read(handle, &mut bs as _, bs.len() as _, &mut readn);
                if rs != 0 {
                    return Err(CGIErrorKind::StdinReadError);
                }
                if readn == 0 {
                    break;
                }
                data.copy_from_slice(&bs[..readn as _]);
            }
        }
        Ok(data)
    }

    pub fn exec_command(&mut self) -> Result<Vec<u8>, CGIErrorKind> {
        self.exec()?;
        self.read_all_stdin()
    }

    fn json_params(&self) -> String {
        let mut obj = Object::new();
        let command = JsonValue::String(self.command.clone());
        obj.insert("command", command);
        let args = self.args.iter().map(|arg| {
            JsonValue::String(arg.to_string())
        }).collect::<Vec<_>>();
        obj.insert("args", JsonValue::Array(args));
        let envs = self.envs.iter().map(|env| {
            let mut obj = Object::new();
            let name = JsonValue::String(env.name.clone());
            obj.insert("name", name);
            let value = JsonValue::String(env.value.clone());
            obj.insert("value", value);
            JsonValue::Object(obj)
        }).collect::<Vec<_>>();
        obj.insert("envs", JsonValue::Array(envs));
        obj.dump()
    }
}

#[derive(Debug)]
pub enum CGIErrorKind {
    ListError,
    EncodingError,
    JsonDecodingError,
    ExecError,
    StdinReadError,
}

struct CGIListExtensions(u32);

impl Drop for CGIListExtensions {
    fn drop(&mut self) {
        unsafe {
            cgi_close(self.0);
        }
    }
}

impl CGIListExtensions {
    fn new() ->  Result<Self, CGIErrorKind> {
        let mut cgi_handle: u32 = 0;
        unsafe{
            let rs = cgi_list_exec(&mut cgi_handle as *mut u32);
            if rs != 0 {
                return Err(CGIErrorKind::ListError);
            }
        };
        Ok(CGIListExtensions(cgi_handle))
    }

    fn list_read_all(&self) -> Result<Vec<u8>, CGIErrorKind> {
        let mut data: Vec<u8> = Vec::new();
        let mut bs = [0u8; 1024];
        let mut readn = 0u32; 
        loop {
            unsafe {
                let rs = cgi_list_read(self.0, &mut bs as _, bs.len() as _, &mut readn);
                if rs != 0 {
                    return Err(CGIErrorKind::ListError);
                }
                if readn == 0 {
                    break;
                }
                data.copy_from_slice(&bs[..readn as _]);
            }
        }
        Ok(data)
    }

    fn list(&self) -> Result<Vec<CGIExtensions>, CGIErrorKind> {
        let data = self.list_read_all()?;
        let s = std::str::from_utf8(&data)
            .map_err(|_| CGIErrorKind::EncodingError)?;
        let json = json::parse(s)
            .map_err(|_| CGIErrorKind::JsonDecodingError)?;
        let externs = json.members()
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