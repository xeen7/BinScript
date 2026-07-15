# BinScript Concurrency Architecture: Work-Stealing Scheduler

*Author: Senior Systems Engineering Team*
*Goal: Escaping the Single-Threaded JS Event Loop for Native, High-Performance M:N Concurrency.*

## 1. Executive Summary

JavaScript’s traditional concurrency model relies on a single-threaded Event Loop. While simple, it becomes a severe bottleneck for CPU-bound tasks, forcing developers into clunky workarounds like Web Workers. 

**BinScript takes a fundamentally different approach.** By compiling `async/await` down into Zero-Cost State Machine Generators, we decouple the *task* from the *execution thread*. This allows BinScript to implement an **M:N Work-Stealing Scheduler**, closely mirroring the architecture of the **Go Scheduler (Goroutines)** and **Rust's Tokio Runtime**.

In this model, **M** BinScript tasks (State Machines) are multiplexed dynamically across **N** OS Threads (Worker Threads). When you `await`, the task yields control, and the OS thread instantly grabs another task from the pool.

---

## 2. Core Components

### A. The "Task" (The State Machine)
Currently, BinScript lowers `async` functions into state machines returning `Promise` objects. 
To support multi-threading, we wrap this state machine into a `Task` struct.
- **State**: The internal variables and current suspension point (the `yield` / `await` index).
- **Waker**: A callback that pushes the `Task` back into the execution queue when its blocking operation (I/O, Timer, or child Promise) completes.

### B. The Worker Threads (The Executors)
At startup, BinScript spawns a pool of OS threads equal to the number of logical CPU cores (e.g., 8 cores = 8 OS threads).
Each OS thread runs a continuous loop:
1. Pop a `Task` from its queue.
2. Call `resume()` on the task's state machine.
3. If the task returns `Pending`, it has suspended itself (e.g., waiting for I/O). The thread drops it and loops back to step 1.
4. If the task returns `Done`, the thread records the result and resolves any dependent promises.

### C. The Queues (Work-Stealing Mechanism)
To minimize lock contention, we use a decentralized queue architecture:
- **Global Queue**: A shared, mutex-protected queue for newly spawned tasks. 
- **Local Run Queue**: Each OS thread has its own lock-free local queue. Threads primarily push and pop from their own local queues to avoid cache-invalidation and lock contention.

---

## 3. The Work-Stealing Algorithm

This is the magic that ensures maximum CPU utilization:

1. **Local Execution**: A Worker Thread always tries to pop a task from its **Local Run Queue** first.
2. **Global Polling**: If the Local Queue is empty, it checks the **Global Queue** and grabs a batch of tasks.
3. **The Steal**: If *both* the Local and Global queues are empty, the thread becomes a "Thief". It randomly selects another Worker Thread and attempts to "steal" half of the tasks from the victim's Local Queue. 

**Why this matters:** If Core 1 gets bogged down with a heavy synchronous calculation, Cores 2, 3, and 4 will automatically steal Core 1's pending tasks. No manual load balancing is required by the developer.

---

## 4. The Reactor (I/O Polling)

What happens when a task `awaits` a network request? It cannot block the OS Thread, or we waste a CPU core.
1. The Task registers interest in a file descriptor (e.g., a TCP Socket) with the **Reactor**.
2. The Task yields `Pending` back to the Worker Thread.
3. The Worker Thread grabs a new Task.
4. The Reactor runs on a background thread utilizing OS-level asynchronous primitives (`epoll` on Linux, `kqueue` on macOS, `IOCP` on Windows). 
5. When the OS signals that the socket is ready, the Reactor triggers the Task's `Waker`, which pushes the Task back into the Global Queue to be picked up by the next available Worker Thread.

---

## 5. Developer API (Syntax)

To the BinScript developer, this massive system is entirely invisible. The syntax remains identical to standard JavaScript/TypeScript, but with native multithreaded performance.

```typescript
// 1. Spawning a background task (Runs on any free CPU core)
const p1 = spawn(async () => {
    return heavyComputation();
});

const p2 = spawn(async () => {
    return networkRequest();
});

// 2. The current thread suspends, and the OS thread instantly picks up other work.
// 3. When p1 and p2 are done, this task is woken up and resumes.
const [res1, res2] = await Promise.all([p1, p2]);
```
*(Note: `spawn` replaces the need for `new Worker()`, acting like a Go `go` routine or `tokio::spawn`)*

---

## 6. Implementation Roadmap

To get BinScript from its current single-threaded polling loop to a full Work-Stealing Scheduler, we will follow these phases:

### Phase 1: The Waker Abstraction
- Modify `rt-stubs/src/promise` to implement a `Waker` system. Instead of the `main` wrapper busy-looping, Promises should actively push their dependent tasks back into a central queue when resolved.

### Phase 2: The Multi-Threaded Executor
- Introduce a Thread Pool in the Rust runtime (`rt-stubs`).
- Replace the `__bs_drain_microtasks` loop with a threaded executor that pulls from a thread-safe Global Queue.

### Phase 3: Work Stealing & Local Queues
- Upgrade the Executor to use Local Queues per thread and implement the "Steal" logic using a library like `crossbeam-deque`.

### Phase 4: The Reactor (Async I/O)
- Integrate an OS-polling backend (like `mio`) into `rt-stubs` to handle non-blocking File and Network I/O, allowing tasks to yield to the OS natively.
