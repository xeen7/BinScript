# 🗺️ Hybrid Memory Model Migration — Roadmap

> **Project:** Replace BinScript's mark-sweep GC + shadow stack with the four-layer hybrid memory model + RAII  
> **Specs:** [memory_model_final.md](./BInary/memory_model_final.md) · [raii_final.md](./BInary/raii_final.md)  
> **Started:** 2026-06-07  
> **Status:** 🟢 Complete — Memory Model Finalized

This document tracks the incremental transition to a Deterministic Hybrid Memory Model (Stack + Arena + Owned + CIRC).

## Progress Overview

| Phase | Name | Status | Files Changed | Blocked By |
|---|---|---|---|---|
| 1 | Foundations: CIRC + VTable drop_fn | `[x]` Complete | ~15 | — |
| 2 | Owned Layer + ASAP Destruction | `[x]` Complete | ~10 | Phase 1 |
| 3 | Stack Layer + Escape Analysis | `[x]` Complete | ~5 | Phase 2 |
| 4 | Arena Layer | `[x]` Complete | ~8 | Phase 2 |
| 5 | RAII Protocol | `[x]` Complete | ~15 | Phases 2, 3 |
| 6 | CIRC Optimizations (BiRC, Nursery, Elision) | `[x]` Complete | ~6 | Phase 1 |
| 7 | Cycle Collection (Bacon-Rajan) | `[x]` Complete | ~4 | Phase 1 |
| 8 | WeakRef, Finalization, Verify Mode | `[x]` Complete | ~5 | Phase 7 |

---

## Phase 1 — Foundations: CIRC + VTable `drop_fn` + Remove GC (✅ COMPLETE)

> **Goal:** Replace the entire GC subsystem with CIRC reference counting.  
> All objects are `Shared(CIRC)` — no optimization yet, but deterministic destruction.

### 1.1 — Runtime: New CIRC ABI
- `[x]` Create `rt-stubs/src/circ.rs`
  - `[x]` Define `CircHeader` struct (`rc: AtomicU32`, `flags: AtomicU32`)
  - `[x]` Implement `circ_inc()` — `Relaxed` fetch_add
  - `[x]` Implement `circ_dec()` — `AcqRel` fetch_sub, destroy at zero
  - `[x]` Implement `circ_destroy()` — call `drop_fn`, then `free()`
  - `[x]` Export all as `#[no_mangle] extern "C"` functions
- `[x]` Create `rt-stubs/src/capture_cell.rs`
  - `[x]` CIRC-managed shared mutable capture box for closures

### 1.2 — Runtime: VTable Extension
- `[x]` Modify `rt-stubs/src/core/vtable.rs`
  - `[x]` Add `drop_fn: Option<unsafe extern "C" fn(obj: *mut u8)>` to `VTable` struct
  - `[x]` Update all static vtable constants with `drop_fn: None`
  - `[x]` Verify struct layout is still `#[repr(C)]` compatible

### 1.3 — Runtime: Allocation Rewrite
- `[x]` Modify `rt-stubs/src/core/alloc.rs`
  - `[x]` Replace `gc_alloc` calls with `malloc` + `CircHeader` init
  - `[x]` `__bs_alloc`: prepend CircHeader (rc=1, flags=0), set vtable, return NaN-boxed ptr
  - `[x]` `__bs_alloc_closure`: same pattern with CircHeader
  - `[x]` `__bs_alloc_generator`: same pattern with CircHeader
  - `[x]` Ensure pointer arithmetic is correct: obj pointer = allocation_base + sizeof(CircHeader)

### 1.4 — Runtime: Delete GC + Shadow Stack
- `[x]` Delete `rt-stubs/src/gc.rs`
- `[x]` Delete `rt-stubs/src/shadow_stack.rs`
- `[x]` Modify `rt-stubs/src/lib.rs`
  - `[x]` Remove `pub mod gc;` and `pub mod shadow_stack;`
  - `[x]` Add `pub mod circ;`
  - `[x]` Remove all GC-related re-exports and references
- `[x]` Fix all compilation errors from removed GC references across rt-stubs
  - `[x]` Update `remove_dynamic_properties` call sites (was called from GC sweep)
  - `[x]` Update `free_array_data` call sites (was called from GC sweep)
  - `[x]` Update `GLOBAL_ROOTS` references (used by promise/async)

### 1.5 — Codegen: Remove Shadow Stack Emission
- `[x]` Modify `crates/codegen-llvm/src/codegen/func.rs`
  - `[x]` Remove `ShadowFrame` alloca from `emit_normal_function()`
  - `[x]` Remove `__bs_shadow_push` call from function prologues
  - `[x]` Remove `__bs_shadow_pop` calls from all epilogue/return paths
  - `[x]` Remove shadow stack from `emit_generator_function()`
  - `[x]` Remove shadow stack from `emit_main()`
- `[x]` Modify `crates/codegen-llvm/src/codegen/mod.rs`
  - `[x]` Remove `shadow_frame_ty` field from `LlvmCodegen` struct
  - `[x]` Remove `shadow_frame_ty` from `LlvmCodegen::new()`
  - `[x]` Remove external declarations: `__bs_shadow_push`, `__bs_shadow_pop`, `__bs_safepoint_poll`
  - `[x]` Add external declarations: `circ_inc(ptr)`, `circ_dec(ptr)`

### 1.6 — MIR: Add Memory-Class Instructions
- `[x]` Modify `crates/mir/src/types.rs`
  - `[x]` Add `AllocShared(MirReg, String)` — CIRC allocation
  - `[x]` Add `RcInc(MirReg)` — emit circ_inc
  - `[x]` Add `RcDec(MirReg)` — emit circ_dec
  - `[x]` Add `Drop(MirReg)` — call drop_fn + free
  - `[x]` Add `StoreSharedField(MirReg, u32, MirOperand)` — field store with RC
- `[x]` Modify MIR lowering to emit `AllocShared` instead of `Alloc`
- `[x]` Modify codegen instr dispatch to handle new instructions

### 1.7 — Codegen: CIRC-Aware Field Stores
- `[x]` Implement `StoreSharedField` codegen
  - `[x]` Load old field value
  - `[x]` Store new value
  - `[x]` `circ_inc(new_value)` if heap-tagged
  - `[x]` `circ_dec(old_value)` if heap-tagged
- `[x]` Update `Return` codegen to emit `circ_dec` on all live registers (or rely on scope-end cleanup)

### 1.8 — Workspace: Update Dependencies
- `[x]` Modify `Cargo.toml`
  - `[x]` Remove `once_cell` if no longer needed (was used by GC_HEAP)
  - `[x]` Add `crossbeam-channel` to rt-stubs (for future cycle buffer)
- `[x]` Modify `rt-stubs/Cargo.toml`
  - `[x]` Remove any GC-only dependencies

### 1.9 — Phase 1 Verification Gate
- `[x]` `cargo build --workspace` succeeds
- `[x]` All existing tests in `tests/` pass
- `[x]` Compile a simple program: class, closure, async/await — runs correctly
- `[x]` No Valgrind errors (no leaks, no UAF) on basic test programs
- `[x]` Confirm: no shadow stack overhead in generated LLVM IR
- `[x]` Confirm: CircHeader present before every heap object

---

## Phase 2 — Owned Layer + ASAP Destruction

> **Goal:** Values with provably zero aliases get `MemoryClass::Owned` — freed at last use, no RC overhead.

### 2.1 — New Crate: ownership-inference
- `[x]` Create `crates/ownership-inference/Cargo.toml` + add to workspace
- `[x]` Create `src/lib.rs` — public pass entry point
- `[x]` Create `src/alias_graph.rs`
  - `[x]` `AliasGraph` struct (petgraph DiGraph)
  - `[x]` `AliasKind` enum: Move, Borrow, Clone, Store, Alias
  - `[x]` Union-find for alias set taint propagation
  - `[x]` `build_alias_graph()` function
- `[x]` Create `src/escape.rs`
  - `[x]` `EscapeAnalysis` struct
  - `[x]` `FunctionSummary` + `EscapeFact` enum
  - `[x]` Escape detection: return, store, capture, pass-to-unknown
- `[x]` Create `src/classify.rs`
  - `[x]` `MemoryClass` enum: Stack, Arena(RegionId), Owned, Shared
  - `[x]` `classify_binding()` → (MemoryClass, Option<PromotionCertificate>)
  - `[x]` `PromotionCertificate` + `PromotionEvidence` types
  - `[x]` **INVARIANT:** never branch on `is_const` for ownership
- `[x]` Create `src/region.rs` — stub for Phase 4

### 2.2 — MIR: Owned Instructions
- `[x]` Add `AllocOwned(MirReg, String)` to MirInstr
- `[x]` Add `Borrow(MirReg, MirReg)` — non-owning ptr
- `[x]` Add `BorrowMut(MirReg, MirReg)` — mutable borrow

### 2.3 — MIR Lowering: Liveness + ASAP Drop Insertion
- `[x]` Implement liveness analysis on MIR registers
- `[x]` Insert `Drop(reg)` at last-use point for Owned bindings
- `[x]` Emit `AllocOwned` when ownership inference classifies as Owned

### 2.4 — Codegen: Owned Emission
- `[x]` `AllocOwned` → `malloc` + zero-init (NO CircHeader)
- `[x]` `Drop` → `load vtable.drop_fn; if non-null: call drop_fn(obj); call free(obj)`
- `[x]` `Borrow` → raw ptr with `readonly` LLVM attribute

### 2.5 — Phase 2 Verification Gate
- `[x]` `cargo build --workspace` succeeds
- `[x]` All tests pass
- `[x]` Verify: simple unique-owner programs use `AllocOwned` (no CircHeader)
- `[x]` Verify: aliased programs still use `AllocShared` (safe fallback)
- `[x]` Verbose mode (`-Wmemory-class=verbose`) emits diagnostics

---

## Phase 3 — Stack Layer + Escape Analysis

> **Goal:** Small non-escaping values use `alloca` — zero heap cost.
**Status**: COMPLETE

### 3.1 — Ownership Inference: Stack Promotion
- `[x]` Add size check: ≤ `STACK_LIMIT` (256 bytes)
- `[x]` Integrate escape analysis: does not escape frame, not captured
- `[x]` Issue `PromotionCertificate::NoEscape` for stack-eligible values
- `[x]` Enhance escape analysis to accurately track object scopes (fixing `this` pointer in constructors).

### 3.2 — MIR + Codegen: Stack Allocation
- `[x]` Add `AllocStack(MirReg, ShapeId)` to MirInstr
- `[x]` Codegen: `alloca` in function entry block
- `[x]` Stack values with `drop_fn` → call at epilogue and early returns

### 3.3 — Phase 3 Verification Gate
- `[x]` Tests pass
- `[x]` Verify: `{ x: number, y: number }` structs are stack-allocated
- `[x]` Verify: stack values are NOT freed with `free()`

---

## Phase 4 — Arena Layer

> **Goal:** Bump-allocate groups of objects with identical lifetimes.

### 4.1 — Runtime: Arena Allocator
- `[x]` Create `rt-stubs/src/arena.rs` — bump allocator + segment chain + dtor_list
- `[x]` Create `rt-stubs/src/arena_pool.rs` — named arena pools

### 4.2 — Ownership Inference: Region Identification
- `[x]` Strategy 1: function-scope with no escape
- `[x]` Strategy 2: dominance-based region merging
- `[x]` Strategy 3: named arena pools (pattern matching)
- `[x]` Strategy 4: JSDoc `@region` pragma

### 4.3 — MIR + Codegen: Arena Instructions
- `[x]` Add `AllocArena(MirReg, ShapeId, RegionId)` + `ArenaRelease(RegionId)`
- `[x]` Codegen: `arena_alloc()` / `arena_destroy()`
- `[x]` RAII_EXTERNAL shapes with `dtor_list` enforcement

### 4.4 — Phase 4 Verification Gate
- `[x]` Tests pass
- `[x]` Verify: loop-scoped objects use arena allocation
- `[x]` Verify: arena_destroy frees all memory in O(1) per segment

---

## Phase 5 — RAII Protocol (✅ COMPLETE)

> **Goal:** Deterministic resource release — scope guards, DRA, lifecycle inference.

### 5A — Drop Functions & Arena dtor_list
- `[x]` Compiler-generated `drop_fn` for user classes (`codegen/drop_fn_gen.rs`)
- `[x]` Add `DtorEntry` and `dtor_list` to `Arena` struct
- `[x]` `arena_register_dtor` and `arena_destroy` logic
- `[x]` MIR `CallDropFnOnly` instruction

### 5B — Local Scope Guards
- `[x]` Runtime support: `scope_guard.rs` (Push, Cancel, FlushTo)
- `[x]` MIR Instructions: `ScopeGuardPush`, `ScopeGuardCancel`, `ScopeGuardFlushTo`
- `[x]` Codegen: Emit scope guards correctly

### 5C — Lifecycle Inference & DRA (✅ COMPLETE)
- `[x]` Implement Definite Release Analysis (DRA) and scope guards to automate resource management and guarantee safety under exceptional paths.
- `[x]` Dynamic Scope Guard Engine (`__bs_scope_guard_push`/`__bs_scope_guard_flush_to`) mapping to implicit `try/finally` blocks around matching acquire/release verb pairs.
- `[x]` Definite Release Analysis (`crates/mir/src/dra.rs`) to trace linear resource paths and compute minimum scope flushes, selectively inserting dynamic releases strictly on divergent CFG branches. 
- `[x]` Exception Safety integrating `GUARD_STACK` depth unwinding into the `setjmp`/`longjmp` exception handler path via `__bs_throw()`.
- `[x]` **Phase 5C: Implement Native Exception Mechanism (`invoke`/`landingpad`)**
    - `[x]` Replace `setjmp`/`longjmp` with LLVM's `invoke` and `landingpad` in `codegen/instr/exceptions`.
    - `[x]` Static link `libunwind.a`, `-llzma`, and `-lstdc++`.
    - `[x]` Connect unwinder to `__bs_scope_guard_flush_to(0)` via `catch i8* null` cleanup block.

### 5D — Phase 5 Verification Gate (✅ COMPLETE)
- `[x]` Compile and run `tests/test_raii_paths.ts`.
- `[x]` Verify output matches the expected `open` / `close` acquisition and release sequence on normal, return, and throw paths.

---

## Phase 6 — CIRC Optimizations

> **Goal:** BiRC, nursery slab, RC delta deferral, compiler RC elision.

### 6.1 — BiRC (Biased Reference Counting)
- `[x]` Create `rt-stubs/src/circ_birc.rs`
  - `[x]` Extended CircHeader: `local_rc`, `global_rc`, `owner_tid`, `flags`
  - `[x]` Thread-local fast path `circ_inc` / `circ_dec`
  - `[x]` Cross-thread share detection → promote to global

### 6.2 — Nursery Pool
- `[x]` Create `rt-stubs/src/circ_nursery.rs`
  - `[x]` Per-thread 512 KB slab
  - `[x]` Bump allocation fast path
  - `[x]` Sweep + promotion with forwarding pointers

### 6.3 — RC Delta Deferral
- `[x]` Create `rt-stubs/src/rc_delta.rs`
  - `[x]` Thread-local delta buffer
  - `[x]` Flush at call boundaries (MIR `FlushRcDelta`)

### 6.4 — Compiler RC Elision Pass
- `[x]` Create `crates/codegen-llvm/src/rc_elision.rs`
  - `[x]` Eliminate paired inc/dec in same basic block
  - `[x]` Run after inlining

### 6.5 — Phase 6 Verification Gate
- `[x]` Tests pass
- `[x]` Benchmark: allocation throughput improved vs Phase 1 baseline
- `[x]` Verify: single-threaded code never touches atomics (BiRC fast path)

---

## Phase 7 — Cycle Collection

> **Goal:** Background Bacon-Rajan cycle collector — no stop-the-world.

### 7.1 — Cycle Collector
- `[x]` Create `rt-stubs/src/cycle_collector.rs`
  - `[x]` `mark_gray`, `scan`, `collect_white` phases
  - `[x]` Background thread with configurable interval (default 5ms)
  - `[x]` `drop_fn` calls in reverse topological order of cycle graph

### 7.2 — Cycle Buffer
- `[x]` Create `rt-stubs/src/cycle_buffer.rs`
  - `[x]` Per-thread ring buffer (512 entries)
  - `[x]` Work-stealing via `crossbeam-deque`
  - `[x]` Backpressure: sync collection at `CYCLE_BUFFER_HWM` (65536)

### 7.3 — ACYCLIC Flag
- `[x]` Ownership inference: `is_shape_acyclic()` — no back-edge in shape dependency graph
- `[x]` Set ACYCLIC flag at allocation time
- `[x]` Skip cycle buffer push for ACYCLIC objects in `circ_dec`

### 7.4 — Phase 7 Verification Gate
- `[x]` Tests pass
- `[x]` Cyclic reference test: `a.next = b; b.next = a;` → both freed
- `[x]` ACYCLIC objects never enter cycle buffer
- `[x]` No stop-the-world pauses observed

---

## Phase 8 — WeakRef, Finalization, Verify Mode

> **Goal:** Complete the memory model with WeakRef support and debug tooling.

### 8.1 — WeakRef
- `[x]` Create `rt-stubs/src/weak_ref.rs`
  - `[x]` `JsWeakRef` layout: `target: AtomicPtr`, `next_weak`, linked list
  - `[x]` `WEAKREF_TARGET` flag on targeted shapes
  - `[x]` `circ_destroy_with_weak()` — nullify weak refs, ZOMBIE state
  - `[x]` `WeakRef.deref()` → promote to strong ref before use

### 8.2 — FinalizationRegistry
- `[x]` Create `rt-stubs/src/finalization.rs`
  - `[x]` Finalizer queue (crossbeam-channel MPSC)
  - `[x]` Background thread drains queue and invokes callbacks

### 8.3 — Verify Mode
- `[x]` Create `rt-stubs/src/verify.rs`
  - `[x]` `--verify-memory` flag: shadow memory tracking
  - `[x]` `__verify_load` — UAF detection with backtraces
  - `[x]` Zero overhead when flag is off

### 8.4 — Phase 8 Verification Gate
- `[x]` Tests pass
- `[x]` WeakRef test: target collected → deref returns undefined
- `[x]` FinalizationRegistry callback fires after target collection
- `[x]` `--verify-memory` catches injected UAF bug in test

---

## Hard Constraints Checklist

> These invariants from the spec must hold at ALL times after Phase 1:

- `[x]` `Shared(CIRC)` is the default — Stack/Arena/Owned require proof certificates
- `[x]` `circ_dec` uses `AcqRel` — NEVER `Relaxed`
- `[x]` `circ_inc` uses `Relaxed` — safe because caller holds a reference
- `[x]` Vtable pointers are NEVER reference-counted
- `[x]` `drop_fn` NEVER calls `free(self)` — allocator layer does that
- `[x]` `CircHeader` is always PREPENDED to the object
- `[x]` Arena + `RAII_EXTERNAL` shape without `dtor_list` is FORBIDDEN
- `[x]` `const` keyword provides ZERO ownership information
- `[x]` Objects passed to unknown external functions → immediately `Shared(CIRC)`
- `[x]` Stack values captured by closures/async → promoted to Owned or Shared

---

## Notes & Decisions Log

| Date | Decision |
|---|---|
| 2026-06-07 | Plan approved. Starting Phase 1. Roadmap moved to ImportentMdFiles/. |
| 2026-06-08 | Completed Phase 5. Memory model core features and RAII automated exception safety completed! Starting Phase 6 (CIRC Optimizations). |
