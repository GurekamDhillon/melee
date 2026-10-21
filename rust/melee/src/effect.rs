//! Effect banks (`efAsync_DatEntries`, ef/efasync.c) read from the user's disc, and the effect
//! models efLib creates from them (`efLib_Create`, ef/eflib.c).
//!
//! A bank file `EfXxData.dat` has one public root, the effect data table (`effCommonDataTable`
//! for bank 0): `{ ptcl cmd list, texg, EF_EffectDesc desc[] }`. `efAsync_LoadSync` registers the
//! particle bank (`psInitDataBankLoad(idx, table[0], table[1])`) and `lookup->data = &table[2]`;
//! `efLib_Create(gfx_id)` reads `desc[gfx_id % 1000]` of bank `gfx_id / 1000`
//! (`EF_EffectDesc` = `{ f32 lifetime; StaticModelDesc { joint, animjoint, matanim, shapeanim } }`,
//! 0x14 bytes, GC layout).
//!
//! Only what reaches the frame hash is modelled: the effect's lifetime and the particle tracks
//! (FObj 0x28) of its joint animation, which spawn generators through `efLib_Cb_DPtcl`. The
//! effect JObj's own transform tracks are not run (TODO(group1 jobj): the animated JObj
//! matrix); a generator on an effect joint sees the joint's bind pose under the effect root.

use melee_runtime::dat::DatArchive;
use melee_runtime::particle::PsBankData;
use std::sync::Arc;

fn be32(b: &[u8], at: u32) -> u32 {
    u32::from_be_bytes(b[at as usize..at as usize + 4].try_into().unwrap())
}

/// `EF_EffectDesc` (GC offsets: +0 lifetime, +4 joint, +8 animjoint, +C matanim, +10 shapeanim).
#[derive(Clone, Copy, Debug)]
pub struct EffectDesc {
    pub lifetime: f32,
    pub joint: u32,
    pub animjoint: u32,
}

pub struct EffectBank {
    pub archive: DatArchive,
    pub table: u32,
}

impl EffectBank {
    /// `efAsync_LoadSync(idx)`: the archive and its data table `name`.
    pub fn load(bytes: &[u8], file: &str, name: &str) -> Result<Self, String> {
        let archive = DatArchive::parse(bytes).map_err(|e| format!("{file}: {e}"))?;
        let table = archive.public_offset(name).ok_or(format!("{file}: no {name}"))?;
        Ok(Self { archive, table })
    }

    /// The bank's particle data (`psInitDataBankLoad(idx, table[0], table[1])`), when it has one.
    pub fn particles(&self) -> Result<Option<PsBankData>, String> {
        let d = &self.archive.data;
        let (c, t) = (be32(d, self.table), be32(d, self.table + 4));
        if c == 0 && t == 0 {
            return Ok(None);
        }
        let data: Arc<[u8]> = Arc::from(&d[..]);
        PsBankData::locate(&data, c, t).map(Some)
    }

    pub fn desc(&self, i: u32) -> EffectDesc {
        let d = &self.archive.data;
        let at = self.table + 8 + i * 0x14;
        EffectDesc { lifetime: f32::from_bits(be32(d, at)), joint: be32(d, at + 4), animjoint: be32(d, at + 8) }
    }
}
