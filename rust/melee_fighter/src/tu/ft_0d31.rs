//! src/melee/ft/ft_0D31.c: blast-zone KO detection and the Dead* states (DeadLeft/Right/Down/Up,
//! DeadUpStar, DeadUpFall), KO accounting (falls, stocks, KOs, self-destructs). Hand port of
//! every function in the TU, with the TARGET_PC additions (GenoFly_Fighter, Script_GameEvent).
//!
//! Fused site (`tools/rust-port/fma_sites.py`, shipping contraction): ftCo_DeadUpStar_Anim
//! `x514 * Stage_GetCamBoundsTopOffset() - cur_pos.y` -> fmsubs. No other float arithmetic fuses.
#![allow(non_snake_case, static_mut_refs, unused_unsafe)]

use crate::hooks as h;
use crate::types::*;
use melee_core::fp::fmsubs;

use super::ft_0d4d::{fn_800D4DD4_cb, ftCo_800D4E50};

/// Calls into TUs that are not ported yet (TODO: rewire when they are).
#[allow(non_snake_case, non_upper_case_globals, unused_variables)]
pub mod ext {
    use crate::types::*;
    /// fighter.c globals (colour tables loaded with the common data).
    pub use crate::tu::fighter::Fighter_804D650C;
    pub use crate::tu::fighter::Fighter_804D6508;
    pub static mut H_Stage_GetBlastZoneRightOffset: Option<fn() -> f32> = None;
    pub fn Stage_GetBlastZoneRightOffset() -> f32 { match unsafe { H_Stage_GetBlastZoneRightOffset } { Some(f) => f(), None => unsafe { melee_world::tu::stage::Stage_GetBlastZoneRightOffset() } } }
    pub static mut H_Stage_GetBlastZoneLeftOffset: Option<fn() -> f32> = None;
    pub fn Stage_GetBlastZoneLeftOffset() -> f32 { match unsafe { H_Stage_GetBlastZoneLeftOffset } { Some(f) => f(), None => unsafe { melee_world::tu::stage::Stage_GetBlastZoneLeftOffset() } } }
    pub static mut H_Stage_GetBlastZoneTopOffset: Option<fn() -> f32> = None;
    pub fn Stage_GetBlastZoneTopOffset() -> f32 { match unsafe { H_Stage_GetBlastZoneTopOffset } { Some(f) => f(), None => unsafe { melee_world::tu::stage::Stage_GetBlastZoneTopOffset() } } }
    pub static mut H_Stage_GetBlastZoneBottomOffset: Option<fn() -> f32> = None;
    pub fn Stage_GetBlastZoneBottomOffset() -> f32 { match unsafe { H_Stage_GetBlastZoneBottomOffset } { Some(f) => f(), None => unsafe { melee_world::tu::stage::Stage_GetBlastZoneBottomOffset() } } }
    pub static mut H_Stage_GetCamBoundsTopOffset: Option<fn() -> f32> = None;
    pub fn Stage_GetCamBoundsTopOffset() -> f32 { match unsafe { H_Stage_GetCamBoundsTopOffset } { Some(f) => f(), None => unsafe { melee_world::tu::stage::Stage_GetCamBoundsTopOffset() } } }
    /// TARGET_PC pc/geno/geno_lab_mode.c debug flight.
    pub static mut H_GenoFly_Fighter: Option<fn(&mut Fighter) -> i32> = None;
    pub fn GenoFly_Fighter(fp: &mut Fighter) -> i32 { match unsafe { H_GenoFly_Fighter } { Some(f) => f(fp), None => panic!("TODO(pc geno) GenoFly_Fighter") } }
    pub static mut H_Camera_8003010C: Option<fn() -> bool> = None;
    pub fn Camera_8003010C() -> bool { match unsafe { H_Camera_8003010C } { Some(f) => f(), None => panic!("TODO(group2 camera) Camera_8003010C") } }
    pub static mut H_Camera_RequestQuake: Option<fn(CmQuakeKind, &Vec3)> = None;
    pub fn Camera_RequestQuake(kind: CmQuakeKind, pos: &Vec3) { match unsafe { H_Camera_RequestQuake } { Some(f) => f(kind, pos), None => panic!("TODO(group2 camera) Camera_RequestQuake") } }
    pub static mut H_ftCo_800DD100: Option<fn(*mut HSD_GObj)> = None;
    pub fn ftCo_800DD100(g: *mut HSD_GObj) { match unsafe { H_ftCo_800DD100 } { Some(f) => f(g), None => { unsafe { crate::tu::ftCo_Throw::ftCo_800DD100(g as _); } return; } } }
    pub static mut H_ftCo_800C8540: Option<fn(*mut HSD_GObj)> = None;
    pub fn ftCo_800C8540(g: *mut HSD_GObj) { match unsafe { H_ftCo_800C8540 } { Some(f) => f(g), None => { unsafe { crate::tu::ftmetal::ftCo_800C8540(g as _); } return; } } }
    pub static mut H_ftCo_800C8A64: Option<fn(*mut HSD_GObj)> = None;
    pub fn ftCo_800C8A64(g: *mut HSD_GObj) { match unsafe { H_ftCo_800C8A64 } { Some(f) => f(g), None => { unsafe { crate::tu::ft_0c88::ftco_800c8a64(g as _); } return; } } }
    pub static mut H_gm_8016B094: Option<fn() -> bool> = None;
    pub fn gm_8016B094() -> bool { match unsafe { H_gm_8016B094 } { Some(f) => f(), None => panic!("TODO(group6 gm) gm_8016B094") } }
    pub static mut H_gm_8016B0E8: Option<fn() -> bool> = None;
    pub fn gm_8016B0E8() -> bool { match unsafe { H_gm_8016B0E8 } { Some(f) => f(), None => panic!("TODO(group6 gm) gm_8016B0E8") } }
    pub static mut H_gm_801693BC: Option<fn(i32) -> bool> = None;
    pub fn gm_801693BC(slot: i32) -> bool { match unsafe { H_gm_801693BC } { Some(f) => f(slot), None => panic!("TODO(group6 gm) gm_801693BC") } }
    pub static mut H_pl_8003D644: Option<fn(i32, i32, i32, i32)> = None;
    pub fn pl_8003D644(slot: i32, sub: i32, msid: i32, x2073: i32) { match unsafe { H_pl_8003D644 } { Some(f) => f(slot, sub, msid, x2073), None => { unsafe { crate::tu::plbonuslib::pl_8003D644(slot as _, sub as _, msid as _, x2073 as _); } return; } } }
    pub static mut H_ftCo_800BFD9C: Option<fn(*mut HSD_GObj)> = None;
    pub fn ftCo_800BFD9C(g: *mut HSD_GObj) { match unsafe { H_ftCo_800BFD9C } { Some(f) => f(g), None => { unsafe { crate::tu::ftcolanim::ftco_800bfd9c(g as _); } return; } } }
    pub static mut H_ftCo_80090AC0: Option<fn(&mut Fighter)> = None;
    pub fn ftCo_80090AC0(fp: &mut Fighter) { match unsafe { H_ftCo_80090AC0 } { Some(f) => f(fp), None => { unsafe { crate::tu::ftCo_DamageIce::ftCo_80090AC0(fp as _); } return; } } }
    pub static mut H_efAsync_Spawn: Option<fn(*mut HSD_GObj, *mut core::ffi::c_void, u32, u32, *mut HSD_JObj, &Vec3)> = None;
    pub fn efAsync_Spawn(g: *mut HSD_GObj, q: *mut core::ffi::c_void, kind: u32, gfx: u32, jobj: *mut HSD_JObj, pos: &Vec3) { match unsafe { H_efAsync_Spawn } { Some(f) => f(g, q, kind, gfx, jobj, pos), None => panic!("TODO(group2 efasync) efAsync_Spawn") } }
    pub static mut H_ftAnim_80070FD0: Option<fn(&mut Fighter) -> bool> = None;
    pub fn ftAnim_80070FD0(fp: &mut Fighter) -> bool { match unsafe { H_ftAnim_80070FD0 } { Some(f) => f(fp), None => { return unsafe { crate::tu::ftanim::ftAnim_80070FD0(fp as _) as _ }; } } }
    /// jobj.c `HSD_JObjSetMtxDirtySub` (behind the HSD_JObjSetMtxDirty macro).
    pub static mut H_HSD_JObjSetMtxDirtySub: Option<fn(*mut HSD_JObj)> = None;
    pub fn HSD_JObjSetMtxDirtySub(j: *mut HSD_JObj) { match unsafe { H_HSD_JObjSetMtxDirtySub } { Some(f) => f(j), None => panic!("TODO(group1 jobj) HSD_JObjSetMtxDirtySub") } }
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
        fn Player_GetMoreFlagsBit5(_a0: i32) -> i32 = { crate::player::get_more_flags_bit5(_a0) as i32 };
        fn Player_GetStocks(_a0: i32) -> i32 = { crate::stale::stocks(_a0) as i32 };
        fn plStale_ResetStaleMoveTableForPlayer(_a0: i32) = { crate::tu::plstale::plStale_ResetStaleMoveTableForPlayer(_a0) };
        fn lbVector_Lerp(_a: *mut core::ffi::c_void, _b: *mut core::ffi::c_void, _out: *mut core::ffi::c_void, _t: f32) = { unsafe { let r = melee_runtime::lbvector::lerp(&super::to_rt(&*(_a as *const Vec3)), &super::to_rt(&*(_b as *const Vec3)), _t); *(_out as *mut Vec3) = Vec3 { x: r.x, y: r.y, z: r.z }; } };
        fn lbVector_Add(_a: *mut core::ffi::c_void, _b: *mut core::ffi::c_void) = { unsafe { let a = &mut *(_a as *mut Vec3); let b = &*(_b as *const Vec3); a.x += b.x; a.y += b.y; a.z += b.z; } };
        fn ftCommon_Fall(_fp: *mut core::ffi::c_void, _a: f32, _b: f32) = { unsafe { crate::ftcommon::fall(&mut *(_fp as *mut Fighter), _a, _b) } };
        fn Fighter_ChangeMotionState(_a0: *mut core::ffi::c_void, _a1: i32, _a2: u32, _a3: f32, _a4: f32, _a5: f32, _a6: *mut core::ffi::c_void) = { unsafe { let _ = super::super::ft_0d4d::ext::Fighter_ChangeMotionState(_a0 as _, _a1 as _, _a2 as _, _a3 as _, _a4 as _, _a5 as _, _a6 as _); } };
        fn Player_GetEntity(_a0: i32) -> *mut core::ffi::c_void = { unsafe { (crate::player::get_entity(_a0 as _)) as _ } };
        fn Player_GetFallsByIndex(_a0: i32, _a1: i32) -> i32 = { crate::tu::player::Player_GetFallsByIndex(_a0, _a1) };
        fn Player_GetUnk45(_a0: i32) -> u32 = { crate::tu::player::Player_GetUnk45(_a0) };
        fn Player_IncSelfDestructs(_a0: i32, _a1: i32) = { crate::tu::player::Player_IncSelfDestructs(_a0, _a1) };
        fn Player_LoseStock(_a0: i32) = { crate::tu::player::Player_LoseStock(_a0) };
        fn Player_SetFallsByIndex(_a0: i32, _a1: i32, _a2: i32) = { crate::tu::player::Player_SetFallsByIndex(_a0, _a1, _a2) };
        fn Player_SetHPByIndex(_a0: i32, _a1: i32, _a2: i32) = { crate::tu::player::Player_SetHPByIndex(_a0, _a1, _a2) };
        fn Player_UpdateKOsBySlot(_a0: i32, _a1: i32, _a2: i32) = { unsafe { let _ = crate::player::update_kos_by_slot(_a0 as _, _a1 as _, _a2 as _); } };
        fn Player_UpdateMatchFrameCount(_a0: i32, _a1: i32) = { unsafe { let _ = crate::player::update_match_frame_count(_a0 as _, _a1 as _); } };
        fn Script_GameEvent(_a0: i32, _a1: i32, _a2: i32, _a3: i32, _a4: i32) = { unsafe { let _ = crate::hooks::script_game_event(_a0 as _, _a1 as _, _a2 as _, _a3 as _, _a4 as _); } };
        fn ftCamera_80076320(_a0: *mut core::ffi::c_void) = { unsafe { let _ = super::super::ft_0d4d::ext::ftCamera_80076320(_a0 as _); } };
        fn ftCo_800D4E50(_a0: *mut core::ffi::c_void, _a1: *mut core::ffi::c_void, _a2: i32, _a3: f32) = { unsafe { let _ = super::ftCo_800D4E50(&mut *(_a0 as *mut Fighter), &*(_a1 as *const Vec3), _a2 as _, _a3 as _); } };
        fn ftCommon_8007DB24(_a0: *mut core::ffi::c_void) = { unsafe { let _ = crate::ftcommon3::ftcommon_8007db24(_a0 as _); } };
        fn ftCommon_8007E2FC(_a0: *mut core::ffi::c_void) = { unsafe { let _ = crate::tu::ftcommon::ftcommon_8007e2fc(&mut *((*(_a0 as *mut HSD_GObj)).user_data as *mut Fighter)); } };
        fn ftCommon_8007EBAC(_a0: *mut core::ffi::c_void, _a1: u32, _a2: u32) = { unsafe { let _ = crate::ftcommon2::ftcommon_8007ebac(&mut *(_a0 as *mut Fighter), _a1 as _, _a2 as _); } };
        fn ftCommon_8007EFC0(_a0: *mut core::ffi::c_void, _a1: u32) = { unsafe { let _ = crate::ftcommon2::ftcommon_8007efc0(&mut *(_a0 as *mut Fighter), _a1 as _); } };
        fn ftCommon_8007F8E8(_a0: *mut core::ffi::c_void) = { unsafe { let _ = crate::ftcommon3::ftcommon_8007f8e8(_a0 as _); } };
        fn ftCommon_8007F9B4(_a0: *mut core::ffi::c_void) = { unsafe { let _ = crate::ftcommon3::ftcommon_8007f9b4(_a0 as _); } };
        fn ftCommon_8007FF74(_a0: *mut core::ffi::c_void) = { unsafe { let _ = crate::ftcommon2::ftcommon_8007ff74(&mut *((*(_a0 as *mut HSD_GObj)).user_data as *mut Fighter)); } };
        fn ftParts_GetBoneIndex(_a0: *mut core::ffi::c_void, _a1: i32) -> i32 = { unsafe { (crate::hooks::ftparts_get_bone_index(&*(_a0 as *const Fighter), _a1 as _)) as _ } };
        fn ft_8008805C(_a0: *mut core::ffi::c_void, _a1: i32) = { unsafe { let _ = crate::ft0877::ft_8008805c(&*(_a0 as *const Fighter), _a1 as _); } };
        fn ft_800880D8(_a0: *mut core::ffi::c_void) = { unsafe { let _ = crate::ft0877::ft_800880d8(&mut *(_a0 as *mut Fighter)); } };
        fn ft_800881D8(_a0: *mut core::ffi::c_void, _a1: i32, _a2: u8, _a3: u8) = { unsafe { let _ = crate::ft0881::ft_800881d8(&mut *(_a0 as *mut Fighter), _a1 as _, _a2 as _, _a3 as _); } };
        fn ft_80088C5C(_a0: *mut core::ffi::c_void) = { unsafe { let _ = crate::ft0881::ft_80088c5c(_a0 as _); } };
        fn ft_PlaySFX(_a0: *mut core::ffi::c_void, _a1: i32, _a2: u8, _a3: u8) = { unsafe { let _ = crate::hooks::play_sfx(&mut *(_a0 as *mut Fighter), _a1 as _, _a2 as _, _a3 as _); } };
        fn pl_8003DF44(_a0: i32, _a1: i32) = { unsafe { let _ = crate::plbonus2::pl_8003df44(_a0 as _, _a1 as _); } };
        fn pl_8003EA74(_a0: i32, _a1: i32) = { unsafe { let _ = crate::plbonus2::pl_8003ea74(_a0 as _, _a1 as _); } };
        fn pl_8004065C(_a0: i32, _a1: i32) = { unsafe { let _ = crate::plbonus::pl_8004065c(_a0 as _, _a1 as _); } };
    }
}

pub type G = *mut HSD_GObj;

/// `lbl_803B7500`.
pub const lbl_803B7500: Quaternion = Quaternion { x: 0.0, y: 3.1415927, z: 0.0, w: 0.0 };

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}
#[inline(always)]
fn cd() -> &'static ftCommonData {
    h::common_data()
}

/// jobj.h `HSD_JObjSetMtxDirty` macro: dirty unless already dirty (or a user matrix).
fn jobj_set_mtx_dirty(j: *mut HSD_JObj) {
    if !melee_types::inline::HSD_JObjMtxIsDirty(unsafe { &*j }) {
        ext::HSD_JObjSetMtxDirtySub(j);
    }
}
/// jobj.h `HSD_JObjAddRotationX` (inline).
fn jobj_add_rotation_x(j: *mut HSD_JObj, x: f32) {
    assert!(!j.is_null(), "jobj.h:1029");
    let jr = unsafe { &mut *j };
    jr.rotate.x += x;
    if jr.flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        jobj_set_mtx_dirty(j);
    }
}
/// jobj.h `HSD_JObjSetRotation` (inline).
fn jobj_set_rotation(j: *mut HSD_JObj, q: &Quaternion) {
    assert!(!j.is_null(), "jobj.h");
    let jr = unsafe { &mut *j };
    jr.rotate = *q;
    if jr.flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        jobj_set_mtx_dirty(j);
    }
}
fn part_joint(f: &Fighter, part: Fighter_Part) -> *mut HSD_JObj {
    let i = ext::ftParts_GetBoneIndex(crate::mtu_ptr::ptr(&(f)), (part as i32) as i32);
    unsafe { (*f.parts.offset(i as isize)).joint }
}
fn jobj_of(g: G) -> *mut HSD_JObj {
    unsafe { (*g).hsd_obj as *mut HSD_JObj }
}

/// `ftCo_800D3158`: past a blast zone? Enter the matching Dead state and return true.
pub fn ftCo_800D3158(g: G) -> bool {
    melee_core::ct!("ftCo_800D3158");
    let f = fp(g);
    let fc = &cd().fall_common;
    if f.x222A_b1() != 0 || f.x2228_b5() != 0 || f.is_sandbag() != 0 {
        return false;
    }
    if f.x2219_b2() != 0 {
        return false;
    }
    if f.x2219_b1() != 0 {
        return false;
    }
    if ext::GenoFly_Fighter(f) != 0 {
        return false;
    }
    if f.cur_pos.x > ext::Stage_GetBlastZoneRightOffset() {
        ftCo_800D3950(g);
        return true;
    }
    if f.cur_pos.x < ext::Stage_GetBlastZoneLeftOffset() {
        ftCo_800D3680(g);
        return true;
    }
    if f.cur_pos.y > ext::Stage_GetBlastZoneTopOffset()
        && (f.ground_or_air == GA_Ground || f.x2222_b3() != 0 || f.x8c_kb_vel.y > cd().x4F0)
    {
        if ext::Player_GetMoreFlagsBit5(f.player_id as i32) != 0 {
            ftCo_800D3E40(g);
        } else {
            let r = h::hsd_randi(100) + 1;
            if !ext::Camera_8003010C() && fc.x520 >= r {
                if f.motion_id == ftCo_MS_DamageIce as i32 {
                    ftCo_800D47B8(g);
                } else {
                    ftCo_800D4780(g);
                }
            } else if f.motion_id == ftCo_MS_DamageIce as i32 {
                ftCo_800D41C4(g);
            } else {
                ftCo_800D40B8(g);
            }
        }
        return true;
    }
    if f.cur_pos.y < ext::Stage_GetBlastZoneBottomOffset() {
        ftCo_800D3BC8(g);
        return true;
    }
    false
}

/// `ftCo_800D331C`: shared KO cleanup (credit the KO, death callbacks, drop items, metal off).
pub fn ftCo_800D331C(g: G) {
    melee_core::ct!("ftCo_800D331C");
    let f = fp(g);
    if !f.victim_gobj.is_null() {
        if f.anon_221b.x221B_b5() != 0 {
            let v = fp(f.victim_gobj);
            if v.is_sub_fighter() == 0 {
                v.x2180 = f.player_id as i32;
            }
        } else if f.is_sub_fighter() == 0 {
            { let __a1 = (h::ftlib_800874bc(f.victim_gobj)) as i32; let __a2 = (h::ftlib_80086be0(f.victim_gobj) as i32) as i32; ext::pl_8004065C(__a2, __a1) };
        }
    }
    let f = fp(g);
    if f.x2180 != 6 {
        ext::pl_8004065C((f.x2180) as i32, (0) as i32);
    }
    unsafe {
        if let Some(cb) = fp(g).death2_cb { cb(g) }
        if let Some(cb) = fp(g).death3_cb { cb(g) }
        if let Some(cb) = fp(g).death1_cb { cb(g) }
    }
    ext::ftCo_800DD100(g);
    ext::ftCommon_8007E2FC(crate::mtu_ptr::ptr(&((fp(g)).gobj)));
    let f = fp(g);
    if !f.item_gobj.is_null() {
        h::item_8026a8ec(f.item_gobj);
        let f = fp(g);
        f.item_gobj = core::ptr::null_mut();
        h::pl_8003ea08(f.player_id as i32, f.is_sub_fighter() as i32);
        ext::pl_8003EA74((f.player_id as i32) as i32, (f.is_sub_fighter() as i32) as i32);
    }
    if !fp(g).x197C.is_null() {
        ext::ftCommon_8007F8E8(crate::mtu_ptr::ptr(&(g)));
        h::item_8026a8ec(fp(g).x197C);
        ext::ftCommon_8007F9B4(crate::mtu_ptr::ptr(&(g)));
    }
    if !fp(g).x1980.is_null() {
        ext::ftCommon_8007F8E8(crate::mtu_ptr::ptr(&(g)));
        h::item_8026a8ec(fp(g).x1980);
        ext::ftCommon_8007FF74(crate::mtu_ptr::ptr(&((fp(g)).gobj)));
    }
    if fp(g).is_metal() != 0 {
        ext::ftCo_800C8540(g);
    }
    if fp(g).x2226_b4() != 0 {
        ext::ftCo_800C8A64(g);
    }
    ext::ftCommon_8007DB24(crate::mtu_ptr::ptr(&(g)));
    let f = fp(g);
    unsafe {
        f.mv.co.unk_800D331C.x6C = f.motion_id as i32;
        f.mv.co.unk_800D331C.x70 = f.x2070.anon_0.x2073 as i32;
    }
}

/// `ftCo_800D34E0`: KO accounting (falls, stale table, stock, KO credit / self-destruct).
pub fn ftCo_800D34E0(g: G) {
    melee_core::ct!("ftCo_800D34E0");
    let f = fp(g);
    let slot = f.player_id as i32;
    let sub = f.is_sub_fighter() as i32;
    ext::Player_SetFallsByIndex((slot) as i32, (sub) as i32, (ext::Player_GetFallsByIndex((slot) as i32, (sub) as i32) + 1) as i32);
    ext::plStale_ResetStaleMoveTableForPlayer(slot);
    if (ext::gm_8016B094() || ext::gm_8016B0E8()) && (ext::Player_GetEntity((slot) as i32) as *mut HSD_GObj) == g {
        ext::Player_LoseStock((slot) as i32);
    }
    if ext::gm_8016B094() && ext::Player_GetStocks(slot) <= 0 {
        ext::Player_UpdateMatchFrameCount((slot) as i32, (sub) as i32);
    }
    let f = fp(g);
    if f.dmg.x18c4_source_ply != 6 {
        ext::Player_UpdateKOsBySlot((f.dmg.x18c4_source_ply) as i32, (sub) as i32, (slot) as i32);
    } else {
        ext::Player_IncSelfDestructs((slot) as i32, (sub) as i32);
    }
    let f = fp(g);
    if f.is_sub_fighter() == 0 {
        // TARGET_PC: retail attribution, after fall/stock accounting; no inferred last hit.
        let mut attacker = f.dmg.x18c4_source_ply;
        if attacker < 0 || attacker >= 6 || attacker == slot {
            attacker = -1;
        }
        ext::Script_GameEvent((14) as i32, (attacker) as i32, (slot) as i32, (0) as i32, (0) as i32);
        ext::Script_GameEvent((15) as i32, (slot) as i32, (ext::Player_GetStocks(slot)) as i32, (0) as i32, (0) as i32);
    }
    let f = fp(g);
    let (a, b) = unsafe { (f.mv.co.unk_800D34E0.x6C, f.mv.co.unk_800D34E0.x70) };
    ext::pl_8003D644(slot, sub, a, b);
    ext::Player_SetHPByIndex((slot) as i32, (sub) as i32, (0) as i32);
}

/// `ftCo_800D35FC`: rumble this fighter (5) and every other present fighter (6).
pub fn ftCo_800D35FC(f: &mut Fighter) {
    melee_core::ct!("ftCo_800D35FC");
    ext::ftCommon_8007EBAC(crate::mtu_ptr::ptr(&(f)), (5) as u32, (cd().x4F8) as u32);
    let me = f as *mut Fighter;
    let mut cur = h::plink_head(HSD_GOBJ_PLINK_FIGHTER);
    while !cur.is_null() {
        let c = unsafe { (*cur).user_data as *mut Fighter };
        if c != me && unsafe { (*c).x221F_b3() } == 0 {
            ext::ftCommon_8007EBAC(crate::mtu_ptr::ptr(&(unsafe { &mut *c })), (6) as u32, (cd().x4FC) as u32);
        }
        cur = unsafe { (*cur).next };
    }
}

/// The four side/bottom blast-zone KOs (ftCo_800D3680 / 3950 / 3BC8 / 3E40) share one body.
/// `vertical`: clamp y to the blast zone (left/right KO); else clamp x (bottom/top KO).
fn dead_side(g: G, msid: i32, sfx: i32, vertical: bool, gfx_angle: f32, last: fn() -> f32, coin_angle: f32) {
    ftCo_800D331C(g);
    unsafe { fp(g).mv.co.unk_800D3680.x40 = cd().x500 };
    h_change_motion(g, msid, 0, 0.0, 1.0, 0.0);
    dead_flags(g);
    let f = fp(g);
    ext::pl_8003DF44((f.player_id as i32) as i32, (f.is_sub_fighter() as i32) as i32);

    let f = fp(g);
    let sfxs = unsafe { &*(*f.ft_data).x4C_sfx };
    f.set_invisible(1);
    f.set_x221F_b1(1);
    ext::Camera_RequestQuake(QuakeKind_Large, &f.cur_pos);
    ftCo_800D35FC(fp(g));
    ftCo_800D34E0(g);
    ext::ft_80088C5C(crate::mtu_ptr::ptr(&(g)));
    ftCo_800D38B8(fp(g), sfxs.x4);
    ftCo_800D38B8(fp(g), sfxs.x8);

    ext::ft_PlaySFX(crate::mtu_ptr::ptr(&(fp(g))), (sfx) as i32, (0x7F) as u8, (0x40) as u8);
    ext::ft_8008805C(crate::mtu_ptr::ptr(&(fp(g))), (sfx) as i32);
    let mut sp = fp(g).cur_pos;
    if vertical {
        let hi = ext::Stage_GetBlastZoneTopOffset();
        let lo = ext::Stage_GetBlastZoneBottomOffset();
        if sp.y > hi { sp.y = hi; }
        if sp.y < lo { sp.y = lo; }
    } else {
        let hi = ext::Stage_GetBlastZoneRightOffset();
        let lo = ext::Stage_GetBlastZoneLeftOffset();
        if sp.x > hi { sp.x = hi; }
        if sp.x < lo { sp.x = lo; }
    }
    let sp24: f32 = gfx_angle;
    spawn_ko_gfx(g, &sp, &sp24);
    if vertical { sp.x = last(); } else { sp.y = last(); }
    ext::ftCo_800D4E50(crate::mtu_ptr::ptr(&(fp(g))), crate::mtu_ptr::ptr(&(&sp)), (1) as i32, (coin_angle) as f32);
}

fn h_change_motion(g: G, msid: i32, flags: u32, a: f32, b: f32, c: f32) {
    ext::Fighter_ChangeMotionState(crate::mtu_ptr::ptr(&(g)), (msid) as i32, (flags) as u32, (a) as f32, (b) as f32, (c) as f32, crate::mtu_ptr::ptr(&(core::ptr::null_mut::<u8>())));
}

/// The inline block after each Dead state change: stop the looping SFX, mark the fighter dead.
fn dead_flags(g: G) {
    let f = fp(g);
    if f.x221D_b6() != 0 {
        ext::ft_800880D8(crate::mtu_ptr::ptr(&(f)));
        f.x2004 = 0;
    }
    f.set_x2219_b1(1);
    f.set_x221E_b1(1);
    f.set_x221E_b2(1);
}

/// The KO blast GFX in the player's colours (0x42C in some team modes, else 0x42B).
fn spawn_ko_gfx(g: G, pos: &Vec3, angle: &f32) {
    let f = fp(g);
    let off = ((ext::Player_GetUnk45((f.player_id as i32) as i32) as u32) << 2) as usize;
    let (c1, c2) = unsafe {
        let b1 = ext::Fighter_804D650C.add(off);
        let b2 = ext::Fighter_804D6508.add(off);
        (
            ((*b1 as i32) << 16) | ((*b1.add(1) as i32) << 8) | *b1.add(2) as i32,
            ((*b2 as i32) << 16) | ((*b2.add(1) as i32) << 8) | *b2.add(2) as i32,
        )
    };
    let kind = if ext::gm_801693BC(f.player_id as i32) { 0x42C } else { 0x42B };
    super::ft_0d4d::ext::efSync_Spawn(
        kind,
        g,
        &[
            pos as *const Vec3 as usize,
            angle as *const f32 as usize,
            &cd().x4F4 as *const f32 as usize,
            c1 as u32 as usize,
            c2 as u32 as usize,
        ]);
}

pub fn ftCo_800D3680(g: G) {
    melee_core::ct!("ftCo_800D3680");
    dead_side(g, 1, 0x88, true, -1.5707964, ext::Stage_GetBlastZoneLeftOffset, 0.0)
}

pub fn ftCo_800D38B8(f: &mut Fighter, arg1: i32) {
    melee_core::ct!("ftCo_800D38B8");
    ext::ft_800881D8(crate::mtu_ptr::ptr(&(f)), (arg1) as i32, (0x7F) as u8, (0x40) as u8);
    if f.x2225_b6() == 0 {
        ext::ft_8008805C(crate::mtu_ptr::ptr(&(f)), (arg1) as i32);
    }
}

fn dead_countdown(g: G) {
    let f = fp(g);
    unsafe {
        f.mv.co.unk_deadleft.x40 -= 1;
        if f.mv.co.unk_deadleft.x40 == 0 {
            ext::ftCo_800BFD9C(g);
        }
    }
}

pub fn ftCo_DeadLeft_Anim(g: G) { melee_core::ct!("ftCo_DeadLeft_Anim"); dead_countdown(g) }
pub fn ftCo_DeadLeft_Cam(_g: G) { melee_core::ct!("ftCo_DeadLeft_Cam");}

pub fn ftCo_800D3950(g: G) {
    melee_core::ct!("ftCo_800D3950");
    dead_side(g, 2, 0x89, true, 1.5707964, ext::Stage_GetBlastZoneRightOffset, 3.1415927)
}

pub fn ftCo_DeadRight_Anim(g: G) { melee_core::ct!("ftCo_DeadRight_Anim"); dead_countdown(g) }
pub fn ftCo_DeadRight_Cam(_g: G) { melee_core::ct!("ftCo_DeadRight_Cam");}

pub fn ftCo_800D3BC8(g: G) {
    melee_core::ct!("ftCo_800D3BC8");
    dead_side(g, 0, 0x61, false, 0.0, ext::Stage_GetBlastZoneBottomOffset, 1.5707964)
}

pub fn ftCo_DeadDown_Anim(g: G) { melee_core::ct!("ftCo_DeadDown_Anim"); dead_countdown(g) }
pub fn ftCo_DeadDown_Cam(_g: G) { melee_core::ct!("ftCo_DeadDown_Cam");}

pub fn ftCo_800D3E40(g: G) {
    melee_core::ct!("ftCo_800D3E40");
    dead_side(g, 3, 0x61, false, 3.1415927, ext::Stage_GetBlastZoneTopOffset, 4.712389)
}

pub fn ftCo_DeadUp_Anim(g: G) { melee_core::ct!("ftCo_DeadUp_Anim"); dead_countdown(g) }
pub fn ftCo_DeadUp_Cam(_g: G) { melee_core::ct!("ftCo_DeadUp_Cam");}

/// Shared body of ftCo_800D40B8 / ftCo_800D41C4 (DeadUpStar / DeadUpStarIce).
fn dead_up_star(g: G, msid: i32) {
    ftCo_800D331C(g);
    let f = fp(g);
    unsafe {
        f.mv.co.unk_deadup.x40 = cd().x504;
        f.mv.co.unk_deadup.x44 = 0;
    }
    h_change_motion(g, msid, 0, 0.0, 1.0, 0.0);
    dead_flags(g);
    ext::ft_80088C5C(crate::mtu_ptr::ptr(&(g)));
    let f = fp(g);
    ext::ftCommon_8007EFC0(crate::mtu_ptr::ptr(&(f)), (1) as u32);
    let s = unsafe { (*(*f.ft_data).x4C_sfx).xC };
    ext::ft_800881D8(crate::mtu_ptr::ptr(&(f)), (s) as i32, (127) as u8, (64) as u8);
    ext::pl_8003DF44((f.player_id as i32) as i32, (f.is_sub_fighter() as i32) as i32);
}

pub fn ftCo_800D40B8(g: G) {
    melee_core::ct!("ftCo_800D40B8");
    dead_up_star(g, ftCo_MS_DeadUpStar as i32);
    unsafe { fp(g).mv.co.unk_deadup.x68 = 0 };
}

pub fn ftCo_800D41C4(g: G) {
    melee_core::ct!("ftCo_800D41C4");
    fp(g).set_x2227_b6(1);
    dead_up_star(g, ftCo_MS_DeadUpStarIce as i32);
    ext::ftCo_80090AC0(fp(g));
    ext::ftCommon_8007EBAC(crate::mtu_ptr::ptr(&(fp(g))), (1) as u32, (0) as u32);
    unsafe { fp(g).mv.co.unk_deadup.x68 = 1 };
}

pub fn ftCo_DeadUpStar_Anim(g: G) {
    melee_core::ct!("ftCo_DeadUpStar_Anim");
    let f = fp(g);
    let d = cd();
    unsafe {
        if f.mv.co.unk_deadup.x44 == 1 && f.mv.co.unk_deadup.x68 != 0 {
            let rot = d.x51C_radians;
            jobj_add_rotation_x(part_joint(f, FtPart_XRotN), rot);
        }
        if f.mv.co.unk_deadup.x40 != 0 {
            f.mv.co.unk_deadup.x40 -= 1;
        }
        if f.mv.co.unk_deadup.x40 == 0 {
            match f.mv.co.unk_deadup.x44 {
                0 => {
                    let top = ext::Stage_GetCamBoundsTopOffset();
                    f.self_vel.y = fmsubs(d.x514, top, f.cur_pos.y) / d.x508 as f32;
                    f.self_vel.z = d.x510 / d.x508 as f32;
                    f.mv.co.unk_deadup.x40 = d.x508;
                    f.mv.co.unk_deadup.x44 = 1;
                }
                1 => {
                    ext::ftCommon_8007E2FC(crate::mtu_ptr::ptr(&((f).gobj)));
                    if f.mv.co.unk_deadup.x68 != 0 {
                        ext::ftCommon_8007DB24(crate::mtu_ptr::ptr(&(g)));
                    }
                    let f = fp(g);
                    ext::efAsync_Spawn(g, &mut f.x60C as *mut _ as *mut core::ffi::c_void, 2, 0x42D, core::ptr::null_mut(), &f.cur_pos);
                    let f = fp(g);
                    ext::ftCo_800D4E50(crate::mtu_ptr::ptr(&(f)), crate::mtu_ptr::at(&f.cur_pos), (0) as i32, (1.5707964) as f32);
                    let f = fp(g);
                    f.set_x221F_b1(1);
                    f.set_invisible(1);
                    ext::ft_80088C5C(crate::mtu_ptr::ptr(&(g)));
                    ext::ft_PlaySFX(crate::mtu_ptr::ptr(&(fp(g))), (0x83) as i32, (0x7F) as u8, (0x40) as u8);
                    ext::ft_8008805C(crate::mtu_ptr::ptr(&(fp(g))), (0x83) as i32);
                    ftCo_800D34E0(g);
                    let f = fp(g);
                    f.mv.co.unk_deadup.x40 = d.x50C;
                    f.mv.co.unk_deadup.x44 = 2;
                }
                2 => ext::ftCo_800BFD9C(g),
                _ => {}
            }
        }
    }
}

pub fn ftCo_DeadUpStar_Cam(g: G) { melee_core::ct!("ftCo_DeadUpStar_Cam"); ext::ftCamera_80076320(crate::mtu_ptr::ptr(&(g))) }

/// `ftCo_800D4580(gobj, msid)`: enter DeadUpFall (fall toward the screen).
pub fn ftCo_800D4580(g: G, arg1: i32) {
    melee_core::ct!("ftCo_800D4580");
    let fc = &cd().fall_common;
    ftCo_800D331C(g);
    let f = fp(g);
    unsafe {
        f.mv.co.unk_deadup.x40 = fc.x524;
        f.mv.co.unk_deadup.x44 = 0;
        f.mv.co.unk_deadup.x50 = fc.x538;
        f.mv.co.common.x24 = 0.0;
        f.mv.co.common.x20 = 0.0;
        f.mv.co.common.x1C = 0.0;
        // common.x1C/x20/x24 are unk_deadup.x5C.x/y/z at their GC offsets (the native layouts differ)
        f.mv.co.unk_deadup.x5C = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    }
    h_change_motion(g, arg1, Ft_MF_None as u32, 0.0, 1.0, 0.0);
    fp(g).set_x2220_b7(1);
    let j = jobj_of(g);
    let q = lbl_803B7500;
    if fp(g).x34_scale.z == 1.0 {
        jobj_set_rotation(j, &q);
    }
    dead_flags(g);
    ext::ft_80088C5C(crate::mtu_ptr::ptr(&(g)));
    let f = fp(g);
    ext::ftCommon_8007EFC0(crate::mtu_ptr::ptr(&(f)), (1) as u32);
    h::ftco_800bffd0(f, 0x2B, 0);
    ext::pl_8003DF44((f.player_id as i32) as i32, (f.is_sub_fighter() as i32) as i32);
}

pub fn ftCo_800D4780(g: G) {
    melee_core::ct!("ftCo_800D4780");
    ftCo_800D4580(g, 6);
    unsafe { fp(g).mv.co.unk_deadup.x68 = 0 };
}

pub fn ftCo_800D47B8(g: G) {
    melee_core::ct!("ftCo_800D47B8");
    fp(g).set_x2227_b6(1);
    ftCo_800D4580(g, 9);
    ext::ftCo_80090AC0(fp(g));
    ext::ftCommon_8007EBAC(crate::mtu_ptr::ptr(&(fp(g))), (1) as u32, (0) as u32);
    unsafe { fp(g).mv.co.unk_deadup.x68 = 1 };
}

/// `ftCo_800D481C(gobj, msid)`: the screen-hit part of DeadUpFall.
pub fn ftCo_800D481C(g: G, arg1: i32) {
    melee_core::ct!("ftCo_800D481C");
    h_change_motion(g, arg1, (Ft_MF_SkipRumble | Ft_MF_SkipColAnim) as u32, 0.0, 1.0, 0.0);
    fp(g).set_x2220_b7(1);
    let j = jobj_of(g);
    let q = lbl_803B7500;
    if fp(g).x34_scale.z == 1.0 {
        jobj_set_rotation(j, &q);
    }
    let f = fp(g);
    f.set_x2219_b1(1);
    f.set_x221E_b1(1);
    f.set_x221E_b2(1);
    ftCo_800D35FC(f);
    ext::Camera_RequestQuake(QuakeKind_Large, &fp(g).cur_pos);
    let f = fp(g);
    let s = unsafe { (*(*f.ft_data).x4C_sfx).x20 };
    h::ft_800889f4(f, s);
    let f = fp(g);
    ext::ftCo_800D4E50(crate::mtu_ptr::ptr(&(f)), crate::mtu_ptr::at(&f.cur_pos), (0) as i32, (1.5707964) as f32);
    fp(g).accessory4_cb = Some(fn_800D4DD4_cb);
}

pub fn ftCo_DeadUpFall_Anim(g: G) {
    melee_core::ct!("ftCo_DeadUpFall_Anim");
    let fc = &cd().fall_common;
    let f = fp(g);
    unsafe {
        if f.mv.co.unk_deadup.x44 == 1 {
            f.mv.co.walk.middle_anim_frame += f.mv.co.walk.slow_anim_frame;
            if f.mv.co.unk_deadup.x68 != 0 {
                let rot = fc.x560_radians;
                jobj_add_rotation_x(part_joint(f, FtPart_XRotN), rot);
            }
        }
        if f.mv.co.unk_deadup.x40 != 0 {
            f.mv.co.unk_deadup.x40 -= 1;
        }
        if f.mv.co.unk_deadup.x40 == 0 {
            match f.mv.co.unk_deadup.x44 {
                0 => {
                    f.mv.co.walk.slow_anim_frame = 1.0f32 / fc.x528 as f32;
                    f.mv.co.walk.middle_anim_frame = f.mv.co.walk.slow_anim_frame;
                    f.mv.co.unk_deadup.x40 = fc.x528;
                    f.mv.co.unk_deadup.x44 = 1;
                }
                1 => {
                    if f.mv.co.unk_deadup.x68 != 0 {
                        ftCo_800D481C(g, 0xA);
                        ext::ftCo_80090AC0(fp(g));
                    } else if fp(g).x34_scale.z != 1.0 {
                        ftCo_800D481C(g, 8);
                    } else {
                        ftCo_800D481C(g, 7);
                    }
                    let f = fp(g);
                    f.mv.co.unk_deadup.x40 = fc.x52C;
                    f.mv.co.unk_deadup.x44 = 2;
                }
                2 => {
                    f.self_vel.y = fc.x550;
                    f.self_vel.z = fc.x55C;
                    f.mv.co.unk_deadup.x40 = fc.x530;
                    f.mv.co.unk_deadup.x44 = 3;
                    if h::slippi_codes() == 2 {
                        // Slippi online FreezeDeadUpFallPhysics @800D4C1C: the fall velocity lives in mv (x48 = vy,
                        // x4C = vz) and self_vel is zeroed, so cur_pos stays where the KO happened
                        let m = &mut f.mv.co.unk_deadup as *mut _ as *mut u8;
                        *(m.add(8) as *mut f32) = fc.x550;
                        *(m.add(0xC) as *mut f32) = fc.x55C;
                        f.self_vel = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
                    }
                }
                3 => {
                    ext::ftCommon_8007E2FC(crate::mtu_ptr::ptr(&((f).gobj)));
                    if f.mv.co.unk_deadup.x68 != 0 {
                        ext::ftCommon_8007DB24(crate::mtu_ptr::ptr(&(g)));
                    }
                    let f = fp(g);
                    f.set_x221F_b1(1);
                    f.set_invisible(1);
                    ftCo_800D34E0(g);
                    ext::ft_80088C5C(crate::mtu_ptr::ptr(&(g)));
                    ext::ft_PlaySFX(crate::mtu_ptr::ptr(&(fp(g))), (0x61) as i32, (0x7F) as u8, (0x40) as u8);
                    ext::ft_8008805C(crate::mtu_ptr::ptr(&(fp(g))), (0x61) as i32);
                    ext::ftCommon_8007EBAC(crate::mtu_ptr::ptr(&(fp(g))), (0xD) as u32, (0) as u32);
                    ext::Camera_RequestQuake(QuakeKind_Large, &fp(g).cur_pos);
                    let f = fp(g);
                    f.mv.co.unk_deadup.x40 = fc.x534;
                    f.mv.co.unk_deadup.x44 = 4;
                }
                4 => ext::ftCo_800BFD9C(g),
                _ => {}
            }
        }
    }
}

#[inline]
fn to_rt(v: &Vec3) -> melee_core::vec::Vec3 { melee_core::vec::Vec3 { x: v.x, y: v.y, z: v.z } }

pub fn ftCo_DeadUpFall_Phys(g: G) {
    melee_core::ct!("ftCo_DeadUpFall_Phys");
    let ca = &cd().fall_common;
    let f = fp(g);
    unsafe {
        match f.mv.co.unk_deadup.x44 {
            1 => {
                if f.x2222_b6() != 0 && !ext::ftAnim_80070FD0(f) {
                    return;
                }
                let f = fp(g);
                ext::lbVector_Lerp(crate::mtu_ptr::at(&ca.x538), crate::mtu_ptr::at(&ca.x544), crate::mtu_ptr::at(&f.mv.co.unk_deadup.x50), f.mv.co.unk_deadup.x4C);
            }
            3 => {
                if h::slippi_codes() == 2 {
                    // Slippi online FreezeDeadUpFallPhysics @800D4D68: fall on mv's x48 (vy) instead of self_vel
                    // (fsub; max with -terminal), then x5C.y += x48, x5C.z += x4C
                    let m = &mut f.mv.co.unk_deadup as *mut _ as *mut u8;
                    let vy = m.add(8) as *mut f32;
                    let vz = m.add(0xC) as *mut f32;
                    let mut v = ((*vy as f64) - (ca.x554 as f64)) as f32;
                    let lim = -ca.x558;
                    if !(v > lim) {
                        v = lim;
                    }
                    *vy = v;
                    f.mv.co.unk_deadup.x5C.y = f.mv.co.unk_deadup.x5C.y + *vy;
                    f.mv.co.unk_deadup.x5C.z = *vz + f.mv.co.unk_deadup.x5C.z;
                } else {
                ext::ftCommon_Fall(crate::mtu_ptr::ptr(&(f)), ca.x554, ca.x558);
                ext::lbVector_Add(crate::mtu_ptr::at(&f.mv.co.unk_deadup.x5C), crate::mtu_ptr::at(&f.self_vel));
                }
                if f.x2222_b6() != 0 && !ext::ftAnim_80070FD0(f) {
                    return;
                }
                let f = fp(g);
                ext::lbVector_Add(crate::mtu_ptr::at(&f.mv.co.unk_deadup.x50), crate::mtu_ptr::at(&f.mv.co.unk_deadup.x5C));
                f.mv.co.unk_deadup.x5C = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            }
            _ => {}
        }
    }
}
