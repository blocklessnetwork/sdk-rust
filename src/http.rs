use std::cmp::Ordering;

use crate::{error::HttpErrorKind, http_host::*};
use json::JsonValue;

pub type Hanlde = u32;

pub type CodeStatus = u32;

pub struct BlocklessHttp {
    inner: Hanlde,
    code: CodeStatus,
}

pub struct HttpOptions {
    method: String,
    connect_timeout: u32,
    read_timeout: u32,
    body: Option<String>,
}

impl HttpOptions {
    pub fn new(method: &str, connect_timeout: u32, read_timeout: u32) -> Self {
        HttpOptions {
            method: method.into(),
            connect_timeout,
            read_timeout,
            body: None,
        }
    }

    pub fn dump(&self) -> String {
        let mut json = JsonValue::new_object();
        json["method"] = self.method.clone().into();
        json["connectTimeout"] = self.connect_timeout.into();
        json["readTimeout"] = self.read_timeout.into();
        json["headers"] = "{}".into();
        json["body"] = self.body.clone().into();
        json.dump()
    }
}

impl BlocklessHttp {
    pub fn open(url: &str, opts: &HttpOptions) -> Result<Self, HttpErrorKind> {
        let opts = opts.dump();
        let mut fd = 0;
        let mut status = 0;
        let rs = unsafe {
            http_open(
                url.as_ptr(),
                url.len() as _,
                opts.as_ptr(),
                opts.len() as _,
                &mut fd,
                &mut status,
            )
        };
        if rs != 0 {
            return Err(HttpErrorKind::from(rs));
        }
        Ok(Self {
            inner: fd,
            code: status,
        })
    }

    pub fn get_code(&self) -> CodeStatus {
        self.code
    }

    pub fn get_all_body(&self) -> Result<Vec<u8>, HttpErrorKind> {
        let mut vec = Vec::new();
        loop {
            let mut buf = [0u8; 1024];
            let mut num: u32 = 0;
            let rs =
                unsafe { http_read_body(self.inner, buf.as_mut_ptr(), buf.len() as _, &mut num) };
            if rs != 0 {
                return Err(HttpErrorKind::from(rs));
            }

            match num.cmp(&0) {
                Ordering::Greater => vec.extend_from_slice(&buf[0..num as _]),
                _ => break,
            }
        }
        Ok(vec)
    }

    pub fn get_header(&self, header: &str) -> Result<String, HttpErrorKind> {
        let mut vec = Vec::new();
        loop {
            let mut buf = [0u8; 1024];
            let mut num: u32 = 0;
            let rs = unsafe {
                http_read_header(
                    self.inner,
                    header.as_ptr(),
                    header.len() as _,
                    buf.as_mut_ptr(),
                    buf.len() as _,
                    &mut num,
                )
            };
            if rs != 0 {
                return Err(HttpErrorKind::from(rs));
            }
            match num.cmp(&0) {
                Ordering::Greater => vec.extend_from_slice(&buf[0..num as _]),
                _ => break,
            }
        }
        String::from_utf8(vec).map_err(|_| HttpErrorKind::Utf8Error)
    }

    pub fn close(self) {
        unsafe {
            http_close(self.inner);
        }
    }

    pub fn read_body(&self, buf: &mut [u8]) -> Result<u32, HttpErrorKind> {
        let mut num: u32 = 0;
        let rs = unsafe { http_read_body(self.inner, buf.as_mut_ptr(), buf.len() as _, &mut num) };
        if rs != 0 {
            return Err(HttpErrorKind::from(rs));
        }
        Ok(num)
    }
}
