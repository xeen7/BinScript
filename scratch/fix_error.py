import os

with open('rt-stubs/src/exception/mod.rs', 'r') as f:
    ex_content = f.read()

# error_start is at `__bs_error_new`
error_start = ex_content.find('#[no_mangle]\npub unsafe extern "C" fn __bs_error_new')
print_start = ex_content.find('unsafe fn print_exception')

error_constructors = ex_content[error_start:print_start]

error_rs = """use crate::gc;
use crate::core::vtable::{VTable, ERROR_VTABLE};
use crate::objects::dynamic_props::set_dynamic_property;
use crate::types::string_utils::create_tagged_string;

""" + error_constructors

# the new exception/mod.rs
ex_rs = ex_content[:error_start] + ex_content[print_start:]

with open('rt-stubs/src/objects/builtins/error.rs', 'w') as f: f.write(error_rs)
with open('rt-stubs/src/exception/mod.rs', 'w') as f: f.write(ex_rs)

with open('rt-stubs/src/lib.rs', 'r') as f:
    lib_rs = f.read()

# Replace the previous error new strings with the right ones
# We know lib_rs currently has `__bs_RegExp_new, \n` (since we removed __bs_Error_new_0)
lib_rs = lib_rs.replace('__bs_RegExp_new,\n', '__bs_RegExp_new,\n    __bs_Error_new, __bs_TypeError_new, __bs_RangeError_new, __bs_ReferenceError_new,\n    __bs_SyntaxError_new, __bs_URIError_new,\n')

with open('rt-stubs/src/lib.rs', 'w') as f:
    f.write(lib_rs)

print("Fixed error constructors in error.rs and exception.rs and updated lib.rs")
