//! src/melee/lb/lbgx.c: `lbGx_8001E2F8`, the item shadow/halo quad (a GX state setup and one
//! 5-vertex triangle strip). Checked against the C by a state oracle (tests/tu_lbgx.rs over
//! gen/turef/tu_lbgx_ref.c, every GX call logged with its arguments).
#![allow(non_snake_case, non_upper_case_globals, dead_code, static_mut_refs)]

use melee_core::fp::fmadds;
use melee_types::*;

/// Callees in TUs that are not ported (dolphin GX, baselib cobj). Each is a settable hook; unset it
/// panics "TODO: <name> is not ported".
pub mod ext {
    use super::*;
    macro_rules! hook {
        ($h:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?) => {
            pub static mut $h: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $h } {
                    Some(f) => f($($a),*),
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            }
        };
    }
    // BEGIN GENERATED ext
    hook!(H_GXSetColorUpdate, GXSetColorUpdate, (a: u8));
    hook!(H_GXSetAlphaUpdate, GXSetAlphaUpdate, (a: u8));
    hook!(H_GXSetBlendMode, GXSetBlendMode, (a: i32, b: i32, c: i32, d: i32));
    hook!(H_GXSetAlphaCompare, GXSetAlphaCompare, (a: i32, b: u8, c: i32, d: i32, e: u8));
    hook!(H_GXSetZMode, GXSetZMode, (a: u8, b: i32, c: u8));
    hook!(H_GXSetZCompLoc, GXSetZCompLoc, (a: u8));
    hook!(H_GXSetNumTexGens, GXSetNumTexGens, (a: u8));
    hook!(H_GXSetTevClampMode, GXSetTevClampMode, (a: i32, b: i32));
    hook!(H_GXSetNumTevStages, GXSetNumTevStages, (a: u8));
    hook!(H_GXSetTevOrder, GXSetTevOrder, (a: i32, b: i32, c: i32, d: i32));
    hook!(H_GXSetTevOp, GXSetTevOp, (a: i32, b: i32));
    hook!(H_GXSetNumChans, GXSetNumChans, (a: u8));
    hook!(H_GXSetChanCtrl, GXSetChanCtrl, (a: i32, b: u8, c: i32, d: i32, e: u32, f: i32, g: i32));
    hook!(H_GXSetCullMode, GXSetCullMode, (a: i32));
    hook!(H_GXClearVtxDesc, GXClearVtxDesc, ());
    hook!(H_GXSetVtxAttrFmt, GXSetVtxAttrFmt, (a: i32, b: i32, c: i32, d: i32, e: u8));
    hook!(H_GXSetVtxDesc, GXSetVtxDesc, (a: i32, b: i32));
    hook!(H_GXLoadPosMtxImm, GXLoadPosMtxImm, (a: *const [f32; 4], b: u32));
    hook!(H_GXSetCurrentMtx, GXSetCurrentMtx, (a: u32));
    hook!(H_GXBegin, GXBegin, (a: i32, b: i32, c: u16));
    hook!(H_GXPosition3f32, GXPosition3f32, (a: f32, b: f32, c: f32));
    hook!(H_GXPosition2u8, GXPosition2u8, (a: u8, b: u8));
    hook!(H_HSD_CObjGetCurrent, HSD_CObjGetCurrent, () -> *mut HSD_CObj);
    // END GENERATED ext
}

/// `lbGx_8001E2F8(arg0, arg1, arg2, arg3, argf1)`: when `arg3 == 2`, sets the GX state and draws a
/// 5-vertex triangle strip around `arg1 + arg0.xy * argf1` (half extents `arg0.z`, `arg0.w`) with
/// colour `arg2`, and returns true; otherwise does nothing and returns false.
///
/// # Safety
/// `arg0`, `arg1`, `arg2` must be valid for reads.
pub unsafe fn lbGx_8001E2F8(arg0: *const Vec4, arg1: *const Vec3, arg2: *const U8Vec4, arg3: u32, argf1: f32) -> bool {
    if arg3 != 2 {
        return false;
    }
    ext::GXSetColorUpdate(1);
    ext::GXSetAlphaUpdate(0);
    ext::GXSetBlendMode(1, 4, 5, 5);
    ext::GXSetAlphaCompare(4, 0, 0, 4, 0);
    ext::GXSetZMode(1, 3, 0);
    ext::GXSetZCompLoc(0);
    ext::GXSetNumTexGens(0);
    ext::GXSetTevClampMode(0, 0);
    ext::GXSetNumTevStages(1);
    ext::GXSetTevOrder(0, 255, 255, 4);
    ext::GXSetTevOp(0, 4);
    ext::GXSetNumChans(1);
    ext::GXSetChanCtrl(4, 0, 0, 1, 0, 0, 2);
    ext::GXSetCullMode(0);
    ext::GXClearVtxDesc();
    ext::GXSetVtxAttrFmt(0, 9, 1, 4, 0);
    ext::GXSetVtxAttrFmt(0, 11, 1, 5, 0);
    ext::GXSetVtxDesc(9, 1);
    ext::GXSetVtxDesc(11, 1);
    let cobj = ext::HSD_CObjGetCurrent();
    // HSD_CObjGetViewingMtxPtrDirect(cobj) == cobj->view_mtx
    ext::GXLoadPosMtxImm((*cobj).view_mtx.as_ptr(), 0);
    ext::GXSetCurrentMtx(0);
    ext::GXBegin(0x98, 0, 5); // GX_DRAW_TRIANGLE_STRIP
    let (a0, a1) = (&*arg0, &*arg1);
    let temp_f4 = fmadds(a0.x, argf1, a1.x);
    let temp_f2 = a0.z;
    let temp_f1 = a1.y + a0.y;
    let temp_f3 = a0.w;
    let temp_f5 = temp_f4 + temp_f2;
    let temp_f2_2 = temp_f4 - temp_f2;
    let temp_f4_2 = temp_f3 + temp_f1;
    let temp_f1_2 = temp_f1 - temp_f3;
    ext::GXPosition3f32(temp_f5, temp_f4_2, 0.0);
    let c = &*arg2;
    let (v_x, v_y, v_z, v_w) = (c.x, c.y, c.z, c.w);
    ext::GXPosition2u8(v_x, v_y);
    ext::GXPosition2u8(v_z, v_w);
    ext::GXPosition3f32(temp_f2_2, temp_f4_2, 0.0);
    ext::GXPosition2u8(v_x, v_y);
    ext::GXPosition2u8(v_z, v_w);
    ext::GXPosition3f32(temp_f5, temp_f1_2, 0.0);
    ext::GXPosition2u8(v_x, v_y);
    ext::GXPosition2u8(v_z, v_w);
    ext::GXPosition3f32(temp_f2_2, temp_f4_2, 0.0);
    ext::GXPosition2u8(v_x, v_y);
    ext::GXPosition2u8(v_z, v_w);
    ext::GXPosition3f32(temp_f2_2, temp_f1_2, 0.0);
    ext::GXPosition2u8(v_x, v_y);
    ext::GXPosition2u8(v_z, v_w);
    // GXEnd() is an empty static inline on PC
    true
}
