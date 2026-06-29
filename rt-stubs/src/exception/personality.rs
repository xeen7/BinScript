use crate::exception::_Unwind_Exception;

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
#[allow(dead_code)]
const _UA_FORCE_UNWIND: _Unwind_Action = 8;
#[allow(dead_code)]
const _UA_END_OF_STACK: _Unwind_Action = 16;

extern "C-unwind" {
    fn _Unwind_GetLanguageSpecificData(context: *mut libc::c_void) -> *const u8;
    fn _Unwind_GetRegionStart(context: *mut libc::c_void) -> usize;
    fn _Unwind_GetIP(context: *mut libc::c_void) -> usize;
    fn _Unwind_SetGR(context: *mut libc::c_void, index: i32, value: usize);
    fn _Unwind_SetIP(context: *mut libc::c_void, value: usize);
}

/// Custom personality function for BinScript exception handling.
///
/// Called by the system unwinder during both search phase (phase 1) and
/// cleanup phase (phase 2). Parses the LSDA (Language Specific Data Area)
/// to find matching landing pads for the current IP.
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_personality_v0(
    version: i32,
    actions: _Unwind_Action,
    _exception_class: u64,
    exception_object: *mut _Unwind_Exception,
    context: *mut libc::c_void,
) -> _Unwind_Reason_Code {
    if version != 1 {
        return _URC_FATAL_PHASE1_ERROR;
    }

    let lsda = _Unwind_GetLanguageSpecificData(context);
    if lsda.is_null() {
        return _URC_CONTINUE_UNWIND;
    }

    let region_start = _Unwind_GetRegionStart(context);
    let mut p = lsda;

    // Read lpStartEncoding
    let lp_start_encoding = *p; p = p.add(1);
    let mut lp_start = region_start;
    if lp_start_encoding != 0xff {
        lp_start = read_encoded_pointer(&mut p, lp_start_encoding);
    }

    // Read ttypeEncoding
    let ttype_encoding = *p; p = p.add(1);
    if ttype_encoding != 0xff {
        read_uleb128(&mut p); // skip ttype base offset
    }

    // Read call site table
    let call_site_encoding = *p; p = p.add(1);
    let call_site_length = read_uleb128(&mut p);
    let call_site_table_end = p.add(call_site_length);

    // IP is the return address, subtract 1 to get the call instruction
    let ip = _Unwind_GetIP(context);
    let offset = ip.wrapping_sub(1).wrapping_sub(region_start);

    let mut landing_pad: usize = 0;
    let mut action_val: usize = 0;

    while p < call_site_table_end {
        let start = read_encoded_pointer(&mut p, call_site_encoding);
        let length = read_encoded_pointer(&mut p, call_site_encoding);
        let lp = read_encoded_pointer(&mut p, call_site_encoding);
        let action = read_uleb128(&mut p);

        if offset >= start && offset < start + length {
            if lp != 0 {
                landing_pad = lp_start + lp;
                action_val = action;
            }
            break;
        }
    }

    if landing_pad == 0 {
        return _URC_CONTINUE_UNWIND;
    }

    // Phase 1 search: only report HANDLER_FOUND for actual catch handlers.
    // Cleanup-only landing pads (action=0) must CONTINUE_UNWIND so the
    // unwinder keeps searching for a real catch handler.
    if (actions & _UA_SEARCH_PHASE) != 0 {
        if action_val != 0 {
            return _URC_HANDLER_FOUND;
        }
        return _URC_CONTINUE_UNWIND;
    }

    // Phase 2 cleanup: install the landing pad context
    if (actions & _UA_CLEANUP_PHASE) != 0 {
        _Unwind_SetGR(context, 0, exception_object as usize);
        _Unwind_SetGR(context, 1, 0); // type selector = 0 for catch-all
        _Unwind_SetIP(context, landing_pad);
        return _URC_INSTALL_CONTEXT;
    }

    _URC_CONTINUE_UNWIND
}

unsafe fn read_uleb128(p: &mut *const u8) -> usize {
    let mut result = 0;
    let mut shift = 0;
    loop {
        let byte = **p;
        *p = (*p).add(1);
        result |= ((byte & 0x7f) as usize) << shift;
        shift += 7;
        if (byte & 0x80) == 0 {
            break;
        }
    }
    result
}

unsafe fn read_encoded_pointer(p: &mut *const u8, encoding: u8) -> usize {
    if encoding == 0xff { return 0; }
    let format = encoding & 0x0f;
    match format {
        0x01 => read_uleb128(p),      // uleb128
        0x03 => {                     // udata4
            let ptr = *p as *const u32;
            *p = (*p).add(4);
            ptr.read_unaligned() as usize
        },
        0x04 => {                     // udata8
            let ptr = *p as *const u64;
            *p = (*p).add(8);
            ptr.read_unaligned() as usize
        },
        0x0b => {                     // sdata4
            let ptr = *p as *const i32;
            *p = (*p).add(4);
            let val = ptr.read_unaligned();
            if (encoding & 0x70) == 0x10 { // DW_EH_PE_pcrel
                (ptr as usize).wrapping_add(val as usize)
            } else {
                val as usize
            }
        },
        _ => {
            std::process::abort();
        }
    }
}
