use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;



unsafe fn update_size(obj_tagged: u64, size: usize) {
    let payload = obj_tagged & 0x0000_FFFF_FFFF_FFFF;
    crate::objects::dynamic_props::set_dynamic_property(payload as *mut u8, "size".to_string(), crate::circ::box_number(size as f64));
}

pub static SET_DATA: Lazy<Mutex<HashMap<u64, HashMap<u64, ()>>>> = Lazy::new(|| Mutex::new(HashMap::new()));


// --- Set Methods ---

pub unsafe extern "C-unwind" fn set_add(_env: u64, this: u64, key: u64) -> u64 {
    let obj_tagged = this;
    if (obj_tagged & 0xFFFF_0000_0000_0000) != 0xFFF6_0000_0000_0000 && (obj_tagged & 0xFFFF_0000_0000_0000) != 0x7FF6_0000_0000_0000 { libc::abort(); }
    let mut data = SET_DATA.lock().unwrap();
    let obj_ptr = obj_tagged & 0x0000_FFFF_FFFF_FFFF;
    let set = data.entry(obj_ptr).or_insert_with(HashMap::new);
    
    if let Some((old_k, _)) = set.remove_entry(&key) {
        crate::circ::circ_dec_tagged(old_k);
    }
    crate::circ::circ_inc_tagged(key);
    set.insert(key, ());
    
    let size = set.len();
    drop(data);
    update_size(obj_tagged, size);
    crate::circ::circ_inc_tagged(obj_tagged);
    
    obj_tagged
}

pub unsafe extern "C-unwind" fn set_has(_env: u64, this: u64, key: u64) -> u64 {
    let obj_tagged = this;
    if (obj_tagged & 0xFFFF_0000_0000_0000) != 0xFFF6_0000_0000_0000 && (obj_tagged & 0xFFFF_0000_0000_0000) != 0x7FF6_0000_0000_0000 { return crate::circ::box_boolean(false); }
    let data = SET_DATA.lock().unwrap();
    let obj_ptr = obj_tagged & 0x0000_FFFF_FFFF_FFFF;
    let has = data.get(&obj_ptr).map_or(false, |set| set.contains_key(&key));
    
    crate::circ::box_boolean(has)
}

pub unsafe extern "C-unwind" fn set_delete(_env: u64, this: u64, key: u64) -> u64 {
    let obj_tagged = this;
    if (obj_tagged & 0xFFFF_0000_0000_0000) != 0xFFF6_0000_0000_0000 && (obj_tagged & 0xFFFF_0000_0000_0000) != 0x7FF6_0000_0000_0000 { return crate::circ::box_boolean(false); }
    let mut data = SET_DATA.lock().unwrap();
    let mut deleted = false;
    let mut size = 0;
    let obj_ptr = obj_tagged & 0x0000_FFFF_FFFF_FFFF;
    if let Some(set) = data.get_mut(&obj_ptr) {
        if let Some((old_k, _)) = set.remove_entry(&key) {
            crate::circ::circ_dec_tagged(old_k);
            deleted = true;
        }
        size = set.len();
    }
    drop(data);
    update_size(obj_tagged, size);
    crate::circ::box_boolean(deleted)
}

pub unsafe extern "C-unwind" fn set_clear(_env: u64, this: u64) -> u64 {
    let obj_tagged = this;
    if (obj_tagged & 0xFFFF_0000_0000_0000) != 0xFFF6_0000_0000_0000 && (obj_tagged & 0xFFFF_0000_0000_0000) != 0x7FF6_0000_0000_0000 { return 0xFFF1_0000_0000_0000; }
    let mut data = SET_DATA.lock().unwrap();
    let obj_ptr = obj_tagged & 0x0000_FFFF_FFFF_FFFF;
    if let Some(set) = data.get_mut(&obj_ptr) {
        for (k, _) in set.drain() {
            crate::circ::circ_dec_tagged(k);
        }
    }
    drop(data);
    update_size(obj_tagged, 0);
    
    0xFFF1_0000_0000_0000
}

#[no_mangle]
pub unsafe extern "C-unwind" fn set_drop(obj_ptr: *mut u8) {
    let mut data = SET_DATA.lock().unwrap();
    if let Some(set) = data.remove(&(obj_ptr as u64)) {
        // println!("set_drop: set FOUND with {} elements", set.len());
        for (k, _) in set {
            crate::circ::circ_dec_tagged(k);
        }
    } else {
        // println!("set_drop: set NOT FOUND in SET_DATA!");
    }
}


#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_Set_new_0() -> u64 {
    crate::core::vtable::__bs_init_set_prototype();
    let obj = crate::__bs_alloc(&crate::core::vtable::SET_VTABLE, 16);
    let payload = obj & 0x0000_FFFF_FFFF_FFFF;
    crate::objects::dynamic_props::set_dynamic_property(payload as *mut u8, "__proto__".to_string(), crate::core::vtable::SET_PROTOTYPE);
    update_size(obj, 0);
    obj
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_Set_new_1(iterable: u64) -> u64 {
    let obj = __bs_Set_new_0();
    if (iterable & 0xFFFF_0000_0000_0000) != 0xFFF1_0000_0000_0000 {
        let len_f = crate::array::__bs_array_length(iterable);
        let len = f64::from_bits(len_f) as usize;
        for i in 0..len {
            let item = crate::array::__bs_array_get(iterable, crate::circ::box_number(i as f64));
            let ret = set_add(0, obj, item);
            crate::circ::circ_dec_tagged(ret);
            crate::circ::circ_dec_tagged(item);
        }
    }
    obj
}
