use crate::types::string_utils::{get_c_string_from_tagged, create_tagged_string};

#[no_mangle]
pub unsafe extern "C" fn __bs_fs_read_file_sync(path_tagged: u64) -> u64 {
    let path_str = get_c_string_from_tagged(path_tagged);
    match std::fs::read_to_string(path_str) {
        Ok(content) => create_tagged_string(&content),
        Err(_) => create_tagged_string(""),
    }
}

#[no_mangle]
pub unsafe extern "C" fn __bs_fs_write_file_sync(path_tagged: u64, data_tagged: u64) {
    let path_str = get_c_string_from_tagged(path_tagged);
    let data_str = get_c_string_from_tagged(data_tagged);
    let _ = std::fs::write(path_str, data_str);
}

#[no_mangle]
pub unsafe extern "C" fn __bs_fs_exists_sync(path_tagged: u64) -> u64 {
    let path_str = get_c_string_from_tagged(path_tagged);
    if std::path::Path::new(path_str).exists() {
        0xFFF4_0000_0000_0000 // TAG_TRUE
    } else {
        0xFFF3_0000_0000_0000 // TAG_FALSE
    }
}
