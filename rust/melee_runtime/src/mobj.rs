//! Port of the logic of `src/sysdolphin/baselib/mobj.c` (the material object): the flag and
//! colour setters, `HSD_MObjGetTObj`, `HSD_MObjSetCurrent` and `MObjUpdateFunc`.
//!
//! Native shape: `HsdMObj<T>` keeps the fields these functions touch; `T` is the TObj handle.
//! `mat` and `pe` are owned records (`None` = NULL; the C dereferences `mat` unconditionally,
//! the port panics). GcLayout is delegated to `melee_types::HSD_MObj` / `HSD_Material` /
//! `HSD_PEDesc`.
//!
//! Float rule: no fused site in these functions (helper_mobj.ll: 0 fmuladd). `(u8)(255.0 * fv)`
//! is `fptoui` saturated to 32 bits then truncated, as gwtool lowers it (same as tobj.c).
//!
//! Not ported: the anim entry points (need TObj anim), load/alloc/class functions, TEV/TExp
//! compile and GX setup, toon/shadow texture lists.
use crate::hsd_util::GxColor;

macro_rules! delegate_layout {
    ($t:ty, $g:ty $(, $p:ident)?) => {
        impl$(<$p>)? melee_core::gc::GcLayout for $t {
            const GC_NAME: &'static str = <$g as melee_core::gc::GcLayout>::GC_NAME;
            const GC_SIZE: u32 = <$g as melee_core::gc::GcLayout>::GC_SIZE;
            const GC_ALIGN: u32 = <$g as melee_core::gc::GcLayout>::GC_ALIGN;
            const GC_FIELDS: &'static [melee_core::gc::GcField] = <$g as melee_core::gc::GcLayout>::GC_FIELDS;
        }
    };
}

/// `HSD_A_M_*` track types.
pub mod track {
    pub const AMBIENT_R: u32 = 1;
    pub const AMBIENT_G: u32 = 2;
    pub const AMBIENT_B: u32 = 3;
    pub const DIFFUSE_R: u32 = 4;
    pub const DIFFUSE_G: u32 = 5;
    pub const DIFFUSE_B: u32 = 6;
    pub const SPECULAR_R: u32 = 7;
    pub const SPECULAR_G: u32 = 8;
    pub const SPECULAR_B: u32 = 9;
    pub const ALPHA: u32 = 10;
    pub const PE_REF0: u32 = 11;
    pub const PE_REF1: u32 = 12;
    pub const PE_DSTALPHA: u32 = 13;
}

/// `RENDER_TOON`.
pub const RENDER_TOON: u32 = 1 << 12;

/// `HSD_Material`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HsdMaterial {
    pub ambient: GxColor,
    pub diffuse: GxColor,
    pub specular: GxColor,
    pub alpha: f32,
    pub shininess: f32,
}
delegate_layout!(HsdMaterial, melee_types::HSD_Material);

/// `HSD_PEDesc`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HsdPeDesc {
    pub flags: u8,
    pub ref0: u8,
    pub ref1: u8,
    pub dst_alpha: u8,
    pub ty: u8,
    pub src_factor: u8,
    pub dst_factor: u8,
    pub logic_op: u8,
    pub z_comp: u8,
    pub alpha_comp0: u8,
    pub alpha_op: u8,
    pub alpha_comp1: u8,
}
delegate_layout!(HsdPeDesc, melee_types::HSD_PEDesc);

/// `HSD_MObj` (the fields the ported logic uses).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HsdMObj<T> {
    pub rendermode: u32,
    pub tobj: Option<T>,
    pub mat: Option<HsdMaterial>,
    pub pe: Option<HsdPeDesc>,
}
delegate_layout!(HsdMObj<T>, melee_types::HSD_MObj, T);

/// mobj.c's file static `current_mobj`, as the caller's MObj handle.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MObjGlobals<M> {
    pub current_mobj: Option<M>,
}

impl<M> MObjGlobals<M> {
    /// `HSD_MObjSetCurrent`.
    pub fn set_current(&mut self, mobj: Option<M>) {
        self.current_mobj = mobj;
    }
}

/// `HSD_MObjSetFlags`.
pub fn set_flags<T>(mobj: Option<&mut HsdMObj<T>>, flags: u32) {
    if let Some(m) = mobj {
        m.rendermode |= flags;
    }
}

/// `HSD_MObjClearFlags`.
pub fn clear_flags<T>(mobj: Option<&mut HsdMObj<T>>, flags: u32) {
    if let Some(m) = mobj {
        m.rendermode &= !flags;
    }
}

/// `HSD_MObjGetTObj`.
pub fn get_tobj<T: Clone>(mobj: Option<&HsdMObj<T>>) -> Option<T> {
    mobj.and_then(|m| m.tobj.clone())
}

fn c8(fv: f32) -> u8 {
    ((255.0 * fv as f64) as u32) as u8
}

impl<T> HsdMObj<T> {
    fn mat(&mut self) -> &mut HsdMaterial {
        self.mat.as_mut().expect("NULL mobj->mat")
    }

    /// `HSD_MObjSetDiffuseColor` (no NULL check in the C).
    pub fn set_diffuse_color(&mut self, r: u8, g: u8, b: u8) {
        let m = self.mat();
        m.diffuse.r = r;
        m.diffuse.g = g;
        m.diffuse.b = b;
    }

    /// `HSD_MObjSetAlpha`.
    pub fn set_alpha(&mut self, alpha: f32) {
        self.mat().alpha = alpha;
    }

    /// `MObjUpdateFunc` (static): applies one animated value. A NULL mobj is a no-op in the C;
    /// callers holding `None` skip the call.
    pub fn update_func(&mut self, ty: u32, fv: f32) {
        use track::*;
        match ty {
            AMBIENT_R => self.mat().ambient.r = c8(fv),
            AMBIENT_G => self.mat().ambient.g = c8(fv),
            AMBIENT_B => self.mat().ambient.b = c8(fv),
            DIFFUSE_R => self.mat().diffuse.r = c8(fv),
            DIFFUSE_G => self.mat().diffuse.g = c8(fv),
            DIFFUSE_B => self.mat().diffuse.b = c8(fv),
            ALPHA => self.mat().alpha = 1.0 - fv,
            SPECULAR_R => self.mat().specular.r = c8(fv),
            SPECULAR_G => self.mat().specular.g = c8(fv),
            SPECULAR_B => self.mat().specular.b = c8(fv),
            PE_REF0 => {
                if let Some(p) = &mut self.pe {
                    p.ref0 = c8(fv)
                }
            }
            PE_REF1 => {
                if let Some(p) = &mut self.pe {
                    p.ref1 = c8(fv)
                }
            }
            PE_DSTALPHA => {
                if let Some(p) = &mut self.pe {
                    p.dst_alpha = c8(fv)
                }
            }
            _ => {}
        }
    }
}
