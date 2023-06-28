use crate::memory_host::*;

pub fn read_stdin(buf: &mut [u8]) -> std::io::Result<u32> {
    let mut len = 0;
    let errno = unsafe { memory_read(buf.as_mut_ptr(), buf.len() as _, &mut len) };
    if errno == 0 {
        return Ok(len);
    }
    let err = std::io::Error::from_raw_os_error(errno as i32);
    Err(err)
}
