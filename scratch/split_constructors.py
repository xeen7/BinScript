import os

with open('rt-stubs/src/objects/constructors.rs', 'r') as f:
    content = f.read()

# We need to parse out the different sections.
# Given the size, maybe it's easier to just use regex or split by known functions.
# Let's write a python script that creates the new files.

os.makedirs('rt-stubs/src/objects/builtins', exist_ok=True)

# Common imports
COMMON_IMPORTS = """use crate::gc;
use crate::core::vtable::{VTable, OBJECT_VTABLE, STRING_VTABLE, NUMBER_VTABLE, BOOLEAN_VTABLE, DATE_VTABLE};
use crate::core::alloc::__bs_alloc;
use crate::types::coercion::{__bs_Object, __bs_String, __bs_Number, __bs_Boolean};
use crate::types::string_utils::{get_c_string_from_tagged, create_tagged_string};
use crate::objects::dynamic_props::{DYNAMIC_PROPERTIES, set_dynamic_property, get_dynamic_property};

"""

# array.rs
array_rs = COMMON_IMPORTS + """
#[no_mangle]
pub unsafe extern "C" fn __bs_Array_new(val: u64) -> u64 {
    let tag = val & 0xFFFF_0000_0000_0000;
    if tag == 0xFFF1_0000_0000_0000 {
        crate::array::__bs_array_new()
    } else if tag == 0 || (tag > 0 && tag < 0xFFF0_0000_0000_0000) {
        let f = f64::from_bits(val);
        let len = f as u32;
        let tagged = crate::array::__bs_array_new();
        let arr = crate::array::untag_array(tagged);
        crate::array::grow_array(arr, len);
        (*arr).length = len;
        for i in 0..len {
            *(*arr).data.add(i as usize) = 0xFFF1_0000_0000_0000; // undefined
        }
        tagged
    } else {
        let tagged = crate::array::__bs_array_new();
        crate::array::__bs_array_push(tagged, val);
        tagged
    }
}
"""

with open('rt-stubs/src/objects/builtins/array.rs', 'w') as f:
    f.write(array_rs)

# Extract sections from constructors.rs
import re

def extract_fn(name, src):
    # Matches #[no_mangle]\n pub unsafe extern "C" fn name(...) -> u64 { ... }
    # but some have multiple arguments.
    # It's tricky to write a perfect regex for Rust, so let's do bracket matching.
    idx = src.find(f"pub unsafe extern \"C\" fn {name}")
    if idx == -1:
        return ""
    start_idx = src.rfind("#[no_mangle]", 0, idx)
    if start_idx == -1: start_idx = idx
    
    # Find the opening brace
    brace_idx = src.find("{", idx)
    if brace_idx == -1: return ""
    
    count = 1
    end_idx = brace_idx + 1
    while count > 0 and end_idx < len(src):
        if src[end_idx] == '{': count += 1
        elif src[end_idx] == '}': count -= 1
        end_idx += 1
        
    return src[start_idx:end_idx]

def extract_fn_raw(name, src):
    idx = src.find(f"fn {name}")
    if idx == -1: return ""
    brace_idx = src.find("{", idx)
    count = 1
    end_idx = brace_idx + 1
    while count > 0 and end_idx < len(src):
        if src[end_idx] == '{': count += 1
        elif src[end_idx] == '}': count -= 1
        end_idx += 1
    return src[idx:end_idx]

# object.rs
object_fns = [
    "__bs_new_object",
    "__bs_Object_new",
    "__bs_Object_new_0",
    "__bs_Object_new_1",
    "__bs_object_keys",
    "__bs_object_rest",
    "__bs_object_values",
    "__bs_object_entries",
    "__bs_object_assign",
    "__bs_object_create",
    "__bs_object_getPrototypeOf",
    "__bs_object_fromEntries",
    "__bs_get_globalThis"
]
object_rs = COMMON_IMPORTS + "\n\n".join([extract_fn(f, content) for f in object_fns])
# add static mut GLOBAL_THIS_OBJ
object_rs = object_rs.replace('#[no_mangle]\npub unsafe extern "C" fn __bs_get_globalThis', 'static mut GLOBAL_THIS_OBJ: u64 = 0;\n\n#[no_mangle]\npub unsafe extern "C" fn __bs_get_globalThis')
with open('rt-stubs/src/objects/builtins/object.rs', 'w') as f:
    f.write(object_rs)

# string.rs
string_fns = [
    "__bs_String_new",
    "__bs_String_new_0",
    "__bs_String_new_1",
    "__bs_string_fromCharCode",
    "__bs_string_fromCodePoint"
]
string_rs = COMMON_IMPORTS + "\n\n".join([extract_fn(f, content) for f in string_fns])
with open('rt-stubs/src/objects/builtins/string.rs', 'w') as f:
    f.write(string_rs)

# number.rs
number_fns = [
    "__bs_Number_new",
    "__bs_Number_new_0",
    "__bs_Number_new_1"
]
number_rs = COMMON_IMPORTS + "\n\n".join([extract_fn(f, content) for f in number_fns])
with open('rt-stubs/src/objects/builtins/number.rs', 'w') as f:
    f.write(number_rs)

# boolean.rs
boolean_fns = [
    "__bs_Boolean_new",
    "__bs_Boolean_new_0",
    "__bs_Boolean_new_1"
]
boolean_rs = COMMON_IMPORTS + "\n\n".join([extract_fn(f, content) for f in boolean_fns])
with open('rt-stubs/src/objects/builtins/boolean.rs', 'w') as f:
    f.write(boolean_rs)

# date.rs
date_fns = [
    "__bs_Date_new",
    "__bs_Date_new_0",
    "__bs_Date_new_1",
    "__bs_Date_new_n",
    "__bs_date_now"
]
date_rs = COMMON_IMPORTS + extract_fn_raw("parse_date_string", content) + "\n\n" + "\n\n".join([extract_fn(f, content) for f in date_fns])
with open('rt-stubs/src/objects/builtins/date.rs', 'w') as f:
    f.write(date_rs)

# mod.rs
mod_rs = """pub mod object;
pub mod string;
pub mod number;
pub mod boolean;
pub mod date;
pub mod array;

pub use object::*;
pub use string::*;
pub use number::*;
pub use boolean::*;
pub use date::*;
pub use array::*;
"""
with open('rt-stubs/src/objects/builtins/mod.rs', 'w') as f:
    f.write(mod_rs)

print("Files created.")
