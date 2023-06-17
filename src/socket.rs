use crate::socket_host::create_tcp_bind_socket_native;

#[derive(Debug)]
pub enum SocketErrorKind {
    ConnectRefused,
    ParameterError,
    ConnectionReset,
    AddressInUse,
}

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