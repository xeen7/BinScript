import os

# --- WeakMap Split ---
with open('rt-stubs/src/objects/builtins/map.rs', 'r') as f:
    map_content = f.read()

weakmap_idx = map_content.find('#[no_mangle]\npub unsafe extern "C" fn __bs_WeakMap_new_0')

weakmap_content = """use crate::gc;

#[no_mangle]
pub unsafe extern "C" fn __bs_WeakMap_new_0() -> u64 {
    let obj = super::map::__bs_Map_new_0();
    let payload = obj & 0x0000_FFFF_FFFF_FFFF;
    let obj_ptr = payload as *mut u8;
    *(obj_ptr as *mut *const crate::core::vtable::VTable) = &crate::WEAKMAP_VTABLE;
    obj
}

#[no_mangle]
pub unsafe extern "C" fn __bs_WeakMap_new_1(iterable: u64) -> u64 {
    let obj = super::map::__bs_Map_new_1(iterable);
    let payload = obj & 0x0000_FFFF_FFFF_FFFF;
    *(payload as *mut *const crate::core::vtable::VTable) = &crate::WEAKMAP_VTABLE;
    obj
}
"""

map_content = map_content[:weakmap_idx]
with open('rt-stubs/src/objects/builtins/map.rs', 'w') as f:
    f.write(map_content.strip() + "\n")
with open('rt-stubs/src/objects/builtins/weakmap.rs', 'w') as f:
    f.write(weakmap_content)

# --- WeakSet Split ---
with open('rt-stubs/src/objects/builtins/set.rs', 'r') as f:
    set_content = f.read()

weakset_idx = set_content.find('#[no_mangle]\npub unsafe extern "C" fn __bs_WeakSet_new_0')

weakset_content = """use crate::gc;

#[no_mangle]
pub unsafe extern "C" fn __bs_WeakSet_new_0() -> u64 {
    let obj = super::set::__bs_Set_new_0();
    let payload = obj & 0x0000_FFFF_FFFF_FFFF;
    let obj_ptr = payload as *mut u8;
    *(obj_ptr as *mut *const crate::core::vtable::VTable) = &crate::WEAKSET_VTABLE;
    obj
}

#[no_mangle]
pub unsafe extern "C" fn __bs_WeakSet_new_1(iterable: u64) -> u64 {
    let obj = super::set::__bs_Set_new_1(iterable);
    let payload = obj & 0x0000_FFFF_FFFF_FFFF;
    *(payload as *mut *const crate::core::vtable::VTable) = &crate::WEAKSET_VTABLE;
    obj
}
"""

set_content = set_content[:weakset_idx]
with open('rt-stubs/src/objects/builtins/set.rs', 'w') as f:
    f.write(set_content.strip() + "\n")
with open('rt-stubs/src/objects/builtins/weakset.rs', 'w') as f:
    f.write(weakset_content)

# Update mod.rs
with open('rt-stubs/src/objects/builtins/mod.rs', 'a') as f:
    f.write('pub mod weakmap;\n')
    f.write('pub mod weakset;\n')
    f.write('pub use weakmap::*;\n')
    f.write('pub use weakset::*;\n')

