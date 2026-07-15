use std::sync::{Mutex, Condvar};
use once_cell::sync::Lazy;
use crossbeam_deque::{Injector, Stealer, Worker};

type Task = Box<dyn FnOnce() + Send + 'static>;

static GLOBAL_QUEUE: Lazy<Injector<Task>> = Lazy::new(|| Injector::new());
static SLEEP_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));
static SLEEP_CV: Lazy<Condvar> = Lazy::new(|| Condvar::new());

pub fn enqueue_microtask<F>(task: F)
where
    F: FnOnce() + Send + 'static,
{
    GLOBAL_QUEUE.push(Box::new(task));
    SLEEP_CV.notify_one();
}

pub fn wake_all_microtasks() {
    SLEEP_CV.notify_all();
}

fn find_task(local: &Worker<Task>, stealers: &[Stealer<Task>]) -> Option<Task> {
    // Pop a task from the local queue, if not empty.
    local.pop().or_else(|| {
        // Otherwise, we need to look for a task elsewhere.
        std::iter::repeat_with(|| {
            // Try stealing a batch of tasks from the global queue.
            GLOBAL_QUEUE.steal_batch_and_pop(local)
                // Or try stealing a task from one of the other threads.
                .or_else(|| stealers.iter().map(|s| s.steal()).collect())
        })
        // Loop while no task was stolen and any steal operation needs to be retried.
        .find(|s| !s.is_retry())
        // Extract the stolen task, if there is one.
        .and_then(|s| s.success())
    })
}

#[no_mangle]
pub extern "C-unwind" fn __bs_execute_async_main(promise_tagged: u64) {
    crate::promise::reactor::init_reactor();
    let num_threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
    
    let mut workers = Vec::new();
    let mut stealers = Vec::new();

    for _ in 0..num_threads {
        let worker = Worker::new_fifo();
        stealers.push(worker.stealer());
        workers.push(worker);
    }

    // Spawn background worker threads
    for _ in 1..num_threads {
        let worker = workers.pop().unwrap();
        let stealers_clone = stealers.clone();
        std::thread::spawn(move || {
            worker_loop(worker, stealers_clone);
        });
    }

    let local_worker = workers.pop().unwrap();
    let ptr = (promise_tagged & 0x0000_FFFF_FFFF_FFFF) as *mut Mutex<crate::promise::Promise>;

    loop {
        // Check root promise
        {
            let p = unsafe { (*ptr).lock().unwrap() };
            if p.state != crate::promise::PromiseState::Pending {
                match p.state {
                    crate::promise::PromiseState::Rejected(_) => {
                        println!("Fatal error: Unhandled Top-Level Await rejection.");
                        std::process::exit(1);
                    }
                    _ => return, // Done!
                }
            }
        }
        
        unsafe { crate::finalization::__bs_drain_finalizers(); }

        let task = find_task(&local_worker, &stealers);
        if let Some(task) = task {
            task();
            crate::cycle_collector::__bs_cycle_collector_flush();
        } else {
            crate::cycle_collector::__bs_cycle_collector_flush();
            let timeout = crate::promise::reactor::get_next_timeout();
            if timeout.is_none() {
                let mut lock = SLEEP_LOCK.lock().unwrap();
                if GLOBAL_QUEUE.is_empty() && local_worker.is_empty() {
                    let (new_lock, timeout_res) = SLEEP_CV.wait_timeout(lock, std::time::Duration::from_millis(50)).unwrap();
                    lock = new_lock;
                    
                    if timeout_res.timed_out() && GLOBAL_QUEUE.is_empty() && local_worker.is_empty() {
                        let p = unsafe { (*ptr).lock().unwrap() };
                        if p.state == crate::promise::PromiseState::Pending {
                            println!("Fatal error: Top-Level Await deadlock. Promise is pending but all queues are empty.");
                            std::process::exit(1);
                        }
                    }
                }
            } else {
                let expired = crate::promise::reactor::poll_events();
                for cb in expired {
                    local_worker.push(cb);
                }
            }
        }
    }
}

fn worker_loop(local: Worker<Task>, stealers: Vec<Stealer<Task>>) {
    loop {
        let task = find_task(&local, &stealers);
        if let Some(task) = task {
            task();
            crate::cycle_collector::__bs_cycle_collector_flush();
        } else {
            crate::cycle_collector::__bs_cycle_collector_flush();
            let timeout = crate::promise::reactor::get_next_timeout();
            if timeout.is_none() {
                let mut lock = SLEEP_LOCK.lock().unwrap();
                if GLOBAL_QUEUE.is_empty() && local.is_empty() {
                    lock = SLEEP_CV.wait(lock).unwrap();
                }
            } else {
                let expired = crate::promise::reactor::poll_events();
                for cb in expired {
                    local.push(cb);
                }
            }
        }
    }
}

