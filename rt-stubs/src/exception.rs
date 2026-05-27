//! Runtime exception handling and error object constructors.

use std::cell::RefCell;

thread_local! {
    static TRY_STACK: RefCell<Vec<*mut libc::c_void>> = RefCell::new(Vec::new());
    static CURRENT_EXCEPTION: RefCell<u64> = RefCell::new(0);
}

#[no_mangle]
pub unsafe extern "C" fn __bs_try_enter(jmp_buf: *mut libc::c_void) {
    TRY_STACK.with(|stack| {
        stack.borrow_mut().push(jmp_buf);
    });
}

#[no_mangle]
pub unsafe extern "C" fn __bs_try_exit() {
    TRY_STACK.with(|stack| {
        stack.borrow_mut().pop();
    });
}

extern "C" {
    pub fn _longjmp(env: *mut libc::c_void, val: libc::c_int) -> !;
}

#[no_mangle]
pub unsafe extern "C" fn __bs_throw(value: u64) -> ! {
    let top = TRY_STACK.with(|stack| {
        stack.borrow_mut().pop()
    });

    if let Some(jmp_buf) = top {
        CURRENT_EXCEPTION.with(|ex| {
            *ex.borrow_mut() = value;
        });
        _longjmp(jmp_buf, 1);
    } else {
        print_exception(value);
        std::process::exit(1);
    }
}

#[no_mangle]
pub unsafe extern "C" fn __bs_get_and_clear_exception() -> u64 {
    CURRENT_EXCEPTION.with(|ex| {
        let val = *ex.borrow();
        *ex.borrow_mut() = 0;
        val
    })
}

#[no_mangle]
pub unsafe extern "C" fn __bs_error_new(message_tagged: u64, name_ptr: *const u8) -> u64 {
    let obj = crate::__bs_new_object();
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

unsafe fn print_exception(val: u64) {
    let tag = val & 0xFFFF_0000_0000_0000;
    if tag == 0xFFF7_0000_0000_0000 {
        let s = crate::get_c_string_from_tagged(val);
        eprintln!("Unhandled Exception: {}", s);
    } else if tag == 0xFFF6_0000_0000_0000 {
        let payload = val & 0x0000_FFFF_FFFF_FFFF;
        if payload != 0 {
            let name_val = crate::get_dynamic_property(payload as *mut u8, "name");
            let message_val = crate::get_dynamic_property(payload as *mut u8, "message");
            let stack_val = crate::get_dynamic_property(payload as *mut u8, "stack");

            let name = name_val.map(|n| crate::get_c_string_from_tagged(n)).unwrap_or("Error");
            let message = message_val.map(|m| crate::get_c_string_from_tagged(m)).unwrap_or("");
            let stack = stack_val.map(|s| crate::get_c_string_from_tagged(s)).unwrap_or("");

            eprintln!("{}: {}", name, message);
            if !stack.is_empty() {
                eprintln!("{}", stack);
            }
        } else {
            eprintln!("Unhandled Exception: [null object]");
        }
    } else if tag == 0 || (tag > 0 && tag < 0xFFF0_0000_0000_0000) {
        eprintln!("Unhandled Exception: {}", f64::from_bits(val));
    } else if tag == 0xFFF3_0000_0000_0000 {
        eprintln!("Unhandled Exception: false");
    } else if tag == 0xFFF4_0000_0000_0000 {
        eprintln!("Unhandled Exception: true");
    } else {
        eprintln!("Unhandled Exception: {:X}", val);
    }
}
