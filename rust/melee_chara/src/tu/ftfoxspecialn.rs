//! src/melee/ft/kinds/ftFox/ftfoxspecialn.c: Fox / Falco Blaster (neutral B) and the blaster
//! throw animation. The launch angle is a C `f64` (`M_PI - (double) x10`) narrowed to `f32` at the
//! item call. No fused sites (tools/rust-port/fma_sites.py: 0 fmuladd in every function).
//! Smoke-tested by tests/tu_ftfoxspecialn.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_runtime::lbtrigf::atan2f;
use melee_types::*;

/// Already ported (rounds 2/6/12/14): re-exported under this TU.
pub use crate::chars::r12::ftFox::ftfoxspecialn::{ftFx_SpecialAirNEnd_IASA, ftFx_SpecialNEnd_IASA};
pub use crate::chars::r14::ftFox::ftfoxspecialn::{ftFox_SpecialN_RemoveBlasterNULL, ftFx_SpecialN_ClearBlaster};
pub use crate::chars::r2::ftFox::ftfoxspecialn::{
    ftFx_SpecialAirNEnd_Phys, ftFx_SpecialAirNLoop_Phys, ftFx_SpecialAirNStart_Phys, ftFx_SpecialNEnd_Coll,
    ftFx_SpecialNEnd_Phys, ftFx_SpecialNLoop_Coll, ftFx_SpecialNLoop_Phys, ftFx_SpecialNStart_Coll,
    ftFx_SpecialNStart_Phys,
};
pub use crate::chars::r6::ftFox::ftfoxspecialn::{
    ftFx_SpecialAirNEnd_Coll, ftFx_SpecialAirNLoop_Coll, ftFx_SpecialAirNStart_Coll,
};

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use melee_types::*;
    macro_rules! hook {
        ($hook:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => f($($a),*),
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            }
        };
    }
    /// Like `hook!`, but an unset hook runs the native port `$port` (a ported TU elsewhere).
    macro_rules! hook_or {
        ($hook:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?, $port:expr) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => f($($a),*),
                    None => $port($($a),*),
                }
            }
        };
    }
    pub static mut LB_8000B1CC: Option<fn(*mut HSD_JObj, *const Vec3, *mut Vec3) > = None;
            #[allow(non_snake_case)]
            pub fn lb_8000B1CC(joint: *mut HSD_JObj, off: *const Vec3, out: *mut Vec3)  {
                match unsafe { LB_8000B1CC } {
                    Some(f) => f(joint, off, out),
                    None => { unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(joint as _, off as _, out as _); } return; },
                }
            }
    crate::port_hook!(FT_800892A0, ft_800892A0, crate::adapt::ft_800892A0, (gobj: *mut HSD_GObj));
    hook_or!(FT_80089824, ft_80089824, (gobj: *mut HSD_GObj), melee_fighter::ft0892::ft_80089824);
    pub static mut FTFX_INIT_800E5534: Option<fn(*mut HSD_GObj) -> bool> = None;
            #[allow(non_snake_case)]
            pub fn ftFx_Init_800E5534(gobj: *mut HSD_GObj) -> bool {
                match unsafe { FTFX_INIT_800E5534 } {
                    Some(f) => f(gobj),
                    None => { return unsafe { crate::tu::ftfox::ftFx_Init_800E5534(gobj as _) as _ }; },
                }
            }
    pub static mut FTFX_INIT_800E5588: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn ftFx_Init_800E5588(gobj: *mut HSD_GObj)  {
                match unsafe { FTFX_INIT_800E5588 } {
                    Some(f) => f(gobj),
                    None => { unsafe { crate::tu::ftfox::ftFx_Init_800E5588(gobj as _); } return; },
                }
            }
    hook_or!(FTPICKUPITEM_80094818, ftpickupitem_80094818, (gobj: *mut HSD_GObj, arg1: bool),
        melee_fighter::tu::ftpickupitem::ftpickupitem_80094818);
    hook_or!(FTPARTS_GETBONEINDEX, ftParts_GetBoneIndex, (fp: *mut Fighter, part: i32) -> i32,
        |fp: *mut Fighter, part: i32| unsafe { melee_fighter::hooks::ftparts_get_bone_index(&*fp, part) });
    hook_or!(FTLIB_GETKIND, ftLib_GetKind, (gobj: *mut HSD_GObj) -> FighterKind, melee_fighter::hooks::ftlib_get_kind);
    hook_or!(FT_PLAYSFX, ft_PlaySFX, (fp: *mut Fighter, sfx: i32, vol: i32, pan: i32),
        |fp: *mut Fighter, sfx: i32, vol: i32, pan: i32| unsafe { melee_fighter::hooks::play_sfx(&mut *fp, sfx, vol, pan) });
    hook_or!(OSREPORT, OSReport, (msg: &str), |_msg: &str| {});
    // Items (it/kinds/itfoxblaster.c, itfoxlaser.c, it_26B1.c)
    pub static mut IT_802AE8A8: Option<fn(f32, *mut HSD_GObj, *const Vec3, i32, ItemKind) -> *mut HSD_GObj> = None;
            #[allow(non_snake_case)]
            pub fn it_802AE8A8(facing_dir: f32, owner: *mut HSD_GObj, pos: *const Vec3, bone: i32, kind: ItemKind) -> *mut HSD_GObj {
                match unsafe { IT_802AE8A8 } {
                    Some(f) => f(facing_dir, owner, pos, bone, kind),
                    None => { return unsafe { melee_items::tu::itfoxblaster::it_802AE8A8(facing_dir as _, owner as _, pos as _, bone as _, kind as _) as _ }; },
                }
            }
    pub static mut IT_8026BAE8: Option<fn(*mut HSD_GObj, f32) > = None;
            #[allow(non_snake_case)]
            pub fn it_8026BAE8(item: *mut HSD_GObj, scale_mul: f32)  {
                match unsafe { IT_8026BAE8 } {
                    Some(f) => f(item, scale_mul),
                    None => { unsafe { melee_items::tu::it_26B1::it_8026BAE8(item as _, scale_mul as _); } return; },
                }
            }
    pub static mut IT_802ADDD0: Option<fn(*mut HSD_GObj, i32) > = None;
            #[allow(non_snake_case)]
            pub fn it_802ADDD0(item: *mut HSD_GObj, visibility: i32)  {
                match unsafe { IT_802ADDD0 } {
                    Some(f) => f(item, visibility),
                    None => { unsafe { melee_items::tu::itfoxblaster::it_802ADDD0(item as _, visibility as _); } return; },
                }
            }
    pub static mut IT_802AE538: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802AE538(item: *mut HSD_GObj)  {
                match unsafe { IT_802AE538 } {
                    Some(f) => f(item),
                    None => { unsafe { melee_items::tu::itfoxblaster::it_802AE538(item as _); } return; },
                }
            }
    pub static mut IT_802AE608: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802AE608(item: *mut HSD_GObj)  {
                match unsafe { IT_802AE608 } {
                    Some(f) => f(item),
                    None => { unsafe { melee_items::tu::itfoxblaster::it_802AE608(item as _); } return; },
                }
            }
    pub static mut IT_802AE1D0: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802AE1D0(item: *mut HSD_GObj)  {
                match unsafe { IT_802AE1D0 } {
                    Some(f) => f(item),
                    None => { unsafe { melee_items::tu::itfoxblaster::it_802AE1D0(item as _); } return; },
                }
            }
    pub static mut IT_802AEAB4: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802AEAB4(item: *mut HSD_GObj)  {
                match unsafe { IT_802AEAB4 } {
                    Some(f) => f(item),
                    None => { unsafe { melee_items::tu::itfoxblaster::it_802AEAB4(item as _); } return; },
                }
            }
    pub static mut IT_8029C6A4: Option<fn(f32, f32, *mut HSD_GObj, *mut Vec3, ItemKind) > = None;
            #[allow(non_snake_case)]
            pub fn it_8029C6A4(angle: f32, vel: f32, parent: *mut HSD_GObj, pos: *mut Vec3, kind: ItemKind)  {
                match unsafe { IT_8029C6A4 } {
                    Some(f) => f(angle, vel, parent, pos, kind),
                    None => { unsafe { melee_items::tu::itfoxlaser::it_8029C6A4(angle as _, vel as _, parent as _, pos as _, kind as _); } return; },
                }
            }
    pub static mut IT_8029C6CC: Option<fn(f32, f32, *mut HSD_GObj, *mut Vec3, ItemKind) > = None;
            #[allow(non_snake_case)]
            pub fn it_8029C6CC(angle: f32, vel: f32, parent: *mut HSD_GObj, pos: *mut Vec3, kind: ItemKind)  {
                match unsafe { IT_8029C6CC } {
                    Some(f) => f(angle, vel, parent, pos, kind),
                    None => { unsafe { melee_items::tu::itfoxlaser::it_8029C6CC(angle as _, vel as _, parent as _, pos as _, kind as _); } return; },
                }
            }
}

/// `foxSFX` / `falcoSFX` (indexed by `facing_dir == -1`).
pub const foxSFX: [u32; 2] = [110103, 110106];
pub const falcoSFX: [u32; 2] = [100099, 100102];

const SFX_VOLUME_MAX: i32 = 127;
const SFX_PAN_MID: i32 = 64;

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}
#[inline(always)]
unsafe fn da(fp: *mut Fighter) -> *const ftFox_DatAttrs {
    (*fp).dat_attrs as *const ftFox_DatAttrs
}

/// `ftGetTemplateKind` (ft/inlines.h, TARGET_PC).
fn ftGetTemplateKind(fp: &Fighter) -> i32 {
    let kind = fp.kind as i32;
    if kind >= Ft_Kind_Mex0 as i32 && melee_fighter::hooks::geno_define_template_skeleton(kind) != 0 {
        return melee_fighter::hooks::geno_define_base_kind(kind);
    }
    kind
}

/// `ftCheckThrowB0` (ft/inlines.h): test and clear throw_flags_b0.
unsafe fn ftCheckThrowB0(fp: *mut Fighter) -> bool {
    if (*fp).anon_2210.anon_0.throw_flags_b0() != 0 {
        (*fp).anon_2210.anon_0.set_throw_flags_b0(0);
        true
    } else {
        false
    }
}

/// `Fighter_SetDamageCallback` (ft/inlines.h).
unsafe fn set_damage_callback(gobj: *mut HSD_GObj, cb: HSD_GObjEvent) {
    let fp = fpp(gobj);
    (*fp).take_dmg_cb = cb;
    (*fp).death2_cb = cb;
}

pub unsafe extern "C" fn cb_ftFx_Init_800E5588(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftFx_Init_800E5588");
    ext::ftFx_Init_800E5588(g);
}
pub unsafe extern "C" fn cb_ftFx_SpecialN_OnChangeAction(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftFx_SpecialN_OnChangeAction");
    ftFx_SpecialN_OnChangeAction(g);
}
pub unsafe extern "C" fn cb_ftFx_SpecialN_CreateBlasterShot(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftFx_SpecialN_CreateBlasterShot");
    ftFx_SpecialN_CreateBlasterShot(g);
}

/// `ftFox_SpecialN_GetHoldJoint` (static inline): world position of the right thumb offset by `z_offset`.
pub fn ftFox_SpecialN_GetHoldJoint(gobj: *mut HSD_GObj, pos: *mut Vec3, z_offset: f32) {
    melee_core::ct!("ftFox_SpecialN_GetHoldJoint");
    unsafe {
        let fp = fpp(gobj);
        let sp14 = Vec3 { x: 0.0, y: 1.2325000762939453_f32, z: z_offset };
        let bone = ext::ftParts_GetBoneIndex(fp, FtPart_RThumbNb as i32);
        ext::lb_8000B1CC((*(*fp).parts.add(bone as usize)).joint, &sp14, pos);
    }
}

pub fn ftFx_SpecialN_FtGetHoldJoint(gobj: *mut HSD_GObj, pos: *mut Vec3) {
    melee_core::ct!("ftFx_SpecialN_FtGetHoldJoint");
    ftFox_SpecialN_GetHoldJoint(gobj, pos, 4.263599872589111_f32);
}

pub fn ftFx_SpecialN_ItGetHoldJoint(gobj: *mut HSD_GObj, pos: *mut Vec3) {
    melee_core::ct!("ftFx_SpecialN_ItGetHoldJoint");
    ftFox_SpecialN_GetHoldJoint(gobj, pos, 0.013600001111626625_f32);
}

pub fn ftFx_SpecialN_OnChangeAction(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialN_OnChangeAction");
    ext::ft_800892A0(gobj);
    ext::ft_80089824(gobj);
}

/// True when no blaster is held.
pub fn ftFx_SpecialN_CheckRemoveBlaster(gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("ftFx_SpecialN_CheckRemoveBlaster");
    unsafe { (*fpp(gobj)).u.fx.x222C_blasterGObj.is_null() }
}

/// Blaster action index for the current motion (9 when none).
pub fn ftFx_SpecialN_GetBlasterAction(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("ftFx_SpecialN_GetBlasterAction");
    unsafe {
        let mut msid = 9;
        if !gobj.is_null() {
            let fp = fpp(gobj);
            if !fp.is_null() {
                let cur = (*fp).motion_id as i32;
                if (ftFx_MS_SpecialNStart as i32..=ftFx_MS_SpecialAirNEnd as i32).contains(&cur) {
                    msid = cur - ftFx_MS_SpecialNStart as i32;
                } else if cur == ftCo_MS_ThrowB as i32 || cur == ftCo_MS_ThrowHi as i32 || cur == ftCo_MS_ThrowLw as i32 {
                    msid = cur - ftCo_MS_CatchDash as i32;
                }
            }
        }
        msid
    }
}

/// True unless a blaster-owning motion has the `x2071_b6` flag clear.
pub fn ftFx_SpecialN_CheckBlasterAction(gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("ftFx_SpecialN_CheckBlasterAction");
    unsafe {
        if !gobj.is_null() {
            let fp = fpp(gobj);
            let cur = (*fp).motion_id as i32;
            if (ftFx_MS_SpecialNStart as i32..=ftFx_MS_SpecialAirNEnd as i32).contains(&cur)
                || cur == ftCo_MS_ThrowB as i32
                || cur == ftCo_MS_ThrowHi as i32
                || cur == ftCo_MS_ThrowLw as i32
            {
                return (*fp).x2070.anon_0.x2071_b6() != 0;
            }
        }
        true
    }
}

pub fn ftFx_SpecialN_RemoveBlaster(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialN_RemoveBlaster");
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.fx.x222C_blasterGObj.is_null() {
            ext::it_802AEAB4((*fp).u.fx.x222C_blasterGObj);
            (*fp).u.fx.x222C_blasterGObj = core::ptr::null_mut();
        }
        if !ext::ftFx_Init_800E5534(gobj) {
            set_damage_callback(gobj, None);
        }
    }
}

/// `ftFox_SpecialN_PrepareBlasterShot` (static inline): shot origin and launch angle (a C `f64`).
pub fn ftFox_SpecialN_PrepareBlasterShot(gobj: *mut HSD_GObj, fp: *mut Fighter, da: *const ftFox_DatAttrs, pos: *mut Vec3) -> f64 {
    melee_core::ct!("ftFox_SpecialN_PrepareBlasterShot");
    unsafe {
        ftFx_SpecialN_FtGetHoldJoint(gobj, pos);
        (*pos).z = 0.0;
        if (*fp).facing_dir == 1.0 {
            return (*da).x10_FOX_BLASTER_ANGLE as f64;
        }
        core::f64::consts::PI - (*da).x10_FOX_BLASTER_ANGLE as f64
    }
}

/// `ftFox_SpecialN_FireBlasterShot` (static inline): launch the laser and play the shot SFX.
pub fn ftFox_SpecialN_FireBlasterShot(gobj: *mut HSD_GObj, fp: *mut Fighter, da: *const ftFox_DatAttrs, pos: *mut Vec3, launch_angle: f64) {
    melee_core::ct!("ftFox_SpecialN_FireBlasterShot");
    unsafe {
        ext::it_8029C6A4(
            launch_angle as f32,
            (*da).x14_FOX_BLASTER_VEL,
            gobj,
            pos,
            (*da).x1C_FOX_BLASTER_SHOT_ITKIND,
        );
        ext::it_802AE1D0((*fp).u.fx.x222C_blasterGObj);

        let idx = ((*fp).facing_dir == -1.0) as usize;
        match ext::ftLib_GetKind(gobj) {
            k if k == Ft_Kind_Fox => ext::ft_PlaySFX(fp, foxSFX[idx] as i32, SFX_VOLUME_MAX, SFX_PAN_MID),
            k if k == Ft_Kind_Falco => ext::ft_PlaySFX(fp, falcoSFX[idx] as i32, SFX_VOLUME_MAX, SFX_PAN_MID),
            _ => {}
        }
    }
}

/// Animation-script hook: fire the shot when the script sets `cmd_vars[2]`.
pub fn ftFx_SpecialN_CreateBlasterShot(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialN_CreateBlasterShot");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        let mut sp2c = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

        if (*fp).cmd_vars[2] != 0 {
            (*fp).cmd_vars[2] = 0;
            let launch_angle = ftFox_SpecialN_PrepareBlasterShot(gobj, fp, da, &mut sp2c);
            ftFox_SpecialN_FireBlasterShot(gobj, fp, da, &mut sp2c, launch_angle);
        }
    }
}

/// `ftFox_SpecialN_SpawnBlaster` (static inline): create the held blaster item.
pub fn ftFox_SpecialN_SpawnBlaster(gobj: *mut HSD_GObj, fp: *mut Fighter, da: *const ftFox_DatAttrs, assert_line: i32) {
    melee_core::ct!("ftFox_SpecialN_SpawnBlaster");
    unsafe {
        let bone = ext::ftParts_GetBoneIndex(fp, FtPart_RThumbNb as i32);
        let blaster_gobj = ext::it_802AE8A8((*fp).facing_dir, gobj, &(*fp).cur_pos, bone, (*da).x20_FOX_BLASTER_GUN_ITKIND);
        (*fp).u.fx.x222C_blasterGObj = blaster_gobj;

        if !blaster_gobj.is_null() {
            ext::it_8026BAE8((*fp).u.fx.x222C_blasterGObj, 0.85_f64 as f32);
            set_damage_callback(gobj, Some(cb_ftFx_Init_800E5588));
            return;
        }

        ext::OSReport("ftToSpecialNFox::Caution!!!\n");
        panic!("__assert: ftfoxspecialn.c:{assert_line}: 0");
    }
}

/// `ftFox_SpecialN_InitializeState` (static inline).
pub fn ftFox_SpecialN_InitializeState(gobj: *mut HSD_GObj, fp: *mut Fighter) {
    melee_core::ct!("ftFox_SpecialN_InitializeState");
    unsafe {
        st::Fighter_ClearCmdVars(&mut *fp);
        st::ftAnim_8006EBA4(gobj);
    }
}

pub fn ftFx_SpecialN_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialN_Enter");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);

        st::ftCommon_8007D7FC(&mut *fp);

        st::Fighter_ChangeMotionState(gobj, ftFx_MS_SpecialNStart as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        ftFox_SpecialN_InitializeState(gobj, fp);

        (*fp).gr_vel = 0.0;
        (*fp).self_vel.z = 0.0;
        (*fp).self_vel.y = 0.0;
        (*fp).self_vel.x = 0.0;

        (*fp).mv.fx.SpecialN.isBlasterLoop = 0;
        ftFox_SpecialN_SpawnBlaster(gobj, fp, da, 305);
    }
}

pub fn ftFx_SpecialAirN_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialAirN_Enter");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);

        st::Fighter_ChangeMotionState(gobj, ftFx_MS_SpecialAirNStart as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        ftFox_SpecialN_InitializeState(gobj, fp);

        (*fp).mv.fx.SpecialN.isBlasterLoop = 0;
        ftFox_SpecialN_SpawnBlaster(gobj, fp, da, 333);
    }
}

/// `ftFox_SpecialN_UpdateBlaster` (static inline).
pub fn ftFox_SpecialN_UpdateBlaster(fp: *mut Fighter) {
    melee_core::ct!("ftFox_SpecialN_UpdateBlaster");
    unsafe {
        ext::it_802ADDD0((*fp).u.fx.x222C_blasterGObj, 1);
        if (*fp).cmd_vars[3] == 1 && !(*fp).u.fx.x222C_blasterGObj.is_null() {
            (*fp).cmd_vars[3] = 0;
            ext::it_802AE538((*fp).u.fx.x222C_blasterGObj);
        }
    }
}

/// `ftFox_SpecialN_StartAnimation` (static inline).
pub fn ftFox_SpecialN_StartAnimation(gobj: *mut HSD_GObj, loop_msid: i32) {
    melee_core::ct!("ftFox_SpecialN_StartAnimation");
    unsafe {
        let fp = fpp(gobj);
        ftFox_SpecialN_UpdateBlaster(fp);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::Fighter_ChangeMotionState(
                gobj,
                loop_msid,
                (Ft_MF_SkipModel | Ft_MF_KeepGfx) as u32,
                0.0,
                1.0,
                0.0,
                core::ptr::null_mut(),
            );
            set_damage_callback(gobj, Some(cb_ftFx_Init_800E5588));
            (*fp).accessory4_cb = Some(cb_ftFx_SpecialN_CreateBlasterShot);
            ext::it_802ADDD0((*fp).u.fx.x222C_blasterGObj, 1);
        }
    }
}

pub fn ftFx_SpecialNStart_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialNStart_Anim");
    ftFox_SpecialN_StartAnimation(gobj, ftFx_MS_SpecialNLoop as i32);
}

/// `ftFox_SpecialN_BeginLoopTransition` (static inline).
pub fn ftFox_SpecialN_BeginLoopTransition(gobj: *mut HSD_GObj, fp: *mut Fighter, loop_msid: i32) {
    melee_core::ct!("ftFox_SpecialN_BeginLoopTransition");
    unsafe {
        (*fp).x21EC = Some(cb_ftFx_SpecialN_OnChangeAction);
        st::Fighter_ChangeMotionState(
            gobj,
            loop_msid,
            (Ft_MF_SkipAttackCount | Ft_MF_SkipModel | Ft_MF_KeepGfx) as u32,
            0.0,
            1.0,
            0.0,
            core::ptr::null_mut(),
        );
    }
}

/// `ftFox_SpecialN_FinishLoopTransition` (static inline).
pub fn ftFox_SpecialN_FinishLoopTransition(fp: *mut Fighter) {
    melee_core::ct!("ftFox_SpecialN_FinishLoopTransition");
    unsafe {
        (*fp).accessory4_cb = Some(cb_ftFx_SpecialN_CreateBlasterShot);
        (*fp).mv.fx.SpecialN.isBlasterLoop = 0;
        ext::it_802ADDD0((*fp).u.fx.x222C_blasterGObj, 1);
    }
}

/// `ftFox_SpecialN_FinishEndTransition` (static inline).
pub fn ftFox_SpecialN_FinishEndTransition(fp: *mut Fighter) {
    melee_core::ct!("ftFox_SpecialN_FinishEndTransition");
    unsafe {
        let blaster_gobj = (*fp).u.fx.x222C_blasterGObj;
        (*fp).cmd_vars[1] = 1;
        ext::it_802ADDD0(blaster_gobj, 1);
    }
}

/// The tail of both Loop_Anim functions: fire the shot the script asked for.
unsafe fn loop_anim_fire(gobj: *mut HSD_GObj) {
    let fp = fpp(gobj);
    let da = da(fp);
    let mut sp2c = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    if (*fp).cmd_vars[2] != 0 {
        (*fp).cmd_vars[2] = 0;
        let launch_angle = ftFox_SpecialN_PrepareBlasterShot(gobj, fp, da, &mut sp2c);
        ftFox_SpecialN_FireBlasterShot(gobj, fp, da, &mut sp2c, launch_angle);
    }
}

pub fn ftFx_SpecialNLoop_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialNLoop_Anim");
    unsafe {
        let fp = fpp(gobj);

        ftFox_SpecialN_UpdateBlaster(fp);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            if (*fp).mv.fx.SpecialN.isBlasterLoop == 1 {
                ftFox_SpecialN_BeginLoopTransition(gobj, fp, ftFx_MS_SpecialNLoop as i32);
                ftFox_SpecialN_FinishLoopTransition(fp);
            } else {
                st::Fighter_ChangeMotionState(
                    gobj,
                    ftFx_MS_SpecialNEnd as i32,
                    (Ft_MF_SkipModel | Ft_MF_KeepGfx) as u32,
                    0.0,
                    1.0,
                    0.0,
                    core::ptr::null_mut(),
                );
                ftFox_SpecialN_FinishEndTransition(fp);
            }
            set_damage_callback(gobj, Some(cb_ftFx_Init_800E5588));
        }
        loop_anim_fire(gobj);
    }
}

/// `ftFox_SpecialN_UpdateEndAnimation` (static inline): true while frames remain.
pub fn ftFox_SpecialN_UpdateEndAnimation(gobj: *mut HSD_GObj, fp: *mut Fighter) -> bool {
    melee_core::ct!("ftFox_SpecialN_UpdateEndAnimation");
    unsafe {
        ext::it_802ADDD0((*fp).u.fx.x222C_blasterGObj, (*fp).cmd_vars[1] as i32);
        if (*fp).cmd_vars[1] == 2 {
            ext::ftpickupitem_80094818(gobj, false);
        }
        if (*fp).cmd_vars[3] == 2 && !(*fp).u.fx.x222C_blasterGObj.is_null() {
            (*fp).cmd_vars[3] = 0;
            ext::it_802AE608((*fp).u.fx.x222C_blasterGObj);
        }
        st::ftAnim_IsFramesRemaining(gobj) != 0
    }
}

pub fn ftFx_SpecialNEnd_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialNEnd_Anim");
    unsafe {
        let fp = fpp(gobj);
        if !ftFox_SpecialN_UpdateEndAnimation(gobj, fp) {
            ftFox_SpecialN_RemoveBlasterNULL(gobj);
            st::ft_8008A2BC(gobj);
        }
    }
}

pub fn ftFx_SpecialAirNStart_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialAirNStart_Anim");
    ftFox_SpecialN_StartAnimation(gobj, ftFx_MS_SpecialAirNLoop as i32);
}

pub fn ftFx_SpecialAirNLoop_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialAirNLoop_Anim");
    unsafe {
        let fp = fpp(gobj);

        ftFox_SpecialN_UpdateBlaster(fp);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            if (*fp).mv.fx.SpecialN.isBlasterLoop == 1 {
                ftFox_SpecialN_BeginLoopTransition(gobj, fp, ftFx_MS_SpecialAirNLoop as i32);
                set_damage_callback(gobj, Some(cb_ftFx_Init_800E5588));
                ftFox_SpecialN_FinishLoopTransition(fp);
            } else {
                st::Fighter_ChangeMotionState(
                    gobj,
                    ftFx_MS_SpecialAirNEnd as i32,
                    (Ft_MF_SkipModel | Ft_MF_KeepGfx) as u32,
                    0.0,
                    1.0,
                    0.0,
                    core::ptr::null_mut(),
                );
                set_damage_callback(gobj, Some(cb_ftFx_Init_800E5588));
                ftFox_SpecialN_FinishEndTransition(fp);
            }
            set_damage_callback(gobj, Some(cb_ftFx_Init_800E5588));
        }
        loop_anim_fire(gobj);
    }
}

pub fn ftFx_SpecialAirNEnd_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialAirNEnd_Anim");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);

        if !ftFox_SpecialN_UpdateEndAnimation(gobj, fp) {
            ftFox_SpecialN_RemoveBlasterNULL(gobj);
            if 0.0 == (*da).x18_FOX_BLASTER_LANDING_LAG {
                st::ftCo_Fall_Enter(gobj);
                return;
            }
            st::ftCo_80096900(gobj, 1, 0, 1, 1.0, (*da).x18_FOX_BLASTER_LANDING_LAG);
        }
    }
}

/// `ftFox_SpecialN_CheckLoopInput` (ftFox/inlines.h): a B press during the loop window.
pub fn ftFox_SpecialN_CheckLoopInput(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFox_SpecialN_CheckLoopInput");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).cmd_vars[0] != 0 && ((*fp).input.pressed_buttons as i32 & HSD_PAD_B) != 0 {
            (*fp).mv.fx.SpecialN.isBlasterLoop = 1;
        }
    }
}

pub fn ftFx_SpecialNStart_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialNStart_IASA");
    ftFox_SpecialN_CheckLoopInput(gobj);
}

pub fn ftFx_SpecialNLoop_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialNLoop_IASA");
    ftFox_SpecialN_CheckLoopInput(gobj);
}

pub fn ftFx_SpecialAirNStart_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialAirNStart_IASA");
    ftFox_SpecialN_CheckLoopInput(gobj);
}

pub fn ftFx_SpecialAirNLoop_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialAirNLoop_IASA");
    ftFox_SpecialN_CheckLoopInput(gobj);
}

/// Blaster throw animation (Fox / Falco only): spawn, show, throw or drop the blaster per the script.
pub fn ftFx_Throw_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_Throw_Anim");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        let kind = ftGetTemplateKind(&*fp);

        if kind == Ft_Kind_Fox as i32 || kind == Ft_Kind_Falco as i32 {
            if st::ftAnim_IsFramesRemaining(gobj) == 0 {
                (*fp).u.fx.x222C_blasterGObj = core::ptr::null_mut();
                set_damage_callback(gobj, None);
                return;
            }
            match (*fp).cmd_vars[1] {
                1 => {
                    if (*fp).u.fx.x222C_blasterGObj.is_null() {
                        let bone = ext::ftParts_GetBoneIndex(fp, FtPart_RThumbNb as i32);
                        let blaster = ext::it_802AE8A8(
                            (*fp).facing_dir,
                            gobj,
                            &(*fp).cur_pos,
                            bone,
                            (*da).x20_FOX_BLASTER_GUN_ITKIND,
                        );
                        (*fp).u.fx.x222C_blasterGObj = blaster;
                        if !blaster.is_null() {
                            ext::it_8026BAE8(
                                (*fp).u.fx.x222C_blasterGObj,
                                0.85_f32 * ((*fp).x34_scale.y * (*fp).co_attrs.model_scaling),
                            );
                            set_damage_callback(gobj, Some(cb_ftFx_Init_800E5588));
                        }
                        return;
                    }
                    ext::it_802ADDD0((*fp).u.fx.x222C_blasterGObj, (*fp).cmd_vars[1] as i32);
                    match (*fp).cmd_vars[3] {
                        1 => {
                            (*fp).cmd_vars[3] = 0;
                            ext::it_802AE538((*fp).u.fx.x222C_blasterGObj);
                        }
                        2 => {
                            (*fp).cmd_vars[3] = 0;
                            ext::it_802AE608((*fp).u.fx.x222C_blasterGObj);
                        }
                        _ => {}
                    }
                    if ftCheckThrowB0(fp) {
                        let mut sp50 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
                        let mut sp44 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

                        ftFx_SpecialN_FtGetHoldJoint(gobj, &mut sp50);
                        ftFx_SpecialN_ItGetHoldJoint(gobj, &mut sp44);

                        sp44.z = 0.0;
                        sp50.z = 0.0;
                        let action = (*fp).motion_id as i32;
                        let is_throw =
                            action == ftCo_MS_ThrowB as i32 || action == ftCo_MS_ThrowHi as i32 || action == ftCo_MS_ThrowLw as i32;
                        let angle = atan2f(sp50.y - sp44.y, sp50.x - sp44.x);
                        if is_throw {
                            ext::it_8029C6CC(angle, (*da).x14_FOX_BLASTER_VEL, gobj, &mut sp50, (*da).x1C_FOX_BLASTER_SHOT_ITKIND);
                        } else {
                            ext::it_8029C6A4(angle, (*da).x14_FOX_BLASTER_VEL, gobj, &mut sp50, (*da).x1C_FOX_BLASTER_SHOT_ITKIND);
                        }
                        ext::it_802AE1D0((*fp).u.fx.x222C_blasterGObj);

                        let lib_kind = ext::ftLib_GetKind(gobj);
                        if action == ftCo_MS_ThrowHi as i32 || action == ftCo_MS_ThrowLw as i32 {
                            if lib_kind == Ft_Kind_Fox {
                                ext::ft_PlaySFX(fp, 110109, SFX_VOLUME_MAX, SFX_PAN_MID);
                            } else if lib_kind == Ft_Kind_Falco {
                                ext::ft_PlaySFX(fp, 100105, SFX_VOLUME_MAX, SFX_PAN_MID);
                            }
                        } else if action == ftCo_MS_ThrowB as i32 {
                            let idx = (1.0 == (*fp).facing_dir) as usize;
                            if lib_kind == Ft_Kind_Fox {
                                ext::ft_PlaySFX(fp, foxSFX[idx] as i32, SFX_VOLUME_MAX, SFX_PAN_MID);
                            } else if lib_kind == Ft_Kind_Falco {
                                ext::ft_PlaySFX(fp, falcoSFX[idx] as i32, SFX_VOLUME_MAX, SFX_PAN_MID);
                            }
                        }
                    }
                }
                2 => {
                    if !(*fp).u.fx.x222C_blasterGObj.is_null() {
                        (*fp).u.fx.x222C_blasterGObj = core::ptr::null_mut();
                        set_damage_callback(gobj, None);
                        match ext::ftLib_GetKind(gobj) {
                            k if k == Ft_Kind_Fox => ext::ft_PlaySFX(fp, 110100, SFX_VOLUME_MAX, SFX_PAN_MID),
                            k if k == Ft_Kind_Falco => ext::ft_PlaySFX(fp, 100096, SFX_VOLUME_MAX, SFX_PAN_MID),
                            _ => {}
                        }
                    }
                }
                0 => {
                    (*fp).u.fx.x222C_blasterGObj = core::ptr::null_mut();
                    set_damage_callback(gobj, None);
                    ext::ftpickupitem_80094818(gobj, false);
                }
                _ => {}
            }
        }
    }
}
