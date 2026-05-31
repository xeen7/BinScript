#[repr(C)]
pub struct VTable {
    pub parent: *const VTable,
    pub name: *const u8,
    pub shape_id: u64,
    pub fields_count: u64,
    pub field_names: *const *const u8,
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
};

#[no_mangle]
pub static STRING_VTABLE: VTable = VTable {
    parent: &OBJECT_VTABLE,
    name: b"String\0".as_ptr(),
    shape_id: 1002,
    fields_count: 0,
    field_names: std::ptr::null(),
};

#[no_mangle]
pub static NUMBER_VTABLE: VTable = VTable {
    parent: &OBJECT_VTABLE,
    name: b"Number\0".as_ptr(),
    shape_id: 1003,
    fields_count: 0,
    field_names: std::ptr::null(),
};

#[no_mangle]
pub static BOOLEAN_VTABLE: VTable = VTable {
    parent: &OBJECT_VTABLE,
    name: b"Boolean\0".as_ptr(),
    shape_id: 1004,
    fields_count: 0,
    field_names: std::ptr::null(),
};

#[no_mangle]
pub static DATE_VTABLE: VTable = VTable {
    parent: &OBJECT_VTABLE,
    name: b"Date\0".as_ptr(),
    shape_id: 1005,
    fields_count: 0,
    field_names: std::ptr::null(),
};

#[no_mangle]
pub static MAP_VTABLE: VTable = VTable {
    parent: &OBJECT_VTABLE,
    name: b"Map\0".as_ptr(),
    shape_id: 1006,
    fields_count: 0,
    field_names: std::ptr::null(),
};

#[no_mangle]
pub static SET_VTABLE: VTable = VTable {
    parent: &OBJECT_VTABLE,
    name: b"Set\0".as_ptr(),
    shape_id: 1007,
    fields_count: 0,
    field_names: std::ptr::null(),
};

#[no_mangle]
pub static WEAKMAP_VTABLE: VTable = VTable {
    parent: &OBJECT_VTABLE,
    name: b"WeakMap\0".as_ptr(),
    shape_id: 1008,
    fields_count: 0,
    field_names: std::ptr::null(),
};

#[no_mangle]
pub static WEAKSET_VTABLE: VTable = VTable {
    parent: &OBJECT_VTABLE,
    name: b"WeakSet\0".as_ptr(),
    shape_id: 1009,
    fields_count: 0,
    field_names: std::ptr::null(),
};

#[no_mangle]
pub static ERROR_VTABLE: VTable = VTable {
    parent: &OBJECT_VTABLE,
    name: b"Error\0".as_ptr(),
    shape_id: 1010,
    fields_count: 0,
    field_names: std::ptr::null(),
};

#[no_mangle]
pub static REGEXP_VTABLE: VTable = VTable {
    parent: &OBJECT_VTABLE,
    name: b"RegExp\0".as_ptr(),
    shape_id: 1011,
    fields_count: 0,
    field_names: std::ptr::null(),
};
