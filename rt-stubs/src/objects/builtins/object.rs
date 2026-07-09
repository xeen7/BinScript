use crate::core::vtable::{VTable, OBJECT_VTABLE};
use crate::core::alloc::__bs_alloc;
use crate::types::coercion::{__bs_Object, __bs_String};
use crate::types::string_utils::{get_c_string_from_tagged, create_tagged_string};
use crate::objects::dynamic_props::{DYNAMIC_PROPERTIES, set_dynamic_property, get_dynamic_property};

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_new_object() -> u64 {
    let obj = __bs_alloc(&OBJECT_VTABLE, 16);
    obj
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_Object_new(val: u64) -> u64 {
    __bs_Object(val)
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_Object_new_0() -> u64 {
    __bs_new_object()
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_Object_new_1(val: u64) -> u64 {
    __bs_Object(val)
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_object_keys(obj: u64) -> u64 {
    let tag = obj & 0xFFFF_0000_0000_0000;
    let array = crate::array::__bs_array_new();
    if tag == 0xFFF6_0000_0000_0000 || tag == 0xFFFC_0000_0000_0000 || tag == 0xFFFE_0000_0000_0000 {
        let payload = obj & 0x0000_FFFF_FFFF_FFFF;
        if payload != 0 {
            let obj_ptr = payload as *mut u8;
            let vtable_ptr = *(obj_ptr as *const *const VTable);
            if !vtable_ptr.is_null() {
                let vtable = &*vtable_ptr;
                let fields_count = vtable.fields_count as usize;
                if fields_count > 0 && !vtable.field_names.is_null() {
                    for i in 0..fields_count {
                        let name_ptr = *vtable.field_names.add(i);
                        if !name_ptr.is_null() {
                            let name_cstr = std::ffi::CStr::from_ptr(name_ptr as *const libc::c_char);
                            let name_str = name_cstr.to_str().unwrap_or("");
                            if !name_str.starts_with("__") && name_str != "[[PrimitiveValue]]" {
                                let k_tagged = create_tagged_string(name_str);
                                crate::array::__bs_array_push(array, k_tagged);
                            }
                        }
                    }
                }
            }
            let props_slot = unsafe { obj_ptr.add(8) as *mut *mut std::collections::HashMap<String, u64> };
            if !unsafe { *props_slot }.is_null() {
                let map = unsafe { &**props_slot };
                let mut dkeys: Vec<_> = map.keys().cloned().collect();
                dkeys.sort();
                for k in dkeys {
                    if k != "[[PrimitiveValue]]" && !k.starts_with("__") {
                        let k_tagged = create_tagged_string(&k);
                        crate::array::__bs_array_push(array, k_tagged);
                    }
                }
            }
        }
    }
    // Return as TAG_OWNED_ARRAY so the compiler drops it properly
    (array & 0x0000_FFFF_FFFF_FFFF) | 0x7FFB_0000_0000_0000
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_object_rest(obj: u64, excluded_arr: u64) -> u64 {
    let new_obj = __bs_new_object();
    let new_payload = new_obj & 0x0000_FFFF_FFFF_FFFF;
    
    let mut excluded_keys = Vec::new();
    let arr = crate::array::untag_array(excluded_arr);
    if !arr.is_null() {
        let len = (*arr).length as usize;
        for i in 0..len {
            let key_val = *(*arr).data.add(i);
            if (key_val & 0xFFFF_0000_0000_0000) == 0xFFF7_0000_0000_0000 {
                let key_str = get_c_string_from_tagged(key_val);
                excluded_keys.push(key_str.to_string());
            }
        }
    }

    let mut props_to_copy = Vec::new();

    let tag = obj & 0xFFFF_0000_0000_0000;
    if tag == 0xFFF6_0000_0000_0000 || tag == 0xFFFC_0000_0000_0000 || tag == 0xFFFE_0000_0000_0000 {
        let payload = obj & 0x0000_FFFF_FFFF_FFFF;
        if payload != 0 {
            let obj_ptr = payload as *mut u8;
            
            let vtable_ptr = *(obj_ptr as *const *const VTable);
            if !vtable_ptr.is_null() {
                let vtable = &*vtable_ptr;
                let fields_count = vtable.fields_count as usize;
                if fields_count > 0 && !vtable.field_names.is_null() {
                    for i in 0..fields_count {
                        let name_ptr = *vtable.field_names.add(i);
                        if !name_ptr.is_null() {
                            let name_cstr = std::ffi::CStr::from_ptr(name_ptr as *const libc::c_char);
                            let name_str = name_cstr.to_str().unwrap_or("");
                            if !name_str.starts_with("__") && name_str != "[[PrimitiveValue]]" {
                                if !excluded_keys.contains(&name_str.to_string()) {
                                    let field_slot = (obj_ptr as *const u64).add(2 + i);
                                    let val = *field_slot;
                                    props_to_copy.push((name_str.to_string(), val));
                                }
                            }
                        }
                    }
                }
            }
            
            {
                let props_slot = unsafe { obj_ptr.add(8) as *mut *mut std::collections::HashMap<String, u64> };
                if !unsafe { *props_slot }.is_null() {
                    let map = unsafe { &**props_slot };
                    for (k, &val) in map.iter() {
                        if k != "[[PrimitiveValue]]" && !k.starts_with("__") {
                            if !excluded_keys.contains(k) {
                                props_to_copy.push((k.clone(), val));
                            }
                        }
                    }
                }
            }
        }
    }

    for (k, val) in props_to_copy {
        let tgt_props_slot = unsafe { (new_payload as *mut u8).add(8) as *mut *mut std::collections::HashMap<String, u64> };
        crate::objects::dynamic_props::set_inline_property(tgt_props_slot, k, val);
    }
    
    new_obj
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_object_values(obj: u64) -> u64 {
    let tag = obj & 0xFFFF_0000_0000_0000;
    let array = crate::array::__bs_array_new();
    if tag == 0xFFF6_0000_0000_0000 || tag == 0xFFFC_0000_0000_0000 || tag == 0xFFFE_0000_0000_0000 {
        let payload = obj & 0x0000_FFFF_FFFF_FFFF;
        if payload != 0 {
            let obj_ptr = payload as *mut u8;
            let vtable_ptr = *(obj_ptr as *const *const VTable);
            if !vtable_ptr.is_null() {
                let vtable = &*vtable_ptr;
                let fields_count = vtable.fields_count as usize;
                if fields_count > 0 && !vtable.field_names.is_null() {
                    for i in 0..fields_count {
                        let name_ptr = *vtable.field_names.add(i);
                        if !name_ptr.is_null() {
                            let name_cstr = std::ffi::CStr::from_ptr(name_ptr as *const libc::c_char);
                            let name_str = name_cstr.to_str().unwrap_or("");
                            if !name_str.starts_with("__") && name_str != "[[PrimitiveValue]]" {
                                let val_ptr = (obj_ptr as *const u64).add(1 + i);
                                crate::array::__bs_array_push(array, *val_ptr);
                            }
                        }
                    }
                }
            }
            let props_slot = unsafe { obj_ptr.add(8) as *mut *mut std::collections::HashMap<String, u64> };
            if !unsafe { *props_slot }.is_null() {
                let map = unsafe { &**props_slot };
                let mut dkeys: Vec<_> = map.keys().cloned().collect();
                dkeys.sort();
                for k in dkeys {
                    if k != "[[PrimitiveValue]]" && !k.starts_with("__") {
                        let v = *map.get(&k).unwrap();
                        crate::array::__bs_array_push(array, v);
                    }
                }
            }
        }
    }
    // Return as TAG_OWNED_ARRAY so the compiler drops it properly
    (array & 0x0000_FFFF_FFFF_FFFF) | 0x7FFB_0000_0000_0000
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_object_entries(obj: u64) -> u64 {
    let tag = obj & 0xFFFF_0000_0000_0000;
    let array = crate::array::__bs_array_new();
    if tag == 0xFFF6_0000_0000_0000 || tag == 0xFFFC_0000_0000_0000 || tag == 0xFFFE_0000_0000_0000 {
        let payload = obj & 0x0000_FFFF_FFFF_FFFF;
        if payload != 0 {
            let obj_ptr = payload as *mut u8;
            let push_entry = |arr, k: &str, v: u64| {
                let entry = crate::array::__bs_array_new();
                crate::array::__bs_array_push(entry, create_tagged_string(k));
                crate::array::__bs_array_push(entry, v);
                crate::array::__bs_array_push(arr, entry);
                crate::circ::circ_dec_tagged(entry);
            };
            let vtable_ptr = *(obj_ptr as *const *const VTable);
            if !vtable_ptr.is_null() {
                let vtable = &*vtable_ptr;
                let fields_count = vtable.fields_count as usize;
                if fields_count > 0 && !vtable.field_names.is_null() {
                    for i in 0..fields_count {
                        let name_ptr = *vtable.field_names.add(i);
                        if !name_ptr.is_null() {
                            let name_cstr = std::ffi::CStr::from_ptr(name_ptr as *const libc::c_char);
                            let name_str = name_cstr.to_str().unwrap_or("");
                            if !name_str.starts_with("__") && name_str != "[[PrimitiveValue]]" {
                                let val_ptr = (obj_ptr as *const u64).add(1 + i);
                                push_entry(array, name_str, *val_ptr);
                            }
                        }
                    }
                }
            }
            let props_slot = unsafe { obj_ptr.add(8) as *mut *mut std::collections::HashMap<String, u64> };
            if !unsafe { *props_slot }.is_null() {
                let map = unsafe { &**props_slot };
                let mut dkeys: Vec<_> = map.keys().cloned().collect();
                dkeys.sort();
                for k in dkeys {
                    if k != "[[PrimitiveValue]]" && !k.starts_with("__") {
                        let v = *map.get(&k).unwrap();
                        push_entry(array, &k, v);
                    }
                }
            }
        }
    }
    // Return as TAG_OWNED_ARRAY so the compiler drops it properly
    (array & 0x0000_FFFF_FFFF_FFFF) | 0x7FFB_0000_0000_0000
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_object_assign(target: u64, source: u64) -> u64 {
    let target_tag = target & 0xFFFF_0000_0000_0000;
    let source_tag = source & 0xFFFF_0000_0000_0000;
    if target_tag == 0xFFF6_0000_0000_0000 {
        let target_payload = target & 0x0000_FFFF_FFFF_FFFF;
        let target_ptr = target_payload as *mut u8;
        if source_tag == 0xFFF6_0000_0000_0000 {
            let source_payload = source & 0x0000_FFFF_FFFF_FFFF;
            let source_ptr = source_payload as *mut u8;
            let vtable_ptr = *(source_ptr as *const *const VTable);
            if !vtable_ptr.is_null() {
                let vtable = &*vtable_ptr;
                let fields_count = vtable.fields_count as usize;
                if fields_count > 0 && !vtable.field_names.is_null() {
                    for i in 0..fields_count {
                        let name_ptr = *vtable.field_names.add(i);
                        if !name_ptr.is_null() {
                            let name_cstr = std::ffi::CStr::from_ptr(name_ptr as *const libc::c_char);
                            let name_str = name_cstr.to_str().unwrap_or("");
                            if !name_str.starts_with("__") && name_str != "[[PrimitiveValue]]" {
                                let val_ptr = (source_ptr as *const u64).add(2 + i);
                                crate::json::tape::__bs_prop_set(target, name_ptr, name_str.len() as u32, *val_ptr);
                            }
                        }
                    }
                }
            }
            let props_slot = unsafe { source_ptr.add(8) as *mut *mut std::collections::HashMap<String, u64> };
            if !unsafe { *props_slot }.is_null() {
                let map = unsafe { &**props_slot };
                for (k, &val) in map.iter() {
                    if k != "[[PrimitiveValue]]" && !k.starts_with("__") {
                        let k_cstr = std::ffi::CString::new(k.clone()).unwrap();
                        crate::json::tape::__bs_prop_set(target, k_cstr.as_ptr() as *const u8, k.len() as u32, val);
                    }
                }
            }
        }
    }
    crate::circ::circ_inc_tagged(target);
    target
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_object_create(proto: u64) -> u64 {
    let obj = __bs_new_object();
    let payload = obj & 0x0000_FFFF_FFFF_FFFF;
    set_dynamic_property(payload as *mut u8, "__proto__".to_string(), proto);
    // Return as TAG_OWNED_OBJECT so the compiler drops it properly
    payload | 0xFFFC_0000_0000_0000
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_object_getPrototypeOf(obj: u64) -> u64 {
    let tag = obj & 0xFFFF_0000_0000_0000;
    if tag == 0xFFF6_0000_0000_0000 || tag == 0xFFFC_0000_0000_0000 || tag == 0xFFFE_0000_0000_0000 {
        let payload = obj & 0x0000_FFFF_FFFF_FFFF;
        let obj_ptr = payload as *mut u8;
        if let Some(proto) = get_dynamic_property(obj_ptr, "__proto__") {
            crate::circ::circ_inc_tagged(proto);
            return proto;
        }
    }
    0xFFF5_0000_0000_0000 // null
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_object_fromEntries(pairs: u64) -> u64 {
    let obj = __bs_alloc(&OBJECT_VTABLE, 16);
    let payload = obj & 0x0000_FFFF_FFFF_FFFF;
    
    let tag = pairs & 0xFFFF_0000_0000_0000;
    if tag == 0xFFFB_0000_0000_0000 {
        let arr = crate::array::untag_array(pairs);
        if !arr.is_null() {
            let len = (*arr).length;
            for i in 0..len {
                let entry_tagged = *(*arr).data.add(i as usize);
                let entry_tag = entry_tagged & 0xFFFF_0000_0000_0000;
                if entry_tag == 0xFFFB_0000_0000_0000 {
                    let entry_arr = crate::array::untag_array(entry_tagged);
                    if !entry_arr.is_null() && (*entry_arr).length >= 2 {
                        let key_tagged = *(*entry_arr).data.add(0);
                        let val_tagged = *(*entry_arr).data.add(1);
                        
                        let key_string_tagged = __bs_String(key_tagged);
                        let key_str = get_c_string_from_tagged(key_string_tagged).to_string();
                        
                        set_dynamic_property(payload as *mut u8, key_str, val_tagged);
                    }
                }
            }
        }
    }
    obj
}

static mut GLOBAL_THIS_OBJ: u64 = 0;

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_get_globalThis() -> u64 {
    if GLOBAL_THIS_OBJ == 0 {
        GLOBAL_THIS_OBJ = __bs_alloc(&OBJECT_VTABLE, 16);
    }
    GLOBAL_THIS_OBJ
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_cleanup_global_this() {
    if GLOBAL_THIS_OBJ != 0 {
        crate::circ::circ_dec_tagged(GLOBAL_THIS_OBJ);
        GLOBAL_THIS_OBJ = 0;
    }
}