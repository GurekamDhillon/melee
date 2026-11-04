#![allow(non_snake_case, non_upper_case_globals, unused_unsafe)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_YoshiEgg.c (trapped in Yoshi's egg: the
//! entry, the hatch-scale tween on accessory4, the mash and the hit-shortens-timer callback).
//! Every function of the TU; calls out go through `ext`. One fused site (ftCo_800BC3D0's
//! `pct * k - grab_timer`), checked by tests/tu_ftCo_YoshiEgg.rs against the C (whole TU,
//! shipping contraction).

use crate::types::*;
use melee_core::fp::fmadds;
use melee_types::inline as ji;

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

/// Calls out of the TU. `hook::<name>` overrides; unset = the port, or a TODO panic.
pub mod ext {
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_fns {
        ($($name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? = $def:expr;)*) => {
            pub mod hook {
                #[allow(unused_imports)] use super::*;
                $( #[allow(non_upper_case_globals)] pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( #[allow(non_snake_case)] pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } { Some(f) => f($($a),*), None => ($def)($($a),*) }
            } )*
        };
    }
    ext_fns! {
        ftColl_8007B62C(g: G, v: i32) = |g: G, v: i32| { unsafe { crate::tu::ftcoll::ftcoll_8007b62c(g as _, v as _); } };
        Fighter_UpdateModelScale(g: G) = |g: G| { unsafe { crate::tu::fighter::Fighter_UpdateModelScale(g as _); } };
        HSD_JObjSetMtxDirtySub(j: *mut HSD_JObj) = |a0| unsafe { melee_runtime::tu::jobj_ptr::HSD_JObjSetMtxDirtySub(a0 as _) };
        ftColl_8007B0C0(g: G, s: i32) = |g: G, s: i32| { unsafe { crate::tu::ftcoll::ftcoll_8007b0c0(g as _, s as _); } };
        ftParts_GetBoneIndex(f: &Fighter, part: i32) -> i32 = crate::ftdmgfall::ftparts_get_bone_index;
        ftColl_HurtboxInit(f: &mut Fighter, c: *mut FighterHurtCapsule, h: &ftHurtboxInit) = |f: &mut Fighter, c: *mut FighterHurtCapsule, h: &ftHurtboxInit| { unsafe { crate::tu::ftcoll::ftcoll_hurtbox_init(f, c as _, h); } };
        ftCommon_8007D5D4(f: &mut Fighter) = crate::ftcommon2::ftcommon_8007d5d4;
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftCo_800BC438(g: G) = crate::tu::ftCo_CaptureKoopa::ftCo_800BC438;
        ftCommon_8007E2F4(f: &mut Fighter, v: i16) = crate::ftcommon2::ftcommon_8007e2f4;
        ftCommon_SetAccessory(f: &mut Fighter, j: *mut HSD_Joint) = |_, _| todo!("ftCommon_SetAccessory: installed by melee_link");
        ftYs_SpecialN_8012CDD4(g: G) -> *mut HSD_Joint = |_| todo!("TODO(port) ftYs_SpecialN_8012CDD4 (ftyoshispecialn.c)");
        lb_8000C2F8(a: *mut HSD_JObj, b: *mut HSD_JObj) = |a: *mut HSD_JObj, b: *mut HSD_JObj| { unsafe { melee_runtime::tu::lb_00B0::lb_8000C2F8(a as _, b as _); } };
        ftYs_SpecialN_SetupItemVel(g: G, out: &mut Vec3) = |_, _| todo!("ftYs_SpecialN_SetupItemVel: installed by melee_link");
        ftYs_SpecialN_GetFacingDir(g: G) -> f32 = |_| todo!("ftYs_SpecialN_GetFacingDir: installed by melee_link");
        ftYs_SpecialN_GetDatAttr18(g: G) -> f32 = |_| todo!("ftYs_SpecialN_GetDatAttr18: installed by melee_link");
        ftYs_SpecialN_GetDatAttr1C(g: G) -> f32 = |_| todo!("ftYs_SpecialN_GetDatAttr1C: installed by melee_link");
        ftYs_SpecialN_GetDatAttr20(g: G) -> f32 = |_| todo!("ftYs_SpecialN_GetDatAttr20: installed by melee_link");
        ftYs_SpecialN_GetDatAttr24(g: G) -> f32 = |_| todo!("ftYs_SpecialN_GetDatAttr24: installed by melee_link");
        ftCommon_8007EFC0(f: &mut Fighter, v: u32) = crate::ftcommon2::ftcommon_8007efc0;
        ftCommon_InitGrab(f: &mut Fighter, a: i32, timer: f32) = crate::ftcommon2::init_grab;
        ftYs_SpecialN_GetExtAttr28() -> f32 = || todo!("ftYs_SpecialN_GetExtAttr28: installed by melee_link");
        ftYs_SpecialN_GetExtAttr2C() -> f32 = || todo!("ftYs_SpecialN_GetExtAttr2C: installed by melee_link");
        ftYs_SpecialN_8012CD40() -> f32 = || todo!("ftYs_SpecialN_8012CD40: installed by melee_link");
        ftYs_SpecialN_GetExtAttr34() -> f32 = || todo!("ftYs_SpecialN_GetExtAttr34: installed by melee_link");
        ftYs_SpecialN_8012CDB4() -> f32 = || todo!("ftYs_SpecialN_8012CDB4: installed by melee_link");
        ftYs_SpecialN_GetExtAttr38() -> bool = || todo!("ftYs_SpecialN_GetExtAttr38: installed by melee_link");
        ftCommon_GrabMash(f: &mut Fighter, v: f32) -> bool = crate::ftcommon2::grab_mash;
        ft_PlaySFX(f: &mut Fighter, id: i32, vol: u8, pan: u8) = crate::ft0877::play_sfx;
        efAsync_Spawn(g: G, q: *mut c_void, kind: u32, gfx: u32, j: *mut HSD_JObj, arg: *const c_void) = |a0, a1, a2, a3, a4, a5| unsafe { let _ = melee_world::tu::efn::efasync::efAsync_Spawn(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, &[a5 as *const core::ffi::c_void]); };
        ftYs_SpecialN_8012CD88(v: &mut Vec3) = |_| todo!("TODO(port) ftYs_SpecialN_8012CD88 (ftyoshispecialn.c)");
        ftColl_8007B760(g: G, v: i32) = |g: G, v: i32| { unsafe { crate::tu::ftcoll::ftcoll_8007b760(g as _, v as _); } };
        ftCo_Fall_Enter_YoshiEgg(g: G) = crate::tu::ftCo_Fall::ftCo_Fall_Enter_YoshiEgg;
        ftAnim_SetAnimRate(g: G, r: f32) = |g, r| unsafe { crate::tu::ftanim::ftAnim_SetAnimRate(g, r) };
        ft_80084F3C(g: G) = crate::ft084e::ft_80084f3c;
        ft_80084DB0(g: G) = crate::ft084e::ft_80084db0;
        ft_8008403C(g: G, cb: fn(G)) = crate::tu::ft_081b::ft_8008403c;
        ft_80082C74(g: G, cb: fn(G)) = crate::tu::ft_081b::ft_80082c74;
        ftCommon_8007D7FC(f: &mut Fighter) = crate::ftcommon3::ftcommon_8007d7fc;
        ftCo_800C0C88(t: i32) -> bool = crate::tu::ftCo_Bury::ftCo_800C0C88;
    }
}

struct Dirty;
impl ji::JObjDirty for Dirty {
    fn set_mtx_dirty(&mut self, j: &mut HSD_JObj) {
        if !ji::HSD_JObjMtxIsDirty(j) {
            ext::HSD_JObjSetMtxDirtySub(j);
        }
    }
    fn set_mtx_dirty_sub(&mut self, j: &mut HSD_JObj) {
        ext::HSD_JObjSetMtxDirtySub(j);
    }
}

/// `HSD_JObjSetScale` (jobj.h inline; the NULL assert is debug-only on PC).
fn set_scale(j: *mut HSD_JObj, s: &Vec3) {
    ji::HSD_JObjSetScale(unsafe { &mut *j }, s, &mut Dirty);
}

/// C callback-pointer forms (`HSD_GObjEvent`) of the functions stored in Fighter.
pub unsafe extern "C" fn ftCo_800BC438_cb(g: G) { melee_core::ct!("ftCo_800BC438_cb"); ext::ftCo_800BC438(g) }
pub unsafe extern "C" fn ftCo_800BBCC0_cb(g: G) { melee_core::ct!("ftCo_800BBCC0_cb"); ftCo_800BBCC0(g) }
pub unsafe extern "C" fn ftCo_800BC3D0_cb(g: G) { melee_core::ct!("ftCo_800BC3D0_cb"); ftCo_800BC3D0(g) }

/// `ftCo_800BBC88`.
pub fn ftCo_800BBC88(g: G) {
    melee_core::ct!("ftCo_800BBC88");
    unsafe {
    fp(g).set_invisible(1);
    ext::ftColl_8007B62C(g, 2);
    }
}

/// `ftCo_800BBCC0`: accessory4 tween of the fighter and egg scale back to normal.
pub fn ftCo_800BBCC0(g: G) {
    melee_core::ct!("ftCo_800BBCC0");
    unsafe {
    let jobj1 = unsafe { (*g).hsd_obj as *mut HSD_JObj };
    let f = fp(g);
    if f.mv.co.yoshiegg.x14 <= 0.0 {
        ext::Fighter_UpdateModelScale(g);
        fp(g).accessory4_cb = None;
        return;
    }
    f.mv.co.yoshiegg.x14 -= 1.0;
    let ye = f.mv.co.yoshiegg;
    let mut temp0 = (ye.x10 - ye.x14) / ye.x10;
    temp0 *= ye.xC;
    let temp1 = (1.0 - ye.xC) + temp0;
    let mut vec1 = Vec3 { x: temp1, y: temp1, z: temp1 };
    let mut vec0 = vec1;
    vec1.x *= ye.x18.x;
    vec1.y *= ye.x18.y;
    vec1.z *= ye.x18.z;
    set_scale(jobj1, &vec1);
    let f = fp(g);
    let ye = f.mv.co.yoshiegg;
    vec0.x *= ye.scale.x;
    vec0.y *= ye.scale.y;
    vec0.z *= ye.scale.z;
    set_scale(f.x20A0_accessory, &vec0);
    }
}

/// `inlineA0`: intangible, then one hurtbox from the egg attributes.
fn inlineA0(g: G) {
    unsafe {
    ext::ftColl_8007B0C0(g, HurtCapsule_Intangible as i32);
    let f = fp(g);
    let a = f.co_attrs.xBC;
    let mut hurt = ftHurtboxInit::zeroed();
    hurt.bone_idx = ext::ftParts_GetBoneIndex(f, FtPart_TransN as i32) as _;
    hurt.height = HurtHeight_Mid;
    hurt.is_grabbable = 0;
    hurt.a_offset = a.x4;
    hurt.b_offset = a.x10;
    hurt.scale = a.x1C;
    let caps = f.hurt_capsules.as_mut_ptr();
    ext::ftColl_HurtboxInit(f, caps, &hurt);
    }
}

/// `ftCo_800BBED4`: enter the egg laid by `arg1` (Yoshi).
pub fn ftCo_800BBED4(g: G, arg1: G) {
    melee_core::ct!("ftCo_800BBED4");
    unsafe {
    let jobj = unsafe { (*g).hsd_obj as *mut HSD_JObj };
    if fp(g).ground_or_air == GA_Ground {
        ext::ftCommon_8007D5D4(fp(g));
    }
    ext::Fighter_ChangeMotionState(g, ftCo_MS_YoshiEgg as i32, Ft_MF_Unk06 as u32, 0.0, 1.0, 0.0, arg1);
    let f = fp(g);
    f.take_dmg_cb = Some(ftCo_800BC438_cb);
    ext::ftCommon_8007E2F4(f, 0x1FF);
    let f = fp(g);
    f.set_invisible(1);
    let s = f.co_attrs.xBC.size;
    let scale = Vec3 { x: s, y: s, z: s };
    let j = ext::ftYs_SpecialN_8012CDD4(arg1);
    ext::ftCommon_SetAccessory(fp(g), j);
    let f = fp(g);
    set_scale(f.x20A0_accessory, &scale);
    f.mv.co.yoshiegg.scale = scale;
    let child = ji::HSD_JObjGetChild(unsafe { f.x20A0_accessory.as_ref() });
    let idx = ext::ftParts_GetBoneIndex(f, FtPart_TransN as i32);
    let joint = unsafe { (*f.parts.offset(idx as isize)).joint as *mut HSD_JObj };
    ext::lb_8000C2F8(child, joint);
    inlineA0(g);
    ext::ftYs_SpecialN_SetupItemVel(arg1, &mut fp(g).self_vel);
    fp(g).facing_dir = ext::ftYs_SpecialN_GetFacingDir(arg1);
    fp(g).dmg.x182c_behavior = ext::ftYs_SpecialN_GetDatAttr18(arg1);
    let f = fp(g);
    f.mv.ca.specialhi.vel.y = 0.0;
    f.mv.co.yoshiegg.x0 = arg1;
    f.take_dmg_2_cb = Some(ftCo_800BC3D0_cb);
    ext::ftCommon_8007EFC0(f, 1);
    fp(g).mv.co.yoshiegg.x10 = ext::ftYs_SpecialN_GetDatAttr20(arg1);
    let f = fp(g);
    f.mv.co.yoshiegg.x14 = f.mv.co.yoshiegg.x10; // C reads walk.fast_anim_frame, the same GC offset (0x2350) as yoshiegg.x10
    fp(g).mv.co.yoshiegg.xC = ext::ftYs_SpecialN_GetDatAttr1C(arg1);
    let t = ext::ftYs_SpecialN_GetDatAttr24(arg1);
    ext::ftCommon_InitGrab(fp(g), 0, t);
    let f = fp(g);
    ji::HSD_JObjGetScale(unsafe { &*jobj }, &mut f.mv.co.yoshiegg.x18);
    f.accessory4_cb = Some(ftCo_800BBCC0_cb);
    }
}

/// `ftCo_YoshiEgg_Anim`: mash out; hatch when the timer runs out.
pub fn ftCo_YoshiEgg_Anim(g: G) {
    melee_core::ct!("ftCo_YoshiEgg_Anim");
    unsafe {
    let d = ext::ftYs_SpecialN_GetExtAttr28();
    fp(g).grab_timer -= d;
    let v = ext::ftYs_SpecialN_GetExtAttr2C();
    let m = ext::ftCommon_GrabMash(fp(g), v);
    fp(g).mv.ca.specialhi.vel.x = f32::from_bits(m as u32); // yoshiegg.x4 is the word specialhi.vel.x occupies (GC 0x2344)
    if fp(g).grab_timer <= 0.0 {
        ext::ft_PlaySFX(fp(g), 280088, 127, 64);
        let f = fp(g);
        let jobj = unsafe { (*f.parts.offset(FtPart_TopN as isize)).joint as *mut HSD_JObj };
        let q = &mut f.x60C as *mut _ as *mut core::ffi::c_void;
        let a = &f.co_attrs.xBC as *const ftCo_DatAttrs_xBC_t as *const core::ffi::c_void;
        ext::efAsync_Spawn(g, q, 4, 1231, jobj, a);
        ext::ftYs_SpecialN_8012CD88(&mut fp(g).self_vel);
        ext::ftCommon_8007D5D4(fp(g));
        ext::Fighter_UpdateModelScale(g);
        let b = ext::ftYs_SpecialN_GetExtAttr38();
        ext::ftColl_8007B760(g, b as i32);
        ext::ftCo_Fall_Enter_YoshiEgg(g);
        return;
    }
    let f = fp(g);
    if f.mv.ca.specialhi.vel.y != 0.0 {
        f.mv.ca.specialhi.vel.y -= 1.0;
        if f.mv.ca.specialhi.vel.y <= 0.0 && f.mv.ca.specialhi.vel.x.to_bits() == 0 {
            ext::ftAnim_SetAnimRate(g, 1.0);
            fp(g).mv.ca.specialhi.vel.y = 0.0;
        }
    }
    let f = fp(g);
    if f.mv.ca.specialhi.vel.y <= 0.0 && f.mv.ca.specialhi.vel.x.to_bits() != 0 {
        fp(g).mv.ca.specialhi.vel.y = ext::ftYs_SpecialN_8012CD40();
        let r = ext::ftYs_SpecialN_GetExtAttr34();
        ext::ftAnim_SetAnimRate(g, r);
    }
    }
}

/// `ftCo_YoshiEgg_IASA`.
pub fn ftCo_YoshiEgg_IASA(_g: G) { melee_core::ct!("ftCo_YoshiEgg_IASA");}

/// `ftCo_YoshiEgg_Phys`.
pub fn ftCo_YoshiEgg_Phys(g: G) {
    melee_core::ct!("ftCo_YoshiEgg_Phys");
    unsafe {
    if fp(g).ground_or_air == GA_Ground {
        ext::ft_80084F3C(g);
    } else {
        ext::ft_80084DB0(g);
    }
    }
}

/// `ftCo_YoshiEgg_Coll`.
pub fn ftCo_YoshiEgg_Coll(g: G) {
    melee_core::ct!("ftCo_YoshiEgg_Coll");
    unsafe {
    if fp(g).ground_or_air == GA_Ground {
        ext::ft_8008403C(g, ftCo_800BC3AC);
    } else {
        ext::ft_80082C74(g, ftCo_800BC388);
    }
    }
}

/// `ftCo_800BC388`.
pub fn ftCo_800BC388(g: G) {
    melee_core::ct!("ftCo_800BC388");
    unsafe {
    ext::ftCommon_8007D7FC(fp(g));
    }
}

/// `ftCo_800BC3AC`.
pub fn ftCo_800BC3AC(g: G) {
    melee_core::ct!("ftCo_800BC3AC");
    unsafe {
    ext::ftCommon_8007D5D4(fp(g));
    }
}

/// `ftCo_800BC3D0`: a hit takes `percent * k` off the egg timer (fused: fmadds).
pub fn ftCo_800BC3D0(g: G) {
    melee_core::ct!("ftCo_800BC3D0");
    unsafe {
    let k = ext::ftYs_SpecialN_8012CDB4();
    let f = fp(g);
    f.grab_timer = -fmadds(f.dmg.x1838_percentTemp, k, -f.grab_timer);
    if f.dmg.x18CC == 3 && ext::ftCo_800C0C88(f.dmg.x18D0) {
        fp(g).grab_timer = 0.0;
    }
    fp(g).x1828 = 4;
    }
}
