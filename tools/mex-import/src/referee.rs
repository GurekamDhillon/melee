//! The DOL referee (notes/referee.md): run a retail function from main.dol in the Gekko-exact interpreter on the
//! inputs the engine's Rust port of it saw, and compare return value and every byte it wrote, bit for bit.
//!
//! Guest memory: every DOL section at its retail address, read-only (code is fetched from the text sections; r2 and
//! r13 point at the retail `_SDA2_BASE_` / `_SDA_BASE_`, so .sdata2 constants load as the game loads them); the bss
//! range as zeros, read-only too; a stack; one region per group of the call's pointer arguments, holding a big-endian
//! copy of the f32 records the capture saved. A store anywhere but the stack and the argument regions ends the run
//! (the function is then not a leaf of the checked class); a read of mutable globals (.data, .sdata, .bss, .sbss) is
//! reported, since the image holds their initial values, not the engine's.
//!
//! Callees: every callee in the text is interpreted too (the retail tree, not the Rust one); a call out of the text
//! faults. Branch coverage: every conditional branch the interpreter executes is counted by outcome.

use crate::dol::{Dol, Sym};
use crate::ppc::ir::{decode, Op};
use crate::ppc::{Bridge, Cpu, Fault, Machine, Mem, Step};
use std::collections::{BTreeMap, BTreeSet, HashMap};

const R13: u32 = 0x804D_B6A0;
const R2: u32 = 0x804D_F9E0;
const STACK_BASE: u32 = 0x8170_0000;
const STACK_SIZE: u32 = 0x10000;
const ARG_BASE: u32 = 0x8160_0000;
const ARG_STRIDE: u32 = 0x1000;
const BUDGET: u32 = 2_000_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    Text,
    /// .rodata, .sdata2, .sbss2, .ctors, .dtors, extab...: constant
    Const,
    /// .data, .sdata: initialised globals the game may change
    Data,
    /// .bss, .sbss: zero-initialised globals
    Bss,
    Stack,
    Arg,
}

struct Region {
    base: u32,
    bytes: Vec<u8>,
    kind: Kind,
}

/// The referee's guest memory.
pub struct RefMem {
    regions: Vec<Region>,
    /// mutable-global addresses read during the current run
    pub global_reads: BTreeSet<u32>,
    /// first store outside the stack and argument regions (address)
    pub bad_store: Option<u32>,
    /// the static initialisers' run: stores to .data / .bss allowed
    pub init: bool,
}

impl RefMem {
    fn find(&self, ea: u32, n: u32) -> Option<usize> {
        self.regions.iter().position(|r| ea >= r.base && ea as u64 + n as u64 <= r.base as u64 + r.bytes.len() as u64)
    }
    fn ld(&mut self, ea: u32, n: u32) -> Result<u64, Fault> {
        let i = self.find(ea, n).ok_or(Fault::Mem(ea))?;
        let r = &self.regions[i];
        if matches!(r.kind, Kind::Data | Kind::Bss) {
            self.global_reads.insert(ea);
        }
        let o = (ea - r.base) as usize;
        Ok(r.bytes[o..o + n as usize].iter().fold(0u64, |v, b| (v << 8) | *b as u64))
    }
    fn st(&mut self, ea: u32, n: u32, v: u64) -> Result<(), Fault> {
        let i = self.find(ea, n).ok_or(Fault::Mem(ea))?;
        let r = &mut self.regions[i];
        if !matches!(r.kind, Kind::Stack | Kind::Arg) && !(self.init && matches!(r.kind, Kind::Data | Kind::Bss)) {
            self.bad_store.get_or_insert(ea);
            return Err(Fault::Mem(ea));
        }
        let o = (ea - r.base) as usize;
        for k in 0..n as usize {
            r.bytes[o + k] = (v >> (8 * (n as usize - 1 - k))) as u8;
        }
        Ok(())
    }
}

impl Mem for RefMem {
    fn ld8(&mut self, ea: u32) -> Result<u8, Fault> {
        self.ld(ea, 1).map(|v| v as u8)
    }
    fn ld16(&mut self, ea: u32) -> Result<u16, Fault> {
        self.ld(ea, 2).map(|v| v as u16)
    }
    fn ld32(&mut self, ea: u32) -> Result<u32, Fault> {
        self.ld(ea, 4).map(|v| v as u32)
    }
    fn ld64(&mut self, ea: u32) -> Result<u64, Fault> {
        self.ld(ea, 8)
    }
    fn st8(&mut self, ea: u32, v: u8) -> Result<(), Fault> {
        self.st(ea, 1, v as u64)
    }
    fn st16(&mut self, ea: u32, v: u16) -> Result<(), Fault> {
        self.st(ea, 2, v as u64)
    }
    fn st32(&mut self, ea: u32, v: u32) -> Result<(), Fault> {
        self.st(ea, 4, v as u64)
    }
    fn st64(&mut self, ea: u32, v: u64) -> Result<(), Fault> {
        self.st(ea, 8, v)
    }
    fn fetch(&mut self, ea: u32) -> Result<u32, Fault> {
        let i = self.find(ea, 4).ok_or(Fault::Fetch(ea))?;
        let r = &self.regions[i];
        if r.kind != Kind::Text {
            return Err(Fault::Fetch(ea));
        }
        let o = (ea - r.base) as usize;
        Ok(u32::from_be_bytes(r.bytes[o..o + 4].try_into().unwrap()))
    }
}

struct TextOnly {
    text: Vec<(u32, u32)>,
}
impl Bridge<RefMem> for TextOnly {
    fn in_blob(&self, a: u32) -> bool {
        self.text.iter().any(|&(lo, hi)| a >= lo && a < hi)
    }
    fn call(&mut self, _c: &mut Cpu, _m: &mut RefMem, t: u32) -> Result<(), Fault> {
        Err(Fault::Unresolved(t))
    }
}

/// One captured call (melee_core::referee's line format).
#[derive(Clone, Debug)]
pub enum CArg {
    I(u32),
    F(u32),
    /// native address, length, bytes before, bytes after (native little-endian f32 words)
    P(u64, usize, Vec<u8>, Vec<u8>),
}
#[derive(Clone, Debug)]
pub enum CRet {
    V,
    I(u32),
    F(u32),
    P(u64),
}
#[derive(Clone, Debug)]
pub struct Call {
    pub name: String,
    pub frame: i32,
    pub args: Vec<CArg>,
    pub ret: CRet,
}

fn unhex(s: &str) -> Result<Vec<u8>, String> {
    (0..s.len() / 2).map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).map_err(|e| e.to_string())).collect()
}

pub fn parse_line(l: &str) -> Result<Call, String> {
    let t: Vec<&str> = l.split_whitespace().collect();
    if t.first() != Some(&"C") || t.len() < 4 {
        return Err("not a call line".into());
    }
    let name = t[1].to_string();
    let frame = t[2].parse().map_err(|_| "frame")?;
    let n: usize = t[3].parse().map_err(|_| "argc")?;
    let mut args = vec![];
    for a in &t[4..4 + n] {
        let (k, v) = a.split_once(':').ok_or("arg")?;
        args.push(match k {
            "i" => CArg::I(u32::from_str_radix(v, 16).map_err(|_| "int")?),
            "f" => CArg::F(u32::from_str_radix(v, 16).map_err(|_| "float")?),
            "p" => {
                let p: Vec<&str> = v.split(':').collect();
                if p.len() != 4 {
                    return Err("pointer arg".into());
                }
                CArg::P(u64::from_str_radix(p[0], 16).map_err(|_| "addr")?, p[1].parse().map_err(|_| "len")?, unhex(p[2])?, unhex(p[3])?)
            }
            _ => return Err(format!("arg kind {k}")),
        });
    }
    let r = t.get(5 + n).ok_or("ret")?;
    let ret = match r.split_once(':') {
        None => CRet::V,
        Some(("i", v)) => CRet::I(u32::from_str_radix(v, 16).map_err(|_| "ret")?),
        Some(("f", v)) => CRet::F(u32::from_str_radix(v, 16).map_err(|_| "ret")?),
        Some(("p", v)) => CRet::P(u64::from_str_radix(v, 16).map_err(|_| "ret")?),
        _ => return Err("ret kind".into()),
    };
    Ok(Call { name, frame, args, ret })
}

/// Branch outcome counts: (taken, not taken) per conditional branch address.
pub type Cov = BTreeMap<u32, (u64, u64)>;

#[derive(Debug, Default)]
pub struct Outcome {
    pub mismatches: Vec<String>,
    pub fault: Option<String>,
    pub global_reads: BTreeSet<u32>,
    pub insns: u64,
    /// functions entered (start addresses), the root included
    pub entered: BTreeSet<u32>,
}

pub struct Referee {
    pub dol: Dol,
    pub syms: Vec<Sym>,
    funcs: Vec<(u32, u32, usize)>, // (start, end, sym index), sorted
    by_name: HashMap<String, usize>,
    base: Vec<Region>,
    text: Vec<(u32, u32)>,
    pub cov: Cov,
    /// static initialisers run at setup: (name, ran to completion)
    pub ctors: Vec<(String, bool)>,
}

fn kind_of(name: &str) -> Kind {
    match name {
        ".text" | ".init" => Kind::Text,
        ".data" | ".sdata" => Kind::Data,
        ".bss" | ".sbss" => Kind::Bss,
        _ => Kind::Const,
    }
}

impl Referee {
    pub fn new(dol: Dol, syms: Vec<Sym>) -> Referee {
        let mut funcs: Vec<(u32, u32, usize)> = syms.iter().enumerate().filter(|(_, s)| s.function && s.size > 0).map(|(i, s)| (s.addr, s.addr + s.size, i)).collect();
        funcs.sort();
        let by_name = syms.iter().enumerate().filter(|(_, s)| s.function).map(|(i, s)| (s.name.clone(), i)).collect();
        // name each DOL data section by the symbols inside it
        let sec_name = |lo: u32, hi: u32| -> String {
            let mut n: HashMap<&str, u32> = HashMap::new();
            for s in syms.iter().filter(|s| s.addr >= lo && s.addr < hi) {
                *n.entry(&s.section).or_default() += 1;
            }
            n.into_iter().max_by_key(|e| e.1).map(|e| e.0.to_string()).unwrap_or_default()
        };
        let mut base = vec![];
        let mut text = vec![];
        for s in &dol.sections {
            let hi = s.addr + s.bytes.len() as u32;
            let kind = if s.text { Kind::Text } else { kind_of(&sec_name(s.addr, hi)) };
            if kind == Kind::Text {
                text.push((s.addr, hi));
            }
            base.push(Region { base: s.addr, bytes: s.bytes.clone(), kind });
        }
        // bss: .sbss2 is constant zeros, the rest globals; split by symbols' sections at section boundaries
        let (blo, bhi) = (dol.bss_addr, dol.bss_addr + dol.bss_size);
        let mut cuts: Vec<(u32, Kind)> = vec![];
        let mut prev: Option<Kind> = None;
        for s in syms.iter().filter(|s| s.addr >= blo && s.addr < bhi) {
            let k = match s.section.as_str() {
                ".sbss2" => Kind::Const,
                ".bss" | ".sbss" => Kind::Bss,
                _ => continue,
            };
            if prev != Some(k) {
                cuts.push((s.addr, k));
                prev = Some(k);
            }
        }
        if cuts.first().map(|c| c.0) != Some(blo) {
            cuts.insert(0, (blo, Kind::Bss));
        }
        for (i, &(a, k)) in cuts.iter().enumerate() {
            let end = cuts.get(i + 1).map(|c| c.0).unwrap_or(bhi);
            // the bss range overlaps initialised sections in some DOLs (.sdata sits inside it): keep those bytes
            let mut lo = a;
            while lo < end {
                let inside = base.iter().find(|r| lo >= r.base && lo < r.base + r.bytes.len() as u32).map(|r| r.base + r.bytes.len() as u32);
                if let Some(e) = inside {
                    lo = e;
                    continue;
                }
                let next = base.iter().filter(|r| r.base > lo && r.base < end).map(|r| r.base).min().unwrap_or(end);
                base.push(Region { base: lo, bytes: vec![0; (next - lo) as usize], kind: k });
                lo = next;
            }
        }
        let mut r = Referee { dol, syms, funcs, by_name, base, text, cov: Cov::new(), ctors: vec![] };
        r.run_ctors();
        r
    }

    /// Run the DOL's static initialisers (the `_ctors` table, as `__init_cpp` does at boot) on the base image, so
    /// globals the game fills before main (MSL's `__four_over_pi_m1` that sinf / cosf read) hold their boot values.
    fn run_ctors(&mut self) {
        let Some(t) = self.syms.iter().find(|s| s.name == "_ctors").map(|s| s.addr) else { return };
        let regions = std::mem::take(&mut self.base);
        let mut mem = RefMem { regions, global_reads: BTreeSet::new(), bad_store: None, init: true };
        mem.regions.push(Region { base: STACK_BASE, bytes: vec![0; STACK_SIZE as usize], kind: Kind::Stack });
        let mut a = t;
        while let Some(f) = self.dol.word(a).filter(|&f| f != 0) {
            let mut cpu = Cpu::default();
            cpu.gpr[1] = STACK_BASE + STACK_SIZE - 0x100;
            cpu.gpr[2] = R2;
            cpu.gpr[13] = R13;
            cpu.pc = f;
            let mut m = Machine::new_gekko(cpu);
            let mut br = TextOnly { text: self.text.clone() };
            let ok = m.run(&mut mem, &mut br, BUDGET).is_ok();
            let name = self.func_at(f).map(|s| s.name.clone()).unwrap_or(format!("{f:08X}"));
            self.ctors.push((name, ok));
            a += 4;
        }
        mem.regions.pop();
        self.base = mem.regions;
    }

    pub fn func_at(&self, a: u32) -> Option<&Sym> {
        let i = self.funcs.partition_point(|f| f.0 <= a);
        let f = self.funcs.get(i.checked_sub(1)?)?;
        (a < f.1).then(|| &self.syms[f.2])
    }
    pub fn func(&self, name: &str) -> Option<&Sym> {
        self.by_name.get(name).map(|&i| &self.syms[i])
    }

    /// The conditional branches of a function body: (address, is a branch to LR/CTR).
    pub fn cond_branches(&self, f: &Sym) -> Vec<u32> {
        let mut v = vec![];
        for a in (f.addr..f.addr + f.size).step_by(4) {
            let Some(w) = self.dol.word(a) else { continue };
            if conditional(&decode(w, a)) {
                v.push(a);
            }
        }
        v
    }

    /// Static callees (bl targets) of a function body.
    pub fn callees(&self, f: &Sym) -> Vec<u32> {
        let mut v = BTreeSet::new();
        for a in (f.addr..f.addr + f.size).step_by(4) {
            let Some(w) = self.dol.word(a) else { continue };
            if let Op::B { target, lk: true } = decode(w, a) {
                v.insert(target);
            }
        }
        v.into_iter().collect()
    }

    /// Instructions of a function the interpreter cannot run (paired singles, OS SPRs, ...), counted.
    pub fn unsupported(&self, f: &Sym) -> usize {
        (f.addr..f.addr + f.size).step_by(4).filter(|&a| self.dol.word(a).is_some_and(|w| !decode(w, a).executable())).count()
    }

    /// Run the retail function `call.name` on the call's inputs and compare with what the Rust port did.
    pub fn check(&mut self, call: &Call) -> Outcome {
        let mut out = Outcome::default();
        let Some(f) = self.func(&call.name).cloned() else {
            out.fault = Some(format!("no symbol {}", call.name));
            return out;
        };
        // group the pointer arguments by native address (aliasing and overlap stay as the engine had them)
        let mut ptrs: Vec<(u64, u64)> = call.args.iter().filter_map(|a| if let CArg::P(p, n, ..) = a { Some((*p, *p + *n as u64)) } else { None }).collect();
        ptrs.sort();
        let mut groups: Vec<(u64, u64)> = vec![];
        for (lo, hi) in ptrs {
            match groups.last_mut() {
                Some(g) if lo <= g.1 => g.1 = g.1.max(hi),
                _ => groups.push((lo, hi)),
            }
        }
        if groups.iter().any(|g| g.1 - g.0 > ARG_STRIDE as u64) {
            out.fault = Some("argument group larger than its guest region".into());
            return out;
        }
        let guest_of = |p: u64| -> Option<u32> {
            groups.iter().position(|g| p >= g.0 && p < g.1.max(g.0 + 1)).map(|i| ARG_BASE + i as u32 * ARG_STRIDE + (p - groups[i].0) as u32)
        };
        // native LE f32 words -> BE guest bytes (a word swap; every record starts 4-aligned)
        let swap = |b: &[u8]| -> Vec<u8> { b.chunks(4).flat_map(|w| w.iter().rev().copied().collect::<Vec<_>>()).collect() };
        let mut before: Vec<Vec<u8>> = groups.iter().map(|g| vec![0u8; (g.1 - g.0) as usize]).collect();
        let mut expect = before.clone();
        for a in &call.args {
            if let CArg::P(p, _, b, e) = a {
                let gi = groups.iter().position(|g| *p >= g.0 && *p < g.1).unwrap();
                let o = (*p - groups[gi].0) as usize;
                before[gi][o..o + b.len()].copy_from_slice(&swap(b));
                expect[gi][o..o + e.len()].copy_from_slice(&swap(e));
            }
        }
        // the base image is lent to this run (it is read-only outside the static initialisers) and taken back below
        let mut regions: Vec<Region> = std::mem::take(&mut self.base);
        let nbase = regions.len();
        regions.push(Region { base: STACK_BASE, bytes: vec![0; STACK_SIZE as usize], kind: Kind::Stack });
        for (i, b) in before.iter().enumerate() {
            regions.push(Region { base: ARG_BASE + i as u32 * ARG_STRIDE, bytes: b.clone(), kind: Kind::Arg });
        }
        let mut mem = RefMem { regions, global_reads: BTreeSet::new(), bad_store: None, init: false };
        let mut cpu = Cpu::default();
        let (mut gi, mut fi) = (3usize, 1usize);
        for a in &call.args {
            match a {
                CArg::I(v) => {
                    cpu.gpr[gi] = *v;
                    gi += 1;
                }
                CArg::P(p, ..) => {
                    cpu.gpr[gi] = guest_of(*p).unwrap();
                    gi += 1;
                }
                CArg::F(v) => {
                    cpu.fpr[fi] = (f32::from_bits(*v) as f64).to_bits();
                    fi += 1;
                }
            }
        }
        cpu.gpr[1] = STACK_BASE + STACK_SIZE - 0x100;
        cpu.gpr[2] = R2;
        cpu.gpr[13] = R13;
        cpu.lr = 0; // returning to 0 (outside the text) ends the run
        cpu.pc = f.addr;
        let mut m = Machine::new_gekko(cpu);
        let mut br = TextOnly { text: self.text.clone() };
        let mut n = 0u64;
        // REFEREE_TRACE=1: print every instruction of the run (debugging the referee itself)
        let trace = std::env::var_os("REFEREE_TRACE").is_some();
        let res: Result<(), Fault> = loop {
            if n >= BUDGET as u64 {
                break Err(Fault::Budget);
            }
            n += 1;
            let ip = m.cpu.pc;
            let w = match mem.fetch(ip) {
                Ok(w) => w,
                Err(e) => break Err(e),
            };
            let op = decode(w, ip);
            if let Some(s) = self.func_at(ip) {
                if s.addr == ip {
                    out.entered.insert(ip);
                }
            }
            let r = m.step(&mut mem, &mut br);
            if trace {
                let c = &m.cpu;
                eprintln!("trace {ip:08X} {op:?} r3 {:08X} f0 {:016X} f1 {:016X} f2 {:016X} f3 {:016X} f4 {:016X} cr {:08X}", c.gpr[3], c.fpr[0], c.fpr[1], c.fpr[2], c.fpr[3], c.fpr[4], c.cr);
            }
            if conditional(&op) {
                let taken = match r {
                    Ok(Step::Return) => true,
                    Ok(Step::Continue) => m.cpu.pc != ip.wrapping_add(4),
                    Err(_) => false,
                };
                if r.is_ok() {
                    let e = self.cov.entry(ip).or_default();
                    if taken {
                        e.0 += 1;
                    } else {
                        e.1 += 1;
                    }
                }
            }
            match r {
                Ok(Step::Continue) => {}
                Ok(Step::Return) => break Ok(()),
                Err(e) => break Err(e),
            }
        };
        out.insns = n;
        out.global_reads = std::mem::take(&mut mem.global_reads);
        let own = mem.regions.split_off(nbase);
        self.base = std::mem::take(&mut mem.regions);
        if let Err(e) = res {
            let at = m.cpu.pc.wrapping_sub(4);
            let wh = self.func_at(at).map(|s| format!("{}+{:#x}", s.name, at - s.addr)).unwrap_or(format!("{at:08X}"));
            out.fault = Some(match (e, mem.bad_store) {
                (_, Some(a)) => format!("store outside the arguments at {a:08X} ({wh})"),
                (Fault::BadOpcode(..), _) => format!("unsupported instruction at {wh}"),
                (Fault::Panic(_), _) => format!("unsupported SPR at {wh}"),
                (other, _) => format!("{other:?} at {wh}"),
            });
            return out;
        }
        // compare: the return value, then every byte of the argument groups
        let c = &m.cpu;
        match call.ret {
            CRet::V => {}
            CRet::I(v) => {
                if c.gpr[3] != v {
                    out.mismatches.push(format!("return r3: rust {v:08X} retail {:08X}", c.gpr[3]));
                }
            }
            CRet::F(v) => {
                let d = f64::from_bits(c.fpr[1]);
                let r = (d as f32).to_bits();
                if r != v || (d as f32) as f64 != d && !d.is_nan() {
                    out.mismatches.push(format!("return f1: rust {v:08X} ({}) retail {r:08X} ({}) f1 {:016X}", f32::from_bits(v), f32::from_bits(r), c.fpr[1]));
                }
            }
            CRet::P(p) => {
                let g = guest_of(p);
                if g != Some(c.gpr[3]) {
                    out.mismatches.push(format!("return pointer: rust {:?} retail {:08X}", g.map(|g| format!("{g:08X}")), c.gpr[3]));
                }
            }
        }
        for (gi, e) in expect.iter().enumerate() {
            let gb = ARG_BASE + gi as u32 * ARG_STRIDE;
            let got = &own.iter().find(|r| r.base == gb).unwrap().bytes;
            for w in (0..e.len()).step_by(4) {
                if got[w..w + 4] != e[w..w + 4] {
                    let (rv, gv) = (u32::from_be_bytes(e[w..w + 4].try_into().unwrap()), u32::from_be_bytes(got[w..w + 4].try_into().unwrap()));
                    // which argument (index) and offset this word is
                    let p = groups[gi].0 + w as u64;
                    let arg = call.args.iter().position(|a| matches!(a, CArg::P(q, n, ..) if p >= *q && p < *q + *n as u64)).unwrap_or(99);
                    let off = match &call.args[arg.min(call.args.len() - 1)] {
                        CArg::P(q, ..) => p - q,
                        _ => 0,
                    };
                    out.mismatches.push(format!("arg{arg}+{off:#x}: rust {rv:08X} ({}) retail {gv:08X} ({})", f32::from_bits(rv), f32::from_bits(gv)));
                }
            }
        }
        out
    }
}

/// A branch whose outcome depends on CR or CTR (BO without both "ignore" bits).
pub fn conditional(op: &Op) -> bool {
    match op {
        Op::Bc { bo, .. } | Op::Bclr { bo, .. } | Op::Bcctr { bo, .. } => bo & 0x14 != 0x14,
        _ => false,
    }
}
