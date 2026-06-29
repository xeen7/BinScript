pub mod tape;

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_json_parse(str_tagged: u64) -> u64 {
    let s = crate::get_c_string_from_tagged(str_tagged);
    crate::json::tape::__bs_json_parse_lazy(s.as_ptr(), s.len() as u32)
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_json_stringify(val: u64) -> u64 {
    let mut out = String::new();
    stringify_value(val, &mut out);
    crate::create_tagged_string(&out)
}

unsafe fn stringify_value(val: u64, out: &mut String) {
    let tag = val & 0xFFFF_0000_0000_0000;
    match tag {
        0xFFF1_0000_0000_0000 => {
            // undefined -> doesn't technically stringify, but JSON.stringify returns undefined. 
            // In arrays it becomes "null". But for simplicity, we'll write "null" here.
            out.push_str("null");
        }
        0xFFF2_0000_0000_0000 => {
            // null
            out.push_str("null");
        }
        0xFFF3_0000_0000_0000 => {
            // false
            out.push_str("false");
        }
        0xFFF4_0000_0000_0000 => {
            // true
            out.push_str("true");
        }
        0xFFF7_0000_0000_0000 => {
            // string
            let s = crate::get_c_string_from_tagged(val);
            // very basic escaping
            out.push('"');
            for c in s.chars() {
                match c {
                    '"' => out.push_str("\\\""),
                    '\\' => out.push_str("\\\\"),
                    '\n' => out.push_str("\\n"),
                    '\r' => out.push_str("\\r"),
                    '\t' => out.push_str("\\t"),
                    _ => out.push(c),
                }
            }
            out.push('"');
        }
        0xFFF8_0000_0000_0000 => {
            // JSON tape (unmaterialized). Materialize it to an object first.
            let materialized = crate::json::tape::__bs_json_tape_materialize(val);
            stringify_value(materialized, out);
        }
        0xFFF9_0000_0000_0000 => {
            // Array
            out.push('[');
            let len_f = crate::array::__bs_array_length(val);
            let len = f64::from_bits(len_f) as usize;
            for i in 0..len {
                if i > 0 {
                    out.push(',');
                }
                let elem = crate::array::__bs_array_get(val, crate::circ::box_number(i as f64));
                stringify_value(elem, out);
            }
            out.push(']');
        }
        _ => {
            if tag < 0xFFF0_0000_0000_0000 {
                // Number
                let num = f64::from_bits(val);
                if num.is_nan() || num.is_infinite() {
                    out.push_str("null");
                } else if num.fract() == 0.0 {
                    out.push_str(&format!("{}", num as i64));
                } else {
                    out.push_str(&format!("{}", num));
                }
            } else if tag == 0xFFF6_0000_0000_0000 {
                // Object
                out.push('{');
                let payload = val & 0x0000_FFFF_FFFF_FFFF;
                let obj_ptr = payload as *mut u8;
                let mut first = true;
                
                // 1. Stringify class fields
                let vtable_ptr = *(obj_ptr as *const *const crate::VTable);
                if !vtable_ptr.is_null() {
                    let vtable = &*vtable_ptr;
                    let fields_count = vtable.fields_count as usize;
                    if fields_count > 0 && !vtable.field_names.is_null() {
                        for i in 0..fields_count {
                            let name_ptr = *vtable.field_names.add(i);
                            if !name_ptr.is_null() {
                                let name_cstr = std::ffi::CStr::from_ptr(name_ptr as *const libc::c_char);
                                if let Ok(name_str) = name_cstr.to_str() {
                                    if !name_str.starts_with("__") && name_str != "[[PrimitiveValue]]" {
                                        let val_ptr = (obj_ptr as *const u64).add(2 + i);
                                        if (*val_ptr & 0xFFFF_0000_0000_0000) != 0xFFF1_0000_0000_0000 {
                                            if !first { out.push(','); }
                                            first = false;
                                            out.push('"'); out.push_str(name_str); out.push_str("\":");
                                            stringify_value(*val_ptr, out);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // 2. Stringify inline properties
                let props_slot = unsafe { obj_ptr.add(8) as *mut *mut std::collections::HashMap<String, u64> };
                if !unsafe { *props_slot }.is_null() {
                    let map = unsafe { &**props_slot };
                    let mut keys: Vec<&String> = map.keys().collect();
                    keys.sort();
                    for k in keys {
                        if k == "[[PrimitiveValue]]" || k == "source" || k == "flags" || k.starts_with("__") {
                            continue;
                        }
                        let v = map[k];
                        if (v & 0xFFFF_0000_0000_0000) == 0xFFF1_0000_0000_0000 {
                            continue;
                        }
                        if !first { out.push(','); }
                        first = false;
                        out.push('"'); out.push_str(k); out.push_str("\":");
                        stringify_value(v, out);
                    }
                }
                out.push('}');
            } else {
                // closures, classes, etc.
                out.push_str("null");
            }
        }
    }
}
