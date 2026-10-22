//! Rust half of gen/cref/tu/mtu_common.h: same xorshift, same draws, same trace hash, same
//! pointer / function ids. Shared by every tests/tu_<stem>.rs of melee_modes::tu.
#![allow(dead_code, static_mut_refs)]

pub static mut TR: u32 = 0;
pub static mut TRACEON: bool = false;
pub fn t(v: u32) {
    unsafe {
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
pub fn hm(v: u32) {
    unsafe { H = H.wrapping_mul(0x0100_0193) ^ v }
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
                for (i, &x) in FP.iter().enumerate() {
                    if x == a {
                        return (i + 1) as u32;
                    }
                }
            }
            0xFFFF_FFFE
        }
    }
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
        for (i, &x) in FP.iter().enumerate() {
            if x == a {
                return 0xF000_0000 | (i + 1) as u32;
            }
        }
    }
    0xFFFF_FFFF
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
        // MTU_DBG=<op>:<case> prints every trace word of that case (compare with the C driver
        // built with -DDBG_OP=<op> -DDBG_K=<case>)
        TRACEON = std::env::var("MTU_DBG").map(|v| v == format!("{op}:{k}")).unwrap_or(false);
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
    pub fn done(mut self) -> u32 {
        assert!(self.lines.next().is_none(), "expected file has extra lines");
        self.n
    }
}
