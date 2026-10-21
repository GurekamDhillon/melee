//! Translation coverage of m-ex mod code (platform night 3 step 3).
//!
//! The execution path for mod code is the IR interpreter (`crate::ppc`: PPC32 -> `ir::Op`, run op
//! for op as pc/platform/gw_ppc.c runs it) over `crate::remap::GuestSpace` (guest field accesses
//! rewritten into native little-endian structs through the GC offset tables). This module measures,
//! per blob and per disc, how much of the inventoried code that path can take:
//!
//!   * decode: every reached instruction (analyze's reachability) decodes to an executable op,
//!     i.e. one gw_ppc.c would execute (`Op::Bad` = gw_ppc_bad_opcode there);
//!   * rewrite: every statically typed struct access (analyze's Mapped / ArrayElem / TypedNoField
//!     with a record) is checked byte for byte against the native tables (`Layouts::locate`):
//!     rewritable = every byte a native scalar/bool/byte-array byte or padding (shadowed), or a
//!     whole pointer word; partial-pointer and width-mismatch (`size_t`-like) accesses are not;
//!     a record without a native layout is counted apart;
//!   * runtime-resolved: Unknown / Opaque / Indexed accesses carry no static type. Under the
//!     interpreter they are resolved by address at run time (a registered native region is
//!     rewritten, DAT data such as `dat_attrs` stays big-endian bytes), so they are counted, not
//!     judged;
//!   * bridge: distinct out-of-blob call targets (vanilla functions the native side must provide).
//!
//! Nothing here runs mod code against a live game: there is no native Sim hook calling it yet.

use crate::analyze::{BlobReport, Class};
use crate::mexfn::{Blob, BLOB_BASE};
use crate::ppc::ir::{decode, Op};
use crate::remap::{Layouts, Loc};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

#[derive(Default, Clone, Debug)]
pub struct Coverage {
    pub reached: u64,
    pub executable: u64,
    /// Non-executable reached words by family ("bad" / "bad-spr") and primary opcode.
    pub bad: BTreeMap<String, u64>,
    pub typed: u64,
    pub rewritable: u64,
    /// Typed accesses that hit only padding (served from the shadow).
    pub padding_only: u64,
    pub partial_ptr: u64,
    pub mismatch: u64,
    pub no_native: u64,
    pub runtime: u64,
    pub stack_global: u64,
    pub bridge_targets: BTreeSet<u32>,
    /// record -> GC offset -> count, for the non-rewritable typed accesses.
    pub blockers: BTreeMap<String, BTreeMap<u32, u64>>,
}

impl Coverage {
    pub fn add(&mut self, o: &Coverage) {
        self.reached += o.reached;
        self.executable += o.executable;
        for (k, v) in &o.bad {
            *self.bad.entry(k.clone()).or_default() += v;
        }
        self.typed += o.typed;
        self.rewritable += o.rewritable;
        self.padding_only += o.padding_only;
        self.partial_ptr += o.partial_ptr;
        self.mismatch += o.mismatch;
        self.no_native += o.no_native;
        self.runtime += o.runtime;
        self.stack_global += o.stack_global;
        self.bridge_targets.extend(&o.bridge_targets);
        for (r, m) in &o.blockers {
            let e = self.blockers.entry(r.clone()).or_default();
            for (k, v) in m {
                *e.entry(*k).or_default() += v;
            }
        }
    }
    fn pct(a: u64, b: u64) -> f64 {
        if b == 0 { 0.0 } else { 100.0 * a as f64 / b as f64 }
    }
    pub fn decode_pct(&self) -> f64 {
        Self::pct(self.executable, self.reached)
    }
    pub fn rewrite_pct(&self) -> f64 {
        Self::pct(self.rewritable, self.typed)
    }
}

/// How one typed access of `size` bytes at GC offset `off` of `rec` is rewritten.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    Rewritable,
    PaddingOnly,
    PartialPtr,
    Mismatch,
    NoNative,
}

pub fn judge(l: &Layouts, rec: &str, off: u32, size: u32) -> Verdict {
    if !l.has(rec) {
        return Verdict::NoNative;
    }
    if let Loc::Ptr(_, 0, _) = l.locate(rec, off) {
        // only the whole word: the 4 bytes are the same pointer field
        return if size == 4 && matches!(l.locate(rec, off + 3), Loc::Ptr(_, 3, _)) { Verdict::Rewritable } else { Verdict::PartialPtr };
    }
    let mut all_pad = true;
    for k in 0..size {
        match l.locate(rec, off + k) {
            Loc::Byte(_) | Loc::Bool(_) => all_pad = false,
            Loc::Pad => {}
            Loc::Ptr(..) => return Verdict::PartialPtr,
            Loc::Mismatch => return Verdict::Mismatch,
        }
    }
    if all_pad { Verdict::PaddingOnly } else { Verdict::Rewritable }
}

pub fn blob_coverage(l: &Layouts, blob: &Blob, rep: &BlobReport) -> Coverage {
    let mut c = Coverage::default();
    let lo = BLOB_BASE;
    let hi = BLOB_BASE.wrapping_add(blob.code.len() as u32);
    for &off in &rep.reached {
        let o = off as usize;
        if o + 4 > blob.code.len() {
            continue;
        }
        let w = u32::from_be_bytes(blob.code[o..o + 4].try_into().unwrap());
        let ip = BLOB_BASE.wrapping_add(off);
        let op = decode(w, ip);
        c.reached += 1;
        if op.executable() {
            c.executable += 1;
        } else {
            *c.bad.entry(format!("{} op{}{}", op.family(), w >> 26, if w >> 26 == 31 || w >> 26 == 19 { format!(" xo{}", (w >> 1) & 0x3FF) } else { String::new() })).or_default() += 1;
        }
        if let Op::B { target, lk: true } | Op::Bc { target, lk: true, .. } = op {
            if !(lo..hi).contains(&target) {
                c.bridge_targets.insert(target);
            }
        } else if let Op::B { target, lk: false } = op {
            if !(lo..hi).contains(&target) {
                c.bridge_targets.insert(target);
            }
        }
    }
    for a in &rep.accesses {
        match a.class {
            Class::Stack | Class::Global => c.stack_global += 1,
            Class::Unknown | Class::Opaque | Class::Indexed => c.runtime += 1,
            Class::Mapped | Class::ArrayElem | Class::TypedNoField => {
                let Some((rec, off)) = a.record else {
                    c.runtime += 1;
                    continue;
                };
                c.typed += 1;
                let v = if off < 0 { Verdict::Mismatch } else { judge(l, rec, off as u32, a.size) };
                match v {
                    Verdict::Rewritable => c.rewritable += 1,
                    Verdict::PaddingOnly => {
                        c.rewritable += 1;
                        c.padding_only += 1
                    }
                    Verdict::PartialPtr => c.partial_ptr += 1,
                    Verdict::Mismatch => c.mismatch += 1,
                    Verdict::NoNative => c.no_native += 1,
                }
                if !matches!(v, Verdict::Rewritable | Verdict::PaddingOnly) {
                    *c.blockers.entry(format!("{rec} ({v:?})")).or_default().entry(off as u32).or_default() += 1;
                }
            }
        }
    }
    c
}

pub fn summary_text(per_blob: &[(String, Coverage)], total: &Coverage) -> String {
    let mut s = String::new();
    let t = total;
    let _ = writeln!(s, "translation (PPC -> IR interpreter + GC->native field rewrite):");
    let _ = writeln!(s, "  decode: {} of {} reached instructions executable ({:.2}%)", t.executable, t.reached, t.decode_pct());
    let mut bad: Vec<_> = t.bad.iter().collect();
    bad.sort_by(|a, b| b.1.cmp(a.1));
    for (k, v) in bad.iter().take(10) {
        let _ = writeln!(s, "    not executable: {k} x{v}");
    }
    let _ = writeln!(
        s,
        "  typed struct accesses: {} rewritable of {} ({:.2}%; {} padding-only via shadow); partial-pointer {}, width-mismatch {}, record without native layout {}",
        t.rewritable,
        t.typed,
        t.rewrite_pct(),
        t.padding_only,
        t.partial_ptr,
        t.mismatch,
        t.no_native
    );
    let _ = writeln!(s, "  untyped accesses resolved at run time by address: {}; stack/global: {}", t.runtime, t.stack_global);
    let _ = writeln!(s, "  bridge: {} distinct out-of-blob call targets", t.bridge_targets.len());
    let mut bl: Vec<(String, u64)> = t.blockers.iter().map(|(r, m)| (r.clone(), m.values().sum())).collect();
    bl.sort_by(|a, b| b.1.cmp(&a.1));
    for (r, n) in bl.iter().take(8) {
        let offs: Vec<String> = t.blockers[r].iter().take(6).map(|(o, k)| format!("{o:#x}x{k}")).collect();
        let _ = writeln!(s, "    blocker {r}: {n} ({})", offs.join(" "));
    }
    for (label, c) in per_blob {
        let _ = writeln!(s, "  {:<40} decode {:>6.2}%  rewrite {:>6.2}% of {:>5}", label, c.decode_pct(), c.rewrite_pct(), c.typed);
    }
    s
}
