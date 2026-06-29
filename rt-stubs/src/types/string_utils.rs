// Helper to extract a Rust &str from a NaN-boxed string pointer
pub unsafe fn get_c_string_from_tagged(val: u64) -> &'static str {
    let tag = val & 0xFFFF_0000_0000_0000;
    if tag != 0xFFF7_0000_0000_0000 {
        panic!("Expected string value");
    }
    let payload = val & 0x0000_FFFF_FFFF_FFFF;
    let c_str = std::ffi::CStr::from_ptr(payload as *const libc::c_char);
    c_str.to_str().expect("Invalid UTF-8 string")
}

// Helper to allocate a null-terminated string using malloc and return it boxed
pub unsafe fn create_tagged_string(s: &str) -> u64 {
    let len = s.len();
    let ptr = libc::malloc(len + 1) as *mut u8;
    std::ptr::copy_nonoverlapping(s.as_ptr(), ptr, len);
    *ptr.add(len) = 0; // null terminator
    (ptr as u64) | 0xFFF7_0000_0000_0000
}

fn encode_uri_str(s: &str, is_component: bool) -> String {
    let mut res = String::new();
    for b in s.bytes() {
        let is_unescaped = match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'!' | b'~' | b'*' | b'\'' | b'(' | b')' => true,
            b';' | b',' | b'/' | b'?' | b':' | b'@' | b'&' | b'=' | b'+' | b'$' | b'#' if !is_component => true,
            _ => false,
        };
        if is_unescaped {
            res.push(b as char);
        } else {
            res.push_str(&format!("%{:02X}", b));
        }
    }
    res
}

fn decode_uri_str(s: &str) -> Option<String> {
    let mut bytes = Vec::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '%' {
            if i + 2 < chars.len() {
                let hex = format!("{}{}", chars[i+1], chars[i+2]);
                if let Ok(b) = u8::from_str_radix(&hex, 16) {
                    bytes.push(b);
                    i += 3;
                    continue;
                }
            }
            return None;
        } else {
            let mut buf = [0; 4];
            for &b in chars[i].encode_utf8(&mut buf).as_bytes() {
                bytes.push(b);
            }
            i += 1;
        }
    }
    String::from_utf8(bytes).ok()
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_encodeURI(val: u64) -> u64 {
    let s_tagged = crate::types::coercion::__bs_String(val);
    let s = get_c_string_from_tagged(s_tagged);
    let encoded = encode_uri_str(s, false);
    create_tagged_string(&encoded)
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_decodeURI(val: u64) -> u64 {
    let s_tagged = crate::types::coercion::__bs_String(val);
    let s = get_c_string_from_tagged(s_tagged);
    if let Some(decoded) = decode_uri_str(s) {
        create_tagged_string(&decoded)
    } else {
        let msg = create_tagged_string("URI malformed");
        crate::exception::__bs_throw(crate::objects::builtins::__bs_URIError_new(msg));
        0 // Unreachable
    }
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_encodeURIComponent(val: u64) -> u64 {
    let s_tagged = crate::types::coercion::__bs_String(val);
    let s = get_c_string_from_tagged(s_tagged);
    let encoded = encode_uri_str(s, true);
    create_tagged_string(&encoded)
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_decodeURIComponent(val: u64) -> u64 {
    let s_tagged = crate::types::coercion::__bs_String(val);
    let s = get_c_string_from_tagged(s_tagged);
    if let Some(decoded) = decode_uri_str(s) {
        create_tagged_string(&decoded)
    } else {
        let msg = create_tagged_string("URI malformed");
        crate::exception::__bs_throw(crate::objects::builtins::__bs_URIError_new(msg));
        0 // Unreachable
    }
}
