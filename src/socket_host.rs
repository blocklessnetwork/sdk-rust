#[link(wasm_import_module = "blockless_socket")]
extern "C" {
    #[link_name = "create_tcp_bind_socket"]
    pub(crate) fn create_tcp_bind_socket_native(
        addr: *const u8,
        addr_len: u32,
        fd: *mut u32,
    ) -> u32;
}
