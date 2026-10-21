//! Struct-offset inventory of m-ex PowerPC code.
//!
//! For each blob: split the code into functions (debug symbols; [start, END) as dump_ftfunction.py
//! proved), run a forward register-type dataflow over each function, and record every D-form
//! load/store with what its base register holds:
//!   * `Ptr(record, off)`: a pointer into a melee_types record -> mapped through
//!     `layout::Registry::resolve` to a native field,
//!   * `Stack` (r1-derived), `Sda` (r2/r13 small-data globals), `Const` (lis/addi address constant:
//!     a vanilla global 0x80xxxxxx or blob-internal data),
//!   * `Unknown`.
//!
//! Type sources (all inference; the report says which):
//!   1. Callback entry: each override target (func_table) gets r3 = HSD_GObj*, except the slots
//!      that are tables, not code (fighter slot 3 MoveLogic = a MotionState table; item slot 0
//!      `states`). That every other slot's r3 is the owner's GObj is an ASSUMPTION: true for the
//!      fighter callbacks the C build dispatches (onLoad, specials, onFrame...), not checked
//!      per slot (e.g. GetTrailData, stage slots).
//!   2. `lwz rX, 0x2C(rGObj)`: HSD_GObj.user_data -> the blob owner's record (Fighter / Ground /
//!      Item). This is the m-ex `lwz r31,0x2C(r3)` idiom; it is a guess for any GObj that is not
//!      the callback's own (e.g. a fighter blob touching an item's GObj).
//!   2b. `HSD_GObj.hsd_obj` (void*) -> HSD_JObj (fighter/item/stage GObjs; assumed).
//!   3. A load of a field whose C type is `T *` with T a known record -> Ptr(T, 0).
//!   4. `addi rD, rPtr, imm` -> Ptr(record, off + imm) (an interior pointer, e.g. &fp->self_vel).
//!   5. Spills: a typed GPR stored to the stack and reloaded keeps its type.
//!   6. Calls (bl and tail-call b) between functions of the same blob carry r3..r10 to the
//!      callee's entry (fixpoint; a caller's Unknown adds nothing, two different known types
//!      conflict).
//!   7. A function whose address the blob takes (abs32 in a MotionState/ItemState table, or a
//!      lo16/hi16 pair) is a callback: r3 = HSD_GObj* assumed, like (1).
//!   8. A `bl` to a vanilla function the decomp headers declare `T *f(...)` leaves r3 = T*.
//! A `bl` clobbers r0, r3..r12.

use crate::layout::{Mapping, Registry};
use crate::mexfn::Blob;
use std::collections::{BTreeMap, HashMap};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Val {
    Unknown,
    Stack(i32),
    Sda,
    Const(u32),
    Ptr(u32, i32),
    /// Loaded from a pointer field whose pointee has no record (`void *dat_attrs`, a
    /// character-private struct...): (record, GC offset of that field).
    Opaque(u32, i32),
}

/// Override slots that point at data tables, not code (no r3 seed).
fn table_slot(owner: crate::mexfn::Owner, slot: u32) -> bool {
    use crate::mexfn::Owner;
    matches!((owner, slot), (Owner::Fighter, 3) | (Owner::Item, 0))
}

/// Pseudo record index for "the GObj this callback was called with": only its user_data /
/// hsd_obj are typed (as the blob owner's record / HSD_JObj). Any other HSD_GObj* (a field,
/// a vanilla return) is a plain Ptr(HSD_GObj) whose user_data stays untyped.
const OWN_GOBJ: u32 = u32::MAX - 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Arg {
    None,
    Known(Val),
    Conflict,
}

#[derive(Clone, Debug, PartialEq)]
struct State {
    r: [Val; 32],
    slots: Vec<(i32, Val)>,
}

impl State {
    fn entry(args: &[Val; 8]) -> State {
        let mut r = [Val::Unknown; 32];
        r[1] = Val::Stack(0);
        r[2] = Val::Sda;
        r[13] = Val::Sda;
        r[3..11].copy_from_slice(args);
        State { r, slots: vec![] }
    }
    fn join(&mut self, o: &State) -> bool {
        let mut ch = false;
        for i in 0..32 {
            if self.r[i] != o.r[i] && self.r[i] != Val::Unknown {
                self.r[i] = Val::Unknown;
                ch = true;
            }
        }
        let before = self.slots.len();
        self.slots.retain(|s| o.slots.contains(s));
        ch || self.slots.len() != before
    }
    fn slot(&self, k: i32) -> Val {
        self.slots.iter().find(|s| s.0 == k).map(|s| s.1).unwrap_or(Val::Unknown)
    }
    fn set_slot(&mut self, k: i32, v: Val) {
        self.slots.retain(|s| s.0 != k);
        if v != Val::Unknown {
            self.slots.push((k, v));
            self.slots.sort_by_key(|s| s.0);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Class {
    /// Typed base; field found.
    Mapped,
    /// Typed base, but no field covers the byte (padding / past the record).
    TypedNoField,
    Unknown,
    Stack,
    Global,
    /// lwzx/stwx...: base + index register.
    Indexed,
    /// Typed base, offset past the record: mapped as element at/size of an array (candidate).
    ArrayElem,
    /// Through a pointer loaded from an untyped field (Val::Opaque): a struct the mod or the
    /// character defines, which no vanilla record describes.
    Opaque,
}

#[derive(Clone, Debug)]
pub struct Access {
    pub func: String,
    pub code_off: u32,
    pub insn: &'static str,
    pub store: bool,
    pub size: u32,
    pub disp: i32,
    pub class: Class,
    /// Record name and GC offset inside it, for typed bases.
    pub record: Option<(&'static str, i32)>,
    pub mapping: Option<Mapping>,
    /// Absolute address for Global accesses through a constant base.
    pub addr: Option<u32>,
}

pub struct BlobReport {
    pub label: String,
    pub functions: usize,
    pub instructions: usize,
    pub accesses: Vec<Access>,
    pub seeded_entries: usize,
    /// Regions judged to be data (tables), not code, and their words.
    #[allow(dead_code)]
    pub data_regions: usize,
    pub data_words: usize,
    /// Words of code regions no path from the region start reaches (padding, literal pools).
    pub unreached_words: usize,
}

struct Insn {
    op: u32,
    d: usize,
    a: usize,
    b: usize,
    simm: i32,
    xo: u32,
}
fn dec(w: u32) -> Insn {
    Insn {
        op: w >> 26,
        d: ((w >> 21) & 31) as usize,
        a: ((w >> 16) & 31) as usize,
        b: ((w >> 11) & 31) as usize,
        simm: (w & 0xFFFF) as u16 as i16 as i32,
        xo: (w >> 1) & 0x3FF,
    }
}

/// D-form memory op: (mnemonic, size, store, update, writes GPR rD).
fn dform(op: u32, w: u32) -> Option<(&'static str, u32, bool, bool, bool)> {
    Some(match op {
        32 => ("lwz", 4, false, false, true),
        33 => ("lwzu", 4, false, true, true),
        34 => ("lbz", 1, false, false, true),
        35 => ("lbzu", 1, false, true, true),
        36 => ("stw", 4, true, false, false),
        37 => ("stwu", 4, true, true, false),
        38 => ("stb", 1, true, false, false),
        39 => ("stbu", 1, true, true, false),
        40 => ("lhz", 2, false, false, true),
        41 => ("lhzu", 2, false, true, true),
        42 => ("lha", 2, false, false, true),
        43 => ("lhau", 2, false, true, true),
        44 => ("sth", 2, true, false, false),
        45 => ("sthu", 2, true, true, false),
        46 => ("lmw", 4, false, false, true),
        47 => ("stmw", 4, true, false, false),
        48 => ("lfs", 4, false, false, false),
        49 => ("lfsu", 4, false, true, false),
        50 => ("lfd", 8, false, false, false),
        51 => ("lfdu", 8, false, true, false),
        52 => ("stfs", 4, true, false, false),
        53 => ("stfsu", 4, true, true, false),
        54 => ("stfd", 8, true, false, false),
        55 => ("stfdu", 8, true, true, false),
        56 | 57 | 60 | 61 => {
            let single = (w >> 15) & 1 != 0; // W: one value instead of a pair
            let n = if single { 4 } else { 8 };
            match op {
                56 => ("psq_l", n, false, false, false),
                57 => ("psq_lu", n, false, true, false),
                60 => ("psq_st", n, true, false, false),
                _ => ("psq_stu", n, true, true, false),
            }
        }
        _ => return None,
    })
}

/// X-form indexed memory ops under primary opcode 31: (store, writes GPR).
fn xform_mem(xo: u32) -> Option<(bool, bool)> {
    Some(match xo {
        23 | 55 | 87 | 119 | 279 | 311 | 343 | 375 | 534 | 790 => (false, true),
        151 | 183 | 215 | 247 | 407 | 439 | 662 | 918 => (true, false),
        535 | 567 | 599 | 631 => (false, false),
        663 | 695 | 727 | 759 | 983 => (true, false),
        _ => return None,
    })
}

struct Ctx<'a> {
    reg: &'a Registry,
    blob: &'a Blob,
    gobj: Option<u32>,
    user: Option<u32>,
    jobj: Option<u32>,
    /// code offset -> reloc target, for branch relocs.
    branch_reloc: HashMap<u32, u32>,
}

impl<'a> Ctx<'a> {
    fn word(&self, off: u32) -> u32 {
        let o = off as usize;
        u32::from_be_bytes(self.blob.code[o..o + 4].try_into().unwrap())
    }

    /// The registry record behind a Ptr (OWN_GOBJ is an HSD_GObj).
    fn real(&self, rec: u32) -> u32 {
        if rec == OWN_GOBJ {
            self.gobj.unwrap_or(rec)
        } else {
            rec
        }
    }

    /// Type of a value loaded from a typed base.
    fn loaded(&self, rec: u32, off: i32) -> Val {
        if off < 0 {
            return Val::Unknown;
        }
        let own = rec == OWN_GOBJ;
        let rec = self.real(rec);
        let Some(m) = self.reg.resolve(rec as usize, off as u32, 4) else { return Val::Unknown };
        if !m.at_start {
            return Val::Unknown;
        }
        if own && m.c_path == "HSD_GObj.user_data" {
            return self.user.map(|u| Val::Ptr(u, 0)).unwrap_or(Val::Unknown);
        }
        // Fighter, item and stage GObjs all carry an HSD_JObj as hsd_obj (class 3 objects);
        // assumed here, like user_data.
        if own && m.c_path == "HSD_GObj.hsd_obj" {
            return self.jobj.map(|u| Val::Ptr(u, 0)).unwrap_or(Val::Unknown);
        }
        match m.leaf_ptr {
            Some(p) => Val::Ptr(p as u32, 0),
            None if m.leaf_type.trim_end().ends_with('*') && !m.leaf_type.contains('(') => Val::Opaque(rec, m.leaf_off as i32),
            None => Val::Unknown,
        }
    }

    fn classify(&self, base: Val, disp: i32, size: u32) -> (Class, Option<(&'static str, i32)>, Option<Mapping>, Option<u32>) {
        match base {
            Val::Stack(_) => (Class::Stack, None, None, None),
            Val::Sda => (Class::Global, None, None, None),
            Val::Const(c) => (Class::Global, None, None, Some(c.wrapping_add(disp as u32))),
            Val::Unknown => (Class::Unknown, None, None, None),
            Val::Opaque(r, o) => (Class::Opaque, Some((self.reg.name(r as usize), o)), None, None),
            Val::Ptr(r, o) => {
                let r = self.real(r) as usize;
                let at = o + disp;
                let name = self.reg.name(r);
                let rs = self.reg.size(r) as i32;
                let m = if at >= 0 { self.reg.resolve(r, at as u32, size) } else { None };
                if m.is_none() && rs > 0 && at >= rs {
                    // Past the record: a pointer to the first element of an array
                    // (`FighterBone *parts`) reads element at/size, or the type is wrong. Kept
                    // apart from Mapped: the element mapping is a candidate, not a proof.
                    if let Some(mut e) = self.reg.resolve(r, (at % rs) as u32, size) {
                        let idx = format!("{name}[{}]", at / rs);
                        e.c_path = e.c_path.replacen(name, &idx, 1);
                        e.native_path = e.native_path.replacen(name, &idx, 1);
                        return (Class::ArrayElem, Some((name, at)), Some(e), None);
                    }
                }
                match m {
                    Some(m) => (Class::Mapped, Some((name, at)), Some(m), None),
                    None => (Class::TypedNoField, Some((name, at)), None, None),
                }
            }
        }
    }
}

/// Successor code offsets of the instruction at `pc`, within [start, end).
fn succs(ctx: &Ctx, pc: u32, start: u32, end: u32) -> (Vec<u32>, Option<u32>) {
    let w = ctx.word(pc);
    let i = dec(w);
    let next = pc + 4;
    let inside = |t: u32| (start..end).contains(&t);
    let mut out = vec![];
    let mut call = None;
    let lk = w & 1 != 0;
    match i.op {
        18 => {
            let target = match ctx.branch_reloc.get(&pc) {
                Some(&t) if t & 0xF000_0000 == 0x8000_0000 => None,
                Some(&t) => Some(t),
                None => {
                    let li = ((w & 0x03FF_FFFC) as i32) << 6 >> 6;
                    if w & 2 != 0 {
                        None
                    } else {
                        Some(pc.wrapping_add(li as u32))
                    }
                }
            };
            if lk {
                call = target;
                out.push(next);
            } else if let Some(t) = target {
                if inside(t) {
                    out.push(t);
                }
            }
        }
        16 => {
            let bd = ((w & 0xFFFC) as u16 as i16) as i32;
            let t = pc.wrapping_add(bd as u32);
            let always = (w >> 21) & 0x14 == 0x14;
            if inside(t) && w & 2 == 0 {
                out.push(t);
            }
            if !always || lk {
                out.push(next);
            }
        }
        19 if i.xo == 16 || i.xo == 528 => {
            let always = (w >> 21) & 0x14 == 0x14;
            if !always || lk {
                out.push(next);
            }
        }
        _ => out.push(next),
    }
    out.retain(|&t| inside(t));
    (out, call)
}

fn step(ctx: &Ctx, pc: u32, st: &mut State, rec: &mut Option<&mut Vec<Access>>, func: &str, calls: &mut Vec<(u32, [Val; 8])>) {
    let w = ctx.word(pc);
    let i = dec(w);
    let base_of = |st: &State, a: usize| if a == 0 { Val::Const(0) } else { st.r[a] };
    if let Some((mn, size, store, update, wgpr)) = dform(i.op, w) {
        let disp = if matches!(i.op, 56 | 57 | 60 | 61) { ((w & 0xFFF) << 20) as i32 >> 20 } else { i.simm };
        let base = base_of(st, i.a);
        if let Some(v) = rec.as_deref_mut() {
            let (class, record, mapping, addr) = ctx.classify(base, disp, size);
            v.push(Access { func: func.to_string(), code_off: pc, insn: mn, store, size, disp, class, record, mapping, addr });
        }
        // value effects
        if i.op == 36 {
            if let Val::Stack(s) = base {
                st.set_slot(s + disp, st.r[i.d]);
            }
        } else if store {
            if let Val::Stack(s) = base {
                st.set_slot(s + disp, Val::Unknown);
            }
        }
        if wgpr {
            if i.op == 46 {
                for r in i.d..32 {
                    st.r[r] = Val::Unknown;
                }
            } else {
                st.r[i.d] = match (i.op, base) {
                    (32 | 33, Val::Stack(s)) => st.slot(s + disp),
                    (32 | 33, Val::Ptr(r, o)) => ctx.loaded(r, o + disp),
                    _ => Val::Unknown,
                };
            }
        }
        if update && i.a != 0 {
            st.r[i.a] = match base {
                Val::Ptr(r, o) => Val::Ptr(r, o + disp),
                Val::Stack(s) => {
                    if i.a == 1 {
                        st.slots.clear();
                    }
                    Val::Stack(s + disp)
                }
                Val::Const(c) => Val::Const(c.wrapping_add(disp as u32)),
                _ => Val::Unknown,
            };
        }
        return;
    }
    match i.op {
        14 => {
            // addi / li
            st.r[i.d] = match base_of(st, i.a) {
                Val::Const(c) => Val::Const(c.wrapping_add(i.simm as u32)),
                Val::Ptr(r, o) => Val::Ptr(r, o + i.simm),
                v @ Val::Opaque(..) => v,
                Val::Stack(s) => Val::Stack(s + i.simm),
                _ => Val::Unknown,
            }
        }
        15 => {
            st.r[i.d] = match base_of(st, i.a) {
                Val::Const(c) => Val::Const(c.wrapping_add((i.simm as u32) << 16)),
                _ => Val::Unknown,
            }
        }
        24 => {
            // ori rA, rS, uimm (rS is the d field)
            st.r[i.a] = match st.r[i.d] {
                Val::Const(c) => Val::Const(c | (w & 0xFFFF)),
                v if w & 0xFFFF == 0 => v,
                _ => Val::Unknown,
            }
        }
        31 if i.xo == 444 => st.r[i.a] = if i.d == i.b { st.r[i.d] } else { Val::Unknown }, // or / mr
        31 => {
            if let Some((store, wgpr)) = xform_mem(i.xo) {
                if let Some(v) = rec.as_deref_mut() {
                    let base = base_of(st, i.a);
                    let class = if matches!(base, Val::Stack(_)) { Class::Stack } else { Class::Indexed };
                    v.push(Access { func: func.to_string(), code_off: pc, insn: "x-form", store, size: 0, disp: 0, class, record: None, mapping: None, addr: None });
                }
                if wgpr {
                    st.r[i.d] = Val::Unknown;
                }
                if matches!(i.xo, 55 | 119 | 311 | 375 | 183 | 247 | 439 | 567 | 631 | 695 | 759) {
                    st.r[i.a] = Val::Unknown;
                }
            } else if !matches!(i.xo, 0 | 32 | 4 | 86 | 470 | 54 | 278 | 246 | 1014 | 982 | 598 | 854 | 150 | 144 | 467) {
                // conservative: any other op-31 instruction may write rD or rA
                st.r[i.d] = Val::Unknown;
                st.r[i.a] = Val::Unknown;
            } else if i.xo == 144 || i.xo == 467 {
                // mtcrf / mtspr write no GPR
            }
        }
        20 | 21 | 23 | 25 | 26 | 27 | 28 | 29 => st.r[i.a] = Val::Unknown,
        7 | 8 | 12 | 13 => st.r[i.d] = Val::Unknown,
        10 | 11 | 16 | 17 | 3 | 4 | 59 | 63 | 19 => {}
        18 => {
            if w & 1 != 0 {
                // bl: arguments to an internal callee, then the ABI's volatile clobber
                if let Some(&t) = ctx.branch_reloc.get(&pc) {
                    if t & 0xF000_0000 != 0x8000_0000 {
                        let mut a = [Val::Unknown; 8];
                        a.copy_from_slice(&st.r[3..11]);
                        calls.push((t, a));
                    }
                } else if w & 2 == 0 {
                    let li = ((w & 0x03FF_FFFC) as i32) << 6 >> 6;
                    let mut a = [Val::Unknown; 8];
                    a.copy_from_slice(&st.r[3..11]);
                    calls.push((pc.wrapping_add(li as u32), a));
                }
                st.r[0] = Val::Unknown;
                for r in 3..13 {
                    st.r[r] = Val::Unknown;
                }
                // Seed 3: a vanilla callee declared `T *f(...)` in the decomp headers.
                if let Some(&t) = ctx.branch_reloc.get(&pc) {
                    if let Some(&(rec, _)) = ctx.reg.returns.get(&t) {
                        st.r[3] = Val::Ptr(rec as u32, 0);
                    }
                }
            }
        }
        _ => st.r[i.d] = Val::Unknown,
    }
    if i.op == 19 && i.xo == 528 && w & 1 != 0 {
        st.r[0] = Val::Unknown;
        for r in 3..13 {
            st.r[r] = Val::Unknown;
        }
    }
}

/// Analyse one blob.
pub fn analyze(reg: &Registry, blob: &Blob) -> BlobReport {
    let code_len = (blob.code.len() as u32) & !3;
    let mut funcs: Vec<(u32, u32, String)> = blob
        .symbols
        .iter()
        .filter(|(s, e, _)| s < e && *s < code_len)
        .map(|(s, e, n)| (*s & !3, (*e).min(code_len), n.clone()))
        .collect();
    funcs.sort();
    funcs.dedup_by(|b, a| a.0 == b.0);
    // gaps (or no symbols at all) become anonymous functions so every word is analysed once
    let mut all = vec![];
    let mut cur = 0;
    for (s, e, n) in funcs {
        if s < cur {
            continue;
        }
        if s > cur {
            all.push((cur, s, format!("gap_{cur:x}")));
        }
        all.push((s, e, n));
        cur = e;
    }
    if cur < code_len {
        all.push((cur, code_len, format!("gap_{cur:x}")));
    }
    let ctx = Ctx {
        reg,
        blob,
        gobj: reg.find("HSD_GObj").map(|x| x as u32),
        user: reg.find(blob.owner.user_data_record()).map(|x| x as u32),
        jobj: reg.find("HSD_JObj").map(|x| x as u32),
        branch_reloc: blob.relocs.iter().filter(|r| r.flag == 0x0A).map(|r| (r.code_off, r.target)).collect(),
    };
    let unknown = [Val::Unknown; 8];
    let mut entry: BTreeMap<u32, [Val; 8]> = BTreeMap::new();
    let mut inferred: BTreeMap<u32, [Arg; 8]> = BTreeMap::new();
    let mut seeded = 0;
    for &(slot, target) in &blob.overrides {
        let is_slot = slot & 0x8000_0000 == 0;
        let gobj_arg = !table_slot(blob.owner, slot);
        let mut a = unknown;
        if is_slot && gobj_arg {
            if ctx.gobj.is_some() {
                a[0] = Val::Ptr(OWN_GOBJ, 0);
            }
        }
        if !(is_slot && gobj_arg) {
            continue;
        }
        seeded += 1;
        let e = entry.entry(target & !3).or_insert(a);
        if *e != a {
            for (x, y) in e.iter_mut().zip(a) {
                if *x != y {
                    *x = Val::Unknown;
                }
            }
        }
    }
    // A region is DATA (MotionState/ItemState tables, constants) when an abs32 pointer reloc
    // lands on its first word, or when it has no return / indirect / unconditional-branch word
    // at all. m-ex keeps those tables in the code blob and gives them debug symbols too.
    let abs32: std::collections::HashSet<u32> = blob.relocs.iter().filter(|r| r.flag == 0x01).map(|r| r.code_off).collect();
    let is_code = |s: u32, e: u32| {
        if abs32.contains(&s) {
            return false;
        }
        (s..e).step_by(4).any(|pc| {
            let w = ctx.word(pc);
            w == 0x4E80_0020 || w == 0x4E80_0420 || (w >> 26 == 18 && w & 1 == 0)
        })
    };
    let (mut data_regions, mut data_words) = (0, 0);
    all.retain(|(s, e, _)| {
        let c = is_code(*s, *e);
        if !c {
            data_regions += 1;
            data_words += ((e - s) / 4) as usize;
        }
        c
    });
    let mut unreached_words = 0;
    let starts: std::collections::HashSet<u32> = all.iter().map(|f| f.0).collect();
    // Seed 2: a function whose address the blob takes (an abs32 word in a MotionState /
    // ItemState table, or a lis/addi lo16+hi16 pair passed to a vanilla setter) is a callback;
    // r3 = HSD_GObj* is assumed for those too, unless an override entry already said otherwise.
    if ctx.gobj.is_some() {
        let mut a = unknown;
        a[0] = Val::Ptr(OWN_GOBJ, 0);
        for r in &blob.relocs {
            if !matches!(r.flag, 0x01 | 0x04) || r.target & 0xF000_0000 == 0x8000_0000 || !starts.contains(&r.target) {
                continue;
            }
            if let std::collections::btree_map::Entry::Vacant(v) = entry.entry(r.target) {
                v.insert(a);
                seeded += 1;
            }
        }
    }
    let mut accesses = vec![];
    let mut instructions = 0;
    for pass in 0..16 {
        let last = pass == 15;
        let mut calls_all: Vec<(u32, [Val; 8])> = vec![];
        let mut acc_pass = vec![];
        let mut n_insn = 0;
        let mut unreached_pass = 0;
        for (s, e, name) in &all {
            let mut args = entry.get(s).copied().unwrap_or(unknown);
            if let Some(inf) = inferred.get(s) {
                for (x, y) in args.iter_mut().zip(inf) {
                    if let (Val::Unknown, Arg::Known(v)) = (*x, y) {
                        *x = *v;
                    }
                }
            }
            let mut states: HashMap<u32, State> = HashMap::new();
            states.insert(*s, State::entry(&args));
            let mut work = vec![*s];
            let mut calls = vec![];
            while let Some(pc) = work.pop() {
                let mut st = states[&pc].clone();
                step(&ctx, pc, &mut st, &mut None, name, &mut calls);
                for t in succs(&ctx, pc, *s, *e).0 {
                    match states.get_mut(&t) {
                        None => {
                            states.insert(t, st.clone());
                            work.push(t);
                        }
                        Some(old) => {
                            if old.join(&st) {
                                work.push(t);
                            }
                        }
                    }
                }
            }
            // record with the converged states, reached instructions only
            let mut pcs: Vec<u32> = states.keys().copied().collect();
            pcs.sort();
            let mut v = vec![];
            let mut dummy = vec![];
            let reached = pcs.len();
            for pc in pcs {
                let mut st = states[&pc].clone();
                step(&ctx, pc, &mut st, &mut Some(&mut v), name, &mut dummy);
                // a tail call (`b` out of this function) passes the arguments on, too
                let w = ctx.word(pc);
                if w >> 26 == 18 && w & 3 == 0 && !ctx.branch_reloc.get(&pc).is_some_and(|t| t & 0xF000_0000 == 0x8000_0000) {
                    let li = ((w & 0x03FF_FFFC) as i32) << 6 >> 6;
                    let t = ctx.branch_reloc.get(&pc).copied().unwrap_or(pc.wrapping_add(li as u32));
                    if !(*s..*e).contains(&t) {
                        let mut a = [Val::Unknown; 8];
                        a.copy_from_slice(&states[&pc].r[3..11]);
                        dummy.push((t, a));
                    }
                }
                n_insn += 1;
            }
            unreached_pass += ((e - s) / 4) as usize - reached;
            acc_pass.extend(v);
            let _ = calls;
            calls_all.extend(dummy);
        }
        // Optimistic join over call sites: a caller whose argument is Unknown adds nothing
        // (it may well pass the same object, unproven); two callers passing different known
        // types make that argument a sticky Conflict.
        let mut changed = false;
        for (t, a) in calls_all {
            if !starts.contains(&t) {
                continue;
            }
            let e = inferred.entry(t).or_insert([Arg::None; 8]);
            for (x, y) in e.iter_mut().zip(a) {
                if matches!(y, Val::Unknown | Val::Stack(_) | Val::Sda) {
                    continue;
                }
                let nx = match *x {
                    Arg::None => Arg::Known(y),
                    Arg::Known(v) if v == y => Arg::Known(v),
                    Arg::Known(_) | Arg::Conflict => Arg::Conflict,
                };
                if nx != *x {
                    *x = nx;
                    changed = true;
                }
            }
        }
        accesses = acc_pass;
        instructions = n_insn;
        unreached_words = unreached_pass;
        if !changed || last {
            break;
        }
    }
    BlobReport {
        label: blob.label.clone(),
        functions: all.len(),
        instructions,
        accesses,
        seeded_entries: seeded,
        data_regions,
        data_words,
        unreached_words,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mexfn::{Blob, Owner};

    fn blob(words: &[u32], overrides: Vec<(u32, u32)>) -> Blob {
        Blob {
            label: "t".into(),
            owner: Owner::Fighter,
            code: words.iter().flat_map(|w| w.to_be_bytes()).collect(),
            relocs: vec![],
            overrides,
            symbols: vec![(0, words.len() as u32 * 4, "f".into())],
        }
    }
    const fn d(op: u32, rd: u32, ra: u32, imm: i16) -> u32 {
        (op << 26) | (rd << 21) | (ra << 16) | (imm as u16 as u32)
    }
    fn mr(dst: u32, src: u32) -> u32 {
        (31 << 26) | (src << 21) | (dst << 16) | (src << 11) | (444 << 1)
    }

    #[test]
    fn gobj_to_fighter_fields() {
        let reg = Registry::new();
        let code = [
            d(37, 1, 1, -0x20),   // stwu r1,-0x20(r1)
            d(36, 31, 1, 0x1C),   // stw r31,0x1C(r1)
            mr(31, 3),            // mr r31,r3
            d(32, 4, 31, 0x2C),   // lwz r4,0x2C(r31)   gobj->user_data -> Fighter*
            d(48, 1, 4, 0x84),    // lfs f1,0x84(r4)    fp->self_vel.y
            d(14, 5, 4, 0x80),    // addi r5,r4,0x80    &fp->self_vel
            d(52, 1, 5, 0x8),     // stfs f1,8(r5)      self_vel.z
            d(36, 4, 1, 0x10),    // stw r4,0x10(r1)    spill
            0x4800_0001,          // bl +0 (vanilla call, clobbers r3-r12)
            d(32, 6, 1, 0x10),    // lwz r6,0x10(r1)    reload -> Fighter*
            d(32, 7, 6, 0x10C),   // lwz r7,0x10C(r6)   fp->ft_data (ftData*)
            d(40, 8, 7, 0),       // lhz r8,0(r7)       ftData field
            d(32, 9, 3, 0),       // lwz r9,0(r3)       r3 clobbered -> unknown
            d(34, 10, 13, -0x5000), // lbz r10,-0x5000(r13) SDA global
            0x4E80_0020,          // blr
        ];
        let b = blob(&code, vec![(4, 0)]);
        let r = analyze(&reg, &b);
        let get = |off: u32| r.accesses.iter().find(|a| a.code_off == off).unwrap();
        assert_eq!(get(4).class, Class::Stack);
        assert_eq!(get(12).mapping.as_ref().unwrap().c_path, "HSD_GObj.user_data");
        assert_eq!(get(16).mapping.as_ref().unwrap().c_path, "Fighter.self_vel.y");
        assert_eq!(get(24).mapping.as_ref().unwrap().c_path, "Fighter.self_vel.z");
        assert_eq!(get(40).mapping.as_ref().unwrap().c_path, "Fighter.ft_data");
        assert!(get(44).mapping.as_ref().unwrap().c_path.starts_with("ftData."));
        assert_eq!(get(48).class, Class::Unknown);
        assert_eq!(get(52).class, Class::Global);
    }

    #[test]
    fn interprocedural_args() {
        let reg = Registry::new();
        // f0: lwz r3,0x2C(r3); bl f1; blr      f1: lfs f1,0xB0(r3); blr
        let code = [d(32, 3, 3, 0x2C), 0x4800_0009, 0x4E80_0020, d(48, 1, 3, 0xB0), 0x4E80_0020];
        let mut b = blob(&code, vec![(23, 0)]);
        b.symbols = vec![(0, 12, "f0".into()), (12, 20, "f1".into())];
        let r = analyze(&reg, &b);
        let a = r.accesses.iter().find(|a| a.code_off == 12).unwrap();
        assert_eq!(a.mapping.as_ref().unwrap().c_path, "Fighter.cur_pos.x");
    }
}
