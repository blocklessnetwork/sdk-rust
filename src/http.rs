use crate::error::HttpErrorKind;
use json::JsonValue;
use std::{cmp::Ordering, collections::BTreeMap};

#[link(wasm_import_module = "blockless_http")]
extern "C" {
    #[link_name = "http_req"]
    pub(crate) fn http_open(
        url: *const u8,
        url_len: u32,
        opts: *const u8,
        opts_len: u32,
        fd: *mut u32,
        status: *mut u32,
    ) -> u32;

    #[link_name = "http_read_header"]
    pub(crate) fn http_read_header(
        handle: u32,
        header: *const u8,
        header_len: u32,
        buf: *mut u8,
        buf_len: u32,
        num: *mut u32,
    ) -> u32;

    #[link_name = "http_read_body"]
    pub(crate) fn http_read_body(handle: u32, buf: *mut u8, buf_len: u32, num: *mut u32) -> u32;

    #[link_name = "http_close"]
    pub(crate) fn http_close(handle: u32) -> u32;
}

type Handle = u32;
type ExitCode = u32;

pub struct BlocklessHttp {
    inner: Handle,
    code: ExitCode,
}

pub struct HttpOptions {
    pub method: String,
    pub connect_timeout: u32,
    pub read_timeout: u32,
    pub body: Option<String>,
    pub headers: Option<BTreeMap<String, String>>,
}

impl HttpOptions {
    pub fn new(method: &str, connect_timeout: u32, read_timeout: u32) -> Self {
        HttpOptions {
            method: method.into(),
            connect_timeout,
            read_timeout,
            body: None,
            headers: None,
        }
    }

    pub fn dump(&self) -> String {
        // convert BTreeMap to json string
        let mut headers_str = self
            .headers
            .clone()
            .unwrap_or_default()
            .iter()
            .map(|(k, v)| format!("\"{}\":\"{}\"", k, v))
            .collect::<Vec<String>>()
            .join(",");
        headers_str = format!("{{{}}}", headers_str);

        let mut json = JsonValue::new_object();
        json["method"] = self.method.clone().into();
        json["connectTimeout"] = self.connect_timeout.into();
        json["readTimeout"] = self.read_timeout.into();
        json["headers"] = headers_str.into();
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

    pub fn get_code(&self) -> ExitCode {
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

impl Drop for BlocklessHttp {
    fn drop(&mut self) {
        unsafe {
            http_close(self.inner);
        }
    }
}
