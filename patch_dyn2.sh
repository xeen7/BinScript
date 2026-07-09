sed -i '/#\[no_mangle\]/{n; /#\[no_mangle\]/d;}' rt-stubs/src/objects/dynamic_props.rs
sed -i '/map.clear();/{n;n;n; d;}' rt-stubs/src/objects/dynamic_props.rs
