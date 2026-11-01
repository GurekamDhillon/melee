//! Port of src/melee/lb/lb_0146.c (a 2D segment test, and two small debug-draw routines that
//! draw a collision-line quad / a yellow line strip with GX). Checked against the C (whole TU,
//! shipping contraction) by tests/tu_lb_0146.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_core::fp::fmadds;
#[allow(unused_imports)]
use melee_types as mt;
use core::ffi::c_void;
use melee_types::*;

// BEGIN GENERATED EXT (rust/melee_modes/gen/cref/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? $(= $def:path)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => { $crate::ext_dflt!([$($def)?] ($($a),*) $name) }
                }
            } )*
        };
    }
    ext_fns! {
    fn GXBegin(a0: u32, a1: u32, a2: u16);
    fn GXClearVtxDesc();
    fn GXColor4u8(a0: u8, a1: u8, a2: u8, a3: u8);
    fn GXLoadPosMtxImm(a0: *mut c_void, a1: u32);
    fn GXPosition3f32(a0: f32, a1: f32, a2: f32);
    fn GXSetAlphaCompare(a0: u32, a1: u8, a2: u32, a3: u32, a4: u8);
    fn GXSetAlphaUpdate(a0: u8);
    fn GXSetBlendMode(a0: u32, a1: u32, a2: u32, a3: u32);
    fn GXSetChanCtrl(a0: u32, a1: u8, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32);
    fn GXSetColorUpdate(a0: u8);
    fn GXSetCullMode(a0: u32);
    fn GXSetCurrentMtx(a0: u32);
    fn GXSetLineWidth(a0: u8, a1: u32);
    fn GXSetNumChans(a0: u8);
    fn GXSetNumTevStages(a0: u8);
    fn GXSetNumTexGens(a0: u8);
    fn GXSetTevClampMode(a0: i32, a1: i32);
    fn GXSetTevOp(a0: u32, a1: u32);
    fn GXSetTevOrder(a0: u32, a1: u32, a2: u32, a3: u32);
    fn GXSetVtxAttrFmt(a0: u32, a1: u32, a2: u32, a3: u32, a4: u8);
    fn GXSetVtxDesc(a0: u32, a1: u32);
    fn GXSetZCompLoc(a0: u8);
    fn GXSetZMode(a0: u8, a1: u32, a2: u8);
    fn HSD_CObjGetCurrent() -> *mut HSD_CObj = crate::tu::adapt::HSD_CObjGetCurrent__6f08dd;
    fn HSD_StateInitDirect(a0: i32, a1: u32);
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT


// GX enum values used here (dolphin/gx/*.h).
const GX_TRUE: u8 = 1;
const GX_FALSE: u8 = 0;
const GX_BM_BLEND: u32 = 1;
const GX_BL_SRCALPHA: u32 = 4;
const GX_BL_INVSRCALPHA: u32 = 5;
const GX_LO_NOOP: u32 = 5;
const GX_GREATER: u32 = 4;
const GX_AOP_AND: u32 = 0;
const GX_LEQUAL: u32 = 3;
const GX_TEVSTAGE0: u32 = 0;
const GX_TEXCOORD_NULL: u32 = 0xFF;
const GX_TEXMAP_NULL: u32 = 0xFF;
const GX_COLOR0A0: u32 = 4;
const GX_PASSCLR: u32 = 4;
const GX_SRC_REG: u32 = 0;
const GX_SRC_VTX: u32 = 1;
const GX_DF_NONE: u32 = 0;
const GX_AF_NONE: u32 = 2;
const GX_CULL_NONE: u32 = 0;
const GX_VTXFMT0: u32 = 0;
const GX_VA_POS: u32 = 9;
const GX_POS_XYZ: u32 = 1;
const GX_F32: u32 = 4;
const GX_VA_CLR0: u32 = 0xB;
const GX_CLR_RGBA: u32 = 1;
const GX_RGBA8: u32 = 5;
const GX_DIRECT: u32 = 1;
const GX_TRIANGLESTRIP: u32 = 0x98;
const GX_LINESTRIP: u32 = 0xB0;
const GX_TO_ONE: u32 = 5;

/// 4D3760
pub use melee_runtime::shared::lb_ColorWhite;
/// 4D3764
pub use melee_runtime::shared::lb_ColorBlack;
/// 4D3768 (static)
pub static mut red: GXColor = GXColor { r: 255, g: 0, b: 0, a: 255 };
/// 4D376C (static)
pub static mut translucent_red: GXColor = GXColor { r: 255, g: 0, b: 0, a: 64 };
/// 4D3770 (static)
pub static mut yellow: GXColor = GXColor { r: 255, g: 255, b: 0, a: 255 };

#[inline(always)]
unsafe fn vert(x: f32, y: f32, z: f32, c: &GXColor) {
    ext::GXPosition3f32(x, y, z);
    ext::GXColor4u8(c.r, c.g, c.b, c.a);
}

/// 014770: draws the quad of a vertical collision line (`arg1 == 2`).
pub unsafe fn lb_80014770(arg0: *mut Vec3, arg1: i32) -> bool {
    melee_core::ct!("lb_80014770");
    if arg1 as u32 == 2 {
        ext::GXSetColorUpdate(GX_TRUE);
        ext::GXSetAlphaUpdate(GX_FALSE);
        ext::GXSetBlendMode(GX_BM_BLEND, GX_BL_SRCALPHA, GX_BL_INVSRCALPHA, GX_LO_NOOP);
        ext::GXSetAlphaCompare(GX_GREATER, 0, GX_AOP_AND, GX_GREATER, 0);
        ext::GXSetZMode(GX_TRUE, GX_LEQUAL, GX_FALSE);
        ext::GXSetZCompLoc(GX_FALSE);
        ext::GXSetNumTexGens(0);
        ext::GXSetTevClampMode(GX_TEVSTAGE0 as i32, 0);
        ext::GXSetNumTevStages(1);
        ext::GXSetTevOrder(GX_TEVSTAGE0, GX_TEXCOORD_NULL, GX_TEXMAP_NULL, GX_COLOR0A0);
        ext::GXSetTevOp(GX_TEVSTAGE0, GX_PASSCLR);
        ext::GXSetNumChans(1);
        ext::GXSetChanCtrl(GX_COLOR0A0, GX_FALSE, GX_SRC_REG, GX_SRC_VTX, 0, GX_DF_NONE, GX_AF_NONE);
        ext::GXSetCullMode(GX_CULL_NONE);
        ext::GXClearVtxDesc();
        ext::GXSetVtxAttrFmt(GX_VTXFMT0, GX_VA_POS, GX_POS_XYZ, GX_F32, 0);
        ext::GXSetVtxAttrFmt(GX_VTXFMT0, GX_VA_CLR0, GX_CLR_RGBA, GX_RGBA8, 0);
        ext::GXSetVtxDesc(GX_VA_POS, GX_DIRECT);
        ext::GXSetVtxDesc(GX_VA_CLR0, GX_DIRECT);
        let mtx = (*ext::HSD_CObjGetCurrent()).view_mtx.as_mut_ptr() as *mut c_void;
        ext::GXLoadPosMtxImm(mtx, 0);
        ext::GXSetCurrentMtx(0);
        ext::GXBegin(GX_TRIANGLESTRIP, GX_VTXFMT0, 5);

        let (near_pt, far_pt, near_clr, far_clr);
        if (*arg0.add(1)).z > (*arg0).z {
            near_pt = arg0.add(1);
            far_pt = arg0;
            near_clr = &raw const red;
            far_clr = &raw const translucent_red;
        } else {
            near_pt = arg0;
            far_pt = arg0.add(1);
            near_clr = &raw const translucent_red;
            far_clr = &raw const red;
        }

        vert((*near_pt).y, (*near_pt).z, 0.0, &*near_clr);
        vert((*near_pt).x, (*near_pt).z, 0.0, &*near_clr);
        vert((*far_pt).y, (*far_pt).z, 0.0, &*far_clr);
        vert((*near_pt).x, (*near_pt).z, 0.0, &*near_clr);
        vert((*far_pt).x, (*far_pt).z, 0.0, &*far_clr);
        // GXEnd(): empty without DEBUG
        return true;
    }
    false
}

/// 0149E0: draws a yellow line strip from a collision-line matrix (`arg1 == 0`).
pub unsafe fn lb_800149E0(arg0: *mut [f32; 4], arg1: u32) -> bool {
    melee_core::ct!("lb_800149E0");
    if arg1 == 0 {
        ext::HSD_StateInitDirect(0, 2);
        let mtx = (*ext::HSD_CObjGetCurrent()).view_mtx.as_mut_ptr() as *mut c_void;
        ext::GXLoadPosMtxImm(mtx, 0);
        ext::GXSetLineWidth(12, GX_TO_ONE);
        ext::GXBegin(GX_LINESTRIP, GX_VTXFMT0, 2);
        let y = (*arg0.add(1))[0];
        vert((*arg0)[2], y, 0.0, &*(&raw const yellow));
        vert((*arg0)[3], y, 0.0, &*(&raw const yellow));
        return true;
    }
    false
}

// ---- folded from lb_ptr.rs (consolidation steps 21/22) ----
/// lb_0146.c `lb_80014638` (two fused sites).
pub fn lb_80014638(arg0: &mt::lb_80014638_arg0_t, arg1: &mt::lb_80014638_arg1_t) -> bool {
    melee_core::ct!("lb_80014638");
    let mut sp30 = arg0.x0;
    let sp24 = arg0.xC;
    let sp18 = arg1.x8;
    sp30.x = sp30.x + arg1.unk_x;
    sp30.y += arg1.unk_x;
    sp30.z += arg1.unk_y;
    if sp30.z < sp24.z {
        return false;
    }
    if sp30.z < sp18.z || sp24.z > sp18.z {
        return false;
    }
    let d = sp24.z - sp30.z;
    let z = if (if d < 0.0 { -d } else { d }) < 0.01f32 { 1.0 } else { (sp18.z - sp30.z) / d };
    if fmadds(z, sp24.x - sp30.x, sp30.x) > sp18.y {
        return false;
    }
    if fmadds(z, sp24.y - sp30.y, sp30.y) < sp18.x {
        return false;
    }
    true
}
