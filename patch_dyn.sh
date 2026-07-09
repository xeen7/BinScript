sed -i '/pub unsafe extern "C-unwind" fn __bs_cleanup_dynamic_properties/,/}/c\
#[no_mangle]\
pub unsafe extern "C-unwind" fn __bs_cleanup_dynamic_properties() {\
    let mut map = crate::objects::dynamic_props::DYNAMIC_PROPERTIES.lock().unwrap();\
    for props in map.values() {\
        for val in props.values() {\
            crate::circ::circ_dec_tagged(*val);\
        }\
    }\
    map.clear();\
}' rt-stubs/src/objects/dynamic_props.rs
