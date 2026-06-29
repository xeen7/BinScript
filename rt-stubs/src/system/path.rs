use crate::types::string_utils::{get_c_string_from_tagged, create_tagged_string};

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_path_join(a_tagged: u64, b_tagged: u64) -> u64 {
    let a_str = get_c_string_from_tagged(a_tagged);
    let b_str = get_c_string_from_tagged(b_tagged);
    let path = std::path::PathBuf::from(a_str).join(b_str);
    create_tagged_string(&path.to_string_lossy())
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_path_resolve(a_tagged: u64, b_tagged: u64) -> u64 {
    let a_str = get_c_string_from_tagged(a_tagged);
    let b_str = get_c_string_from_tagged(b_tagged);
    let joined = std::path::Path::new(a_str).join(b_str);
    create_tagged_string(&joined.to_string_lossy())
}
