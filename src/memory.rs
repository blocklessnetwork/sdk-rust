#[link(wasm_import_module = "blockless_memory")]
extern "C" {
    #[link_name = "memory_read"]
    pub(crate) fn memory_read(buf: *mut u8, len: u32, num: *mut u32) -> u32;
    #[link_name = "env_var_read"]
    pub(crate) fn env_var_read(buf: *mut u8, len: u32, num: *mut u32) -> u32;
}


pub fn read_stdin(buf: &mut [u8]) -> std::io::Result<u32> {
    let mut len = 0;
    let errno = unsafe { memory_read(buf.as_mut_ptr(), buf.len() as _, &mut len) };
    if errno == 0 {
        return Ok(len);
    }
    let err = std::io::Error::from_raw_os_error(errno as i32);
    Err(err)
}

pub fn read_env_vars(buf: &mut [u8]) -> std::io::Result<u32> {
    let mut len = 0;
    let errno = unsafe { env_var_read(buf.as_mut_ptr(), buf.len() as _, &mut len) };
    if errno == 0 {
        return Ok(len);
    }
    let err = std::io::Error::from_raw_os_error(errno as i32);
    Err(err)
}
