//! Runtime stubs for BinScript compiled binaries.
//!
//! In Stage 2, runtime helpers contain memory allocation, zero-initialization,
//! and prototype inheritance verification stubs.

pub mod gc;
pub mod shadow_stack;
pub mod promise;
pub mod json;
pub mod array;
pub mod string;
#[macro_use]
pub mod dynamic_call;
pub mod exception;

// Modular subdirectories
pub mod core;
pub mod types;
pub mod objects;
pub mod system;
pub mod generators;

// Re-export core module symbols
pub use core::vtable::{
    VTable, OBJECT_VTABLE, STRING_VTABLE, NUMBER_VTABLE, BOOLEAN_VTABLE, DATE_VTABLE,
    MAP_VTABLE, SET_VTABLE, WEAKMAP_VTABLE, WEAKSET_VTABLE, ERROR_VTABLE, REGEXP_VTABLE,
};
pub use core::alloc::{__bs_alloc, __bs_alloc_closure, __bs_alloc_generator};
pub use core::instanceof::__bs_instanceof;

// Re-export types module symbols
pub use types::typeof_rt::__bs_typeof;
pub use types::string_utils::{
    get_c_string_from_tagged, create_tagged_string,
    __bs_encodeURI, __bs_decodeURI, __bs_encodeURIComponent, __bs_decodeURIComponent,
};
pub use types::coercion::{__bs_String, __bs_Number, __bs_Boolean, __bs_Object, __bs_Date};
pub use types::operators::{
    __bs_strict_eq, __bs_strict_ne, __bs_add, __bs_is_nullish, __bs_exp, __bs_in, __bs_delete_prop,
};
pub use types::symbol::{__bs_Symbol, __bs_Symbol_0, __bs_Symbol_1, __bs_get_Symbol_global};

// Re-export objects module symbols
pub use objects::dynamic_props::{
    DYNAMIC_PROPERTIES, set_dynamic_property, get_dynamic_property, delete_dynamic_property,
    remove_dynamic_properties, trace_dynamic_properties,
};
pub use objects::builtins::{
    __bs_new_object, __bs_Object_new, __bs_Object_new_0, __bs_Object_new_1,
    __bs_String_new, __bs_String_new_0, __bs_String_new_1,
    __bs_Number_new, __bs_Number_new_0, __bs_Number_new_1,
    __bs_Boolean_new, __bs_Boolean_new_0, __bs_Boolean_new_1,
    __bs_Date_new, __bs_Date_new_0, __bs_Date_new_1, __bs_Date_new_n,
    __bs_Array_new,
    __bs_date_now, __bs_string_fromCharCode, __bs_string_fromCodePoint,
    __bs_object_keys, __bs_object_rest, __bs_object_values, __bs_object_entries, __bs_object_assign,
    __bs_object_create, __bs_object_getPrototypeOf, __bs_object_fromEntries,
    __bs_get_globalThis,
    __bs_Map_new_0, __bs_Map_new_1, __bs_Set_new_0, __bs_Set_new_1,
    __bs_WeakMap_new_0, __bs_WeakMap_new_1, __bs_WeakSet_new_0, __bs_WeakSet_new_1,
    __bs_RegExp_new,  
    __bs_math_floor, __bs_math_ceil, __bs_math_round, __bs_math_abs, __bs_math_sqrt,
    __bs_math_pow, __bs_math_min, __bs_math_max, __bs_math_log, __bs_math_log2,
    __bs_math_sin, __bs_math_cos, __bs_math_tan, __bs_math_random, __bs_math_trunc,
    __bs_parseInt, __bs_parseInt_1, __bs_parseInt_2, __bs_parseFloat, __bs_isNaN, __bs_isFinite,
    __bs_number_isInteger, __bs_number_isSafeInteger,
};
pub use objects::spread::__bs_object_spread;

// Re-export system module symbols
pub use system::fs::{__bs_fs_read_file_sync, __bs_fs_write_file_sync, __bs_fs_exists_sync};
pub use system::path::{__bs_path_join, __bs_path_resolve};
pub use system::os::{__bs_os_platform, __bs_os_arch};

// Re-export generators module symbols
pub use generators::runtime::{
    GeneratorState, ArrayIteratorState, ARRAY_ITERATORS, __bs_generator_next, __bs_generator_is_done,
};

pub fn stub_version() -> &'static str {
    "0.3.0-stage3"
}

/// Dynamic import runtime stub: returns a resolved Promise containing a fresh empty namespace object.
#[no_mangle]
pub unsafe extern "C" fn __bs_dynamic_import(_specifier: u64) -> u64 {
    let p = crate::promise::__bs_promise_new();
    let obj = __bs_new_object();
    crate::promise::__bs_promise_resolve(p, obj);
    p
}
