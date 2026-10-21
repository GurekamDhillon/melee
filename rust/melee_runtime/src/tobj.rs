//! Port of the logic of `src/sysdolphin/baselib/tobj.c`: the GX id <-> index maps
//! (`HSD_TGTex2Index` .. `HSD_TexMap2Index`, `HSD_TexMapID2PTTexMtx`), `DifferentTluts`,
//! `MakeTextureMtx` and `TObjUpdateFunc`.
//!
//! Native shape: `HsdTObj<I>` keeps the fields these functions touch; `I` is the image-desc
//! handle (`imagedesc` / `imagetbl` hold handles, not pointers). `lod` and `tev` are owned
//! records (`None` = NULL; the C dereferences them unconditionally, the port panics). The TLUT
//! table is only tested for presence by `TObjUpdateFunc`, so it is a flag. GcLayout is delegated
//! to `melee_types::HSD_TObj` / `_HSD_TObjTev` / `_HSD_Tlut` / `_HSD_TexLODDesc`.
//!
//! Assertions / `HSD_Panic` on an id outside the switch are Rust panics (the C does not return).
//! `HSD_TexCoordID2TexGenSrc(GX_TEXCOORD7)` asserts in the C (the case falls into `default`) and
//! panics here too.
//!
//! Float rule: no fused site in these functions (helper_tobj.ll: 0 fmuladd outside the callees).
//! Conversions as gwtool lowers them: `(u8)(255.0 * fv)` is `fptoui` saturated to 32 bits then
//! truncated; `(int)` / `(s32)` saturate signed ([`crate::cabi::f2i32`]). `tlut_no` is the
//! TARGET_PC `s32` (see tobj.h).
//!
//! Not ported: the anim entry points (Add/Remove/Req/Anim[All]), load/alloc/class functions,
//! the GX setup and TEV-expression half.
use crate::cabi::f2i32;
use crate::hsd_mtx::mk_rotation_mtx;
use crate::hsd_util::GxColor;
use crate::mtx::{self, Mtx, Quaternion, Vec3};

macro_rules! delegate_layout {
    ($t:ty, $g:ty) => {
        impl<I> melee_core::gc::GcLayout for $t {
            const GC_NAME: &'static str = <$g as melee_core::gc::GcLayout>::GC_NAME;
            const GC_SIZE: u32 = <$g as melee_core::gc::GcLayout>::GC_SIZE;
            const GC_ALIGN: u32 = <$g as melee_core::gc::GcLayout>::GC_ALIGN;
            const GC_FIELDS: &'static [melee_core::gc::GcField] = <$g as melee_core::gc::GcLayout>::GC_FIELDS;
        }
    };
}
macro_rules! delegate_layout0 {
    ($t:ty, $g:ty) => {
        impl melee_core::gc::GcLayout for $t {
            const GC_NAME: &'static str = <$g as melee_core::gc::GcLayout>::GC_NAME;
            const GC_SIZE: u32 = <$g as melee_core::gc::GcLayout>::GC_SIZE;
            const GC_ALIGN: u32 = <$g as melee_core::gc::GcLayout>::GC_ALIGN;
            const GC_FIELDS: &'static [melee_core::gc::GcField] = <$g as melee_core::gc::GcLayout>::GC_FIELDS;
        }
    };
}

pub const GX_MIRROR: u32 = 2;
pub const GX_TG_TEX0: u32 = 4;
pub const GX_TG_TEXCOORD0: u32 = 12;
pub const GX_TEXMTX0: u32 = 30;
pub const GX_IDENTITY: u32 = 60;
pub const GX_PTTEXMTX0: u32 = 64;
/// `TEX_MTX_DIRTY`.
pub const TEX_MTX_DIRTY: u32 = 1 << 31;
/// `FLT_EPSILON` as tobj.c defines it.
pub const TOBJ_FLT_EPSILON: f32 = 1.000_000_013_35e-10;

/// `HSD_A_T_*` track types.
pub mod track {
    pub const TIMG: u32 = 1;
    pub const TRAU: u32 = 2;
    pub const TRAV: u32 = 3;
    pub const SCAU: u32 = 4;
    pub const SCAV: u32 = 5;
    pub const ROTX: u32 = 6;
    pub const ROTY: u32 = 7;
    pub const ROTZ: u32 = 8;
    pub const BLEND: u32 = 9;
    pub const TCLT: u32 = 10;
    pub const LOD_BIAS: u32 = 11;
    pub const KONST_R: u32 = 12;
    pub const TEV1_A: u32 = 23;
    pub const TS_BLEND: u32 = 24;
}

/// Native `HSD_TObjTev`'s three colours (the rest of the record is GX setup state).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HsdTObjTev {
    pub konst: GxColor,
    pub tev0: GxColor,
    pub tev1: GxColor,
}
delegate_layout0!(HsdTObjTev, melee_types::_HSD_TObjTev);

/// Native `HSD_Tlut` (GameCube record 0x10).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HsdTlut {
    pub lut: u32,
    pub fmt: u32,
    pub tlut_name: u32,
    pub n_entries: u16,
}
delegate_layout0!(HsdTlut, melee_types::_HSD_Tlut);

/// Native `HSD_TexLODDesc` (GameCube record 0x10).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HsdTexLodDesc {
    pub min_filt: u32,
    pub lod_bias: f32,
    pub bias_clamp: u8,
    pub edge_lod_enable: u8,
    pub max_anisotropy: u32,
}
delegate_layout0!(HsdTexLodDesc, melee_types::_HSD_TexLODDesc);

/// Native `HSD_TObj` (GameCube record 0xAC), the fields the ported logic touches.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HsdTObj<I> {
    pub rotate: Quaternion,
    pub scale: Vec3,
    pub translate: Vec3,
    pub wrap_s: u32,
    pub wrap_t: u32,
    pub repeat_s: u8,
    pub repeat_t: u8,
    pub flags: u32,
    pub blending: f32,
    pub imagedesc: Option<I>,
    pub lod: Option<HsdTexLodDesc>,
    pub imagetbl: Option<Vec<Option<I>>>,
    pub has_tluttbl: bool,
    pub tlut_no: i32,
    pub mtx: Mtx,
    pub tev: Option<HsdTObjTev>,
}
delegate_layout!(HsdTObj<I>, melee_types::HSD_TObj);

/// `HSD_TexMapID2PTTexMtx` (static).
pub fn tex_map_id_to_pt_tex_mtx(id: u32) -> u32 {
    match id {
        0..=7 => GX_PTTEXMTX0 + 3 * id,
        _ => panic!("tobj.c:574 unexpected texmap id."),
    }
}

/// `HSD_TGTex2Index`.
pub fn tg_tex_to_index(tgtex: u32) -> u32 {
    match tgtex {
        4..=11 => tgtex - GX_TG_TEX0,
        _ => panic!("tobj.c:1701 assert"),
    }
}

/// `HSD_TexCoordID2TexGenSrc` (GX_TEXCOORD7 asserts, as in the C).
pub fn tex_coord_id_to_tex_gen_src(coord: u32) -> u32 {
    match coord {
        0..=6 => GX_TG_TEXCOORD0 + coord,
        _ => panic!("tobj.c:1738 assert"),
    }
}

/// `HSD_TexCoord2Index`.
pub fn tex_coord_to_index(coord: u32) -> u32 {
    match coord {
        0..=7 => coord,
        _ => panic!("tobj.c:1776 assert"),
    }
}

/// `HSD_Index2TexCoord`.
pub fn index_to_tex_coord(index: u32) -> u32 {
    match index {
        0..=7 => index,
        _ => panic!("tobj.c:1800 assert"),
    }
}

/// `HSD_TexMtx2Index`.
pub fn tex_mtx_to_index(texmtx: u32) -> u32 {
    match texmtx {
        30 | 33 | 36 | 39 | 42 | 45 | 48 | 51 | 54 | 57 | 60 => (texmtx - GX_TEXMTX0) / 3,
        _ => panic!("tobj.c:1845 specified texmtx id desn't exist."),
    }
}

/// `HSD_Index2TexMtx`.
pub fn index_to_tex_mtx(index: u32) -> u32 {
    match index {
        0..=10 => GX_TEXMTX0 + 3 * index,
        _ => panic!("texmtx index exceed hardware limit ({index})."),
    }
}

/// `HSD_Index2TexMap`.
pub fn index_to_tex_map(index: u32) -> u32 {
    match index {
        0..=7 => index,
        _ => panic!("tobj.c:1915 assert"),
    }
}

/// `HSD_TexMap2Index`.
pub fn tex_map_to_index(mapid: u32) -> u32 {
    match mapid {
        0..=7 => mapid,
        _ => panic!("tobj.c:1939 assert"),
    }
}

/// `DifferentTluts` (static; non-MUST_MATCH: compares only the entry counts).
pub fn different_tluts(t0: &HsdTlut, t1: &HsdTlut) -> bool {
    t0.n_entries != t1.n_entries
}

impl<I: Clone> HsdTObj<I> {
    /// `MakeTextureMtx` (static): rebuilds `mtx` from scale / rotate / translate / repeat.
    pub fn make_texture_mtx(&mut self) {
        assert!(self.repeat_s != 0 && self.repeat_t != 0, "tobj.c:589 assert");
        let sx = if self.scale.x.abs() < TOBJ_FLT_EPSILON { 0.0 } else { self.repeat_s as f32 / self.scale.x };
        let sy = if self.scale.y.abs() < TOBJ_FLT_EPSILON { 0.0 } else { self.repeat_t as f32 / self.scale.y };
        let sz = self.scale.z;
        let rot = Vec3 { x: self.rotate.x, y: self.rotate.y, z: -self.rotate.z };
        let tx = -self.translate.x;
        let mirror = if self.wrap_t == GX_MIRROR { 1.0 / (self.repeat_t as f32 / self.scale.y) } else { 0.0 };
        let ty = -(self.translate.y + mirror);
        let tz = self.translate.z;
        self.mtx = mtx::trans(tx, ty, tz);
        let m = mk_rotation_mtx(&rot);
        self.mtx = mtx::concat(&m, &self.mtx);
        let m = mtx::scale(sx, sy, sz);
        self.mtx = mtx::concat(&m, &self.mtx);
    }

    /// `TObjUpdateFunc` (static): applies one animated value.
    pub fn update_func(&mut self, ty: u32, fv: f32) {
        fn c8(fv: f32) -> u8 {
            ((255.0 * fv as f64) as u32) as u8
        }
        match ty {
            track::TIMG => {
                let tbl = self.imagetbl.as_ref().expect("tobj.c:276 assert");
                let n = f2i32(fv as f64);
                if let Some(d) = &tbl[n as usize] {
                    self.imagedesc = Some(d.clone());
                }
            }
            track::TCLT => {
                if self.has_tluttbl {
                    self.tlut_no = f2i32(fv as f64);
                }
            }
            track::BLEND | track::TS_BLEND => self.blending = fv,
            track::TRAU..=track::ROTZ => {
                match ty {
                    track::ROTX => self.rotate.x = fv,
                    track::ROTY => self.rotate.y = fv,
                    track::ROTZ => self.rotate.z = fv,
                    track::TRAU => self.translate.x = fv,
                    track::TRAV => self.translate.y = fv,
                    track::SCAU => self.scale.x = fv,
                    _ => self.scale.y = fv,
                }
                self.flags |= TEX_MTX_DIRTY;
            }
            track::LOD_BIAS => self.lod.as_mut().expect("NULL lod").lod_bias = fv,
            track::KONST_R..=track::TEV1_A => {
                let tev = self.tev.as_mut().expect("NULL tev");
                let i = ty - track::KONST_R;
                let c = match i / 4 {
                    0 => &mut tev.konst,
                    1 => &mut tev.tev0,
                    _ => &mut tev.tev1,
                };
                let v = c8(fv);
                match i % 4 {
                    0 => c.r = v,
                    1 => c.g = v,
                    2 => c.b = v,
                    _ => c.a = v,
                }
            }
            _ => {}
        }
    }
}
