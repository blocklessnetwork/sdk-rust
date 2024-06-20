
#[link(wasm_import_module = "blockless_memory")]
extern "C" {
    #[link_name = "memory_read"]
    pub(crate) fn memory_read(buf: *mut u8, len: u32, num: *mut u32) -> u32;
    #[link_name = "env_var_read"]
    pub(crate) fn env_var_read(buf: *mut u8, len: u32, num: *mut u32) -> u32;
}
