use crate::core::vtable::VTable;
use crate::types::string_utils::create_tagged_string;
use super::dynamic_props::{DYNAMIC_PROPERTIES, set_dynamic_property};
use sonic_rs::{JsonValueTrait, JsonContainerTrait};



#[no_mangle]
pub unsafe extern "C" fn __bs_object_spread(target_tagged: u64, source_tagged: u64) -> u64 {
    let target_tag = target_tagged & 0xFFFF_0000_0000_0000;
    if target_tag != 0xFFF6_0000_0000_0000 {
        return target_tagged;
    }
    let target_payload = target_tagged & 0x0000_FFFF_FFFF_FFFF;
    if target_payload == 0 {
        return target_tagged;
    }
    let target_ptr = target_payload as *mut u8;
    
    let source_tag = source_tagged & 0xFFFF_0000_0000_0000;
    if source_tag == 0xFFF6_0000_0000_0000 {
        let src_payload = source_tagged & 0x0000_FFFF_FFFF_FFFF;
        if src_payload != 0 {
            let src_ptr = src_payload as *mut u8;
            
            // 1. Copy class fields if vtable is present in source
            let vtable_ptr = *(src_ptr as *const *const VTable);
            if !vtable_ptr.is_null() {
                let vtable = &*vtable_ptr;
                let fields_count = vtable.fields_count as usize;
                if fields_count > 0 && !vtable.field_names.is_null() {
                    for i in 0..fields_count {
                        let name_ptr = *vtable.field_names.add(i);
                        if !name_ptr.is_null() {
                            let name_cstr = std::ffi::CStr::from_ptr(name_ptr as *const libc::c_char);
                            if let Ok(name_str) = name_cstr.to_str() {
                                let val = *(src_ptr as *const u64).add(1 + i);
                                set_dynamic_property(target_ptr, name_str.to_string(), val);
                            }
                        }
                    }
                }
            }
            
            // 2. Copy dynamic properties of source
            let props: Vec<(String, u64)> = {
                let map = DYNAMIC_PROPERTIES.lock().unwrap();
                if let Some(obj_entry) = map.get(&(src_payload as usize)) {
                    obj_entry.iter().map(|(k, &v)| (k.clone(), v)).collect()
                } else {
                    Vec::new()
                }
            };
            for (k, v) in props {
                if k != "__proto__" {
                    set_dynamic_property(target_ptr, k, v);
                }
            }
        }
    } else if source_tag == 0xFFF8_0000_0000_0000 { // TAG_JSON_TAPE
        let src_payload = source_tagged & 0x0000_FFFF_FFFF_FFFF;
        if src_payload != 0 {
            let ptr = src_payload as *mut std::sync::Mutex<crate::json::tape::JsonTape>;
            let mut tape_obj = (*ptr).lock().unwrap();
            
            if tape_obj.state == crate::json::tape::TapeState::Raw {
                let raw_str = std::str::from_utf8_unchecked(&tape_obj.raw);
                if let Ok(value) = sonic_rs::from_str::<sonic_rs::Value>(raw_str) {
                    tape_obj.tape = Some(value);
                    tape_obj.state = crate::json::tape::TapeState::Indexed;
                }
            }
            
            if let Some(val) = &tape_obj.tape {
                if let Some(obj) = val.as_object() {
                    for (k, v) in obj.iter() {
                        let val_tagged = sonic_value_to_tagged(v);
                        set_dynamic_property(target_ptr, k.to_string(), val_tagged);
                    }
                }
            }
        }
    }
    
    target_tagged
}

unsafe fn sonic_value_to_tagged(field: &sonic_rs::Value) -> u64 {
    if field.is_number() {
        let num = field.as_f64().unwrap_or(0.0);
        crate::gc::box_number(num)
    } else if field.is_str() {
        if let Some(s) = field.as_str() {
            create_tagged_string(s)
        } else {
            0xFFF1_0000_0000_0000
        }
    } else if field.is_boolean() {
        if field.as_bool().unwrap_or(false) {
            0xFFF4_0000_0000_0000
        } else {
            0xFFF3_0000_0000_0000
        }
    } else if field.is_null() {
        0xFFF2_0000_0000_0000
    } else {
        0xFFF1_0000_0000_0000 // undefined
    }
}
