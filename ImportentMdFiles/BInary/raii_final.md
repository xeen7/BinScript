# RAII Integration — Final Architecture

> **Cross-cutting protocol on top of all four memory layers (Stack, Arena, Owned, CIRC).**
> RAII governs *when and how* destructors are called — deterministically, scope-tied,
> with zero runtime involvement and zero developer-facing annotations required.
> Every piece of RAII is inferred by the compiler from existing TypeScript and JavaScript
> patterns. A codebase written in 2012 ES5-style JavaScript compiles with full RAII
> resource management. The developer changes nothing.

---

## Table of Contents

1. [What RAII Is — and What It Is Not](#1-what-raii-is--and-what-it-is-not)
2. [Why RAII Is Necessary](#2-why-raii-is-necessary)
3. [The RAII Protocol: Two Components](#3-the-raii-protocol-two-components)
4. [Component 1 — The Drop Trait (Vtable `drop_fn`)](#4-component-1--the-drop-trait-vtable-drop_fn)
5. [Component 2 — Scope Guards and Destruction Order](#5-component-2--scope-guards-and-destruction-order)
6. [The Four Sources of RAII Signal](#6-the-four-sources-of-raii-signal)
7. [Source 1 — Resource Type Recognition](#7-source-1--resource-type-recognition)
8. [Source 2 — Lifecycle Pattern Matching](#8-source-2--lifecycle-pattern-matching)
9. [Source 3 — Control Flow Graph Analysis and DRA](#9-source-3--control-flow-graph-analysis-and-dra)
10. [Source 4 — `try/finally` and Promise `.finally()` as RAII Intent](#10-source-4--tryfinally-and-promise-finally-as-raii-intent)
11. [The Resource Descriptor Table (RDT)](#11-the-resource-descriptor-table-rdt)
12. [The Lifecycle Inference Pass](#12-the-lifecycle-inference-pass)
13. [Implicit Scope Guard Insertion](#13-implicit-scope-guard-insertion)
14. [Conflict Resolution: Explicit Manual Release + Implicit RAII](#14-conflict-resolution-explicit-manual-release--implicit-raii)
15. [Legacy Pattern Catalogue](#15-legacy-pattern-catalogue)
16. [RAII Across All Four Memory Layers](#16-raii-across-all-four-memory-layers)
17. [RAII and ASAP: The Relationship](#17-raii-and-asap-the-relationship)
18. [Exception Safety and `try/catch` Under RAII](#18-exception-safety-and-trycatch-under-raii)
19. [RAII and Async/Await](#19-raii-and-asyncawait)
20. [RAII and Closures](#20-raii-and-closures)
21. [RAII and CIRC: Interaction Protocol](#21-raii-and-circ-interaction-protocol)
22. [Handling Conditional Resource Acquisition](#22-handling-conditional-resource-acquisition)
23. [Handling Resources Stored in Objects](#23-handling-resources-stored-in-objects)
24. [Handling Resources Passed Across Functions](#24-handling-resources-passed-across-functions)
25. [Global and Module-Level Resources](#25-global-and-module-level-resources)
26. [Safety: When the Compiler Cannot Prove Lifecycle](#26-safety-when-the-compiler-cannot-prove-lifecycle)
27. [MIR Instruction Set for RAII](#27-mir-instruction-set-for-raii)
28. [LLVM IR Emission for RAII](#28-llvm-ir-emission-for-raii)
29. [The Complete Memory Model Spectrum](#29-the-complete-memory-model-spectrum)
30. [Full Directory and Crate Structure](#30-full-directory-and-crate-structure)
31. [Hard Constraints](#31-hard-constraints)

---

## 1. What RAII Is — and What It Is Not

**RAII** stands for *Resource Acquisition Is Initialisation*. The concept is better
described by its inverse: **Resource Release Is Destruction**.

A resource — heap memory, file handle, mutex lock, network socket, database
transaction, GPU buffer — is tied to the lifetime of an object. When the object is
constructed the resource is acquired. When the object is destroyed — deterministically,
at a known point — the resource is released. No manual `close()`, no `finally` block,
no `dispose()` convention required. The destruction call *is* the release.

**What RAII is NOT:**

- It is not garbage collection. GC is non-deterministic. RAII is deterministic.
- It is not `try/finally`. `finally` is a programmer responsibility that can be
  forgotten or omitted on certain code paths. RAII is enforced by the compiler via
  destructor insertion on every path, unconditionally.
- It is not the same as ASAP destruction. ASAP decides *when* to free memory. RAII
  decides *what to do* at that moment — the destructor protocol. They are complementary
  and both are needed.
- It is not limited to memory. Memory is one resource. RAII manages any resource
  with an acquire/release lifecycle.

**The canonical RAII guarantee:** a destructor is called **exactly once**, at the end
of the object's lifetime, on every code path — including early returns, thrown
exceptions, `break`, and `continue`.

---

## 2. Why RAII Is Necessary

The four-layer memory model (Stack + Arena + Owned + CIRC) handles *memory* correctly.
But memory is not the only resource:

```javascript
const conn = await db.connect();       // acquires a DB connection
const lock = await mutex.acquire();    // acquires a mutex
const fd   = fs.openSync('file','r'); // acquires a file descriptor
const tx   = db.beginTransaction();   // begins a DB transaction

// ... 200 lines of logic ...

// What guarantees these are released if an exception is thrown at line 100?
// Without RAII: nothing. You need a finally block on every path.
// With RAII: the compiler emits the release on every path unconditionally.
```

Without RAII, resource acquisition requires a manual `try/finally` pair. In a
codebase with hundreds of acquisitions this causes missed branches, verbose boilerplate,
and fragile refactoring. RAII makes correct behaviour the default with no programmer
effort — including in legacy code where `try/finally` was already written by hand.

---

## 3. The RAII Protocol: Two Components

```
┌──────────────────────────────────────────────────────────────────────┐
│                         RAII Protocol                                 │
│                                                                       │
│  Component 1: Drop Trait                                              │
│    — vtable slot: drop_fn                                             │
│    — every class that holds a resource gets a compiler-generated      │
│      drop_fn, inferred from its type, methods, and call patterns      │
│    — drop_fn is the single authoritative destructor for that class    │
│    — drop_fn releases resources and decrements child refs;            │
│      it does NOT free memory — the allocator layer does that          │
│                                                                       │
│  Component 2: Scope Guard                                             │
│    — compiler-synthesised, never visible to the programmer            │
│    — registered at every resource acquisition site                    │
│    — flush (call drop_fn) is emitted on ALL exit paths from scope:    │
│      normal end, return, break, continue, throw, async suspension     │
└──────────────────────────────────────────────────────────────────────┘
```

These two components are driven entirely by compiler analysis. No `using` keyword,
no `Symbol.dispose`, no annotations. The compiler detects resource intent through
four signal sources and converts it into scope guards mechanically.

---

## 4. Component 1 — The Drop Trait (Vtable `drop_fn`)

The `drop_fn` field in the vtable is the implementation slot for RAII:

```rust
pub struct VTable {
    pub parent_vtable:  *const VTable,
    pub shape_id:       u64,
    pub type_name:      *const u8,

    // RAII: the destructor. Called exactly once per object lifetime.
    // Responsibilities:
    //   1. Release all held external resources (close fd, unlock mutex, etc.)
    //   2. Call drop_fn on all Owned child fields
    //   3. Call circ_dec on all Shared(CIRC) child fields
    //   4. NEVER call free(self) — the allocator layer does that
    pub drop_fn: Option<unsafe extern "C" fn(obj: *mut u8)>,

    // Async RAII: called for async release patterns.
    // Returns a *JsPromise that the caller must await before proceeding.
    pub async_drop_fn: Option<unsafe extern "C" fn(obj: *mut u8) -> *mut JsPromise>,

    // ... method slots ...
}
```

### The Separation of Concerns

```
RAII (drop_fn)  =  "release the resource"
Memory layer    =  "free the memory"

These are two separate concerns and must never be conflated.
```

`drop_fn` is identical regardless of which memory layer the object lives on. A file
handle's destructor closes the fd whether the object is `Stack`, `Owned`, `Shared(CIRC)`,
or `Arena`-allocated (arena has a special case — see Section 16).

### Drop Function Generation

The compiler generates `drop_fn` for every class that has at least one of:

- A resource field whose type is in the Resource Type Registry (Section 7).
- A method named `close`, `destroy`, `dispose`, `release`, `end`, `terminate`,
  `shutdown`, or `cleanup` that touches native handles or external state.
- Child fields of type `Owned` or `Shared(CIRC)` that need RC decrements or child drops.
- A `[Symbol.dispose]()` or `[Symbol.asyncDispose]()` method (if present in source).

For classes with none of the above, `drop_fn` is `None`. The vtable stores null.
LLVM eliminates the null-check and branch for classes whose vtable is a compile-time
constant — which is the common case.

### Drop Order for Fields

Within `drop_fn`, fields are destroyed in **reverse declaration order** — the same
rule as C++ and Rust. This ensures that a field whose destructor references another
field always sees valid data:

```typescript
class Server {
    db:     DatabaseConnection;  // acquired first → destroyed last
    cache:  CacheClient;         // acquired second → destroyed second
    logger: Logger;              // acquired third → destroyed first
}
// Compiled drop order: logger.drop_fn() → cache.drop_fn() → db.drop_fn()
```

### Drop Function Composition

If a class has both a user-defined dispose method AND CIRC child fields, the compiler
generates a **composed destructor**: first the user's dispose logic, then child
RC decrements. The composition is generated automatically from the `ShapeTable`.

### When `drop_fn` Itself Throws

If `drop_fn` throws during a scope guard flush, the exception is caught and held.
All remaining scope guards in the scope still flush (destruction continues). After
all guards have flushed, any held exceptions are combined into a `SuppressedError`
and re-thrown. This matches the TC39 `using` specification exactly.

```rust
fn emit_scope_guard_flush(scope_id: ScopeId, builder: &mut LlvmBuilder) {
    let guards = scope_guard_stack[scope_id].iter().rev();
    let mut suppressed: Vec<LlvmValue> = vec![];

    for guard in guards {
        emit_try_call_drop_fn(guard, &mut suppressed, builder);
    }

    emit_suppressed_error_rethrow(suppressed, builder);
}
```

---

## 5. Component 2 — Scope Guards and Destruction Order

A **scope guard** is a compiler-synthesised mechanism that binds a resource to
a scope. It is purely an artefact of MIR lowering, invisible to the programmer.

### Insertion Trigger

A scope guard is pushed at the **resource acquisition site** — the call to `open()`,
`connect()`, `lock()`, `new ResourceClass()`, or any equivalent — as determined by
the Lifecycle Inference Pass (Section 12). The scope guard is not triggered by any
syntax in the source.

### Destruction on ALL Exit Paths

This is the core RAII guarantee. `ScopeGuardFlush` is inserted at every exit:

```
Scope exits that require ScopeGuardFlush:
  1. Normal block end                → ScopeGuardFlush(scope)
  2. `return` statement              → ScopeGuardFlushTo(current, function_scope)
  3. `break` from loop               → ScopeGuardFlushTo(current, loop_scope)
  4. `continue` in loop              → ScopeGuardFlushTo(current, loop_body_scope)
  5. `throw` / exception propagation → ScopeGuardFlushTo(current, nearest_catch_scope)
  6. `await` suspension point        → special state-machine handling (Section 19)
```

### Reverse Destruction Order

Multiple resources acquired in the same scope are destroyed in **reverse acquisition
order**. The scope guard stack is a `Vec<(ScopeId, MirReg)>`. `ScopeGuardFlush`
iterates it in reverse:

```javascript
// Resources acquired in this order:
const fd   = fs.openSync(path, 'r');   // pushed first
const conn = db.connect();              // pushed second
const lock = mutex.lock();             // pushed third

// Scope end — destruction order:
// lock.drop_fn() → conn.drop_fn() → fd.drop_fn()
```

---

## 6. The Four Sources of RAII Signal

The compiler recognises resource intent through exactly four signal sources, in
descending order of certainty:

```
Signal Source               Certainty   Example
──────────────────────────────────────────────────────────────────────────
1. Resource Type             100%       fs.FileHandle, net.Socket, pg.Client
   Recognition                         — the type itself IS a resource

2. Lifecycle Pattern          95%       open() → close(), lock() → unlock()
   Matching                            — the call pattern encodes acquisition
                                         and release

3. Control Flow Graph         80–90%    DRA finds that Acquired state reaches
   Analysis (DRA)                       a scope exit without a release call

4. try/finally and            90%       try { use(x) } finally { x.close() }
   Promise .finally()                   — programmer already wrote RAII manually;
                                         compiler makes it unconditional and
                                         converts to zero-cost landing pads
──────────────────────────────────────────────────────────────────────────
```

All four sources produce entries in the **Resource Descriptor Table (RDT)** — the
single authoritative data structure that drives all scope guard insertion.

---

## 7. Source 1 — Resource Type Recognition

The most reliable signal is the type of the value. Some types are inherently resources.
The compiler maintains a **Resource Type Registry** in `crates/semantic/src/raii_builtins.rs`:

```rust
pub struct ResourceTypeEntry {
    pub type_name:       &'static str,
    pub release_methods: &'static [ReleaseMethod],
    pub release_mode:    ReleaseMode,
    pub idempotent:      bool,
}

pub enum ReleaseMode { Sync, Async, Either }

pub static RESOURCE_TYPE_REGISTRY: &[ResourceTypeEntry] = &[
    // === File System ===
    ResourceTypeEntry {
        type_name:       "fs.FileHandle",
        release_methods: &[ReleaseMethod::named("close")],
        release_mode:    ReleaseMode::Async,
        idempotent:      false,
    },
    ResourceTypeEntry {
        type_name:       "fs.ReadStream",
        release_methods: &[ReleaseMethod::named("close"), ReleaseMethod::named("destroy")],
        release_mode:    ReleaseMode::Either,
        idempotent:      true,
    },
    ResourceTypeEntry {
        type_name:       "fs.WriteStream",
        release_methods: &[ReleaseMethod::named("close"), ReleaseMethod::named("destroy"),
                           ReleaseMethod::named("end")],
        release_mode:    ReleaseMode::Either,
        idempotent:      true,
    },

    // === Networking ===
    ResourceTypeEntry {
        type_name:       "net.Socket",
        release_methods: &[ReleaseMethod::named("destroy"), ReleaseMethod::named("end")],
        release_mode:    ReleaseMode::Either,
        idempotent:      true,
    },
    ResourceTypeEntry {
        type_name:       "net.Server",
        release_methods: &[ReleaseMethod::named("close")],
        release_mode:    ReleaseMode::Either,
        idempotent:      false,
    },
    ResourceTypeEntry {
        type_name:       "http.ClientRequest",
        release_methods: &[ReleaseMethod::named("destroy")],
        release_mode:    ReleaseMode::Sync,
        idempotent:      true,
    },
    ResourceTypeEntry {
        type_name:       "http.IncomingMessage",
        release_methods: &[ReleaseMethod::named("destroy")],
        release_mode:    ReleaseMode::Sync,
        idempotent:      true,
    },
    ResourceTypeEntry {
        type_name:       "http.ServerResponse",
        release_methods: &[ReleaseMethod::named("end"), ReleaseMethod::named("destroy")],
        release_mode:    ReleaseMode::Either,
        idempotent:      false,
    },

    // === Streams ===
    ResourceTypeEntry {
        type_name:       "stream.Readable",
        release_methods: &[ReleaseMethod::named("destroy")],
        release_mode:    ReleaseMode::Sync,
        idempotent:      true,
    },
    ResourceTypeEntry {
        type_name:       "stream.Writable",
        release_methods: &[ReleaseMethod::named("end"), ReleaseMethod::named("destroy")],
        release_mode:    ReleaseMode::Either,
        idempotent:      true,
    },
    ResourceTypeEntry {
        type_name:       "stream.Transform",
        release_methods: &[ReleaseMethod::named("destroy")],
        release_mode:    ReleaseMode::Sync,
        idempotent:      true,
    },

    // === Child Process ===
    ResourceTypeEntry {
        type_name:       "child_process.ChildProcess",
        release_methods: &[ReleaseMethod::named("kill")],
        release_mode:    ReleaseMode::Sync,
        idempotent:      true,
    },

    // === Timers ===
    ResourceTypeEntry {
        type_name:       "NodeJS.Timeout",
        release_methods: &[ReleaseMethod::global("clearTimeout")],
        release_mode:    ReleaseMode::Sync,
        idempotent:      true,
    },
    ResourceTypeEntry {
        type_name:       "NodeJS.Immediate",
        release_methods: &[ReleaseMethod::global("clearImmediate")],
        release_mode:    ReleaseMode::Sync,
        idempotent:      true,
    },

    // === Worker Threads ===
    ResourceTypeEntry {
        type_name:       "worker_threads.Worker",
        release_methods: &[ReleaseMethod::named("terminate")],
        release_mode:    ReleaseMode::Async,
        idempotent:      false,
    },

    // === Popular npm packages ===
    ResourceTypeEntry {
        type_name:       "pg.Client",
        release_methods: &[ReleaseMethod::named("end"), ReleaseMethod::named("release")],
        release_mode:    ReleaseMode::Async,
        idempotent:      false,
    },
    ResourceTypeEntry {
        type_name:       "pg.Pool",
        release_methods: &[ReleaseMethod::named("end")],
        release_mode:    ReleaseMode::Async,
        idempotent:      false,
    },
    ResourceTypeEntry {
        type_name:       "mysql2.Connection",
        release_methods: &[ReleaseMethod::named("end"), ReleaseMethod::named("destroy")],
        release_mode:    ReleaseMode::Either,
        idempotent:      false,
    },
    ResourceTypeEntry {
        type_name:       "ioredis.Redis",
        release_methods: &[ReleaseMethod::named("quit"), ReleaseMethod::named("disconnect")],
        release_mode:    ReleaseMode::Async,
        idempotent:      false,
    },
    ResourceTypeEntry {
        type_name:       "mongoose.Connection",
        release_methods: &[ReleaseMethod::named("close")],
        release_mode:    ReleaseMode::Async,
        idempotent:      false,
    },
];
```

When the semantic pass assigns a known resource type to a binding, the compiler
annotates it immediately. No further analysis is needed — the type is sufficient.

### Extensible via Config File

Teams add their own resource types in `tscompiler.toml`:

```toml
[[raii.resource_types]]
type_name       = "mylib.DatabaseConnection"
release_methods = ["close", "end"]
release_mode    = "async"
idempotent      = false

[[raii.resource_types]]
type_name       = "mylib.LockGuard"
release_methods = ["unlock", "release"]
release_mode    = "sync"
idempotent      = true
```

The config is read at compilation startup and merged into the registry at compile time.

### Method-Name Heuristics for Unknown Types

Even without type information, the presence of certain methods on a class is strong
evidence of resource ownership. During shape inference:

```rust
// crates/semantic/src/lifecycle_patterns.rs

pub fn infer_resource_from_methods(shape: &Shape) -> Option<ResourceDescriptor> {
    let release_method =
        find_method(shape, "close")    .or_else(||
        find_method(shape, "destroy")  .or_else(||
        find_method(shape, "dispose")  .or_else(||
        find_method(shape, "release")  .or_else(||
        find_method(shape, "end")      .or_else(||
        find_method(shape, "terminate").or_else(||
        find_method(shape, "shutdown") .or_else(||
        find_method(shape, "cleanup")))))));

    let release = release_method?;

    // Confidence check: does this method touch native handles or external state?
    let touches_external = method_touches_native_stub_or_io(release, shape);

    if touches_external {
        Some(ResourceDescriptor {
            release_fn: ReleaseFn::NamedMethod(release.name.clone()),
            source:     RaiiSource::LifecyclePattern,
            idempotent: method_is_idempotent(release),
            ..infer_defaults(shape)
        })
    } else {
        None
    }
}
```

This covers all user-defined classes with a conventional cleanup method, without any
annotation:

```javascript
// Legacy class — no using, no Symbol.dispose — but close() exists
class DatabasePool {
    constructor(config) {
        this.connections = [];
    }
    close() {
        this.connections.forEach(c => c.end());
        this.connections = [];
    }
}
// → close() detected → Shape.drop_fn = DatabasePool_close
// → every DatabasePool binding gets RAII lifecycle management
```

---

## 8. Source 2 — Lifecycle Pattern Matching

When a type is not in the registry, the compiler examines the call pattern around
a binding to infer acquisition and release:

### Acquisition Patterns

A call is classified as resource acquisition when:

1. It is a constructor call (`new X()`) where `X`'s constructor performs I/O or
   calls a native stub returning a handle.
2. It is a call whose name matches an acquisition verb and whose return type is
   non-primitive.
3. It is a pool-style method call: `getConnection`, `getClient`, `checkout`, `lease`.

```rust
pub static ACQUISITION_VERBS: &[&str] = &[
    "open", "connect", "create", "acquire", "lock", "begin",
    "start", "spawn", "fork", "listen", "bind", "attach",
    "getConnection", "getClient", "getChannel", "checkout", "lease", "borrow",
];

pub static RELEASE_VERBS: &[&str] = &[
    "close", "destroy", "dispose", "release", "unlock", "end",
    "stop", "terminate", "kill", "disconnect", "detach", "free",
    "commit", "rollback", "abort", "finish", "done", "cleanup",
    "return", "checkin",
];
```

### Pairing Acquisition with Release

1. Build a use-chain graph for the acquired value: all call sites where it is used
   as `this` or as an argument.
2. Among those, find any call whose method name matches `RELEASE_VERBS`.
3. Record: acquisition site → `ResourceAcquire { binding, release_fn }` and
   release site → `ResourceRelease { binding, call_site }`.
4. Run DRA (Source 3) to verify coverage on all paths and insert guards on missing ones.
5. If no explicit release exists and the value does not escape: compiler warning +
   implicit release at scope end.

---

## 9. Source 3 — Control Flow Graph Analysis and DRA

Source 3 is **Definite Release Analysis (DRA)** — a forward dataflow analysis over
the CFG that verifies release coverage on every code path and inserts implicit guards
where coverage is missing.

### Release States

```
ReleaseState per resource binding r, per CFG node:
  Unreachable   — this path never acquires r
  MayAcquire    — might be acquired depending on branch (conditional acquisition)
  Acquired      — r is acquired and not yet released
  Released      — r has been released on this path
  MaybeReleased — some paths released, some did not (join point)
```

### Dataflow Equations

```
out(n) = transfer(n, in(n))

transfer(n, state):
  AcquireNode(r)  → Acquired
  ReleaseNode(r)  → Released
  ReturnNode:
    if Acquired   → INSERT implicit release before return; return Released
  ThrowNode:
    if Acquired   → INSERT implicit release on exception path; return Released
  otherwise       → return state

join(states):
  all Released    → Released
  any Acquired    → MaybeReleased   ← missing paths — insert guards here
  all Unreachable → Unreachable
```

When `out(function_exit)` is `MaybeReleased`, the compiler inserts implicit
`ScopeGuardFlushTo` instructions on all paths where release is missing.

### Exception Paths

Every `call` that may throw is a CFG edge to the nearest exception handler. DRA
analyses these edges:

```
For every CallNode that may throw:
  Add CFG edge: CallNode → NearestCatchScope
  Run DRA along this edge.
  If Acquired state reaches NearestCatchScope without release:
    INSERT ScopeGuardFlushTo(current_scope, catch_scope) on this edge.
```

### Loop Analysis

DRA uses fixpoint iteration for loops:

```
Repeat until stable:
  Run DRA forward through loop body.
  At loop back-edge: join state with pre-loop state.
  If state changes: re-run.
```

This handles `continue` statements inside loops that skip the release:

```javascript
for (const item of items) {
    const conn = pool.getConnection();  // Acquired each iteration
    if (item.skip) continue;           // ← DRA: Acquired reaches continue
                                        //   → INSERT conn.release() before continue
    processItem(item, conn);
    pool.release(conn);                 // Explicit release on normal path
}
```

### Legacy Code Example

```javascript
// ES5 — no using, no annotations
function processFile(path) {
    var fd = fs.openSync(path, 'r');   // Acquired
    if (someCondition) {
        return earlyResult;             // ← MISSING RELEASE — DRA detects
    }
    var data = readAll(fd);
    fs.closeSync(fd);                  // Explicit release on normal path
    return data;
}
```

DRA analysis:

```
After open():   Acquired
At if-branch:
  true path:    Acquired + ReturnNode → INSERT fs.closeSync(fd) before return
  false path:   Acquired (continues)
After close():  Released
At return:      Released ✓
```

The compiler inserts `fs.closeSync(fd)` on the early-return path. The developer
changes nothing. The binary is leak-free.

---

## 10. Source 4 — `try/finally` and Promise `.finally()` as RAII Intent

When a programmer writes `try/finally` with a release in the `finally` block, they
are expressing RAII intent using the only mechanism JavaScript offered before modern
tooling. The compiler reads this as authoritative intent and:

1. **Recognises the intent:** `conn.end()` in `finally` = release of `conn`.
2. **Converts to first-class RAII:** produces a `ResourceBinding` entry in the RDT.
3. **Replaces the `try/finally` structure with landing pads:** the emitted binary has
   no `try/finally` exception table overhead — only zero-cost LLVM `invoke`/`landingpad`
   pairs that are consulted only when an exception is actually thrown.
4. **Extends coverage via DRA:** if any paths were already handled by `finally`, DRA
   verifies them and adds guards on any additional paths it finds.

```
BEFORE (programmer's code):             AFTER (compiler transforms to):
──────────────────────────────          ──────────────────────────────
var conn = db.connect();                var conn = db.connect();
try {                                   // ScopeGuardPush(conn, conn.end)
    doWork(conn);                       doWork(conn);
} finally {                             // [scope end: ScopeGuardFlush]
    conn.end();                         // → conn.end() via LLVM landingpad
}                                       // No try/finally overhead
```

### Pattern Recogniser

```rust
// crates/semantic/src/try_finally_raii.rs

pub fn detect_try_finally_raii(stmt: &HirStmt) -> Option<ResourceBinding> {
    let HirStmt::TryCatch {
        body,
        catch: None,
        finally: Some(finally_stmts),
    } = stmt else { return None; };

    let acquired = find_resource_acquisition_in_prologue(body)?;
    let released = find_release_of(&acquired.binding, finally_stmts)?;

    Some(ResourceBinding {
        binding:    acquired.binding,
        acquire_at: acquired.call_site,
        release_fn: released.call_expr,
        mode:       ReleaseMode::from_call(&released.call_expr),
    })
}

pub fn detect_try_catch_finally_raii(stmt: &HirStmt) -> Option<ResourceBinding> {
    let HirStmt::TryCatch {
        body,
        catch: Some(_),
        finally: Some(finally_stmts),
    } = stmt else { return None; };

    let acquired = find_resource_acquisition_in_prologue(body)?;
    let released = find_release_of(&acquired.binding, finally_stmts)?;
    Some(ResourceBinding { ... })
}
```

### Handled Variations

```javascript
// Variation 1: acquisition before try
var fd = fs.openSync(path, 'r');
try { ... } finally { fs.closeSync(fd); }

// Variation 2: acquisition inside try, conditional release in finally
try {
    var conn = db.connect();
    doWork(conn);
} finally {
    if (conn) conn.end();   // guard against failed acquisition
}

// Variation 3: named cleanup function inlined by compiler
function cleanup() { conn.end(); lock.unlock(); }
try { ... } finally { cleanup(); }
// → compiler inlines cleanup() → two ResourceBindings extracted

// Variation 4: callback-style (pre-Promise Node.js)
db.connect(function(err, conn) {
    if (err) return callback(err);
    doWork(conn, function(err, result) {
        conn.end();
        callback(err, result);
    });
});
// → Source 2 pattern matching on callback-shaped code

// Variation 5: Promise chain
db.connect()
    .then(conn => doWork(conn).finally(() => conn.end()))
// → .finally() = Promise equivalent of try/finally → same recogniser

// Variation 6: Pool pattern with try/catch
const client = await pool.connect();
try {
    const result = await client.query(sql);
    client.release();
    return result;
} catch (err) {
    client.release(err);
    throw err;
}
// → DRA: Released on all paths → no implicit guards needed
//   → compiler converts to efficient landing pad structure
```

### Promise `.finally()` as RAII Intent

`.finally()` is semantically identical to `try/finally` in async code. The same
pattern detector handles it:

```javascript
db.connect()
  .then(conn => processData(conn))
  .finally(() => conn.end());
```

This produces the same `ResourceBinding` as the synchronous version. The release
is injected as an async scope guard into the Promise chain's state machine.

---

## 11. The Resource Descriptor Table (RDT)

All four signal sources produce entries in the **Resource Descriptor Table** — the
single authoritative data structure that drives all scope guard insertion.

```rust
// crates/semantic/src/rdt.rs

pub struct ResourceDescriptorTable {
    pub entries: HashMap<BindingId, ResourceDescriptor>,
}

pub struct ResourceDescriptor {
    pub binding:        BindingId,
    pub resource_kind:  ResourceKind,
    pub acquire_site:   CallSiteId,
    pub release_fn:     ReleaseFn,
    pub release_mode:   ReleaseMode,
    pub transfer_state: TransferState,
    pub idempotent:     bool,
    pub confidence:     u8,       // 0–100; drives warning threshold
    pub source:         RaiiSource,
}

pub enum ResourceKind {
    FileDescriptor,
    NetworkSocket,
    DatabaseConnection,
    Lock,
    Transaction,
    Timer,
    Process,
    Custom(String),
}

pub enum ReleaseFn {
    NamedMethod(String),              // obj.close()
    GlobalFn(String),                 // clearTimeout(timer)
    MemoryOnly,                       // no external resource
    MultipleOptions(Vec<String>),     // compiler picks safest (prefer idempotent)
}

pub enum TransferState {
    OwnedHere,                        // release is this scope's responsibility
    TransferredTo(BindingId),         // responsibility moved to receiver
    ExplicitlyReleased { at: CallSiteId }, // programmer wrote it explicitly
    Unreachable,                      // warning emitted
}

pub enum RaiiSource {
    TypeRegistry,
    LifecyclePattern,
    TryFinally,
    ControlFlowCompletion,
}
```

The RDT is built during the Lifecycle Inference Pass and consumed by MIR lowering.
Every scope guard insertion is driven by RDT entries — never by syntax in the source.

---

## 12. The Lifecycle Inference Pass

This compiler pass is inserted between **Semantic Analysis** and **HIR Lowering**.
It takes the full HIR, scope tree, and alias graph as input and produces the RDT.

### Pass Structure

```
LifecycleInferencePass:

  Phase A — Type-Based Detection (Source 1):
    Walk all BindingInfos with a resolved type.
    For each type in RESOURCE_TYPE_REGISTRY → add RDT entry (confidence = 100).
    For each class Shape with a qualifying method → add RDT entry (confidence = 90).

  Phase B — Pattern Matching (Source 2):
    Walk all HIR call expressions.
    For each call matching ACQUISITION_VERBS and non-primitive return type
      → record candidate.
    For each candidate: look for paired release in use-chain → add RDT entry
      (confidence = 95).

  Phase C — try/finally and Promise.finally() Recognition (Source 4):
    Walk all HirStmt::TryCatch nodes with finally blocks.
    Walk all Promise .finally() chain patterns.
    For each match: detect_try_finally_raii → add RDT entry (confidence = 90).

  Phase D — DRA Completion (Source 3):
    For every RDT entry from A/B/C:
      Run DRA over the binding's containing function CFG.
      For every MaybeReleased path: add implicit ScopeGuardFlushTo to the entry.
      For every definitely-missing release: add implicit release at scope/fn exit.

  Phase E — Transfer Analysis:
    For every RDT entry: run escape analysis.
    If binding escapes (returned, stored in field, passed to callback that stores it):
      set TransferState::TransferredTo → release responsibility moves to receiver.
    If receiver is a known type: synthesise drop_fn on receiver's Shape.
```

### Pass Output

A fully annotated RDT. MIR lowering consumes it. No scope guard is ever emitted
from syntax — only from RDT entries produced by this pass.

---

## 13. Implicit Scope Guard Insertion

For every `ResourceDescriptor` in the RDT with `TransferState::OwnedHere`:

```rust
fn lower_resource_binding(desc: &ResourceDescriptor, mir: &mut MirBuilder) {
    // 1. At acquisition site: push implicit scope guard
    mir.emit_after(desc.acquire_site, MirInstr::ImplicitScopeGuardPush {
        scope_id:   desc.binding.scope_id,
        reg:        desc.binding.reg,
        release:    desc.release_fn.clone(),
        source:     desc.source,
        confidence: desc.confidence,
    });

    // 2. If explicit release exists: cancel guard before it (prevent double-release)
    if let TransferState::ExplicitlyReleased { at } = desc.transfer_state {
        mir.emit_before(at, MirInstr::ScopeGuardCancel {
            scope_id: desc.binding.scope_id,
            reg:      desc.binding.reg,
        });
        // The explicit programmer call remains and IS the release on that path
    }

    // 3. At all scope exits where release is missing: flush
    for missing_path in desc.missing_release_paths() {
        mir.emit_at(missing_path, MirInstr::ScopeGuardFlushTo {
            from_scope: desc.binding.scope_id,
            to_scope:   missing_path.target_scope,
        });
    }
}
```

### The `ScopeGuardCancel` Mechanism

When the programmer wrote an explicit release on some paths, the implicit guard is
pushed at acquisition and **cancelled immediately before each explicit release call**.
The explicit call runs unchanged. This prevents double-release.

```
MIR sequence — explicit release on normal path, missing on early return:

  ImplicitScopeGuardPush(fd, closeSync)   ← at open() — armed
  ... code ...
  [normal path]:
    ScopeGuardCancel(fd)                  ← disarms guard
    CallDirect(fs_close_sync, [fd])       ← programmer's explicit close runs
  [early return path]:
    ScopeGuardFlushTo(scope, fn)          ← guard fires here — programmer missed this
    Return(early_result)
```

**The explicit release in the programmer's code is always honoured. Implicit RAII
only covers paths the programmer did not cover.**

---

## 14. Conflict Resolution: Explicit Manual Release + Implicit RAII

```
Case 1: Explicit release on ALL paths.
  → DRA: Released at all exits.
  → Compiler: validates completeness, emits no implicit guards.
  → Converts try/finally structure to landing pads for efficiency.
  → Zero overhead added.

Case 2: Explicit release on SOME paths, missing on others.
  → DRA: MaybeReleased at some exits.
  → Compiler: ScopeGuardCancel before each explicit release.
              ScopeGuardFlushTo on missing paths.
  → Explicit release runs where written; implicit runs where missing.

Case 3: No explicit release anywhere, resource does not escape.
  → DRA: Acquired at all exits.
  → Compiler: full implicit RAII (ImplicitScopeGuardPush + Flush).
  → Warning if confidence < 95%. Silent if confidence == 100% (known type).

Case 4: Multiple release calls (potential double-release).
  → DRA: Released → Released (second call after first).
  → If idempotent: allow (streams, sockets with destroy()).
  → If not idempotent: compiler warning.
     Replaces second call with: if (!released_flag) release().
```

---

## 15. Legacy Pattern Catalogue

### Pattern 1 — Raw `try/finally` (Node.js pre-2015)

```javascript
var fd;
try {
    fd = fs.openSync(path, 'r');
    var data = readAll(fd);
    return data;
} finally {
    if (fd !== undefined) fs.closeSync(fd);
}
```

Source 4 detection. The `if (fd !== undefined)` guard is encoded into the emitted
scope guard as a null-check in the release function: `if (fd_reg != undefined_nan_box) call closeSync(fd)`.

### Pattern 2 — Callback-Based Acquisition (Express style)

```javascript
app.get('/users', function(req, res) {
    var conn = db.getConnection();
    conn.query('SELECT ...', function(err, rows) {
        conn.release();
        if (err) return res.status(500).send(err);
        res.json(rows);
    });
});
```

Source 2 detection. DRA descends interprocedurally into the locally-passed callback
literal to verify `conn.release()` is reached on all paths inside it. If verified:
`Acquired` state is resolved at the outer level; no implicit guard inserted.

### Pattern 3 — Prototype-Based OOP (Legacy ES5)

```javascript
function Server() {
    this.socket = net.createServer();
    this.db = mysql.createConnection(config);
}
Server.prototype.shutdown = function() {
    this.socket.close();
    this.db.end();
};
```

Source 1 detects `this.socket` (type `net.Server`) and `this.db` (type `mysql.Connection`).
Source 2 detects `shutdown` as a multi-release method. The `Shape` for `Server` gets
`drop_fn = Server_shutdown`. Every `Server` binding is RAII-managed.

### Pattern 4 — Event Listener Accumulation

```javascript
function setupHandlers(emitter) {
    emitter.on('data', handleData);
    emitter.on('error', handleError);
}
```

DRA: `Acquired` with no `Released` at function exit. The compiler does NOT insert
`removeAllListeners` — listener lifetime is intentionally tied to the emitter, not
to the function call. Instead: `TransferState::TransferredTo(emitter_binding)`. When
the emitter is destroyed, its `drop_fn` calls `removeAllListeners` as part of cleanup.

### Pattern 5 — Async Function Without Cleanup

```javascript
async function processData() {
    const conn = await db.connect();
    const data = await conn.query(sql);
    return data;    // conn never released
}
```

Source 1: `conn` is a known type. DRA: `Acquired` at `return`. Compiler inserts
`await conn.end()` as an injected async state machine state before the return.
Warning emitted: `implicit release inserted: conn (pg.Client)`.

### Pattern 6 — Pool Pattern with Explicit Branches

```javascript
async function query(sql) {
    const client = await pool.connect();
    try {
        const result = await client.query(sql);
        client.release();
        return result;
    } catch (err) {
        client.release(err);
        throw err;
    }
}
```

DRA: `Released` on all paths. Compiler validates completeness, emits no implicit
guards, converts the try/catch structure to efficient landing pads.

### Pattern 7 — Module-Level Resource

```javascript
const globalConn = mysql.createConnection(config);
globalConn.connect();
module.exports.query = function(sql, cb) { globalConn.query(sql, cb); };
```

Source 1 detection. `globalConn` is module-scoped — no conventional scope exit.
Compiler registers it in `__global_raii_registry` and inserts `globalConn.end()`
via `libc::atexit`:

```llvm
@__global_raii_entries = global [N x %RaiiEntry] [
    { ptr @globalConn, ptr @mysql_connection_end },
    ...
]
; Registered in main():
call void @libc_atexit(ptr @__flush_global_raii)
```

---

## 16. RAII Across All Four Memory Layers

### Stack Layer

`drop_fn` is called in the function epilogue (before stack pop) and at every early
return site. Memory release = stack pop, implicit, zero cost. `drop_fn` must NOT
call `free()`.

```llvm
define i64 @process() {
entry:
    %guard = alloca %MutexGuard
    call void @MutexGuard_init(ptr %guard, ptr %global_mutex)
    ; ... body ...
    ; All return paths emit:
    call void @MutexGuard_drop_fn(ptr %guard)  ; release mutex
    ; no free() — stack pops automatically
    ret i64 %result
}
```

### Arena Layer

By design, arenas free memory in bulk without calling per-object destructors.
A value whose `drop_fn` releases external resources (flagged `RAII_EXTERNAL` in
its `Shape`) **cannot be arena-allocated without a destructor list**.

The arena maintains a destructor list for this case:

```rust
pub struct Arena {
    bump:     *mut u8,
    end:      *mut u8,
    segments: Vec<Segment>,
    dtor_list: Vec<DtorEntry>,   // registered drop_fns for RAII_EXTERNAL objects
}

pub struct DtorEntry {
    obj_ptr: *mut u8,
    drop_fn: unsafe extern "C" fn(*mut u8),
}
```

On `arena_destroy`: walk `dtor_list` in reverse order, call each `drop_fn`, then
bulk-free the memory. Detection: `RAII_EXTERNAL` is set when `drop_fn` calls any
function outside `circ_dec` and `free` (i.e. touches the OS or global state).

### Owned Layer

The primary use case. At ASAP last-use point, MIR emits `Drop(reg)` which codegen
expands to: `call drop_fn(obj)`, then `call free(obj)`. For early returns:
`ScopeGuardFlush` emits the same sequence at the return site. Destruction is exactly
once, at a statically known point. This is the model that replaces `try/finally`.

### Shared (CIRC) Layer

`drop_fn` is called inside `circ_destroy`, before `free()`:

```rust
unsafe fn circ_destroy(obj: *const CircHeader) {
    let vtable = get_vtable(obj);
    if let Some(drop_fn) = (*vtable).drop_fn {
        drop_fn(obj as *mut u8);    // RAII: release resources
    }
    libc::free(obj as *mut _);      // Memory: free
}
```

`drop_fn` is called exactly once — only one thread can observe `prev == 1` from
the `fetch_sub`, so there is no double-drop race. For cycle members, the Bacon-Rajan
collector calls `drop_fn` on each member in **reverse topological order** of the
cycle graph, ensuring members can safely access peers during destruction.

**CIRC objects with timing-sensitive `drop_fn` (e.g. mutex guards) must be `Owned`.**
Use the `Unique<T>` annotation to force this classification:

```typescript
// @unique
const guard = mutex.lock();
// → compiler forces Owned memory class → ASAP + RAII is immediate and unconditional
```

---

## 17. RAII and ASAP: The Relationship

```
ASAP answers: WHEN does destruction happen?
              → At the last-use point in the MIR liveness interval.

RAII answers:  WHAT happens at destruction?
              → drop_fn: release resources, decrement children, then free.
```

They compose:

```
Owned + drop_fn:
  ASAP identifies last-use → inserts Drop(reg)
  Drop(reg) codegen emits: call drop_fn(ptr), then call free(ptr)

Stack + drop_fn:
  ASAP last-use = scope end (stack values cannot outlive frame)
  Scope epilogue emits: call drop_fn(alloca_ptr)
  No free — stack pops automatically

CIRC + drop_fn:
  ASAP = RC-driven: last RcDec reaches zero
  circ_destroy emits: call drop_fn(obj_ptr), then call free(obj_ptr)
```

**ASAP + RAII together = correct resource management at the earliest possible moment.**
This is strictly more powerful than `try/finally`, which only covers paths the
programmer writes, and only for normal and exception exits.

---

## 18. Exception Safety and `try/catch` Under RAII

Every call that may throw is emitted with LLVM `invoke` instead of `call`. The
landing pad flushes all active scope guards before re-throwing:

```llvm
define ptr @processFile(ptr %path) personality ptr @__gxx_personality_v0 {
entry:
    %fd_guard   = call ptr @raii_guard_open(ptr %path)
    %lock_guard = call ptr @raii_guard_lock(ptr @global_mutex)

    %result = invoke ptr @do_work(ptr %fd_guard)
              to label %normal
              unwind label %cleanup

cleanup:
    %exn = landingpad { ptr, i32 } catch ptr null
    call void @MutexGuard_drop_fn(ptr %lock_guard)  ; reverse order
    call void @free(ptr %lock_guard)
    call void @FileHandle_drop_fn(ptr %fd_guard)
    call void @free(ptr %fd_guard)
    resume { ptr, i32 } %exn

normal:
    call void @MutexGuard_drop_fn(ptr %lock_guard)
    call void @free(ptr %lock_guard)
    call void @FileHandle_drop_fn(ptr %fd_guard)
    call void @free(ptr %fd_guard)
    ret ptr %result
}
```

The `invoke`/`landingpad` pattern is zero-cost on the non-exception path. Exception
handling tables (DWARF) are consulted only when an exception is actually thrown.

### Exception Safety Levels

| Level | Guarantee | How achieved |
|---|---|---|
| **Basic** | No resource leaks, no dangling pointers | RAII scope guards flush on all paths |
| **Strong** | Operation either fully succeeds or has no effect | Transactional `drop_fn` (e.g. `tx.rollback()` if not committed) |
| **No-throw** | Function never throws | Mark `// @nothrow`; compiler emits `call` instead of `invoke` |

Basic exception safety is provided automatically. Strong requires the programmer to
write transactional destructor logic.

---

## 19. RAII and Async/Await

When DRA determines that a resource needs release on an async function's exit path
and the release is async (returns a `Promise`), the compiler injects an await point
into the async state machine. This applies to all async resources regardless of how
they were written — no special syntax required.

### Injected Disposal States

```javascript
// Programmer wrote this — conn never released:
async function process() {
    const conn = await db.connect();
    const result = await doWork(conn);
    return result;
}

// Compiler transforms the state machine to:
// State 0: await db.connect() → conn
// State 1: await doWork(conn) → result
// State 2: [INJECTED] await conn.end() → (void)
// State 3: return result
```

The injected state is inserted between the last use of `conn` and the function's
return. It is entirely invisible to the programmer.

### Multiple Async Resources

Each async resource gets its own injected disposal state, in reverse acquisition order:

```javascript
async function example() {
    const conn  = await db.connect();    // injected disposal state: conn
    const lock  = await mutex.acquire(); // injected disposal state: lock (disposed first)
    await doWork(conn, lock);
}
// Disposal sequence: lock.end() (state N) → conn.end() (state N+1)
```

### Exception Path in Async RAII

If `doWork()` throws:
1. The state machine's poll loop catches the exception.
2. The machine transitions through all injected disposal states in reverse.
3. Each disposal is awaited before proceeding.
4. After all disposals complete, the outer Promise rejects with the original exception
   (or `SuppressedError` if any disposal also threw).

### Timeout Safety for Async Release

Every injected async release is wrapped in a timeout to prevent hang:

```javascript
// Injected as: Promise.race([conn.end(), timeout(5000)])
```

Timeout and fallback are configurable:

```toml
# tscompiler.toml
[raii]
async_release_timeout_ms = 5000
async_release_fallback   = "destroy"  # or "warn" or "ignore"
```

---

## 20. RAII and Closures

When a resource is captured by a closure that outlives the acquisition scope, the
RAII ownership transfers to the closure's lifetime:

```javascript
function makeHandler(db) {
    var conn = db.connect();       // resource acquired
    return function(req) {
        conn.query(req.params.id); // conn captured — closure outlives function
    };
}
// conn released when the returned closure is destroyed (its RC hits zero)
```

### Capture Transfer Protocol

When DRA detects a resource binding captured by an outliving closure:

1. The scope guard for the acquisition scope is **cancelled**.
2. The resource is moved into the closure's capture struct as an `OwnedMove` capture.
3. The closure's synthesised `drop_fn` calls the resource's `drop_fn` before freeing
   the closure struct itself.

```rust
pub struct Closure_handler {
    circ_header: CircHeader,
    vtable:      *const Closure_handler_VTable, // vtable.drop_fn = closure_handler_drop
    conn:        *mut DbConnection,             // OwnedMove — RAII transferred
}

unsafe extern "C" fn closure_handler_drop(obj: *mut u8) {
    let closure = obj as *mut Closure_handler;
    if let Some(drop_fn) = DbConnection_vtable.drop_fn {
        drop_fn((*closure).conn as *mut u8); // release connection
    }
    free((*closure).conn as *mut _);
    // closure memory freed by circ_destroy after this returns
}
```

The resource is released exactly once, when the closure is destroyed — not when
the creating function returns.

---

## 21. RAII and CIRC: Interaction Protocol

```
circ_dec(obj):
    prev = fetch_sub(obj.rc, 1, AcqRel)
    if prev == 1:
        drop_fn(obj)          // RAII: release resources + decrement children
        free(obj)             // Memory: return to allocator
    elif prev <= THRESHOLD and not ACYCLIC:
        cycle_buffer_push(obj)
```

Under BiRC (Biased Reference Counting):

```rust
unsafe fn circ_dec_birc(obj: *const CircHeader) {
    let tid = current_thread_id();
    if (*obj).owner_tid == tid {
        (*obj).local_rc -= 1;  // non-atomic fast path
        if (*obj).local_rc == 0 {
            let global = (*obj).global_rc.load(Ordering::Acquire);
            if global == 0 {
                drop_fn_and_free(obj);
            }
        }
    } else {
        let prev_global = (*obj).global_rc.fetch_sub(1, Ordering::AcqRel);
        if prev_global == 1 {
            if (*obj).owner_tid == NO_OWNER && (*obj).local_rc == 0 {
                drop_fn_and_free(obj);
            }
        }
    }
}
```

`drop_fn` is called exactly once regardless of which thread observes RC == 0. The
atomic `fetch_sub` guarantees only one thread can observe `prev == 1`.

---

## 22. Handling Conditional Resource Acquisition

```javascript
let conn;
if (needsDatabase) {
    conn = db.connect();  // conditional acquisition
}
if (conn) {
    conn.end();           // conditional release matching acquisition
}
```

DRA uses the `MayAcquire` state for conditional acquisitions. The `if (conn)` check
before release transitions correctly: `Released` on the true branch (acquired +
released), `Released` trivially on the false branch (never acquired).

If the conditional release is missing:

```javascript
let conn;
if (needsDatabase) { conn = db.connect(); }
doWork(conn);
// No release anywhere
```

DRA: `MayAcquire` at function exit. Compiler inserts:
`if (conn !== undefined) conn.end()` in the scope guard flush, matching the
acquisition condition.

---

## 23. Handling Resources Stored in Objects

```javascript
function createContext(config) {
    return {
        db:    mysql.createConnection(config.db),
        redis: new Redis(config.redis),
        logger: winston.createLogger(config.logger),
    };
}
```

Transfer Analysis (Phase E of the Lifecycle Inference Pass):

1. Each resource is acquired inside `createContext`.
2. All are stored in the returned object → `TransferState::TransferredTo(return_value)`.
3. The returned object's `Shape` gets a synthesised `drop_fn` that calls:
   - `this.logger.close()` → `this.redis.disconnect()` → `this.db.end()`
   in reverse field-declaration order.
4. The caller manages all three resources through the object's single RAII lifecycle.

```javascript
const ctx = createContext(config);
handleRequest(ctx, req, res);
// ctx destroyed → drop_fn releases logger, redis, db — automatically
```

---

## 24. Handling Resources Passed Across Functions

```
Rule 1 — Callback Ownership:
  Resource passed to function that stores it in a closure or object outliving
  the call → TransferredTo(callee_binding). Caller emits no implicit release.

Rule 2 — Short-Term Borrow:
  Resource passed to function AND caller still calls release methods after
  the call → Borrow (no transfer). Caller retains release responsibility.

Rule 3 — Last-Use Transfer:
  Resource passed to function AND never used again in caller AND callee has
  a known release pattern → TransferredTo(callee).

Rule 4 — Unknown Callee:
  Callee has no summary (external) → conservative Borrow.
  Caller retains release responsibility.
```

```javascript
// Rule 2: borrow — caller still uses fd after readHeader
var fd = fs.openSync(path, 'r');
readHeader(fd);       // borrow: caller owns fd
var body = readRest(fd);
fs.closeSync(fd);     // explicit release — DRA: complete, no guards added

// Rule 3: last-use transfer
var conn = db.connect();
spawnWorker(conn);    // TransferredTo(workerThread)
// No implicit release in caller — worker owns it
```

---

## 25. Global and Module-Level Resources

Module-level bindings have no scope end in the conventional sense. Resources
at module scope are registered in a compiler-emitted global RAII registry
and released via `atexit`:

```llvm
@__global_raii_entries = global [N x %RaiiEntry] [
    { ptr @globalConn, ptr @mysql_connection_end },
    { ptr @globalTimer, ptr @clearTimeout_stub },
    ...
]

; Registered in the binary's main() prologue:
call void @libc_atexit(ptr @__flush_global_raii)
```

`__flush_global_raii` walks the registry in reverse registration order and calls
each `drop_fn`. Resources are released when the process exits — the correct
behaviour for module-level handles.

---

## 26. Safety: When the Compiler Cannot Prove Lifecycle

```
Preference 1 (best):   Prove it → emit correct implicit RAII silently.
Preference 2:          Emit warning → do nothing extra.
                       (A leak is safer than wrong teardown.)
Preference 3 (worst):  Emit incorrect release → never acceptable.
```

The compiler defers to Preference 2 when:

1. **Ambiguous release:** multiple candidate release functions; context insufficient
   to choose.
2. **Non-local release:** resource passed to an external function with `Unknown`
   summary — cannot follow it.
3. **Complex conditional acquisition:** predicate too complex to encode in the
   scope guard's null-check.
4. **Cross-module export:** resource exported and used in another compilation unit;
   transfer analysis incomplete.

### Warning Format

```
warning[RAII-001]: resource may not be released on all code paths
  --> src/db.ts:14:18
   |
14 |   const conn = db.connect();
   |                ^^^^^^^^^^^^ resource of type `pg.Client` acquired here
   |
23 |   if (err) return null;
   |            ^^^^^^^^^^^^ this path does not release `conn`
   |
   = help: add `conn.end()` before returning, or annotate with `// @raii`
           to enable conservative implicit release for this binding
   = note: implicit release was NOT inserted (ambiguous release semantics)
```

### The `// @raii` Escape Hatch

For ambiguous cases, a single inline comment triggers unconditional release at scope end:

```javascript
// @raii
const conn = db.connect();
// → unconditional release at scope end, no analysis required
```

This is the only annotation the compiler provides and it is purely an escape hatch.
The vast majority of code needs no annotation.

---

## 27. MIR Instruction Set for RAII

```rust
pub enum MirInstr {
    // === Existing memory instructions (unchanged) ===
    AllocStack(MirReg, ShapeId),
    AllocArena(MirReg, ShapeId, RegionId),
    AllocOwned(MirReg, ShapeId),
    AllocShared(MirReg, ShapeId),
    Move(MirReg, MirReg),
    Borrow(MirReg, MirReg),
    BorrowMut(MirReg, MirReg),
    RcInc(MirReg),
    RcDec(MirReg),
    Drop(MirReg),
    ArenaRelease(RegionId),

    // === RAII — explicit (from Symbol.dispose or using, if present in source) ===

    /// Register a value's drop_fn with the current scope's guard stack.
    ScopeGuardPush {
        scope_id: ScopeId,
        reg:      MirReg,
    },

    /// Cancel a pushed scope guard (ownership transferred to closure or callee).
    ScopeGuardCancel {
        scope_id: ScopeId,
        reg:      MirReg,
    },

    /// Flush all guards in scope_id in reverse push order.
    /// Emitted at: block end, return, break, continue, throw.
    ScopeGuardFlush {
        scope_id: ScopeId,
    },

    /// Flush guards from from_scope outward to to_scope (exclusive).
    ScopeGuardFlushTo {
        from_scope: ScopeId,
        to_scope:   ScopeId,
    },

    /// Async scope guard: call async_drop_fn and inject a state machine state.
    AsyncScopeGuardFlush {
        scope_id:    ScopeId,
        state_index: u32,
    },

    /// Call drop_fn without freeing (for arena objects with RAII_EXTERNAL shape).
    CallDropFnOnly {
        reg: MirReg,
    },

    /// Transfer RAII ownership to a closure's capture struct.
    RaiiTransferToCapture {
        source_reg:  MirReg,
        closure_reg: MirReg,
        field_idx:   FieldIdx,
    },

    // === RAII — implicit (emitted by Lifecycle Inference Pass) ===

    /// Identical to ScopeGuardPush but tagged as compiler-inferred.
    /// confidence < 80 → emit RAII-001 warning. >= 80 → silent. == 100 → no noise.
    ImplicitScopeGuardPush {
        scope_id:   ScopeId,
        reg:        MirReg,
        release:    ReleaseFn,
        source:     RaiiSource,
        confidence: u8,
    },

    /// Inject an async disposal state into the enclosing async state machine.
    /// Triggered by DRA on async functions — no await using syntax required.
    ImplicitAsyncScopeGuardFlush {
        scope_id:    ScopeId,
        reg:         MirReg,
        release:     ReleaseFn,
        state_index: u32,
        timeout_ms:  u32,
    },
}
```

---

## 28. LLVM IR Emission for RAII

### Normal Scope Exit with Multiple Guards

```llvm
; Two resources: fd (pushed first), lock (pushed second)
; Destruction order: lock → fd (reverse push order)

define void @handle_request(ptr %req) personality ptr @__gxx_personality_v0 {
entry:
    %fd   = call ptr @malloc(i64 sizeof_FileHandle)
    call void @FileHandle_init(ptr %fd, ptr %path)

    %lock = call ptr @malloc(i64 sizeof_MutexGuard)
    call void @MutexGuard_init(ptr %lock, ptr @global_mutex)

    %result = invoke ptr @process(ptr %req, ptr %fd, ptr %lock)
              to label %normal_exit
              unwind label %exception_exit

normal_exit:
    call void @MutexGuard_drop_fn(ptr %lock)  ; reverse order: lock first
    call void @free(ptr %lock)
    call void @FileHandle_drop_fn(ptr %fd)    ; then fd
    call void @free(ptr %fd)
    ret void

exception_exit:
    %exn = landingpad { ptr, i32 } catch ptr null
    call void @MutexGuard_drop_fn(ptr %lock)  ; same order on exception path
    call void @free(ptr %lock)
    call void @FileHandle_drop_fn(ptr %fd)
    call void @free(ptr %fd)
    resume { ptr, i32 } %exn
}
```

### RAII Transfer to Closure

```llvm
; conn captured by closure → scope guard cancelled → conn dropped by closure
entry:
    %conn = call ptr @malloc(i64 sizeof_DbConnection)
    call void @DbConnection_init(ptr %conn, ...)

    ; No ScopeGuardPush emitted — RaiiTransferToCapture moves responsibility
    %closure = call ptr @malloc(i64 sizeof_Closure_handler)
    %conn_slot = getelementptr %Closure_handler, ptr %closure, i32 0, i32 2
    store ptr %conn, ptr %conn_slot
    ; closure.drop_fn calls conn.drop_fn then frees conn
    ret ptr %closure
```

### Implicit Async RAII (Injected State)

```llvm
; State 2 injected by compiler — not written by programmer
define void @process_poll(ptr %state, ptr %waker) {
    %cur_state = load i32, ptr %state
    switch i32 %cur_state, label %invalid [
        i32 0, label %state_0_connect
        i32 1, label %state_1_work
        i32 2, label %state_2_dispose   ; INJECTED
        i32 3, label %state_3_return
    ]

state_2_dispose:
    ; Retrieve the pending dispose future or start it
    %conn = load ptr, ptr %state_conn_field
    %future = call ptr @DbConnection_async_drop_fn(ptr %conn)
    ; Race against timeout:
    %raced = call ptr @promise_race_timeout(ptr %future, i64 5000)
    ; Await: suspend until resolved
    store i32 3, ptr %state   ; advance to state 3 after resume
    ret void                  ; suspend

state_3_return:
    %result = load i64, ptr %state_result_field
    call void @promise_resolve(ptr %outer_promise, i64 %result)
    ret void
}
```

---

## 29. The Complete Memory Model Spectrum

```
╔════════════════════════════════════════════════════════════════════════════════╗
║          COMPLETE HYBRID MEMORY MODEL — Stack + Arena + Owned + CIRC + RAII   ║
╠═══════════════╦══════════════╦════════════════╦═══════════════════════════════╣
║ Layer         ║ Allocation   ║ Memory Free    ║ RAII (drop_fn call)           ║
╠═══════════════╬══════════════╬════════════════╬═══════════════════════════════╣
║ Stack         ║ alloca       ║ stack pop      ║ function epilogue / early ret  ║
║ Arena         ║ bump ptr     ║ arena_destroy  ║ dtor_list (RAII_EXTERNAL only) ║
║ Owned         ║ malloc       ║ free()         ║ ASAP last-use + scope guards   ║
║ Shared(CIRC)  ║ malloc+hdr   ║ free() in      ║ circ_destroy (RC=0) or         ║
║               ║              ║ circ_destroy   ║ Bacon-Rajan cycle collect      ║
╠═══════════════╩══════════════╩════════════════╩═══════════════════════════════╣
║ Cross-cutting: RAII Protocol (zero developer syntax required)                  ║
║                                                                                ║
║   Detection (Lifecycle Inference Pass):                                        ║
║     • Source 1: Resource Type Registry — 100% confidence, silent              ║
║     • Source 2: Acquisition/release verb matching — 95% confidence            ║
║     • Source 3: DRA — fills missing paths from Sources 1/2/4                  ║
║     • Source 4: try/finally and Promise.finally() elevation to landing pads   ║
║                                                                                ║
║   Enforcement (MIR + LLVM):                                                    ║
║     • drop_fn in vtable — compiler-generated for every resource-holding class ║
║     • ImplicitScopeGuardPush/Flush — inferred, not written                    ║
║     • ScopeGuardCancel before explicit programmer release — no double-release  ║
║     • Exception safety via LLVM invoke + landing pads (zero happy-path cost)   ║
║     • Async RAII via injected state machine disposal states                   ║
║     • RAII transfer to closures via RaiiTransferToCapture                     ║
║     • Global resources via libc atexit registry                               ║
║     • Only annotation: // @raii — escape hatch for ambiguous cases            ║
╚════════════════════════════════════════════════════════════════════════════════╝
```

---

## 30. Full Directory and Crate Structure

### New Crate: `crates/lifecycle-inference/`

```
crates/lifecycle-inference/
└── src/
    ├── lib.rs
    ├── rdt.rs               ← ResourceDescriptorTable, ResourceDescriptor, enums
    ├── pass.rs              ← LifecycleInferencePass (phases A–E)
    ├── dra.rs               ← Definite Release Analysis (forward + backward DFA)
    ├── pattern_match.rs     ← Source 2: ACQUISITION_VERBS, RELEASE_VERBS, pairing
    ├── try_finally.rs       ← Source 4: try/finally and Promise.finally() detectors
    ├── transfer.rs          ← Phase E: escape + transfer analysis
    └── conflict.rs          ← Cases 1–4 conflict resolution
```

### Updated `crates/semantic/`

```
crates/semantic/src/
├── raii_builtins.rs         ← RESOURCE_TYPE_REGISTRY + tscompiler.toml merging
├── lifecycle_patterns.rs    ← ACQUISITION_VERBS, RELEASE_VERBS, heuristics
└── [existing files unchanged]
```

### Updated `rt-stubs/src/`

```
rt-stubs/src/
├── arena.rs                 (extended: dtor_list for RAII_EXTERNAL shapes)
├── circ.rs                  (extended: drop_fn call in circ_destroy)
├── circ_nursery.rs
├── circ_birc.rs             (extended: BiRC-aware drop_fn_and_free)
├── rc_delta.rs
├── cycle_collector.rs       (extended: reverse topological drop order for cycles)
├── cycle_buffer.rs
├── capture_cell.rs
├── weak_ref.rs
├── finalization.rs
├── arena_pool.rs
├── verify.rs
│
├── raii/
│   ├── mod.rs
│   ├── scope_guard.rs       ← ScopeGuard stack: push / flush / cancel / flush-to
│   ├── drop_protocol.rs     ← drop_fn call separated from free — protocol rules
│   ├── suppressed_error.rs  ← SuppressedError for multi-guard throws
│   └── async_dispose.rs     ← Injected state machine helpers, timeout racing
│
└── node_compat/
    ├── fs.rs                (extended: FileHandle struct with RAII drop_fn)
    ├── net.rs               (extended: Socket/Server with RAII drop_fn)
    ├── child_process.rs     (extended: ChildProcess with RAII drop_fn)
    ├── path.rs
    └── os.rs
```

---

## 31. Hard Constraints

| Constraint | Reason |
|---|---|
| `drop_fn` must NEVER call `free(self)` | The allocator layer always calls `free` after `drop_fn`. Calling it inside `drop_fn` causes a double-free. |
| `drop_fn` must be idempotent where possible | Async disposal states may be entered from multiple code paths; use the `fd = -1` sentinel or `released_flag` guard to prevent double-release. |
| `RAII_EXTERNAL` shapes cannot be arena-allocated without `dtor_list` | Arena bulk-free skips destructors. External resources (OS handles, sockets) would leak. |
| `ScopeGuardFlush` must be emitted at EVERY scope exit | Missing a single exit (early return, break inside loop, rethrow in nested catch) causes a resource leak on that path. |
| `async_drop_fn` must always be awaited — never fire-and-forget | Returning before async dispose completes leaves the resource in an undefined state. Use `Promise.race` with a timeout. |
| Resources captured by closures that outlive the scope must cancel the source scope guard | If both the scope and the closure call `drop_fn`, the resource is double-released. |
| CIRC objects with timing-sensitive `drop_fn` must be `Owned` | CIRC `drop_fn` is deferred by cycle collection. Use `// @unique` to force `Owned`. |
| `drop_fn` for cycle members must be called in reverse topological order | Ensures members can safely access their still-live peers during destruction. |
| Never insert a release not already in the programmer's type signature or call graph | Inserting arbitrary `close()` calls on unknown types is wrong. Only act on known resource types or proven lifecycle patterns. |
| `TransferredTo` bindings never get implicit release in the source scope | Release responsibility has moved. Emitting a release in the source scope = double-release at the target. |
| Module-level resources use `atexit`, not scope end | Module-level bindings have no conventional scope exit. `atexit` is the correct mechanism. |
| When confidence < 80%: warn but do not insert | Incorrect implicit release is worse than a leak. Leaks are debuggable. Wrong teardown corrupts state. |
| DRA must handle every CFG edge including exception edges, loop back-edges, and cross-scope break/continue | Missing a single edge means a resource leak on that path. Use LLVM `invoke`/`landingpad` for exception edges. |
| The Lifecycle Inference Pass is read-only — it does not modify the HIR | It only produces the RDT. MIR lowering performs all instrumentation. Strict separation of analysis and transformation. |

---

*End of RAII Integration — Final Architecture.*
*Version: 2.0.0*
*The complete hybrid memory model: Stack + Arena + Owned + CIRC + RAII (cross-cutting, fully implicit).*
