# Exception Mechanism: `setjmp`/`longjmp` vs `invoke`/`landingpad`

> **Context:** BinScript currently uses `setjmp`/`longjmp` for exception propagation.
> The memory model (`memory_model_final.md`) and RAII protocol (`raii_final.md`)
> require scope guard flushing on every exit path including exception paths.
> This document resolves the Phase 5C question: which mechanism to use, why,
> and the exact migration path.
>
> **Verdict:** Replace `setjmp`/`longjmp` with LLVM `invoke`/`landingpad` entirely.
> Neither option A nor option B as originally framed is correct.

---

## Table of Contents

1. [Why the Original Options Are Both Wrong](#1-why-the-original-options-are-both-wrong)
2. [What `longjmp` Actually Does at the Machine Level](#2-what-longjmp-actually-does-at-the-machine-level)
3. [Why Option B Must Be Rejected](#3-why-option-b-must-be-rejected)
4. [Why Option A Is Incomplete](#4-why-option-a-is-incomplete)
5. [How `invoke`/`landingpad` Works](#5-how-invokelanding-pad-works)
6. [Performance Comparison](#6-performance-comparison)
7. [Safety Comparison](#7-safety-comparison)
8. [Integration with Scope Guards](#8-integration-with-scope-guards)
9. [Integration with CIRC and Owned Destruction](#9-integration-with-circ-and-owned-destruction)
10. [Integration with Async State Machines](#10-integration-with-async-state-machines)
11. [The Migration Path from `setjmp`/`longjmp`](#11-the-migration-path-from-setjmplongjmp)
12. [Personality Function](#12-personality-function)
13. [Static Linking Considerations](#13-static-linking-considerations)
14. [MIR Changes](#14-mir-changes)
15. [LLVM IR Emission Changes](#15-llvm-ir-emission-changes)
16. [Updated Hard Constraints](#16-updated-hard-constraints)

---

## 1. Why the Original Options Are Both Wrong

The two options as framed assume `setjmp`/`longjmp` is the correct foundation and
ask only how to layer RAII on top of it. That assumption is wrong. `longjmp` cannot
call destructors — not as a fixable limitation, but as a fundamental property of
its machine-level semantics. Any option that keeps `longjmp` as the exception
mechanism cannot provide sound RAII on exception paths.

```
Option A — "integrate scope guard flushing into longjmp propagation"
  Problem: longjmp executes NO code between the throw site and the setjmp site.
           There is nothing to integrate into. Scope guard flushing requires code
           execution during unwind. longjmp cannot provide that.

Option B — "defer exception-path RAII, only handle normal control flow"
  Problem: This is not a trade-off. It is a permanent correctness hole.
           File descriptors, mutex guards, database connections, and owned memory
           all leak permanently on every exception path. This makes the RAII model
           entirely unsound. Reject unconditionally.

Correct answer: Replace setjmp/longjmp with invoke/landingpad.
  — invoke/landingpad is what RAII on exception paths requires.
  — The migration is mechanical and maps existing MIR constructs directly.
  — Normal-path performance is strictly better than setjmp.
  — Exception-path correctness is guaranteed by the LLVM unwinding infrastructure.
```

---

## 2. What `longjmp` Actually Does at the Machine Level

`longjmp` unwinds the call stack by **restoring a saved register state**. When
`setjmp` is called it saves the current values of the stack pointer, frame pointer,
instruction pointer, and callee-saved registers into a `jmp_buf`. When `longjmp`
is called it restores those registers, effectively jumping the CPU back to where
`setjmp` was called with the stack pointer restored to that point.

```
Call stack before longjmp:

  main()          ← jmp_buf saved here by setjmp
    └─ tryBlock()
         └─ inner()
              └─ deepest()   ← longjmp called here

After longjmp:

  main()          ← execution resumes here
  (tryBlock, inner, deepest are gone — stack pointer restored)
  (NO code in tryBlock, inner, or deepest executed during the jump)
```

The stack frames for `tryBlock`, `inner`, and `deepest` are **gone**. Not unwound.
Not cleaned up. Physically overwritten by subsequent stack growth. Any local
variables in those frames — including scope guards, owned allocations, and CIRC
reference counts — are abandoned without any cleanup code running.

This is not a missing feature. `longjmp` by design executes zero instructions
between the throw site and the catch site. It is a raw register restore.

### What Happens to Each Memory Class Under `longjmp`

| Memory class | Fate under `longjmp` |
|---|---|
| Stack | Frame is popped — stack memory is reclaimed, but `drop_fn` is never called. External resources (file handles, mutexes) leak. |
| Arena | `arena_destroy` is never called. All arena memory leaks for the lifetime of the process (or until the next arena reset). |
| Owned | `drop_fn` is never called. `free()` is never called. Memory and external resources leak permanently. |
| Shared (CIRC) | `circ_dec` is never called. RC never reaches zero. Object is never destroyed. Permanent leak. |

---

## 3. Why Option B Must Be Rejected

Option B — "defer exception-path RAII, handle only normal control flow" — produces
a system where every thrown exception is a resource leak event. Consider the most
common production patterns:

```typescript
// Every one of these leaks permanently when an exception propagates:

async function handleRequest(req: Request) {
    const conn = await db.connect();     // Leaked on exception — connection pool exhausted
    const lock = await mutex.acquire();  // Leaked on exception — deadlock
    const fd   = fs.openSync(log, 'a'); // Leaked on exception — fd table exhausted
    const tx   = db.begin();            // Leaked on exception — transaction never rolled back

    await processRequest(req, conn, lock, fd, tx);
    // If processRequest throws: all four resources leak permanently
}
```

In a long-running server, option B guarantees:

- Connection pool exhaustion after sufficient exceptions.
- Deadlocks from leaked mutex guards.
- File descriptor table exhaustion.
- Database transaction log growth without rollback.
- CIRC RC inflation causing permanent memory growth.

These are not edge cases. They are the normal failure modes of real servers under
load. Option B makes the compiler produce binaries that are correct only when
exceptions never occur — which is not a correctness guarantee at all.

**Option B must be rejected at the architecture level, not treated as a performance
trade-off.**

---

## 4. Why Option A Is Incomplete

Option A says "integrate scope guard flushing into the existing `longjmp`
propagation." The intention is sound — scope guards should flush on exception paths.
The problem is that `longjmp` provides no hook for executing code during the unwind.

The only way to make scope guards flush during a `longjmp` unwind is to:

1. Maintain a **runtime scope guard stack** as a thread-local linked list.
2. Before every `longjmp` call, manually walk this stack and call each guard's
   `drop_fn`.
3. At every `setjmp` site, record the current stack depth so the walk knows when
   to stop.
4. Handle nested `try/catch` by maintaining a stack of `jmp_buf`s alongside
   the guard stack.
5. Handle exceptions thrown from `drop_fn` during the manual walk (which requires
   another level of `setjmp`/`longjmp`).
6. Handle async state machines that may be suspended across a `try` block.
7. Handle exceptions propagating across module boundaries.

This is a complete reimplementation of stack unwinding — in software, without
hardware or OS support, without the decades of correctness work that LLVM's
unwinding infrastructure represents. The result would be:

- More complex than `invoke`/`landingpad` to implement.
- Slower on normal paths (the `setjmp` cost remains).
- Still incorrect in corner cases (exceptions from `drop_fn`, async unwind,
  cross-module propagation).
- Impossible to debug with standard tools (debuggers, sanitizers, profilers all
  understand DWARF unwinding; none understand a custom parallel guard stack).

Option A is the right instinct applied to the wrong foundation. The correct move
is to switch the foundation.

---

## 5. How `invoke`/`landingpad` Works

### The Model

Every call that may throw is emitted as an LLVM `invoke` instruction instead of
`call`. `invoke` names two successor blocks: the normal-path block (execution
continues here if the call returns normally) and the unwind block (execution
continues here if the call throws).

```llvm
; call (no exception handling):
%result = call ptr @do_work(ptr %arg)

; invoke (with exception handling):
%result = invoke ptr @do_work(ptr %arg)
          to label %normal_block
          unwind label %unwind_block
```

The `unwind_block` begins with a `landingpad` instruction that receives the
exception object from the LLVM unwinding infrastructure:

```llvm
unwind_block:
    %exn = landingpad { ptr, i32 }
               catch ptr null    ; catch all exceptions
    ; ... scope guard flush code ...
    resume { ptr, i32 } %exn    ; re-throw
```

### How the OS Unwinder Finds the `landingpad`

When an exception is thrown (via `__cxa_throw` or the compiler's throw primitive),
the OS unwinding library (`libunwind`) walks the call stack. For each stack frame,
it consults the DWARF exception tables in the `.eh_frame` section of the binary.
These tables are generated automatically by LLVM for every function that contains
an `invoke` instruction. The tables tell the unwinder:

- Which ranges of instructions in this function may throw.
- Which `landingpad` to jump to when an exception propagates through this frame.
- Which cleanup actions (destructor calls) to perform in this frame.

The unwinder is called by the C++ runtime. For a compiler that uses its own
exception system (not C++ exceptions), a custom **personality function** is
registered with the `landingpad` instruction to tell the unwinder how to interpret
the exception tables (see Section 12).

### Zero Normal-Path Overhead

The `.eh_frame` section is a read-only table stored in the binary. On the normal
(non-exception) path, it is never consulted. There are no branches, no checks,
no register saves. A function with `invoke` instructions is identical in performance
to the same function with `call` instructions on the normal path.

The cost is paid only when an exception is actually thrown — which is the correct
trade-off for error handling (errors are exceptional; normal paths must not pay
for them).

---

## 6. Performance Comparison

### Normal-Path Performance

| Mechanism | Cost per `try` block entry | Cost per function call within `try` |
|---|---|---|
| `setjmp`/`longjmp` | Full register-state save to `jmp_buf` (~20–40 instructions depending on ISA) | `setjmp` must be re-entered on each nested try; call itself is plain `call` |
| `invoke`/`landingpad` | **Zero** — exception tables are static data, not executed | Call is `invoke` instead of `call` — one extra successor operand in the IR, identical machine code |

On x86-64, `setjmp` saves 8 callee-saved registers, the stack pointer, the frame
pointer, and the return address — typically 17 words. This happens at every `try`
block entry, even when exceptions are never thrown. In a server that processes
millions of requests per second, each with multiple `try` blocks, this is measurable.

### Exception-Path Performance

| Mechanism | Exception propagation cost |
|---|---|
| `setjmp`/`longjmp` + manual guard stack | Walk thread-local guard list in software; call each drop_fn manually; restore `jmp_buf` state |
| `invoke`/`landingpad` | OS unwinder walks DWARF tables; calls personality function; jumps to `landingpad`; `drop_fn` calls emitted directly in LLVM IR |

Both pay a cost on the exception path. The `invoke`/`landingpad` path is slightly
heavier per frame (DWARF table lookup), but it is correct, parallelisable across
frames, and handled by a mature, optimised runtime library. The manual guard stack
approach cannot be parallelised and must be correct in software.

### Summary

```
Normal path:   invoke/landingpad wins by a large margin (zero overhead vs setjmp cost)
Exception path: invoke/landingpad wins on correctness; roughly equivalent on raw speed
Overall:        invoke/landingpad is strictly better on both axes
```

---

## 7. Safety Comparison

### Memory Safety

| Scenario | `setjmp`/`longjmp` | `invoke`/`landingpad` |
|---|---|---|
| Owned value in try block, exception thrown | `drop_fn` never called — leak + potential OS handle leak | `landingpad` calls `drop_fn` + `free` — correct |
| CIRC object in try block, exception thrown | `circ_dec` never called — RC inflation — permanent leak | `circ_dec` called in `landingpad` — correct |
| Mutex guard in try block, exception thrown | `unlock` never called — deadlock | `drop_fn` calls `unlock` in `landingpad` — correct |
| `drop_fn` itself throws during cleanup | Undefined — nested `longjmp` corrupts the first `jmp_buf` chain | `landingpad` catches it; `SuppressedError` constructed; re-thrown after all guards flushed |
| Stack variable with destructor, exception thrown | Stack frame abandoned — destructor silently skipped | LLVM emits cleanup for the stack frame automatically |
| Exception across module boundary | `jmp_buf` is not accessible across module; undefined behaviour | DWARF tables in each module; unwinder handles transparently |
| Exception in a thread other than the catching thread | `jmp_buf` is not shareable across threads; undefined | Per-thread unwinding; each thread has its own exception state |

### C Standard Compliance

The C standard (C11 §7.13.2.1) states:

> "The `longjmp` function restores the environment saved by the most recent
> invocation of the `setjmp` macro in the same invocation of the program...
> If there has been no such invocation, or if the function containing the
> invocation of the `setjmp` macro has terminated execution in the interim...
> the behavior is undefined."

Undefined behaviour under the C standard is triggered by `longjmp` whenever:

- The function that called `setjmp` has returned (stack frame is gone).
- The `setjmp` was called in a different thread.
- C++ objects with destructors exist in the frames being unwound.

The third point is directly relevant: any C++ RAII object (including those from
Rust's FFI boundary) in a frame unwound by `longjmp` triggers undefined behaviour.
This includes any use of `crossbeam-channel`, `slabmalloc`, or any Rust crate
linked into `rt-stubs` that has destructors — which is all of them.

`invoke`/`landingpad` has no such restriction. It is defined behaviour in all cases.

---

## 8. Integration with Scope Guards

With `invoke`/`landingpad`, scope guard flushing on the exception path is identical
to scope guard flushing on the normal path — it is just LLVM IR in a different
basic block.

### Single Resource

```llvm
define void @process(ptr %path) personality ptr @__binscript_personality_v0 {
entry:
    ; Acquire resource — ScopeGuardPush was emitted in MIR
    %fd = call ptr @alloc_file_handle(ptr %path)

    ; Call that may throw — emitted as invoke
    %data = invoke ptr @read_all(ptr %fd)
            to label %normal
            unwind label %unwind

normal:
    ; Normal path: ScopeGuardFlush
    call void @FileHandle_drop_fn(ptr %fd)
    call void @free(ptr %fd)
    ; ... use data ...
    ret void

unwind:
    %exn = landingpad { ptr, i32 } catch ptr null
    ; Exception path: SAME ScopeGuardFlush (compiler emits identical sequence)
    call void @FileHandle_drop_fn(ptr %fd)
    call void @free(ptr %fd)
    resume { ptr, i32 } %exn    ; re-throw original exception
}
```

### Multiple Resources — Reverse Order

```llvm
define void @handle(ptr %req) personality ptr @__binscript_personality_v0 {
entry:
    ; Two resources acquired (fd first, lock second)
    %fd   = call ptr @alloc_file_handle(ptr %log_path)
    %lock = call ptr @alloc_mutex_guard(ptr @global_mutex)

    %result = invoke ptr @do_work(ptr %req)
              to label %normal
              unwind label %unwind

normal:
    ; Reverse acquisition order: lock first, then fd
    call void @MutexGuard_drop_fn(ptr %lock)
    call void @free(ptr %lock)
    call void @FileHandle_drop_fn(ptr %fd)
    call void @free(ptr %fd)
    ret void

unwind:
    %exn = landingpad { ptr, i32 } catch ptr null
    ; Exception path: identical reverse-order flush
    call void @MutexGuard_drop_fn(ptr %lock)
    call void @free(ptr %lock)
    call void @FileHandle_drop_fn(ptr %fd)
    call void @free(ptr %fd)
    resume { ptr, i32 } %exn
}
```

### `drop_fn` Throws During Flush

When `drop_fn` itself may throw (e.g. `conn.end()` fails), the scope guard flush
must catch that exception, continue flushing remaining guards, then combine all
thrown exceptions into a `SuppressedError`:

```llvm
unwind:
    %exn = landingpad { ptr, i32 } catch ptr null

    ; Try to flush lock guard — may itself throw
    %lock_result = invoke void @MutexGuard_drop_fn(ptr %lock)
                   to label %lock_flushed
                   unwind label %lock_threw

lock_threw:
    %lock_exn = landingpad { ptr, i32 } catch ptr null
    call void @suppressed_error_push(ptr %lock_exn)    ; accumulate
    br label %lock_flushed

lock_flushed:
    call void @free(ptr %lock)

    ; Continue: flush fd guard — may also throw
    %fd_result = invoke void @FileHandle_drop_fn(ptr %fd)
                 to label %fd_flushed
                 unwind label %fd_threw

fd_threw:
    %fd_exn = landingpad { ptr, i32 } catch ptr null
    call void @suppressed_error_push(ptr %fd_exn)      ; accumulate
    br label %fd_flushed

fd_flushed:
    call void @free(ptr %fd)

    ; Combine original exception + any suppressed exceptions
    %final_exn = call ptr @suppressed_error_combine(ptr %exn)
    resume { ptr, i32 } %final_exn
}
```

The MIR lowering generates this pattern for every scope that has active guards at
a throw site. The `suppressed_error_push` and `suppressed_error_combine` functions
live in `rt-stubs/src/raii/suppressed_error.rs` (already part of the RAII module
from `raii_final.md`).

---

## 9. Integration with CIRC and Owned Destruction

On the exception path, CIRC `RcDec` and Owned `Drop` instructions are emitted
in the `landingpad` block exactly as they would be on the normal path. The MIR
liveness analysis identifies which `MirReg`s are live at each `invoke` instruction.
For every live `MirReg` that is `Owned` or `Shared(CIRC)`, the corresponding
cleanup (`Drop` or `RcDec`) is emitted in the `landingpad` block.

```llvm
unwind:
    %exn = landingpad { ptr, i32 } catch ptr null

    ; Live Owned value at this invoke site:
    call void @BigBuffer_drop_fn(ptr %buf)
    call void @free(ptr %buf)

    ; Live CIRC value at this invoke site:
    %hdr = getelementptr i8, ptr %shared_obj, i64 -8   ; back to CircHeader
    call void @circ_dec(ptr %hdr)

    resume { ptr, i32 } %exn
```

This is the same cleanup that ASAP destruction would have emitted on the normal
path — the liveness interval ends at the same instruction whether the exit is
normal or exceptional.

---

## 10. Integration with Async State Machines

Async state machine unwinding is the most complex case. When an exception propagates
out of an `invoke` inside a `poll` function (the compiled async function body), the
`landingpad` must:

1. Flush all scope guards active at the suspension point.
2. Call any injected async disposal states (from `raii_final.md` Section 19) that
   were registered for the suspended function.
3. Reject the outer `Promise` with the exception.

```llvm
; Inside an async state machine's poll() function:

state_1_body:
    ; ... work at state 1 ...
    %result = invoke ptr @some_async_op(ptr %conn)
              to label %state_2
              unwind label %state_1_exception

state_1_exception:
    %exn = landingpad { ptr, i32 } catch ptr null

    ; Flush scope guards registered for state 1
    call void @conn_drop_fn(ptr %conn_guard)
    call void @free(ptr %conn_guard)

    ; Transition state machine to disposal states (injected RAII dispose)
    store i32 DISPOSE_STATE_INDEX, ptr %state_field
    ; Reject the outer promise
    call void @promise_reject(ptr %outer_promise, ptr %exn)
    ret void    ; return to event loop — disposal states run on next poll
```

The disposal states (injected by the Lifecycle Inference Pass for async resources
like `pg.Client` connections) run on the next poll cycle, properly awaiting each
async `drop_fn`. The exception is held in the promise's rejection reason until
all disposal is complete.

---

## 11. The Migration Path from `setjmp`/`longjmp`

The migration is **phased and mechanical**. No change to HIR. No change to the
semantic pass. No change to the ownership inference pass. Changes are confined to
MIR interpretation and LLVM codegen.

### Phase 1 — MIR Reinterpretation

Map the existing `TryEnter` and `Throw` MIR instructions to new semantics without
changing their structure:

```rust
// BEFORE: TryEnter sets up a jmp_buf; Throw calls longjmp
// AFTER:
//   TryEnter — marks the current scope as "exception-aware"
//              Records the set of active scope guards at this point
//              Registers the personality function on the current LLVM function
//   Throw    — signals that the preceding call must be emitted as invoke
//              Names the unwind target block

pub enum MirInstr {
    // ... existing instructions ...

    // Reinterpreted: no longer generates setjmp. Now marks the scope.
    TryEnter { scope_id: ScopeId, catch_target: BlockId },

    // Reinterpreted: no longer generates longjmp. Now marks the call as invoke.
    // The actual invoke emission happens at the CallDirect/CallVTable/CallDynamic
    // site when that site is inside a TryEnter scope.
    Throw { exception_reg: MirReg },

    // NEW: explicit re-throw after landingpad cleanup
    Resume { exn_reg: MirReg },
}
```

No existing MIR that BinScript already generates becomes invalid. The instructions
exist and are retained; their codegen changes.

### Phase 2 — LLVM Codegen: `call` → `invoke`

In `codegen-llvm/src/emit.rs`, the emit function for call instructions checks
whether the current position is inside a `TryEnter` scope with active guards.
If yes: emit `invoke`. If no: emit plain `call`.

```rust
fn emit_call(
    &mut self,
    callee: FunctionValue<'ctx>,
    args: &[BasicValueEnum<'ctx>],
    name: &str,
) -> CallSiteValue<'ctx> {
    let may_throw = self.current_call_may_throw();
    let has_guards = self.has_active_scope_guards();

    if may_throw && has_guards {
        let normal_bb = self.ctx.append_basic_block(self.current_fn, "normal");
        let unwind_bb = self.get_or_create_unwind_block();
        self.builder.build_invoke(callee, args, normal_bb, unwind_bb, name)
            .expect("invoke build failed");
        self.builder.position_at_end(normal_bb);
        // Return a sentinel — actual value is in normal_bb
        // (inkwell's invoke returns InstructionValue; handle appropriately)
    } else {
        self.builder.build_call(callee, args, name)
            .expect("call build failed")
    }
}
```

`may_throw` is determined conservatively: any call to a non-`@nothrow` function
may throw. Functions marked with the `// @nothrow` pragma (from `raii_final.md`
Section 18) are emitted as plain `call` even inside a `TryEnter` scope.

### Phase 3 — Unwind Block Generation

For each function that has at least one `invoke`, generate the unwind block:

```rust
fn get_or_create_unwind_block(&mut self) -> BasicBlock<'ctx> {
    if let Some(bb) = self.current_unwind_block {
        return bb;
    }

    let unwind_bb = self.ctx.append_basic_block(self.current_fn, "unwind");
    let save_pos  = self.builder.get_insert_block();

    self.builder.position_at_end(unwind_bb);

    // Emit landingpad instruction
    let lp_ty = self.ctx.struct_type(&[
        self.ctx.ptr_type(AddressSpace::default()).into(),
        self.ctx.i32_type().into(),
    ], false);
    let lp = self.builder.build_landingpad(lp_ty, self.personality_fn, 0, "exn")
        .expect("landingpad failed");
    lp.set_cleanup(true);   // catch all exceptions for cleanup purposes

    // Emit scope guard flush for all active guards (reverse push order)
    self.emit_scope_guard_flush_all();

    // Emit CIRC/Owned cleanup for live values at this point
    self.emit_live_value_cleanup_on_exception();

    // Re-throw
    self.builder.build_resume(lp.as_basic_value_enum())
        .expect("resume failed");

    // Restore position
    if let Some(pos) = save_pos {
        self.builder.position_at_end(pos);
    }

    self.current_unwind_block = Some(unwind_bb);
    unwind_bb
}
```

### Phase 4 — Remove `setjmp`/`longjmp` Infrastructure

Once Phase 3 is complete and all exception paths go through `landingpad`:

- Remove `jmp_buf` allocation from the `TryEnter` codegen path.
- Remove `longjmp` call from the `Throw` codegen path.
- Remove the thread-local `jmp_buf` stack from `rt-stubs`.
- Remove the `setjmp.h` include from `rt-stubs/src/lib.rs`.

The `TryEnter` MIR instruction now solely marks scopes as exception-aware.
The `Throw` MIR instruction now solely marks the preceding call as `invoke`-eligible.

### Phase 5 — `catch` Clause Typing (Optional, after Phase 4)

For typed `catch` clauses (`catch (e: SpecificError)`), the personality function
must perform type matching. This is implemented in Phase 5 after the basic
`catch ptr null` (catch-all) is working correctly. See Section 12.

---

## 12. Personality Function

The personality function is called by the OS unwinder for each frame during stack
unwinding. It decides whether the current frame's `landingpad` matches the thrown
exception, and if so, transfers control to the `landingpad` block.

### Minimal Implementation: `__gxx_personality_v0`

For an initial implementation, use the C++ personality function directly. This is
available via `libunwind` (statically linkable) and handles `landingpad` with
`catch ptr null` correctly — it matches every exception and transfers control to
the cleanup block.

```llvm
; Declare the personality function in every function that has a landingpad:
define void @myFunc() personality ptr @__gxx_personality_v0 {
    ...
}
```

This works correctly for the scope guard flush use case. The limitation: typed
`catch` clauses require a custom personality function.

### Custom Personality: `__binscript_personality_v0`

For full typed exception handling (`catch (e: TypeError)`, `catch (e: RangeError)`),
implement a custom personality function in `rt-stubs/src/exception/personality.rs`:

```rust
// rt-stubs/src/exception/personality.rs
//
// The personality function is called by _Unwind_RaiseException for each frame.
// It must be a C-ABI function.

#[no_mangle]
pub unsafe extern "C" fn __binscript_personality_v0(
    version:        i32,
    actions:        UnwindAction,
    exception_class: u64,
    exception:      *mut UnwindException,
    context:        *mut UnwindContext,
) -> UnwindReasonCode {
    // Phase 1 (search): walk this frame's LSDA to find a matching catch clause
    // Phase 2 (cleanup): transfer control to the landingpad
    binscript_personality_impl(version, actions, exception_class, exception, context)
}
```

The LSDA (Language-Specific Data Area) is the compiler-generated table embedded
in the binary that describes which exception types each `landingpad` catches.
`inkwell` provides APIs to emit LSDA entries via `build_landingpad` clause types.

For BinScript's exception types (JavaScript `Error`, `TypeError`, `RangeError`,
user-defined error classes), the LSDA entry contains the `ShapeId` of the caught
type. The personality function compares the thrown exception's `ShapeId` against
the LSDA entry using `__instanceof` (already implemented in `rt-stubs/src/prototype.rs`).

### Implementation Strategy

Start with `__gxx_personality_v0` and `catch ptr null` for Phase 5C. This gets
correct cleanup (no resource leaks) on all exception paths immediately. Implement
`__binscript_personality_v0` with typed catch matching in a follow-on phase after
the cleanup infrastructure is validated.

---

## 13. Static Linking Considerations

### `libunwind`

The OS unwinding library must be linked. For fully static musl builds, use
LLVM's `libunwind` compiled as a static library:

```bash
# Build libunwind statically for the target:
cmake -DLIBUNWIND_ENABLE_SHARED=OFF \
      -DLIBUNWIND_ENABLE_STATIC=ON  \
      -DCMAKE_C_COMPILER=clang      \
      llvm-project/libunwind

# Link in the final binary:
# cargo:rustc-link-lib=static=unwind
# cargo:rustc-link-search=native=/path/to/libunwind/build/lib
```

`libunwind` is approximately 50 KB compiled statically. This replaces the `setjmp`
infrastructure at negligible size cost.

### `libgcc_s` Alternative

On Linux x86-64, `libgcc_s` provides the unwinding primitives and is available
as a static archive (`libgcc_eh.a`). Either `libunwind` or `libgcc_s` works.
`libunwind` is preferred for cross-compilation (it is part of the LLVM project
and builds cleanly for all LLVM targets).

### Link Command Update

```bash
lld \
  --lto-O3 \
  -static \
  entry.o module_a.bc module_b.bc ... \
  -l:librt_stubs.a \
  -l:libunwind.a \       # NEW: replaces setjmp (which was in libc)
  -l:libc.a \
  -o my_program
```

`setjmp`/`longjmp` are part of libc and do not require an extra link dependency.
Adding `libunwind` is the only link-line change.

---

## 14. MIR Changes

### Existing Instructions — Reinterpreted

```rust
pub enum MirInstr {
    // Reinterpreted from "set up jmp_buf" to "mark scope as exception-aware":
    // — Registers personality function on the enclosing LLVM function.
    // — Records the catch target block for this scope.
    // — Snapshots the active scope guard stack for use in unwind block generation.
    TryEnter {
        scope_id:     ScopeId,
        catch_target: BlockId,
    },

    // Reinterpreted from "call longjmp" to "mark the preceding call as invoke-eligible":
    // — Does not emit machine code directly.
    // — Sets a flag consumed by the next CallDirect/CallVTable/CallDynamic emit.
    Throw {
        exception_reg: MirReg,
    },

    // NEW: emit LLVM `resume` to re-throw after landingpad cleanup.
    // Emitted at the end of every unwind block after scope guard flush.
    Resume {
        exn_reg: MirReg,
    },

    // NEW: emit a `landingpad` instruction at the start of an unwind block.
    // Captures the exception object into exn_reg.
    LandingPad {
        exn_reg:         MirReg,
        is_cleanup:      bool,   // true = catch all for cleanup; false = typed catch
        catch_type_ids:  Vec<ShapeId>,  // empty if is_cleanup = true
    },
}
```

### New Instructions for Typed Catch

```rust
pub enum MirInstr {
    // ... existing instructions ...

    // Extract the exception object from a landingpad result.
    // Used in typed catch clauses to bind the exception to a name.
    ExtractException {
        dest:    MirReg,
        lp_reg:  MirReg,
    },

    // Test whether a caught exception matches a given shape (for typed catch).
    // Emits a call to __instanceof(exn, shape_id).
    ExceptionIsA {
        dest:      MirReg,
        exn_reg:   MirReg,
        shape_id:  ShapeId,
    },
}
```

---

## 15. LLVM IR Emission Changes

### Function Prologue: Register Personality

Every LLVM function that contains a `TryEnter` scope must declare a personality:

```rust
// In codegen-llvm/src/emit.rs, when processing TryEnter:
fn on_try_enter(&mut self, scope_id: ScopeId, catch_target: BlockId) {
    // Set personality on the current function (idempotent — safe to call multiple times)
    self.current_fn.set_personality_function(self.personality_fn);
    // Record the active scope guard snapshot for unwind block generation
    self.exception_scope_stack.push(ExceptionScope {
        scope_id,
        catch_target,
        guards_at_entry: self.scope_guard_stack.snapshot(),
        live_values_at_entry: self.liveness.snapshot(),
    });
}
```

### Call Sites: `call` vs `invoke`

```rust
fn emit_call_direct(&mut self, func: FuncId, args: &[MirOperand]) -> MirReg {
    let callee    = self.func_table[&func];
    let llvm_args = self.lower_operands(args);

    if self.is_in_exception_scope() && self.func_may_throw(func) {
        // Emit invoke
        let normal_bb = self.ctx.append_basic_block(self.current_fn, "invoke_normal");
        let unwind_bb = self.get_or_create_unwind_block();

        let invoke = self.builder.build_invoke(
            callee, &llvm_args, normal_bb, unwind_bb, "invoke_result"
        ).expect("invoke");

        self.builder.position_at_end(normal_bb);
        self.current_invoke_result = Some(invoke);
    } else {
        // Emit plain call — no overhead
        let call = self.builder.build_call(callee, &llvm_args, "call_result")
            .expect("call");
        self.current_call_result = Some(call);
    }
}
```

### Unwind Block: Landingpad + Flush + Resume

```rust
fn build_unwind_block(&mut self, scope: &ExceptionScope) -> BasicBlock<'ctx> {
    let unwind_bb = self.ctx.append_basic_block(self.current_fn, "unwind");
    self.builder.position_at_end(unwind_bb);

    // Landingpad — catch all for cleanup
    let lp_ty = self.ctx.struct_type(&[
        self.ptr_ty.into(),
        self.i32_ty.into(),
    ], false);
    let lp = self.builder.build_landingpad(lp_ty, self.personality_fn, 0, "lp")
        .expect("landingpad");
    lp.set_cleanup(true);

    // Flush scope guards in reverse order (same as normal path)
    for guard in scope.guards_at_entry.iter().rev() {
        self.emit_drop_fn_call(guard.reg);
        self.emit_free(guard.reg);
    }

    // Cleanup live Owned and CIRC values
    for live in scope.live_values_at_entry.iter() {
        match live.class {
            MemoryClass::Owned => {
                self.emit_drop_fn_call(live.reg);
                self.emit_free(live.reg);
            }
            MemoryClass::Shared => {
                self.emit_circ_dec(live.reg);
            }
            _ => {}
        }
    }

    // Re-throw
    self.builder.build_resume(lp.as_basic_value_enum())
        .expect("resume");

    unwind_bb
}
```

---

## 16. Updated Hard Constraints

These replace and extend the exception-related entries in `memory_model_final.md`
Section 31 and `raii_final.md` Section 31.

| Constraint | Reason |
|---|---|
| `setjmp`/`longjmp` must not be used for exception propagation in emitted code. | `longjmp` cannot execute code during stack unwinding. It is fundamentally incompatible with RAII, CIRC cleanup, and owned value destruction. |
| Every call that may throw, inside a scope with active guards or live Owned/CIRC values, must be emitted as `invoke`, not `call`. | Plain `call` has no unwind block; thrown exceptions bypass all cleanup. |
| Every LLVM function containing a `landingpad` must declare a personality function. | The OS unwinder requires the personality function pointer to be present in the function's DWARF frame record. Without it, unwinding through the frame is undefined behaviour. |
| The `landingpad` block must emit scope guard flushes in **reverse push order**, identical to the normal exit path. | Destruction order must be consistent between normal and exceptional exits. Inconsistent order causes use-after-free in destructors that reference sibling fields. |
| `drop_fn` calls in the `landingpad` must themselves be emitted as `invoke` if `drop_fn` may throw. | If a `drop_fn` throws and the call was a plain `call`, the exception propagates without flushing the remaining guards. |
| Accumulated exceptions from `drop_fn` throws must be combined into `SuppressedError` before `resume`. | All guards must flush regardless of whether earlier flushes threw. The original exception plus all suppressed exceptions must be reported. |
| The unwind block must clean up all live `Owned` and `Shared(CIRC)` values at the `invoke` site, not just scope-guard-registered resources. | Any `Owned` value whose liveness interval includes the `invoke` instruction has its `Drop` emitted in the unwind block. Any `Shared(CIRC)` value has its `RcDec` emitted there. |
| `libunwind` (or `libgcc_s`) must be statically linked. | Dynamic linking introduces a runtime dependency. The binary must be fully self-contained. |
| Functions proven to never throw (annotated `// @nothrow` or calling only `@nothrow` functions) must be emitted as plain `call` even inside exception-aware scopes. | Unnecessary `invoke` instructions inflate code size and may inhibit LLVM optimisations. |
| The `// @nothrow` annotation must be verified, not trusted blindly. | If a `@nothrow` function does throw, the unwind bypasses all cleanup. Verify at compile time by checking the call graph for any reachable throw sites. |

---

*End of Exception Mechanism Decision Document.*
*Version: 1.0.0*
*Decision: Replace `setjmp`/`longjmp` with LLVM `invoke`/`landingpad`.*
*Companion documents: `memory_model_final.md`, `raii_final.md`.*
