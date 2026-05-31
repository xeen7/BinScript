pub mod tape;
use crate::gc;

#[no_mangle]
pub unsafe extern "C" fn __bs_json_parse(str_tagged: u64) -> u64 {
    let s = crate::get_c_string_from_tagged(str_tagged);
    crate::json::tape::__bs_json_parse_lazy(s.as_ptr(), s.len() as u32)
}

#[no_mangle]
pub unsafe extern "C" fn __bs_json_stringify(val: u64) -> u64 {
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
                let elem = crate::array::__bs_array_get(val, gc::box_number(i as f64));
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
                let map = crate::DYNAMIC_PROPERTIES.lock().unwrap();
                let payload = val & 0x0000_FFFF_FFFF_FFFF;
                if let Some(obj_entry) = map.get(&(payload as usize)) {
                    let mut first = true;
                    // Sort keys to ensure stable output for tests
                    let mut keys: Vec<&String> = obj_entry.keys().collect();
                    keys.sort();
                    for k in keys {
                        if k == "[[PrimitiveValue]]" || k == "source" || k == "flags" {
                            continue;
                        }
                        let v = obj_entry[k];
                        if (v & 0xFFFF_0000_0000_0000) == 0xFFF1_0000_0000_0000 {
                            continue; // undefined values are skipped
                        }
                        if !first {
                            out.push(',');
                        }
                        first = false;
                        out.push('"');
                        out.push_str(k);
                        out.push_str("\":");
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
