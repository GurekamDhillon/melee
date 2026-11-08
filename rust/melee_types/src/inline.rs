//! The `static inline` helpers defined in group-0 headers (sysdolphin/baselib/*.h), ported to
//! native Rust over the generated structs. `HSD_ASSERT(p)` on a pointer becomes a reference;
//! other HSD_ASSERTs become `assert!` (the PC build aborts on them too).
//!
//! Not here (they need functions owned by other groups; see docs/rust-port/group-0 ...md):
//! HSD_MtxColMag (MSL sqrtf/__frsqrte), vec_normalize_check (PSVECNormalize), atan2f_check
//! (atan2f), randi (HSD_Randi), HSD_AObjGet*Frame, HSD_PadGetNml*, HSD_VIGet*, HSD_IDGetData,
//! HSD_CObjGetViewingMtxPtrDirect, HSD_JObjRef*, hsdDelete, HSD_WObj*, HSD_PerfCountMtxLoad,
//! HSD_SisLib_BytePtr, HSD_JObjSetupMatrix, HSD_JObjGetMtxPtr, HSD_JObjCopyMtx.
#![allow(non_snake_case)]

use crate::gen::*;

/// The jobj.c functions the JObj setters call. Implemented by the runtime group (jobj.c).
/// `set_mtx_dirty` is the out-of-line `(HSD_JObjSetMtxDirty)(jobj)`; `set_mtx_dirty_sub` is
/// `HSD_JObjSetMtxDirtySub`, called by the macro form after its own dirty check.
pub trait JObjDirty {
    fn set_mtx_dirty(&mut self, jobj: &mut HSD_JObj);
    fn set_mtx_dirty_sub(&mut self, jobj: &mut HSD_JObj);
}

// ---- mtx.h ----
/// `fabsf_bitwise`: clear the sign bit.
#[inline]
pub fn fabsf_bitwise(v: f32) -> f32 {
    f32::from_bits(v.to_bits() & !0x8000_0000)
}
pub fn HSD_MtxColVec(mtx: &Mtx, col: usize, vec: &mut Vec3) {
    vec.x = mtx[0][col];
    vec.y = mtx[1][col];
    vec.z = mtx[2][col];
}
pub fn HSD_MtxSetColVec(mtx: &mut Mtx, col: usize, vec: &Vec3) {
    mtx[0][col] = vec.x;
    mtx[1][col] = vec.y;
    mtx[2][col] = vec.z;
}

// ---- object.h ----
pub const HSD_OBJ_NOREF: u16 = u16::MAX;
/// `ref_DEC`: true if the object is unreferenced-forever, else post-decrement and test old == 0.
pub fn ref_DEC(o: &mut HSD_Obj) -> bool {
    if o.ref_count == HSD_OBJ_NOREF {
        return true;
    }
    let old = o.ref_count;
    o.ref_count = old.wrapping_sub(1);
    old == 0
}
pub fn ref_INC(o: Option<&mut HSD_Obj>) {
    if let Some(o) = o {
        o.ref_count = o.ref_count.wrapping_add(1);
        assert!(o.ref_count != HSD_OBJ_NOREF, "object.h:93");
    }
}
pub fn ref_CNT(o: &HSD_Obj) -> i32 {
    if o.ref_count == HSD_OBJ_NOREF { -1 } else { o.ref_count as i32 }
}
pub fn iref_CNT(o: &HSD_Obj) -> i32 {
    o.ref_count_individual as i32
}
pub fn iref_DEC(o: &mut HSD_Obj) -> bool {
    if o.ref_count_individual == 0 {
        return true;
    }
    o.ref_count_individual -= 1;
    o.ref_count_individual == 0
}
pub fn iref_INC(o: &mut HSD_Obj) {
    o.ref_count_individual = o.ref_count_individual.wrapping_add(1);
    assert!(o.ref_count_individual != 0, "object.h:158");
}

// ---- objalloc.h ----
pub fn HSD_ObjAllocGetUsing(d: &HSD_ObjAllocData) -> u32 { d.used }
pub fn HSD_ObjAllocGetFreed(d: &HSD_ObjAllocData) -> u32 { d.free }
pub fn HSD_ObjAllocGetPeak(d: &HSD_ObjAllocData) -> u32 { d.peak }
pub fn HSD_ObjAllocSetNumLimit(d: &mut HSD_ObjAllocData, n: u32) { d.num_limit = n }
pub fn HSD_ObjAllocEnableNumLimit(d: &mut HSD_ObjAllocData) { d.set_num_limit_flag(1) }
pub fn HSD_ObjAllocDisableNumLimit(d: &mut HSD_ObjAllocData) { d.set_num_limit_flag(0) }

// ---- gobj.h ----
pub fn HSD_GObjGetUserData(g: &HSD_GObj) -> *mut core::ffi::c_void { g.user_data }
pub fn HSD_GObjGetHSDObj(g: &HSD_GObj) -> *mut core::ffi::c_void { g.hsd_obj }
pub fn HSD_GObjGetClassifier(g: &HSD_GObj) -> u16 { g.classifier }
pub fn HSD_GObjGetNext(g: &HSD_GObj) -> *mut HSD_GObj { g.next }

// ---- lobj.h ----
pub fn HSD_LObjGetPriority(l: &HSD_LObj) -> u8 { l.priority as u8 } // C returns u8 from a u16 field: truncates
pub fn HSD_LObjGetNext(l: Option<&HSD_LObj>) -> *mut HSD_LObj { l.map_or(core::ptr::null_mut(), |l| l.next) }
pub fn HSD_LObjSetNext(l: Option<&mut HSD_LObj>, next: *mut HSD_LObj) {
    if let Some(l) = l {
        l.next = next;
    }
}

// ---- robj.h ----
pub fn RObjHasFlags(r: &HSD_RObj) -> bool { r.flags & ROBJ_TYPE_MASK as u32 == 0 }
pub fn RObjHasFlags2(r: &HSD_RObj) -> bool { r.flags & 0x8000_0000 != 0 }
pub fn RObjHasLimitReftype(r: &HSD_RObj) -> bool { r.flags & ROBJ_TYPE_MASK as u32 == REFTYPE_LIMIT as u32 }

// ---- texp.h ----
pub fn IsThroughColor(t: &HSD_TExp) -> bool {
    let tev = unsafe { &t.tev };
    tev.c_op as u32 == GX_TEV_ADD as u32 && tev.c_in[0].sel as u32 == HSD_TE_0 as u32
        && tev.c_in[1].sel as u32 == HSD_TE_0 as u32 && tev.c_bias == 0 && tev.c_scale == 0
}
pub fn IsThroughAlpha(t: &HSD_TExp) -> bool {
    let tev = unsafe { &t.tev };
    tev.a_op as u32 == GX_TEV_ADD as u32 && tev.a_in[0].sel as u32 == HSD_TE_0 as u32
        && tev.a_in[1].sel as u32 == HSD_TE_0 as u32 && tev.a_bias == 0 && tev.a_scale == 0
}

// ---- jobj.h ----
pub fn HSD_JObjGetChild(j: Option<&HSD_JObj>) -> *mut HSD_JObj { j.map_or(core::ptr::null_mut(), |j| j.child) }
pub fn HSD_JObjGetNext(j: Option<&HSD_JObj>) -> *mut HSD_JObj { j.map_or(core::ptr::null_mut(), |j| j.next) }
pub fn HSD_JObjGetParent(j: Option<&HSD_JObj>) -> *mut HSD_JObj { j.map_or(core::ptr::null_mut(), |j| j.parent) }
pub fn HSD_JObjGetRObj(j: &HSD_JObj) -> *mut HSD_RObj { j.robj }
pub fn HSD_JObjMtxIsDirty(j: &HSD_JObj) -> bool {
    j.flags & JOBJ_USER_DEF_MTX as u32 == 0 && j.flags & JOBJ_MTX_DIRTY as u32 != 0
}
/// The `HSD_JObjSetMtxDirty(jobj)` macro / `HSD_JObjSetMtxDirtyInline`.
pub fn HSD_JObjSetMtxDirtyInline(j: &mut HSD_JObj, h: &mut impl JObjDirty) {
    if !HSD_JObjMtxIsDirty(j) {
        h.set_mtx_dirty_sub(j);
    }
}
#[inline]
fn indep(j: &HSD_JObj) -> bool {
    j.flags & JOBJ_MTX_INDEP_SRT as u32 != 0
}
#[inline]
fn no_quat(j: &HSD_JObj, line: u32) {
    assert!(j.flags & JOBJ_USE_QUATERNION as u32 == 0, "jobj.h:{line}");
}
/// Defines the macro-dirty variant `$a` and the out-of-line-dirty variant `$b` of a JObj setter.
macro_rules! jset {
    ($a:ident, $b:ident, |$j:ident, $v:ident : $t:ty| $body:block) => {
        pub fn $a($j: &mut HSD_JObj, $v: $t, h: &mut impl JObjDirty) {
            $body
            if !indep($j) { HSD_JObjSetMtxDirtyInline($j, h) }
        }
        pub fn $b($j: &mut HSD_JObj, $v: $t, h: &mut impl JObjDirty) {
            $body
            if !indep($j) { h.set_mtx_dirty($j) }
        }
    };
    ($a:ident, |$j:ident, $v:ident : $t:ty| $body:block) => {
        pub fn $a($j: &mut HSD_JObj, $v: $t, h: &mut impl JObjDirty) {
            $body
            if !indep($j) { HSD_JObjSetMtxDirtyInline($j, h) }
        }
    };
}
jset!(HSD_JObjSetRotation, HSD_JObjSetRotationWithMtxDirty, |j, r: &Quaternion| { j.rotate = *r; });
jset!(HSD_JObjSetRotationX, HSD_JObjSetRotationXWithMtxDirty, |j, x: f32| { no_quat(j, 640); j.rotate.x = x; });
jset!(HSD_JObjSetRotationY, HSD_JObjSetRotationYWithMtxDirty, |j, y: f32| { no_quat(j, 661); j.rotate.y = y; });
jset!(HSD_JObjSetRotationZ, HSD_JObjSetRotationZWithMtxDirty, |j, z: f32| { no_quat(j, 682); j.rotate.z = z; });
jset!(HSD_JObjSetScale, HSD_JObjSetScaleWithMtxDirty, |j, s: &Vec3| { j.scale = *s; });
jset!(HSD_JObjSetScaleX, HSD_JObjSetScaleXWithMtxDirty, |j, x: f32| { j.scale.x = x; });
jset!(HSD_JObjSetScaleY, HSD_JObjSetScaleYWithMtxDirty, |j, y: f32| { j.scale.y = y; });
jset!(HSD_JObjSetScaleZ, HSD_JObjSetScaleZWithMtxDirty, |j, z: f32| { j.scale.z = z; });
jset!(HSD_JObjSetTranslate, HSD_JObjSetTranslateWithMtxDirty, |j, t: &Vec3| { j.translate = *t; });
jset!(HSD_JObjSetTranslateX, HSD_JObjSetTranslateXWithMtxDirty, |j, x: f32| { j.translate.x = x; });
jset!(HSD_JObjSetTranslateY, HSD_JObjSetTranslateYWithMtxDirty, |j, y: f32| { j.translate.y = y; });
jset!(HSD_JObjSetTranslateZ, HSD_JObjSetTranslateZWithMtxDirty, |j, z: f32| { j.translate.z = z; });
jset!(HSD_JObjAddRotationX, HSD_JObjAddRotationXWithMtxDirty, |j, x: f32| { j.rotate.x += x; });
jset!(HSD_JObjAddRotationY, |j, y: f32| { j.rotate.y += y; });
/// `HSD_JObjAddRotationY(j, -(a * c))` as MWCC compiles it once the inline is expanded: the product and the add are
/// one `fnmsubs` (`rotate.y - a*c`, single rounding). ft_800CB6EC (Yoshi's turning double jump; Dolphin oracle,
/// root bone rotate.y 3fa78d37 for a = 15, c = 0.01745329252).
jset!(HSD_JObjAddRotationYFnmsubs, |j, ac: (f32, f32)| { j.rotate.y = melee_core::fp::fnmsubs(ac.0, ac.1, j.rotate.y); });
jset!(HSD_JObjAddRotationZ, |j, z: f32| { j.rotate.z += z; });
jset!(HSD_JObjAddScaleX, |j, x: f32| { j.scale.x += x; });
jset!(HSD_JObjAddScaleY, |j, y: f32| { j.scale.y += y; });
jset!(HSD_JObjAddScaleZ, |j, z: f32| { j.scale.z += z; });
jset!(HSD_JObjAddTranslationX, |j, x: f32| { j.translate.x += x; });
jset!(HSD_JObjAddTranslationY, HSD_JObjAddTranslationYWithMtxDirty, |j, y: f32| { j.translate.y += y; });
jset!(HSD_JObjAddTranslationZ, |j, z: f32| { j.translate.z += z; });

/// `HSD_JObjSetTranslateWithMtxDirtyOutOfLine` == the out-of-line form.
pub fn HSD_JObjSetTranslateWithMtxDirtyOutOfLine(j: &mut HSD_JObj, t: &Vec3, h: &mut impl JObjDirty) {
    HSD_JObjSetTranslateWithMtxDirty(j, t, h)
}
pub fn HSD_JObjGetRotation(j: &HSD_JObj, r: &mut Quaternion) { *r = j.rotate }
pub fn HSD_JObjGetRotationX(j: &HSD_JObj) -> f32 { j.rotate.x }
pub fn HSD_JObjGetRotationY(j: &HSD_JObj) -> f32 { j.rotate.y }
pub fn HSD_JObjGetRotationZ(j: &HSD_JObj) -> f32 { j.rotate.z }
pub fn HSD_JObjGetScale(j: &HSD_JObj, s: &mut Vec3) { *s = j.scale }
pub fn HSD_JObjGetScaleX(j: &HSD_JObj) -> f32 { j.scale.x }
pub fn HSD_JObjGetScaleY(j: &HSD_JObj) -> f32 { j.scale.y }
pub fn HSD_JObjGetScaleZ(j: &HSD_JObj) -> f32 { j.scale.z }
pub fn HSD_JObjGetTranslation(j: &HSD_JObj, t: &mut Vec3) { *t = j.translate }
pub fn HSD_JObjGetTranslation2(j: &HSD_JObj, t: &mut Vec3) { *t = j.translate }
pub fn HSD_JObjGetTranslationX(j: &HSD_JObj) -> f32 { j.translate.x }
pub fn HSD_JObjGetTranslationY(j: &HSD_JObj) -> f32 { j.translate.y }
pub fn HSD_JObjGetTranslationZ(j: &HSD_JObj) -> f32 { j.translate.z }

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Rec { sub: u32, full: u32 }
    impl JObjDirty for Rec {
        fn set_mtx_dirty(&mut self, _: &mut HSD_JObj) { self.full += 1 }
        fn set_mtx_dirty_sub(&mut self, j: &mut HSD_JObj) { self.sub += 1; j.flags |= JOBJ_MTX_DIRTY as u32 }
    }

    #[test]
    fn jobj_dirty_rules() {
        let mut j: HSD_JObj = unsafe { core::mem::zeroed() };
        let mut h = Rec::default();
        HSD_JObjSetTranslateX(&mut j, 2.0, &mut h);
        HSD_JObjSetTranslateY(&mut j, 3.0, &mut h); // already dirty: no second Sub call
        assert_eq!((h.sub, h.full, j.translate.x, j.translate.y), (1, 0, 2.0, 3.0));
        HSD_JObjSetScaleXWithMtxDirty(&mut j, 1.5, &mut h);
        assert_eq!(h.full, 1);
        j.flags = JOBJ_MTX_INDEP_SRT as u32;
        HSD_JObjAddRotationZ(&mut j, 0.25, &mut h);
        assert_eq!((h.sub, j.rotate.z), (1, 0.25));
        j.flags = JOBJ_USER_DEF_MTX as u32 | JOBJ_MTX_DIRTY as u32;
        assert!(!HSD_JObjMtxIsDirty(&j));
    }

    #[test]
    fn refcounts_wrap_like_c() {
        let mut o: HSD_Obj = unsafe { core::mem::zeroed() };
        assert!(ref_DEC(&mut o)); // 0 -> 0xFFFF (== NOREF), returns old == 0
        assert_eq!(ref_CNT(&o), -1);
        assert!(ref_DEC(&mut o)); // NOREF stays
        o.ref_count = 2;
        assert!(!ref_DEC(&mut o));
        assert_eq!(ref_CNT(&o), 1);
        iref_INC(&mut o);
        assert!(iref_DEC(&mut o));
        assert!(iref_DEC(&mut o)); // already 0
    }

    #[test]
    fn fabsf_bitwise_clears_sign_only() {
        assert_eq!(fabsf_bitwise(-0.0).to_bits(), 0);
        assert_eq!(fabsf_bitwise(-1.5), 1.5);
        let nan = f32::from_bits(0xFFC0_0001);
        assert_eq!(fabsf_bitwise(nan).to_bits(), 0x7FC0_0001);
    }

    #[test]
    fn objalloc_limit_flag_is_gc_msb() {
        let mut d: HSD_ObjAllocData = unsafe { core::mem::zeroed() };
        HSD_ObjAllocEnableNumLimit(&mut d);
        assert_eq!(d.bf_0[0], 0x80);
        HSD_ObjAllocDisableNumLimit(&mut d);
        assert_eq!(d.bf_0[0], 0);
    }
}
