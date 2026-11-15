//! Log lines from worker threads, printed by the game thread.
//!
//! A rollback load rewrites the executable's statics, std's own among them: the stderr handle's lock and `RefCell`.
//! A worker thread (the asset service's `asset-worker`, which runs m-ex decodes that log) printing while a load
//! rewrote that state found the `RefCell` "already borrowed", panicked inside std's stdio, and the synctest that ran
//! the load hung (ACE subspace_check, `mexdata: kind 33: motion rows ...`). So in this crate `eprintln!` (shadowed in
//! `lib.rs`) comes here: on a worker the line is queued in a block on the system heap, made before the first
//! snapshot; the game thread prints the queue before its own next line. A worker never touches stdio.

use std::sync::{Mutex, OnceLock};

thread_local! {
    static WORKER: core::cell::Cell<bool> = const { core::cell::Cell::new(false) };
}

/// The queue. Set once, before any worker starts (so before any snapshot): its value is never rewritten.
static Q: OnceLock<&'static Mutex<Vec<String>>> = OnceLock::new();

/// Make the queue (call on the game thread at start-up, before any worker logs).
pub fn init() {
    Q.get_or_init(|| {
        let _s = crate::arena::SysScope::enter();
        &*Box::leak(Box::new(Mutex::new(Vec::new())))
    });
}

/// The calling thread is a worker: its lines are queued, never printed by it.
pub fn mark_worker() {
    WORKER.with(|w| w.set(true));
}

/// One log line (the shadowed `eprintln!`).
pub fn line(s: String) {
    if WORKER.with(|w| w.get()) {
        // a worker before `init` drops the line rather than touch stdio
        if let Some(q) = Q.get() {
            q.lock().unwrap_or_else(|e| e.into_inner()).push(s);
        }
        return;
    }
    drain();
    std::eprintln!("{s}");
}

/// Print the workers' queued lines (game thread).
pub fn drain() {
    let Some(q) = Q.get() else { return };
    let _s = crate::arena::SysScope::enter();
    let v = core::mem::take(&mut *q.lock().unwrap_or_else(|e| e.into_inner()));
    for l in v {
        std::eprintln!("{l}");
    }
}
