#![allow(non_snake_case)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_DamageIce.c (frozen by an ice attack, and the
//! DamageIceJump break-out). Checked against the C by tests/tu_ftCo_DamageIce.rs.

use crate::types::*;
use melee_core::fp::fmadds;

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}
#[inline(always)]
fn cd() -> &'static ftCommonData {
    crate::hooks::common_data()
}

/// Calls out of the TU. `hook::<name>` overrides; unset = the port, or a TODO panic.
pub mod ext {
    use super::*;
    use melee_runtime::{hsd_mtx, mtx};
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
    fn get_translate(m: &Mtx, v: &mut Vec3) {
        let t = hsd_mtx::get_translate(m);
        *v = Vec3 { x: t.x, y: t.y, z: t.z };
    }
    ext_fns! {
        ftCo_80098928(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_PassiveStand::ftCo_80098928(g as _) as _ } };
        ftCo_8009872C(g: G) -> bool = crate::tu::ftCo_DownAttack::ftCo_8009872C;
        ftCo_80097D40(g: G) = crate::tu::ftCo_DownBound::ftCo_80097D40;
        ftParts_GetBoneIndex(fp: &mut Fighter, part: i32) -> i32 = |f: &mut Fighter, p| crate::hooks::ftparts_get_bone_index(f, p);
        // `lb_8000B1CC(jobj, NULL, out)`: world position of a joint (offset pointer may be NULL).
        lb_8000B1CC(j: *mut HSD_JObj, off: *const Vec3, out: &mut Vec3) = |j: *mut HSD_JObj, off: *const Vec3, out: &mut Vec3| { unsafe { let _ = melee_runtime::tu::lb_00B0::lb_8000B1CC(j as _, off as _, out as _); } };
        lbVector_Sub(a: &mut Vec3, b: &Vec3) = |a: &mut Vec3, b: &Vec3| { a.x -= b.x; a.y -= b.y; a.z -= b.z; };
        ftCommon_UnlockECB(fp: &mut Fighter) = crate::ftcommon2::unlock_ecb;
        ft_80082638(g: G, b: *mut ftCollisionBox) -> bool = |g: G, b: *mut ftCollisionBox| -> bool { unsafe { crate::tu::ft_081b::ft_80082638(g as _, b as _) as _ } };
        ft_80082888(g: G, b: *mut ftCollisionBox) -> bool = |g: G, b: *mut ftCollisionBox| -> bool { unsafe { crate::tu::ft_081b::ft_80082888(g as _, b as _) as _ } };
        // `efAsync_Spawn(gobj, queue, kind, gfx_id, jobj, ...)`: kind 3 passes `float*`,
        // kind 5 passes `Vec3*, float*` (a1, a2 here; a2 null when unused).
        efAsync_Spawn(g: G, queue: *mut core::ffi::c_void, kind: u32, id: u32, jobj: *mut HSD_JObj, a1: *mut core::ffi::c_void, a2: *mut core::ffi::c_void) = |_, _, _, _, _, _, _| todo!("TODO(port) efAsync_Spawn (ef/efasync.c)");
        ftCo_8009750C(g: G) = |g: G| { unsafe { let _ = crate::tu::ftCo_Lift::ftCo_8009750C(g as _); } };
        ftCo_800DD168(g: G) = |g: G| { unsafe { let _ = crate::tu::ftCo_Throw::ftCo_800DD168(g as _); } };
        ftCommon_8007DB58(g: G) = |g: G| { unsafe { let _ = crate::ftcommon3::ftcommon_8007db58(g as _); } };
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { let _ = crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftCommon_InitGrab(fp: &mut Fighter, a: i32, timer: f32) = crate::ftcommon2::init_grab;
        PSMTXTrans(m: &mut Mtx, x: f32, y: f32, z: f32) = |m: &mut Mtx, x, y, z| *m = mtx::trans(x, y, z);
        HSD_MkRotationMtx(m: &mut Mtx, r: &Vec3) = |m: &mut Mtx, r: &Vec3| *m = hsd_mtx::mk_rotation_mtx(&melee_core::vec::Vec3 { x: r.x, y: r.y, z: r.z });
        HSD_MtxQuat(m: &mut Mtx, q: &Quaternion) = |m: &mut Mtx, q: &Quaternion| *m = hsd_mtx::mtx_quat(&mtx::Quaternion { x: q.x, y: q.y, z: q.z, w: q.w });
        PSMTXTranspose(s: &Mtx, d: &mut Mtx) = |s: &Mtx, d: &mut Mtx| *d = mtx::transpose(s);
        PSMTXConcat(a: &Mtx, b: &Mtx, ab: &mut Mtx) = |a: &Mtx, b: &Mtx, ab: &mut Mtx| *ab = mtx::concat(a, b);
        HSD_MtxGetTranslate(m: &Mtx, v: &mut Vec3) = get_translate;
        HSD_JObjSetMtxDirtySub(j: *mut HSD_JObj) = |_| todo!("TODO(port) HSD_JObjSetMtxDirtySub (baselib/jobj.c)");
        ftCo_8009E140(fp: &mut Fighter, b: i32) = |fp: &mut Fighter, b: i32| { unsafe { let _ = crate::tu::ftdynamics::ftCo_8009E140(fp as _, b as _); } };
        ftCommon_8007F824(g: G) = |g: G| { unsafe { let _ = crate::ftcommon3::ftcommon_8007f824(g as _); } };
        ftCommon_8007E2F4(fp: &mut Fighter, v: i16) = crate::ftcommon2::ftcommon_8007e2f4;
        HSD_Randf() -> f32 = || todo!("TODO(port) HSD_Randf (baselib/random.c)");
        ftColl_8007B0C0(g: G, state: i32) = |_, _| todo!("TODO(port) ftColl_8007B0C0 (ftcoll.c)");
        ftColl_HurtboxInit(fp: &mut Fighter, hurt: *mut FighterHurtCapsule, init: &ftHurtboxInit) = |_, _, _| todo!("TODO(port) ftColl_HurtboxInit (ftcoll.c)");
        ftCommon_8007EBAC(fp: &mut Fighter, a: u32, b: u32) = crate::ftcommon2::ftcommon_8007ebac;
        ftCo_800886D8(fp: &mut Fighter, id: i32, vol: u8, pan: u8) = |fp: &mut Fighter, id: i32, vol: u8, pan: u8| { unsafe { let _ = crate::ft0881::ftco_800886d8(fp as _, id as _, vol as _, pan as _); } };
        ftCommon_GrabMash(fp: &mut Fighter, v: f32) -> bool = crate::ftcommon2::grab_mash;
        ftCommon_CalcSelfAccel_DeaccelAir(fp: &mut Fighter) = |_| todo!("TODO(port) ftCommon_CalcSelfAccel_DeaccelAir (ftcommon.c)");
        ftCommon_Fall(fp: &mut Fighter, grav: f32, term: f32) = |_, _, _| todo!("TODO(port) ftCommon_Fall (ftcommon.c)");
        ft_80084F3C(g: G) = crate::ft084e::ft_80084f3c;
        ftCommon_8007D5D4(fp: &mut Fighter) = crate::ftcommon2::ftcommon_8007d5d4;
        ft_800824A0(g: G, b: *mut ftCollisionBox) -> bool = |g: G, b: *mut ftCollisionBox| -> bool { unsafe { crate::tu::ft_081b::ft_800824a0(g as _, b as _) as _ } };
        ftKb_SpecialN_800F1F1C(g: G, v: &mut Vec3) = |_, _| todo!("TODO(port) ftKb_SpecialN_800F1F1C (ftKb_SpecialN.c)");
        ftCommon_8007D7FC(fp: &mut Fighter) = crate::ftcommon3::ftcommon_8007d7fc;
        atan2f(y: f32, x: f32) -> f32 = melee_runtime::lbtrigf::atan2f;
        Camera_RequestQuake(kind: i32, pos: *mut Vec) = |_, _| todo!("TODO(port) Camera_RequestQuake (cm/camera.c)");
        ft_PlaySFX(fp: &mut Fighter, id: i32, vol: u8, pan: u8) = |_, _, _, _| todo!("TODO(port) ft_PlaySFX (ft_0881.c)");
        ftCo_80090780(g: G) = |g: G| { unsafe { let _ = crate::tu::ftCo_DamageFall::ftCo_80090780(g as _); } };
        lbVector_Mirror(a: &mut Vec3, n: &Vec3) = |_, _| todo!("TODO(port) lbVector_Mirror (lb/lbvector.c)");
        ft_80081D0C(g: G) -> i32 = |g: G| -> i32 { unsafe { crate::tu::ft_081b::ft_80081d0c(g as _) as _ } };
        ftCo_800C5240(g: G) -> bool = crate::hooks::ftco_800c5240;
        ftCo_800C5A98(g: G) = |g: G| { unsafe { let _ = crate::tu::ftCo_HammerJump::ftCo_800C5A98(g as _); } };
        ftCo_Fall_Enter(g: G) = crate::hooks::ftco_fall_enter;
        ft_80084EEC(g: G) = crate::ft084e::ft_80084eec;
        ftCo_80096CC8(g: G, line: i32) -> bool = |g: G, line: i32| -> bool { unsafe { crate::tu::ftCo_FallSpecial::ftCo_80096CC8(g as _, line as _) as _ } };
        ft_80082B1C(g: G) = |g: G| { unsafe { let _ = crate::tu::ft_081b::ft_80082b1c(g as _); } };
        ft_800831CC(g: G, cb1: fn(G, i32) -> bool, cb2: fn(G)) = |g: G, cb1: fn(G, i32) -> bool, cb2: fn(G)| { unsafe { let _ = crate::tu::ft_081b::ft_800831cc(g as _, cb1 as _, cb2 as _); } };
    }
}

/// static const `ftCo_DamageIce_HurtboxOffset`.
const HURTBOX_OFFSET: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
/// static const `ftCo_803B74BC`.
const FTCO_803B74BC: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

const SFX_VOLUME_MAX: u8 = 127;
const SFX_PAN_MID: u8 = 64;

/// `HSD_JObjSetMtxDirty` through the TU's out-of-line callee.
struct Dirty;
impl melee_types::inline::JObjDirty for Dirty {
    fn set_mtx_dirty(&mut self, j: &mut HSD_JObj) {
        if !melee_types::inline::HSD_JObjMtxIsDirty(j) {
            ext::HSD_JObjSetMtxDirtySub(j);
        }
    }
    fn set_mtx_dirty_sub(&mut self, j: &mut HSD_JObj) {
        ext::HSD_JObjSetMtxDirtySub(j);
    }
}

fn bone_joint(g: G, part: Fighter_Part) -> *mut HSD_JObj {
    let idx = ext::ftParts_GetBoneIndex(fp(g), part as i32);
    unsafe { (*fp(g).parts.offset(idx as isize)).joint }
}

fn fpj(f: &mut Fighter, part: Fighter_Part) -> *mut HSD_JObj {
    let idx = ext::ftParts_GetBoneIndex(f, part as i32);
    unsafe { (*f.parts.offset(idx as isize)).joint }
}

fn queue(g: G) -> *mut core::ffi::c_void {
    &mut fp(g).x60C as *mut *mut core::ffi::c_void as *mut core::ffi::c_void
}

pub fn ftCo_80090984(g: G) {
    if ext::ftCo_80098928(g) {
        return;
    }
    if ext::ftCo_8009872C(g) {
        return;
    }
    ext::ftCo_80097D40(g);
}

/// static `ftCo_800909D0`: size the ice block's collision box around the fighter.
pub fn ftCo_800909D0(f: &mut Fighter) {
    let radius = f.x34_scale.y * f.co_attrs.damageice_ice_size;
    let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut offset = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let top = unsafe { (*f.parts.offset(FtPart_TopN as isize)).joint };
    ext::lb_8000B1CC(top, core::ptr::null(), &mut pos);
    let xr = fpj(f, FtPart_XRotN);
    ext::lb_8000B1CC(xr, core::ptr::null(), &mut offset);
    ext::lbVector_Sub(&mut offset, &pos);
    let c = unsafe { &mut f.mv.co.damageice.ice_coll };
    c.top = radius + offset.y;
    c.bottom = -radius + offset.y;
    c.right.x = radius + offset.x;
    c.right.y = 0.0;
    c.left.x = -radius + offset.x;
    c.left.y = 0.0;
    ext::ftCommon_UnlockECB(f);
    let b: *mut ftCollisionBox = unsafe { &mut f.mv.co.damageice.ice_coll };
    if f.ground_or_air == GA_Air {
        ext::ft_80082638(f.gobj, b);
    } else {
        ext::ft_80082888(f.gobj, b);
    }
}

/// static inline `ftCo_SpawnEffect_x415`.
pub fn ftCo_SpawnEffect_x415(g: G, joint: *mut HSD_JObj, param: &mut f32) {
    ext::efAsync_Spawn(g, queue(g), 3, 0x415, joint, param as *mut f32 as *mut core::ffi::c_void, core::ptr::null_mut());
}

pub fn ftCo_80090AC0(f: &mut Fighter) {
    let joint = fpj(f, FtPart_XRotN);
    let size = f.co_attrs.damageice_ice_size;
    let scl_y = f.x34_scale.y * size;
    let mut param = scl_y / cd().damageice_ice_size;
    ftCo_SpawnEffect_x415(f.gobj, joint, &mut param);
    f.set_x2219_b0(1);
}

/// static `ftCo_DamageIce_OnHit`.
pub unsafe extern "C" fn ftCo_DamageIce_OnHit(g: G) {
    fp(g).set_x2227_b6(0);
}

fn rot_mtx(j: *mut HSD_JObj) -> Mtx {
    let mut rot = Quaternion { x: 0.0, y: 0.0, z: 0.0, w: 0.0 };
    melee_types::inline::HSD_JObjGetRotation(unsafe { &*j }, &mut rot);
    let mut m: Mtx = [[0.0; 4]; 3];
    if unsafe { (*j).flags } & JOBJ_USE_QUATERNION as u32 == 0 {
        // `(Vec3*) &rot`: the first three components.
        let v = Vec3 { x: rot.x, y: rot.y, z: rot.z };
        ext::HSD_MkRotationMtx(&mut m, &v);
    } else {
        ext::HSD_MtxQuat(&mut m, &rot);
    }
    m
}

/// The shared tail of `ftCo_DamageIce_Init` and `ftCo_DamageIce_HitWhileFrozen`.
fn freeze_tail(g: G) {
    let f = fp(g);
    ext::ftCo_8009E140(f, 0);
    ext::ftCommon_8007F824(fp(g).gobj);
    let f = fp(g);
    f.set_x2222_b3(1);
    if f.ground_or_air == GA_Air {
        f.self_vel = f.x8c_kb_vel;
        f.x8c_kb_vel.z = 0.0;
        f.x8c_kb_vel.y = 0.0;
        f.x8c_kb_vel.x = 0.0;
    } else {
        f.gr_vel = f.xF0_ground_kb_vel;
        f.xF0_ground_kb_vel = 0.0;
    }
    ext::ftCommon_8007E2F4(fp(g), 0x1FF);
    unsafe { fp(g).mv.co.damageice.wall_hit_dir = 0 };
    let rand = ext::HSD_Randf();
    let rot_min = cd().damageice_rot_speed_min;
    let rot_max = cd().damageice_rot_speed_max;
    let rand_range = rot_max - rot_min;
    unsafe { fp(g).mv.co.damageice.rot_speed = fmadds(rand_range, rand, rot_min) };
    ftCo_800909D0(fp(g));
    let effect_joint = fpj(fp(g), FtPart_XRotN);
    let f = fp(g);
    let ice_size = f.co_attrs.damageice_ice_size;
    let mut param = f.x34_scale.y * ice_size / cd().damageice_ice_size;
    let fg = f.gobj;
    ext::efAsync_Spawn(fg, queue(fg), 3, 0x415, effect_joint, &mut param as *mut f32 as *mut core::ffi::c_void, core::ptr::null_mut());
    fp(g).set_x2219_b0(1);
    ext::ftColl_8007B0C0(fp(g).gobj, HurtCapsule_Intangible as i32);
    let offset = HURTBOX_OFFSET;
    let bone_idx = ext::ftParts_GetBoneIndex(fp(g), FtPart_XRotN as i32);
    let f = fp(g);
    let hurt = ftHurtboxInit {
        bone_idx: bone_idx as _,
        height: HurtHeight_Mid as _,
        is_grabbable: 0,
        a_offset: offset,
        b_offset: offset,
        scale: f.co_attrs.damageice_ice_size,
    };
    let hc = f.hurt_capsules.as_mut_ptr();
    ext::ftColl_HurtboxInit(f, hc, &hurt);
    ext::ftCommon_8007EBAC(fp(g), 1, 0);
    let f = fp(g);
    f.take_dmg_2_cb = Some(ftCo_DamageIce_OnHit2);
    f.take_dmg_cb = Some(ftCo_DamageIce_OnHit);
}

pub fn ftCo_DamageIce_Init(g: G) {
    ext::ftCo_8009750C(g);
    ext::ftCo_800DD168(g);
    fp(g).set_x2227_b6(1);
    ext::Fighter_ChangeMotionState(g, ftCo_MS_DamageIce as i32, Ft_MF_Unk06 as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    let t = fp(g).dmg.x1838_percentTemp * cd().x790_damageice_unk;
    ext::ftCommon_InitGrab(fp(g), 0, t);
    let xrotn = bone_joint(g, FtPart_XRotN);
    let yrotn = bone_joint(g, FtPart_YRotN);
    let f = fp(g);
    let mut pos = Vec3 { x: 0.0, y: f.co_attrs.x154_damageice_unk, z: f.co_attrs.x150_damageice_unk };
    let mut sp_ec: Mtx = [[0.0; 4]; 3];
    ext::PSMTXTrans(&mut sp_ec, pos.x, pos.y, pos.z);
    let rot_mtx_y = rot_mtx(yrotn);
    let mut sp11c: Mtx = [[0.0; 4]; 3];
    ext::PSMTXTranspose(&rot_mtx_y, &mut sp11c);
    let rot_mtx_x = rot_mtx(xrotn);
    let mut sp14c: Mtx = [[0.0; 4]; 3];
    ext::PSMTXTranspose(&rot_mtx_x, &mut sp14c);
    let mut sp17c: Mtx = [[0.0; 4]; 3];
    ext::PSMTXConcat(&sp_ec, &sp11c, &mut sp17c);
    let a = sp17c;
    ext::PSMTXConcat(&a, &sp14c, &mut sp17c);
    ext::HSD_MtxGetTranslate(&sp17c, &mut pos);
    melee_types::inline::HSD_JObjAddTranslationY(unsafe { &mut *yrotn }, pos.y, &mut Dirty);
    melee_types::inline::HSD_JObjAddTranslationZ(unsafe { &mut *yrotn }, pos.z, &mut Dirty);
    freeze_tail(g);
    ext::ftCo_800886D8(fp(g), 0x122, 0x7F, 0x40);
}

/// static inline `ftCo_ReleaseItemAndVictim` (ftCommon/inlines.h).
fn ftCo_ReleaseItemAndVictim(g: G) {
    ext::ftCommon_8007DB58(g);
    ext::ftCo_8009750C(g);
    ext::ftCo_800DD168(g);
}

pub fn ftCo_DamageIce_HitWhileFrozen(g: G) {
    ftCo_ReleaseItemAndVictim(g);
    fp(g).set_x2227_b6(1);
    ext::Fighter_ChangeMotionState(g, ftCo_MS_DamageIce as i32, (Ft_MF_KeepGfx | Ft_MF_Unk06) as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    freeze_tail(g);
}

/// static `ftCo_DamageIce_OnHit2`: hits shorten the freeze; fire breaks it.
pub unsafe extern "C" fn ftCo_DamageIce_OnHit2(g: G) {
    let f = fp(g);
    // `grab_timer -= pct * mult`: one fmuladd(-pct, mult, grab_timer).
    f.grab_timer = fmadds(-f.dmg.x1838_percentTemp, cd().damageice_dmg_time_reduction_mult, f.grab_timer);
    if f.dmg.x1860_element == HitElement_Fire as u32 {
        f.grab_timer = 0.0;
    }
}

pub fn ftCo_DamageIce_Anim(g: G) {
    if fp(g).ground_or_air == GA_Air {
        let rot_x = unsafe { fp(g).mv.co.damageice.rot_speed };
        let jobj = bone_joint(g, FtPart_XRotN);
        melee_types::inline::HSD_JObjAddRotationX(unsafe { &mut *jobj }, rot_x, &mut Dirty);
    }
    let f = fp(g);
    if f.x2224_b2() == 0 {
        f.grab_timer -= cd().x794_damageice_unk;
    }
    ext::ftCommon_GrabMash(fp(g), cd().x798_damageice_unk);
    if fp(g).grab_timer <= 0.0 {
        ftCo_80091854(g);
    }
}

pub fn ftCo_DamageIce_IASA(_g: G) {}

pub fn ftCo_DamageIce_Phys(g: G) {
    if fp(g).ground_or_air == GA_Air {
        ext::ftCommon_CalcSelfAccel_DeaccelAir(fp(g));
        let f = fp(g);
        let grav = f.co_attrs.gravity * cd().damageice_gravity_mult;
        let term = f.co_attrs.terminal_velocity;
        ext::ftCommon_Fall(f, grav, term);
    } else {
        ext::ft_80084F3C(g);
    }
}

pub fn ftCo_DamageIce_Coll(g: G) {
    if fp(g).ground_or_air == GA_Air {
        ftCo_DamageIce_InAirUpdate(g);
    } else {
        let b: *mut ftCollisionBox = unsafe { &mut fp(g).mv.co.damageice.ice_coll };
        if !ext::ft_80082888(g, b) {
            ext::ftCommon_8007D5D4(fp(g));
        }
    }
}

pub fn ftCo_DamageIce_InAirUpdate(g: G) {
    let b: *mut ftCollisionBox = unsafe { &mut fp(g).mv.co.damageice.ice_coll };
    let ret = if fp(g).cur_anim_frame <= 3.0 { ext::ft_80082638(g, b) } else { ext::ft_800824A0(g, b) };
    let f = fp(g);
    let flags = f.coll_data.env_flags;
    let dir = unsafe { f.mv.co.damageice.wall_hit_dir };
    if flags & Collide_RightWallHug as i32 != 0 && dir != 1 {
        let mut vec = Vec3 { x: f.coll_data.ecb.left.x, y: f.coll_data.ecb.left.y, z: 0.0 };
        ext::ftKb_SpecialN_800F1F1C(g, &mut vec);
        let normal = fp(g).coll_data.right_facing_wall.normal;
        ftCo_DamageIce_Collide(g, &normal, &vec);
        unsafe { fp(g).mv.co.damageice.wall_hit_dir = 1 };
    } else if flags & Collide_LeftWallHug as i32 != 0 && dir != 2 {
        let mut vec = Vec3 { x: f.coll_data.ecb.right.x, y: f.coll_data.ecb.right.y, z: 0.0 };
        ext::ftKb_SpecialN_800F1F1C(g, &mut vec);
        let normal = fp(g).coll_data.left_facing_wall.normal;
        ftCo_DamageIce_Collide(g, &normal, &vec);
        unsafe { fp(g).mv.co.damageice.wall_hit_dir = 2 };
    } else if flags & Collide_CeilingHug as i32 != 0 && dir != 3 {
        let mut vec = Vec3 { x: 0.0, y: f.coll_data.ecb.top.y, z: 0.0 };
        ext::ftKb_SpecialN_800F1F1C(g, &mut vec);
        let normal = fp(g).coll_data.ceiling.normal;
        ftCo_DamageIce_Collide(g, &normal, &vec);
        unsafe { fp(g).mv.co.damageice.wall_hit_dir = 3 };
    } else if ret {
        ext::ftCommon_8007D7FC(fp(g));
    }
}

/// static inline `ftCo_SpawnEffect_x406`.
fn ftCo_SpawnEffect_x406(g: G, vec: &mut Vec3, mut f: f32) {
    ext::efAsync_Spawn(g, queue(g), 5, 0x406, core::ptr::null_mut(),
        vec as *mut Vec3 as *mut core::ffi::c_void, &mut f as *mut f32 as *mut core::ffi::c_void);
}

/// static inline `ftCo_Speed`: two fmuladds (x*x + y*y, then + z*z).
pub fn ftCo_Speed(f: &Fighter) -> f32 {
    let v = &f.self_vel;
    melee_runtime::gekko::sqrtf(fmadds(v.z, v.z, fmadds(v.x, v.x, v.y * v.y)))
}

/// static `ftCo_DamageIce_Collide`: the block hits a wall/ceiling: shatter or bounce.
pub fn ftCo_DamageIce_Collide(g: G, normal: &Vec3, vec: &Vec3) {
    let f = fp(g);
    let mut next_pos = Vec3 {
        x: f.cur_pos.x + vec.x,
        y: f.cur_pos.y + vec.y,
        z: f.cur_pos.z + vec.z,
    };
    let a = ext::atan2f(-normal.x, normal.y);
    ftCo_SpawnEffect_x406(g, &mut next_pos, a);
    ext::Camera_RequestQuake(QuakeKind_Small as i32, &mut next_pos);
    ext::ftCommon_8007EBAC(fp(g), 7, 0);
    ext::ft_PlaySFX(fp(g), 0x123, SFX_VOLUME_MAX, SFX_PAN_MID);
    if ftCo_Speed(fp(g)) > cd().damageice_min_speed {
        ext::ft_PlaySFX(fp(g), 0x123, SFX_VOLUME_MAX, SFX_PAN_MID);
        let f = fp(g);
        f.cur_pos = next_pos;
        f.set_x2227_b6(0);
        ext::ftCo_80090780(g);
    } else {
        ext::lbVector_Mirror(&mut fp(g).self_vel, normal);
        let f = fp(g);
        let mult = cd().damageice_speed_mult_on_break;
        f.self_vel.x *= mult;
        f.self_vel.y *= mult;
        if f.coll_data.env_flags & Collide_RightWallHug as i32 != 0 {
            // `-((transN.z * -facing) - (cur.x + vec.x))`: one fmuladd, then negated.
            f.cur_pos.x = -fmadds(f.x68C_transNPos.z, -f.facing_dir, -(f.cur_pos.x + vec.x));
        } else {
            f.cur_pos.y = f.cur_pos.y + vec.y + f.x68C_transNPos.y;
        }
        if ext::ft_80081D0C(g) != 0 {
            ext::ftCommon_8007D7FC(fp(g));
        }
    }
}

pub fn ftCo_80091854(g: G) {
    if ext::ftCo_800C5240(g) {
        ext::ftCo_800C5A98(g);
        return;
    }
    ext::ftCommon_8007D5D4(fp(g));
    fp(g).set_x2227_b6(0);
    ext::Fighter_ChangeMotionState(g, ftCo_MS_DamageIceJump as i32, Ft_MF_Unk06 as u32, 0.0, 0.0, 0.0, core::ptr::null_mut());
    ext::ft_PlaySFX(fp(g), 0x123, SFX_VOLUME_MAX, SFX_PAN_MID);
    let jobj = bone_joint(g, FtPart_XRotN);
    let mut vec = FTCO_803B74BC;
    let f = fp(g);
    let mut sp24 = f.x34_scale.y * f.co_attrs.damageice_ice_size / cd().damageice_ice_size;
    ext::efAsync_Spawn(g, queue(g), 5, 1091, jobj,
        &mut vec as *mut Vec3 as *mut core::ffi::c_void, &mut sp24 as *mut f32 as *mut core::ffi::c_void);
    let f = fp(g);
    f.self_vel.x = f.input.lstick[0].x * f.co_attrs.damageicejump_vel_x_mult;
    f.self_vel.y = f.co_attrs.damageicejump_vel_y;
    unsafe { f.mv.co.damageicejump.escape_timer = cd().damageicejump_escape_time };
}

pub fn ftCo_DamageIceJump_Anim(g: G) {
    let f = fp(g);
    unsafe {
        if f.mv.co.damageicejump.escape_timer > 0.0 {
            f.mv.co.damageicejump.escape_timer -= 1.0;
            if f.mv.co.damageicejump.escape_timer <= 0.0 {
                ext::ftCo_Fall_Enter(g);
            }
        }
    }
}

pub fn ftCo_DamageIceJump_IASA(_g: G) {}

pub fn ftCo_DamageIceJump_Phys(g: G) {
    ext::ft_80084EEC(g);
}

pub fn ftCo_DamageIceJump_Coll(g: G) {
    ext::ft_800831CC(g, ext::ftCo_80096CC8, ext::ft_80082B1C);
}
