use super::string_utils::create_tagged_string;

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_typeof(val: u64) -> u64 {
    let tag = val & 0xFFFF_0000_0000_0000;
    let s = if tag == 0xFFF1_0000_0000_0000 {
        "undefined"
    } else if tag == 0xFFF2_0000_0000_0000 {
        "object" // null
    } else if tag == 0xFFF3_0000_0000_0000 || tag == 0xFFF4_0000_0000_0000 {
        "boolean"
    } else if tag == 0xFFF7_0000_0000_0000 {
        "string"
    } else if tag == 0xFFF8_0000_0000_0000 {
        "symbol"
    } else if tag == 0xFFF9_0000_0000_0000 {
        "function" // closure
    } else if tag == 0xFFFA_0000_0000_0000 {
        "object" // generator
    } else if tag == 0xFFFB_0000_0000_0000 {
        "object" // array
    } else if tag == 0xFFFC_0000_0000_0000 {
        "object" // promise
    } else if tag == 0xFFF6_0000_0000_0000 {
        // object or class instance
        "object"
    } else {
        "number"
    };
    create_tagged_string(s)
}
