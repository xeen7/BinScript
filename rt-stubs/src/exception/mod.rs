//! Runtime exception handling and error object constructors.

pub mod personality;


#[repr(C)]
pub struct _Unwind_Exception {
    pub exception_class: u64,
    pub exception_cleanup: Option<extern "C-unwind" fn(u32, *mut _Unwind_Exception)>,
    pub private_1: u64,
    pub private_2: u64,
}

#[repr(C)]
pub struct BinScriptException {
    pub unwind_header: _Unwind_Exception,
    pub value: u64,
}

extern "C-unwind" {
    fn _Unwind_RaiseException(exception_object: *mut _Unwind_Exception) -> u32;
}


#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_throw(value: u64) {
    // Allocate enough memory for BinScriptException
    // We allocate on the heap so it lives during unwinding.
    // Use BinScript exception class: "BinScr\x70\x74"
    let bs_ex_box = Box::new(BinScriptException {
        unwind_header: _Unwind_Exception {
            exception_class: 0x42696E5363727074,
            exception_cleanup: None,
            private_1: 0,
            private_2: 0,
        },
        value,
    });
    let exn_ptr = Box::into_raw(bs_ex_box) as *mut _Unwind_Exception;

    let _res = _Unwind_RaiseException(exn_ptr);
    libc::printf("Unwind failed\n\0".as_ptr() as *const i8);
    std::process::abort();
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_free_exception(exception_object: *mut _Unwind_Exception) {
    // The value's ownership was transferred to the catch block via __bs_get_exception_value,
    // so we only deallocate the wrapper.
    let layout = std::alloc::Layout::new::<BinScriptException>();
    std::alloc::dealloc(exception_object as *mut u8, layout);
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_get_exception_value(exception_object: *mut _Unwind_Exception) -> u64 {
    // We embedded the BinScriptException value in the exception object
    let bs_ex = exception_object as *mut BinScriptException;
    (*bs_ex).value
}

extern "C-unwind" fn exception_cleanup_fn(_reason: u32, exn_ptr: *mut _Unwind_Exception) {
    unsafe {
        let layout = std::alloc::Layout::new::<BinScriptException>();
        std::alloc::dealloc(exn_ptr as *mut u8, layout);
    }
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
    } else if crate::dynamic_call::helpers::is_number_tag(tag) {
        eprintln!("Unhandled Exception: {}", f64::from_bits(val));
    } else if tag == 0xFFF3_0000_0000_0000 {
        eprintln!("Unhandled Exception: false");
    } else if tag == 0xFFF4_0000_0000_0000 {
        eprintln!("Unhandled Exception: true");
    } else {
        eprintln!("Unhandled Exception: {:X}", val);
    }
}

#[allow(non_camel_case_types)]
type _Unwind_Reason_Code = u32;
#[allow(non_camel_case_types)]
type _Unwind_Action = u32;

const _URC_NO_REASON: _Unwind_Reason_Code = 0;
const _URC_FOREIGN_EXCEPTION_CAUGHT: _Unwind_Reason_Code = 1;
const _URC_FATAL_PHASE2_ERROR: _Unwind_Reason_Code = 2;
const _URC_FATAL_PHASE1_ERROR: _Unwind_Reason_Code = 3;
const _URC_NORMAL_STOP: _Unwind_Reason_Code = 4;
const _URC_END_OF_STACK: _Unwind_Reason_Code = 5;
const _URC_HANDLER_FOUND: _Unwind_Reason_Code = 6;
const _URC_INSTALL_CONTEXT: _Unwind_Reason_Code = 7;
const _URC_CONTINUE_UNWIND: _Unwind_Reason_Code = 8;

const _UA_SEARCH_PHASE: _Unwind_Action = 1;
const _UA_CLEANUP_PHASE: _Unwind_Action = 2;
const _UA_HANDLER_FRAME: _Unwind_Action = 4;
const _UA_FORCE_UNWIND: _Unwind_Action = 8;
const _UA_END_OF_STACK: _Unwind_Action = 16;


