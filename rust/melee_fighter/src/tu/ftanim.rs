//! Native port of src/melee/ft/ftanim.c (fighter skeleton animation: figatree binding, blend
//! between the anim skeleton and the model, part (hand) animations, costume texture anims),
//! TARGET_PC variant (bounded part walks, m-ex `FT_ANIM_KIND_SELF`, Geno "none" defines).
//!
//! Already ported in `crate::ftanim` and re-exported: `scale_inline`, `get_part_joint`,
//! `ftAnim_8007077C`, `ftAnim_80070FB4`. Float math is plain (`a / (a + (b - c))`, products
//! with the model scale, no `a*b+c` in one expression: no fused sites). Callees in baselib, lb,
//! ftparts/ftdata/ftcommon/ftaction that are not ported go through `ext`.
#![allow(non_snake_case, static_mut_refs, clippy::missing_safety_doc)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::ftanim::{G as G_ftanim, fp};
use crate::hooks as h;
use crate::types::*;
use melee_types::inline::*;

pub use crate::ftanim::{ftanim_8007077c as ftAnim_8007077C, ftanim_80070fb4 as ftAnim_80070FB4};
type G = *mut HSD_GObj;
type J = *mut HSD_JObj;

/// `AOBJ_MASK | ... | CBOBJ_MASK` (ftAnim_8006EBE8's mask) == the 0xFB7F literal used elsewhere.
pub const ANIM_MASK: u32 = 0xFB7F;

/// Calls into TUs that are not ported yet. Each panics until a later pass rewires it.
pub mod ext {
    use crate::types::*;
    type G = *mut HSD_GObj;
    type J = *mut HSD_JObj;
    /// `HSD_ForeachAnim(jobj, JOBJ_TYPE, mask, HSD_AObjSetFlags, AOBJ_ARG_AU, AOBJ_LOOP)`.
    /// `HSD_ForeachAnim(jobj, JOBJ_TYPE, mask, HSD_AObjSetRate, AOBJ_ARG_AF, rate)`.
    /// `ftData_UnkCallbackPairs0[kind].x0` / `.x4`.
    /// `CostumeListsForeachCharacter[kind].costume_list[costume].x4`.
    /// `FT_COSTUME_VIS_IDX(fp)`.
    pub fn OSReport(_s: &str) {}
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? = $d:expr;)*) => {
            pub mod hook {
                use super::*;
                $( #[allow(non_upper_case_globals)] pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( #[allow(unreachable_code, unused_variables)] pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => $d,
                }
            } )*
        };
    }
    ext_fns! {
        fn ftParts_8007506C(_kind: i32, _i: i32) -> u32 = { crate::ftdmgfall::ftparts_8007506c(_kind, _i) };
        fn ftParts_GetBoneIndex(_fp: *mut Fighter, _part: i32) -> i32 = { unsafe { crate::ftdmgfall::ftparts_get_bone_index(&*_fp, _part) } };
        fn ftPartsRemap(_a: u32, _b: u32, _c: u32) -> i32 = { crate::ftdmgfall::ftparts_remap(_a, _b, _c) };
        fn lbVector_Diff(_a: *mut Vec3, _b: *mut Vec3, _out: *mut Vec3) = { unsafe { (*_out).x = (*_a).x - (*_b).x; (*_out).y = (*_a).y - (*_b).y; (*_out).z = (*_a).z - (*_b).z; } };
        fn lbVector_Sub(_a: *mut Vec3, _b: *mut Vec3) = { unsafe { (*_a).x -= (*_b).x; (*_a).y -= (*_b).y; (*_a).z -= (*_b).z; } };
        fn HSD_JObjGetFlags(_j: J) -> u32 = { unsafe { (*_j).flags } };
        fn Geno_DefineTemplateSkeleton(_kind: i32) -> i32 = { crate::hooks::geno_define_template_skeleton(_kind) };
        fn ftPartGetRotX(_fp: *mut Fighter, _idx: i32) -> f32 = { unsafe { crate::tu::ftparts::ftPartGetRotX(&mut *_fp, _idx) } };
        fn HSD_AObjInitEndCallBack() = { unsafe { melee_runtime::tu::aobj::HSD_AObjInitEndCallBack(); } };
        fn HSD_AObjInvokeCallBacks() = { unsafe { melee_runtime::tu::aobj::HSD_AObjInvokeCallBacks(); } };
        fn HSD_AObjSetRate(_a: *mut HSD_AObj, _r: f32) = { unsafe { melee_runtime::tu::aobj::HSD_AObjSetRate(_a as _, _r as _); } };
        fn HSD_AObjReqAnim(_a: *mut HSD_AObj, _f: f32) = { unsafe { melee_runtime::tu::aobj::HSD_AObjReqAnim(_a as _, _f as _); } };
        fn HSD_TObjAnim(_t: *mut HSD_TObj) = { unsafe { melee_runtime::tu::tobj::HSD_TObjAnim(_t as _); } };
        fn HSD_JObjAnim(_j: J) = { unsafe { melee_runtime::tu::jobj_ptr::HSD_JObjAnim(_j as _) } };
        fn HSD_JObjAnimAll(_j: J) = { unsafe { melee_runtime::tu::jobj_ptr::HSD_JObjAnimAll(_j as _) } };
        fn HSD_JObjSetMtxDirtySub(_j: J) = { unsafe { melee_runtime::tu::jobj_ptr::HSD_JObjSetMtxDirtySub(_j as _) } };
        fn HSD_JObjClearFlags(_j: J, _f: u32) = { unsafe { melee_runtime::tu::jobj_ptr::HSD_JObjClearFlags(_j as _, _f) } };
        fn HSD_JObjClearFlagsAll(_j: J, _f: u32) = { unsafe { melee_runtime::tu::jobj_ptr::HSD_JObjClearFlagsAll(_j as _, _f) } };
        fn HSD_JObjAddAnim(_j: J, _a: *mut HSD_AnimJoint, _m: *mut HSD_MatAnimJoint, _s: *mut HSD_ShapeAnimJoint) = { unsafe { melee_runtime::tu::jobj_ptr::HSD_JObjAddAnim(_j as _, _a as _, _m as _, _s as _); } };
        fn HSD_JObjAddAnimAll(_j: J, _a: *mut HSD_AnimJoint, _m: *mut HSD_MatAnimJoint, _s: *mut HSD_ShapeAnimJoint) = { unsafe { melee_runtime::tu::jobj_ptr::HSD_JObjAddAnimAll(_j as _, _a as _, _m as _, _s as _) } };
        fn HSD_JObjReqAnimAll(_j: J, _f: f32) = { unsafe { melee_runtime::tu::jobj_ptr::HSD_JObjReqAnimAll(_j as _, _f) } };
        fn HSD_JObjReqAnimAllByFlags(_j: J, _flags: u32, _f: f32) = { unsafe { melee_runtime::tu::jobj_ptr::HSD_JObjReqAnimAllByFlags(_j as _, _flags, _f) } };
        fn HSD_JObjReqAnimByFlags(_j: J, _flags: u32, _f: f32) = { unsafe { melee_runtime::tu::jobj_ptr::HSD_JObjReqAnimByFlags(_j as _, _flags, _f) } };
        fn HSD_JObjRemoveAnimAllByFlags(_j: J, _flags: u32) = { unsafe { melee_runtime::tu::jobj_ptr::HSD_JObjRemoveAnimAllByFlags(_j as _, _flags) } };
        fn HSD_ForeachAnim_SetLoop(_j: J, _mask: u32) = { todo!("HSD_ForeachAnim (baselib/object.c)") };
        fn HSD_ForeachAnim_SetRate(_j: J, _mask: u32, _rate: f32) = { todo!("HSD_ForeachAnim (baselib/object.c)") };
        fn HSD_MtxInverse(_src: *mut Mtx, _dst: *mut Mtx) = { unsafe { let _ = crate::bridge::hsd_mtx_inverse(_src, _dst); } };
        fn PSMTXMultVec(_m: *mut Mtx, _src: *const Vec3, _dst: *mut Vec3) = { unsafe { let _ = crate::bridge::psmtx_mult_vec(_m, _src, _dst); } };
        fn lb_8000B1CC(_j: J, _off: *const Vec3, _out: *mut Vec3) = { unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(_j as _, _off as _, _out as _); } };
        fn lb_8000B074(_j: J) -> bool = { unsafe { melee_runtime::tu::lb_00B0::lb_8000B074(_j as _) } };
        fn lb_8000B4FC(_j: J, _joint: *mut HSD_Joint) = { unsafe { melee_runtime::tu::lb_00B0::lb_8000B4FC(_j as _, _joint as _); } };
        fn lb_8000B5DC(_j: J, _joint: *mut HSD_Joint) = { unsafe { melee_runtime::tu::lb_00B0::lb_8000B5DC(_j as _, _joint as _); } };
        fn lb_8000B6A4(_j: J, _joint: *mut HSD_Joint) = { unsafe { melee_runtime::tu::lb_00B0::lb_8000B6A4(_j as _, _joint as _); } };
        fn lb_8000B760(_j: J, _joint: *mut HSD_Joint) = { unsafe { melee_runtime::tu::lb_00B0::lb_8000B760(_j as _, _joint as _); } };
        fn lb_8000BFF0(_a: *mut HSD_AnimJoint) -> f32 = { unsafe { melee_runtime::tu::lb_00B0::lb_8000BFF0(_a as _) } };
        fn lb_8000C490(_a: J, _b: J, _c: J, _t: f32, _ti: f32) = { unsafe { melee_runtime::tu::lb_00B0::lb_8000C490(_a as _, _b as _, _c as _, _t as _, _ti as _); } };
        fn lb_8000C868(_joint: *mut HSD_Joint, _b: J, _c: J, _t: f32, _ti: f32) = { unsafe { melee_runtime::tu::lb_00B0::lb_8000C868(_joint as _, _b as _, _c as _, _t as _, _ti as _); } };
        fn lbCopyJObjSRT(_a: J, _b: J) = { unsafe { melee_runtime::tu::lb_00B0::lbCopyJObjSRT(_a as _, _b as _); } };
        fn lbGetJObjFramerate(_j: J) -> f32 = { unsafe { melee_runtime::tu::lb_00B0::lbGetJObjFramerate(_j as _) } };
        fn lbGetJObjCurrFrame(_j: J) -> f32 = { unsafe { melee_runtime::tu::lb_00B0::lbGetJObjCurrFrame(_j as _) } };
        fn lbGetJObjEndFrame(_j: J) -> f32 = { unsafe { melee_runtime::tu::lb_00B0::lbGetJObjEndFrame(_j as _) } };
        fn lbAnim_8001E6D8(_j: J, _t: *mut FigaTree, _tr: *mut FigaTrack, _n: i8) = { unsafe { melee_runtime::tu::lbanim::lbAnim_8001E6D8(_j as _, _t as _, _tr as _, _n as _); } };
        fn lbAnim_8001E7E8(_j: J, _t: *mut FigaTree, _tr: *mut FigaTrack, _n: i8) = { unsafe { melee_runtime::tu::lbanim::lbAnim_8001E7E8(_j as _, _t as _, _tr as _, _n as _); } };
        fn ftCommon_GetModelScale(_fp: *mut Fighter) -> f32 = { unsafe { crate::ftcommon2::get_model_scale(&*(_fp as *const _)) } };
        fn ftCommon_8007F6A4(_fp: *mut Fighter, _j: J) = { unsafe { let _ = crate::ftcommon3::ftcommon_8007f6a4(&mut *(_fp as *mut _), _j as _); } };
        fn ftParts_8007482C(_joint: *mut HSD_Joint) -> J = { unsafe { crate::tu::ftparts_render::ftParts_8007482C(_joint) } };
        fn ftParts_8007462C(_g: G) = { unsafe { crate::tu::ftparts::ftParts_8007462C(_g) } };
        fn ftParts_80075240(_d: *mut DObjList, _idx: u16) -> *mut HSD_TObj = { unsafe { crate::tu::ftparts::ftParts_80075240(_d, _idx as i32) } };
        fn ftData_80085E50(_fp: *mut Fighter, _id: i32) -> *mut FigaTree = { unsafe { crate::tu::ftdata::ftData_80085E50(_fp as _, _id as _) as _ } };
        fn ftData_80085FD4(_fp: *mut Fighter, _id: i32) -> *mut Fighter_WaitAnimData = { unsafe { crate::tu::ftdata::ftData_80085FD4(&mut *(_fp as *mut _), _id as _) as _ } };
        fn ftData_UnkCallbackPairs0_x0(_kind: i32) -> Option<unsafe extern "C" fn(G)> = { unsafe { crate::tu::ftdata::ftData_UnkCallbackPairs0[_kind as usize].x0 } };
        fn ftData_UnkCallbackPairs0_x4(_kind: i32) -> Option<unsafe extern "C" fn(G, i32, f32)> = { unsafe { crate::tu::ftdata::ftData_UnkCallbackPairs0[_kind as usize].x4 } };
        fn costume_matanim(_kind: i32, _costume: u8) -> *mut HSD_MatAnimJoint = { unsafe { (*crate::tu::ftdata::CostumeListsForeachCharacter[_kind as usize].costume_list.add(_costume as usize)).x4 } };
        fn Mex_CostumeVisIdx(_kind: i32, _costume: i32) -> i32 = { unsafe { melee_runtime::tu::mex_data::Mex_CostumeVisIdx(_kind, _costume) as i32 } };
        fn ftAction_80073240(_g: G) = { unsafe { crate::tu::ftaction::ftAction_80073240(_g as _); } };
        fn ftCo_800DB500(_g: G) = { unsafe { crate::tu::ftCo_CapturePulled::ftCo_800DB500(_g as _); } };
        fn Geno_DefineIsNone(_kind: i32) -> i32 = { unsafe { crate::platform::Geno_DefineIsNone(_kind) } };
    }
}

struct Dirty;
impl JObjDirty for Dirty {
    fn set_mtx_dirty(&mut self, j: &mut HSD_JObj) { ext::HSD_JObjSetMtxDirtySub(j) }
    fn set_mtx_dirty_sub(&mut self, j: &mut HSD_JObj) { ext::HSD_JObjSetMtxDirtySub(j) }
}

#[inline(always)]
unsafe fn fpm<'a>(g: G) -> &'a mut Fighter { &mut *((*g).user_data as *mut Fighter) }
#[inline(always)]
unsafe fn bone<'a>(fp: &Fighter, i: i32) -> &'a mut FighterBone { &mut *fp.parts.offset(i as isize) }
#[inline(always)]
unsafe fn b0(b: &FighterBone) -> bool { b.x8.x0.flags_b0() != 0 }
#[inline(always)]
unsafe fn b1(b: &FighterBone) -> bool { b.x8.x0.flags_b1() != 0 }
#[inline(always)]
unsafe fn b2(b: &FighterBone) -> bool { b.x8.x0.flags_b2() != 0 }
#[inline(always)]
unsafe fn b3(b: &FighterBone) -> bool { b.x8.x0.flags_b3() != 0 }
#[inline(always)]
unsafe fn b4(b: &FighterBone) -> bool { b.x8.x0.flags_b4() != 0 }
#[inline(always)]
unsafe fn b5(b: &FighterBone) -> bool { b.x8.x0.flags_b5() != 0 }
#[inline(always)]
unsafe fn xc(b: &FighterBone) -> i32 { b.xC_u.x0.xC as i32 }
#[inline(always)]
unsafe fn x594(fp: &Fighter) -> &Fighter_x594_x0 { &fp.x594.x0 }
#[inline(always)]
unsafe fn x594_bits(fp: &Fighter) -> u32 { fp.x594.x0_1.x594_bits() }
unsafe fn parts_num(fp: &Fighter) -> i32 { (**h::ft_parts_table().offset(fp.kind as isize)).parts_num as i32 }
fn p8007506c(kind: i32, i: i32) -> u32 { ext::ftParts_8007506C(kind, i) }
fn bone_index(fp: &Fighter, part: i32) -> i32 { ext::ftParts_GetBoneIndex(fp as *const Fighter as *mut Fighter, part) }
unsafe fn x8_x10(fp: &Fighter) -> i32 { (*(*fp.ft_data).x8).x10 as i32 }
unsafe fn set_translate(j: J, v: &Vec3) {
    if j.is_null() { panic!("jobj.h: HSD_JObjSetTranslate: NULL jobj") } // the C's HSD_ASSERT
    HSD_JObjSetTranslate(&mut *j, v, &mut Dirty)
}

/// `ftAnim_AuthorKind` (static inline, TARGET_PC).
pub unsafe fn ftAnim_AuthorKind(fp: &Fighter) -> i32 {
    melee_core::ct!("ftAnim_AuthorKind");
    let k = fp.x594.x0_1.x597_bits() as i32;
    if k == FT_ANIM_KIND_SELF && fp.kind as i32 >= 64 {
        return fp.kind as i32;
    }
    k
}

static mut ftAnim_804590D8: [*mut HSD_AnimJoint; 30] = [core::ptr::null_mut(); 30];
static mut ftAnim_804590D8_unk: [*mut HSD_MatAnimJoint; 30] = [core::ptr::null_mut(); 30];
static mut ftAnim_804590D8_F0: [*mut HSD_Joint; 30] = [core::ptr::null_mut(); 30];

macro_rules! next_in_tree {
    ($name:ident, $t:ty, $stack:ident, $line:literal) => {
        /// Pre-order walk step over a joint tree, parents kept on the module stack.
        pub unsafe fn $name(p: &mut *mut $t, depth: &mut i32) {
            let cur = *p;
            let mut i = *depth;
            let next;
            if !(*cur).child.is_null() {
                $stack[i as usize] = cur;
                i += 1;
                if i >= 30 {
                    panic!(concat!("ftanim.c:", $line, ": fighter joint depth num over!"));
                }
                next = (*cur).child;
            } else if !(*cur).next.is_null() {
                next = (*cur).next;
            } else {
                loop {
                    if i == 0 {
                        next = core::ptr::null_mut();
                        break;
                    }
                    if !(*$stack[(i - 1) as usize]).next.is_null() {
                        i -= 1;
                        next = (*$stack[i as usize]).next;
                        break;
                    }
                    i -= 1;
                }
            }
            *p = next;
            *depth = i;
        }
    };
}
next_in_tree!(ftAnim_GetNextAnimJointInTree, HSD_AnimJoint, ftAnim_804590D8, "46");
next_in_tree!(ftAnim_GetNextMatAnimJointInTree, HSD_MatAnimJoint, ftAnim_804590D8_unk, "87");
next_in_tree!(ftAnim_GetNextJointInTree, HSD_Joint, ftAnim_804590D8_F0, "128");

/// `ftAnim_8006DF0C`.
pub unsafe fn ftAnim_8006DF0C(fp: &mut Fighter) {
    melee_core::ct!("ftAnim_8006DF0C");
    if fp.x2221_b2() != 0 {
        let root = bone(fp, 0).joint;
        let t = bone(fp, x8_x10(fp)).joint;
        let mut v = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        ext::lb_8000B1CC(bone(fp, bone_index(fp, FtPart_HipN as i32)).joint, &h::common_data().x808, &mut v);
        let mut m: Mtx = core::mem::zeroed();
        ext::HSD_MtxInverse(&mut (*root).mtx, &mut m);
        let src = v;
        ext::PSMTXMultVec(&mut m, &src, &mut v);
        set_translate(t, &v);
    }
}

/// `lbVector_Diff`: a - b per component.
fn vdiff(a: &Vec, b: &Vec) -> Vec { Vec { x: a.x - b.x, y: a.y - b.y, z: a.z - b.z } }

fn scale3(v: &mut Vec3, s: f32) {
    v.x *= s;
    v.y *= s;
    v.z *= s;
}

unsafe fn model_scale(fp: &mut Fighter) -> f32 {
    if x594(fp).x594_b6() == 0 { ext::ftCommon_GetModelScale(fp) } else { fp.co_attrs.model_scaling }
}

/// The C's depth-first jobj step (`!(flags & 0x1000) && child` / next / up), bounded by `stop`.
unsafe fn next_jobj(jobj: J, stop: J) -> J {
    let j = &*jobj;
    if ext::HSD_JObjGetFlags(jobj) & 0x1000 == 0 && !j.child.is_null() {
        return j.child;
    }
    if !j.next.is_null() {
        return j.next;
    }
    let mut cur = jobj;
    loop {
        let p = (*cur).parent;
        if p.is_null() || (!stop.is_null() && p == stop) {
            return core::ptr::null_mut();
        }
        if !(*p).next.is_null() {
            return (*p).next;
        }
        cur = p;
    }
}

/// `ftAnim_8006E054`: animate the tree, pulling TransN's (and x594_b5's 2nd joint's) motion
/// out into `x68C_transNPos` / `x6C0`.
pub unsafe fn ftAnim_8006E054(fp: &mut Fighter, mut jobj: J, arg2: J, arg3: J) {
    melee_core::ct!("ftAnim_8006E054");
    ext::HSD_AObjInitEndCallBack();
    while !jobj.is_null() {
        if jobj == arg2 {
            let zero = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            ext::HSD_JObjAnim(jobj);
            fp.x698 = fp.x68C_transNPos;
            HSD_JObjGetTranslation(&*jobj, &mut fp.x68C_transNPos);
            let s = model_scale(fp);
            scale3(&mut fp.x68C_transNPos, s);
            fp.x6B0 = fp.x6A4_transNOffset;
            ext::lbVector_Diff(&raw mut fp.x68C_transNPos, &raw mut fp.x698, &raw mut fp.x6A4_transNOffset);
            set_translate(jobj, &zero);
        } else if x594(fp).x594_b5() != 0 && jobj == arg3 {
            let zero = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            ext::HSD_JObjAnim(jobj);
            fp.x6CC = fp.x6C0;
            HSD_JObjGetTranslation(&*jobj, &mut fp.x6C0);
            let s = model_scale(fp);
            scale3(&mut fp.x6C0, s);
            fp.x6E4 = fp.x6D8;
            ext::lbVector_Diff(&raw mut fp.x6C0, &raw mut fp.x6CC, &raw mut fp.x6D8);
            set_translate(jobj, &zero);
        } else {
            ext::HSD_JObjAnim(jobj);
        }
        jobj = next_jobj(jobj, core::ptr::null_mut());
    }
    if x594(fp).x594_b5() != 0 {
        let v = Vec3 {
            x: fp.x68C_transNPos.x - fp.x6C0.x,
            y: fp.x68C_transNPos.y - fp.x6C0.y,
            z: fp.x68C_transNPos.z - fp.x6C0.z,
        };
        set_translate(arg2, &v);
        fp.x68C_transNPos = fp.x6C0;
        fp.x698 = fp.x6CC;
        fp.x6A4_transNOffset = fp.x6D8;
        fp.x6B0 = fp.x6E4;
    }
    if fp.x2226_b2() == 0 && fp.x2221_b2() != 0 {
        let j = bone(fp, x8_x10(fp)).joint;
        ext::ftCommon_GetModelScale(fp);
        let mut t = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        HSD_JObjGetTranslation(&*j, &mut t);
        let mut s = fp.x68C_transNPos;
        let sc = model_scale(fp);
        scale_inline(sc, &mut s);
        ext::lbVector_Sub(&mut t, &mut s);
        set_translate(j, &t);
    }
    ext::HSD_AObjInvokeCallBacks();
}

/// `ftAnim_8006E7B8` (TARGET_PC: the part walk is bounded by `parts_num`).
pub unsafe fn ftAnim_8006E7B8(fp: &mut Fighter, part: i32) {
    melee_core::ct!("ftAnim_8006E7B8");
    ext::HSD_AObjInitEndCallBack();
    let mut i = part;
    let top = bone(fp, part).joint;
    let depth = xc(bone(fp, part));
    let mut jobj = top;
    while !jobj.is_null() {
        let n = parts_num(fp);
        while i < n && !b1(bone(fp, i)) {
            i += 1;
        }
        if i >= n {
            ext::OSReport("ftAnim: part walk ran past the parts table; stopped\n");
            break;
        }
        if xc(bone(fp, i)) <= depth && i != part {
            break;
        }
        if !b0(bone(fp, i)) && !b5(bone(fp, i)) {
            ext::HSD_JObjAnim(jobj);
        }
        i += 1;
        jobj = next_jobj(jobj, top);
    }
    ext::HSD_AObjInvokeCallBacks();
}

/// `ftAnim_8006E9B4`: per-frame skeleton animation with blend.
pub unsafe fn ftAnim_8006E9B4(gobj: G) {
    melee_core::ct!("ftAnim_8006E9B4");
    let fp = fpm(gobj);
    if fp.anim_id == -1 {
        return;
    }
    let jobj = (*gobj).hsd_obj as J;
    // 8006E9E4 `lfs f1, 0.0` for the blend test: the f1 a part-less ftAnim_8006F3DC returns unless an AObj sets it.
    melee_runtime::tu::aobj::AOBJ_STALE_F1 = 0.0;
    if fp.x8A4_animBlendFrames == 0.0 {
        ext::HSD_JObjClearFlagsAll(jobj, JOBJ_USE_QUATERNION as u32);
        if x594(fp).x594_b0() != 0 {
            let b = bone(fp, bone_index(fp, 0x35)).joint; // C argument order: last first
            let a = bone(fp, bone_index(fp, FtPart_TransN as i32)).joint;
            ftAnim_8006E054(fp, jobj, a, b);
        } else {
            ftAnim_8006E7B8(fp, FtPart_TopN as i32);
        }
    } else {
        let anim = fp.x8AC_animSkeleton;
        let rate = ext::lbGetJObjFramerate(anim);
        fp.x8A8_anim_frame += rate;
        let (t, ti);
        if fp.x8A4_animBlendFrames <= fp.x8A8_anim_frame {
            fp.x8A8_anim_frame = fp.x8A4_animBlendFrames;
            t = 1.0f32;
            ti = 0.0f32;
        } else {
            let remaining = fp.x8A4_animBlendFrames - fp.x8A8_anim_frame;
            t = rate / (rate + remaining);
            ti = 1.0f32 - t;
        }
        ftAnim_8006E7B8(fp, FtPart_TopN as i32);
        if x594(fp).x594_b0() != 0 {
            let b = bone(fp, bone_index(fp, 0x35)).joint; // C argument order: last first
            let a = bone(fp, bone_index(fp, FtPart_TransN as i32)).x4_jobj2;
            ftAnim_8006E054(fp, anim, a, b);
        } else {
            ext::HSD_JObjAnimAll(anim);
        }
        if ti != 0.0 {
            ftAnim_8006FE9C(fp, FtPart_TransN as i32, t, ti);
        } else {
            ftAnim_8006FF74(fp, FtPart_TransN as i32);
        }
    }
    if x594(fp).x594_b2() != 0 && ftAnim_8006F3DC(gobj) < fp.cur_anim_frame {
        // 8006EB6C loads the old x898 into f1 for the add
        melee_runtime::tu::aobj::AOBJ_STALE_F1 = fp.x898_unk;
        fp.x898_unk += fp.cur_anim_frame + fp.frame_speed_mul;
    }
    fp.cur_anim_frame = ftAnim_8006F3DC(gobj);
}

/// `ftAnim_8006EBA4`.
pub unsafe fn ftAnim_8006EBA4(gobj: G) {
    melee_core::ct!("ftAnim_8006EBA4");
    ftAnim_8006E9B4(gobj);
    ext::ftAction_80073240(gobj);
    ftAnim_800707B0(gobj);
    ext::ftCo_800DB500(gobj);
}

/// `ftAnim_8006EBE8`.
pub unsafe fn ftAnim_8006EBE8(gobj: G, anim_start: f32, anim_rate: f32, anim_blend_frames: f32) {
    melee_core::ct!("ftAnim_8006EBE8");
    let root = (*gobj).hsd_obj as J;
    let fp = fpm(gobj);
    let anim = fp.x8AC_animSkeleton;
    ftAnim_80070758(root);
    ftAnim_80070758(anim);
    if anim_blend_frames == 0.0 {
        ftAnim_8006FA58(fp, FtPart_TransN as i32, (*fp.x108_costume_joint).child);
        ftAnim_8006FE08(fp, false);
        ftAnim_80070710(root, anim_start);
        if x594(fp).x594_b1_loop() != 0 {
            ext::HSD_ForeachAnim_SetLoop(root, ANIM_MASK);
        }
        ext::HSD_ForeachAnim_SetRate(root, ANIM_MASK, anim_rate);
    } else {
        ftAnim_8006FB88(fp, FtPart_TransN as i32, (*fp.x108_costume_joint).child);
        ftAnim_8006FE08(fp, true);
        ftAnim_80070710(anim, anim_start);
        ftAnim_80070710(root, anim_start);
        if x594(fp).x594_b1_loop() != 0 {
            ext::HSD_ForeachAnim_SetLoop(anim, ANIM_MASK);
            ext::HSD_ForeachAnim_SetLoop(root, ANIM_MASK);
        }
        ext::HSD_ForeachAnim_SetRate(anim, ANIM_MASK, anim_rate);
        ext::HSD_ForeachAnim_SetRate(root, ANIM_MASK, anim_rate);
    }
    fp.x8A4_animBlendFrames = anim_blend_frames;
    fp.x8A8_anim_frame = 0.0;
}

/// `ftAnim_8006EDD0`.
pub unsafe fn ftAnim_8006EDD0(fp: &mut Fighter, arg1: i32, frame: f32, rate: f32) {
    melee_core::ct!("ftAnim_8006EDD0");
    let jobj = fp.x8AC_animSkeleton;
    ftAnim_80070758(jobj);
    ftAnim_8006FB88(fp, FtPart_TransN as i32, (*fp.x108_costume_joint).child);
    let tree = ext::ftData_80085E50(fp, arg1);
    ftAnim_80070A10(fp, FtPart_TopN as i32, tree);
    ftAnim_80070710(jobj, frame);
    let r = ext::ftData_80085FD4(fp, arg1);
    if ((*r).x10_animCurrFlags as u32 >> 30) & 1 != 0 { // C: x10_b1 = bit 1 of the row's byte at +0x10 (MSB-first), i.e. bit 30 of the big-endian word
        ext::HSD_ForeachAnim_SetLoop(jobj, ANIM_MASK);
    }
    ext::HSD_ForeachAnim_SetRate(jobj, ANIM_MASK, rate);
}

/// `ftAnim_8006EED4`.
pub unsafe fn ftAnim_8006EED4(fp: &mut Fighter, part: i32, tree: *mut FigaTree, frame: f32, speed: f32) {
    melee_core::ct!("ftAnim_8006EED4");
    let j = bone(fp, part).joint;
    if fp.x8A4_animBlendFrames == 0.0 {
        let joint = ftAnim_8006F994(fp, j, fp.x108_costume_joint);
        ftAnim_8006FA58(fp, part, joint);
        ftAnim_8006F954(fp, part, false, tree);
        ftAnim_80070710(j, frame);
        let r = ext::ftData_80085FD4(fp, fp.anim_id);
        if ((*r).x10_animCurrFlags as u32 >> 30) & 1 != 0 { // C: x10_b1 = bit 1 of the row's byte at +0x10 (MSB-first), i.e. bit 30 of the big-endian word
            ext::HSD_ForeachAnim_SetLoop(j, ANIM_MASK);
        }
        ext::HSD_ForeachAnim_SetRate(j, ANIM_MASK, speed);
        ftAnim_8006E7B8(fp, part);
        return;
    }
    let j2 = bone(fp, part).x4_jobj2;
    let joint = ftAnim_8006F994(fp, j, fp.x108_costume_joint);
    ftAnim_8006FB88(fp, part, joint);
    ftAnim_8006F954(fp, part, true, tree);
    ftAnim_80070710(j2, frame);
    ftAnim_80070710(j, frame);
    if x594(fp).x594_b1_loop() != 0 {
        ext::HSD_ForeachAnim_SetLoop(j2, ANIM_MASK);
        ext::HSD_ForeachAnim_SetLoop(j, ANIM_MASK);
    }
    ext::HSD_ForeachAnim_SetRate(j2, ANIM_MASK, speed);
    ext::HSD_ForeachAnim_SetRate(j, ANIM_MASK, speed);
    ext::HSD_JObjAnimAll(j2);
    ftAnim_8006E7B8(fp, part);
}

/// `ftAnim_8006F0FC`.
pub unsafe fn ftAnim_8006F0FC(gobj: G, anim_rate: f32) {
    melee_core::ct!("ftAnim_8006F0FC");
    let fp = fpm(gobj);
    ext::HSD_ForeachAnim_SetRate((*gobj).hsd_obj as J, ANIM_MASK, anim_rate);
    ext::HSD_ForeachAnim_SetRate(fp.x8AC_animSkeleton, ANIM_MASK, anim_rate);
    fp.frame_speed_mul = anim_rate;
}

/// `ftAnim_SetAnimRate`.
pub unsafe fn ftAnim_SetAnimRate(gobj: G, anim_rate: f32) {
    melee_core::ct!("ftAnim_SetAnimRate");
    let fp = fpm(gobj);
    if fp.x2223_b0() != 0 {
        fp.x8A0_unk = anim_rate;
        return;
    }
    ftAnim_8006F0FC(gobj, anim_rate);
}

/// `ftAnim_IsFramesRemaining`.
pub unsafe fn ftAnim_IsFramesRemaining(gobj: G) -> bool {
    melee_core::ct!("ftAnim_IsFramesRemaining");
    let fp = fpm(gobj);
    let blend = fp.x8A4_animBlendFrames != 0.0;
    for i in 0..parts_num(fp) {
        let b = bone(fp, i);
        if b1(b) && !b0(b) && !b5(b) && ext::lb_8000B074(if blend { b.x4_jobj2 } else { b.joint }) {
            return true;
        }
    }
    false
}

/// `ftAnim_8006F368`.
pub unsafe fn ftAnim_8006F368(fp: &Fighter, part: i32) -> bool {
    melee_core::ct!("ftAnim_8006F368");
    let b = bone(fp, part);
    ext::lb_8000B074(if fp.x8A4_animBlendFrames == 0.0 { b.joint } else { b.x4_jobj2 })
}

/// `ftAnim_8006F3DC`: the current frame of the first animated part. The C falls off the end
/// (no return) when no part has an AObj; this returns 0.0 there.
pub unsafe fn ftAnim_8006F3DC(gobj: G) -> f32 {
    melee_core::ct!("ftAnim_8006F3DC");
    let fp = fpm(gobj);
    if fp.x8A4_animBlendFrames == 0.0 {
        for i in 0..parts_num(fp) {
            let b = bone(fp, i);
            if b1(b) && !b0(b) && !b5(b) {
                let a = (*b.joint).aobj;
                if !a.is_null() {
                    return (*a).curr_frame;
                }
            }
        }
        // The C has no return here: the console returns the caller's f1 (melee_runtime::tu::aobj::AOBJ_STALE_F1).
        melee_runtime::tu::aobj::AOBJ_STALE_F1
    } else {
        ext::lbGetJObjCurrFrame(fp.x8AC_animSkeleton)
    }
}

/// `ftAnim_8006F484`.
pub unsafe fn ftAnim_8006F484(gobj: G) -> f32 {
    melee_core::ct!("ftAnim_8006F484");
    let fp = fpm(gobj);
    if fp.x8A4_animBlendFrames == 0.0 {
        return ext::lbGetJObjEndFrame((*gobj).hsd_obj as J);
    }
    ext::lbGetJObjEndFrame(fp.x8AC_animSkeleton)
}

unsafe fn pj(fp: &Fighter, part: i32, blend: bool) -> J { get_part_joint(fp, part, blend) }

/// `ftAnim_8006F4C8`.
pub unsafe fn ftAnim_8006F4C8(fp: &mut Fighter, do_blending: bool, tree: *mut FigaTree) {
    melee_core::ct!("ftAnim_8006F4C8");
    let mut node = (*tree).nodes;
    let mut track = (*tree).tracks;
    let flags = x594_bits(fp);
    let mut i = 0;
    while *node != -1 {
        while !b1(bone(fp, i)) {
            i += 1;
        }
        while b2(bone(fp, i)) {
            let r = p8007506c(fp.kind as i32, i);
            if r == 0 || r & flags != 0 {
                break;
            }
            i += 1;
        }
        if i >= MAX_FT_PARTS {
            panic!("ftanim.c:767: atree data error! player {}", fp.player_idx);
        }
        if !b0(bone(fp, i)) && !b5(bone(fp, i)) {
            ext::lbAnim_8001E6D8(pj(fp, i, do_blending), tree, track, *node);
        }
        let frames = *node;
        node = node.add(1);
        i += 1;
        track = track.offset(frames as isize);
    }
}

/// `ftAnim_8006F628`: bind a figatree authored for another kind, remapping parts.
pub unsafe fn ftAnim_8006F628(fp: &mut Fighter, part: i32, do_blending: bool) {
    melee_core::ct!("ftAnim_8006F628");
    let tree = fp.x590;
    let mut node = (*tree).nodes;
    let bits = x594_bits(fp);
    let mut track = (*tree).tracks;
    let kind = ftAnim_AuthorKind(fp);
    let depth = xc(bone(fp, part));
    let mut i = 0;
    while i < part {
        let r = p8007506c(kind, i);
        if r == 0 || r & bits != 0 {
            track = track.offset(*node as isize);
            node = node.add(1);
        }
        i += 1;
    }
    while *node != -1 {
        loop {
            let r = p8007506c(kind, i);
            if r == 0 || r & bits != 0 {
                break;
            }
            i += 1;
        }
        if xc(bone(fp, i)) <= depth && i != part {
            break;
        }
        let tmp = ext::ftPartsRemap(fp.kind as u32, kind as u32, i as u32);
        if tmp as u32 != 0xFF {
            let b = bone(fp, tmp);
            if b1(b) && !b0(b) && !b5(b) {
                let jobj = pj(fp, tmp, do_blending);
                if jobj.is_null() || tmp >= parts_num(fp) {
                    ext::OSReport("ftanim guard: 8006F628\n");
                }
                if b3(b) {
                    ext::lbAnim_8001E6D8(jobj, tree, track, *node);
                } else {
                    ext::lbAnim_8001E7E8(jobj, tree, track, *node);
                }
            }
        }
        let frames = *node;
        node = node.add(1);
        i += 1;
        track = track.offset(frames as isize);
    }
}

/// `ftAnim_8006F7C8` and `ftAnim_80070A10` (the same walk; 80070A10 always binds x4_jobj2).
unsafe fn bind_own_tree(ft: &mut Fighter, part: i32, use_jobj2: bool, tree: *mut FigaTree) {
    let mut nodes = (*tree).nodes;
    let mut tracks = (*tree).tracks;
    let bits = x594_bits(ft);
    let depth = xc(bone(ft, part));
    let mut i = 0;
    while i < part {
        let r = p8007506c(ft.kind as i32, i);
        if r == 0 || r & bits != 0 {
            tracks = tracks.offset(*nodes as isize);
            nodes = nodes.add(1);
        }
        i += 1;
    }
    let mut r22 = i;
    while *nodes != -1 {
        while !b1(bone(ft, r22)) {
            i += 1;
            r22 += 1;
        }
        let mut r21 = i;
        loop {
            if !b2(bone(ft, r21)) {
                break;
            }
            let r = p8007506c(ft.kind as i32, i);
            if r == 0 || r & bits != 0 {
                break;
            }
            r21 += 1;
            i += 1;
            r22 += 1;
        }
            if xc(bone(ft, r22)) <= depth && i != part {
            break;
        }
        let b = bone(ft, r22);
        if !b0(b) && !b5(b) {
            ext::lbAnim_8001E6D8(if use_jobj2 { b.x4_jobj2 } else { b.joint }, tree, tracks, *nodes);
        }
        tracks = tracks.offset(*nodes as isize);
        i += 1;
        r22 += 1;
        nodes = nodes.add(1);
    }
}

/// `ftAnim_8006F7C8`.
pub unsafe fn ftAnim_8006F7C8(ft: &mut Fighter, part: i32, arg2: i32, tree: *mut FigaTree) { melee_core::ct!("ftAnim_8006F7C8"); bind_own_tree(ft, part, arg2 != 0, tree) }

/// `ftAnim_8006F954`.
pub unsafe fn ftAnim_8006F954(fp: &mut Fighter, part: i32, do_blending: bool, _unused: *mut FigaTree) {
    melee_core::ct!("ftAnim_8006F954");
    if fp.kind as i32 != ftAnim_AuthorKind(fp) {
        ftAnim_8006F628(fp, part, do_blending);
    } else {
        let t = fp.x590;
        ftAnim_8006F7C8(fp, part, do_blending as i32, t);
    }
}

/// `ftAnim_8006F994`: the HSD_Joint matching `jobj` (the C falls off the end with no return
/// when none does; this returns NULL there).
pub unsafe fn ftAnim_8006F994(fp: &Fighter, jobj: J, mut joint: *mut HSD_Joint) -> *mut HSD_Joint {
    melee_core::ct!("ftAnim_8006F994");
    let mut depth = 0;
    let mut i = 0;
    while !joint.is_null() {
        while p8007506c(fp.kind as i32, i) != 0 {
            i += 1;
        }
        let b = bone(fp, i);
        if jobj == b.joint || jobj == b.x4_jobj2 {
            return joint;
        }
        i += 1;
        ftAnim_GetNextJointInTree(&mut joint, &mut depth);
    }
    core::ptr::null_mut()
}

unsafe fn copy_joint_tree(fp: &mut Fighter, part: i32, mut joint: *mut HSD_Joint, second: bool) {
    let mut i = part;
    let mut depth = 0;
    while !joint.is_null() {
        while p8007506c(fp.kind as i32, i) != 0 {
            i += 1;
        }
        let b = bone(fp, i);
        let j = if second { b.x4_jobj2 } else { b.joint };
        if i == x8_x10(fp) {
            if b0(b) {
                ext::lb_8000B760(j, joint);
                if second {
                    ext::HSD_JObjClearFlags(j, JOBJ_USE_QUATERNION as u32);
                }
            } else if !b5(b) {
                ext::lb_8000B5DC(j, joint);
            }
            ext::ftCommon_8007F6A4(fp, j);
        } else if b0(b) {
            ext::lb_8000B6A4(j, joint);
            if second {
                ext::HSD_JObjClearFlags(j, JOBJ_USE_QUATERNION as u32);
            }
        } else if !b5(b) {
            ext::lb_8000B4FC(j, joint);
        }
        i += 1;
        ftAnim_GetNextJointInTree(&mut joint, &mut depth);
    }
}

/// `ftAnim_8006FA58`: reset the model joints to the joint descs.
pub unsafe fn ftAnim_8006FA58(fp: &mut Fighter, part: i32, joint: *mut HSD_Joint) { melee_core::ct!("ftAnim_8006FA58"); copy_joint_tree(fp, part, joint, false) }
/// `ftAnim_8006FB88`: same for the anim skeleton (x4_jobj2).
pub unsafe fn ftAnim_8006FB88(fp: &mut Fighter, part: i32, joint: *mut HSD_Joint) { melee_core::ct!("ftAnim_8006FB88"); copy_joint_tree(fp, part, joint, true) }

/// `ftAnim_8006FCE4`.
pub unsafe fn ftAnim_8006FCE4(fp: &mut Fighter, do_blending: bool) {
    melee_core::ct!("ftAnim_8006FCE4");
    let tree = fp.x590;
    let mut node = (*tree).nodes;
    let bits = x594_bits(fp);
    let mut track = (*tree).tracks;
    let kind = ftAnim_AuthorKind(fp);
    let mut i = 0;
    while *node != -1 {
        loop {
            let r = p8007506c(kind, i);
            if r == 0 || r & bits != 0 {
                break;
            }
            i += 1;
        }
        let part = ext::ftPartsRemap(fp.kind as u32, kind as u32, i as u32);
        if part as u32 != 0xFF {
            let b = bone(fp, part);
            if b1(b) && !b0(b) && !b5(b) {
                let j = pj(fp, part, do_blending);
                if b3(b) {
                    ext::lbAnim_8001E6D8(j, tree, track, *node);
                } else {
                    ext::lbAnim_8001E7E8(j, tree, track, *node);
                }
            }
        }
        track = track.offset(*node as isize);
        i += 1;
        node = node.add(1);
    }
}

/// `ftAnim_8006FE08`.
pub unsafe fn ftAnim_8006FE08(fp: &mut Fighter, do_blending: bool) {
    melee_core::ct!("ftAnim_8006FE08");
    if fp.kind as i32 != ftAnim_AuthorKind(fp) {
        ftAnim_8006FCE4(fp, do_blending);
    } else {
        let t = fp.x590;
        ftAnim_8006F4C8(fp, do_blending, t);
    }
}

/// `ftAnim_8006FE48`.
pub unsafe fn ftAnim_8006FE48(gobj: G) {
    melee_core::ct!("ftAnim_8006FE48");
    let fp = fpm(gobj);
    fp.x8AC_animSkeleton = ext::ftParts_8007482C(fp.x108_costume_joint);
    fp.x8A4_animBlendFrames = 0.0;
    fp.x8A8_anim_frame = 0.0;
    ext::ftParts_8007462C(gobj);
}

/// `ftAnim_8006FE9C`.
pub unsafe fn ftAnim_8006FE9C(fp: &mut Fighter, start: i32, t: f32, t_inv: f32) {
    melee_core::ct!("ftAnim_8006FE9C");
    for i in start..parts_num(fp) {
        let b = bone(fp, i);
        if b1(b) && !b0(b) && !b5(b) {
            if b4(b) {
                ext::lbCopyJObjSRT(b.x4_jobj2, b.joint);
            } else {
                ext::lb_8000C490(b.x4_jobj2, b.joint, b.joint, t, t_inv);
            }
        }
    }
}

/// `ftAnim_8006FF74`.
pub unsafe fn ftAnim_8006FF74(fp: &mut Fighter, start: i32) {
    melee_core::ct!("ftAnim_8006FF74");
    for i in start..parts_num(fp) {
        let b = bone(fp, i);
        if b1(b) && !b0(b) && !b5(b) {
            ext::lbCopyJObjSRT(b.x4_jobj2, b.joint);
        }
    }
}

unsafe fn blend_joint_tree(fp: &mut Fighter, start: i32, t: f32, ti: f32, mut joint: *mut HSD_Joint, second: bool) {
    let mut i = start;
    let mut depth = 0;
    while !joint.is_null() {
        while p8007506c(fp.kind as i32, i) != 0 {
            i += 1;
        }
        let b = bone(fp, i);
        if !b0(b) && !b5(b) {
            let j = if second { b.x4_jobj2 } else { b.joint };
            if b4(b) {
                ext::lb_8000B4FC(j, joint);
            } else {
                ext::lb_8000C868(joint, j, j, t, ti);
            }
        }
        i += 1;
        ftAnim_GetNextJointInTree(&mut joint, &mut depth);
    }
}

/// `ftAnim_80070010`.
pub unsafe fn ftAnim_80070010(fp: &mut Fighter, start: i32, t: f32, ti: f32, joint: *mut HSD_Joint) { melee_core::ct!("ftAnim_80070010"); blend_joint_tree(fp, start, t, ti, joint, false) }
/// `ftAnim_80070108`.
pub unsafe fn ftAnim_80070108(fp: &mut Fighter, start: i32, t: f32, ti: f32, joint: *mut HSD_Joint) { melee_core::ct!("ftAnim_80070108"); blend_joint_tree(fp, start, t, ti, joint, true) }

/// `ftAnim_80070200`: bind the costume texture animations.
pub unsafe fn ftAnim_80070200(fp: &mut Fighter, r4: *mut ftData_x8_x8, r5: *mut CostumeTObjList, r6: *mut DObjList) {
    melee_core::ct!("ftAnim_80070200");
    (*r5).n_costume_tobjs = (*r4).x8 as u32;
    if (*r5).n_costume_tobjs as usize > (*r5).costume_tobjs.len() {
        panic!("ftanim.c:1228: fighter tobj num over!");
    }
    let vis = ext::Mex_CostumeVisIdx(fp.kind as i32, fp.costume_id as i32) as u32;
    let row = *(*r4).xC.add(vis as usize);
    (*r5).x5D0 = if !row.is_null() { row } else { *(*r4).xC };
    for i in 0..(*r5).n_costume_tobjs as usize {
        (*r5).costume_tobjs[i] = ext::ftParts_80075240(r6, *(*r5).x5D0.add(i));
        if (*(*r5).costume_tobjs[i]).aobj.is_null() {
            panic!("ftanim.c:1236: can't find fighter texture anim!");
        }
        ext::HSD_AObjSetRate((*(*r5).costume_tobjs[i]).aobj, 0.0);
    }
}

/// `ftAnim_80070308`.
pub unsafe fn ftAnim_80070308(gobj: G) {
    melee_core::ct!("ftAnim_80070308");
    let fp = fpm(gobj);
    let jobj = (*gobj).hsd_obj as J;
    ext::HSD_JObjAddAnimAll(jobj, core::ptr::null_mut(), ext::costume_matanim(fp.kind as i32, fp.costume_id), core::ptr::null_mut());
    ext::HSD_JObjReqAnimAll(jobj, 0.0);
    let x8 = &mut (*(*fp.ft_data).x8).x8 as *mut _;
    let tl = &mut fp.tobj_list as *mut _;
    let dl = &mut fp.dobj_list as *mut _;
    ftAnim_80070200(fp, x8, tl, dl);
}

/// `ftAnim_80070458` (TARGET_PC: an index past the list is skipped, not asserted).
pub unsafe fn ftAnim_80070458(_fp: &Fighter, tobj_list: &mut CostumeTObjList, tobj_idx: u32, frame: f32) {
    melee_core::ct!("ftAnim_80070458");
    if tobj_idx >= tobj_list.n_costume_tobjs {
        ext::OSReport("gw: costume texture anim request skipped\n");
        return;
    }
    let t = tobj_list.costume_tobjs[tobj_idx as usize];
    ext::HSD_AObjReqAnim((*t).aobj, frame);
    ext::HSD_TObjAnim(t);
}

/// `ftAnim_800704F0`.
pub unsafe fn ftAnim_800704F0(gobj: G, tobj_idx: i32, frame: f32) {
    melee_core::ct!("ftAnim_800704F0");
    let fp = fpm(gobj);
    let tl = &mut *(&mut fp.tobj_list as *mut CostumeTObjList);
    ftAnim_80070458(fp, tl, tobj_idx as u32, frame);
    if let Some(cb) = ext::ftData_UnkCallbackPairs0_x4(fp.kind as i32) {
        cb(gobj, tobj_idx, frame);
    }
    fp.set_x221E_b7(1);
}

/// `ftAnim_800705E0`.
pub unsafe fn ftAnim_800705E0(tobj_list: &mut CostumeTObjList) {
    melee_core::ct!("ftAnim_800705E0");
    for i in 0..tobj_list.n_costume_tobjs as usize {
        ext::HSD_AObjReqAnim((*tobj_list.costume_tobjs[i]).aobj, 0.0);
        ext::HSD_TObjAnim(tobj_list.costume_tobjs[i]);
    }
}

/// `ftAnim_80070654`.
pub unsafe fn ftAnim_80070654(gobj: G) {
    melee_core::ct!("ftAnim_80070654");
    let fp = fpm(gobj);
    ftAnim_800705E0(&mut fp.tobj_list);
    if let Some(cb) = ext::ftData_UnkCallbackPairs0_x0(fp.kind as i32) {
        cb(gobj);
    }
    fp.set_x221E_b7(0);
}

/// `ftAnim_80070710`.
pub fn ftAnim_80070710(jobj: J, frame: f32) { melee_core::ct!("ftAnim_80070710"); ext::HSD_JObjReqAnimAllByFlags(jobj, 1, frame) }
/// `ftAnim_80070734`.
pub fn ftAnim_80070734(jobj: J, frame: f32) { melee_core::ct!("ftAnim_80070734"); ext::HSD_JObjReqAnimByFlags(jobj, 1, frame) }
/// `ftAnim_80070758`.
pub fn ftAnim_80070758(jobj: J) { melee_core::ct!("ftAnim_80070758"); ext::HSD_JObjRemoveAnimAllByFlags(jobj, 1) }

/// The blend weights of a part-anim slot after advancing it one frame (`ftAnim_800707B0`).
pub fn part_anim_step(s: &mut Fighter_x8B0_t) -> (f32, f32) {
    melee_core::ct!("part_anim_step");
    s.x8 += s.xC;
    if s.x4 <= s.x8 {
        s.x8 = s.x4;
        (1.0, 0.0)
    } else {
        let t = s.xC / (s.xC + (s.x4 - s.x8));
        (t, 1.0f32 - t)
    }
}

/// `ftAnim_800707B0`: blend the active part (hand) animations.
pub unsafe fn ftAnim_800707B0(gobj: G) {
    melee_core::ct!("ftAnim_800707B0");
    let fp = fpm(gobj);
    for i in 0..fp.x8B0.len() {
        if fp.x8B0[i].x11 != -1 {
            let d = &**(*fp.ft_data).x1C.add(i);
            let (t, ti) = part_anim_step(&mut fp.x8B0[i]);
            for j in 0..d.x2 as usize {
                let p = *d.x4.add(j) as i32;
                let b = bone(fp, p);
                if b5(b) {
                    if ti != 0.0 {
                        ext::lb_8000C490(b.x4_jobj2, b.joint, b.joint, t, ti);
                    } else {
                        ext::lbCopyJObjSRT(b.x4_jobj2, b.joint);
                    }
                }
            }
        }
    }
}

/// `ftAnim_80070904` (TARGET_PC: stops at the end of the fighter's parts).
pub unsafe fn ftAnim_80070904(fp: &mut Fighter, start: i32, mut animjoint: *mut HSD_AnimJoint) {
    melee_core::ct!("ftAnim_80070904");
    let mut i = start;
    let mut depth = 0;
    while !animjoint.is_null() {
        while p8007506c(fp.kind as i32, i) != 0 {
            i += 1;
        }
        if i >= parts_num(fp) {
            break;
        }
        let b = bone(fp, i);
        if !b0(b) && !(*animjoint).aobjdesc.is_null() {
            let j = b.x4_jobj2;
            ext::HSD_JObjAddAnim(j, animjoint, core::ptr::null_mut(), core::ptr::null_mut());
            ext::HSD_JObjClearFlags(j, 0x20000);
            ext::HSD_JObjReqAnimByFlags(j, 1, 0.0);
            ext::HSD_JObjAnim(j);
            b.x8.x0.set_flags_b5(1);
        }
        i += 1;
        ftAnim_GetNextAnimJointInTree(&mut animjoint, &mut depth);
    }
}

/// `ftAnim_80070A10`.
pub unsafe fn ftAnim_80070A10(ft: &mut Fighter, part: i32, tree: *mut FigaTree) { melee_core::ct!("ftAnim_80070A10"); bind_own_tree(ft, part, true, tree) }

/// `ftAnim_ApplyPartAnim` (TARGET_PC: ignored on a Geno "none" define without a template skeleton).
pub unsafe fn ftAnim_ApplyPartAnim(gobj: G, arg1: i32, arg2: i32, arg3: f32) {
    melee_core::ct!("ftAnim_ApplyPartAnim");
    let fp = fpm(gobj);
    if ext::Geno_DefineIsNone(fp.kind as i32) != 0 && ext::Geno_DefineTemplateSkeleton(fp.kind as i32) == 0 {
        ext::OSReport("geno: part animation ignored on a none define\n");
        return;
    }
    let d = *(*fp.ft_data).x1C.add(arg1 as usize);
    let s = &mut fp.x8B0[arg1 as usize];
    s.x11 = arg2 as i8;
    s.x4 = arg3;
    s.x8 = 0.0;
    s.xC = if arg3 != 0.0 { ext::lb_8000BFF0(*(*d).x8.add(arg2 as usize)) / arg3 } else { 0.0 };
    ftAnim_80070904(fp, (*d).x0 as i32, *(*d).x8.add(arg2 as usize));
}

/// `ftAnim_80070C48`.
pub unsafe fn ftAnim_80070C48(gobj: G, arg1: i32) {
    melee_core::ct!("ftAnim_80070C48");
    let x10 = fpm(gobj).x8B0[arg1 as usize].x10;
    if x10 != -1 {
        ftAnim_ApplyPartAnim(gobj, arg1, x10 as i32, 0.0);
    }
}

/// `ftAnim_80070CC4`: end a part animation and rebind the motion on those parts.
pub unsafe fn ftAnim_80070CC4(gobj: G, arg1: i32) {
    melee_core::ct!("ftAnim_80070CC4");
    let fp = fpm(gobj);
    let x11 = fp.x8B0[arg1 as usize].x11;
    if x11 == -1 {
        return;
    }
    let d = *(*fp.ft_data).x1C.add(arg1 as usize);
    // some_inline
    let mut i = (*d).x0 as i32;
    let mut depth = 0;
    let mut aj = *(*d).x8.add(x11 as u8 as usize);
    while !aj.is_null() {
        while p8007506c(fp.kind as i32, i) != 0 {
            i += 1;
        }
        if i >= parts_num(fp) {
            break;
        }
        let b = bone(fp, i);
        if b5(b) {
            b.x8.x0.set_flags_b5(0);
        }
        i += 1;
        ftAnim_GetNextAnimJointInTree(&mut aj, &mut depth);
    }
    fp.x8B0[arg1 as usize].x11 = -1;
    if !fp.x590.is_null() {
        let (t, f, s) = (fp.x590, fp.cur_anim_frame, fp.frame_speed_mul);
        ftAnim_8006EED4(fp, (*d).x0 as i32, t, f, s);
        return;
    }
    let j = bone(fp, (*d).x0 as i32).joint;
    let joint = ftAnim_8006F994(fp, j, fp.x108_costume_joint);
    ftAnim_8006FA58(fp, (*d).x0 as i32, joint);
}

/// `ftAnim_80070E74`.
pub unsafe fn ftAnim_80070E74(gobj: G) {
    melee_core::ct!("ftAnim_80070E74");
    for i in 0..5 {
        let x10 = fpm(gobj).x8B0[i].x10;
        if x10 != -1 {
            ftAnim_ApplyPartAnim(gobj, i as i32, x10 as i32, 0.0);
        }
    }
}

/// `ftAnim_80070F28`.
pub unsafe fn ftAnim_80070F28(gobj: G) {
    melee_core::ct!("ftAnim_80070F28");
    let fp = fpm(gobj);
    for i in 0..fp.x8B0.len() {
        if fp.x8B0[i].x11 != -1 {
            let d = &**(*fp.ft_data).x1C.add(i);
            for j in 0..d.x2 as usize {
                bone(fp, *d.x4.add(j) as i32).x8.x0.set_flags_b5(0);
            }
            fp.x8B0[i].x11 = -1;
        }
    }
}

/// `ftAnim_80070FD0`.
pub unsafe fn ftAnim_80070FD0(fp: &mut Fighter) -> bool {
    melee_core::ct!("ftAnim_80070FD0");
    let r = ext::ftPartGetRotX(fp, bone_index(fp, 0x35));
    if fp.x100 != r {
        fp.x100 = r;
        return true;
    }
    false
}

// ---- folded from ftanim.rs (consolidation steps 21/22) ----
/// `scale_inline` (static): `v *= 1 / scale` per component (no fused sites).
pub fn scale_inline(scale: f32, v: &mut Vec3) {
    melee_core::ct!("scale_inline");
    let f1 = 1.0f32 / scale;
    v.x *= f1;
    v.y *= f1;
    v.z *= f1;
}

/// `get_part_joint` (static).
/// # Safety
/// `f.parts` must point to at least `part + 1` bones.
pub unsafe fn get_part_joint(f: &Fighter, part: i32, do_blending: bool) -> *mut HSD_JObj {
    melee_core::ct!("get_part_joint");
    let b = &*f.parts.offset(part as isize);
    if do_blending { b.x4_jobj2 } else { b.joint }
}

/// `ftAnim_8007077C`: every x8B0 slot's x11 and x10 to -1.
pub fn ftanim_8007077c(g: G_ftanim) {
    melee_core::ct!("ftanim_8007077c");
    for s in fp(g).x8B0.iter_mut() {
        s.x11 = -1;
        s.x10 = -1;
    }
}

/// `ftAnim_80070FB4`: `x8B0[arg1].x10 = arg2` (s8 truncation).
pub fn ftanim_80070fb4(g: G_ftanim, arg1: i32, arg2: i32) {
    melee_core::ct!("ftanim_80070fb4");
    fp(g).x8B0[arg1 as usize].x10 = arg2 as i8;
}
