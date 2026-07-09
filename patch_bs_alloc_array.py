with open("rt-stubs/src/array/mod.rs", "r") as f:
    code = f.read()

code = code.replace("""pub unsafe extern "C-unwind" fn __bs_array_new() -> u64 {""", """pub unsafe extern "C-unwind" fn __bs_array_new() -> u64 {
    __bs_alloc_array()
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_alloc_array() -> u64 {""")

with open("rt-stubs/src/array/mod.rs", "w") as f:
    f.write(code)
