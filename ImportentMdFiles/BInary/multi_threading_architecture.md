# BinScript Advanced Concurrency & Memory Architecture: A Deep Dive

BinScript departs radically from the traditional JavaScript single-threaded Event Loop. Instead of relying on a single execution thread for all JavaScript code and pushing I/O to background threads (like V8 or Node.js), BinScript executes *all* asynchronous state machines concurrently across a high-performance **M:N Work-Stealing Thread Pool**, paired with a native **Multi-Reactor Async Architecture** and **Thread-Local Garbage Collection**.

This architecture brings BinScript's performance profile closer to modern systems languages like **Go** or **Rust (Tokio)**, unlocking true parallel computing capabilities with zero global lock contention, while maintaining the familiar `async`/`await` syntax of TypeScript.

---

## 1. The Death of the Single-Threaded Event Loop

In traditional JavaScript engines, all synchronous code executes on a single main thread. If you run a CPU-intensive `while` loop, the entire application freezes—even if you have 16 CPU cores sitting idle. Asynchronous operations merely schedule callbacks to run *back on that exact same single thread* once the I/O operation completes.

**BinScript's Approach:**
BinScript abandons interpreters and Just-In-Time (JIT) engines entirely. Instead, it compiles `async` functions Ahead-Of-Time (AOT) into raw **LLVM Native Coroutines**. 

When an `await` is encountered, the LLVM coroutine yields control. Its internal state machine (variables, execution pointer) is preserved, and the coroutine pointer is decoupled from the current OS thread. It becomes a standalone "Microtask" that is tossed into a lock-free work queue. Any idle CPU core in the thread pool can immediately pick it up, restore the registers, and resume its execution.

This means that if you spawn 10 `async` tasks in BinScript, they will literally execute in parallel across 10 different OS physical cores simultaneously.

---

## 2. The M:N Work-Stealing Scheduler

At the core of BinScript's concurrency model is a lock-free, ultra-low-overhead Work-Stealing Scheduler powered by `crossbeam-deque`. 

When a BinScript application boots up, it detects the number of available physical CPU cores (e.g., 8) and spawns an equal number of OS Worker Threads.

### The Anatomy of the Scheduler:
1. **The Global Injector (`GLOBAL_QUEUE`)**: 
   When a completely new task is spawned (or an I/O event resolves a promise from the OS), the task is pushed to a global `Injector`. This is a thread-safe, lock-free queue designed for high-throughput multi-producer/multi-consumer ingestion.
   
2. **Local Worker Queues (`Worker<Task>`)**: 
   Each OS thread has its own private lock-free Ring Buffer (`Worker`). Pushing to and popping from this local queue requires absolutely no atomic synchronization with other threads, yielding execution speeds indistinguishable from synchronous function calls.

3. **The Stealer Algorithm**:
   - **Pop Local**: An active thread will always attempt to pop a task from its own local queue first.
   - **Steal Global**: If its local queue is empty, the thread looks at the `GLOBAL_QUEUE` and "steals" a batch of tasks, moving them into its local queue to amortize atomic access costs.
   - **Steal from Peers**: If the global queue is also empty, the thread becomes a "thief". It iterates over the lock-free `Stealer` handles of all *other* OS threads and attempts to steal half of their pending tasks. This guarantees perfect CPU saturation and load-balancing.

---

## 3. Multi-Reactor Architecture (`io_uring` & `mio`)

Historically, BinScript relied on a single background Reactor thread to multiplex I/O. However, in extreme-throughput scenarios (millions of concurrent connections), a single reactor thread becomes a severe bottleneck because all 16 CPU cores must contest for a single lock to register file or network events.

To obliterate this bottleneck, BinScript implemented a **Thread-Local Multi-Reactor Architecture**.

### The Thread-Local Reactor Model
1. **Independent OS Ring Buffers**: Every single worker thread is initialized with its own private `LocalReactor` bound to a `thread_local!` slot. There are zero cross-thread mutexes.
2. **True Zero-Copy via `io_uring` (Linux)**: On Linux, the reactor utilizes the ultra-modern **`io_uring`** kernel subsystem. 
   - When an `async` task running on Thread A needs a timer (`sleep()`), it formats a `__kernel_timespec` and pushes a `Timeout` Submission Queue Entry (SQE) directly into Thread A's private kernel memory ring.
   - This bypasses standard system call overhead. The kernel processes the timer asynchronously, and Thread A later consumes the Completion Queue Entry (CQE) when the timer fires.
3. **Graceful Degradation**: On macOS and Windows, the build system uses conditional compilation (`#[cfg(not(target_os = "linux"))]`) to gracefully fall back to an `epoll`/`kqueue`/`IOCP` backed `mio::Poll` instance per thread.

### The Ultimate `worker_loop` State Machine
The worker loop perfectly interleaves computation, GC, and I/O polling without ever needlessly parking the thread:
1. Execute a task from the Stealer queues.
2. Immediately flush Thread-Local GC to cleanup any garbage generated by the task.
3. If no tasks are available to steal, calculate the delta to the next pending `io_uring` timeout.
4. Issue an `io_uring_enter` syscall to sleep the thread precisely until the I/O event fires.
5. Wake up, process the I/O callbacks, push them into the Local Queue, and loop back to step 1.

---

## 4. Concurrency-Safe Memory Management (BiRC + Thread-Local GC)

Handling memory in a highly parallel, M:N scheduled environment is notoriously difficult. A global Garbage Collector with "Stop-the-World" (STW) pauses destroys the low-latency guarantees required by modern backend servers.

BinScript solves this using a custom, heavily optimized variant of **Bacon-Rajan Concurrent Cycle Collection (BiRC)**.

### Lock-Free Shared Ownership
Every object is allocated with a `CircHeader` containing two distinct Reference Counts:
- **`local_rc` (Non-Atomic)**: Mutated incredibly fast (simple CPU increment) by the thread that currently "owns" the object. This bypasses L1/L2 cache-line invalidation bottlenecks entirely.
- **`global_rc` (Atomic)**: Mutated via hardware atomics (`lock xadd`) by *other* threads (e.g., if Thread B drops an object that was originally created by Thread A).

When a thread observes `local_rc + global_rc == 0`, the memory is instantly freed to a custom Slab Allocator.

### Thread-Local Garbage Accumulation
Not all objects reach `RC == 0` immediately. If an object is suspected of participating in a memory leak cycle (a parent pointing to a child, and the child pointing back to the parent), it is painted "Purple" and flagged as a cycle candidate.

In legacy BinScript, these candidates were pushed to a global locking array, causing severe contention. In the modern architecture, candidates are appended to a `RefCell<Vec<*mut CircHeader>>` stored in a `thread_local!` slot. 

### Idle-Time Flushes (Zero Pause)
Garbage collection does not happen asynchronously on a background thread. Background threads inevitably cause cache-trashing, lock contention, and unpredictable memory spikes.

Instead, cycle collection is embedded directly into the Work-Stealing loop. When an OS thread exhausts its local task queue, *before* it polls its `io_uring` reactor, it invokes `__bs_cycle_collector_flush()`. 

The thread independently traces, scans, and frees memory cycles for its own local objects. This means:
1. **Zero Pauses**: Active threads crunching complex math or serving HTTP requests are *never* interrupted by GC pauses.
2. **Scavenging Idle Time**: Idle threads automatically perform memory cleanup, utilizing dead CPU time that would otherwise be wasted.
3. **Linear Scaling**: Garbage collection throughput scales perfectly linearly with the number of CPU cores. More cores = faster garbage cleanup.

---

## 5. Proactive Deadlock Detection

Because BinScript executes Top-Level Await (TLA) state machines without a centralized, single-threaded coordinator, it incorporates a proactive Deadlock Detector within the runtime loop.

If the main module's Promise is still pending, but all Worker Queues are empty, and there are no active `io_uring` events or timers pending in any of the Thread-Local Reactors, the runtime mathematically proves that the program will never progress. Instead of hanging indefinitely and confusing the developer, it safely crashes with a clear diagnostic error: 

`Fatal error: Top-Level Await deadlock. Promise is pending but all queues are empty.`

---

## 6. Q&A: Are We Handling Async/Await the "Right Way"?

A common question when migrating away from a single-threaded Event Loop is whether the new concurrency model is architecturally sound. The short answer is **Yes**, BinScript handles `async/await` the exact same way modern, high-performance languages like **Rust (Tokio)** do.

Here is a breakdown of why this approach is state-of-the-art, and the one major "gotcha" to be aware of:

### Why Our Approach is State-of-the-Art:

1. **Escaping the "Node.js Trap" (Single-Threaded JS)**
   Node.js and Deno use a single event loop. If you run a heavy `for` loop in an `async` function, the entire server freezes. In BinScript, because the `async` state machine is decoupled from the main thread and pushed into the lock-free `GLOBAL_QUEUE`, your `async` tasks execute in **true parallel** across all physical CPU cores. We turned JavaScript into a natively multi-threaded language.

2. **Zero-Overhead State Machines**
   Languages like Go and Java (Project Loom) use "Green Threads," which allocate actual memory stacks (e.g., 2KB-8KB) for every task. If you spawn 1 million tasks, you consume gigabytes of RAM. 
   Instead, BinScript uses **LLVM Native Coroutines**. When a function `await`s, LLVM calculates exactly which variables are alive and generates a perfectly sized, highly compact C++ style `struct` to hold them. This means our `async` tasks consume almost zero memory footprint compared to Green Threads.

3. **M:N Lock-Free Scheduling**
   By combining `crossbeam-deque` for work-stealing and a `thread_local!` multi-reactor with `io_uring`, we eliminated central bottlenecks. A thread never locks the whole runtime to fetch its next microtask or register a timer. This is the exact architecture that allows Rust web frameworks (like Axum/Actix) to handle millions of requests per second.

### The One Catch: Cooperative vs. Preemptive Scheduling

Because we went the Rust/LLVM route instead of the Go route, our `async` functions are **Cooperative**, not Preemptive.

**What does that mean?**
If a developer writes this in BinScript:
```typescript
async function badCode() {
    while (true) {
        // do heavy math, never call await
    }
}
```
Because the code never hits an `await` statement, the state machine never yields control back to the `Worker`. This will permanently hijack one of our OS worker threads. If the user spawns 8 of these on an 8-core machine, the entire BinScript scheduler will starve, and no other `async` tasks will run.

In **Go**, the runtime has a background thread (`sysmon`) that forcefully pauses a running Goroutine if it takes too long, allowing others to run (Preemptive). 

### Is Cooperative Scheduling Bad?
No. **Rust**, **C#**, **C++20**, and **Node.js** all use Cooperative scheduling. It is significantly faster and allows for easier integration with LLVM compiler optimizations. Developers simply must understand that they shouldn't run infinite synchronous loops inside `async` tasks. If they have a massive math calculation, they should manually `await sleep(0)` periodically to yield control back to the Work-Stealing scheduler and let other tasks run.

---

## Conclusion

By surgically migrating away from global background threads and fully embracing **Thread-Local Isolation**, BinScript brings the extreme-performance threading philosophy of modern infrastructure directly into the TypeScript ecosystem:
- **LLVM Coroutines** for syntactical simplicity (`async` / `await`) at native C++ speeds.
- **M:N Threading** to harness 100% of multi-core CPUs.
- **Lock-free Stealing** to minimize latency and synchronization bottlenecks.
- **Multi-Reactors (`io_uring`)** to eradicate kernel-level event registration bottlenecks.
- **Thread-Local BiRC GC** for zero-pause, linearly scalable memory reclamation that happens strictly during idle cycles.
