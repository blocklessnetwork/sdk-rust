use crate::SocketErrorKind;

#[cfg(not(feature = "mock-ffi"))]
#[link(wasm_import_module = "blockless_socket")]
extern "C" {
    #[link_name = "create_tcp_bind_socket"]
    pub(crate) fn create_tcp_bind_socket_native(
        addr: *const u8,
        addr_len: u32,
        fd: *mut u32,
    ) -> u32;
}

#[cfg(feature = "mock-ffi")]
mod mock_ffi {
    pub unsafe fn create_tcp_bind_socket_native(
        _addr: *const u8,
        _addr_len: u32,
        _fd: *mut u32,
    ) -> u32 {
        unimplemented!()
    }
}

#[cfg(feature = "mock-ffi")]
use mock_ffi::*;

pub fn create_tcp_bind_socket(addr: &str) -> Result<u32, SocketErrorKind> {
    unsafe {
        let addr_ptr = addr.as_ptr();
        let mut fd: u32 = 0;
        let rs = create_tcp_bind_socket_native(addr_ptr, addr.len() as _, (&mut fd) as *mut u32);
        if rs == 0 {
            return Ok(fd);
        }
        Err(match rs {
            1 => SocketErrorKind::ConnectRefused,
            2 => SocketErrorKind::ParameterError,
            3 => SocketErrorKind::ConnectionReset,
            4 => SocketErrorKind::AddressInUse,
            _ => unreachable!("unreach."),
        })
    }
}
