//! Rust twin of rust/melee_modes/oracle/s497/rt.c: reads the same scenario text, builds the same
//! synthetic state (a PRNG-filled arena plus the globals), calls the Rust port and prints the
//! same trace. The C harness's output (tests/data/s497_<tu>_exp.txt) must come out byte for byte.
#![allow(dead_code, clippy::missing_safety_doc)]

use melee_modes::hook::A;
use std::cell::RefCell;
use std::fmt::Write as _;

pub const ARENA_SIZE: usize = 512 * 1024;
const MAXRET: usize = 1024;

/// What a callee returns, for the default (unscripted) return value.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Rk {
    Void,
    /// integer of this many bits
    I(u32),
    F,
    D,
    P,
}

/// What a function under test returned.
#[derive(Clone, Copy, Debug)]
pub enum Ret {
    Void,
    Int(u64),
    Ptr(u64),
}

pub struct Glue {
    pub ftab: Vec<(&'static str, usize)>,
    /// name, address, size, filled from the seed at the start of a scenario
    pub gtab: Vec<(&'static str, usize, usize, bool)>,
    /// None: unknown function; Some(Ret::Void) ...
    pub dispatch: unsafe fn(&str, &[u64]) -> Option<Ret>,
    /// false: unknown type
    pub dump: unsafe fn(&str, *const u8, usize) -> bool,
    /// logical bitfield fill: type, base, count, seed
    pub bffill: unsafe fn(&str, *mut u8, usize, u64) -> bool,
    pub retkind: fn(&str) -> Rk,
    /// stub behaviours that write through out parameters (the C stubs' bodies)
    pub custom: unsafe fn(&str, &[A]) -> Option<u64>,
}

struct ScriptedRet {
    name: String,
    v: Vec<u64>,
    next: usize,
}

pub struct Rt {
    arena: Vec<u8>,
    pub rng: u64,
    ncalls: u64,
    out: String,
    rets: Vec<ScriptedRet>,
    glue: Option<&'static Glue>,
}

thread_local! {
    static RT: RefCell<Rt> = RefCell::new(Rt { arena: vec![], rng: 1, ncalls: 0, out: String::new(), rets: vec![], glue: None });
}

pub fn with<R>(f: impl FnOnce(&mut Rt) -> R) -> R {
    RT.with(|r| f(&mut r.borrow_mut()))
}

fn rng_next(s: &mut u64) -> u64 {
    let mut x = *s;
    x ^= x >> 12;
    x ^= x << 25;
    x ^= x >> 27;
    *s = x;
    x.wrapping_mul(0x2545F4914F6CDD1D)
}

pub fn fnv(p: &[u8]) -> u64 {
    let mut h = 0xCBF29CE484222325u64;
    for &b in p {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001B3);
    }
    h
}

fn fill(p: &mut [u8], seed: u64) {
    let mut st = seed.wrapping_mul(0x9E3779B97F4A7C15).wrapping_add(1);
    if st == 0 {
        st = 1;
    }
    let n = p.len();
    let mut i = 0;
    while i + 8 <= n {
        p[i..i + 8].copy_from_slice(&rng_next(&mut st).to_le_bytes());
        i += 8;
    }
    if i < n {
        let v = rng_next(&mut st).to_le_bytes();
        p[i..].copy_from_slice(&v[..n - i]);
    }
}

fn hash_name(s: &str) -> u64 {
    fnv(s.as_bytes())
}

impl Rt {
    pub fn arena_base(&self) -> usize {
        self.arena.as_ptr() as usize
    }
    fn norm(&self, v: u64) -> String {
        let g = self.glue.unwrap();
        let base = self.arena_base() as u64;
        if v == 0 {
            return "N".into();
        }
        if v >= base && v < base + ARENA_SIZE as u64 {
            return format!("P:{:x}", v - base);
        }
        for (n, p) in &g.ftab {
            if *p as u64 == v {
                return format!("F:{n}");
            }
        }
        for (n, p, sz, _) in &g.gtab {
            if v >= *p as u64 && v < (*p + *sz) as u64 {
                return format!("G:{n}+{:x}", v - *p as u64);
            }
        }
        format!("X:{v:x}")
    }
    fn next_scripted(&mut self, name: &str) -> Option<u64> {
        for r in &mut self.rets {
            if r.name == name {
                if r.v.is_empty() {
                    return None;
                }
                let v = r.v[r.next];
                r.next = (r.next + 1) % r.v.len();
                return Some(v);
            }
        }
        None
    }
    pub fn scripted_ptr(&mut self, name: &str) -> u64 {
        self.next_scripted(name).unwrap_or(0)
    }
    pub fn ret_int(&mut self, name: &str, bits: u32) -> u64 {
        self.ret_i(name, bits)
    }
    pub fn rng(&mut self) -> u64 {
        rng_next(&mut self.rng)
    }
    fn ret_i(&mut self, name: &str, bits: u32) -> u64 {
        if let Some(v) = self.next_scripted(name) {
            return v;
        }
        let v = self.rng();
        if bits <= 8 {
            v & 1
        } else {
            (v >> 33) % 5
        }
    }
    fn ret_f(&mut self, name: &str) -> u64 {
        if let Some(v) = self.next_scripted(name) {
            return v & 0xFFFF_FFFF;
        }
        let f = ((self.rng() >> 33) % 2000) as i32 as f32 / 16.0f32 - 40.0f32;
        f.to_bits() as u64
    }
    fn ret_d(&mut self, name: &str) -> u64 {
        if let Some(v) = self.next_scripted(name) {
            return v;
        }
        let f = ((self.rng() >> 33) % 2000) as i32 as f64 / 16.0 - 40.0;
        f.to_bits()
    }
    pub fn fill_out(&mut self, p: *mut u8, n: usize) {
        for i in 0..n {
            unsafe { *p.add(i) = (self.rng() >> 24) as u8 };
        }
    }
    fn put(&mut self, s: &str) {
        self.out.push_str(s);
    }
}

fn arg_str(rt: &Rt, a: &A) -> String {
    match *a {
        A::I(v) => format!(" I:{v:x}"),
        A::F(f) => format!(" F:{:08x}", f.to_bits()),
        A::D(d) => format!(" D:{:016x}", d.to_bits()),
        A::P(p) => format!(" {}", rt.norm(p as u64)),
        A::M(p, n) => format!(" M:{:x}", fnv(unsafe { core::slice::from_raw_parts(p as *const u8, n) })),
        A::O(_) => " O".to_string(),
        A::S(p) => {
            let v = p as u64;
            let base = rt.arena_base() as u64;
            if v == 0 {
                return " N".into();
            }
            if v >= base && v < base + ARENA_SIZE as u64 {
                return format!(" P:{:x}", v - base);
            }
            let g = rt.glue.unwrap();
            for (n, gp, sz, _) in &g.gtab {
                if v >= *gp as u64 && v < (*gp + *sz) as u64 {
                    return format!(" G:{n}+{:x}", v - *gp as u64);
                }
            }
            let mut s = String::from(" S:\"");
            let mut i = 0;
            unsafe {
                while i < 80 && *p.add(i) != 0 {
                    let c = *p.add(i) as u8;
                    if (0x20..0x7f).contains(&c) && c != b'"' && c != b'\\' {
                        s.push(c as char);
                    } else {
                        let _ = write!(s, "\\x{c:02x}");
                    }
                    i += 1;
                }
            }
            s.push('"');
            s
        }
    }
}

/// The hook every ext module calls: trace the call, then return the scripted/default value.
pub unsafe fn hook(name: &'static str, args: &[A]) -> u64 {
    let glue = with(|rt| rt.glue.unwrap());
    // custom behaviours trace themselves (they know which arguments to hash)
    with(|rt| {
        rt.ncalls += 1;
        let mut line = format!("L {name}");
        for a in args {
            line.push_str(&arg_str(rt, a));
        }
        rt.put(&line);
    });
    let c = (glue.custom)(name, args);
    with(|rt| {
        rt.put("\n");
        let rk = (glue.retkind)(name);
        match c {
            Some(v) => v,
            None => match rk {
                Rk::Void => 0,
                Rk::I(b) => rt.ret_i(name, b),
                Rk::F => rt.ret_f(name),
                Rk::D => rt.ret_d(name),
                Rk::P => rt.next_scripted(name).unwrap_or(0),
            },
        }
    })
}

/// Helpers for custom behaviours (the Rust twins of the C stub bodies).
pub fn tr_m(p: *const u8, n: usize) {
    with(|rt| {
        let h = fnv(unsafe { core::slice::from_raw_parts(p, n) });
        let _ = write!(rt.out, " M:{h:x}");
    })
}
pub fn fill_out(p: *mut u8, n: usize) {
    with(|rt| rt.fill_out(p, n));
}

// dump callbacks
pub fn d_i(path: &str, v: u64) {
    with(|rt| {
        let _ = writeln!(rt.out, "  {path}={v:x}");
    })
}
pub fn d_f(path: &str, bits: u32) {
    with(|rt| {
        let _ = writeln!(rt.out, "  {path}=f{bits:08x}");
    })
}
pub fn d_d(path: &str, bits: u64) {
    with(|rt| {
        let _ = writeln!(rt.out, "  {path}=d{bits:016x}");
    })
}
pub fn d_p(path: &str, v: u64) {
    with(|rt| {
        let s = rt.norm(v);
        let _ = writeln!(rt.out, "  {path}={s}");
    })
}
pub fn d_blob(path: &str, p: *const u8, n: usize) {
    with(|rt| {
        let h = fnv(unsafe { core::slice::from_raw_parts(p, n) });
        let _ = writeln!(rt.out, "  {path}=#{n}:{h:x}");
    })
}

fn hex(s: &str) -> u64 {
    u64::from_str_radix(s, 16).unwrap_or_else(|_| panic!("bad hex {s}"))
}

fn store(p: usize, n: usize, v: u64) {
    unsafe { core::ptr::copy_nonoverlapping(v.to_le_bytes().as_ptr(), p as *mut u8, n) }
}

fn tok_val(rt: &Rt, g: &Glue, t: &str) -> u64 {
    if let Some(r) = t.strip_prefix("s:") {
        return hex(r);
    }
    if let Some(r) = t.strip_prefix("a:") {
        return rt.arena_base() as u64 + hex(r);
    }
    if t == "n" {
        return 0;
    }
    if let Some(r) = t.strip_prefix("f:") {
        return ffind(g, r) as u64;
    }
    if let Some(r) = t.strip_prefix("g:") {
        let (nm, off) = match r.split_once('+') {
            Some((a, b)) => (a, hex(b)),
            None => (r, 0),
        };
        return gfind(g, nm).1 as u64 + off;
    }
    panic!("bad token {t}");
}

fn gfind<'a>(g: &'a Glue, name: &str) -> &'a (&'static str, usize, usize, bool) {
    g.gtab.iter().find(|e| e.0 == name).unwrap_or_else(|| panic!("no global {name}"))
}
fn ffind(g: &Glue, name: &str) -> usize {
    g.ftab.iter().find(|e| e.0 == name).unwrap_or_else(|| panic!("no function {name}")).1
}

/// Run a scenario file against the port. `glue` lives for the whole process.
pub fn run(scenario: &str, glue: &'static Glue) -> String {
    with(|rt| {
        rt.arena = vec![0u8; ARENA_SIZE + 16];
        // keep the base 16-aligned like malloc does for the C side
        let off = (16 - (rt.arena.as_ptr() as usize & 15)) & 15;
        rt.arena.drain(..off);
        rt.arena.truncate(ARENA_SIZE);
        rt.glue = Some(glue);
        rt.out.clear();
    });
    let mut cur: Option<(String, u64, Vec<String>)> = None;
    for line in scenario.lines() {
        if let Some(r) = line.strip_prefix("begin ") {
            let mut it = r.split_whitespace();
            cur = Some((it.next().unwrap().to_string(), hex(it.next().unwrap()), vec![]));
        } else if line.starts_with("end") && cur.is_some() {
            let (fname, seed, lines) = cur.take().unwrap();
            run_block(glue, &lines, &fname, seed);
        } else if let Some(c) = cur.as_mut() {
            c.2.push(line.to_string());
        }
    }
    with(|rt| core::mem::take(&mut rt.out))
}

fn run_block(glue: &'static Glue, lines: &[String], fname: &str, seed: u64) {
    with(|rt| {
        rt.rets.clear();
        rt.ncalls = 0;
        rt.arena.iter_mut().for_each(|b| *b = 0);
        fill(&mut rt.arena, seed);
        for (n, p, sz, f) in &glue.gtab {
            if *f {
                let s = unsafe { core::slice::from_raw_parts_mut(*p as *mut u8, *sz) };
                fill(s, seed ^ hash_name(n));
            }
        }
        rt.rng = seed ^ 0xD1B54A32D192ED03;
        if rt.rng == 0 {
            rt.rng = 1;
        }
        let _ = writeln!(rt.out, "== {fname} {seed:016x}");
    });
    for l in lines {
        let tk: Vec<&str> = l.split_whitespace().collect();
        if tk.is_empty() {
            continue;
        }
        let base = with(|rt| rt.arena_base());
        let aptr = |off: u64| -> u64 { if off == 0xFFFF { 0 } else { base as u64 + off } };
        match tk[0] {
            "W" => store(base + hex(tk[1]) as usize, hex(tk[2]) as usize, hex(tk[3])),
            "P" => store(base + hex(tk[1]) as usize, 8, aptr(hex(tk[2]))),
            "PF" => store(base + hex(tk[1]) as usize, 8, ffind(glue, tk[2]) as u64),
            "GW" => store(gfind(glue, tk[1]).1 + hex(tk[2]) as usize, hex(tk[3]) as usize, hex(tk[4])),
            "GP" => store(gfind(glue, tk[1]).1 + hex(tk[2]) as usize, 8, aptr(hex(tk[3]))),
            "GPF" => store(gfind(glue, tk[1]).1 + hex(tk[2]) as usize, 8, ffind(glue, tk[3]) as u64),
            "GSET" => store(gfind(glue, tk[1]).1, 8, aptr(hex(tk[2]))),
            "RET" => with(|rt| {
                let v: Vec<u64> = tk[2..].iter().take(MAXRET).map(|t| tok_val(rt, glue, t)).collect();
                rt.rets.push(ScriptedRet { name: tk[1].to_string(), v, next: 0 });
            }),
            "call" | "callf" => {
                let first = if tk[0] == "callf" { 2 } else { 1 };
                let target = if first == 2 { tk[1] } else { fname };
                let a: Vec<u64> = with(|rt| tk[first..].iter().map(|t| tok_val(rt, glue, t)).collect());
                if first == 2 {
                    with(|rt| {
                        let _ = writeln!(rt.out, "C {target}");
                    });
                }
                let r = unsafe { (glue.dispatch)(target, &a) };
                with(|rt| match r {
                    None => panic!("dispatch: unknown function {target}"),
                    Some(Ret::Void) => rt.put("ret -\n"),
                    Some(Ret::Int(v)) => {
                        let _ = writeln!(rt.out, "ret I:{v:x}");
                    }
                    Some(Ret::Ptr(v)) => {
                        let s = rt.norm(v);
                        let _ = writeln!(rt.out, "ret {s}");
                    }
                });
            }
            "dump" => {
                let cnt = hex(tk[3]) as usize;
                let b = with(|rt| {
                    let _ = writeln!(rt.out, "D {} {} {}", tk[1], tk[2], tk[3]);
                    tok_val(rt, glue, tk[1])
                });
                if !unsafe { (glue.dump)(tk[2], b as *const u8, cnt) } {
                    panic!("no dumper {}", tk[2]);
                }
            }
            "bf" => {
                let b = with(|rt| tok_val(rt, glue, tk[1]));
                if !unsafe { (glue.bffill)(tk[2], b as *mut u8, hex(tk[3]) as usize, hex(tk[4])) } {
                    panic!("no bffill {}", tk[2]);
                }
            }
            "PW" => {
                let o = hex(tk[1]) as usize;
                let st = hex(tk[2]) as usize;
                let cnt = hex(tk[3]) as usize;
                for j in 0..cnt {
                    store(base + o + j * st, 8, aptr(hex(tk[4])));
                }
            }
            "hashn" => with(|rt| {
                let o = hex(tk[1]) as usize;
                let len = hex(tk[2]) as usize;
                let b = rt.arena_base() as u64;
                let mut h = 0xCBF29CE484222325u64;
                let mut j = 0;
                while j + 8 <= len {
                    let mut wv = u64::from_le_bytes(rt.arena[o + j..o + j + 8].try_into().unwrap());
                    if wv >= b && wv < b + ARENA_SIZE as u64 {
                        wv = 0x5EED000000000000 | (wv - b);
                    } else if let Some(fi) = glue.ftab.iter().position(|e| e.1 as u64 == wv) {
                        wv = 0xF00D000000000000 | fi as u64;
                    }
                    for q in 0..8 {
                        h ^= (wv >> (8 * q)) & 0xFF;
                        h = h.wrapping_mul(0x100000001B3);
                    }
                    j += 8;
                }
                let _ = writeln!(rt.out, "N {} {} {:x}", tk[1], tk[2], h);
            }),
            "hash" => with(|rt| {
                let o = hex(tk[1]) as usize;
                let n = hex(tk[2]) as usize;
                let h = fnv(&rt.arena[o..o + n]);
                let _ = writeln!(rt.out, "H {} {} {:x}", tk[1], tk[2], h);
            }),
            "pdump" => with(|rt| {
                let g = gfind(glue, tk[1]);
                let v = unsafe { core::ptr::read_unaligned(g.1 as *const u64) };
                let s = rt.norm(v);
                let _ = writeln!(rt.out, "P {} {}", tk[1], s);
            }),
            "padump" => with(|rt| {
                let g = gfind(glue, tk[1]);
                for i in 0..g.2 / 8 {
                    let v = unsafe { core::ptr::read_unaligned((g.1 + 8 * i) as *const u64) };
                    let s = rt.norm(v);
                    let _ = writeln!(rt.out, "P {}[{}] {}", tk[1], i, s);
                }
            }),
            "gdump" => with(|rt| {
                let g = gfind(glue, tk[1]);
                let h = fnv(unsafe { core::slice::from_raw_parts(g.1 as *const u8, g.2) });
                let _ = writeln!(rt.out, "G {} {:x}", tk[1], h);
            }),
            o => panic!("bad op {o}"),
        }
    }
}

/// First differing line of two traces, for assertion messages.
pub fn first_diff(a: &str, b: &str) -> String {
    let (mut la, mut lb) = (a.lines(), b.lines());
    let mut n = 0;
    let mut ctx = String::new();
    loop {
        n += 1;
        match (la.next(), lb.next()) {
            (None, None) => return "identical".into(),
            (x, y) if x == y => {
                if let Some(x) = x {
                    ctx = x.to_string();
                }
            }
            (x, y) => return format!("line {n}: rust {:?} vs c {:?} (after {:?})", x, y, ctx),
        }
    }
}
