use crate::dynamic_call::helpers::*;

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_index_get(obj: u64, index: u64) -> u64 {
    let tag = obj & TAG_MASK;
    if tag == TAG_ARRAY {
        crate::array::__bs_array_get(obj, index)
    } else {
        let prop_name = value_to_string(index);
        let prop_bytes = prop_name.as_bytes();
        crate::json::tape::__bs_prop_get(obj, prop_bytes.as_ptr(), prop_bytes.len() as u32)
    }
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_index_set(obj: u64, index: u64, val: u64) {
    let tag = obj & TAG_MASK;
    if tag == TAG_ARRAY {
        crate::array::__bs_array_set(obj, index, val);
    } else {
        let prop_name = value_to_string(index);
        let prop_bytes = prop_name.as_bytes();
        crate::json::tape::__bs_prop_set(obj, prop_bytes.as_ptr(), prop_bytes.len() as u32, val);
    }
}

