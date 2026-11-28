//! DOL referee capture (notes/referee.md): with the `referee` feature, functions carrying `referee_hook!` record
//! sampled calls (arguments, the bytes of the f32 records their pointer arguments name before and after the call,
//! the return value) to `MELEE_REFEREE_OUT`, and `mex-import referee` later runs the retail function from main.dol
//! on the same inputs and compares. Without the feature `referee_hook!` expands to nothing.
//!
//! Env (read once): `MELEE_REFEREE_OUT=<file>` turns capture on; `MELEE_REFEREE_FNS=a,b` limits it to those names;
//! `MELEE_REFEREE_MAX=<n>` calls kept per function (default 400); `MELEE_REFEREE_EVERY=<k>` keeps every k-th call
//! (default 1). A call nested inside a captured call is not captured (its result is part of the outer one).
//!
//! The file holds engine values computed from a replay (no disc bytes): it stays local, under `_build/`.

/// One argument of a captured call.
#[derive(Clone, Copy, Debug)]
pub enum Arg {
    /// An integer argument (r3.. in order of the integer arguments).
    I(u32),
    /// A float argument (f1.. in order).
    F(f32),
    /// A pointer argument to `len` bytes of f32 words (Vec, Mtx): the integer register gets a guest address of a
    /// big-endian copy of them.
    P(*const u8, usize),
    /// A native record pointer (native-records mode, `referee_native!`): the record's GC name, e.g. `HSD_GObj`; the
    /// integer register gets its guest handle (`remap::GuestSpace`).
    R(*mut u8, &'static str),
}

/// The return of a captured call.
#[derive(Clone, Copy, Debug)]
pub enum Ret {
    Void,
    I(u32),
    F(f32),
    P(*const u8),
}

#[cfg(feature = "referee")]
mod imp {
    use super::{Arg, Ret};
    use std::io::Write;
    use std::sync::Mutex;

    struct St {
        out: Option<std::io::BufWriter<std::fs::File>>,
        only: Option<Vec<String>>,
        max: u32,
        every: u32,
        seen: std::collections::HashMap<&'static str, (u32, u32)>,
    }
    static ST: Mutex<Option<St>> = Mutex::new(None);
    static ACTIVE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();

    fn init() -> bool {
        *ON.get_or_init(|| {
            let Some(p) = std::env::var_os("MELEE_REFEREE_OUT") else { return false };
            let Ok(f) = std::fs::File::create(&p) else {
                eprintln!("referee: cannot create the capture file");
                return false;
            };
            let only = std::env::var("MELEE_REFEREE_FNS").ok().map(|v| v.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect());
            let max = std::env::var("MELEE_REFEREE_MAX").ok().and_then(|v| v.parse().ok()).unwrap_or(400);
            let every = std::env::var("MELEE_REFEREE_EVERY").ok().and_then(|v| v.parse().ok()).unwrap_or(1u32).max(1);
            *ST.lock().unwrap() = Some(St { out: Some(std::io::BufWriter::new(f)), only, max, every, seen: Default::default() });
            true
        })
    }

    /// A call being captured: what went in, kept until `finish`.
    pub struct Cap {
        name: &'static str,
        frame: i32,
        args: Vec<Arg>,
        before: Vec<Vec<u8>>,
    }

    fn read(p: *const u8, n: usize) -> Vec<u8> {
        // SAFETY: the hook names a live record of `n` bytes (its declared size) for the duration of the call
        unsafe { core::slice::from_raw_parts(p, n) }.to_vec()
    }

    pub fn begin(name: &'static str, args: &[Arg]) -> Option<Cap> {
        if !init() || ACTIVE.load(std::sync::atomic::Ordering::Relaxed) {
            return None;
        }
        {
            let mut g = ST.lock().ok()?;
            let s = g.as_mut()?;
            if let Some(o) = &s.only {
                if !o.iter().any(|n| n == name) {
                    return None;
                }
            }
            let (max, every) = (s.max, s.every);
            let e = s.seen.entry(name).or_insert((0, 0));
            e.0 += 1;
            if (e.0 - 1) % every != 0 || e.1 >= max {
                return None;
            }
            e.1 += 1;
        }
        if args.iter().any(|a| matches!(a, Arg::P(p, _) if p.is_null())) {
            return None;
        }
        ACTIVE.store(true, std::sync::atomic::Ordering::Relaxed);
        let before = args.iter().filter_map(|a| if let Arg::P(p, n) = a { Some(read(*p, *n)) } else { None }).collect();
        Some(Cap { name, frame: crate::ct::DBG_FRAME.load(std::sync::atomic::Ordering::Relaxed), args: args.to_vec(), before })
    }

    fn hex(b: &[u8]) -> String {
        b.iter().map(|x| format!("{x:02x}")).collect()
    }

    impl Cap {
        pub fn finish(self, ret: Ret) {
            ACTIVE.store(false, std::sync::atomic::Ordering::Relaxed);
            let mut line = format!("C {} {} {}", self.name, self.frame, self.args.len());
            let mut k = 0;
            for a in &self.args {
                match a {
                    Arg::I(v) => line += &format!(" i:{v:08x}"),
                    Arg::F(v) => line += &format!(" f:{:08x}", v.to_bits()),
                    Arg::R(p, r) => line += &format!(" r:{r}:{:x}", *p as usize),
                    Arg::P(p, n) => {
                        line += &format!(" p:{:x}:{n}:{}:{}", *p as usize, hex(&self.before[k]), hex(&read(*p, *n)));
                        k += 1;
                    }
                }
            }
            match ret {
                Ret::Void => line += " R v",
                Ret::I(v) => line += &format!(" R i:{v:08x}"),
                Ret::F(v) => line += &format!(" R f:{:08x}", v.to_bits()),
                Ret::P(p) => line += &format!(" R p:{:x}", p as usize),
            }
            if let Ok(mut g) = ST.lock() {
                if let Some(o) = g.as_mut().and_then(|s| s.out.as_mut()) {
                    let _ = writeln!(o, "{line}");
                    let _ = o.flush();
                }
            }
        }
    }
}

#[cfg(feature = "referee")]
pub use imp::{begin, Cap};

/// Native-records mode (notes/referee.md section 10): the in-process checker, registered by `melee_engine::live_referee`.
/// `pre` runs the retail function on the live records (and undoes its writes) before the Rust call and returns a token,
/// `post` compares after it; `boundary` is called at the top of a Rust function retail's run may stop at.
#[derive(Clone, Copy)]
pub struct NativeHooks {
    pub wants: fn(&'static str) -> bool,
    pub pre: fn(&'static str, &[Arg]) -> Option<usize>,
    pub post: fn(usize, Ret),
    pub boundary: fn(&'static str, &[Arg]),
}

#[cfg(feature = "referee")]
static NATIVE: std::sync::OnceLock<NativeHooks> = std::sync::OnceLock::new();

/// Register the native-records checker (once per process).
#[cfg(feature = "referee")]
pub fn set_native(h: NativeHooks) {
    let _ = NATIVE.set(h);
}

#[cfg(feature = "referee")]
#[doc(hidden)]
pub fn native() -> Option<&'static NativeHooks> {
    NATIVE.get()
}

/// Check a call of the enclosing function in native-records mode: `referee_native!(name, [R(gobj, "HSD_GObj"),
/// I(x), F(y)], v|i|f, call)` at the top of the function, `call` re-invoking it (nested checks are not taken: the
/// checker refuses a `pre` while one is open).
#[cfg(feature = "referee")]
#[macro_export]
macro_rules! referee_native {
    ($name:ident, [$($kind:ident($($a:expr),*)),*], $ret:ident, $call:expr) => {
        if let Some(h) = $crate::referee::native() {
            if (h.wants)(stringify!($name)) {
                if let Some(t) = (h.pre)(stringify!($name), &[$($crate::referee_hook!(@arg $kind($($a),*))),*]) {
                    #[allow(unused_unsafe)]
                    let r = unsafe { $call };
                    (h.post)(t, $crate::referee_hook!(@ret $ret, r));
                    return r;
                }
            }
        }
    };
}
#[cfg(not(feature = "referee"))]
#[macro_export]
macro_rules! referee_native {
    ($($t:tt)*) => {};
}

/// A boundary callee of native-records mode: `referee_boundary!(name, [args])` at the top of the Rust function.
#[cfg(feature = "referee")]
#[macro_export]
macro_rules! referee_boundary {
    ($name:ident, [$($kind:ident($($a:expr),*)),*]) => {
        if let Some(h) = $crate::referee::native() {
            (h.boundary)(stringify!($name), &[$($crate::referee_hook!(@arg $kind($($a),*))),*]);
        }
    };
}
#[cfg(not(feature = "referee"))]
#[macro_export]
macro_rules! referee_boundary {
    ($($t:tt)*) => {};
}

/// Capture a call of the enclosing function: `referee_hook!(name, [args], ret_kind, call)` at the top of the
/// function, where `call` re-invokes the function itself (the nested entry is not captured) and `ret_kind` is one of
/// `v`, `i`, `f`, `p`. Arguments: `I(expr)`, `F(expr)`, `P(ptr, bytes)`.
#[cfg(feature = "referee")]
#[macro_export]
macro_rules! referee_hook {
    ($name:ident, [$($kind:ident($($a:expr),*)),*], $ret:ident, $call:expr) => {
        if let Some(cap) = $crate::referee::begin(stringify!($name), &[$($crate::referee_hook!(@arg $kind($($a),*))),*]) {
            #[allow(unused_unsafe)]
            let r = unsafe { $call };
            cap.finish($crate::referee_hook!(@ret $ret, r));
            return r;
        }
    };
    (@arg I($a:expr)) => { $crate::referee::Arg::I($a as u32) };
    (@arg F($a:expr)) => { $crate::referee::Arg::F($a as f32) };
    (@arg P($a:expr, $n:expr)) => { $crate::referee::Arg::P($a as *const u8, $n) };
    (@arg R($a:expr, $r:expr)) => { $crate::referee::Arg::R($a as *mut u8, $r) };
    (@ret v, $r:expr) => { { let _ = &$r; $crate::referee::Ret::Void } };
    (@ret i, $r:expr) => { $crate::referee::Ret::I($r as u32) };
    (@ret f, $r:expr) => { $crate::referee::Ret::F($r as f32) };
    (@ret p, $r:expr) => { $crate::referee::Ret::P($r as *const u8) };
}
#[cfg(not(feature = "referee"))]
#[macro_export]
macro_rules! referee_hook {
    ($($t:tt)*) => {};
}
