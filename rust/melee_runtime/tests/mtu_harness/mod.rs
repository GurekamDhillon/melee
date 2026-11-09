//! Rust half of gen/mtu/tu/mtu_common.h: same xorshift, same draws, same trace hash, same
//! pointer / function ids. Shared by every tests/tu_<stem>.rs of melee_world::tu.
#![allow(dead_code, static_mut_refs)]
#[path = "../../../melee_core/test_support/fnsym.rs"]
mod fnsym;

pub static mut TR: u32 = 0;
pub static mut TRACEON: bool = false;
pub static mut NCALLS: u32 = 0;

// ---- cases that end in a failed assertion / panic / runaway loop ("<op> <k> ABORT" in the C output) ----
// The ports are `extern "C"` functions, so a panic cannot unwind out of them (it would abort the process).
// Each case therefore runs on its own thread; the panic hook (or the runaway guard in `t`) reports the case as
// aborted and ends the thread on the spot, and the next case starts on a fresh thread with freshly drawn state.
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
static IN_CASE: AtomicBool = AtomicBool::new(false);
static ABORT_TX: Mutex<Option<std::sync::mpsc::Sender<bool>>> = Mutex::new(None);
pub fn abort_case() -> ! {
    if let Some(tx) = ABORT_TX.lock().unwrap_or_else(|e| e.into_inner()).as_ref() {
        let _ = tx.send(true);
    }
    #[cfg(windows)]
    {
        // End the thread without unwinding (the ports' frames cannot unwind); parked threads would keep the
        // process from exiting.
        extern "system" {
            fn ExitThread(code: u32) -> !;
        }
        unsafe { ExitThread(0) }
    }
    #[cfg(not(windows))]
    loop {
        std::thread::park();
    }
}
/// Installs the hook that turns a panic inside a case into an aborted case (other panics print as usual).
pub fn install_abort_hook() {
    let default = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        if IN_CASE.load(Ordering::SeqCst) {
            if std::env::var_os("MTU_SHOWPANIC").is_some() {
                eprintln!("case panic: {info}");
            }
            abort_case();
        }
        default(info)
    }));
}
/// Runs one case; true when it was aborted (the C driver printed `ABORT` for the same case).
pub fn run_case<F: FnOnce()>(f: F) -> bool {
    let (tx, rx) = std::sync::mpsc::channel::<bool>();
    *ABORT_TX.lock().unwrap_or_else(|e| e.into_inner()) = Some(tx.clone());
    let f: Box<dyn FnOnce() + '_> = Box::new(f);
    // SAFETY: the caller thread blocks on `rx` until the case finished or was aborted; an aborted case's
    // thread is parked forever and never touches the borrowed state again.
    let f: Box<dyn FnOnce() + Send + 'static> = unsafe { core::mem::transmute(f) };
    IN_CASE.store(true, Ordering::SeqCst);
    std::thread::Builder::new()
        .stack_size(16 << 20)
        .spawn(move || {
            f();
            let _ = tx.send(false);
        })
        .unwrap();
    let aborted = rx.recv().unwrap();
    IN_CASE.store(false, Ordering::SeqCst);
    aborted
}
pub fn t(v: u32) {
    unsafe {
        NCALLS += 1;
        if NCALLS > 20000 {
            abort_case();
        }
        TR = TR.wrapping_mul(31).wrapping_add(v);
        if TRACEON {
            println!("T {:08X}", v);
        }
    }
}
pub fn t64(d: f64) {
    let u = d.to_bits();
    t(u as u32);
    t((u >> 32) as u32);
}
pub static mut H: u32 = 0;
pub static mut HDBG: bool = false;
pub fn hm(v: u32) {
    unsafe {
        H = H.wrapping_mul(0x0100_0193) ^ v;
        if HDBG {
            println!("H {:08X}", v);
        }
    }
}
pub fn hm64(d: f64) {
    let u = d.to_bits();
    hm(u as u32);
    hm((u >> 32) as u32);
}
pub static mut RS: u32 = 0;
pub fn rnd() -> u32 {
    unsafe {
        RS ^= RS << 13;
        RS ^= RS >> 17;
        RS ^= RS << 5;
        RS
    }
}
pub fn rg() -> f32 {
    match rnd() % 16 {
        0 => 0.0,
        1 => -0.0,
        2 => f32::from_bits(0x7FC0_0000),
        3 => 1.0,
        4 => -1.0,
        _ => ((rnd() % 41) as i32 - 20) as f32 / 20.0,
    }
}
pub fn rw() -> f32 {
    const K: [f32; 4] = [1.0, 13.0, 1000.0, 100000.0];
    let a = rg();
    a * K[(rnd() % 4) as usize]
}
pub fn fv() -> u32 {
    let r = rnd();
    match r & 15 {
        0 | 1 => 0,
        2 | 3 => 1,
        4 => 2,
        5 => 3,
        6 => (r >> 8) & 7,
        7 => (r >> 8) & 0xF,
        8 => (r >> 8) & 0xFF,
        9 => (r >> 8) & 0x7F,
        10 => 0xFF,
        11 => 0x21,
        _ => r >> 5,
    }
}
pub fn fv64() -> u64 {
    let hi = fv() as u64;
    let lo = fv() as u64;
    (hi << 32) | lo
}
pub static mut SRV: [u32; 16] = [0; 16];
pub fn chk(id: u32) -> bool {
    unsafe { (SRV[(id & 15) as usize] >> (id & 7)) & 1 != 0 }
}

static mut RB: Vec<(usize, usize)> = Vec::new();
pub fn preg<T>(p: *const T, n: usize) {
    unsafe { RB.push((p as usize, n)) }
}
static mut FP: Vec<usize> = Vec::new();
pub fn freg(f: usize) {
    unsafe { FP.push(f) }
}
pub fn fi<F: Copy>(f: Option<F>) -> u32 {
    match f {
        None => 0,
        Some(f) => {
            assert_eq!(core::mem::size_of::<F>(), core::mem::size_of::<usize>());
            let a: usize = unsafe { core::mem::transmute_copy(&f) };
            unsafe {
                // by address, else by symbol (a lib-stored pointer can be a crate-local copy: fnsym.rs)
                if let Some(i) = fnsym::find(&*core::ptr::addr_of!(FP), a) {
                    return (i + 1) as u32;
                }
            }
            0xFFFF_FFFE
        }
    }
}
/// A function address truncated to 32 bits (`(s32) fnptr`): named by the registered function it is.
pub fn fi32(v: u32) -> u32 {
    if v == 0 {
        return 0;
    }
    unsafe {
        for (i, &x) in FP.iter().enumerate() {
            if x as u32 == v {
                return (i + 1) as u32;
            }
        }
    }
    0xFFFF_FFFD
}
/// A data address truncated to 32 bits (`(u32) ptr`): named by the registered region it falls in.
pub fn pi32(v: u32) -> u32 {
    if v == 0 {
        return 0;
    }
    unsafe {
        for (i, &(b, z)) in RB.iter().enumerate() {
            let b = b as u32;
            if v >= b && ((v - b) as usize) < z {
                return (((i + 1) as u32) << 16) | (v - b);
            }
        }
    }
    0xFFFF_FFFC
}
/// A data pointer that is really a function address (UNK_T slots) gets the function's id.
pub fn pi<T>(p: *const T) -> u32 {
    let a = p as usize;
    if a == 0 {
        return 0;
    }
    unsafe {
        for (i, &(b, z)) in RB.iter().enumerate() {
            if a >= b && a < b + z {
                return (((i + 1) as u32) << 16) | (a - b) as u32;
            }
        }
        // by address, else by symbol (a lib-stored pointer can be a crate-local copy: fnsym.rs)
        if let Some(i) = fnsym::find(&*core::ptr::addr_of!(FP), a) {
            return 0xF000_0000 | (i + 1) as u32;
        }
    }
    0xFFFF_FFFF
}
/// A pointer slot inside a union that other views overwrite byte-wise: only an exact region base counts.
pub fn pib<T>(p: *const T) -> u32 {
    let v = pi(p);
    if v == 0 || v == 0xFFFF_FFFF || (v & 0xFFFF) == 0 {
        v
    } else {
        0xFFFF_FFFE
    }
}
pub fn sh(s: *const u8) -> u32 {
    if s.is_null() {
        return 0;
    }
    let mut h: u32 = 0x811C_9DC5;
    let mut p = s;
    unsafe {
        while *p != 0 {
            h = (h ^ *p as u32).wrapping_mul(0x0100_0193);
            p = p.add(1);
        }
    }
    h
}
pub fn rh<T>(v: &T) -> u32 {
    let mut h: u32 = 0x811C_9DC5;
    let b = unsafe { core::slice::from_raw_parts(v as *const T as *const u8, core::mem::size_of::<T>()) };
    for &x in b {
        h = (h ^ x as u32).wrapping_mul(0x0100_0193);
    }
    h
}
/// Address of a record as bytes (the anonymous-member walkers of the generated fill / hash).
pub fn base<T>(r: &T) -> *const u8 {
    r as *const T as *const u8
}
pub fn case(op: u32, k: u32) {
    unsafe {
        TR = 0;
        NCALLS = 0;
        if std::env::var_os("MTU_PROG").is_some() {
            eprintln!("P {op} {k}");
        }
        // MTU_DBG=<op>:<case> prints every trace word of that case (compare with the C driver
        // built with -DDBG_OP=<op> -DDBG_K=<case>)
        TRACEON = std::env::var("MTU_DBG").map(|v| v == format!("{op}:{k}")).unwrap_or(false);
        HDBG = TRACEON;
        H = 0x811C_9DC5;
        RS = 0x9059u32.wrapping_add(op.wrapping_mul(7919)).wrapping_add(k.wrapping_mul(104729));
        for i in 0..16 {
            SRV[i] = rnd();
        }
    }
}
/// Compares the per-case lines with tests/data/mtu_<stem>.txt.
pub struct Check {
    lines: std::str::Lines<'static>,
    pub n: u32,
}
impl Check {
    pub fn new(s: &'static str) -> Self {
        Check { lines: s.lines(), n: 0 }
    }
    pub fn line(&mut self, op: u32, k: u32, ret: u32) {
        let got = format!("{} {} {:08X} {:08X} {:08X}", op, k, ret, unsafe { H }, unsafe { TR });
        let want = self.lines.next().expect("expected file too short");
        assert_eq!(got, want, "op {op} case {k}");
        self.n += 1;
    }
    /// A case the C driver ended with a failed assertion ("<op> <k> ABORT"): the port must have panicked there.
    pub fn abort(&mut self, op: u32, k: u32) {
        let got = format!("{} {} ABORT", op, k);
        let want = self.lines.next().expect("expected file too short");
        assert_eq!(got, want, "op {op} case {k}: the port panicked where the C did not (or vice versa)");
        self.n += 1;
    }
    pub fn done(mut self) -> u32 {
        assert!(self.lines.next().is_none(), "expected file has extra lines");
        self.n
    }
}

