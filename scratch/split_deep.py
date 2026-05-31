import os
import shutil

# --- Collections Split ---
with open('rt-stubs/src/collections/mod.rs', 'r') as f:
    col_content = f.read()

# Common imports
col_imports = """use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use crate::gc;

unsafe fn create_method(obj_tagged: u64, func_ptr: *const u8) -> u64 {
    let closure_tagged = crate::__bs_alloc_closure(16);
    let closure_ptr = (closure_tagged & 0x0000_FFFF_FFFF_FFFF) as *mut u64;
    *closure_ptr = func_ptr as u64; // offset 0
    *(closure_ptr.add(1)) = obj_tagged; // offset 8
    closure_tagged
}

unsafe fn update_size(obj_tagged: u64, size: usize) {
    let payload = obj_tagged & 0x0000_FFFF_FFFF_FFFF;
    crate::objects::dynamic_props::set_dynamic_property(payload as *mut u8, "size".to_string(), gc::box_number(size as f64));
}
"""

map_start = col_content.find('pub static MAP_DATA')
set_start = col_content.find('pub static SET_DATA')
helpers_end = col_content.find('// --- Map Methods ---')
set_methods = col_content.find('// --- Set Methods ---')
constructors = col_content.find('// --- Constructors ---')
regexp_start = col_content.find('// --- RegExp Methods ---')

# Map parts
map_data = col_content[map_start:set_start]
map_methods = col_content[helpers_end:set_methods]
map_constructors = col_content[constructors:col_content.find('#[no_mangle]\npub unsafe extern "C" fn __bs_Set_new_0', constructors)]
weakmap_constructors = col_content[col_content.find('#[no_mangle]\npub unsafe extern "C" fn __bs_WeakMap_new_0'):col_content.find('#[no_mangle]\npub unsafe extern "C" fn __bs_WeakSet_new_0')]

map_rs = col_imports + "\n" + map_data + "\n" + map_methods + "\n" + map_constructors + "\n" + weakmap_constructors
map_rs = map_rs.replace('crate::set_dynamic_property', 'crate::objects::dynamic_props::set_dynamic_property')
with open('rt-stubs/src/objects/builtins/map.rs', 'w') as f: f.write(map_rs)

# Set parts
set_data = col_content[set_start:col_content.find('// Helpers to create native closures')]
set_methods_code = col_content[set_methods:constructors]
set_constructors = col_content[col_content.find('#[no_mangle]\npub unsafe extern "C" fn __bs_Set_new_0', constructors):col_content.find('// WeakMap and WeakSet')]
weakset_constructors = col_content[col_content.find('#[no_mangle]\npub unsafe extern "C" fn __bs_WeakSet_new_0'):regexp_start]

set_rs = col_imports + "\n" + set_data + "\n" + set_methods_code + "\n" + set_constructors + "\n" + weakset_constructors
set_rs = set_rs.replace('crate::set_dynamic_property', 'crate::objects::dynamic_props::set_dynamic_property')
with open('rt-stubs/src/objects/builtins/set.rs', 'w') as f: f.write(set_rs)

# RegExp parts
regexp_rs = """use crate::gc;

""" + col_content[regexp_start:]
regexp_rs = regexp_rs.replace('crate::get_dynamic_property', 'crate::objects::dynamic_props::get_dynamic_property')
regexp_rs = regexp_rs.replace('crate::set_dynamic_property', 'crate::objects::dynamic_props::set_dynamic_property')
# add create_method since regexp needs it
regexp_rs += """
unsafe fn create_method(obj_tagged: u64, func_ptr: *const u8) -> u64 {
    let closure_tagged = crate::__bs_alloc_closure(16);
    let closure_ptr = (closure_tagged & 0x0000_FFFF_FFFF_FFFF) as *mut u64;
    *closure_ptr = func_ptr as u64; // offset 0
    *(closure_ptr.add(1)) = obj_tagged; // offset 8
    closure_tagged
}
"""
with open('rt-stubs/src/objects/builtins/regexp.rs', 'w') as f: f.write(regexp_rs)

# --- Math Split ---
with open('rt-stubs/src/math/mod.rs', 'r') as f:
    math_content = f.read()

global_start = math_content.find('// --- Global Functions ---')
number_methods = math_content.find('#[no_mangle]\npub unsafe extern "C" fn __bs_number_isInteger')

math_rs = math_content[:global_start]
with open('rt-stubs/src/objects/builtins/math.rs', 'w') as f: f.write(math_rs)

global_rs = """use crate::gc;
const TAG_MASK: u64 = 0xFFFF_0000_0000_0000;
const TAG_STRING: u64 = 0xFFF7_0000_0000_0000;

""" + math_content[global_start:number_methods]
with open('rt-stubs/src/objects/builtins/global.rs', 'w') as f: f.write(global_rs)

number_rs_addition = "\n" + math_content[number_methods:]
with open('rt-stubs/src/objects/builtins/number.rs', 'a') as f: f.write(number_rs_addition)

# --- Exception Split ---
with open('rt-stubs/src/exception/mod.rs', 'r') as f:
    ex_content = f.read()

error_start = ex_content.find('#[no_mangle]\npub unsafe extern "C" fn __bs_Error_new_0')
error_rs = """use crate::gc;
use crate::core::vtable::{VTable, ERROR_VTABLE};
use crate::objects::dynamic_props::set_dynamic_property;
use crate::types::string_utils::create_tagged_string;

""" + ex_content[error_start:]

ex_rs = ex_content[:error_start]

with open('rt-stubs/src/objects/builtins/error.rs', 'w') as f: f.write(error_rs)
with open('rt-stubs/src/exception/mod.rs', 'w') as f: f.write(ex_rs)

# --- Clean up & Update builtins/mod.rs ---
shutil.rmtree('rt-stubs/src/collections')
shutil.rmtree('rt-stubs/src/math')

with open('rt-stubs/src/objects/builtins/mod.rs', 'a') as f:
    f.write('pub mod map;\n')
    f.write('pub mod set;\n')
    f.write('pub mod regexp;\n')
    f.write('pub mod math;\n')
    f.write('pub mod global;\n')
    f.write('pub mod error;\n')
    f.write('pub use map::*;\n')
    f.write('pub use set::*;\n')
    f.write('pub use regexp::*;\n')
    f.write('pub use math::*;\n')
    f.write('pub use global::*;\n')
    f.write('pub use error::*;\n')

print("Split logic done!")
