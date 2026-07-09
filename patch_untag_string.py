with open("rt-stubs/src/types/string_utils.rs", "r") as f:
    code = f.read()

code = code.replace("""pub(crate) unsafe fn untag_string(tagged: u64) -> *mut BsString {
    if (tagged & TAG_MASK) != 0xFFF7_0000_0000_0000 {
        return std::ptr::null_mut();
    }""", """pub(crate) unsafe fn untag_string(tagged: u64) -> *mut BsString {
    let tag = tagged & TAG_MASK;
    if tag != 0xFFF7_0000_0000_0000 && tag != 0x7FF7_0000_0000_0000 {
        return std::ptr::null_mut();
    }""")

with open("rt-stubs/src/types/string_utils.rs", "w") as f:
    f.write(code)
