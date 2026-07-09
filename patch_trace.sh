sed -i 's/let mut children = Vec::new();//g' rt-stubs/src/cycle_collector.rs
sed -i 's/children.push(child);/let visitor_fn: unsafe extern "C-unwind" fn(\*mut CircHeader) = std::mem::transmute(visitor);\n                        visitor_fn(child);/g' rt-stubs/src/cycle_collector.rs
