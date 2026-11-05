//! Port of the logic of `src/sysdolphin/baselib/fog.c`: fog / fog-adjust init from a desc or the
//! viewport, `FogUpdateFunc`, the anim entry points, and `HSD_FogSet`'s parameter computation
//! (range centre, the fog-adjust projection rebuilt from `GXGetProjectionv`) with the GX calls
//! going through [`FogGx`].
//!
//! Native shape: `HsdFog<O>` owns its `HsdFogAdj` (`Option<Box<_>>`; the C shares it by refcount,
//! FogRelease's `ref_DEC` + release/destroy is the drop) and its AObj (`HsdAObj<O>`, `O` = the
//! AObj's JObj handle, as in aobj.rs). The class header (`HSD_Obj`) is not kept. GcLayout is
//! delegated to `melee_types::HSD_Fog` / `HSD_FogAdj` / `HSD_FogDesc` / `HSD_FogAdjDesc`.
//!
//! Float rule: no fused site (`HSD_FogSet`'s range is `v0 + v2 * c / 640`, the divide sits between
//! the multiply and the add). Conversions as gwtool lowers them: `(s32)` saturating; `u8` colour
//! and `u16` widths are `fptoui` saturated to 32 bits then truncated; `s16` centre `fptosi` the
//! same ([`crate::cabi::f2i16`]).
//!
//! Note: `PSMTXCopy` / `PSMTXIdentity` touch only the first three rows of the `Mtx44`; row 3 of a
//! desc-initialised FogAdj keeps what it had (zero from `hsdNew`).
//!
//! Not ported: `FogRelease`, `FogInfoInit`, `FogAdjInfoInit` (class system), `HSD_FogAlloc` /
//! `HSD_FogAdjAlloc` beyond a zeroed native record.
use crate::aobj::{AObjGlobals, HsdAObj, HsdAObjDesc};
use crate::cabi::f2i16;
use crate::hsd_util::GxColor;
use crate::mtx::Mtx44;

/// `GX_FOG_LIN`.
pub const GX_FOG_LIN: u32 = 2;

macro_rules! delegate_layout {
    ($t:ty, $g:ty) => {
        impl melee_core::gc::GcLayout for $t {
            const GC_NAME: &'static str = <$g as melee_core::gc::GcLayout>::GC_NAME;
            const GC_SIZE: u32 = <$g as melee_core::gc::GcLayout>::GC_SIZE;
            const GC_ALIGN: u32 = <$g as melee_core::gc::GcLayout>::GC_ALIGN;
            const GC_FIELDS: &'static [melee_core::gc::GcField] = <$g as melee_core::gc::GcLayout>::GC_FIELDS;
        }
    };
}

/// Native `HSD_FogAdj` (GameCube record 0x40).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HsdFogAdj {
    pub center: i16,
    pub width: u16,
    pub mtx: Mtx44,
}
delegate_layout!(HsdFogAdj, melee_types::HSD_FogAdj);

/// Native `HSD_Fog` (GameCube record 0x20).
#[derive(Clone, Debug, PartialEq)]
pub struct HsdFog<O> {
    pub ty: u32,
    pub fog_adj: Option<Box<HsdFogAdj>>,
    pub start: f32,
    pub end: f32,
    pub color: GxColor,
    pub aobj: Option<HsdAObj<O>>,
}
impl<O> melee_core::gc::GcLayout for HsdFog<O> {
    const GC_NAME: &'static str = <melee_types::HSD_Fog as melee_core::gc::GcLayout>::GC_NAME;
    const GC_SIZE: u32 = <melee_types::HSD_Fog as melee_core::gc::GcLayout>::GC_SIZE;
    const GC_ALIGN: u32 = <melee_types::HSD_Fog as melee_core::gc::GcLayout>::GC_ALIGN;
    const GC_FIELDS: &'static [melee_core::gc::GcField] = <melee_types::HSD_Fog as melee_core::gc::GcLayout>::GC_FIELDS;
}

/// Native `HSD_FogAdjDesc` (0x44).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HsdFogAdjDesc {
    pub center: u16,
    pub width: u16,
    pub mtx: Mtx44,
}
delegate_layout!(HsdFogAdjDesc, melee_types::HSD_FogAdjDesc);

/// Native `HSD_FogDesc` (0x14).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HsdFogDesc {
    pub ty: u32,
    pub fogadjdesc: Option<HsdFogAdjDesc>,
    pub start: f32,
    pub end: f32,
    pub color: GxColor,
}
delegate_layout!(HsdFogDesc, melee_types::HSD_FogDesc);

/// The GX calls fog.c makes (the platform's GX shim implements them).
pub trait FogGx {
    /// What `GXInitFogAdjTable` fills (`GXFogAdjTable`).
    type Table;
    fn set_fog(&mut self, ty: u32, start: f32, end: f32, near: f32, far: f32, color: GxColor);
    /// `GXGetViewportv`.
    fn get_viewport(&mut self) -> [f32; 6];
    /// `GXGetProjectionv`: `[type, p0..p5]`.
    fn get_projection(&mut self) -> [f32; 7];
    fn init_fog_adj_table(&mut self, width: u16, mtx: &Mtx44) -> Self::Table;
    fn set_fog_range_adj(&mut self, enable: bool, center: u16, table: Option<&Self::Table>);
}

/// `(u16) f` / `(u8) f` from a float: fctiwz then truncation (gwtool lowers every fpto[su]i narrower than 32 bits that way).
fn f2u16_unsigned(f: f32) -> u16 {
    melee_core::ppc::fctiwz_f32(f) as u16
}
fn f2u8_unsigned(f: f32) -> u8 {
    melee_core::ppc::fctiwz_f32(f) as u8
}

impl HsdFogAdj {
    /// `HSD_FogAdjInit` (the C's NULL `adj` check is the caller's). `viewport` is consulted only
    /// for a NULL desc, as the C calls `GXGetViewportv` only then.
    pub fn init(&mut self, desc: Option<&HsdFogAdjDesc>, gx: &mut impl FogGx) {
        match desc {
            Some(d) => {
                self.width = d.width;
                self.center = d.center as i16;
                self.mtx[..3].copy_from_slice(&d.mtx[..3]);
            }
            None => {
                let v = gx.get_viewport();
                self.width = f2u16_unsigned(v[2]);
                self.center = (self.width / 2) as i16;
                self.mtx[..3].copy_from_slice(&crate::mtx::IDENTITY);
                self.mtx[3] = [0.0, 0.0, 0.0, 1.0];
            }
        }
    }

    /// `HSD_FogAdjLoadDesc` (alloc = a zeroed record).
    pub fn load_desc(desc: Option<&HsdFogAdjDesc>, gx: &mut impl FogGx) -> Box<Self> {
        let mut a = Box::<Self>::default();
        a.init(desc, gx);
        a
    }
}

impl<O> HsdFog<O> {
    /// `HSD_FogAlloc`: the zeroed record.
    pub fn alloc() -> Self {
        HsdFog { ty: 0, fog_adj: None, start: 0.0, end: 0.0, color: GxColor::default(), aobj: None }
    }

    /// `HSD_FogInit` (NULL `fog` is the caller's).
    pub fn init(&mut self, desc: Option<&HsdFogDesc>, gx: &mut impl FogGx) {
        match desc {
            Some(d) => {
                self.ty = d.ty;
                self.start = d.start;
                self.end = d.end;
                self.color = d.color;
            }
            None => {
                let v = gx.get_viewport();
                self.ty = GX_FOG_LIN;
                self.start = v[4];
                self.end = v[5];
                self.color = GxColor { r: 0xFF, g: 0xFF, b: 0xFF, a: 0xFF };
            }
        }
    }

    /// `HSD_FogLoadDesc` (the C dereferences `desc`; it is required here).
    pub fn load_desc(desc: &HsdFogDesc, gx: &mut impl FogGx) -> Self {
        let mut f = Self::alloc();
        f.init(Some(desc), gx);
        if let Some(ad) = &desc.fogadjdesc {
            f.fog_adj = Some(HsdFogAdj::load_desc(Some(ad), gx));
        }
        f
    }

    /// `FogUpdateFunc`.
    pub fn update_func(&mut self, ty: u32, fv: f32) {
        match ty {
            1 => self.start = fv,
            2 => self.end = fv,
            5 => self.color.r = f2u8_unsigned(255.0f32 * fv),
            6 => self.color.g = f2u8_unsigned(255.0f32 * fv),
            7 => self.color.b = f2u8_unsigned(255.0f32 * fv),
            8 => self.color.a = f2u8_unsigned(255.0f32 * fv),
            20 => {
                if let Some(a) = self.fog_adj.as_mut() {
                    a.center = f2i16(fv as f64);
                }
            }
            _ => {}
        }
    }

    /// `HSD_Fog_8037DE7C` (HSD_FogAddAnim): replace the AObj with one loaded from `desc`.
    pub fn add_anim(&mut self, desc: Option<&HsdAObjDesc>, resolve_obj: impl FnOnce(u32) -> Option<O>) {
        if let Some(a) = self.aobj.take() {
            a.remove();
        }
        self.aobj = desc.map(|d| HsdAObj::load_desc(d, resolve_obj));
    }

    /// `HSD_FogInterpretAnim`.
    pub fn interpret_anim(&mut self, g: &mut AObjGlobals) {
        if let Some(mut a) = self.aobj.take() {
            {
                let mut cb = |t: u8, fv: f32| self.update_func(t as u32, fv);
                a.interpret_anim(g, &mut Some(&mut cb));
            }
            self.aobj = Some(a);
        }
    }
}

/// `HSD_FogReqAnimByFlags`.
pub fn req_anim_by_flags<O>(fog: Option<&mut HsdFog<O>>, flags: u32, frame: f32) {
    let Some(fog) = fog else { return };
    if flags & 0x200 == 0 {
        return;
    }
    if let Some(a) = fog.aobj.as_mut() {
        a.req_anim(frame);
    }
}

/// `HSD_FogReqAnim`.
pub fn req_anim<O>(fog: Option<&mut HsdFog<O>>, frame: f32) {
    req_anim_by_flags(fog, 0x7FF, frame);
}

/// `HSD_FogSet`. `cobj_near_far` is `HSD_CObjGetCurrent()`'s near / far (`None`: the C panics).
pub fn fog_set<O, G: FogGx>(fog: Option<&HsdFog<O>>, cobj_near_far: Option<(f32, f32)>, gx: &mut G) {
    let Some(fog) = fog else {
        gx.set_fog(0, 0.0, 0.0, 0.0, 0.0, GxColor::default());
        return;
    };
    let Some((near, far)) = cobj_near_far else { panic!("You must specify CObj first.") };
    gx.set_fog(fog.ty, fog.start, fog.end, near, far, fog.color);
    let Some(adj) = fog.fog_adj.as_deref() else {
        gx.set_fog_range_adj(false, 0, None);
        return;
    };
    let v = gx.get_viewport();
    let mut range = melee_core::ppc::fctiwz_f32(v[0] + v[2] * ((adj.center as i32 + 320) as f32) / 640.0f32);
    if range < 0 {
        range = 0;
    }
    if range > 640 {
        range = 640;
    }
    let tbl = if adj.width != 0 {
        gx.init_fog_adj_table(adj.width, &adj.mtx)
    } else {
        let mut m: Mtx44 = [[0.0; 4]; 4];
        let p = gx.get_projection();
        match melee_core::ppc::fctiwz_f32(p[0]) {
            0 => {
                m[0][0] = p[1];
                m[0][2] = p[2];
                m[1][1] = p[3];
                m[1][2] = p[4];
                m[2][2] = p[5];
                m[2][3] = p[6];
                m[3][2] = -1.0;
            }
            1 => {
                m[0][0] = p[1];
                m[0][3] = p[2];
                m[1][1] = p[3];
                m[1][3] = p[4];
                m[2][2] = p[5];
                m[2][3] = p[6];
                m[3][3] = 1.0;
            }
            _ => {}
        }
        gx.init_fog_adj_table(f2u16_unsigned(v[2]), &m)
    };
    gx.set_fog_range_adj(true, range as u16, Some(&tbl));
}
