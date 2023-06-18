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
