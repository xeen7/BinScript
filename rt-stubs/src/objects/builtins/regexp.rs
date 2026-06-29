
// --- RegExp Methods ---

fn build_regex(pattern_tagged: u64, flags_tagged: u64) -> Option<regex::Regex> {
    let pattern_str = unsafe { crate::get_c_string_from_tagged(pattern_tagged) };
    
    let mut flags_str = String::new();
    if (flags_tagged & 0xFFFF_0000_0000_0000) == 0xFFF7_0000_0000_0000 {
        flags_str = unsafe { crate::get_c_string_from_tagged(flags_tagged).to_string() };
    }
    
    let mut rust_pattern = pattern_str.to_string();
    if flags_str.contains('i') {
        rust_pattern = format!("(?i){}", rust_pattern);
    }
    
    regex::Regex::new(&rust_pattern).ok()
}

pub unsafe extern "C-unwind" fn regexp_test(env: u64, text_tagged: u64) -> u64 {
    let closure_ptr = (env & 0x0000_FFFF_FFFF_FFFF) as *const u64;
    let obj_tagged = *(closure_ptr.add(3));
    let payload = obj_tagged & 0x0000_FFFF_FFFF_FFFF;
    
    let pattern_tagged = crate::objects::dynamic_props::get_dynamic_property(payload as *mut u8, "source").unwrap_or(0xFFF1_0000_0000_0000);
    let flags_tagged = crate::objects::dynamic_props::get_dynamic_property(payload as *mut u8, "flags").unwrap_or(0xFFF1_0000_0000_0000);
    
    // println!("regexp obj_tagged: {:x}, env: {:x}, text: {:x}, pattern: {:x}", obj_tagged, env, text_tagged, pattern_tagged);
    let text_str = crate::types::string_utils::get_c_string_from_tagged(text_tagged);
    let mut matched = false;
    
    if let Some(re) = build_regex(pattern_tagged, flags_tagged) {
        matched = re.is_match(&text_str);
    }
    crate::circ::box_boolean(matched)
}

pub unsafe extern "C-unwind" fn regexp_exec(env: u64, text_tagged: u64) -> u64 {
    let closure_ptr = (env & 0x0000_FFFF_FFFF_FFFF) as *const u64;
    let obj_tagged = *(closure_ptr.add(3));
    let payload = obj_tagged & 0x0000_FFFF_FFFF_FFFF;
    
    let pattern_tagged = crate::objects::dynamic_props::get_dynamic_property(payload as *mut u8, "source").unwrap_or(0xFFF1_0000_0000_0000);
    let flags_tagged = crate::objects::dynamic_props::get_dynamic_property(payload as *mut u8, "flags").unwrap_or(0xFFF1_0000_0000_0000);
    
    // println!("regexp obj_tagged: {:x}, env: {:x}, text: {:x}, pattern: {:x}", obj_tagged, env, text_tagged, pattern_tagged);
    let text_str = crate::types::string_utils::get_c_string_from_tagged(text_tagged);
    
    if let Some(re) = build_regex(pattern_tagged, flags_tagged) {
        if let Some(caps) = re.captures(&text_str) {
            // Return an array with match at [0], groups at [1..], and index property
            let arr_tagged = crate::array::__bs_array_new();
            for (_i, cap) in caps.iter().enumerate() {
                if let Some(m) = cap {
                    let s_tagged = crate::create_tagged_string(m.as_str());
                    crate::array::__bs_array_push(arr_tagged, s_tagged);
                } else {
                    crate::array::__bs_array_push(arr_tagged, 0xFFF1_0000_0000_0000); // undefined
                }
            }
            
            // Set index property
            if let Some(m) = caps.get(0) {
                let index = m.start() as f64;
                let arr_payload = arr_tagged & 0x0000_FFFF_FFFF_FFFF;
                crate::objects::dynamic_props::set_dynamic_property(arr_payload as *mut u8, "index".to_string(), crate::circ::box_number(index));
            }
            
            return arr_tagged;
        }
    }
    0xFFF2_0000_0000_0000 // null
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_RegExp_new(pattern_tagged: u64, flags_tagged: u64) -> u64 {
    let obj = crate::__bs_alloc(&crate::REGEXP_VTABLE, 16);
    let payload = obj & 0x0000_FFFF_FFFF_FFFF;
    crate::objects::dynamic_props::set_dynamic_property(payload as *mut u8, "source".to_string(), pattern_tagged);
    crate::objects::dynamic_props::set_dynamic_property(payload as *mut u8, "flags".to_string(), flags_tagged);
    
    crate::objects::dynamic_props::set_dynamic_property_moved(payload as *mut u8, "test".to_string(), crate::circ::create_builtin_method(obj, regexp_test as *const u8));
    crate::objects::dynamic_props::set_dynamic_property_moved(payload as *mut u8, "exec".to_string(), crate::circ::create_builtin_method(obj, regexp_exec as *const u8));
    obj
}


