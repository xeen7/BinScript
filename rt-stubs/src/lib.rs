//! Runtime stubs for BinScript compiled binaries.
//!
//! Memory model: four-layer hybrid (Stack, Arena, Owned, CIRC).
//! All objects are Shared(CIRC) in Phase 1 — optimisations come later.

pub mod circ;
pub mod promise;
pub mod json;
pub mod array;
pub mod string;
#[macro_use]
pub mod dynamic_call;
pub mod exception;

pub fn __bs_init_runtime() {
    // This function can be called by the startup code or compiler to force
    // all essential unreferenced C-ABI functions to be linked.
    let _ = arena::arena_create as *const ();
    let _ = arena::arena_alloc as *const ();
    let _ = arena::arena_reset as *const ();
    let _ = arena::arena_destroy as *const ();
    let _ = arena::arena_register_dtor as *const ();
    let _ = raii::scope_guard::__bs_scope_guard_push as *const ();
    let _ = raii::scope_guard::__bs_scope_guard_cancel as *const ();
    let _ = raii::scope_guard::__bs_scope_guard_flush_to as *const ();
    let _ = raii::scope_guard::__bs_scope_guard_flush_all as *const ();
    let _ = raii::scope_guard::__bs_scope_guard_get_depth as *const ();
    let _ = raii::scope_guard::__bs_scope_guard_flush_down_to as *const ();
    let _ = rc_delta::__bs_rc_flush as *const ();
    let _ = cycle_collector::__bs_cycle_collector_init as *const ();
    let _ = rc_delta::__bs_rc_inc_deferred as *const ();
    let _ = rc_delta::__bs_rc_dec_deferred as *const ();
    let _ = weak_ref::__bs_weakref_new as *const ();
    let _ = weak_ref::__bs_weakref_deref as *const ();
    let _ = weak_ref::__bs_weakref_drop as *const ();
    let _ = finalization::__bs_finalizer_thread_init as *const ();
    let _ = finalization::__bs_finalization_registry_register as *const ();
    let _ = finalization::__bs_drain_finalizers as *const ();
    let _ = verify::__bs_verify_track_alloc as *const ();
    let _ = verify::__bs_verify_track_free as *const ();
    let _ = verify::__verify_load as *const ();
    let _ = verify::__verify_store as *const ();
    let _ = verify::__bs_verify_check_leaks as *const ();
    let _ = weak_ref::__bs_WeakRef_new_1 as *const ();
    let _ = finalization::__bs_FinalizationRegistry_new_1 as *const ();
}

// Modular subdirectories
pub mod core;
pub mod arena;
pub mod types;
pub mod objects;
pub mod system;
pub mod generators;
pub mod raii;

// Re-export core module symbols
pub use core::vtable::{
    VTable, OBJECT_VTABLE, STRING_VTABLE, NUMBER_VTABLE, BOOLEAN_VTABLE, DATE_VTABLE,
    MAP_VTABLE, SET_VTABLE, WEAKMAP_VTABLE, WEAKSET_VTABLE, ERROR_VTABLE, REGEXP_VTABLE,
};
pub use core::alloc::{__bs_alloc, __bs_alloc_closure, __bs_alloc_generator};
pub use core::instanceof::__bs_instanceof;
pub use arena::*;

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
    "0.4.0-circ"
}

/// Dynamic import runtime stub: returns a resolved Promise containing a fresh empty namespace object.
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_dynamic_import(_specifier: u64, _attributes: u64) -> u64 {
    let p = crate::promise::__bs_promise_new();
    let obj = __bs_new_object();
    crate::promise::__bs_promise_resolve(p, obj);
    p
}

// ── GC-era stubs kept as no-ops for backward compatibility ─────────────────
// These will be removed once the codegen no longer emits them.

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_safepoint_poll() {
    // No-op: GC has been removed. CIRC handles destruction immediately.
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_write_barrier(_parent: u64, _child: u64) {
    // No-op: no generational GC.
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_shadow_push(_frame: *mut u8) {
    // No-op: shadow stack has been removed.
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_shadow_pop() {
    // No-op: shadow stack has been removed.
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_shadow_set(_top_ptr: *mut u8) {
    // No-op: shadow stack has been removed.
}
pub mod slab;
pub mod rc_delta;
pub mod cycle_buffer;
pub mod cycle_collector;
pub mod weak_ref;
pub mod finalization;
pub mod verify;

#[cfg(test)]
pub mod tests;
