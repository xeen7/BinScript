cat << 'INNER_EOF' > /tmp/sed_script.sed
/pub unsafe extern "C-unwind" fn __bs_cleanup_dynamic_properties() {/,/}/c\
#[no_mangle]\
pub unsafe extern "C-unwind" fn __bs_cleanup_dynamic_properties() {\
    let old_map = {\
        let mut map = crate::objects::dynamic_props::DYNAMIC_PROPERTIES.lock().unwrap();\
        std::mem::take(&mut *map)\
    };\
    for props in old_map.values() {\
        for val in props.values() {\
            crate::circ::circ_dec_tagged(*val);\
        }\
    }\
}
INNER_EOF
sed -i -f /tmp/sed_script.sed rt-stubs/src/objects/dynamic_props.rs
