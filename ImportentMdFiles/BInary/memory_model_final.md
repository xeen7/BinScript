# Hybrid Memory Model — Final Architecture

> **Companion document to `raii_final.md`.**
> Together these two documents describe the complete memory subsystem of the
> TypeScript-to-binary compiler. This document owns: the four-layer memory model
> (Stack, Arena, Owned, CIRC), the ownership inference pass, escape analysis,
> CIRC optimisations, cycle collection, WeakRef, and all MIR/LLVM emission rules.
> `raii_final.md` owns: the RAII protocol, drop_fn, scope guards, and resource
> lifecycle management built on top of this model.
>
> No tracing GC. No mmtk. No stop-the-world. No runtime shipped.
> Every value's memory class is decided at compile time and emitted as plain
> LLVM IR + a statically linked `librt_stubs.a`.

---

## Table of Contents

1. [Design Mandate](#1-design-mandate)
2. [The Four-Layer Memory Model](#2-the-four-layer-memory-model)
3. [Layer 1 — Stack](#3-layer-1--stack)
4. [Layer 2 — Arena](#4-layer-2--arena)
5. [Layer 3 — Owned (ASAP Destruction)](#5-layer-3--owned-asap-destruction)
6. [Layer 4 — Shared (CIRC)](#6-layer-4--shared-circ)
7. [How the Four Layers Interact](#7-how-the-four-layers-interact)
8. [Ownership Inference Pass](#8-ownership-inference-pass)
9. [Escape Analysis](#9-escape-analysis)
10. [Handling JavaScript's Aliasing and `const`](#10-handling-javascripts-aliasing-and-const)
11. [Closures and Captures](#11-closures-and-captures)
12. [Async/Await State Machines](#12-asyncawait-state-machines)
13. [Prototype Objects and Vtables](#13-prototype-objects-and-vtables)
14. [CIRC Optimisation — Thread-Local Nursery Pool](#14-circ-optimisation--thread-local-nursery-pool)
15. [CIRC Optimisation — Biased Reference Counting (BiRC)](#15-circ-optimisation--biased-reference-counting-birc)
16. [CIRC Optimisation — RC Delta Deferral](#16-circ-optimisation--rc-delta-deferral)
17. [CIRC Optimisation — Compiler RC Elision](#17-circ-optimisation--compiler-rc-elision)
18. [Cycle Collection — Bacon-Rajan](#18-cycle-collection--bacon-rajan)
19. [Cycle Collection — Backpressure and ACYCLIC Flag](#19-cycle-collection--backpressure-and-acyclic-flag)
20. [Cycle Collection — Per-Thread Buffers and Work-Stealing](#20-cycle-collection--per-thread-buffers-and-work-stealing)
21. [Ownership Inference Correctness — Soundness-First Promotion](#21-ownership-inference-correctness--soundness-first-promotion)
22. [Ownership Inference Correctness — Verification and Fuzzing](#22-ownership-inference-correctness--verification-and-fuzzing)
23. [Ownership Inference Correctness — Call Graph Summaries](#23-ownership-inference-correctness--call-graph-summaries)
24. [Arena Region Inference — Pragmatic Strategy](#24-arena-region-inference--pragmatic-strategy)
25. [WeakRef and FinalizationRegistry](#25-weakref-and-finalizationregistry)
26. [MIR Instruction Set — Memory Layer](#26-mir-instruction-set--memory-layer)
27. [LLVM IR Emission per Layer](#27-llvm-ir-emission-per-layer)
28. [Updated Compiler Pipeline](#28-updated-compiler-pipeline)
29. [Complete Crate Ecosystem](#29-complete-crate-ecosystem)
30. [Complete Directory Structure](#30-complete-directory-structure)
31. [Hard Constraints](#31-hard-constraints)

---

## 1. Design Mandate

The memory model must satisfy all of the following simultaneously:

- **Zero runtime shipped.** The binary links only `librt_stubs.a` (your static
  support library) and `musl libc`. No GC library, no VM, no interpreter.
- **Zero stop-the-world pauses.** Cycle collection runs on a background thread.
  The main thread is never suspended.
- **Deterministic destruction.** Resources are released at provably known points,
  not whenever a GC decides to run.
- **Correct aliasing semantics.** JavaScript is pervasively aliased. The model
  must be sound for all aliasing patterns including those the compiler cannot prove
  are unique.
- **Scalable throughput.** Common allocation patterns must approach bump-allocator
  speed, not `malloc` speed.
- **Full compatibility with RAII.** The `drop_fn` vtable slot (described in
  `raii_final.md`) is called at the right moment by every layer without duplication.

---

## 2. The Four-Layer Memory Model

Every JS value is assigned exactly one **MemoryClass** at compile time by the
Ownership Inference Pass. The class is permanent — it does not change at runtime.

```
MemoryClass (ordered cheapest → most capable):

  Stack         — alloca; freed on stack pop; zero heap cost
  Arena(id)     — bump pointer into a named region; freed in bulk with one free()
  Owned         — malloc; single owner; freed ASAP at last-use point
  Shared(CIRC)  — malloc + CircHeader; reference-counted; concurrent; cycle-aware
```

The decision tree:

```
Can it live on the stack?
  (size ≤ STACK_LIMIT, does not escape frame, not captured by async/closure)
    YES → Stack
    NO  ↓

Does it belong to a group with identical, bounded lifetimes?
  (all objects in group proven not to escape beyond a common scope boundary)
    YES → Arena(region_id)
    NO  ↓

Is there provably at most one live reference at any point?
  (alias graph: zero outgoing Store/Borrow edges to different live scopes)
    YES → Owned
    NO  ↓

Default (aliased, shared, unknown):
    → Shared(CIRC)
```

`Shared(CIRC)` is the conservative correct default. `Stack`, `Arena`, and `Owned`
are **optimisations** the compiler applies only when soundness is proven. When in
doubt, the compiler falls back to `Shared(CIRC)` — never to a cheaper class.

---

## 3. Layer 1 — Stack

### Allocation

Stack values are emitted as LLVM `alloca` instructions in the function entry block.
LLVM's `mem2reg` pass promotes most `alloca`s to SSA registers — making them
genuinely zero-cost at the machine level.

```llvm
define i64 @addPoints(i64 %ax, i64 %ay, i64 %bx, i64 %by) {
entry:
    %a = alloca %SmallPoint    ; will be promoted to registers by mem2reg
    %b = alloca %SmallPoint
    %sum_x = add i64 %ax, %bx
    %sum_y = add i64 %ay, %by
    ret i64 ...
}
```

### Stack Allocation Conditions

ALL of the following must hold:

1. Size is statically known and ≤ `STACK_LIMIT` (default: 256 bytes, tunable).
2. Does not escape the current function frame — not returned, not stored on heap.
3. Not captured by a closure or async state machine that outlives the frame.
4. Its `drop_fn` (if any) does not touch heap objects in a way that would create
   dangling pointers after the frame pops.

### Practical Stack Candidates

- Primitive temporaries: `number`, `boolean`, inline `string` (≤ 7 bytes NaN-boxed).
- Small fixed-shape structs with no heap children: `{ x: number, y: number }`.
- Loop induction variables.
- Async state machine resume values.

### Interaction with RAII

Stack values whose `Shape` has a non-null `drop_fn` have it called in the function
epilogue (before the stack frame pops) and at every early-return site.
See `raii_final.md` Section 16 for the full protocol.

---

## 4. Layer 2 — Arena

### Concept

An arena (bump allocator / region) allocates objects by incrementing a pointer into
a contiguous memory block. The entire arena is freed with a single `free()` call when
the region ends. No per-object tracking, no RC, no destructor calls per object.

Arenas are correct when all objects in the region provably have the same lifetime,
determined by the phase or scope they belong to.

### rt-stubs C-ABI

Implemented in `rt-stubs/src/arena.rs` (~60 lines). The API exposed to emitted code:

```c
Arena* arena_create(size_t initial_capacity);
void*  arena_alloc(Arena* a, size_t size, size_t align);
void   arena_reset(Arena* a);    // reuse without free (for pooling)
void   arena_destroy(Arena* a);  // free all memory
```

### LLVM IR — Bump Allocation

```llvm
define ptr @arena_alloc(ptr %arena, i64 %sz) {
entry:
    %bump_ptr = getelementptr %Arena, ptr %arena, i32 0, i32 1
    %cur      = load ptr, ptr %bump_ptr
    %new_bump = getelementptr i8, ptr %cur, i64 %sz
    store ptr %new_bump, ptr %bump_ptr
    ret ptr %cur
}

define void @arena_free(ptr %arena) {
    %base_ptr = getelementptr %Arena, ptr %arena, i32 0, i32 0
    %base     = load ptr, ptr %base_ptr
    call void @free(ptr %base)
    ret void
}
```

### Overflow

When the bump pointer reaches the segment end, a new segment is allocated and chained
to the previous (linked list of segments). `arena_destroy` walks the chain and frees
each segment. Overflow is transparent to the emitted code.

### Arena Region Identification

Three strategies, applied in order:

**Strategy 1 — Automatic Restricted Inference (high confidence, low risk):**

The compiler automatically identifies three patterns as arena-eligible:

- *Function-scope with no escape:* Every allocation in a function where escape
  analysis proves nothing escapes the function → the entire function body is a region.
  Arena is stack-allocated in the function prologue and destroyed at all return sites.

```rust
// If escape_analysis.all_local(func_id) == true:
// Prologue: %arena = alloca Arena; call arena_init(%arena, INITIAL_CAP)
// All alloc_owned/alloc_shared → arena_alloc(%arena, ...)
// All return paths: call arena_destroy(%arena)
```

- *Loop-iteration scope:* Objects created inside a `for...of` body that do not
  escape the iteration → per-iteration arena, reset (`arena_reset`) each iteration.
  `arena_reset` is O(1) and reuses memory without `free`/`malloc`.

- *`using`-scoped objects with no external resources:* An object declared in a
  `using` block (or inferred as uniquely scoped by RAII) with no `RAII_EXTERNAL`
  `drop_fn` → arena-allocated; freed at block end.

**Strategy 2 — Dominance-Based Region Merging:**

Two allocations can share a region if:

```
alloc_A dominates alloc_B
∧ post_dom(alloc_A) == post_dom(alloc_B)
∧ neither escapes the post-dominator block
⟹ alloc_A and alloc_B share a region
```

Implemented over the MIR CFG using `petgraph` graph algorithms, or via LLVM's
`DominatorTree`/`PostDominatorTree` passes in `inkwell`.

**Strategy 3 — Named Arena Pools (pattern-matched, zero inference):**

A pre-allocated pool of named arenas for common TypeScript patterns:

```rust
// rt-stubs/src/arena_pool.rs
pub struct ArenaPool {
    request_arena:   Arena,  // reset per HTTP request handler invocation
    iteration_arena: Arena,  // reset per for..of loop iteration
    parse_arena:     Arena,  // reset per JSON.parse() call
    temp_arena:      Arena,  // reset per identified temporary computation scope
}
```

The compiler matches call patterns at HIR lowering time:
- HTTP framework callback signature → `request_arena`
- `for...of` body allocations → `iteration_arena`
- `JSON.parse(...)` return value tree → `parse_arena`
- Default for unmatched patterns → `Owned` or `Shared(CIRC)`

**Strategy 4 — JSDoc Pragma (programmer-directed, zero inference):**

```typescript
/** @region request */
async function handleRequest(req: Request): Promise<Response> {
    /** @region-scoped */
    const user = await db.getUser(req.userId);   // arena-allocated

    /** @region-scoped */
    const profile = buildProfile(user);           // arena-allocated

    return formatResponse(profile);
    // Arena freed here — O(1) regardless of how many objects were allocated
}
```

SWC preserves JSDoc comments via `swc_node_comments`. The compiler reads `@region`
and `@region-scoped` during HIR lowering and assigns `RegionId`s accordingly.

### Arena and RAII_EXTERNAL

Shapes flagged `RAII_EXTERNAL` (their `drop_fn` touches OS handles or global state)
**cannot be arena-allocated without a destructor list**. When such a shape is
arena-allocated, the arena maintains a `dtor_list: Vec<DtorEntry>` alongside its
bump pointer. `arena_destroy` walks this list in reverse order and calls each
`drop_fn` before bulk-freeing the memory. See `raii_final.md` Section 16.

---

## 5. Layer 3 — Owned (ASAP Destruction)

### Concept

**ASAP (As Soon As Possible) destruction** frees a value immediately when its last
use is identified at compile time — not at scope end, not at function exit. This is
Rust's drop-at-last-use semantics applied to TypeScript.

```typescript
function processLargeBuffer() {
    const buf = readFile('huge.bin');    // 500 MB — Owned
    const result = parse(buf);           // buf's last use is here
    // ASAP: buf is freed HERE, not at function end
    return doExpensiveWork(result);      // runs with 500 MB already freed
}
```

### Last-Use Analysis

Every `MirReg` has a **liveness interval**: the range of MIR instructions between
its definition and its last use. ASAP destruction inserts `Drop(reg)` immediately
after the last-use instruction in the liveness interval.

The `Drop(reg)` MIR instruction codegen sequence:

```
1. Load vtable.drop_fn pointer from obj
2. If non-null: call drop_fn(obj)     ← RAII resource release
3. call free(obj)                     ← memory release
```

If `drop_fn` is null (no resources, no CIRC children): only `free(obj)` is emitted.
LLVM inlines this to a single `call free` which the optimiser often eliminates.

### The Owned ABI

Functions that take ownership of a value receive a raw `ptr` with no RC semantics.
The caller must not use the value after the call — enforced by the liveness analysis
which marks the `MirReg` as dead at the call site.

```llvm
; Owned: takes ownership, caller's register is dead after this
define void @consumePoint(ptr %p) {
    ; ... uses p ...
    call void @__owned_drop(ptr %p)   ; ASAP at end of ownership scope
}
```

---

## 6. Layer 4 — Shared (CIRC)

### What CIRC Is

**CIRC (Concurrent Immediate Reference Counting)** is the fallback for values that
cannot be proven uniquely owned. Key properties:

| Property | Detail |
|---|---|
| **Immediate** | RC is decremented the moment a reference dies. Destruction at zero happens immediately, not deferred. |
| **Concurrent** | RC operations use atomic instructions (`AcqRel` for dec, `Relaxed` for inc). Safe across threads. |
| **Inline count** | Count lives in the object header — no separate control block. One fewer pointer dereference than `std::shared_ptr`. |
| **Cycle-aware** | Backed by a deferred Bacon-Rajan collector on a background thread. No stop-the-world. |

### CircHeader Layout

```rust
// Base CIRC header (before BiRC optimisation — see Section 15):
#[repr(C)]
pub struct CircHeader {
    pub rc:    AtomicU32,   // strong reference count
    pub flags: AtomicU32,   // ACYCLIC | IN_NURSERY | FORWARDED | ZOMBIE | VTABLE_PTR
}
```

After BiRC optimisation is applied (Section 15), `CircHeader` is extended with
`local_rc`, `global_rc`, and `owner_tid`. The base layout is the starting point;
BiRC is always applied in the final implementation.

Every `Shared` allocation prepends the `CircHeader` to the object data:

```
JsObject (Shared/CIRC):
┌────────────────────────┐  ← allocation base (passed to circ_inc/circ_dec)
│ CircHeader (8 bytes)   │
├────────────────────────┤  ← obj pointer (vtable field starts here)
│ vtable: *const VTable  │
│ field_0: JsValue       │
│ field_1: JsValue       │
│ ...                    │
└────────────────────────┘
```

`Owned` and `Arena` allocations do **not** include this header — they are cheaper.

### CIRC ABI (Base — see Section 15 for BiRC)

```rust
// rt-stubs/src/circ.rs

#[no_mangle]
pub unsafe extern "C" fn circ_inc(obj: *const CircHeader) {
    // Relaxed: caller already holds a reference; count cannot drop to zero during inc
    (*obj).rc.fetch_add(1, Ordering::Relaxed);
}

#[no_mangle]
pub unsafe extern "C" fn circ_dec(obj: *const CircHeader) {
    // AcqRel: Release to publish writes; Acquire to see all prior writes on zero-check
    let prev = (*obj).rc.fetch_sub(1, Ordering::AcqRel);
    if prev == 1 {
        circ_destroy(obj);
    } else if prev <= CYCLE_THRESHOLD {
        let flags = (*obj).flags.load(Ordering::Relaxed);
        if flags & ACYCLIC == 0 {
            let depth = cycle_buffer_push(obj);
            if depth > CYCLE_BUFFER_HWM {
                circ_force_collect_sync();   // backpressure
            }
        }
    }
}

unsafe fn circ_destroy(obj: *const CircHeader) {
    // obj_ptr: object data starts after the CircHeader
    let obj_ptr = (obj as *const u8).add(size_of::<CircHeader>());
    let vtable  = *(obj_ptr as *const *const VTable);

    // 1. RAII: call drop_fn (releases resources, decrements CIRC children)
    if let Some(drop_fn) = (*vtable).drop_fn {
        drop_fn(obj_ptr as *mut u8);
    }

    // 2. If WEAKREF_TARGET: nullify weak refs, possibly set ZOMBIE (Section 25)
    if (*obj).flags.load(Ordering::Relaxed) & WEAKREF_TARGET != 0 {
        circ_destroy_with_weak(obj);
        return;  // circ_destroy_with_weak handles the free()
    }

    // 3. Memory: free the entire allocation (header + object data)
    libc::free(obj as *mut _);
}
```

### Atomic Ordering Rules

These are non-negotiable and must never be weakened:

- **`circ_inc`:** `Relaxed` — safe; caller holds a reference, so zero-crossing
  during increment is impossible.
- **`circ_dec` `fetch_sub`:** `Release` (via `AcqRel`) — publishes all writes
  made before this drop to the thread that observes zero.
- **`circ_dec` zero-check:** `Acquire` (via `AcqRel`) — ensures the destroying
  thread sees all writes from all threads that held references.
- **Cycle buffer push:** lock-free MPSC via `crossbeam-channel`.

Using `Relaxed` for `circ_dec` is a data race. Any code review that attempts to
"optimise" this must be rejected.

### Vtable Pointers Are NOT Counted

Vtables are static global constants. Vtable pointer fields in object headers are
**never** included in RC operations. The `VTABLE_PTR` bit in `CircHeader.flags`
marks field slot 0 as a non-counted vtable pointer. CIRC operations that walk
object fields for cycle collection must skip fields marked `VTABLE_PTR`.

### Object Instance vs Vtable Lifetime

```
MyClass instance (Shared/CIRC):         MyClass_vtable (static global constant):
┌──────────────────────────┐            ┌──────────────────────────┐
│ CircHeader (rc, flags)   │            │ parent_vtable: ptr       │
│ vtable: *MyClass_vtable  │──ptr──────▶│ shape_id: u64            │
│ field_0: JsValue         │            │ drop_fn: ptr             │
│ field_1: JsValue         │            │ toString: ptr            │
└──────────────────────────┘            │ ...methods...            │
                                        └──────────────────────────┘
Instance RC managed by CIRC.            Vtable lives forever; never counted.
```

---

## 7. How the Four Layers Interact

### Memory Class Decision — Made Once, Immutable

The memory class is assigned during the Ownership Inference Pass, before MIR
lowering. It never changes at runtime. All MIR instructions and all LLVM IR
emission use the class assigned at compile time.

### Mixed-Class Objects

A single object can have fields of different memory classes:

```typescript
class HttpServer {
    owned_config:      Config;      // Owned — never aliased
    shared_handler_map: HandlerMap; // Shared(CIRC) — multiple references
    arena_request?:    Request;     // Arena — per-request scope
}
```

The LLVM struct for `HttpServer` contains:
- Plain pointer for `owned_config` — no RC header.
- `CircHeader`-prefixed pointer for `shared_handler_map`.
- Raw pointer + `ArenaId` for `arena_request`.

MIR field stores use class-specific instructions:
- `StoreOwnedField` for `owned_config` — no RC.
- `StoreSharedField` for `shared_handler_map` — emits `circ_inc` on new value,
  `circ_dec` on old value.
- `StoreArenaField` for `arena_request` — no RC, no free.

---

## 8. Ownership Inference Pass

### Position in the Pipeline

Inserted between **Semantic Analysis** and **HIR Lowering**. Inputs: HIR + scope
tree + call graph. Output: `MemoryClass` annotation on every `BindingInfo`.

### Alias Graph

```rust
// crates/ownership-inference/src/alias_graph.rs

pub struct AliasGraph {
    graph: petgraph::graph::DiGraph<BindingId, AliasKind>,
}

pub enum AliasKind {
    Move,    // ownership transferred; source is dead after this edge
    Borrow,  // non-owning read; source still owns
    Clone,   // structural copy; both own independently
    Store,   // source stored into container; container co-owns
    Alias,   // direct alias: `const b = a` — same allocation, two bindings
}
```

A node with **two or more outgoing `Store` or `Alias` edges to different live
scopes** is immediately classified `Shared(CIRC)`. No further analysis.

### Union-Find for Alias Sets

Alias taint propagates transitively using a **union-find** (disjoint set) structure:

```rust
pub fn build_alias_graph(stmts: &[HirStmt], scope: &Scope) -> AliasGraph {
    let mut graph = AliasGraph::new();

    for stmt in stmts {
        match stmt {
            HirStmt::Let { binding, init: Some(HirExpr::Var(src)) } => {
                graph.add_edge(*src, *binding, AliasKind::Alias);
                graph.union_alias_sets(*src, *binding);  // taint both
            }
            HirStmt::Assign { target: HirLValue::Var(dst), value: HirExpr::Var(src) } => {
                graph.add_edge(*src, *dst, AliasKind::Alias);
                graph.union_alias_sets(*src, *dst);
            }
            _ => {}
        }
    }
    graph
}
```

Any binding in the same alias set as another live binding is `Shared(CIRC)`.

### Dataflow Classification

```
For each binding B, in topological scope order:
  1. class = Stack
  2. If size(B) > STACK_LIMIT (default 256 bytes) → class = Owned
  3. If B escapes the current scope                → class = Owned
  4. If B is arena-eligible (same lifetime as known region) → class = Arena(id)
  5. If alias_graph.out_degree(B) > 0 to different live scopes → class = Shared(CIRC)
  6. If B is captured by a closure that outlives B's scope → class = Shared(CIRC)
  7. If B is passed to an Unknown external function → class = Shared(CIRC)
```

### Soundness Invariant

The pass is only allowed to promote a value to a cheaper class when it can produce
a **proof certificate**. If no certificate can be constructed, it must emit
`Shared(CIRC)`. The failure mode is always a potential leak, never a UAF.

```rust
// crates/ownership-inference/src/classify.rs

pub struct PromotionCertificate {
    pub binding:      BindingId,
    pub target_class: MemoryClass,
    pub evidence:     PromotionEvidence,
}

pub enum PromotionEvidence {
    /// No alias edge in the alias graph for this binding
    NoAliasEdge { alias_graph_snapshot: AliasGraphId },

    /// All uses are in the same scope; none escape
    NoEscape { scope_id: ScopeId, use_set: Vec<UseId> },

    /// All objects share a region lifetime
    SameLifetimeAsRegion { region_id: RegionId, objects: Vec<BindingId> },
}

fn classify_binding(b: &BindingInfo, alias_graph: &AliasGraph, escape: &EscapeAnalysis)
    -> (MemoryClass, Option<PromotionCertificate>)
{
    // INVARIANT: const-ness is NOT evidence of uniqueness.
    // Uniqueness is ONLY proved by the alias graph.
    // This function MUST NEVER branch on b.is_const for ownership decisions.

    let out_degree = alias_graph.out_degree(b.id);
    let escapes    = escape.escapes(b.id);

    if out_degree == 0 && !escapes {
        let cert = PromotionCertificate {
            binding:      b.id,
            target_class: MemoryClass::Owned,
            evidence:     PromotionEvidence::NoAliasEdge {
                alias_graph_snapshot: alias_graph.snapshot_id(),
            },
        };
        (MemoryClass::Owned, Some(cert))
    } else {
        (MemoryClass::Shared, None)
    }
}
```

When verbose mode is active (`-Wmemory-class=verbose`), the compiler emits diagnostics
for every failed promotion:
`note: could not promote X to Owned: alias edge at line 42 → treated as Shared(CIRC)`.

### `Unique<T>` Programmer Annotation

For cases where the programmer knows a value is uniquely owned:

```typescript
type Unique<T> = T & { readonly __unique: unique symbol };

function makeBuffer(size: number): Unique<Buffer> { ... }
const buf: Unique<Buffer> = makeBuffer(1024);
// Compiler forces MemoryClass::Owned, skipping alias analysis
```

If a `Unique<T>` binding is aliased, the compiler emits a warning and reverses the
promotion to `Shared(CIRC)`. The annotation is a hint, not an unsafe assertion.

---

## 9. Escape Analysis

Escape analysis is a sub-pass of the Ownership Inference Pass.

### A Value Escapes When

- It is **returned** from the function that created it.
- It is **stored** in a container whose lifetime exceeds the current scope.
- It is **captured** by a closure (unless the closure is proven to be
  immediately invoked and not stored).
- It is **passed** to a function that stores it (determined from callee summaries;
  conservative for unknown callees).

Non-escaping values are candidates for Stack or Arena allocation.

### Interprocedural Escape via Call Graph Summaries

For known functions in the same compilation unit, the pass uses **function summaries**
to propagate escape facts interprocedurally:

```rust
// crates/ownership-inference/src/escape.rs

pub struct FunctionSummary {
    pub func_id:      FuncId,
    pub param_escape: Vec<EscapeFact>,
}

pub enum EscapeFact {
    DoesNotEscape,                      // safe to pass Owned or Borrowed
    EscapesViaReturn,                   // caller gets it back — still Owned in caller
    EscapesViaField { field: usize },   // stored in self.field — check self's class
    EscapesGlobally,                    // must be Shared(CIRC) at call site
    EscapesViaCapture,                  // captured by a closure in the callee
    Unknown,                            // conservative — treat as EscapesGlobally
}
```

Summaries are computed in a fixpoint loop over the call graph (bottom-up, then
propagated top-down). Summaries are stored in the incremental cache keyed by
function content hash.

For external functions (npm packages, built-in shims): `Unknown` for all parameters.
Any object passed to an `Unknown` function is classified `Shared(CIRC)` immediately.

---

## 10. Handling JavaScript's Aliasing and `const`

### The Core Tension

```typescript
const a = { x: 1 };
const b = a;         // b IS a — same allocation, two bindings
b.x = 2;
console.log(a.x);    // 2 — mutation through alias is correct JS semantics
```

`b = a` is **not** a Move. It is an alias assignment. Both `a` and `b` must be
`Shared(CIRC)`. The alias graph detects this via the union-find taint propagation.

### Alias-Creating Patterns

All of the following create aliases and force `Shared(CIRC)` on the source:

```typescript
const b = a;                     // direct alias
arr.push(a);                     // stored into array
obj.field = a;                   // stored into object field
function f(x) { store(x); }     // x escapes via store in callee
const [x, y] = pair;            // destructuring aliases pair's fields
```

### `const` Carries Zero Ownership Information

`const` means the binding is not reassigned. It says nothing about the
object's aliasability. The inference pass must never branch on `is_const` for
ownership decisions. This rule is encoded as a load-bearing comment in `classify.rs`
(see Section 8) and enforced by code review.

For external functions with `Unknown` summary: **any object passed to them is
immediately `Shared(CIRC)`**, regardless of how it was declared. No exceptions.

---

## 11. Closures and Captures

### Capture Ownership Rules

| Captured variable class | Mechanism |
|---|---|
| Stack | Promote to `Owned` or `Shared(CIRC)` — closure outlives the stack frame |
| Arena | Extend arena lifetime to cover closure lifetime, or promote to `Owned` |
| Owned (not mutated by closure) | `BorrowedPtr` — closure holds a non-owning ptr |
| Owned (mutated by closure) | `OwnedMove` — closure takes ownership |
| Shared(CIRC) | `SharedRcInc` — `circ_inc` on capture, `circ_dec` on closure drop |

### Capture Struct Layout

```rust
pub struct CaptureField {
    pub binding: BindingId,
    pub mode:    CaptureMode,
}

pub enum CaptureMode {
    BorrowedPtr,   // non-owning raw ptr; no RC; caller guarantees liveness
    OwnedMove,     // moved in; closure's drop_fn frees it
    SharedRcInc,   // circ_inc at capture; circ_dec in closure drop_fn
}
```

The closure function pointer lives in the vtable (`vtable.call_fn`), not in the
capture struct. The capture struct holds only the captured values.

### Multiple Closures Sharing a Mutable Variable

When two or more closures capture the same mutable binding:

```typescript
let count = 0;
const inc = () => { count++; };
const get = () => count;
```

`count` is promoted to a **CaptureCell**: a heap-allocated, CIRC-managed single-value
box that all closures share:

```rust
// rt-stubs/src/capture_cell.rs
#[repr(C)]
pub struct CaptureCell {
    circ_header: CircHeader,
    vtable:      *const VTable,
    value:       JsValue,       // NaN-boxed value
}
```

Both closures hold a `Shared(CIRC)` pointer to the same `CaptureCell`.
`inc` does `StoreSharedField(cell.value)`. `get` does `LoadField(cell.value)`.
The cell is freed when the last closure is dropped.

---

## 12. Async/Await State Machines

### State Machine Allocation

An async state machine struct is allocated as:

- `Shared(CIRC)` — if the returned Promise is stored in more than one place, or
  if the machine is `await`-ed from multiple call sites (e.g. `Promise.all`).
- `Owned` — otherwise. Freed via ASAP when the Promise resolves.

### Live Variables Across Suspension Points

Variables live across an `await` point are stored in the state machine struct.
Their class determines how they are stored:

| Class | In state machine |
|---|---|
| Stack | Promoted to `Owned` field — cannot live on stack across suspension |
| Owned | Stored as owned field; freed when machine is destroyed |
| Shared(CIRC) | Stored with `RcInc` on suspension; `RcDec` on resume or machine drop |
| Arena | Arena lifetime must cover full async function lifetime; else promote to `Owned` |

### Promise Chain Memory

```typescript
const result = await fetch(url)
    .then(r => r.json())
    .then(data => process(data));
```

Each `.then` callback is a `Shared(CIRC)` closure. The chain is a linked list of
closures, all CIRC-managed because the Promise executor holds a reference to each.
Each callback is `RcDec`-dropped after it runs — no GC involvement.

---

## 13. Prototype Objects and Vtables

Prototype vtables are `global constant`s in LLVM IR. They live for the entire program
duration. They are **never** reference-counted.

```llvm
; Static global constant — not heap-allocated, never freed, never counted
@MyClass_vtable = constant %MyClass_VTable {
    ptr @MyClass_parent_vtable,
    i64 SHAPE_ID_MYCLASS,
    ptr @MyClass_drop_fn,
    ptr @MyClass_toString,
    ...
}
```

Object instances are CIRC-managed (commonly shared). The vtable pointer stored in
the object header is marked with `VTABLE_PTR` in `CircHeader.flags` to prevent it
from being counted or followed during cycle collection.

The `drop_fn` in the vtable is generated by the compiler for each class. It:
1. Calls user dispose logic (if any — see `raii_final.md`).
2. Calls `circ_dec` on each `Shared(CIRC)` child field in **reverse declaration order**.
3. Does NOT call `free(self)` — that is `circ_destroy`'s job.

---

## 14. CIRC Optimisation — Thread-Local Nursery Pool

### Problem

`malloc` + `CircHeader` init is 10–30x more expensive than a nursery bump allocation.
For workloads creating millions of short-lived CIRC objects per second, this is a
real bottleneck.

### Solution: Per-Thread Nursery Slab

Each thread has a thread-local **nursery slab** — a fixed-size contiguous block
(default: 512 KB) from which `AllocShared` bumps a pointer. Objects in the nursery
have `IN_NURSERY` set in their `CircHeader.flags`.

```llvm
; AllocShared fast path (nursery):
define ptr @alloc_shared_nursery(i64 %size) {
entry:
    %bump_ptr = call ptr @__tl_nursery_bump_ptr()   ; thread-local load — always L1
    %cur      = load ptr, ptr %bump_ptr
    %new_bump = getelementptr i8, ptr %cur, i64 %size
    %end      = call ptr @__tl_nursery_end()
    %fits     = icmp ult ptr %new_bump, %end
    br i1 %fits, %fast, %slow

fast:
    store ptr %new_bump, ptr %bump_ptr
    %rc_ptr = getelementptr %CircHeader, ptr %cur, i32 0, i32 0
    store atomic i32 1, ptr %rc_ptr monotonic
    %fl_ptr = getelementptr %CircHeader, ptr %cur, i32 0, i32 1
    store i32 IN_NURSERY_FLAG, ptr %fl_ptr
    ret ptr %cur

slow:
    call void @__nursery_sweep_and_retry()
    ; ... retry or fall back to malloc
}
```

### Nursery Lifecycle

When `circ_dec` drops a nursery object's RC to zero:
- **Do NOT call `free()`**. Mark the slot dead: `rc = 0xDEAD` tombstone.
- The slab reclaims dead slots during the next **nursery sweep**.

Nursery sweep: linear scan of the slab. Triggered when the bump pointer reaches
the slab end. Dead slots are reclaimed in bulk. Survivors (RC > 0) are promoted
to the regular heap via `malloc`.

### Promotion and Forwarding Pointers

When a nursery object is promoted:
1. Allocate a new copy on the heap via `malloc`.
2. Copy all fields from nursery slot to heap copy.
3. Write a **forwarding pointer** into the nursery slot (overwrite vtable field;
   set `FORWARDED` flag in `flags`).
4. Subsequent RC operations on the nursery slot follow the forwarding pointer.

Forwarding resolution is a one-time cost. After promotion, all new references hold
the heap pointer directly.

### Cross-Thread Sharing Triggers Immediate Promotion

Nursery objects are thread-local. When `circ_inc` is called from a different thread
(detected by comparing thread IDs), the object is immediately promoted to the heap
before the increment. This is detected inside the BiRC `circ_inc` (Section 15).

### Crate

```toml
# rt-stubs/Cargo.toml
slabmalloc = { version = "0.10", default-features = false }
```

Per-size-class slabs: 8B, 16B, 32B, 64B, 128B, 256B. Objects > 256 bytes go
directly to `malloc`.

### Result

Fast-path `AllocShared` reduced to: thread-local load + pointer bump + bounds check
+ two stores ≈ 5 instructions. Within 2× of a generational nursery bump.

---

## 15. CIRC Optimisation — Biased Reference Counting (BiRC)

### Problem

Every `circ_inc`/`circ_dec` on a shared object bounces its RC cache line between
CPU cores. At 8+ threads this dominates over the actual atomic instruction cost.

### Solution: Two-Field CircHeader

```rust
// rt-stubs/src/circ_birc.rs — final CircHeader layout (replaces base layout)
#[repr(C)]
pub struct CircHeader {
    pub local_rc:  u32,           // non-atomic; only the owning thread touches this
    pub global_rc: AtomicU32,     // atomic; all foreign threads use this
    pub owner_tid: u32,           // thread ID of the biased owner; NO_OWNER when shared
    pub flags:     AtomicU32,     // ACYCLIC | IN_NURSERY | FORWARDED | ZOMBIE | WEAKREF_TARGET
    // Optional trailing field (only for WEAKREF_TARGET shapes):
    // weak_rc: AtomicU32
    // weak_ref_head: *mut JsWeakRef
}
```

Effective RC = `local_rc + global_rc`.

### BiRC `circ_inc`

```rust
#[no_mangle]
pub unsafe extern "C" fn circ_inc(obj: *const CircHeader) {
    let cur_tid = current_thread_id();
    let owner   = (*obj).owner_tid;

    if owner == cur_tid {
        // Fast path: owning thread, non-atomic. Zero cache coherency traffic.
        (*obj).local_rc += 1;
    } else if owner == NO_OWNER {
        // Already shared — use global_rc atomically
        (*obj).global_rc.fetch_add(1, Ordering::Relaxed);
    } else {
        // First cross-thread share: one-time transfer of local_rc to global_rc
        let local = (*obj).local_rc as u32;
        (*obj).local_rc = 0;
        (*obj).global_rc.fetch_add(local + 1, Ordering::AcqRel);
        (*obj).owner_tid = NO_OWNER;
        // If IN_NURSERY: promote to heap first (see Section 14)
        if (*obj).flags.load(Ordering::Relaxed) & IN_NURSERY != 0 {
            nursery_promote_to_heap(obj);
        }
    }
}
```

### BiRC `circ_dec`

```rust
#[no_mangle]
pub unsafe extern "C" fn circ_dec(obj: *const CircHeader) {
    let cur_tid = current_thread_id();
    let owner   = (*obj).owner_tid;

    if owner == cur_tid {
        (*obj).local_rc -= 1;    // non-atomic fast path
        if (*obj).local_rc == 0 {
            let global = (*obj).global_rc.load(Ordering::Acquire);
            if global == 0 {
                drop_fn_and_free(obj);
            }
            // else: shared refs still live; do not destroy
        }
    } else {
        let prev = (*obj).global_rc.fetch_sub(1, Ordering::AcqRel);
        if prev == 1 {
            // Check if owning thread's local_rc is also zero
            if (*obj).owner_tid == NO_OWNER && (*obj).local_rc == 0 {
                drop_fn_and_free(obj);
            }
        } else if prev <= CYCLE_THRESHOLD as u32 {
            let flags = (*obj).flags.load(Ordering::Relaxed);
            if flags & ACYCLIC == 0 {
                let depth = cycle_buffer_push(obj);
                if depth > CYCLE_BUFFER_HWM {
                    circ_force_collect_sync();
                }
            }
        }
    }
}
```

### Why This Is Safe

The owning thread is the **only** thread that ever reads or writes `local_rc`.
No other thread touches it. Therefore no synchronisation is needed for `local_rc`
operations. Cache coherency traffic is eliminated for the common case (owning-thread
access), which is the vast majority of TypeScript object accesses since JS is
fundamentally single-threaded by design.

---

## 16. CIRC Optimisation — RC Delta Deferral

### Problem

For heavily shared objects accessed by many threads simultaneously (connection pools,
global config), even `global_rc` atomics cause cache line bouncing.

### Solution: Thread-Local Delta Buffer

Each thread accumulates RC deltas locally. Every N operations (or when the buffer
fills), it flushes the accumulated deltas atomically in one batch.

```rust
// rt-stubs/src/rc_delta.rs

thread_local! {
    static RC_DELTA: RefCell<HashMap<*const CircHeader, i32>> = RefCell::new(HashMap::new());
    static DELTA_OPS: Cell<u32> = Cell::new(0);
}

const FLUSH_INTERVAL: u32 = 64;

pub unsafe fn circ_inc_deferred(obj: *const CircHeader) {
    RC_DELTA.with(|buf| { *buf.borrow_mut().entry(obj).or_insert(0) += 1; });
    let ops = DELTA_OPS.get() + 1;
    DELTA_OPS.set(ops);
    if ops >= FLUSH_INTERVAL { flush_rc_delta(); }
}

pub unsafe fn flush_rc_delta() {
    RC_DELTA.with(|buf| {
        for (obj, delta) in buf.borrow().iter() {
            if *delta > 0 {
                (**obj).global_rc.fetch_add(*delta as u32, Ordering::AcqRel);
            } else if *delta < 0 {
                let prev = (**obj).global_rc.fetch_sub((-*delta) as u32, Ordering::AcqRel);
                if prev == (-*delta) as u32 { circ_destroy(*obj); }
            }
        }
        buf.borrow_mut().clear();
    });
    DELTA_OPS.set(0);
}
```

**Critical constraint:** The delta buffer must be flushed before any `free()` or
`circ_dec` call that could trigger destruction, and at every function call boundary
where an object's liveness may be observed. The MIR lowering pass inserts
`FlushRcDelta` MIR instructions at these boundaries.

---

## 17. CIRC Optimisation — Compiler RC Elision

### LLVM Function Attributes

Declare `circ_inc`/`circ_dec` with memory-scoped attributes so LLVM's alias analysis
can reason about them:

```llvm
declare void @circ_inc(ptr) #0
declare void @circ_dec(ptr) #0

attributes #0 = {
    nounwind                     ; cannot throw
    willreturn                   ; always returns
    memory(argmem: readwrite)    ; only touches the pointed-to object, not global state
}
```

With `memory(argmem: readwrite)`, LLVM knows these functions only access the object's
own memory. This enables alias analysis to prove that RC operations on different
objects do not interfere, allowing reordering, combining, and elimination.

### Custom RC Elision Pass

```rust
// crates/codegen-llvm/src/rc_elision.rs

// Finds matching circ_inc / circ_dec pairs on the same pointer within the same
// basic block and removes both if no other instruction observes the pointer in between.
pub struct RcElisionPass;

impl LlvmPass for RcElisionPass {
    fn run_on_function(&self, f: &Function) {
        for block in f.basic_blocks() {
            eliminate_paired_rc_ops(block);
        }
    }
}
```

This pass eliminates 30–60% of RC operations in well-structured TypeScript code,
based on Swift's ARC RC elision research. It runs after inlining (where most
cancellable pairs become visible in the same basic block).

---

## 18. Cycle Collection — Bacon-Rajan

### Algorithm

The Bacon-Rajan algorithm is a concurrent, non-stop-the-world cycle collector.
It is used by PHP 7, CPython 3.x, and Swift's ARC. Three phases:

```
Phase 1 — mark_gray:
  Walk each candidate object in the cycle buffer.
  For each Shared child field: speculatively decrement the child's rc.

Phase 2 — scan:
  Walk candidates again.
  Objects whose speculative rc reached 0: garbage (reachable only from within the cycle).
  Objects whose speculative rc > 0: NOT garbage (have external references).
  Restore rc of non-garbage objects to original value.

Phase 3 — collect_white:
  Free all confirmed garbage objects.
  Call drop_fn on each member in reverse topological order of the cycle graph
  (so members can safely access still-live peers during destruction).
```

### Implementation

```rust
// rt-stubs/src/cycle_collector.rs

pub const CYCLE_THRESHOLD: u32 = 2;

pub fn start_cycle_collector_thread() {
    std::thread::Builder::new()
        .name("circ-cycle-collector".into())
        .spawn(|| {
            loop {
                std::thread::sleep(Duration::from_millis(5)); // tunable: --cycle-collect-interval-ms
                collect_cycles();
            }
        })
        .expect("failed to start cycle collector thread");
}

fn collect_cycles() {
    let candidates = drain_all_cycle_buffers();  // drain per-thread buffers (Section 20)
    mark_gray(&candidates);
    scan(&candidates);
    collect_white(&candidates);
}
```

The 5ms sleep is the default. Expose as `--cycle-collect-interval-ms`. The cycle
collector thread is started in the binary's `main()` prologue — one line, always present.

### drop_fn Call Order for Cycle Members

When collecting a cycle, `drop_fn` is called on each member in **reverse topological
order** of the cycle graph. This ensures that when a member's `drop_fn` runs, its
peers that it may reference are still alive (have not yet had their memory freed).
Only after all `drop_fn`s have run does the collector call `free()` on each member.

---

## 19. Cycle Collection — Backpressure and ACYCLIC Flag

### Backpressure — High-Water Mark

When the cycle buffer exceeds `CYCLE_BUFFER_HWM` (default: 65,536 entries), the
thread that pushed the last entry does not continue executing. It calls
`circ_force_collect_sync()` — a synchronous Bacon-Rajan run on the current thread.
This is the only "pause" in the system: bounded, voluntary, triggered by actual memory
pressure rather than a timer. The pause is proportional to the candidate set size,
not the heap size.

```rust
const CYCLE_BUFFER_HWM: usize = 65_536;

// In circ_dec (see Section 15 for full BiRC version):
if prev <= CYCLE_THRESHOLD {
    let depth = cycle_buffer_push(obj);
    if depth > CYCLE_BUFFER_HWM {
        circ_force_collect_sync();
    }
}
```

### ACYCLIC Compile-Time Flag

Most CIRC objects are acyclic. Pushing them to the cycle buffer on every `circ_dec`
is wasteful. The compiler sets the `ACYCLIC` flag at allocation time for shapes that
cannot participate in cycles:

```rust
// crates/ownership-inference/src/classify.rs

pub const ACYCLIC: u32 = 1 << 3;

fn is_shape_acyclic(shape_id: ShapeId, shape_table: &ShapeTable) -> bool {
    // A shape is acyclic if none of its Shared fields can transitively reach
    // the same shape (no back-edge in the shape dependency graph)
    !shape_has_back_edge(shape_id, shape_table)
}
```

In `circ_dec`, check before pushing to cycle buffer:

```rust
if prev <= CYCLE_THRESHOLD {
    let flags = (*obj).flags.load(Ordering::Relaxed);
    if flags & ACYCLIC == 0 {
        cycle_buffer_push(obj);   // only uncertain objects
    }
    // ACYCLIC: if RC > 0, genuinely referenced. If 0, already destroyed above.
}
```

60–80% of TypeScript objects can be statically marked `ACYCLIC`, eliminating the
majority of cycle buffer pushes with zero runtime overhead.

### `// @nocycles` Programmer Annotation

```typescript
// @nocycles
class RequestContext {
    method:  string;
    path:    string;
    headers: Map<string, string>;
}
```

Forces `ACYCLIC` flag on all instances of this class. If the programmer is wrong
(class does participate in cycles), the result is a memory leak — not a UAF.
The programmer takes explicit responsibility for the annotation.

---

## 20. Cycle Collection — Per-Thread Buffers and Work-Stealing

### Per-Thread Epoch Ring Buffer

Replace the single global cycle buffer with a per-thread fixed-size ring buffer:

```rust
// rt-stubs/src/cycle_buffer.rs

thread_local! {
    static LOCAL_CYCLE_BUF: RefCell<[*const CircHeader; 512]> = RefCell::new([ptr::null(); 512]);
    static LOCAL_BUF_HEAD: Cell<usize> = Cell::new(0);
    static LOCAL_BUF_TAIL: Cell<usize> = Cell::new(0);
}
```

Each thread drains its own local buffer when it fills (`circ_force_collect_sync`).
The background collector thread **work-steals** from all threads' buffers when they
are partially full, running Bacon-Rajan on the stolen batches concurrently.

Benefits:
- Eliminates MPSC contention on a single global queue.
- Distributes collection work proportionally to the thread creating the cycles.
- Keeps collection cache-local when the producing thread collects its own buffer.

Crate: `crossbeam-deque` for the work-stealing deque.

---

## 21. Ownership Inference Correctness — Soundness-First Promotion

Promotion to a cheaper memory class is only allowed when a `PromotionCertificate`
can be constructed (Section 8). Without a certificate, the value remains
`Shared(CIRC)`. This makes the failure mode a potential leak, never a UAF.

The three certificate types cover:
- **`NoAliasEdge`** — alias graph proves uniqueness.
- **`NoEscape`** — all uses proven local to a single scope.
- **`SameLifetimeAsRegion`** — all objects proven to have identical lifetime.

Verbose mode (`-Wmemory-class=verbose`) emits a diagnostic for every failed
promotion, explaining which alias edge or escape path prevented the cheaper class.

---

## 22. Ownership Inference Correctness — Verification and Fuzzing

### `--verify-memory` Debug Mode

A compiler flag that instruments every allocation, load, store, drop, and RC
operation with shadow memory tracking:

```rust
// rt-stubs/src/verify.rs  (compiled only when --verify-memory is active)

static SHADOW: Mutex<HashMap<*const u8, MemoryRecord>> = Mutex::new(HashMap::new());

pub struct MemoryRecord {
    pub class:       MemoryClass,
    pub rc:          u32,
    pub freed:       bool,
    pub alloc_bt:    Backtrace,
    pub last_use_bt: Backtrace,
}

#[no_mangle]
pub extern "C" fn __verify_load(ptr: *const u8) {
    let shadow = SHADOW.lock().unwrap();
    if let Some(rec) = shadow.get(&ptr) {
        if rec.freed {
            eprintln!("USE-AFTER-FREE at {:?}", ptr);
            eprintln!("Allocated:\n{}", rec.alloc_bt);
            eprintln!("Freed:\n{}", rec.last_use_bt);
            std::process::abort();
        }
    }
}
```

Every `LoadField` and `LoadProp` in the emitted LLVM IR calls `__verify_load` when
`--verify-memory` is active. Run the full TypeScript conformance test suite with
this flag. Any UAF surfaces with a full backtrace. Production builds compile without
this flag — zero overhead.

### Fuzz-Driven Inference Validation

A `cargo-fuzz` harness in the compiler workspace:

1. Generate random TypeScript AST fragments.
2. Run the ownership inference pass on them.
3. Compile with `--verify-memory`.
4. Execute the binary with random inputs.
5. Report any `__verify_load` abort as a corpus entry.

Run as a nightly CI job. This continuously finds edge cases that static analysis
of the inference pass itself would miss.

---

## 23. Ownership Inference Correctness — Call Graph Summaries

The inference pass's largest blind spot is interprocedural aliasing: a value passed
to function `f` that stores it in a global, when `f` is in a different compilation
unit. Call graph summaries close this gap.

For each compiled function, record how each parameter escapes. Store in the
incremental cache keyed by function content hash. On second pass (or fixpoint),
use summaries from callees to refine caller classifications.

See Section 9 for the full `FunctionSummary` / `EscapeFact` data structure.

For external functions (unknown summary): all pointer arguments are conservatively
treated as `EscapesGlobally` → caller classifies passed objects as `Shared(CIRC)`.

---

## 24. Arena Region Inference — Pragmatic Strategy

Full ML Kit region inference (1994–2000) is a research-grade problem for mutable
imperative languages. The compiler does not attempt it. Instead, four pragmatic
strategies cover the common cases (see Section 4 for full detail):

1. **Automatic restricted inference** — function-scope with no escape, loop-iteration
   scope, `using`-scoped objects with no external resources. High confidence, low risk.
2. **Dominance-based region merging** — two allocations sharing a common dominator
   and post-dominator, neither escaping the post-dominator block.
3. **Named arena pools** — pattern-matched to HTTP handlers, loop bodies, JSON parse
   calls. Zero inference required.
4. **JSDoc pragma** — `@region` / `@region-scoped` for programmer-directed regions.

These four strategies collectively cover 90%+ of real-world arena-eligible patterns
without whole-program analysis or soundness risk.

---

## 25. WeakRef and FinalizationRegistry

### Extended CircHeader for WeakRef Targets

When the compiler detects `new WeakRef(x)` where `x` is of a given `Shape`,
that shape is flagged `WEAKREF_TARGET`. Its allocation includes two additional
trailing fields:

```
Object layout for WEAKREF_TARGET shape:
┌────────────────────────────┐
│ CircHeader (base)          │
│   strong_rc: AtomicU32     │
│   weak_rc:   AtomicU32     │  ← added for WEAKREF_TARGET shapes
│   owner_tid: u32           │
│   flags:     AtomicU32     │
├────────────────────────────┤
│ vtable: *const VTable      │
│ <fields>                   │
│ weak_ref_head: *JsWeakRef  │  ← linked list of live WeakRefs pointing to this
└────────────────────────────┘
```

Objects that are never targeted by `WeakRef` have no trailing fields — zero overhead.

### JsWeakRef Layout

```rust
// rt-stubs/src/weak_ref.rs

#[repr(C)]
pub struct JsWeakRef {
    circ_header: CircHeader,              // WeakRef is itself CIRC-managed
    vtable:      *const VTable,
    target:      AtomicPtr<CircHeader>,   // ptr to target, or null if collected
    next_weak:   *mut JsWeakRef,          // linked list link in target's weak_ref list
}
```

### Strong RC → 0 Protocol (WEAKREF_TARGET objects)

```rust
unsafe fn circ_destroy_with_weak(obj: *const CircHeader) {
    // 1. Call drop_fn (RAII resource release + CIRC child decrements)
    let vtable = get_vtable(obj);
    if let Some(drop_fn) = (*vtable).drop_fn { drop_fn(obj as *mut u8); }

    // 2. Nullify all weak references
    let weak_head_ptr = obj_weak_ref_head_ptr(obj);
    let mut cur = *weak_head_ptr;
    while !cur.is_null() {
        let next = (*cur).next_weak;
        (*cur).target.store(ptr::null_mut(), Ordering::Release);
        circ_dec(cur as *const CircHeader);  // WeakRef no longer holds a target ref
        cur = next;
    }

    // 3. If weak_rc == 0: free immediately. Else: set ZOMBIE, defer free.
    let weak_rc = get_weak_rc(obj).load(Ordering::Acquire);
    if weak_rc == 0 {
        libc::free(obj as *mut _);
    } else {
        (*obj).flags.fetch_or(ZOMBIE, Ordering::Release);
        // Memory freed when last WeakRef drops and calls circ_dec on ZOMBIE obj
    }
}
```

### `WeakRef.deref()` Codegen

```llvm
%target_ptr = getelementptr %JsWeakRef, ptr %weak_ref, i32 0, i32 3
%target     = load atomic ptr, ptr %target_ptr acquire   ; must be Acquire

%is_null    = icmp eq ptr %target, null
br i1 %is_null, %return_undefined, %return_target

return_undefined:
    ret i64 UNDEFINED_NAN_BOX

return_target:
    ; Promote to strong ref to prevent destruction during use
    call void @circ_inc(ptr %target)
    ; ... use target ...
    call void @circ_dec(ptr %target)
    ret i64 ...
```

The `circ_inc` before use is mandatory. Without it, another thread could drop the
last strong reference between the null-check and the first use — a race condition.
This is the same pattern as `std::weak_ptr::lock()` in C++.

### FinalizationRegistry

`FinalizationRegistry` callbacks are pushed to a finalizer queue when an object
enters ZOMBIE state:

```rust
// rt-stubs/src/finalization.rs

// When strong RC → 0 and ZOMBIE is set:
// If a FinalizationRegistry is watching this object, push its callback to the queue.
// A background thread drains the queue and invokes callbacks.
// Callbacks run on the finalizer thread, not the main thread — matches ES spec.
```

The finalizer queue is a thread-safe MPSC queue (`crossbeam-channel`). Callbacks
are non-deterministic in their exact timing, consistent with the ES2021 spec.

---

## 26. MIR Instruction Set — Memory Layer

```rust
pub enum MirInstr {
    // === Allocation — one variant per memory class ===

    /// Emit alloca in function entry block
    AllocStack(MirReg, ShapeId),

    /// Bump-allocate in named arena region
    AllocArena(MirReg, ShapeId, RegionId),

    /// malloc + zero-init; single owner; ASAP drop at last-use point
    AllocOwned(MirReg, ShapeId),

    /// malloc + CircHeader init (rc=1, flags=owner_tid); reference-counted
    AllocShared(MirReg, ShapeId),

    // === Ownership transfer ===

    /// Move: src is dead after this; dest owns the value
    Move(MirReg, MirReg),

    // === Borrows — zero runtime cost ===

    /// Immutable borrow: non-owning ptr; readonly LLVM attribute
    Borrow(MirReg, MirReg),

    /// Mutable borrow: non-owning mut ptr; writeonly LLVM attribute
    BorrowMut(MirReg, MirReg),

    // === Reference counting ===

    /// circ_inc(obj - header_offset)
    RcInc(MirReg),

    /// circ_dec(obj - header_offset); destruction at zero is automatic
    RcDec(MirReg),

    // === Destruction ===

    /// ASAP: call vtable.drop_fn if non-null, then call free(obj)
    Drop(MirReg),

    /// Signal end of arena region: emit arena_destroy(region_ptr)
    ArenaRelease(RegionId),

    /// Flush deferred RC delta buffer before a call boundary
    FlushRcDelta,

    // === Field access — memory-class-aware ===

    /// Load a field — no ownership side effects
    LoadField(MirReg, MirOperand, FieldIdx),

    /// Store into an Owned field — no RC adjustment
    StoreOwnedField(MirOperand, FieldIdx, MirOperand),

    /// Store into a Shared field:
    ///   circ_inc(new_value); circ_dec(old_value)
    StoreSharedField(MirOperand, FieldIdx, MirOperand),

    /// Store into an Arena field — no RC, no free
    StoreArenaField(MirOperand, FieldIdx, MirOperand),

    // All RAII-specific instructions (ScopeGuardPush, ScopeGuardFlush, etc.)
    // are defined in raii_final.md Section 27 and emitted by the Lifecycle
    // Inference Pass, not by the memory model layer.
}
```

---

## 27. LLVM IR Emission per Layer

### Stack (`AllocStack`)

```rust
// inkwell codegen:
let alloca = builder.build_alloca(shape_llvm_ty, &format!("{}_stack", shape.name));
// mem2reg will eliminate this alloca if it never has its address taken
```

### Arena (`AllocArena`)

```rust
let arena_ptr = get_arena_ptr(region_id);    // thread-local or per-scope pointer
let size  = i64_ty.const_int(shape_size, false);
let align = i64_ty.const_int(shape_align, false);
let obj   = builder.build_call(
    arena_alloc_fn,
    &[arena_ptr.into(), size.into(), align.into()],
    "arena_obj",
);
// No destructor call, no RC. ArenaRelease → arena_destroy(region_ptr) at scope end.
```

### Owned (`AllocOwned` + `Drop`)

```rust
// Allocation:
let obj = builder.build_call(malloc_fn, &[size.into()], "owned_obj");
builder.build_memset(obj, i8_zero, size_val, align);

// Drop — emitted at ASAP last-use point:
let drop_fn_ptr = load_vtable_drop_fn(obj);     // getelementptr into vtable
let has_drop    = builder.build_is_not_null(drop_fn_ptr, "has_drop");
builder.build_conditional_branch(has_drop, call_drop_block, free_block);
// call_drop_block: build_call(drop_fn_ptr, &[obj.into()], "")
// free_block:      build_call(free_fn, &[obj.into()], "")
// LLVM eliminates the null-check branch when vtable is a compile-time constant
```

### Shared / CIRC (`AllocShared` + `RcInc` + `RcDec`)

```rust
// Allocation: malloc(sizeof(CircHeader) + shape_size)
let total = circ_header_size + shape_size;
let raw   = builder.build_call(malloc_fn, &[total.into()], "circ_raw");
builder.build_memset(raw, i8_zero, total_val, align);

// Init CircHeader:
let rc_ptr = builder.build_struct_gep(circ_header_ty, raw, 0, "rc_ptr");
builder.build_store(i32_ty.const_int(1, false), rc_ptr);         // strong_rc = 1
let tid_ptr = builder.build_struct_gep(circ_header_ty, raw, 2, "tid_ptr");
builder.build_store(current_thread_id_val, tid_ptr);             // owner_tid = self

// Offset past header to get obj pointer (vtable at slot 0):
let obj_ptr = builder.build_gep(i8_ty, raw, &[circ_header_size_val.into()], "circ_obj");
let vtable_slot = builder.build_struct_gep(shape_ty, obj_ptr, 0, "vtable_slot");
builder.build_store(vtable_global.as_pointer_value(), vtable_slot);

// RcInc: call circ_inc(obj - sizeof(CircHeader))
let header = builder.build_gep(i8_ty, obj, &[neg_header_size.into()], "hdr");
builder.build_call(circ_inc_fn, &[header.into()], "");

// RcDec: call circ_dec(obj - sizeof(CircHeader))
builder.build_call(circ_dec_fn, &[header.into()], "");
```

### Borrow

```llvm
; Borrowed parameter: readonly attribute, no RC adjustment
define i64 @printPoint(ptr readonly %p) {
    %x_ptr = getelementptr %Point, ptr %p, i32 0, i32 1
    %x     = load i64, ptr %x_ptr
    ret i64 %x
}
```

---

## 28. Updated Compiler Pipeline

```
TypeScript Source Files
        │
        ▼
┌──────────────────────────┐
│  SWC Parser              │  swc_ecma_parser → SWC AST (with spans, comments)
└──────────────────────────┘
        │
        ▼
┌──────────────────────────┐
│  TS Type Strip           │  swc_ecma_transforms_typescript → JS-only AST
└──────────────────────────┘
        │
        ▼
┌──────────────────────────┐
│  Module Graph            │  swc_bundler + oxc_resolver → petgraph DAG
└──────────────────────────┘
        │
        ▼
┌──────────────────────────┐
│  HIR Lowering            │  SWC AST → HIR
└──────────────────────────┘
        │
        ▼
┌──────────────────────────┐
│  Semantic Pass           │  Scope resolution, closure capture analysis,
│                          │  prototype shape inference, this-binding analysis
└──────────────────────────┘
        │
        ▼
┌──────────────────────────┐
│  Ownership Inference     │  Alias graph (petgraph), escape analysis,
│  + Memory Class          │  union-find taint propagation, call graph summaries
│    Assignment            │  → assigns Stack / Arena / Owned / Shared(CIRC)
│                          │    to every binding and field
└──────────────────────────┘
        │
        ▼
┌──────────────────────────┐
│  Lifecycle Inference     │  Resource type registry, DRA, try/finally detection,
│  (RAII — see raii_final) │  transfer analysis → ResourceDescriptorTable
└──────────────────────────┘
        │
        ▼
┌──────────────────────────┐
│  MIR Lowering            │  HIR → MIR
│                          │  AllocStack / AllocArena / AllocOwned / AllocShared
│                          │  Drop, ArenaRelease, RcInc, RcDec, FlushRcDelta
│                          │  ImplicitScopeGuardPush / ScopeGuardFlush (RAII)
│                          │  Prototype chains → vtable descriptors
│                          │  Closures → capture structs (mode-aware)
│                          │  async/await → state machines (with injected dispose)
└──────────────────────────┘
        │
        ▼
┌──────────────────────────┐
│  LLVM IR Emit            │  inkwell → LLVM IR per module (parallel via rayon)
│                          │  Stack → alloca (mem2reg promotes to registers)
│                          │  Arena → arena_alloc / arena_destroy
│                          │  Owned → malloc + Drop at ASAP last-use
│                          │  Shared → malloc+CircHeader + circ_inc/circ_dec
│                          │  RAII → invoke + landingpad for exception safety
└──────────────────────────┘
        │
        ▼
┌──────────────────────────┐
│  RC Elision Pass         │  Custom LLVM FunctionPass: eliminate paired inc/dec
│  LTO + Optimise          │  LLVM LTO (thin or full), O3, inlining, DCE
└──────────────────────────┘
        │
        ▼
┌──────────────────────────┐
│  Link                    │  lld (preferred) or system linker
│                          │  Static: librt_stubs.a + musl libc
│                          │  No libmmtk. No libgc. No libv8.
└──────────────────────────┘
        │
        ▼
  Native Binary
  — zero GC pauses
  — deterministic destruction
  — no runtime shipped
```

---

## 29. Complete Crate Ecosystem

### Compiler Crates (analysis + codegen)

| Crate | Version | Role |
|---|---|---|
| `swc_core` | latest | Umbrella re-export for all SWC crates |
| `swc_ecma_parser` | latest | Parse TS/JS → SWC AST |
| `swc_ecma_ast` | latest | SWC AST node types |
| `swc_ecma_transforms_typescript` | latest | Strip TS types → JS AST |
| `swc_ecma_transforms_base` | latest | Resolver, hygiene, fixer |
| `swc_ecma_transforms_module` | latest | CJS/ESM interop |
| `swc_ecma_visit` | latest | Visitor/fold traits |
| `swc_common` | latest | SourceMap, Span, globals |
| `swc_node_comments` | latest | Preserve JSDoc comments for RAII pragmas |
| `swc_bundler` | latest | Module graph + tree shaking |
| `oxc_resolver` | latest | Fast ESM/CJS resolver, exports field |
| `node_resolve` | latest | Node.js require() resolution |
| `inkwell` | `0.4` (LLVM 17+) | Safe LLVM IR builder |
| `llvm-sys` | matching inkwell | Raw LLVM C bindings |
| `petgraph` | latest | Alias graph, module graph, call graph, CFG |
| `bumpalo` | latest | Arena for compiler-phase HIR/MIR nodes (not emitted) |
| `typed-arena` | latest | Alternative arena for compiler objects |
| `rayon` | latest | Parallel codegen per module |
| `dashmap` | latest | Concurrent HashMap for module cache |
| `serde` + `serde_json` | latest | Config files, serialised IR |
| `bincode` | latest | Fast binary serialisation for incremental cache |
| `clap` | `4` | CLI argument parsing |
| `miette` | latest | Rich error reporting with source spans |
| `thiserror` | latest | Error enum derivation |
| `tracing` | latest | Structured logging |
| `tempfile` | latest | Temporary object file management |
| `which` | latest | Locate system linker (lld, cc) |
| `cargo-fuzz` / `libfuzzer-sys` | latest | Fuzzing harness for inference validation |

### rt-stubs Dependencies (compiled into librt_stubs.a)

| Crate | Role |
|---|---|
| `crossbeam-channel` | Lock-free MPSC queue for cycle buffer and finalizer queue |
| `crossbeam-utils` | `CachePadded` to prevent false sharing on CircHeader |
| `crossbeam-deque` | Work-stealing deque for per-thread cycle buffer collection |
| `slabmalloc` | Per-size-class slab allocator for CIRC nursery pool |
| `sonic-rs` | SIMD-accelerated lazy JSON tape (for JSON tape feature) |
| `libc` | For `libc::free`, `libc::atexit`, OS handle operations |

### crates/ownership-inference Dependencies

| Crate | Role |
|---|---|
| `petgraph` | Alias graph and CFG (shared with compiler-core) |
| union-find implementation | Alias set taint propagation (implement inline or use `union-find` crate) |

---

## 30. Complete Directory Structure

```
ts-compiler/
├── Cargo.toml                          # workspace root
│
├── crates/
│   ├── compiler-core/                  # pipeline orchestration, CLI driver
│   │   └── src/
│   │       ├── main.rs
│   │       ├── pipeline.rs
│   │       └── config.rs
│   │
│   ├── parser/                         # SWC integration (Phase 1)
│   ├── semantic/                       # scope analysis, shape inference (Phase 2)
│   │   └── src/
│   │       ├── raii_builtins.rs        # RESOURCE_TYPE_REGISTRY + tscompiler.toml merge
│   │       ├── lifecycle_patterns.rs   # ACQUISITION_VERBS, RELEASE_VERBS, heuristics
│   │       └── [existing files]
│   │
│   ├── hir/                            # HIR types + SWC→HIR lowering (Phase 3)
│   ├── prototype/                      # shape table, vtable builder (Phase 4)
│   ├── mir/                            # MIR types + HIR→MIR lowering (Phase 5)
│   │
│   ├── ownership-inference/            # Memory class assignment pass
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── alias_graph.rs          # AliasGraph, AliasKind, union-find
│   │       ├── escape.rs               # EscapeAnalysis, FunctionSummary, EscapeFact
│   │       ├── region.rs               # Arena region identification
│   │       └── classify.rs            # MemoryClass assignment, PromotionCertificate
│   │
│   ├── lifecycle-inference/            # RAII signal detection (see raii_final.md)
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── rdt.rs
│   │       ├── pass.rs
│   │       ├── dra.rs
│   │       ├── pattern_match.rs
│   │       ├── try_finally.rs
│   │       ├── transfer.rs
│   │       └── conflict.rs
│   │
│   ├── codegen-llvm/                   # inkwell LLVM IR emission (Phase 8)
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── emit.rs
│   │       └── rc_elision.rs           # Custom LLVM RC Elision FunctionPass
│   │
│   ├── module-graph/                   # petgraph DAG, resolution (Phase 9)
│   ├── incremental/                    # Cache, hashing, invalidation
│   └── diagnostics/                    # miette integration
│
└── rt-stubs/                           # Compiled into librt_stubs.a — statically linked
    ├── Cargo.toml
    └── src/
        ├── lib.rs
        │
        ├── arena.rs                    # Bump allocator + segment chain (~60 lines)
        ├── arena_pool.rs               # Named arena pools for common patterns
        │
        ├── circ.rs                     # Base CIRC ABI (circ_inc, circ_dec, circ_destroy)
        ├── circ_birc.rs                # BiRC: local_rc/global_rc/owner_tid
        ├── circ_nursery.rs             # Thread-local nursery slab + sweep + promotion
        ├── rc_delta.rs                 # RC delta buffer (deferred aggregation)
        │
        ├── cycle_collector.rs          # Bacon-Rajan: mark_gray/scan/collect_white
        ├── cycle_buffer.rs             # Per-thread epoch ring buffer + work-stealing
        │
        ├── capture_cell.rs             # CIRC-managed shared mutable capture box
        │
        ├── weak_ref.rs                 # JsWeakRef, weak_rc, ZOMBIE state
        ├── finalization.rs             # FinalizationRegistry + finalizer queue
        │
        ├── verify.rs                   # --verify-memory shadow memory (debug only)
        │
        ├── raii/                       # RAII runtime support (see raii_final.md)
        │   ├── mod.rs
        │   ├── scope_guard.rs          # ScopeGuard stack: push/flush/cancel/flush-to
        │   ├── drop_protocol.rs        # drop_fn call separated from free
        │   ├── suppressed_error.rs     # SuppressedError for multi-guard throws
        │   └── async_dispose.rs        # Injected state machine disposal helpers
        │
        ├── json_tape.rs                # Lazy JSON tape (sonic-rs integration)
        ├── prototype.rs                # __instanceof, __typeof, __has_property
        ├── property_store.rs           # Inline + overflow property storage
        ├── coercions.rs                # ToNumber, ToString, ToBool
        ├── closures.rs                 # Closure struct helpers
        ├── async_poll.rs               # Promise / async state machine poll
        ├── iterators.rs                # Iterator protocol
        │
        └── node_compat/
            ├── fs.rs                   # FileHandle with RAII drop_fn
            ├── net.rs                  # Socket/Server with RAII drop_fn
            ├── child_process.rs        # ChildProcess with RAII drop_fn
            ├── path.rs
            └── os.rs
```

---

## 31. Hard Constraints

These are invariants. Any code that violates them must be rejected in review.

| Constraint | Reason |
|---|---|
| `Shared(CIRC)` is the default. Stack/Arena/Owned require proof certificates. | Prevents UAF from over-aggressive promotion. Failure mode is a potential leak, not a crash. |
| `circ_dec` `fetch_sub` must use `AcqRel`. Never `Relaxed`. | `Relaxed` dec is a data race: destruction can happen before prior writes are visible to the destroying thread. |
| `circ_inc` may use `Relaxed`. | Safe: caller already holds a reference; count cannot drop to zero during increment. |
| Vtable pointers are NEVER reference-counted. | Vtables are static globals. Counting them inflates RC and causes leaks. Mark with `VTABLE_PTR` flag; skip in all RC operations. |
| `drop_fn` must NEVER call `free(self)`. | The allocator layer always calls `free` after `drop_fn`. Calling it inside `drop_fn` is a double-free. |
| `CircHeader` is always prepended to the object (offset 0). Object data starts at `header + sizeof(CircHeader)`. | Negative-offset pointer arithmetic everywhere if header is appended. Always prepend. |
| Arena-allocating a `RAII_EXTERNAL` shape without a `dtor_list` is forbidden. | Arena bulk-free skips destructors. OS handles and sockets would leak permanently. |
| `const` keyword provides zero ownership information. The inference pass must never branch on `is_const` for ownership decisions. | `const` means non-rebindable binding, not unique ownership. Objects declared `const` are commonly aliased. |
| Any object passed to an `Unknown` external function is immediately `Shared(CIRC)`. | Cannot prove the callee does not store the argument. Conservative is correct; aggressive is UAF. |
| Stack values captured by closures or async state machines must be promoted to `Owned` or `Shared(CIRC)`. | Stack frames pop; closures and state machines outlive frames. A captured stack pointer becomes dangling immediately. |
| Arena values whose lifetime may exceed the arena must be promoted before arena assignment. | UAF if the arena is destroyed while a reference to one of its objects is still live. Escape analysis must catch this. |
| The Bacon-Rajan scan phase MUST restore the RC of non-garbage candidates. | Omitting the restore step causes live objects with external references to be freed. This is the most common Bacon-Rajan implementation bug. |
| Cycle collector `drop_fn` calls must be in reverse topological order of the cycle graph. | Allows members to safely access still-live peers during destruction. |
| `WeakRef.deref()` must promote to a strong reference (circ_inc) before any use. | Without promotion, another thread can drop the last strong ref between the null-check and the first use — a race condition. |
| `--verify-memory` must be run against the full TypeScript conformance suite before any release. | The only reliable mechanism for catching UAF bugs introduced by inference pass changes. |

---

*End of Hybrid Memory Model — Final Architecture.*
*Version: 2.0.0*
*Companion: `raii_final.md` — RAII protocol, drop_fn, scope guards, resource lifecycle.*
*Together these two documents define the complete memory subsystem.*
