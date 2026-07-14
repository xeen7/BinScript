use std::sync::{Arc, Mutex};
use sonic_rs::JsonValueTrait;

pub const TAG_JSON_TAPE: u64 = 0xFFF8_0000_0000_0000;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TapeState {
    Raw,
    Indexed,
    Parsed,
}

pub struct JsonTape {
    pub raw: Arc<[u8]>,
    pub tape: Option<sonic_rs::Value>,
    pub state: TapeState,
}

#[no_mangle]
pub unsafe extern "C" fn __bs_json_parse_lazy(ptr: *const u8, len: u32) -> u64 {
    let slice = std::slice::from_raw_parts(ptr, len as usize);
    let raw = Arc::from(slice);

    let tape = Box::new(Mutex::new(JsonTape {
        raw,
        tape: None,
        state: TapeState::Raw,
    }));
    let boxed_ptr = Box::into_raw(tape);
    
    // Register to global roots to prevent the JSON tape itself from being collected
    let tagged = (boxed_ptr as u64) | TAG_JSON_TAPE;
    // crate::circ::GLOBAL_ROOTS.lock().unwrap().push(tagged);
    tagged
}

#[no_mangle]
pub unsafe extern "C" fn __bs_json_tape_get(tape_tagged: u64, key_ptr: *const u8, key_len: u32) -> u64 {
    let tag = tape_tagged & 0xFFFF_0000_0000_0000;
    if tag != TAG_JSON_TAPE {
        return 0xFFF1_0000_0000_0000; // undefined
    }

    let ptr = (tape_tagged & 0x0000_FFFF_FFFF_FFFF) as *mut Mutex<JsonTape>;
    let mut tape_obj = (*ptr).lock().unwrap();

    let key_slice = std::slice::from_raw_parts(key_ptr, key_len as usize);
    let key_str = std::str::from_utf8_unchecked(key_slice);

    if tape_obj.state == TapeState::Raw {
        // Parse it lazily
        let raw_str = std::str::from_utf8_unchecked(&tape_obj.raw);
        if let Ok(value) = sonic_rs::from_str::<sonic_rs::Value>(raw_str) {
            tape_obj.tape = Some(value);
            tape_obj.state = TapeState::Indexed;
        } else {
            return 0xFFF1_0000_0000_0000; // Return undefined on parse error
        }
    }

    if let Some(val) = &tape_obj.tape {
        if let Some(field) = val.get(key_str) {
            if field.is_number() {
                let num = field.as_f64().unwrap_or(0.0);
                return crate::circ::box_number(num);
            } else if field.is_str() {
                let s = field.as_str().unwrap_or("");
                return crate::create_tagged_string(s);
            } else if field.is_boolean() {
                if field.as_bool().unwrap() {
                    return 0xFFF4_0000_0000_0000; // true
                } else {
                    return 0xFFF3_0000_0000_0000; // false
                }
            } else if field.is_null() {
                return 0xFFF2_0000_0000_0000; // null
            }
        }
    }

    0xFFF1_0000_0000_0000 // undefined
}

#[no_mangle]
pub unsafe extern "C" fn __bs_json_tape_materialize(tape_tagged: u64) -> u64 {
    // For now, this just returns the tape tag unmodified as we don't have a full object
    // allocator to materialize the tree into a JsObject representation yet.
    tape_tagged
}

#[no_mangle]
pub unsafe extern "C" fn __bs_prop_get(obj_tagged: u64, prop_str: *const u8, len: u32) -> u64 {
    let tag = obj_tagged & 0xFFFF_0000_0000_0000;
    
    let mut c_prop = vec![0u8; len as usize + 1];
    c_prop[..len as usize].copy_from_slice(std::slice::from_raw_parts(prop_str, len as usize));
    libc::printf(b"__bs_prop_get: obj=%p tag=%llx prop=%s\n\0".as_ptr() as *const i8, obj_tagged, tag, c_prop.as_ptr() as *const i8);
    
    if tag == TAG_JSON_TAPE {
        return __bs_json_tape_get(obj_tagged, prop_str, len);
    }
    if tag == 0xFFFB_0000_0000_0000 || tag == 0x7FFB_0000_0000_0000 || tag == 0x7FFA_0000_0000_0000 {
        let prop_slice = std::slice::from_raw_parts(prop_str, len as usize);
        if prop_slice == b"length" {
            return crate::array::__bs_array_length(obj_tagged);
        }
        if let Ok(s) = std::str::from_utf8(prop_slice) {
            if let Ok(idx) = s.parse::<f64>() {
                let idx_boxed = crate::circ::box_number(idx);
                return crate::array::__bs_array_get(obj_tagged, idx_boxed);
            }
            // Fall through to check dynamic properties for arrays
            let payload = obj_tagged & 0x0000_FFFF_FFFF_FFFF;
            if let Some(val) = crate::get_dynamic_property(payload as *mut u8, s) {
                crate::circ::circ_inc_tagged(val);
                return val;
            }
        }
        libc::printf(b"__bs_prop_get returning undefined (array prop)\n\0".as_ptr() as *const i8);
        return 0xFFF1_0000_0000_0000; // undefined
    }
    if tag == 0xFFF7_0000_0000_0000 || tag == 0x7FF7_0000_0000_0000 {
        let prop_slice = std::slice::from_raw_parts(prop_str, len as usize);
        if prop_slice == b"length" {
            return crate::string::__bs_string_length(obj_tagged);
        }
        libc::printf(b"__bs_prop_get returning undefined (string prop)\n\0".as_ptr() as *const i8);
        return 0xFFF1_0000_0000_0000; // undefined
    }
    if tag != 0xFFF6_0000_0000_0000 && tag != 0xFFFC_0000_0000_0000 && tag != 0xFFFE_0000_0000_0000 && tag != 0x7FF6_0000_0000_0000 {
        libc::printf(b"__bs_prop_get returning undefined (invalid tag)\n\0".as_ptr() as *const i8);
        return 0xFFF1_0000_0000_0000; // undefined
    }
    
    let payload = obj_tagged & 0x0000_FFFF_FFFF_FFFF;
    if payload == 0 {
        libc::printf(b"__bs_prop_get returning undefined (payload 0)\n\0".as_ptr() as *const i8);
        return 0xFFF1_0000_0000_0000; // undefined
    }

    let prop_slice = std::slice::from_raw_parts(prop_str, len as usize);
    let obj_ptr = payload as *mut u8;
    
    // 1. Check class fields if vtable is present
    let vtable_ptr = *(obj_ptr as *const *const crate::VTable);
    if !vtable_ptr.is_null() {
        let vtable = &*vtable_ptr;
        let fields_count = vtable.fields_count as usize;
        if fields_count > 0 && !vtable.field_names.is_null() {
            for i in 0..fields_count {
                let name_ptr = *vtable.field_names.add(i);
                let name_len = libc::strlen(name_ptr as *const i8) as usize;
                let name_slice = std::slice::from_raw_parts(name_ptr as *const u8, name_len);
                
                // DEBUG PRINT
                let mut c_prop = vec![0u8; prop_slice.len() + 1];
                c_prop[..prop_slice.len()].copy_from_slice(prop_slice);
                
                libc::printf(
                    b"__bs_prop_get: comparing class field '%s' with requested '%s'\n\0".as_ptr() as *const i8,
                    name_ptr as *const i8,
                    c_prop.as_ptr() as *const i8
                );
                if name_slice == prop_slice {
                    let field_slot = (obj_ptr as *mut u64).add(2 + i);
                    let val = *field_slot;
                    crate::circ::circ_inc_tagged(val);
                    return val;
                }
            }
        }
    }

    // 2. Check inline and dynamic properties
    if let Ok(prop_name) = std::str::from_utf8(prop_slice) {
        if true {
            let props_slot = obj_ptr.add(8) as *mut *mut std::collections::HashMap<String, u64>;
            if let Some(val) = crate::objects::dynamic_props::get_inline_property(props_slot, prop_name) {
                crate::circ::circ_inc_tagged(val);
                return val;
            }
        }
        if let Some(val) = crate::get_dynamic_property(obj_ptr, prop_name) {
            crate::circ::circ_inc_tagged(val);
            return val;
        }
    }

    // 3. Walk the __proto__ chain
    let mut current_obj = obj_tagged;
    loop {
        let current_payload = current_obj & 0x0000_FFFF_FFFF_FFFF;
        if current_payload == 0 {
            break;
        }
        let current_ptr = current_payload as *mut u8;
        if let Some(proto) = crate::get_dynamic_property(current_ptr, "__proto__") {
            let proto_tag = proto & 0xFFFF_0000_0000_0000;
            if proto_tag == 0xFFF6_0000_0000_0000 {
                let proto_payload = proto & 0x0000_FFFF_FFFF_FFFF;
                let proto_ptr = proto_payload as *mut u8;
                // Check class fields of the prototype first
                let proto_vtable_ptr = *(proto_ptr as *const *const crate::VTable);
                if !proto_vtable_ptr.is_null() {
                    let vtable = &*proto_vtable_ptr;
                    let fields_count = vtable.fields_count as usize;
                    if fields_count > 0 && !vtable.field_names.is_null() {
                        for i in 0..fields_count {
                            let name_ptr = *vtable.field_names.add(i);
                            if !name_ptr.is_null() {
                                let name_cstr = std::ffi::CStr::from_ptr(name_ptr as *const libc::c_char);
                                let name_bytes = name_cstr.to_bytes();
                                if name_bytes == prop_slice {
                                    let slot_ptr = (proto_ptr as *const u64).add(1 + i);
                                    let val = *slot_ptr;
                                    crate::circ::circ_inc_tagged(val);
                                    return val;
                                }
                            }
                        }
                    }
                }
                // Check dynamic properties of the prototype
                if let Ok(prop_name) = std::str::from_utf8(prop_slice) {
                    let props_slot = proto_ptr.add(8) as *mut *mut std::collections::HashMap<String, u64>;
                    if let Some(val) = crate::objects::dynamic_props::get_inline_property(props_slot, prop_name) {
                        crate::circ::circ_inc_tagged(val);
                        return val;
                    }
                    if let Some(val) = crate::get_dynamic_property(proto_ptr, prop_name) {
                        crate::circ::circ_inc_tagged(val);
                        return val;
                    }
                }
                current_obj = proto;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    0xFFF1_0000_0000_0000 // undefined
}

#[no_mangle]
pub unsafe extern "C" fn __bs_prop_set(obj_tagged: u64, prop_str: *const u8, len: u32, val_tagged: u64) {
    let tag = obj_tagged & 0xFFFF_0000_0000_0000;
    
    let mut c_prop = vec![0u8; len as usize + 1];
    c_prop[..len as usize].copy_from_slice(std::slice::from_raw_parts(prop_str, len as usize));
    libc::printf(b"__bs_prop_set: obj=%p tag=%llx prop=%s val=%llx\n\0".as_ptr() as *const i8, obj_tagged, tag, c_prop.as_ptr() as *const i8, val_tagged);
    
    if tag == TAG_JSON_TAPE {
        return;
    }
    if tag == 0xFFFB_0000_0000_0000 || tag == 0x7FFB_0000_0000_0000 || tag == 0x7FFA_0000_0000_0000 {
        let prop_slice = std::slice::from_raw_parts(prop_str, len as usize);
        if let Ok(s) = std::str::from_utf8(prop_slice) {
            if let Ok(idx) = s.parse::<f64>() {
                let idx_boxed = crate::circ::box_number(idx);
                crate::array::__bs_array_set(obj_tagged, idx_boxed, val_tagged);
            }
        }
        return;
    }
    if tag != 0xFFF6_0000_0000_0000 && tag != 0xFFFC_0000_0000_0000 && tag != 0xFFFE_0000_0000_0000 && tag != 0x7FF6_0000_0000_0000 {
        return;
    }
    
    
    let payload = obj_tagged & 0x0000_FFFF_FFFF_FFFF;
    if payload == 0 {
        return;
    }

    let prop_slice = std::slice::from_raw_parts(prop_str, len as usize);
    let obj_ptr = payload as *mut u8;
    
    // 1. Check class fields if vtable is present
    let vtable_ptr = *(obj_ptr as *const *const crate::VTable);
    if !vtable_ptr.is_null() {
        let vtable = &*vtable_ptr;
        let fields_count = vtable.fields_count as usize;
        if fields_count > 0 && !vtable.field_names.is_null() {
            for i in 0..fields_count {
                let name_ptr = *vtable.field_names.add(i);
                if !name_ptr.is_null() {
                    let name_cstr = std::ffi::CStr::from_ptr(name_ptr as *const libc::c_char);
                    let name_bytes = name_cstr.to_bytes();
                    if name_bytes == prop_slice {
                        let slot_mut_ptr = (obj_ptr as *mut u64).add(2 + i);
                        let old_val = *slot_mut_ptr;
                        crate::circ::circ_inc_tagged(val_tagged);
                        if old_val != 0 {
                            crate::circ::circ_dec_tagged(old_val);
                        }
                        *slot_mut_ptr = val_tagged;
                        return;
                    }
                }
            }
        }
    }

    // 2. Otherwise set as inline property
    if let Ok(prop_name) = std::str::from_utf8(prop_slice) {
        if true {
            let props_slot = obj_ptr.add(8) as *mut *mut std::collections::HashMap<String, u64>;
            crate::objects::dynamic_props::set_inline_property(props_slot, prop_name.to_string(), val_tagged);
        } else {
            crate::set_dynamic_property(obj_ptr, prop_name.to_string(), val_tagged);
        }
    }
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_prop_set_moved(obj_tagged: u64, prop_ptr: *const u8, prop_len: u32, val_tagged: u64) -> u64 {
    let tag = obj_tagged & 0xFFFF_0000_0000_0000;
    
    let mut c_prop = vec![0u8; prop_len as usize + 1];
    c_prop[..prop_len as usize].copy_from_slice(std::slice::from_raw_parts(prop_ptr, prop_len as usize));
    libc::printf(b"__bs_prop_set_moved: obj=%p tag=%llx prop=%s val=%llx\n\0".as_ptr() as *const i8, obj_tagged, tag, c_prop.as_ptr() as *const i8, val_tagged);
    
    if tag != 0xFFF6_0000_0000_0000 && tag != 0xFFF9_0000_0000_0000 && tag != 0xFFFA_0000_0000_0000 && tag != 0xFFFC_0000_0000_0000 && tag != 0xFFFE_0000_0000_0000 {
        let prop_slice = std::slice::from_raw_parts(prop_ptr, prop_len as usize);
        if let Ok(s) = std::str::from_utf8(prop_slice) {
            if let Ok(idx) = s.parse::<f64>() {
                let idx_boxed = crate::circ::box_number(idx);
                crate::array::__bs_array_set(obj_tagged, idx_boxed, val_tagged);
            }
        }
        return val_tagged;
    }
    
    let payload = obj_tagged & 0x0000_FFFF_FFFF_FFFF;
    if payload == 0 {
        return val_tagged;
    }

    let prop_slice = std::slice::from_raw_parts(prop_ptr, prop_len as usize);
    let obj_ptr = payload as *mut u8;
    
    // 1. Check class fields if vtable is present (only objects)
    if tag == 0xFFF6_0000_0000_0000 || tag == 0xFFFC_0000_0000_0000 {
        let vtable_ptr_ptr = obj_ptr as *const *const crate::VTable;
        let vtable_ptr = *vtable_ptr_ptr;
        if !vtable_ptr.is_null() {
            let vtable = &*vtable_ptr;
            let fields_count = vtable.fields_count as usize;
            if fields_count > 0 && !vtable.field_names.is_null() {
                for i in 0..fields_count {
                    let name_ptr = *vtable.field_names.add(i);
                    if !name_ptr.is_null() {
                        let name_cstr = std::ffi::CStr::from_ptr(name_ptr as *const libc::c_char);
                        let name_bytes = name_cstr.to_bytes();
                        if name_bytes == prop_slice {
                            let slot_mut_ptr = (obj_ptr as *mut u64).add(2 + i);
                            let old_val = *slot_mut_ptr;
                            // Move semantics: do NOT call circ_inc_tagged(val_tagged)
                            if old_val != 0 {
                                crate::circ::circ_dec_tagged(old_val);
                            }
                            *slot_mut_ptr = val_tagged;
                            return val_tagged;
                        }
                    }
                }
            }
        }
    }

    // 2. Otherwise set as inline property
    if let Ok(prop_name) = std::str::from_utf8(prop_slice) {
        if tag == 0xFFF6_0000_0000_0000 || tag == 0xFFFC_0000_0000_0000 {
            let props_slot = obj_ptr.add(8) as *mut *mut std::collections::HashMap<String, u64>;
            crate::objects::dynamic_props::set_inline_property_moved(props_slot, prop_name.to_string(), val_tagged);
        } else {
            crate::objects::dynamic_props::set_dynamic_property_moved(obj_ptr, prop_name.to_string(), val_tagged);
        }
    }
    val_tagged
}
