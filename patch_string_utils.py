with open("rt-stubs/src/types/string_utils.rs", "r") as f:
    code = f.read()

new_funcs = """#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_string_concat_owned(a: u64, b: u64) -> u64 {
    let res = __bs_string_concat(a, b);
    (res & 0x0000_FFFF_FFFF_FFFF) | 0x7FF7_0000_0000_0000 // TAG_OWNED_STRING
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_number_to_string_owned(n: u64) -> u64 {
    let res = super::coercion::__bs_number_to_string(n);
    (res & 0x0000_FFFF_FFFF_FFFF) | 0x7FF7_0000_0000_0000 // TAG_OWNED_STRING
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_boolean_to_string_owned(b: u64) -> u64 {
    let res = super::coercion::__bs_boolean_to_string(b);
    (res & 0x0000_FFFF_FFFF_FFFF) | 0x7FF7_0000_0000_0000 // TAG_OWNED_STRING
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_string_concat(a: u64, b: u64) -> u64 {"""

code = code.replace("""#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_string_concat(a: u64, b: u64) -> u64 {""", new_funcs)

with open("rt-stubs/src/types/string_utils.rs", "w") as f:
    f.write(code)
