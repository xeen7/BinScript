use std::collections::BTreeMap;
use std::time::{Instant, Duration};
use std::cell::RefCell;

type TimerCallback = Box<dyn FnOnce() + Send + 'static>;

#[cfg(not(target_os = "linux"))]
pub struct LocalReactor {
    poll: mio::Poll,
    events: mio::Events,
    timers: BTreeMap<Instant, Vec<TimerCallback>>,
}

#[cfg(not(target_os = "linux"))]
impl LocalReactor {
    pub fn new() -> Self {
        Self {
            poll: mio::Poll::new().unwrap(),
            events: mio::Events::with_capacity(128),
            timers: BTreeMap::new(),
        }
    }
    
    pub fn add_timer(&mut self, expire_time: Instant, cb: TimerCallback) {
        self.timers.entry(expire_time).or_insert_with(Vec::new).push(cb);
    }
    
    pub fn get_next_timeout(&self) -> Option<Duration> {
        if let Some((&first_instant, _)) = self.timers.iter().next() {
            let now = Instant::now();
            if first_instant <= now {
                Some(Duration::from_millis(0))
            } else {
                Some(first_instant - now)
            }
        } else {
            None
        }
    }
    
    pub fn poll_events(&mut self) -> Vec<TimerCallback> {
        let timeout = self.get_next_timeout().unwrap_or(Duration::from_millis(10));
        
        self.poll.poll(&mut self.events, Some(timeout)).unwrap();
        
        let now = Instant::now();
        let mut expired_callbacks = Vec::new();
        
        let mut after_now = self.timers.split_off(&(now + Duration::from_nanos(1)));
        std::mem::swap(&mut self.timers, &mut after_now);
        for (_, cbs) in after_now {
            expired_callbacks.extend(cbs);
        }
        
        expired_callbacks
    }
    
    pub fn has_active_events(&self) -> bool {
        !self.timers.is_empty()
    }
}

#[cfg(target_os = "linux")]
pub struct LocalReactor {
    ring: io_uring::IoUring,
    timers: BTreeMap<Instant, Vec<TimerCallback>>,
}

#[cfg(target_os = "linux")]
impl LocalReactor {
    pub fn new() -> Self {
        Self {
            ring: io_uring::IoUring::new(128).unwrap(),
            timers: BTreeMap::new(),
        }
    }
    
    pub fn add_timer(&mut self, expire_time: Instant, cb: TimerCallback) {
        self.timers.entry(expire_time).or_insert_with(Vec::new).push(cb);
    }
    
    pub fn get_next_timeout(&self) -> Option<Duration> {
        if let Some((&first_instant, _)) = self.timers.iter().next() {
            let now = Instant::now();
            if first_instant <= now {
                Some(Duration::from_millis(0))
            } else {
                Some(first_instant - now)
            }
        } else {
            None
        }
    }
    
    pub fn poll_events(&mut self) -> Vec<TimerCallback> {
        let timeout = self.get_next_timeout().unwrap_or(Duration::from_millis(10));
        
        if timeout.as_millis() > 0 {
            let ts = io_uring::types::Timespec::new()
                .sec(timeout.as_secs())
                .nsec(timeout.subsec_nanos());
            
            unsafe {
                let sqe = io_uring::opcode::Timeout::new(&ts as *const _ as *const io_uring::types::Timespec).build().user_data(1);
                let mut sq = self.ring.submission();
                let _ = sq.push(&sqe);
                sq.sync();
            }
            
            self.ring.submit_and_wait(1).unwrap();
            
            unsafe {
                self.ring.completion().sync();
            }
        }
        
        let now = Instant::now();
        let mut expired_callbacks = Vec::new();
        let mut after_now = self.timers.split_off(&(now + Duration::from_nanos(1)));
        std::mem::swap(&mut self.timers, &mut after_now);
        for (_, cbs) in after_now {
            expired_callbacks.extend(cbs);
        }
        
        expired_callbacks
    }
    
    pub fn has_active_events(&self) -> bool {
        !self.timers.is_empty()
    }
}

thread_local! {
    pub static LOCAL_REACTOR: RefCell<LocalReactor> = RefCell::new(LocalReactor::new());
}

pub fn init_reactor() {
    // No-op
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_set_timeout(ms_tagged: u64) -> u64 {
    let ms_f64 = f64::from_bits(ms_tagged);
    let ms = if ms_f64.is_nan() { 0 } else { ms_f64 as u64 };
    
    let promise = crate::promise::__bs_promise_new();
    let expire_time = Instant::now() + Duration::from_millis(ms);
    let cb: TimerCallback = Box::new(move || {
        crate::promise::__bs_promise_resolve(promise, 0xFFF1_0000_0000_0000 /* undefined */);
    });
    
    LOCAL_REACTOR.with(|r| {
        r.borrow_mut().add_timer(expire_time, cb);
    });
    
    promise
}

pub fn has_active_events() -> bool {
    LOCAL_REACTOR.with(|r| r.borrow().has_active_events())
}

pub fn get_next_timeout() -> Option<Duration> {
    LOCAL_REACTOR.with(|r| r.borrow().get_next_timeout())
}

pub fn poll_events() -> Vec<TimerCallback> {
    LOCAL_REACTOR.with(|r| r.borrow_mut().poll_events())
}
