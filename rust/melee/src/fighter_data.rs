//! A fighter's files on the user's disc: `PlXx.dat` (`ftDataXx`: `struct ftData`, ft/types.h) and
//! the animation bank `PlXxAJ.dat`. Read locally, never written anywhere.
//!
//! Only what the native match reads so far: the common attributes block (`ftData->x0`,
//! `ftCo_DatAttrs`, GC layout from melee_types) and a motion's animation length
//! (`ftData->xC[anim_id]` = `Fighter_WaitAnimData { char* name; s32 aj_offset; s32 aj_size; ... }`,
//! 0x18 bytes; the AJ slice is a DAT whose root is the `HSD_FigaTree`, `frames` at +8;
//! `ftData_80085CD8` loads it and `HSD_AObj.end_frame` is that `frames`).

use melee_runtime::dat::DatArchive;
use melee_types::ftCo_DatAttrs;

// Declared here (not in main.rs) so this round touches only the integration files it owns.
#[path = "skeleton.rs"]
pub mod skeleton;

fn be32(b: &[u8], at: u32) -> u32 {
    u32::from_be_bytes(b[at as usize..at as usize + 4].try_into().unwrap())
}

#[derive(Clone, Copy, Debug)]
pub struct AnimRow {
    pub frames: f32,
    pub flags: u32,
}

impl AnimRow {
    /// `x594_b1_loop`.
    pub fn looping(&self) -> bool {
        self.flags & 0x4000_0000 != 0
    }
}

pub struct FighterFiles {
    pub dat: DatArchive,
    pub ft_data: u32,
    pub aj: Vec<u8>,
}

/// (`PlXx.dat`, `ftDataXx`, `PlXxAJ.dat`) per kind: ftcaptain.c / ftfox.c `ft*_Init_DatFilename`,
/// `_DataName`, `_AnimDatFilename`.
pub fn files_for(name: &str) -> Option<(&'static str, &'static str, &'static str)> {
    match name {
        "falcon" => Some(("PlCa.dat", "ftDataCaptain", "PlCaAJ.dat")),
        "fox" => Some(("PlFx.dat", "ftDataFox", "PlFxAJ.dat")),
        _ => None,
    }
}

/// Costume 0's model file and joint symbol (ftcaptain.c `nr_dat` / `nr_joint`, ftfox.c
/// `ftFx_Init_803C7C00` / `803C7C0C`).
pub fn costume_for(name: &str) -> Option<(&'static str, &'static str)> {
    match name {
        "falcon" => Some(("PlCaNr.dat", "PlyCaptain5K_Share_joint")),
        "fox" => Some(("PlFxNr.dat", "PlyFox5K_Share_joint")),
        _ => None,
    }
}

/// `ftData->x44` (`ftData_x44_t`): the six ECB part indices, `unkC` and the ledge snap box.
#[derive(Clone, Copy, Debug)]
pub struct EcbDesc {
    pub parts: [i16; 6],
    pub unk_c: f32,
    pub ledge_snap_x: f32,
    pub ledge_snap_y: f32,
    pub ledge_snap_height: f32,
}

impl FighterFiles {
    pub fn load(disc: &mut crate::disc::Disc, name: &str) -> Result<Self, String> {
        let (file, data_name, aj_file) = files_for(name).ok_or(format!("fighter {name:?}: files not wired"))?;
        let dat = DatArchive::parse(&disc.read(file)?).map_err(|e| format!("{file}: {e}"))?;
        let ft_data = dat.public_offset(data_name).ok_or(format!("{file}: no {data_name}"))?;
        let aj = disc.read(aj_file)?;
        Ok(Self { dat, ft_data, aj })
    }

    /// `ftData->x0->trophy_scale` (`ftCo_DatAttrs` +0x110; copied to `fp->co_attrs`).
    pub fn trophy_scale(&self) -> f32 {
        use melee_core::gc::GcLayout;
        let attrs = be32(&self.dat.data, self.ft_data);
        let off = ftCo_DatAttrs::gc_field("trophy_scale").map(|f| f.offset).unwrap_or(0x110);
        f32::from_bits(be32(&self.dat.data, attrs + off))
    }

    /// `ftData->x0->model_scaling` (`ftCo_DatAttrs` +0x8C).
    pub fn model_scaling(&self) -> f32 {
        use melee_core::gc::GcLayout;
        let attrs = be32(&self.dat.data, self.ft_data);
        let off = ftCo_DatAttrs::gc_field("model_scaling").map(|f| f.offset).unwrap_or(0x8C);
        f32::from_bits(be32(&self.dat.data, attrs + off))
    }

    /// `ftData->x0` decoded (`Fighter_UnkInitLoad` copies it to `fp->co_attrs`; vanilla's
    /// FT_SCRIPT_VALUE is the identity).
    pub fn attrs(&self) -> ftCo_DatAttrs {
        *self.dat.load_at::<ftCo_DatAttrs>(be32(&self.dat.data, self.ft_data)).get()
    }

    /// Animation row `anim_id`: its figatree length (`HSD_AObj.end_frame`) and the row's
    /// `x10_animCurrFlags` (`fp->x594`; MSB-first bits: b1 = 0x40000000 loop). None: no figatree.
    pub fn anim_row(&self, anim_id: u32) -> Result<Option<AnimRow>, String> {
        let d = &self.dat.data;
        let row = be32(d, self.ft_data + 0xC) + anim_id * 0x18;
        let (off, size, flags) = (be32(d, row + 4) as usize, be32(d, row + 8) as usize, be32(d, row + 0x10));
        if size == 0 {
            return Ok(None);
        }
        let slice = self.aj.get(off..off + size).ok_or("AJ row out of range")?;
        let a = DatArchive::parse(slice).map_err(|e| format!("AJ row {anim_id}: {e}"))?;
        let root = a.publics.first().map(|p| p.offset).ok_or(format!("AJ row {anim_id}: no root"))?;
        Ok(Some(AnimRow { frames: f32::from_bits(be32(&a.data, root + 8)), flags }))
    }

    /// Animation row `anim_id`'s figatree (`ftData_80085CD8` -> `fp->x590`).
    pub fn figatree(&self, anim_id: u32) -> Result<Option<skeleton::FigaTree>, String> {
        let d = &self.dat.data;
        let row = be32(d, self.ft_data + 0xC) + anim_id * 0x18;
        let (off, size) = (be32(d, row + 4) as usize, be32(d, row + 8) as usize);
        if size == 0 {
            return Ok(None);
        }
        let slice = self.aj.get(off..off + size).ok_or("AJ row out of range")?;
        let a = DatArchive::parse(slice).map_err(|e| format!("AJ row {anim_id}: {e}"))?;
        let root = a.publics.first().map(|p| p.offset).ok_or(format!("AJ row {anim_id}: no root"))?;
        skeleton::FigaTree::parse(&a.data, root).map(Some)
    }

    /// `ftData->x8->x10` (the part ftCommon_8007F6A4 rescales in ftAnim_8006FA58).
    pub fn x8_x10(&self) -> usize {
        let x8 = be32(&self.dat.data, self.ft_data + 8);
        self.dat.data[x8 as usize + 0x10] as usize
    }

    /// `ftData->x44`.
    pub fn ecb_desc(&self) -> EcbDesc {
        let d = &self.dat.data;
        let at = be32(d, self.ft_data + 0x44);
        let h = |o: u32| u16::from_be_bytes(d[(at + o) as usize..][..2].try_into().unwrap()) as i16;
        let f = |o: u32| f32::from_bits(be32(d, at + o));
        EcbDesc {
            parts: [h(0), h(2), h(4), h(6), h(8), h(0xA)],
            unk_c: f(0xC),
            ledge_snap_x: f(0x10),
            ledge_snap_y: f(0x14),
            ledge_snap_height: f(0x18),
        }
    }
}
