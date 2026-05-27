use std::sync::Mutex;
use once_cell::sync::Lazy;

/// A simple microtask queue.
/// In a real engine, this would be tied to the event loop.
static MICROTASK_QUEUE: Lazy<Mutex<Vec<Box<dyn FnOnce() + Send + 'static>>>> = Lazy::new(|| Mutex::new(Vec::new()));

pub fn enqueue_microtask<F>(task: F)
where
    F: FnOnce() + Send + 'static,
{
    let mut q = MICROTASK_QUEUE.lock().unwrap();
    q.push(Box::new(task));
}

/// Drains and executes all pending microtasks.
/// Call this after resolving a promise or at the end of the script.
#[no_mangle]
pub extern "C" fn __bs_drain_microtasks() {
    loop {
        let mut tasks = {
            let mut q = MICROTASK_QUEUE.lock().unwrap();
            if q.is_empty() {
                break;
            }
            std::mem::take(&mut *q)
        };

        for task in tasks.drain(..) {
            task();
        }
    }
}
