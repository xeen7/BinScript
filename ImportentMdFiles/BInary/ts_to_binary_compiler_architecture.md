# TypeScript → Native Binary Compiler: Full Architecture Guide

> **Audience:** An LLM implementing this compiler from scratch in Rust.
> **Goal:** Compile TypeScript (including full npm project trees) into a single self-contained native binary. No runtime shipped. No GC library linked. No V8. No Node.js.

---

## Table of Contents

1. [Guiding Principles](#1-guiding-principles)
2. [Crate Ecosystem — Use These, Don't Reinvent](#2-crate-ecosystem)
3. [Compiler Pipeline Overview](#3-compiler-pipeline-overview)
4. [Phase 1 — Parsing & AST (SWC)](#4-phase-1--parsing--ast-swc)
5. [Phase 2 — Semantic Analysis & Type Erasure](#5-phase-2--semantic-analysis--type-erasure)
6. [Phase 3 — HIR (High-Level IR)](#6-phase-3--hir-high-level-ir)
7. [Phase 4 — Prototype Mechanism (No Runtime)](#7-phase-4--prototype-mechanism-no-runtime)
8. [Phase 5 — MIR (Mid-Level IR) & Monomorphisation](#8-phase-5--mir-mid-level-ir--monomorphisation)
9. [Phase 6 — Generational GC (Compiled-In, Not Linked)](#9-phase-6--generational-gc)
10. [Phase 7 — Lazy JSON Tape](#10-phase-7--lazy-json-tape)
11. [Phase 8 — LLVM Codegen](#11-phase-8--llvm-codegen)
12. [Phase 9 — npm / Project-Scale Compilation](#12-phase-9--npm--project-scale-compilation)
13. [Module Resolution & Bundling Strategy](#13-module-resolution--bundling-strategy)
14. [Incremental Compilation & Caching](#14-incremental-compilation--caching)
15. [Directory Structure](#15-directory-structure)
16. [Scaling Roadmap](#16-scaling-roadmap)
17. [Hard Constraints & Anti-Patterns to Avoid](#17-hard-constraints--anti-patterns-to-avoid)

---

## 1. Guiding Principles

- **Zero shipped runtime.** Every runtime behaviour (GC, prototype chains, closures, iterators, generators, async/await) must be compiled down to plain stack + heap operations and inlined LLVM IR stubs. The final binary must link only libc (or musl for fully static builds).
- **SWC is the front-end, not a custom parser.** SWC's Rust API gives you a battle-tested TS/JS parser, transformer pipeline, and module graph. Use it as the authoritative parse layer.
- **LLVM is the back-end.** Target `inkwell` (safe Rust LLVM bindings). Never emit assembly or object code by hand.
- **Prototype mechanism is a compile-time concern.** JavaScript prototype chains are resolved at compile time where possible, and represented as plain struct vtables in the binary where dynamic dispatch is unavoidable.
- **Generational GC is baked into emitted code.** The collector is a Rust library that gets statically linked and its write-barriers are emitted as LLVM intrinsic calls during codegen. Nothing is shipped as a separate `.so`.
- **Lazy JSON Tape is the default JSON representation.** Raw JSON bytes are kept as a tape (index into a byte slice) until a field is actually accessed, eliminating unnecessary allocation on parse.
- **Scalability by design.** The architecture supports single-file → single module → whole project → npm dependency tree, all through the same pipeline with progressively richer module graphs.

---

## 2. Crate Ecosystem

Use every crate listed here. Do not reimplement what they already do well.

### Parsing & Transformation

| Crate | Version | Role |
|---|---|---|
| `swc_core` | latest | Umbrella re-export for all SWC crates |
| `swc_ecma_parser` | latest (via `swc_core`) | Parse TS/JS → SWC AST |
| `swc_ecma_ast` | latest | SWC AST node types |
| `swc_ecma_transforms_typescript` | latest | Strip TypeScript types → JS AST |
| `swc_ecma_transforms_base` | latest | Helpers, resolver, hygiene |
| `swc_ecma_transforms_module` | latest | CommonJS / ESM interop transforms |
| `swc_ecma_visit` | latest | Visitor/fold traits over SWC AST |
| `swc_common` | latest | SourceMap, Span, globals |
| `swc_node_comments` | latest | Preserve doc comments |

### Module Graph & Resolution

| Crate | Version | Role |
|---|---|---|
| `swc_bundler` | latest | Module graph construction, tree shaking |
| `node_resolve` | latest | Node.js `require()` / `import` resolution |
| `oxc_resolver` | latest | Fast ESM/CJS resolver, alias support |

### LLVM Bindings

| Crate | Version | Role |
|---|---|---|
| `inkwell` | latest | Safe LLVM IR builder |
| `llvm-sys` | matching inkwell | Raw LLVM C bindings (transitive) |

> **LLVM version:** latest.

### Memory & GC

| Crate | Version | Role |
|---|---|---|
| `mmtk` | latest | Production-grade generational GC framework in Rust — this IS your GC |
| `typed-arena` | latest | Fast bump allocation for short-lived compiler-phase objects |
| `bumpalo` | latest | Bump allocator for HIR/MIR arena nodes |

> `mmtk` (Memory Management Toolkit) is the same framework used by JikesRVM and OpenJDK's Epsilon/Shenandoah alternatives. It supports generational collection, write barriers, and integrates with LLVM-emitted code via a thin FFI layer.

### JSON

| Crate | Version | Role |
|---|---|---|
| `sonic-rs` | latest | SIMD-accelerated lazy JSON tape — use this as your tape implementation |
| `simd-json` | latest | Fallback / cross-check |

### Diagnostics & Errors

| Crate | Version | Role |
|---|---|---|
| `miette` | latest | Rich error reporting with source spans |
| `ariadne` | latest | Alternative: pretty diagnostic rendering |
| `thiserror` | latest | Error enum derivation |

### CLI & Build

| Crate | Version | Role |
|---|---|---|
| `clap` | `latest` | CLI argument parsing |
| `rayon` | latest | Parallel compilation of module units |
| `dashmap` | latest | Concurrent HashMap for module cache |
| `petgraph` | latest | Dependency graph, cycle detection, topological sort |
| `serde` + `serde_json` | latest | Config files, serialised IR caching |
| `bincode` | latest | Fast binary serialisation for incremental cache |
| `tracing` | latest | Structured logging throughout pipeline |
| `tempfile` | latest | Temporary object file management |
| `which` | latest | Locate system linker (`lld`, `cc`) |

---

## 3. Compiler Pipeline Overview

```
TypeScript Source Files
        │
        ▼
┌──────────────────┐
│  SWC Parser      │  swc_ecma_parser → SWC AST (with spans & comments)
└──────────────────┘
        │
        ▼
┌──────────────────┐
│  TS Type Strip   │  swc_ecma_transforms_typescript → JS-only AST
└──────────────────┘
        │
        ▼
┌──────────────────┐
│  Module Graph    │  swc_bundler + oxc_resolver → ModuleGraph (petgraph DAG)
└──────────────────┘
        │
        ▼
┌──────────────────┐
│  HIR Lowering    │  SWC AST → compiler's own High-Level IR
└──────────────────┘
        │
        ▼
┌──────────────────┐
│  Semantic Pass   │  Scope resolution, closure capture analysis,
│                  │  prototype shape inference, `this` binding analysis
└──────────────────┘
        │
        ▼
┌──────────────────┐
│  MIR Lowering    │  HIR → three-address Mid-Level IR
│                  │  Prototype chains → vtable descriptors
│                  │  Closures → flat structs
│                  │  async/await → state machines
└──────────────────┘
        │
        ▼
┌──────────────────┐
│  GC Pass         │  Insert write-barrier calls, root registration,
│                  │  safepoint polls via mmtk's compiler interface
└──────────────────┘
        │
        ▼
┌──────────────────┐
│  LLVM IR Emit    │  inkwell → LLVM IR per module unit (parallel, rayon)
└──────────────────┘
        │
        ▼
┌──────────────────┐
│  LTO + Optimise  │  LLVM LTO (thin or full), O2/O3, dead-code elimination
└──────────────────┘
        │
        ▼
┌──────────────────┐
│  Link            │  lld (preferred) or system linker
│                  │  Static link: musl libc + mmtk GC stubs
└──────────────────┘
        │
        ▼
  Native Binary (ELF / Mach-O / PE)
```

---

## 4. Phase 1 — Parsing & AST (SWC)

### Setup

```rust
use swc_core::{
    common::{SourceMap, sync::Lrc, Globals, GLOBALS, Mark},
    ecma::{
        ast::*,
        parser::{lexer::Lexer, Parser, StringInput, Syntax, TsSyntax},
        transforms::{
            base::{resolver, hygiene, fixer},
            typescript::strip,
        },
        visit::FoldWith,
    },
};

pub fn parse_and_strip(src: &str, file_name: &str) -> Module {
    let cm: Lrc<SourceMap> = Default::default();
    let fm = cm.new_source_file(
        swc_common::FileName::Custom(file_name.into()),
        src.into(),
    );

    let lexer = Lexer::new(
        Syntax::Typescript(TsSyntax {
            tsx: file_name.ends_with(".tsx"),
            decorators: true,
            ..Default::default()
        }),
        EsVersion::Es2022,
        StringInput::from(&*fm),
        None,
    );

    let mut parser = Parser::new_from(lexer);
    let module = parser.parse_module().expect("parse error");

    GLOBALS.set(&Globals::new(), || {
        let unresolved_mark = Mark::new();
        let top_level_mark = Mark::new();

        module
            .fold_with(&mut resolver(unresolved_mark, top_level_mark, true))
            .fold_with(&mut strip(top_level_mark))          // remove TS types
            .fold_with(&mut hygiene())
            .fold_with(&mut fixer(None))
    })
}
```

### Key Notes

- Always run `resolver` before `strip`. SWC's type-stripping pass depends on hygiene marks set by the resolver.
- `tsx: true` must be toggled per-file based on extension, not globally.
- Preserve the `SourceMap` (`Lrc<SourceMap>`) for the entire compilation session — you need it for error spans and debug info (DWARF).
- SWC spans are byte offsets, not line/column. Convert lazily using `SourceMap::lookup_char_pos` only when emitting diagnostics.

---

## 5. Phase 2 — Semantic Analysis & Type Erasure

After SWC strips types, you are working with a fully valid ES2022 AST. You need to build your own semantic layer on top:

### Scope Analysis

Implement a scope tree using `swc_ecma_visit::Visit`. For each `Module`, `Function`, `ArrowExpr`, `BlockStmt`, build a `Scope` node:

```
Scope {
    kind: Module | Function | Block | Class,
    bindings: HashMap<JsWord, BindingInfo>,
    parent: Option<ScopeId>,
    captures: Vec<BindingId>,   // populated during closure analysis
}
```

Use `bumpalo` or `typed-arena` for scope tree allocation — the entire scope forest is discarded after HIR lowering.

### Closure Capture Analysis

Walk every `ArrowExpr` and `Function`. For each identifier reference, walk parent scopes. If the binding lives in a non-enclosing function scope, mark it as captured. This drives the closure struct layout emitted in MIR.

### `this` Binding Analysis

Track `this` context per function boundary:
- Arrow functions: inherit `this` from enclosing scope (captured as a field).
- Regular functions: `this` is a parameter (dynamic at call site, or bound via `.call`/`.apply`/`.bind`).
- Class methods: `this` is the receiver (first parameter in the emitted ABI).

### Prototype Shape Inference

This is the most important semantic pass. See Phase 4 for the full mechanism. At this stage, collect:

- All `class` declarations → `ClassShape { fields: Vec<FieldDef>, methods: Vec<MethodDef>, super_class: Option<ClassId> }`
- All object literals used as prototypes (`Object.create`, `__proto__` assignments, `prototype` property assignments on functions)
- All `new Foo()` call sites → link to `ClassShape`

Build a `ShapeTable: HashMap<ShapeId, ClassShape>`. This is your compile-time prototype registry.

---

## 6. Phase 3 — HIR (High-Level IR)

HIR is a simplified, desugared representation that stays close to JS semantics but removes SWC's AST noise.

```rust
pub enum HirExpr {
    Lit(Literal),
    Var(BindingId),
    BinOp(BinOp, Box<HirExpr>, Box<HirExpr>),
    UnaryOp(UnaryOp, Box<HirExpr>),
    Call { callee: Box<HirExpr>, args: Vec<HirExpr>, this: Option<Box<HirExpr>> },
    New { class: ShapeId, args: Vec<HirExpr> },
    MemberGet { obj: Box<HirExpr>, prop: PropKey },
    MemberSet { obj: Box<HirExpr>, prop: PropKey, val: Box<HirExpr> },
    Closure { captures: Vec<BindingId>, func: FuncId },
    ArrayLit(Vec<HirExpr>),
    ObjectLit { shape: ShapeId, fields: Vec<(PropKey, HirExpr)> },
    Await(Box<HirExpr>),
    Yield(Option<Box<HirExpr>>),
    Ternary { cond: Box<HirExpr>, then: Box<HirExpr>, else_: Box<HirExpr> },
    TaggedTemplate { tag: Box<HirExpr>, parts: Vec<HirExpr> },
    Seq(Vec<HirExpr>),                           // comma expression
    JsonTape { raw_bytes: Arc<[u8]> },           // lazy JSON value
}

pub enum HirStmt {
    Expr(HirExpr),
    Let { binding: BindingId, init: Option<HirExpr> },
    Assign { target: HirLValue, value: HirExpr },
    If { cond: HirExpr, then: Vec<HirStmt>, else_: Option<Vec<HirStmt>> },
    Loop { label: Option<Label>, body: Vec<HirStmt> },
    For { label: Option<Label>, init: Option<Box<HirStmt>>, cond: Option<HirExpr>, update: Option<HirExpr>, body: Vec<HirStmt> },
    ForOf { binding: BindingId, iter: HirExpr, body: Vec<HirStmt> },
    Return(Option<HirExpr>),
    Break(Option<Label>),
    Continue(Option<Label>),
    Throw(HirExpr),
    TryCatch { body: Vec<HirStmt>, catch: Option<CatchClause>, finally: Option<Vec<HirStmt>> },
    Switch { disc: HirExpr, cases: Vec<SwitchCase> },
    Block(Vec<HirStmt>),
}
```

### Desugaring Rules (SWC AST → HIR)

| SWC construct | HIR equivalent |
|---|---|
| `for...of` over array | `ForOf` with iterator protocol inlined |
| `for...of` over `Map`/`Set` | `ForOf` dispatched through iterator vtable |
| Template literals | `TaggedTemplate` or string concat sequence |
| `?.` optional chaining | Ternary + temp var null check |
| `??` nullish coalescing | Ternary with `=== null \|\| === undefined` check |
| `&&=`, `\|\|=`, `??=` | Assign + short-circuit Ternary |
| `async function` | Desugar to generator state machine (see below) |
| `await expr` | `Yield`-resume point in state machine |
| `function*` | State machine struct + `HirExpr::Yield` |
| Destructuring | Sequence of `Let` + `MemberGet` |
| Spread in array | `Array.concat` or push-loop HIR sequence |
| Spread in object | `Object.assign` equivalent HIR |
| `typeof` | Intrinsic call `__typeof(val)` |
| `instanceof` | `__instanceof(val, ShapeId)` intrinsic |
| `in` operator | `__has_property(obj, key)` intrinsic |
| `delete obj.x` | `__delete_property(obj, key)` intrinsic |
| `arguments` | Capture as array in function prologue |

---

## 7. Phase 4 — Prototype Mechanism (No Runtime)

This is the centrepiece of the design. The full JavaScript prototype chain must be implemented without shipping any runtime. The mechanism works in three stages: shape inference, vtable emission, and dynamic fallback.

### 7.1 Compile-Time Shape Inference

Every `class` declaration and every `function` used as a constructor produces a `Shape`:

```rust
pub struct Shape {
    pub id: ShapeId,
    pub name: Option<String>,
    pub parent: Option<ShapeId>,          // super class
    pub own_fields: Vec<FieldDef>,        // instance properties defined in constructor
    pub methods: IndexMap<String, FuncId>,// own methods
    pub vtable: Vec<VTableSlot>,          // flattened, including inherited
    pub static_props: IndexMap<String, HirExpr>,
    pub prototype_shape: Option<ShapeId>, // the shape of `.prototype` object
}

pub struct VTableSlot {
    pub name: String,
    pub func: FuncId,
    pub is_getter: bool,
    pub is_setter: bool,
}
```

Build `vtable` by walking the inheritance chain bottom-up: start with the deepest subclass and merge parent vtables, overriding slots where the subclass defines a method with the same name.

### 7.2 LLVM Struct Layout

Each shape maps to an LLVM struct:

```
%MyClass = type {
    ptr,        ; *VTable  — always slot 0
    <field_0>,
    <field_1>,
    ...
}

%MyClass_VTable = type {
    ptr,        ; *ParentVTable or null
    ptr,        ; *type_name_str  (for typeof/instanceof)
    i64,        ; shape_id
    ptr,        ; *toString_fn
    ptr,        ; *valueOf_fn
    ptr,        ; *method_A_fn
    ptr,        ; *method_B_fn
    ...
}
```

The vtable struct is emitted as a `global constant` in LLVM IR. No heap allocation. No runtime registration.

```rust
// In inkwell codegen:
let vtable_ty = context.struct_type(&[
    ptr_ty,  // parent vtable
    ptr_ty,  // type name string
    i64_ty,  // shape id
    // ... one ptr per method
], false);

let vtable_global = module.add_global(vtable_ty, Some(AddressSpace::default()), "MyClass_vtable");
vtable_global.set_constant(true);
vtable_global.set_initializer(&vtable_const_value);
```

### 7.3 Property Access Compilation

**Static dispatch (known shape at call site):**

```typescript
const p = new Point(1, 2);
p.toString();
```

Compiles to a direct LLVM `call` — no vtable lookup, no indirection.

**Dynamic dispatch (unknown shape, known method name):**

```typescript
function render(shape: any) { shape.draw(); }
```

Compiles to:
1. Load `vtable_ptr = load ptr from obj[0]`
2. Load `fn_ptr = load ptr from vtable_ptr[slot_for_"draw"]`
3. `call fn_ptr(obj, ...args)`

The slot index for `"draw"` is resolved at compile time by scanning all known shapes that implement `draw` and asserting they use a consistent vtable layout. If slot layout conflicts, emit a slow-path: a linear scan through a flat property list embedded in the vtable.

**Property access on plain objects (non-class):**

Plain object literals are represented as `HashMap<String, JsValue>` structs on the heap, addressed through a generic `JsObject` type:

```
%JsObject = type {
    ptr,    ; *VTable (points to JsObject_vtable, a generic object vtable)
    ptr,    ; *PropertyStore (pointer to inline or heap property array)
    i64,    ; property_count
}
```

### 7.4 Prototype Chain Lookup (Compiled)

`Object.getPrototypeOf`, `__proto__` access, and `instanceof` all compile to:

```rust
// Emitted as a plain LLVM function — statically linked, not a runtime:
fn js_instanceof(obj: *const JsObject, shape_id: u64) -> bool {
    let mut vtable = obj.vtable;
    loop {
        if vtable.shape_id == shape_id { return true; }
        if vtable.parent.is_null() { return false; }
        vtable = vtable.parent;
    }
}
```

This is emitted once as an LLVM function, LTO'd, and inlined at call sites where the compiler can prove `shape_id` is a constant.

### 7.5 `Object.create`, `Object.assign`, Property Descriptors

- `Object.create(proto)` → allocate a `JsObject`, set vtable pointer to `proto`'s vtable. Emitted as an inline LLVM sequence, no runtime call.
- `Object.assign(target, ...sources)` → emit a loop over the source property stores using the compiled `PropertyStore` API.
- Property descriptors (`Object.defineProperty`) with `get`/`set` → the getter/setter pair are stored as function pointers in the property store, with a flag bit indicating they are accessors. The property load/store path checks this flag and branches.

### 7.6 Dynamic Property Addition (Post-Construction)

JavaScript allows adding properties to objects after construction. Support this with a two-level property store:

```
PropertyStore {
    inline_slots: [PropertySlot; 8],   // fast path: first 8 props
    overflow: Option<Box<HashMap<JsWord, PropertySlot>>>,
}
```

The inline slots are a fixed-size array allocated as part of the object. Overflow spills to a heap HashMap. Write barriers (Phase 6) wrap every store to either path.

---

## 8. Phase 5 — MIR (Mid-Level IR) & Monomorphisation

MIR is a three-address, SSA-like IR that maps closely to LLVM IR but stays typed at the JS semantic level.

```rust
pub enum MirInstr {
    // Arithmetic
    Add(MirReg, MirOperand, MirOperand),
    Sub(MirReg, MirOperand, MirOperand),
    // ...

    // JS coercions
    ToNumber(MirReg, MirOperand),
    ToString(MirReg, MirOperand),
    ToBool(MirReg, MirOperand),

    // Object operations
    Alloc(MirReg, ShapeId),                        // new + malloc
    LoadField(MirReg, MirOperand, FieldIdx),
    StoreField(MirOperand, FieldIdx, MirOperand),  // triggers write barrier
    LoadProp(MirReg, MirOperand, PropKey),          // dynamic lookup
    StoreProp(MirOperand, PropKey, MirOperand),

    // Calls
    CallDirect(MirReg, FuncId, Vec<MirOperand>),
    CallVTable(MirReg, MirOperand, VTableSlot, Vec<MirOperand>),
    CallDynamic(MirReg, MirOperand, Vec<MirOperand>), // call through fn ptr

    // Control flow
    Branch(MirOperand, BlockId, BlockId),
    Jump(BlockId),
    Return(Option<MirOperand>),
    Throw(MirOperand),

    // GC
    WriteBarrier(MirOperand, MirOperand),          // barrier(parent_obj, child_obj)
    SuspendPoint,                                  // GC safepoint poll

    // Async state machine
    Suspend(u32),                                  // yield point index
    Resume(MirReg, u32),                           // resume value + point

    // JSON tape
    TapeGet(MirReg, MirOperand, PropKey),          // lazy field access into tape
    TapeParse(MirReg, MirOperand),                 // force full parse
}
```

### Async/Await State Machine

Every `async function` is transformed into a struct + a `poll` function:

```
AsyncState_myFn {
    state: u32,          // current suspension point index
    <captured locals>    // all live locals at each yield point
    future_slot: *mut JsObject,  // pending awaited future
}

fn myFn_poll(state: *mut AsyncState_myFn, waker: *const Waker) -> PollResult
```

The transformation happens at MIR level, not at HIR. Each `Suspend(n)` becomes a `switch` on `state_ptr.state` in the emitted LLVM IR. No heap allocations for the state machine itself — callers allocate the `AsyncState` struct on the stack (or as a GC-managed object if the future escapes).

### Closure Lowering

Each closure is a pair:

```
ClosureStruct_<id> {
    fn_ptr: ptr,              // pointer to the flat closure function
    <capture_0>: T0,
    <capture_1>: T1,
    ...
}
```

The captured variables are either:
- Copied by value if they are never reassigned after capture.
- Stored behind a `CaptureCell<T>` (a single-element GC-managed box) if they are mutable and shared across closures.

---

## 9. Phase 6 — Generational GC

### Using `mmtk`

`mmtk-core` is the GC framework. It provides:
- Generational collection (minor/major)
- Write barrier API
- Safepoint API
- Object scanning interface (you implement `ObjectModel` and `Scanning` traits)

```toml
[dependencies]
mmtk = { git = "https://github.com/mmtk/mmtk-core", features = ["gen_immix"] }
```

Use the `GenImmix` plan: generational collection with the Immix tracing collector for the old generation. This is a production-proven configuration used in JikesRVM.

### Implementing `ObjectModel`

```rust
use mmtk::util::ObjectReference;
use mmtk::vm::ObjectModel;

// Every GC-managed JS object has this 8-byte header prepended:
#[repr(C)]
pub struct JsObjectHeader {
    pub gc_word: usize,   // mmtk's GC metadata word (forwarding pointer, mark bit, etc.)
    pub vtable: *const VTable,
}

impl ObjectModel<JsRuntime> for JsObjectModel {
    const GC_BYTE_OFFSET: usize = 0;  // gc_word is at offset 0
    // ...
}
```

### Write Barriers

Every `StoreField` and `StoreProp` in MIR emits a write barrier. In LLVM IR codegen this becomes:

```rust
// In inkwell codegen, after every heap store:
let barrier_fn = module.get_function("mmtk_object_reference_write_post")
    .expect("mmtk barrier must be declared");
builder.build_call(barrier_fn, &[parent_obj.into(), child_obj.into()], "wb");
```

Declare the barrier as an `extern "C"` LLVM function. The actual implementation is in the `mmtk` static library that you link at final link time.

### Safepoints

Every backward branch (loop back-edge) and every function prologue emits a safepoint poll:

```rust
// MirInstr::SuspendPoint → in codegen:
let poll_fn = module.get_function("mmtk_safepoint_poll").expect("mmtk safepoint");
builder.build_call(poll_fn, &[], "safepoint");
```

### Root Registration

The GC needs to know which stack slots and globals are GC roots. Implement LLVM's `gc.root` statepoint mechanism via inkwell's `gc` intrinsics, or use a shadow stack: maintain a thread-local linked list of active frames, each containing a bitmap of which slots hold pointers.

The shadow stack approach is simpler to implement correctly and is what most LLVM-based language runtimes use for correctness before switching to statepoints.

### Static Linking the GC

```toml
# build.rs
fn main() {
    println!("cargo:rustc-link-lib=static=mmtk");
    println!("cargo:rustc-link-search=native=./target/mmtk_build");
}
```

Build mmtk as a static library (`libmmtk.a`) during the compiler's build step. The emitted binary links it statically. Nothing is shipped separately.

---

## 10. Phase 7 — Lazy JSON Tape

### Default Representation

When a JSON string literal or a `JSON.parse(str)` call is encountered, do not immediately parse it into a `JsObject` heap structure. Instead represent it as:

```rust
pub struct JsonTape {
    pub raw: Arc<[u8]>,              // original UTF-8 bytes
    pub tape: Option<sonic_rs::Tape>, // None until first access
    pub state: TapeState,
}

pub enum TapeState {
    Raw,                             // bytes not yet touched
    Indexed,                         // tape index built, fields accessible
    Parsed,                          // fully materialised into JsObject tree
}
```

Use `sonic-rs` for the tape index. Its `to_tape` API builds a flat index over the raw bytes without allocating per-field objects.

### Integration in MIR

`TapeGet(dest, tape_operand, key)` compiles to:
1. Check `tape.state`: if `Raw`, call `sonic_rs::to_tape` and transition to `Indexed`.
2. Use `sonic_rs` tape navigation to find `key` in O(1) or O(depth) without full materialisation.
3. Return the value as a `JsValue` that itself may be a nested tape.

`TapeParse(dest, tape_operand)` forces full materialisation: walk the tape and allocate the full `JsObject` tree via the GC allocator. Use this only when the object must be mutated or passed to a context that requires a live `JsObject`.

### LLVM IR

Both `TapeGet` and `TapeParse` compile to `CallDirect` into small Rust helper functions that are compiled into the binary via the same static linking strategy as mmtk. These functions live in a `ts_rt_stubs` crate that is part of your compiler workspace and is statically linked into every emitted binary.

```
ts_rt_stubs/
  src/
    json_tape.rs     <- sonic_rs integration
    prototype.rs     <- __instanceof, __typeof, __has_property
    property_store.rs
    closures.rs
    async_poll.rs
    iterators.rs
    coercions.rs     <- ToNumber, ToString, ToBool
```

This is the only "runtime" that exists — it is a static Rust library, compiled once, LTO'd into the final binary, with dead code eliminated by the linker. It is not a shipped runtime; it is a compiler-managed support library like `compiler-rt` in LLVM.

---

## 11. Phase 8 — LLVM Codegen

### Module Strategy

Emit one LLVM `Module` per source file (or per logical compilation unit in a large project). This enables:
- Parallel codegen with `rayon`
- Incremental compilation (skip modules whose content hash hasn't changed)
- Thin LTO across module boundaries

### Value Representation

JavaScript is dynamically typed. Represent `JsValue` as a NaN-boxed 64-bit value:

```
NaN-boxed JsValue (64 bits):
  ┌──────────────────────────────────────────────────┐
  │ If all exponent bits set + quiet NaN bit:        │
  │   [48-bit payload]   [4-bit tag]                 │
  │   tag = 0001 → undefined                         │
  │   tag = 0010 → null                              │
  │   tag = 0011 → false                             │
  │   tag = 0100 → true                              │
  │   tag = 0101 → integer (32-bit in payload)       │
  │   tag = 0110 → pointer to JsObject               │
  │   tag = 0111 → pointer to JsString (interned)    │
  │   tag = 1000 → pointer to JsonTape               │
  │   tag = 1001 → pointer to Closure                │
  │   tag = 1010 → Symbol (interned id in payload)   │
  └──────────────────────────────────────────────────┘
  Otherwise: IEEE 754 double (number)
```

NaN-boxing means `JsValue` is always exactly 64 bits. No boxing overhead for doubles. Pointer extraction is a bitmask operation. This is the same representation used by WebKit JavaScriptCore and LuaJIT.

In LLVM IR, `JsValue` is `i64`. Emit helper functions for tagging/untagging.

### String Representation

Use an interned string table:

```rust
pub struct JsString {
    pub hash: u64,
    pub len: u32,
    pub bytes: *const u8,   // pointer into GC-managed string heap
}
```

Short strings (≤ 7 bytes) are stored inline in the NaN-box payload. Longer strings are GC-managed heap objects. String interning is done via a global `DashMap<u64, NonNull<JsString>>`.

### inkwell Codegen Skeleton

```rust
use inkwell::{context::Context, module::Module, builder::Builder, values::*, types::*};

pub struct LlvmCodegen<'ctx> {
    pub ctx: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    pub i64_ty: IntType<'ctx>,
    pub ptr_ty: PointerType<'ctx>,
    pub js_value_ty: IntType<'ctx>,  // i64, NaN-boxed
    pub func_table: HashMap<FuncId, FunctionValue<'ctx>>,
    pub vtable_globals: HashMap<ShapeId, GlobalValue<'ctx>>,
    pub string_table: HashMap<JsWord, GlobalValue<'ctx>>,
}

impl<'ctx> LlvmCodegen<'ctx> {
    pub fn emit_function(&mut self, func: &MirFunction) -> FunctionValue<'ctx> { ... }
    pub fn emit_instr(&mut self, instr: &MirInstr) -> Option<BasicValueEnum<'ctx>> { ... }
    pub fn emit_alloc(&mut self, shape_id: ShapeId) -> PointerValue<'ctx> { ... }
    pub fn emit_write_barrier(&mut self, parent: PointerValue<'ctx>, child: PointerValue<'ctx>) { ... }
    pub fn emit_vtable_lookup(&mut self, obj: BasicValueEnum<'ctx>, slot: usize) -> PointerValue<'ctx> { ... }
}
```

### Optimisation Pipeline

```rust
use inkwell::passes::{PassManager, PassManagerBuilder};

let pmb = PassManagerBuilder::create();
pmb.set_optimization_level(OptimizationLevel::Aggressive); // O3
pmb.set_inliner_with_threshold(275);

let pm: PassManager<Module> = PassManager::create(());
pmb.populate_module_pass_manager(&pm);
pm.run_on(&llvm_module);
```

For whole-program LTO, emit bitcode (`.bc`) per module and invoke `llvm-lto` or `lld`'s built-in LTO via the linker plugin.

---

## 12. Phase 9 — npm / Project-Scale Compilation

### Module Graph Construction

Use `swc_bundler` combined with a custom `Resolve` implementation backed by `oxc_resolver`:

```rust
struct NpmResolver {
    root: PathBuf,
    node_modules: Vec<PathBuf>,
    tsconfig_paths: HashMap<String, Vec<PathBuf>>,
}

impl swc_bundler::Resolve for NpmResolver {
    fn resolve(&self, base: &FileName, module_specifier: &str) -> Result<FileName> {
        // 1. Check tsconfig paths aliases
        // 2. Check relative path
        // 3. Walk node_modules using oxc_resolver
    }
}
```

### petgraph Module DAG

```rust
use petgraph::graph::{DiGraph, NodeIndex};

pub struct ModuleGraph {
    pub graph: DiGraph<ModuleNode, ImportEdge>,
    pub entry: NodeIndex,
    pub index: HashMap<PathBuf, NodeIndex>,
}

pub struct ModuleNode {
    pub path: PathBuf,
    pub source_hash: u64,
    pub hir: Option<Arc<Vec<HirStmt>>>,
    pub llvm_bc: Option<Vec<u8>>,   // cached bitcode
}
```

Detect circular dependencies with `petgraph::algo::is_cyclic_directed`. Topological sort with `petgraph::algo::toposort` gives you the compilation order.

### Tree Shaking

Leverage `swc_bundler`'s built-in tree shaking: it tracks `export` usage across the module graph and eliminates unreachable exports before you even see the AST. This dramatically reduces the amount of HIR/MIR you generate for large npm packages.

### npm Package Handling

Most npm packages are either:
1. **Pure JS/TS** — compile normally through the pipeline.
2. **Native addons (`.node` files)** — these cannot be compiled to binary. Detect them during resolution and emit a compile error with a suggestion to use a pure-JS alternative.
3. **Node.js built-ins (`fs`, `path`, `os`, etc.)** — provide a `node-compat` shim layer in `ts_rt_stubs` that implements the most common APIs using libc/libuv calls. Mark unsupported APIs as compile-time errors.

### Single Executable Output

The final link step combines:
1. All LLVM bitcode modules (post-LTO)
2. `libts_rt_stubs.a` (your support stubs)
3. `libmmtk.a` (GC)
4. `libsonic_rs.a` or the relevant parts compiled from your stubs
5. musl libc (for fully static `--target x86_64-unknown-linux-musl`)

```bash
lld \
  --lto-O3 \
  -static \
  entry.o module_a.bc module_b.bc ... \
  -l:libts_rt_stubs.a \
  -l:libmmtk.a \
  -l:libc.a \
  -o my_program
```

---

## 13. Module Resolution & Bundling Strategy

### Resolution Priority (per import specifier)

1. `tsconfig.json` `paths` aliases → absolute path rewrite
2. Relative path (`./foo`, `../bar`) → resolve relative to current file
3. Bare specifier (`lodash`, `react`) →
   a. Check `node_modules/.pnpm` / `node_modules` walking up the directory tree
   b. Honour `exports` field in `package.json` (ESM conditional exports)
   c. Honour `main`/`module` fields as fallback
4. Node built-ins (`node:fs`, `fs`) → shim layer in `ts_rt_stubs`

### `package.json` `exports` Field

Use `oxc_resolver`'s `exports` resolution — it correctly handles:
- Condition matching (`import`, `require`, `browser`, `node`, `default`)
- Subpath patterns (`"./utils/*": "./dist/utils/*.js"`)
- Fallback arrays

### Deduplication

Multiple modules importing the same package version should compile to a single LLVM module for that package. Use the resolved absolute path as the canonical key in `ModuleGraph`. `swc_bundler`'s deduplication handles this automatically within a single bundling session.

---

## 14. Incremental Compilation & Caching

### Per-Module Cache

Key: `SHA-256(source_bytes + compiler_version + flags)`
Value: `bincode`-serialised `{ hir: Vec<HirStmt>, mir: Vec<MirFunction>, llvm_bc: Vec<u8> }`

Store in `~/.cache/tscompiler/<project_hash>/` (or `.tscompiler_cache/` in the project root for CI reproducibility).

### What to Invalidate

- Source file changed → recompile that module + all dependents (use the petgraph reverse edge set)
- Compiler version changed → full recompile
- tsconfig changed → full recompile
- Only a dependency changed → recompile dependent modules but reuse their `.bc` files if the module's exported interface (shape table + function signatures) hasn't changed

### Parallel Compilation

```rust
use rayon::prelude::*;

let sorted_modules = toposort(&graph).expect("cycle detected");

// Compile independent layers in parallel:
sorted_modules
    .par_iter()
    .for_each(|module_id| {
        compile_module(module_id, &cache, &shape_table);
    });
```

Modules with no dependencies on other in-progress modules can be compiled in parallel. Use `DashMap` for the shared shape table and cache.

---

## 15. Directory Structure

```
ts-compiler/
├── Cargo.toml                  # workspace root
├── crates/
│   ├── compiler-core/          # orchestration, CLI, pipeline driver
│   │   └── src/
│   │       ├── main.rs
│   │       ├── pipeline.rs
│   │       └── config.rs
│   ├── parser/                 # SWC integration, Phase 1
│   ├── semantic/               # scope analysis, shape inference, Phase 2
│   ├── hir/                    # HIR types + SWC→HIR lowering, Phase 3
│   ├── prototype/              # shape table, vtable builder, Phase 4
│   ├── mir/                    # MIR types + HIR→MIR lowering, Phase 5
│   ├── gc-pass/                # write barrier insertion, safepoints, Phase 6
│   ├── json-tape/              # tape IR + sonic-rs integration, Phase 7
│   ├── codegen-llvm/           # inkwell codegen, Phase 8
│   ├── module-graph/           # petgraph DAG, resolution, Phase 9
│   ├── incremental/            # cache, hashing, invalidation
│   └── diagnostics/            # miette integration, error types
└── rt-stubs/                   # ts_rt_stubs static library
    └── src/
        ├── lib.rs
        ├── json_tape.rs
        ├── prototype.rs
        ├── property_store.rs
        ├── coercions.rs
        ├── closures.rs
        ├── async_poll.rs
        ├── iterators.rs
        ├── node_compat/
        │   ├── fs.rs
        │   ├── path.rs
        │   └── os.rs
        └── gc_stubs.rs         # mmtk extern declarations
```

---

## 16. Scaling Roadmap

### Stage 1 — Single File

- SWC parse + type strip
- HIR lowering (no closures, no classes, no async)
- MIR lowering (arithmetic, basic control flow, function calls)
- LLVM codegen (NaN-boxed values, basic string ops)
- Link with musl → working binary

### Stage 2 — Classes & Prototypes

- Shape inference pass
- Vtable emission
- `new`, `instanceof`, method dispatch
- Static property access optimisation

### Stage 3 — Closures & Captures

- Capture analysis
- Closure struct emission
- `CaptureCell` for mutable captures

### Stage 4 — Async/Await & Generators

- State machine transformation in MIR
- `Promise` implementation in `rt-stubs`
- Microtask queue (libc-based event loop in `rt-stubs`, no libuv)

### Stage 5 — GC Integration

- mmtk integration
- Write barriers, safepoints
- Shadow stack root tracking
- Generational collection tuning

### Stage 6 — JSON Tape

- `sonic-rs` tape integration
- `TapeGet` / `TapeParse` codegen
- Lazy materialisation path

### Stage 7 — Module Graph & npm

- `swc_bundler` + `oxc_resolver`
- petgraph DAG
- Tree shaking
- Incremental cache

### Stage 8 — Full npm Project

- `package.json` `exports` resolution
- Node built-in shims
- Parallel codegen with rayon
- Thin LTO across modules
- Single executable link

---

## 17. Hard Constraints & Anti-Patterns to Avoid

| Anti-pattern | Why | What to do instead |
|---|---|---|
| Shipping a JS interpreter | Defeats the entire purpose | All semantics must be compiled to LLVM IR |
| Using `libnode` or V8 as a backend | Brings a massive runtime | Use inkwell → LLVM directly |
| Emitting a GC as a `.so` / `.dylib` | Runtime dependency | Statically link `libmmtk.a` |
| Emitting `eval()` or `Function()` support | Cannot compile dynamic code at runtime | Emit a compile error; these are not supported |
| Using `WidthType.PERCENTAGE` in docx | (Not relevant here) | — |
| Parsing all JSON eagerly | Massive allocation spike | Default to `JsonTape`; materialise only on mutation |
| Typed LLVM pointers | Deprecated in LLVM 15+ | Always use opaque `ptr` type |
| Separate vtable per instance | Massive memory waste | Vtable is a single global constant shared by all instances of a shape |
| Rewriting `oxc_resolver` or `swc_bundler` | Huge complexity, already solved | Use them as-is |
| Implementing GC from scratch | Extremely complex to get correct | Use `mmtk` with `GenImmix` plan |
| Synchronous `require()` at runtime | Requires a loader at runtime | All modules are resolved and compiled statically |
| Using `HashMap` with `String` keys for property access in hot paths | Perf | Use shape-indexed struct fields for known-shape objects; `HashMap` only for truly dynamic objects |

---

*End of architecture document.*
*Version: 1.0.0 — Generated for LLM compiler implementation guidance.*
