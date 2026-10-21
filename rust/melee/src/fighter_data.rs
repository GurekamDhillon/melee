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

fn be32(b: &[u8], at: u32) -> u32 {
    u32::from_be_bytes(b[at as usize..at as usize + 4].try_into().unwrap())
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

    /// The figatree length of animation row `anim_id`, or None when the row has no animation.
    pub fn anim_frames(&self, anim_id: u32) -> Result<Option<f32>, String> {
        let d = &self.dat.data;
        let row = be32(d, self.ft_data + 0xC) + anim_id * 0x18;
        let (off, size) = (be32(d, row + 4) as usize, be32(d, row + 8) as usize);
        if size == 0 {
            return Ok(None);
        }
        let slice = self.aj.get(off..off + size).ok_or("AJ row out of range")?;
        let a = DatArchive::parse(slice).map_err(|e| format!("AJ row {anim_id}: {e}"))?;
        let root = a.publics.first().map(|p| p.offset).ok_or(format!("AJ row {anim_id}: no root"))?;
        Ok(Some(f32::from_bits(be32(&a.data, root + 8))))
    }
}
