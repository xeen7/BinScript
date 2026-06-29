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
