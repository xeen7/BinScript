// test_verify_uaf.ts
// Note: In standard TypeScript, triggering a Use-After-Free is impossible by design.
// This test serves as a placeholder to demonstrate what a memory verify check looks like.
// In the BinScript Rust runtime tests (`rt-stubs/src/tests.rs`), we simulate this by manually
// decrementing the reference count to 0 (triggering a free), and then accessing the pointer.
// The compiler's Verify Mode (`--verify-memory`) automatically intercepts the access and panics/aborts.

class DummyObject {
    field: number = 42;
}

function test_verify_uaf() {
    let obj = new DummyObject();
    
    // Memory verification hooks dynamically wrap all property accesses (load/store).
    // If `--verify-memory` is enabled:
    // 1. `new DummyObject()` triggers `__bs_verify_track_alloc`
    // 2. `obj.field` triggers `__verify_load` / `__verify_store`
    let val = obj.field;
    
    console.log("Value:", val);
    
    // In actual verified execution, if `obj` was erroneously collected here due to an RC bug,
    // the next line would trigger an immediate FATAL ABORT: Use-After-Free.
    obj.field = 99;
}

test_verify_uaf();
