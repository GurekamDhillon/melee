//! A small persistent pool for the game thread's fork-join work (the asset replay's block writes): the threads are
//! made once and parked, so a split costs a wake-up instead of a thread spawn per worker (Windows: ~50-100 us each, and
//! three per big file at a match start). The threads live on the system heap (`arena::SysScope`), never the arena.
//!
//! [`run`]`(n, f)` calls `f(0)` on the caller and `f(1..n)` on pool threads, and returns when all are done (a scoped
//! fork-join: `f` may borrow the caller's stack). A panic in a worker is re-raised on the caller.

use std::sync::atomic::{AtomicUsize, Ordering::*};
use std::sync::{Condvar, Mutex, OnceLock};

/// Workers (the caller is the n-th participant).
const MAX: usize = 7;

struct Job {
    /// `&(dyn Fn(usize) + Sync)` with its lifetime erased; valid until `left` reaches 0
    f: *const (dyn Fn(usize) + Sync),
    i: usize,
}
unsafe impl Send for Job {}

struct Slot {
    job: Mutex<Option<Job>>,
    cv: Condvar,
}

struct Pool {
    slots: Vec<&'static Slot>,
    left: AtomicUsize,
    panicked: AtomicUsize,
    done: Mutex<()>,
    done_cv: Condvar,
}

static POOL: OnceLock<Option<&'static Pool>> = OnceLock::new();
/// One fork-join at a time (the game thread is the only caller; a second caller runs serially).
static BUSY: Mutex<()> = Mutex::new(());

fn pool() -> Option<&'static Pool> {
    *POOL.get_or_init(|| {
        let _s = crate::arena::SysScope::enter();
        let p: &'static Pool = Box::leak(Box::new(Pool {
            slots: (0..MAX).map(|_| &*Box::leak(Box::new(Slot { job: Mutex::new(None), cv: Condvar::new() }))).collect(),
            left: AtomicUsize::new(0),
            panicked: AtomicUsize::new(0),
            done: Mutex::new(()),
            done_cv: Condvar::new(),
        }));
        for (k, &slot) in p.slots.iter().enumerate() {
            let ok = std::thread::Builder::new().name(format!("melee-worker-{k}")).spawn(move || loop {
                let job = {
                    let mut g = slot.job.lock().unwrap_or_else(|e| e.into_inner());
                    loop {
                        if let Some(j) = g.take() {
                            break j;
                        }
                        g = slot.cv.wait(g).unwrap_or_else(|e| e.into_inner());
                    }
                };
                // SAFETY: the caller of `run` waits for `left` to reach 0 before `f` goes out of scope
                let f = unsafe { &*job.f };
                if std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(job.i))).is_err() {
                    p.panicked.fetch_add(1, Relaxed);
                }
                if p.left.fetch_sub(1, AcqRel) == 1 {
                    let _g = p.done.lock().unwrap_or_else(|e| e.into_inner());
                    p.done_cv.notify_one();
                }
            });
            if ok.is_err() {
                return None;
            }
        }
        Some(p)
    })
}

/// Make the pool now (its spawns then never land in a timed frame).
pub fn warm() {
    let _ = pool();
}

/// `f(0)` here and `f(1)..f(n-1)` on the pool, all done on return. Without a pool (or more parts than workers + 1,
/// or another fork-join running) the parts beyond what the pool takes run here, in order.
pub fn run(n: usize, f: &(dyn Fn(usize) + Sync)) {
    let guard = if n > 1 { BUSY.try_lock().ok() } else { None };
    let Some(p) = guard.as_ref().and_then(|_| pool()) else {
        (0..n).for_each(f);
        return;
    };
    let k = (n - 1).min(MAX);
    p.left.store(k, Release);
    p.panicked.store(0, Relaxed);
    // SAFETY: lifetime erased; every worker is done with it before this function returns (the wait below)
    let fp: *const (dyn Fn(usize) + Sync) = unsafe { core::mem::transmute::<&(dyn Fn(usize) + Sync), &'static (dyn Fn(usize) + Sync)>(f) };
    for (w, slot) in p.slots.iter().take(k).enumerate() {
        *slot.job.lock().unwrap_or_else(|e| e.into_inner()) = Some(Job { f: fp, i: w + 1 });
        slot.cv.notify_one();
    }
    let mine = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        f(0);
        for i in k + 1..n {
            f(i);
        }
    }));
    // wait: a short spin (the parts are ~1 ms), then the condvar
    let mut spins = 0u32;
    while p.left.load(Acquire) != 0 && spins < 20_000 {
        core::hint::spin_loop();
        spins += 1;
    }
    if p.left.load(Acquire) != 0 {
        let mut g = p.done.lock().unwrap_or_else(|e| e.into_inner());
        while p.left.load(Acquire) != 0 {
            g = p.done_cv.wait(g).unwrap_or_else(|e| e.into_inner());
        }
    }
    if let Err(e) = mine {
        std::panic::resume_unwind(e);
    }
    assert!(p.panicked.load(Relaxed) == 0, "workers: a worker part panicked");
}

#[cfg(test)]
mod tests {
    #[test]
    fn runs_every_part_once() {
        use std::sync::atomic::{AtomicU64, Ordering::*};
        for n in [1usize, 2, 4, 8, 12] {
            let hits: Vec<AtomicU64> = (0..n).map(|_| AtomicU64::new(0)).collect();
            super::run(n, &|i| {
                hits[i].fetch_add(1, Relaxed);
            });
            assert!(hits.iter().all(|h| h.load(Relaxed) == 1), "n = {n}");
        }
    }
}
