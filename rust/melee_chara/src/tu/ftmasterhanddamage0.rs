//! src/melee/ft/kinds/ftMasterHand/ftmasterhanddamage0.c: Master Hand damage states (the camera
//! anchor callback, the damage entry, the two damage Anim callbacks). No float arithmetic (no
//! fused sites); the only float->int conversions are `unk0.x8 = da->x15C` (saturating, like the
//! C build). Smoke-tested by tests/tu_ftmasterhanddamage0.rs.
//!
//! Native layout caveat (same as ftcrazyhandpoke): `mv.mh.dmg0.x34..x40` are 8-byte pointers here
//! and 4 bytes on the GC. They are only freed and then NULLed inside one function, and the
//! neighbouring ids (`x28/x2C/x30`) sit below the first slot, so the C's end state is unchanged.
//!
//! Second caveat of the same kind: `HSD_GObjEvent unk0.x4` is 8 bytes here, so every `unk0` field
//! after it sits 4 bytes later than on the GC and `unk0.x20` (the "captured fighter" flag read in
//! `func_80151484_inline1`) overlays `dmg0.x28` (a sound handle). The port reads both by name, as
//! the C does; the overlap needs the melee_types layout fixed, not this TU.
//!
//! This file also hosts the hooks shared by the ftmasterhand* TUs of this shard (`ext`).
#![allow(non_snake_case, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_types::*;

/// Like `ch_hook!`, but an unset hook falls back to the verified port `$port` (when one exists in
/// melee_fighter) instead of panicking.
#[macro_export]
macro_rules! mh_hook_or {
    ($hook:ident, $name:ident, $port:path, ($($a:ident: $t:ty),*) $(-> $r:ty)?) => {
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

/// Calls into TUs that are not ported yet (or ported in a crate this one reaches through a hook).
/// Each is a settable hook; unset it panics, or runs the melee_fighter port where noted.
/// The hooks shared by the ftmasterhand* TUs of this shard are defined here once.
pub mod ext {
    use melee_types::*;
    pub static mut LB_8000B1CC: Option<fn(*mut HSD_JObj, *const Vec3, *mut Vec3)> = None;
    #[allow(non_snake_case)]
    pub fn lb_8000B1CC(jobj: *mut HSD_JObj, pos0: *const Vec3, pos1: *mut Vec3) {
        match unsafe { LB_8000B1CC } {
            Some(f) => f(jobj, pos0, pos1),
            None => { unsafe { let _ = melee_runtime::tu::lb_00B0::lb_8000B1CC(jobj as _, pos0 as _, pos1 as _); } },
        }
    }
    pub static mut LBAUDIOAX_800236B8: Option<fn(i32) -> i32> = None;
    #[allow(non_snake_case)]
    pub fn lbAudioAx_800236B8(id: i32) -> i32 {
        match unsafe { LBAUDIOAX_800236B8 } {
            Some(f) => f(id),
            None => { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_800236B8(id as _) as _ } },
        }
    }
    pub static mut LBAUDIOAX_8002438C: Option<fn(i32) -> i32> = None;
    #[allow(non_snake_case)]
    pub fn lbAudioAx_8002438C(id: i32) -> i32 {
        match unsafe { LBAUDIOAX_8002438C } {
            Some(f) => f(id),
            None => { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_8002438C(id as _) as _ } },
        }
    }
    pub static mut IT_802F046C: Option<fn(*mut Item_GObj)> = None;
    #[allow(non_snake_case)]
    pub fn it_802F046C(gobj: *mut Item_GObj) {
        match unsafe { IT_802F046C } {
            Some(f) => f(gobj),
            None => { unsafe { let _ = melee_items::tu::itmasterhandlaser::it_802F046C(gobj as _); } },
        }
    }
    // `ftMh_MS_389_80151018` (ftmasterhandwait12.c, not ported).
    pub static mut FTMH_MS_389_80151018: Option<fn(*mut HSD_GObj)> = None;
    #[allow(non_snake_case)]
    pub fn ftMh_MS_389_80151018(gobj: *mut HSD_GObj) {
        match unsafe { FTMH_MS_389_80151018 } {
            Some(f) => f(gobj),
            None => { unsafe { let _ = crate::tu::ftmasterhandwait12::ftMh_MS_389_80151018(gobj as _); } },
        }
    }
    // `ftMh_CaptureWaitMasterHand_80155D1C` (ftmasterhandthrownmasterhand.c, not ported).
    pub static mut FTMH_CAPTUREWAITMASTERHAND_80155D1C: Option<fn(*mut HSD_GObj)> = None;
    #[allow(non_snake_case)]
    pub fn ftMh_CaptureWaitMasterHand_80155D1C(gobj: *mut HSD_GObj) {
        match unsafe { FTMH_CAPTUREWAITMASTERHAND_80155D1C } {
            Some(f) => f(gobj),
            None => { unsafe { let _ = crate::tu::ftmasterhandthrownmasterhand::ftMh_CaptureWaitMasterHand_80155D1C(gobj as _); } },
        }
    }

    fn ft_800881D8_port(fp: *mut Fighter, sfx_id: i32, vol: u8, pan: u8) {
        melee_fighter::ft0881::ft_800881d8(unsafe { &mut *fp }, sfx_id, vol, pan)
    }
    fn ftBossLib_GetFighterGObj_port(kind: FighterKind) -> *mut HSD_GObj {
        melee_fighter::tu::ftbosslib::ftBossLib_GetFighterGObj(kind)
    }
    fn ftCo_800D4F24_port(gobj: *mut HSD_GObj, index: i32) {
        melee_fighter::tu::ft_0d4d::ftCo_800D4F24(gobj, index)
    }
    // Falls back to melee_fighter's port.
    crate::mh_hook_or!(FT_800881D8, ft_800881D8, ft_800881D8_port, (fp: *mut Fighter, sfx_id: i32, vol: u8, pan: u8));
    // Falls back to melee_fighter's port.
    crate::mh_hook_or!(FTBOSSLIB_GETFIGHTERGOBJ, ftBossLib_GetFighterGObj, ftBossLib_GetFighterGObj_port, (kind: FighterKind) -> *mut HSD_GObj);
    // Falls back to melee_fighter's port.
    crate::mh_hook_or!(FTCO_800D4F24, ftCo_800D4F24, ftCo_800D4F24_port, (gobj: *mut HSD_GObj, index: i32));
}

/// Camera anchor callback (`Camera_8002E948` argument): the Master Hand's waist position.
pub unsafe extern "C" fn ftMh_MS_343_80151428(vec: *mut Vec3) -> bool {
    unsafe {
        let gobj = ext::ftBossLib_GetFighterGObj(Ft_Kind_MasterH);
        if !gobj.is_null() {
            let fp = (*gobj).user_data as *mut Fighter;
            ext::lb_8000B1CC((*(*fp).parts.add(FtPart_WaistN as usize)).joint, core::ptr::null(), vec);
            return true;
        }
        false
    }
}

/// `func_80151484_inline1` (static inline).
unsafe fn func_80151484_inline1(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let da = (*(*fp).ft_data).ext_attr as *const ftMasterHand_SpecialAttrs;
        (*fp).self_vel.x = 0.0;
        (*fp).self_vel.y = 0.0;
        (*fp).self_vel.z = 0.0;
        ext::it_802F046C((*fp).mv.mh.dmg0.x34);
        ext::it_802F046C((*fp).mv.mh.dmg0.x38);
        ext::it_802F046C((*fp).mv.mh.dmg0.x3C);
        ext::it_802F046C((*fp).mv.mh.dmg0.x40);
        (*fp).mv.mh.dmg0.x34 = core::ptr::null_mut();
        (*fp).mv.mh.dmg0.x38 = core::ptr::null_mut();
        (*fp).mv.mh.dmg0.x3C = core::ptr::null_mut();
        (*fp).mv.mh.dmg0.x40 = core::ptr::null_mut();
        ext::lbAudioAx_800236B8((*fp).mv.mh.dmg0.x28);
        ext::lbAudioAx_800236B8((*fp).mv.mh.dmg0.x2C);
        ext::lbAudioAx_800236B8((*fp).mv.mh.dmg0.x30);
        if (*fp).mv.mh.unk0.x20 != 0 {
            ext::ftMh_CaptureWaitMasterHand_80155D1C((*fp).victim_gobj as *mut HSD_GObj);
        }
        (*fp).self_vel.y = (*da).x14C;
        (*fp).self_vel.z = (*da).x154;
        (*fp).mv.mh.unk0.x8 = (*da).x15C as i32;
    }
}

pub fn ftMh_MS_343_80151484(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        (*fp).cmd_vars[0] = 0;
        func_80151484_inline1(gobj);
        st::Fighter_ChangeMotionState(gobj, ftMh_MS_Damage as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        ext::ft_800881D8(fp, 320023, 127, 64);
        melee_fighter::hooks::play_sfx(&mut *fp, 320024, 127, 64);
        (*fp).mv.mh.unk0.x74 = 0;
        (*fp).mv.mh.unk0.x78 = 0;
    }
}

/// Shared body of `ftMh_Damage_Anim` / `ftMh_MS_345_Anim` (the two C bodies are identical).
unsafe fn damage_anim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        if (*fp).cmd_vars[0] != 0 {
            let da = (*(*fp).ft_data).ext_attr as *const ftMasterHand_SpecialAttrs;
            (*fp).mv.mh.unk0.x74 = (*fp).mv.mh.unk0.x74.wrapping_add(1);
            if (*fp).mv.mh.unk0.x74 <= (*da).x144 {
                ext::lbAudioAx_8002438C(530002);
            } else {
                (*fp).mv.mh.unk0.x78 = (*fp).mv.mh.unk0.x78.wrapping_add(1);
                if (*fp).mv.mh.unk0.x78 <= (*da).x148 {
                    ext::lbAudioAx_8002438C(530003);
                }
            }
            (*fp).cmd_vars[0] = 0;
        }
        if (*fp).mv.mh.unk0.x8 > 0 && st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::Fighter_ChangeMotionState(gobj, ftMh_MS_Damage2 as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
            st::ftAnim_8006EBA4(gobj);
        }
        (*fp).mv.mh.unk0.x8 = (*fp).mv.mh.unk0.x8.wrapping_sub(1);
        if (*fp).mv.mh.unk0.x8 == 0 {
            ext::ftCo_800D4F24(gobj, 0);
        }
    }
}

pub fn ftMh_Damage_Anim(gobj: *mut HSD_GObj) {
    unsafe { damage_anim(gobj) }
}

pub fn ftMh_MS_345_Anim(gobj: *mut HSD_GObj) {
    unsafe { damage_anim(gobj) }
}
