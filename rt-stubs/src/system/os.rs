use crate::types::string_utils::create_tagged_string;

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_os_platform() -> u64 {
    create_tagged_string("linux")
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_os_arch() -> u64 {
    create_tagged_string("x64")
}
