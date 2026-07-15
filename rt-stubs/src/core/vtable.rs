#[repr(C)]
pub struct VTable {
    pub parent: *const VTable,
    pub name: *const u8,
    pub shape_id: u64,
    pub fields_count: u64,
    pub field_names: *const *const u8,

    /// RAII destructor. Called exactly once per object lifetime when the object
    /// is destroyed (RC reaches zero, ASAP last-use, arena dtor_list, or scope exit).
    ///
    /// Responsibilities of `drop_fn`:
    ///   1. Release all held external resources (close fd, unlock mutex, etc.)
    ///   2. Call `drop_fn` on all Owned child fields
    ///   3. Call `circ_dec` on all Shared(CIRC) child fields
    pub drop_fn: Option<unsafe extern "C-unwind" fn(obj: *mut u8)>,
    pub trace_fn: Option<unsafe extern "C-unwind" fn(obj: *mut u8)>,
}

unsafe impl Sync for VTable {}
unsafe impl Send for VTable {}

// ===========================================================================
// Builtin Objects VTables
// ===========================================================================

#[no_mangle]
pub static OBJECT_VTABLE: VTable = VTable {
    parent: std::ptr::null(),
    name: b"Object\0".as_ptr(),
    shape_id: 1001,
    fields_count: 0,
    field_names: std::ptr::null(),
    drop_fn: None,
    trace_fn: None,
};
#[no_mangle]
pub static STRING_VTABLE: VTable = VTable {
    parent: &OBJECT_VTABLE,
    name: b"String\0".as_ptr(),
    shape_id: 1002,
    fields_count: 0,
    field_names: std::ptr::null(),
    drop_fn: None,
    trace_fn: None,
};
#[no_mangle]
pub static NUMBER_VTABLE: VTable = VTable {
    parent: &OBJECT_VTABLE,
    name: b"Number\0".as_ptr(),
    shape_id: 1003,
    fields_count: 0,
    field_names: std::ptr::null(),
    drop_fn: None,
    trace_fn: None,
};
#[no_mangle]
pub static BOOLEAN_VTABLE: VTable = VTable {
    parent: &OBJECT_VTABLE,
    name: b"Boolean\0".as_ptr(),
    shape_id: 1004,
    fields_count: 0,
    field_names: std::ptr::null(),
    drop_fn: None,
    trace_fn: None,
};
#[no_mangle]
pub static DATE_VTABLE: VTable = VTable {
    parent: &OBJECT_VTABLE,
    name: b"Date\0".as_ptr(),
    shape_id: 1005,
    fields_count: 0,
    field_names: std::ptr::null(),
    drop_fn: None,
    trace_fn: None,
};
#[no_mangle]
pub static MAP_VTABLE: VTable = VTable {
    parent: &OBJECT_VTABLE,
    name: b"Map\0".as_ptr(),
    shape_id: 1006,
    fields_count: 0,
    field_names: std::ptr::null(),
    drop_fn: Some(crate::objects::builtins::map::map_drop),
    trace_fn: None,
};
#[no_mangle]
pub static SET_VTABLE: VTable = VTable {
    parent: &OBJECT_VTABLE,
    name: b"Set\0".as_ptr(),
    shape_id: 1007,
    fields_count: 0,
    field_names: std::ptr::null(),
    drop_fn: Some(crate::objects::builtins::set::set_drop),
    trace_fn: None,
};
#[no_mangle]
pub static WEAKMAP_VTABLE: VTable = VTable {
    parent: &OBJECT_VTABLE,
    name: b"WeakMap\0".as_ptr(),
    shape_id: 1008,
    fields_count: 0,
    field_names: std::ptr::null(),
    drop_fn: Some(crate::objects::builtins::map::map_drop),
    trace_fn: None,
};
#[no_mangle]
pub static WEAKSET_VTABLE: VTable = VTable {
    parent: &OBJECT_VTABLE,
    name: b"WeakSet\0".as_ptr(),
    shape_id: 1009,
    fields_count: 0,
    field_names: std::ptr::null(),
    drop_fn: Some(crate::objects::builtins::set::set_drop),
    trace_fn: None,
};
#[no_mangle]
pub static WEAKREF_VTABLE: VTable = VTable {
    parent: &OBJECT_VTABLE,
    name: b"WeakRef\0".as_ptr(),
    shape_id: 1020,
    fields_count: 0,
    field_names: std::ptr::null(),
    drop_fn: Some(crate::weak_ref::__bs_weakref_drop),
    trace_fn: None,
};
#[no_mangle]
pub static FINALIZATION_REGISTRY_VTABLE: VTable = VTable {
    parent: &OBJECT_VTABLE,
    name: b"FinalizationRegistry\0".as_ptr(),
    shape_id: 1021,
    fields_count: 0,
    field_names: std::ptr::null(),
    drop_fn: None, // No specific cleanup needed when the registry dies (finalizers just won't run)
    trace_fn: None,
};
#[no_mangle]
pub static ERROR_VTABLE: VTable = VTable {
    parent: &OBJECT_VTABLE,
    name: b"Error\0".as_ptr(),
    shape_id: 1010,
    fields_count: 0,
    field_names: std::ptr::null(),
    drop_fn: None,
    trace_fn: None,
};
#[no_mangle]
pub static REGEXP_VTABLE: VTable = VTable {
    parent: &OBJECT_VTABLE,
    name: b"RegExp\0".as_ptr(),
    shape_id: 1011,
    fields_count: 0,
    field_names: std::ptr::null(),
    drop_fn: None,
    trace_fn: None,
};

struct FieldNames([*const u8; 2]);
unsafe impl Sync for FieldNames {}

#[no_mangle]
static GENERATOR_RESULT_FIELD_NAMES: FieldNames = FieldNames([
    b"value\0".as_ptr(),
    b"done\0".as_ptr(),
]);

#[no_mangle]
pub static GENERATOR_RESULT_VTABLE: VTable = VTable {
    parent: &OBJECT_VTABLE,
    name: b"GeneratorResult\0".as_ptr(),
    shape_id: 1022,
    fields_count: 2,
    field_names: GENERATOR_RESULT_FIELD_NAMES.0.as_ptr(),
    drop_fn: None,
    trace_fn: None,
};

// ===========================================================================
// Global Prototypes Initialization
// ===========================================================================

pub static mut MAP_PROTOTYPE: u64 = 0;
pub static mut SET_PROTOTYPE: u64 = 0;
pub static mut WEAKMAP_PROTOTYPE: u64 = 0;
pub static mut WEAKSET_PROTOTYPE: u64 = 0;
pub static mut REGEXP_PROTOTYPE: u64 = 0;

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_init_map_prototype() {
    if MAP_PROTOTYPE != 0 { return; }
    MAP_PROTOTYPE = crate::__bs_alloc(&MAP_VTABLE, 16);
    let map_payload = MAP_PROTOTYPE & 0x0000_FFFF_FFFF_FFFF;
    crate::objects::dynamic_props::set_dynamic_property_moved(map_payload as *mut u8, "set".to_string(), crate::circ::create_builtin_method(0, crate::objects::builtins::map::map_set as *const u8));
    crate::objects::dynamic_props::set_dynamic_property_moved(map_payload as *mut u8, "get".to_string(), crate::circ::create_builtin_method(0, crate::objects::builtins::map::map_get as *const u8));
    crate::objects::dynamic_props::set_dynamic_property_moved(map_payload as *mut u8, "has".to_string(), crate::circ::create_builtin_method(0, crate::objects::builtins::map::map_has as *const u8));
    crate::objects::dynamic_props::set_dynamic_property_moved(map_payload as *mut u8, "delete".to_string(), crate::circ::create_builtin_method(0, crate::objects::builtins::map::map_delete as *const u8));
    crate::objects::dynamic_props::set_dynamic_property_moved(map_payload as *mut u8, "clear".to_string(), crate::circ::create_builtin_method(0, crate::objects::builtins::map::map_clear as *const u8));
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_init_set_prototype() {
    if SET_PROTOTYPE != 0 { return; }
    SET_PROTOTYPE = crate::__bs_alloc(&SET_VTABLE, 16);
    let set_payload = SET_PROTOTYPE & 0x0000_FFFF_FFFF_FFFF;
    crate::objects::dynamic_props::set_dynamic_property_moved(set_payload as *mut u8, "add".to_string(), crate::circ::create_builtin_method(0, crate::objects::builtins::set::set_add as *const u8));
    crate::objects::dynamic_props::set_dynamic_property_moved(set_payload as *mut u8, "has".to_string(), crate::circ::create_builtin_method(0, crate::objects::builtins::set::set_has as *const u8));
    crate::objects::dynamic_props::set_dynamic_property_moved(set_payload as *mut u8, "delete".to_string(), crate::circ::create_builtin_method(0, crate::objects::builtins::set::set_delete as *const u8));
    crate::objects::dynamic_props::set_dynamic_property_moved(set_payload as *mut u8, "clear".to_string(), crate::circ::create_builtin_method(0, crate::objects::builtins::set::set_clear as *const u8));
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_init_weakmap_prototype() {
    if WEAKMAP_PROTOTYPE != 0 { return; }
    WEAKMAP_PROTOTYPE = crate::__bs_alloc(&WEAKMAP_VTABLE, 16);
    let weakmap_payload = WEAKMAP_PROTOTYPE & 0x0000_FFFF_FFFF_FFFF;
    crate::objects::dynamic_props::set_dynamic_property_moved(weakmap_payload as *mut u8, "set".to_string(), crate::circ::create_builtin_method(0, crate::objects::builtins::map::map_set as *const u8));
    crate::objects::dynamic_props::set_dynamic_property_moved(weakmap_payload as *mut u8, "get".to_string(), crate::circ::create_builtin_method(0, crate::objects::builtins::map::map_get as *const u8));
    crate::objects::dynamic_props::set_dynamic_property_moved(weakmap_payload as *mut u8, "has".to_string(), crate::circ::create_builtin_method(0, crate::objects::builtins::map::map_has as *const u8));
    crate::objects::dynamic_props::set_dynamic_property_moved(weakmap_payload as *mut u8, "delete".to_string(), crate::circ::create_builtin_method(0, crate::objects::builtins::map::map_delete as *const u8));
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_init_weakset_prototype() {
    if WEAKSET_PROTOTYPE != 0 { return; }
    WEAKSET_PROTOTYPE = crate::__bs_alloc(&WEAKSET_VTABLE, 16);
    let weakset_payload = WEAKSET_PROTOTYPE & 0x0000_FFFF_FFFF_FFFF;
    crate::objects::dynamic_props::set_dynamic_property_moved(weakset_payload as *mut u8, "add".to_string(), crate::circ::create_builtin_method(0, crate::objects::builtins::set::set_add as *const u8));
    crate::objects::dynamic_props::set_dynamic_property_moved(weakset_payload as *mut u8, "has".to_string(), crate::circ::create_builtin_method(0, crate::objects::builtins::set::set_has as *const u8));
    crate::objects::dynamic_props::set_dynamic_property_moved(weakset_payload as *mut u8, "delete".to_string(), crate::circ::create_builtin_method(0, crate::objects::builtins::set::set_delete as *const u8));
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_init_regexp_prototype() {
    if REGEXP_PROTOTYPE != 0 { return; }
    REGEXP_PROTOTYPE = crate::__bs_alloc(&REGEXP_VTABLE, 16);
    let regexp_payload = REGEXP_PROTOTYPE & 0x0000_FFFF_FFFF_FFFF;
    crate::objects::dynamic_props::set_dynamic_property_moved(regexp_payload as *mut u8, "test".to_string(), crate::circ::create_builtin_method(0, crate::objects::builtins::regexp::regexp_test as *const u8));
    crate::objects::dynamic_props::set_dynamic_property_moved(regexp_payload as *mut u8, "exec".to_string(), crate::circ::create_builtin_method(0, crate::objects::builtins::regexp::regexp_exec as *const u8));
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_cleanup_prototypes() {
    crate::circ::circ_dec_tagged(MAP_PROTOTYPE);
    crate::circ::circ_dec_tagged(SET_PROTOTYPE);
    crate::circ::circ_dec_tagged(WEAKMAP_PROTOTYPE);
    crate::circ::circ_dec_tagged(WEAKSET_PROTOTYPE);
    crate::circ::circ_dec_tagged(REGEXP_PROTOTYPE);
}
