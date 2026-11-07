//! Float trace log for the trace build (`RUSTFLAGS=--cfg fptrace`, melee_fptrace's `#[trace]` attribute;
//! docs/rust-port/notes/semantics.md, round 3). Matched by value against the Gekko interpreter's log
//! (tools/rust-port/fptrace/match.py).
//!
//! Env: `MELEE_FPTRACE_FN=<name>[,<name>...]` (the traced functions: logging is on from its entry to its return, callees included),
//! `MELEE_FPTRACE_FRAME=<slippi frame>` (first call on or after it; the engine reports frames with [`set_frame`]),
//! `MELEE_FPTRACE_NTH=<k>` (skip the first k such calls), `MELEE_FPTRACE_MINOPS=<n>` (skip calls with fewer float ops), `MELEE_FPTRACE_OUT=<file>` (TSV), or `MELEE_FPTRACE_DIR=<dir>` with `MELEE_FPTRACE_PER=<k>` (the first k calls of each function, as `<dir>/<name>.f<frame>.c<call in frame>.tsv`), `MELEE_FPTRACE_EXIT=1` (exit the process after writing).
//! In a normal build nothing calls into here except the cheap `on()` checks in `fp`.
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU32, Ordering};
use std::sync::Mutex;

static ON: AtomicBool = AtomicBool::new(false);
static DONE: AtomicBool = AtomicBool::new(false);
static DEPTH: AtomicU32 = AtomicU32::new(0);
static FRAME: AtomicI32 = AtomicI32::new(i32::MIN);
static LOG: Mutex<Vec<Rec>> = Mutex::new(Vec::new());
static STACK: Mutex<Vec<&'static str>> = Mutex::new(Vec::new());

struct Rec {
    op: &'static str,
    func: &'static str,
    ins: Vec<u64>,
    out: u64,
    double: bool,
}

/// The engine's current slippi frame (called once per frame by the trace build's driver).
pub fn set_frame(f: i32) {
    FRAME.store(f, Ordering::Relaxed);
}

#[inline(always)]
pub fn on() -> bool {
    ON.load(Ordering::Relaxed)
}

pub struct Guard {
    outer: Option<(&'static str, i32, u32)>,
    pushed: bool,
}

struct Cfg {
    fns: Vec<String>,
    frame: Option<i32>,
    nth: u32,
    minops: usize,
    per: u32,
    out: String,
    dir: Option<String>,
    exit: bool,
}

fn cfg() -> &'static Cfg {
    static C: std::sync::OnceLock<Cfg> = std::sync::OnceLock::new();
    C.get_or_init(|| {
        let e = |k: &str| std::env::var(k).ok();
        Cfg {
            fns: e("MELEE_FPTRACE_FN").map(|s| s.split(',').map(|x| x.trim().to_string()).collect()).unwrap_or_default(),
            frame: e("MELEE_FPTRACE_FRAME").and_then(|s| s.parse().ok()),
            nth: e("MELEE_FPTRACE_NTH").and_then(|s| s.parse().ok()).unwrap_or(0),
            minops: e("MELEE_FPTRACE_MINOPS").and_then(|s| s.parse().ok()).unwrap_or(0),
            per: e("MELEE_FPTRACE_PER").and_then(|s| s.parse().ok()).unwrap_or(1),
            out: e("MELEE_FPTRACE_OUT").unwrap_or_else(|| "fptrace_rust.tsv".into()),
            dir: e("MELEE_FPTRACE_DIR"),
            exit: e("MELEE_FPTRACE_EXIT").is_some(),
        }
    })
}

#[derive(Default)]
struct FnState {
    frame: i32,
    idx: u32,   // calls so far in `frame`
    seen: u32,  // calls on or after the frame filter
    written: u32,
}

static FNS: Mutex<Vec<(&'static str, FnState)>> = Mutex::new(Vec::new());

/// Entry of a `#[trace]` function.
pub fn enter(name: &'static str) -> Guard {
    let mut outer = None;
    if !on() && !DONE.load(Ordering::Relaxed) {
        let c = cfg();
        if c.fns.iter().any(|w| w == name) {
            let f = FRAME.load(Ordering::Relaxed);
            let mut fns = FNS.lock().unwrap();
            let i = match fns.iter().position(|(n, _)| *n == name) {
                Some(i) => i,
                None => {
                    fns.push((name, FnState { frame: i32::MIN, ..Default::default() }));
                    fns.len() - 1
                }
            };
            let st = &mut fns[i].1;
            if st.frame != f {
                st.frame = f;
                st.idx = 0;
            }
            let idx = st.idx;
            st.idx += 1;
            if c.frame.is_none_or(|ff| f >= ff) && st.written < c.per {
                st.seen += 1;
                if st.seen > c.nth {
                    ON.store(true, Ordering::Relaxed);
                    LOG.lock().unwrap().clear();
                    outer = Some((name, f, idx));
                }
            }
        }
    }
    let pushed = on();
    if pushed {
        STACK.lock().unwrap().push(name);
        DEPTH.fetch_add(1, Ordering::Relaxed);
    }
    Guard { outer, pushed }
}

impl Drop for Guard {
    fn drop(&mut self) {
        if self.pushed {
            STACK.lock().unwrap().pop();
            DEPTH.fetch_sub(1, Ordering::Relaxed);
        }
        if let Some((name, frame, idx)) = self.outer {
            ON.store(false, Ordering::Relaxed);
            let c = cfg();
            if LOG.lock().unwrap().len() < c.minops.max(1) {
                return; // too small: wait for the next call
            }
            let path = match &c.dir {
                Some(d) => format!("{d}/{name}.f{frame}.c{idx}.tsv"),
                None => c.out.clone(),
            };
            write_out(&path, frame, idx);
            let mut fns = FNS.lock().unwrap();
            if let Some((_, st)) = fns.iter_mut().find(|(n, _)| *n == name) {
                st.written += 1;
            }
            let all = c.fns.iter().all(|w| fns.iter().any(|(n, st)| n == w && st.written >= c.per));
            if c.dir.is_none() || all {
                DONE.store(true, Ordering::Relaxed);
                if c.exit {
                    std::process::exit(0);
                }
            }
        }
    }
}

fn write_out(path: &str, frame: i32, idx: u32) {
    let log = LOG.lock().unwrap();
    let mut s = format!("# frame {frame} call {idx}
seq	func	op	prec	inputs	out
");
    for (i, r) in log.iter().enumerate() {
        let ins: Vec<String> = r.ins.iter().map(|b| format!("{b:016X}")).collect();
        s += &format!("{}	{}	{}	{}	{}	{:016X}
", i + 1, r.func, r.op, if r.double { "d" } else { "s" },
                      ins.join(","), r.out);
    }
    let _ = std::fs::write(path, s);
    eprintln!("fptrace: {} float ops -> {path}", log.len());
}

/// A float's value as the double bit pattern (f32 widened exactly), and whether it is an f64.
#[inline]
pub fn bits<T: ?Sized>(x: &T) -> Option<(u64, bool)> {
    let n = std::any::type_name::<T>();
    // SAFETY: the type name says exactly which primitive T is.
    unsafe {
        match n {
            "f32" => Some(((*(x as *const T as *const f32) as f64).to_bits(), false)),
            "f64" => Some(((*(x as *const T as *const f64)).to_bits(), true)),
            "&f32" | "&mut f32" => Some(((**(x as *const T as *const &f32) as f64).to_bits(), false)),
            "&f64" | "&mut f64" => Some(((**(x as *const T as *const &f64)).to_bits(), true)),
            _ => None,
        }
    }
}

fn push(op: &'static str, ins: &[Option<(u64, bool)>], out: Option<(u64, bool)>) {
    if !on() {
        return;
    }
    let Some((o, od)) = out else { return };
    if ins.iter().any(|x| x.is_none()) {
        return;
    }
    let func = STACK.lock().unwrap().last().copied().unwrap_or("?");
    LOG.lock().unwrap().push(Rec { op, func, ins: ins.iter().map(|x| x.unwrap().0).collect(), out: o, double: od });
}

/// Logs a binary operator (the attribute's expansion calls this with the operands' bits taken before the operation).
#[inline]
pub fn log2(op: &'static str, a: Option<(u64, bool)>, b: Option<(u64, bool)>, out: Option<(u64, bool)>) {
    if on() {
        push(op, &[a, b], out);
    }
}

/// Logs an `as` conversion to a float (logged only when the source is a float of the other width or an integer).
#[inline]
pub fn log_cast<S>(src: &S, out: Option<(u64, bool)>) {
    if !on() {
        return;
    }
    let n = std::any::type_name::<S>();
    let srcb = bits(src);
    match (srcb, out) {
        (Some((sb, sd)), Some((_, od))) if sd != od => push(if od { "cvt_f64" } else { "frsp" }, &[Some((sb, sd))], out),
        (None, Some(_)) if n.starts_with('i') || n.starts_with('u') => push("i2f", &[], out),
        _ => {}
    }
}

/// Logs a helper (fmadds, fnmsubs, sqrtf...): operands and result as f32 / f64 values.
#[inline]
pub fn log_helper(op: &'static str, ins: &[f64], out: f64, double: bool) {
    if on() {
        let v: Vec<Option<(u64, bool)>> = ins.iter().map(|x| Some((x.to_bits(), double))).collect();
        push(op, &v, Some((out.to_bits(), double)));
    }
}
