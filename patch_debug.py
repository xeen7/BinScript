import re
with open("rt-stubs/src/finalization.rs", "r") as f:
    c = f.read()
c = re.sub(r'pub unsafe extern "C-unwind" fn __bs_FinalizationRegistry_new_1\(callback: u64\) -> u64 \{', 
           'pub unsafe extern "C-unwind" fn __bs_FinalizationRegistry_new_1(callback: u64) -> u64 {\n    println!("NEW REGISTRY: {:?}", crate::core::alloc::__bs_alloc_acyclic as *const ());', c)
c = c.replace('let obj = crate::core::alloc::__bs_alloc_acyclic(&crate::core::vtable::FINALIZATION_REGISTRY_VTABLE, 16);',
              'let obj = crate::core::alloc::__bs_alloc_acyclic(&crate::core::vtable::FINALIZATION_REGISTRY_VTABLE, 16);\n    println!("NEW REGISTRY OBJ: {:#x}", obj & 0xFFFFFFFFFFFF);')
with open("rt-stubs/src/finalization.rs", "w") as f:
    f.write(c)

with open("rt-stubs/src/circ.rs", "r") as f:
    c = f.read()
c = c.replace('crate::finalization::enqueue_finalizers(header);',
              'println!("ENQUEUING FINALIZERS FOR: {:#x}", header as usize); crate::finalization::enqueue_finalizers(header);')
with open("rt-stubs/src/circ.rs", "w") as f:
    f.write(c)
