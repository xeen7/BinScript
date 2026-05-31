
#[no_mangle]
pub unsafe extern "C" fn __bs_error_new(message_tagged: u64, name_ptr: *const u8) -> u64 {
    let obj = crate::__bs_alloc(&crate::ERROR_VTABLE, 8);
    let payload = obj & 0x0000_FFFF_FFFF_FFFF;

    // Set message
    crate::set_dynamic_property(payload as *mut u8, "message".to_string(), message_tagged);

    // Set name
    let name_c = std::ffi::CStr::from_ptr(name_ptr as *const libc::c_char);
    let name_str = name_c.to_str().unwrap_or("Error");
    let name_tagged = crate::create_tagged_string(name_str);
    crate::set_dynamic_property(payload as *mut u8, "name".to_string(), name_tagged);

    // Set stack
    let stack_str = format!("    at <native>\n    at main");
    let stack_tagged = crate::create_tagged_string(&stack_str);
    crate::set_dynamic_property(payload as *mut u8, "stack".to_string(), stack_tagged);

    obj
}

#[no_mangle]
pub unsafe extern "C" fn __bs_Error_new(message_tagged: u64) -> u64 {
    __bs_error_new(message_tagged, b"Error\0".as_ptr())
}

#[no_mangle]
pub unsafe extern "C" fn __bs_TypeError_new(message_tagged: u64) -> u64 {
    __bs_error_new(message_tagged, b"TypeError\0".as_ptr())
}

#[no_mangle]
pub unsafe extern "C" fn __bs_RangeError_new(message_tagged: u64) -> u64 {
    __bs_error_new(message_tagged, b"RangeError\0".as_ptr())
}

#[no_mangle]
pub unsafe extern "C" fn __bs_ReferenceError_new(message_tagged: u64) -> u64 {
    __bs_error_new(message_tagged, b"ReferenceError\0".as_ptr())
}

#[no_mangle]
pub unsafe extern "C" fn __bs_SyntaxError_new(message_tagged: u64) -> u64 {
    __bs_error_new(message_tagged, b"SyntaxError\0".as_ptr())
}

#[no_mangle]
pub unsafe extern "C" fn __bs_URIError_new(message_tagged: u64) -> u64 {
    __bs_error_new(message_tagged, b"URIError\0".as_ptr())
}

