//! Native port of src/melee/ft/ftbosslib.c (Master Hand / Crazy Hand shared helpers).
//!
//! Fused sites (`tools/rust-port/fma_sites.py`): only the static inline `my_lbvector_Len`
//! (two f32 fmuladd, `x*x + y*y + z*z` -> `fmadds(z, z, fmadds(x, x, y*y))`) and the
//! math_ppc.h sqrtf (ported in melee_runtime::gekko). `ftBossLib_8015BE40` is checked bit for bit
//! against the C by tests/tu_ftbosslib.rs; the rest walk game state and are smoke-tested.
#![allow(non_snake_case)]

use crate::ftcommon::abs_c;
use crate::hooks as h;
use crate::types::*;
use melee_core::fp::fmadds;
use melee_runtime::gekko::sqrtf;

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

/// Calls into TUs that are not ported yet. Each panics until a later pass rewires it.
pub mod ext {
    use super::G;
    use crate::types::*;
    pub fn Player_GetPlayerSlotType(_slot: i32) -> Gm_PKind { unsafe { crate::tu::player::Player_GetPlayerSlotType(_slot as _) } }
    pub fn HSD_JObjSetRotation(_j: *mut HSD_JObj, _q: &Quaternion) { unsafe { let _ = melee_world::tu::grinl::HSD_JObjSetRotation(_j as _, _q as *const _ as *mut _); } }
    pub fn mpFloorGetRight(_line: i32, _out: &mut Vec3) { todo!("mpFloorGetRight (mp/mplib.c)") }
    pub fn mpFloorGetLeft(_line: i32, _out: &mut Vec3) { todo!("mpFloorGetLeft (mp/mplib.c)") }
    pub fn it_8026C3FC() { todo!("it_8026C3FC (it/it_26B1.c)") }
    pub fn it_8026C42C() { todo!("it_8026C42C (it/it_26B1.c)") }
    pub fn Camera_8002E818(_v: &Vec3) { todo!("Camera_8002E818 (cm/camera.c)") }
    pub fn Camera_8002EA64(_v: &Vec3) { todo!("Camera_8002EA64 (cm/camera.c)") }
    pub fn Camera_8002F0E4(_n: i32) { todo!("Camera_8002F0E4 (cm/camera.c)") }
    pub fn Camera_SetModeToStandard() { todo!("Camera_SetModeToStandard (cm/camera.c)") }
    #[allow(dead_code)]
    pub fn _unused(_g: G) {}
}

/// `__assert` on a failed HSD_ASSERTREPORT.
fn assert_report(line: i32, cond: bool, msg: &str) {
    if !cond {
        panic!("ftbosslib.c:{line}: {msg}");
    }
}

/// `ftBossLib_8015BD20`: no-op.
pub fn ftBossLib_8015BD20(_g: G) {}

/// `ftBossLib_8015BD24`: `*arg1 = (arg3/arg0 + HSD_Randi(arg4-arg5) + arg5) / arg2` (int sum,
/// then int->float, then f32 divide).
pub fn ftBossLib_8015BD24(arg0: i32, arg1: &mut f32, arg2: f32, arg3: i32, arg4: i32, arg5: i32) {
    let q = melee_runtime::cabi::sdiv(arg3, arg0);
    let r = h::hsd_randi(arg4.wrapping_sub(arg5));
    let n = q.wrapping_add(r).wrapping_add(arg5);
    *arg1 = n as f32 / arg2;
}

/// `ftBossLib_ReportGObjSlotType`.
pub fn ftBossLib_ReportGObjSlotType(g: G) {
    let kind = ext::Player_GetPlayerSlotType(fp(g).player_id as i32);
    assert_report(103, kind == Gm_PKind_Human || kind == Gm_PKind_Boss || kind == Gm_PKind_Cpu,
                  "boss is human or boss!\n");
}

/// `my_lbvector_Len` (static inline).
#[inline]
fn my_lbvector_len(v: &Vec3) -> f32 {
    sqrtf(fmadds(v.z, v.z, fmadds(v.x, v.x, v.y * v.y)))
}

/// `lbVector_Normalize` (lb/lbvector.c; melee_runtime::lbvector, verified there).
fn lb_normalize(v: &mut Vec3) {
    let mut w = melee_core::vec::Vec3 { x: v.x, y: v.y, z: v.z };
    melee_runtime::lbvector::normalize(&mut w);
    *v = Vec3 { x: w.x, y: w.y, z: w.z };
}

/// The arithmetic of `ftBossLib_8015BE40` on plain values: returns (self_vel.x, self_vel.y, *arg2).
pub fn boss_8015be40_math(target: &Vec3, cur: &Vec3, arg3: f32, arg4: f32) -> (f32, f32, f32) {
    let mut diff = Vec3 { x: target.x - cur.x, y: target.y - cur.y, z: target.z - cur.z };
    let distance = my_lbvector_len(&diff);
    let out;
    if distance < arg3 {
        out = 0.0;
    } else {
        out = distance;
        lb_normalize(&mut diff);
        diff.x *= distance * arg4;
        diff.y *= distance * arg4;
        diff.z *= distance * arg4;
    }
    (diff.x, diff.y, out)
}

/// `ftBossLib_8015BE40`.
pub fn ftBossLib_8015BE40(g: G, arg1: &Vec3, arg2: &mut f32, arg3: f32, arg4: f32) {
    let f = fp(g);
    let (x, y, o) = boss_8015be40_math(arg1, &f.cur_pos, arg3, arg4);
    *arg2 = o;
    f.self_vel.x = x;
    f.self_vel.y = y;
}

/// The clamp shared by 8015BF74 / 8015C010.
#[inline]
fn clamp_diff(x_diff: f32, max: f32) -> f32 {
    if abs_c(x_diff) > max {
        if x_diff > 0.0 { max } else { -max }
    } else {
        x_diff
    }
}

/// `ftBossLib_8015BF74`: `self_vel.x += clamp(target.x - cur.x)`.
pub fn ftBossLib_8015BF74(g: G, x_diff_max: f32) {
    let mut v = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    ftBossLib_8015C208(g, &mut v);
    let f = fp(g);
    f.self_vel.x += clamp_diff(v.x - f.cur_pos.x, x_diff_max);
}

/// `ftBossLib_8015C010`: `self_vel.x = clamp(target.x - cur.x)`.
pub fn ftBossLib_8015C010(g: G, x_diff_max: f32) {
    let mut v = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    ftBossLib_8015C208(g, &mut v);
    let f = fp(g);
    f.self_vel.x = clamp_diff(v.x - f.cur_pos.x, x_diff_max);
}

/// `ftBossLib_8015C09C`: set facing and the root's Y rotation (`M_PI_2 * dir`, in double).
pub fn ftBossLib_8015C09C(g: G, facing_dir: f32) {
    let jobj = unsafe { (*g).hsd_obj as *mut HSD_JObj };
    let f = fp(g);
    f.facing_dir = facing_dir;
    let mut q: Quaternion = unsafe { core::mem::zeroed() };
    q.y = (core::f64::consts::FRAC_PI_2 * f.facing_dir as f64) as f32;
    ext::HSD_JObjSetRotation(jobj, &q);
}

/// `ftBossLib_8015C190`: clamp x to floor line 0's ends.
pub fn ftBossLib_8015C190(g: G) {
    let f = fp(g);
    let mut v = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    ext::mpFloorGetRight(0, &mut v);
    if f.cur_pos.x > v.x {
        f.cur_pos.x = v.x;
        f.self_vel.x = 0.0;
    }
    ext::mpFloorGetLeft(0, &mut v);
    if f.cur_pos.x < v.x {
        f.cur_pos.x = v.x;
        f.self_vel.x = 0.0;
    }
}

/// `ftBossLib_8015C208`: position of the nearest other fighter.
pub fn ftBossLib_8015C208(arg0: G, arg1: &mut Vec3) {
    let pos = fp(arg0).cur_pos;
    let g = ftBossLib_8015C244(arg0, &pos);
    crate::ftlib::ftlib_80086644(g, arg1);
}

/// `ftBossLib_8015C244`.
pub fn ftBossLib_8015C244(arg0: G, arg1: &Vec3) -> G {
    crate::ftlib::ftlib_8008627c(arg1, arg0)
}

pub fn ftBossLib_IsMasterHandEntry() -> bool {
    ftBossLib_GetMotionId(Ft_Kind_MasterH) == ftMh_MS_Entry as i32
}
pub fn ftBossLib_8015C2A8() -> bool {
    ftBossLib_GetMotionId(Ft_Kind_CrezyH) == 0x183
}
pub fn ftBossLib_8015C2E0() -> bool {
    let m = ftBossLib_GetMotionId(Ft_Kind_MasterH);
    m == 0x158 || m == 0x159
}
pub fn ftBossLib_8015C31C() -> bool {
    let m = ftBossLib_GetMotionId(Ft_Kind_CrezyH);
    m == 0x181 || m == 0x182
}
pub fn ftBossLib_8015C358() -> bool {
    let g = ftBossLib_GetFighterGObj(Ft_Kind_MasterH);
    !g.is_null() && fp(g).x221F_b3() != 0
}
pub fn ftBossLib_8015C3A0() -> bool {
    let g = ftBossLib_GetFighterGObj(Ft_Kind_CrezyH);
    !g.is_null() && fp(g).x221F_b3() != 0
}

/// `ftBossLib_GetFighterGObj`: first fighter of `kind` on the fighter p-link.
pub fn ftBossLib_GetFighterGObj(kind: FighterKind) -> G {
    let mut cur = h::plink_head(HSD_GOBJ_PLINK_FIGHTER);
    while !cur.is_null() {
        if kind == h::ftlib_get_kind(cur) {
            return cur;
        }
        cur = unsafe { (*cur).next };
    }
    core::ptr::null_mut()
}

pub fn ftBossLib_GetMotionId(kind: FighterKind) -> i32 {
    let g = ftBossLib_GetFighterGObj(kind);
    if !g.is_null() { crate::ftlib::get_motion_id(g) } else { ftCo_MS_DeadDown as i32 }
}

pub fn ftBossLib_8015C4C4() -> i32 {
    let g = ftBossLib_GetFighterGObj(Ft_Kind_CrezyH);
    if !g.is_null() { unsafe { fp(g).u.mh.x2250 } } else { 0 }
}

pub fn ftBossLib_8015C530(cpu_level: i32) -> i32 {
    let g = ftBossLib_GetFighterGObj(Ft_Kind_MasterH);
    if g.is_null() {
        return 0;
    }
    let a = unsafe { &*((*fp(g).ft_data).ext_attr as *const ftMasterHand_SpecialAttrs) };
    match cpu_level {
        0 => a.x0,
        1 => a.x4,
        2 => a.x8,
        3 => a.xC,
        4 => a.x10,
        _ => a.x14,
    }
}

pub fn ftBossLib_8015C5F8(g: G) {
    let f = fp(g);
    match h::hsd_randi(4) {
        0 => h::play_sfx(f, 0x4E21A, 0x7F, 0x40),
        1 => h::play_sfx(f, 0x4E21B, 0x7F, 0x40),
        2 => h::play_sfx(f, 0x4E21C, 0x7F, 0x40),
        3 => h::play_sfx(f, 0x4E21D, 0x7F, 0x40),
        _ => {}
    }
}

pub fn ftBossLib_8015C6BC() -> *mut ftMasterHand_SpecialAttrs {
    let g = ftBossLib_GetFighterGObj(Ft_Kind_MasterH);
    if g.is_null() {
        return core::ptr::null_mut();
    }
    let f = unsafe { (*g).user_data as *mut Fighter };
    if f.is_null() {
        return core::ptr::null_mut();
    }
    unsafe { (*(*f).ft_data).ext_attr as *mut ftMasterHand_SpecialAttrs }
}

macro_rules! attr_getter {
    ($($name:ident => $field:ident;)*) => {$(
        pub fn $name() -> i32 {
            let da = ftBossLib_8015C6BC();
            if da.is_null() { -1 } else { unsafe { (*da).$field } }
        }
    )*};
}
attr_getter! {
    ftBossLib_8015C74C => x164;
    ftBossLib_8015C7EC => x168;
    ftBossLib_8015C88C => x16C;
    ftBossLib_8015C92C => x170;
    ftBossLib_8015C9CC => x174;
}

pub fn ftBossLib_8015CA6C(arg0: i32) {
    crate::player::player_80036790(0, arg0 as f32);
    let mh = ftBossLib_GetFighterGObj(Ft_Kind_MasterH);
    if !mh.is_null() {
        h::ftlib_80086a4c(mh, arg0 as f32);
    }
    let ch = ftBossLib_GetFighterGObj(Ft_Kind_CrezyH);
    if !ch.is_null() {
        h::ftlib_80086a4c(ch, arg0 as f32);
    }
    ext::it_8026C3FC();
}

pub fn ftBossLib_8015CB7C() {
    ext::it_8026C42C();
}

pub fn ftBossLib_8015CB9C(arg0: i32) {
    let vec = crate::player::load_player_coords(arg0);
    ext::Camera_8002E818(&vec);
    let mut spc = vec;
    let da = ftBossLib_8015C6BC();
    spc.z += if da.is_null() { -1.0 } else { unsafe { (*da).x178 } };
    ext::Camera_8002EA64(&spc);
    ext::Camera_8002F0E4(0xA);
}

pub fn ftBossLib_8015CC14() {
    ext::Camera_SetModeToStandard();
}
