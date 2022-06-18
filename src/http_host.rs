#[link(wasm_import_module = "blockless_http")]
extern "C" {
    #[link_name = "http_open"]
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
