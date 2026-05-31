//! Runtime exception handling and error object constructors.

use std::cell::RefCell;

struct TryEntry {
    jmp_buf: *mut libc::c_void,
    shadow_stack_top: *mut crate::shadow_stack::ShadowFrame,
}

thread_local! {
    static TRY_STACK: RefCell<Vec<TryEntry>> = RefCell::new(Vec::new());
    static CURRENT_EXCEPTION: RefCell<u64> = RefCell::new(0);
}

#[no_mangle]
pub unsafe extern "C" fn __bs_try_enter(jmp_buf: *mut libc::c_void) {
    let shadow_top = crate::shadow_stack::get_shadow_stack_top();
    TRY_STACK.with(|stack| {
        stack.borrow_mut().push(TryEntry {
            jmp_buf,
            shadow_stack_top: shadow_top,
        });
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

    if let Some(entry) = top {
        // Restore shadow stack to the state it was at try-enter,
        // discarding dangling frames from functions unwound by longjmp.
        crate::shadow_stack::__bs_shadow_set(entry.shadow_stack_top);

        CURRENT_EXCEPTION.with(|ex| {
            *ex.borrow_mut() = value;
        });
        _longjmp(entry.jmp_buf, 1);
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