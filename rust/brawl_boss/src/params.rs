//! The parameter block the param ids index (format spec 5.1). Ids and their offsets follow the spec's table; the
//! values are read from the file, never compiled in.

use crate::error::{Error, Result};
use crate::file::BossFile;
use crate::fx::Fx;
use crate::script::Target;
use alloc::vec::Vec;

/// (param id, offset in the block, length in bytes).
pub const LAYOUT: [(u32, u32, u32); 20] = [
    (44000, 0x000, 0x100),
    (44001, 0x100, 0x24),
    (44002, 0x124, 0x20),
    (24000, 0x144, 4),
    (24001, 0x148, 4),
    (24002, 0x14C, 4),
    (4000, 0x150, 4),
    (44003, 0x154, 0x10),
    (44004, 0x164, 0x10),
    (44005, 0x174, 8),
    (44006, 0x17C, 0xC),
    (44007, 0x188, 0xC),
    (44008, 0x194, 0x18),
    (44009, 0x1AC, 0x24),
    (44010, 0x1D0, 0x18),
    (44011, 0x1E8, 0x18),
    (44012, 0x200, 0x7C),
    (44013, 0x27C, 0x7C),
    (44014, 0x2F8, 0x1C),
    (44015, 0x314, 0x1C),
];
/// Bytes the block spans.
pub const BLOCK_LEN: u32 = 0x330;
/// The weighted lists (12-byte entries {int base, float slope, ptr condition}).
pub const WEIGHTED: [u32; 4] = [44008, 44009, 44010, 44011];
/// Movement kinds in 44000, in order.
pub const KINETIC_KINDS: [i32; 4] = [14, 15, 16, 17];
/// Kinetic sentinel "keep current" (35791.0), and the two related ones.
pub const KEEP: Fx = Fx::from_int(35791);
pub const SENTINEL_35789: Fx = Fx::from_int(35789);
pub const SENTINEL_35790: Fx = Fx::from_int(35790);

/// A weighted-list entry: weight = base + slope * L, optionally filtered by a condition script.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WeightEntry {
    pub base: i32,
    pub slope: Fx,
    pub condition: Target,
}

/// A cage proc parameter record (44014 / 44015).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CageProc {
    pub a: Fx,
    pub b: Fx,
    /// Phase thresholds in integer percent.
    pub phase1_pct: i32,
    pub phase2_pct: i32,
    pub effect: i32,
    pub effect_bone: i32,
    pub effect_ref: Target,
}

#[derive(Clone, Debug, Default)]
pub struct Params {
    pub at: u32,
    words: Vec<u32>,
    lists: Vec<(u32, Vec<WeightEntry>)>,
    cage_effect_ref: [Target; 2],
    spawn_ptrs: [[Target; 31]; 2],
}

fn find(id: u32) -> Option<(u32, u32)> {
    LAYOUT.iter().find(|x| x.0 == id).map(|x| (x.1, x.2))
}

impl Params {
    pub(crate) fn decode(bf: &BossFile, at: u32) -> Result<Params> {
        let mut words = Vec::with_capacity(BLOCK_LEN as usize / 4);
        for i in 0..BLOCK_LEN / 4 {
            words.push(bf.word(at + 4 * i)?);
        }
        let mut lists = Vec::new();
        for id in WEIGHTED {
            let (off, len) = find(id).ok_or(Error::BadHeader("layout"))?;
            let mut v = Vec::new();
            for e in 0..len / 12 {
                let p = at + off + 12 * e;
                v.push(WeightEntry {
                    base: bf.word(p)? as i32,
                    slope: Fx::from_f32_bits(bf.word(p + 4)?),
                    condition: bf.ptr_field(p + 8)?,
                });
            }
            lists.push((id, v));
        }
        let cage_effect_ref = [bf.ptr_field(at + 0x2F8 + 24)?, bf.ptr_field(at + 0x314 + 24)?];
        let mut spawn_ptrs = [[Target::None; 31]; 2];
        for (s, base) in [0x200u32, 0x27C].into_iter().enumerate() {
            for w in 0..31 {
                spawn_ptrs[s][w as usize] = bf.ptr_field(at + base + 4 * w)?;
            }
        }
        Ok(Params { at, words, lists, cage_effect_ref, spawn_ptrs })
    }

    fn w(&self, off: u32) -> Option<u32> {
        self.words.get(off as usize / 4).copied()
    }

    /// The record's words, raw.
    pub fn raw(&self, id: u32) -> Option<&[u32]> {
        let (off, len) = find(id)?;
        self.words.get(off as usize / 4..(off + len) as usize / 4)
    }
    /// The record's words read as floats.
    pub fn floats(&self, id: u32) -> Option<Vec<Fx>> {
        Some(self.raw(id)?.iter().map(|&b| Fx::from_f32_bits(b)).collect())
    }
    /// An int parameter (24000-24002).
    pub fn int(&self, id: u32) -> Option<i32> {
        match id {
            24000..=24002 => self.w(find(id)?.0).map(|v| v as i32),
            _ => None,
        }
    }
    /// A single-value parameter as read by a variable (memory 0): ints and the 4000 float.
    pub fn scalar(&self, id: u32) -> Option<Fx> {
        match id {
            24000..=24002 => self.int(id).map(|i| Fx::from_int(i as i64)),
            4000 => self.w(find(id)?.0).map(Fx::from_f32_bits),
            _ => None,
        }
    }
    /// A range record (44003, 44004): lo = a + b L, hi = c + d L.
    pub fn range(&self, id: u32) -> Option<[Fx; 4]> {
        let f = self.floats(id)?;
        Some([*f.first()?, *f.get(1)?, *f.get(2)?, *f.get(3)?])
    }
    /// A weighted list with `n` entries (the switch's count; at most the list's size).
    pub fn weighted(&self, id: u32, n: i32) -> Result<Vec<WeightEntry>> {
        let l = self.lists.iter().find(|x| x.0 == id).ok_or(Error::BadOffset { what: "weighted list", at: id })?;
        if n < 0 || n as usize > l.1.len() {
            return Err(Error::BadOffset { what: "weighted count", at: id });
        }
        Ok(l.1[..n as usize].to_vec())
    }
    pub fn weighted_lists(&self) -> impl Iterator<Item = &Vec<WeightEntry>> {
        self.lists.iter().map(|x| &x.1)
    }
    /// A movement kind's 16 words, raw (14, 15, 16, 17).
    pub fn kinetic_raw(&self, kind: i32) -> Option<&[u32]> {
        let i = KINETIC_KINDS.iter().position(|&k| k == kind)?;
        self.words.get(16 * i..16 * i + 16)
    }
    /// The same as fixed-point (int words such as the -1 at word 13 read as floats are reported as 0 / NaN = 0;
    /// use `kinetic_raw` for those).
    pub fn kinetic(&self, kind: i32) -> Option<[Fx; 16]> {
        let r = self.kinetic_raw(kind)?;
        let mut o = [Fx::ZERO; 16];
        for i in 0..16 {
            o[i] = Fx::from_f32_bits(r[i]);
        }
        Some(o)
    }
    /// The cage spawn record (side 0 = left / 44012, 1 = right / 44013), raw words.
    pub fn cage_spawn(&self, side: usize) -> Option<&[u32]> {
        self.raw(44012 + side.min(1) as u32)
    }
    /// The pointer words of a cage spawn record, resolved.
    pub fn cage_spawn_ptrs(&self, side: usize) -> &[Target; 31] {
        &self.spawn_ptrs[side.min(1)]
    }
    /// Cage HP (spawn record word 0).
    pub fn cage_hp(&self, side: usize) -> Option<i32> {
        self.cage_spawn(side)?.first().map(|&v| v as i32)
    }
    pub fn cage_proc(&self, side: usize) -> Option<CageProc> {
        let r = self.raw(44014 + side.min(1) as u32)?;
        Some(CageProc {
            a: Fx::from_f32_bits(r[0]),
            b: Fx::from_f32_bits(r[1]),
            phase1_pct: r[2] as i32,
            phase2_pct: r[3] as i32,
            effect: r[4] as i32,
            effect_bone: r[5] as i32,
            effect_ref: self.cage_effect_ref[side.min(1)],
        })
    }
}
