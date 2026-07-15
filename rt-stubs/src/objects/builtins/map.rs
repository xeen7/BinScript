use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;



unsafe fn update_size(obj_tagged: u64, size: usize) {
    let payload = obj_tagged & 0x0000_FFFF_FFFF_FFFF;
    crate::objects::dynamic_props::set_dynamic_property(payload as *mut u8, "size".to_string(), crate::circ::box_number(size as f64));
}

pub static MAP_DATA: Lazy<Mutex<HashMap<u64, HashMap<u64, u64>>>> = Lazy::new(|| Mutex::new(HashMap::new()));

// --- Map Methods ---

pub unsafe extern "C-unwind" fn map_set(this: u64, key: u64, val: u64) -> u64 {
    let obj_tagged = this;
    if (obj_tagged & 0xFFFF_0000_0000_0000) != 0xFFF6_0000_0000_0000 && (obj_tagged & 0xFFFF_0000_0000_0000) != 0x7FF6_0000_0000_0000 { libc::abort(); }
    let fmt = b"map_set: map=%lx key=%lx val=%lx\n\0".as_ptr() as *const libc::c_char;
    libc::printf(fmt, obj_tagged, key, val);
    libc::fflush(std::ptr::null_mut());

    let mut data = MAP_DATA.lock().unwrap();
    let obj_ptr = obj_tagged & 0x0000_FFFF_FFFF_FFFF;
    let map = data.entry(obj_ptr).or_insert_with(HashMap::new);
    
    if let Some((old_k, old_v)) = map.remove_entry(&key) {
        crate::circ::circ_dec_tagged(old_k);
        crate::circ::circ_dec_tagged(old_v);
    }
    crate::circ::circ_inc_tagged(key);
    crate::circ::circ_inc_tagged(val);
    map.insert(key, val);
    
    let size = map.len();
    drop(data);
    update_size(obj_tagged, size);
    crate::circ::circ_inc_tagged(obj_tagged);
    
    obj_tagged
}

pub unsafe extern "C-unwind" fn map_get(this: u64, key: u64) -> u64 {
    let obj_tagged = this;
    if (obj_tagged & 0xFFFF_0000_0000_0000) != 0xFFF6_0000_0000_0000 && (obj_tagged & 0xFFFF_0000_0000_0000) != 0x7FF6_0000_0000_0000 { return 0xFFF1_0000_0000_0000; }
    let fmt = b"map_get: map=%lx key=%lx\n\0".as_ptr() as *const libc::c_char;
    libc::printf(fmt, obj_tagged, key);
    libc::fflush(std::ptr::null_mut());

    let data = MAP_DATA.lock().unwrap();
    let obj_ptr = obj_tagged & 0x0000_FFFF_FFFF_FFFF;
    let result = if let Some(map) = data.get(&obj_ptr) {
        if let Some(&val) = map.get(&key) {
            crate::circ::circ_inc_tagged(val);
            val
        } else {
            0xFFF1_0000_0000_0000
        }
    } else {
        0xFFF1_0000_0000_0000 // undefined
    };
    
    result
}

pub unsafe extern "C-unwind" fn map_has(this: u64, key: u64) -> u64 {
    let obj_tagged = this;
    if (obj_tagged & 0xFFFF_0000_0000_0000) != 0xFFF6_0000_0000_0000 && (obj_tagged & 0xFFFF_0000_0000_0000) != 0x7FF6_0000_0000_0000 { return crate::circ::box_boolean(false); }
    let data = MAP_DATA.lock().unwrap();
    let obj_ptr = obj_tagged & 0x0000_FFFF_FFFF_FFFF;
    let has = data.get(&obj_ptr).map_or(false, |map| map.contains_key(&key));
    
    crate::circ::box_boolean(has)
}

pub unsafe extern "C-unwind" fn map_delete(this: u64, key: u64) -> u64 {
    let obj_tagged = this;
    if (obj_tagged & 0xFFFF_0000_0000_0000) != 0xFFF6_0000_0000_0000 && (obj_tagged & 0xFFFF_0000_0000_0000) != 0x7FF6_0000_0000_0000 { return crate::circ::box_boolean(false); }
    let mut data = MAP_DATA.lock().unwrap();
    let mut deleted = false;
    let mut size = 0;
    let obj_ptr = obj_tagged & 0x0000_FFFF_FFFF_FFFF;
    if let Some(map) = data.get_mut(&obj_ptr) {
        if let Some((old_k, old_v)) = map.remove_entry(&key) {
            crate::circ::circ_dec_tagged(old_k);
            crate::circ::circ_dec_tagged(old_v);
            deleted = true;
        }
        size = map.len();
    }
    drop(data);
    update_size(obj_tagged, size);
    
    crate::circ::box_boolean(deleted)
}

pub unsafe extern "C-unwind" fn map_clear(this: u64) -> u64 {
    let obj_tagged = this;
    if (obj_tagged & 0xFFFF_0000_0000_0000) != 0xFFF6_0000_0000_0000 && (obj_tagged & 0xFFFF_0000_0000_0000) != 0x7FF6_0000_0000_0000 { return 0xFFF1_0000_0000_0000; }
    let mut data = MAP_DATA.lock().unwrap();
    let obj_ptr = obj_tagged & 0x0000_FFFF_FFFF_FFFF;
    if let Some(map) = data.get_mut(&obj_ptr) {
        for (k, v) in map.drain() {
            crate::circ::circ_dec_tagged(k);
            crate::circ::circ_dec_tagged(v);
        }
    }
    drop(data);
    update_size(obj_tagged, 0);
    
    0xFFF1_0000_0000_0000 // undefined
}

#[no_mangle]
pub unsafe extern "C-unwind" fn map_drop(obj_ptr: *mut u8) {
    let mut data = MAP_DATA.lock().unwrap();
    if let Some(map) = data.remove(&(obj_ptr as u64)) {
        // println!("map_drop: map FOUND with {} elements", map.len());
        for (k, v) in map {
            crate::circ::circ_dec_tagged(k);
            crate::circ::circ_dec_tagged(v);
        }
    } else {
        // println!("map_drop: map NOT FOUND in MAP_DATA!");
    }
}


// --- Constructors ---

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_Map_new_0() -> u64 {
    crate::core::vtable::__bs_init_map_prototype();
    let obj = crate::__bs_alloc(&crate::core::vtable::MAP_VTABLE, 32);
    let payload = obj & 0x0000_FFFF_FFFF_FFFF;
    crate::objects::dynamic_props::set_dynamic_property(payload as *mut u8, "__proto__".to_string(), crate::core::vtable::MAP_PROTOTYPE);
    update_size(obj, 0);
    obj
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_Map_new_1(iterable: u64) -> u64 {
    let obj = __bs_Map_new_0();
    if (iterable & 0xFFFF_0000_0000_0000) != 0xFFF1_0000_0000_0000 {
        let len_f = crate::array::__bs_array_length(iterable);
        let len = f64::from_bits(len_f) as usize;
        for i in 0..len {
            let pair = crate::array::__bs_array_get(iterable, crate::circ::box_number(i as f64));
            let k = crate::array::__bs_array_get(pair, crate::circ::box_number(0.0));
            let v = crate::array::__bs_array_get(pair, crate::circ::box_number(1.0));
            let ret = map_set(obj, k, v);
            crate::circ::circ_dec_tagged(ret);
            crate::circ::circ_dec_tagged(k);
            crate::circ::circ_dec_tagged(v);
            crate::circ::circ_dec_tagged(pair);
        }
    }
    obj
}
