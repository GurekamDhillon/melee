//! src/melee/ft/kinds/ftFox/ftfoxappeals.c: Fox / Falco's Corneria side taunt (the Great Fox
//! transmission). No float math (no fused sites; `order_sdata2` is `MUST_MATCH`-only, not built
//! on PC). Smoke-tested through recording hooks by tests/tu_ftfoxappeals.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_fighter::hooks as fh;
use melee_types::*;

/// Already ported (rounds 2, 12): re-exported under this TU.
pub use crate::chars::r12::ftFox::ftfoxappeals::ftFx_AppealS_IASA;
pub use crate::chars::r2::ftFox::ftfoxappeals::ftFx_AppealS_Phys;

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics, except the
/// ones with a verified port in melee_fighter, which the unset hook calls.
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
    macro_rules! hook_or {
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
    fn pl_800386d8_port(stats: *mut plActionStats, attack: i32) -> u32 {
        melee_fighter::plbonus3::pl_800386d8(unsafe { &*stats }, attack)
    }
    pub static mut PLAYER_GETACTIONSTATS: Option<fn(i32) -> *mut plActionStats> = None;
            #[allow(non_snake_case)]
            pub fn Player_GetActionStats(slot: i32) -> *mut plActionStats {
                match unsafe { PLAYER_GETACTIONSTATS } {
                    Some(f) => f(slot),
                    None => { return unsafe { melee_fighter::tu::player::Player_GetActionStats(slot as _) as _ }; },
                }
            }
    hook_or!(PL_800386D8, pl_800386D8, pl_800386d8_port, (stats: *mut plActionStats, attack: i32) -> u32);
    hook_or!(PL_80040120, pl_80040120, melee_fighter::plbonus::pl_80040120, (slot: i32, sub: i32));
    pub static mut GRCORNERIA_801E2AF4: Option<fn()> = None;
    #[allow(non_snake_case)]
    pub fn grCorneria_801E2AF4() {
        match unsafe { GRCORNERIA_801E2AF4 } {
            Some(f) => f(),
            None => { unsafe { let _ = melee_world::tu::grcorneria::grCorneria_801E2AF4(); } },
        }
    }
    pub static mut GRCORNERIA_801E2B80: Option<fn() -> bool> = None;
    #[allow(non_snake_case)]
    pub fn grCorneria_801E2B80() -> bool {
        match unsafe { GRCORNERIA_801E2B80 } {
            Some(f) => f(),
            None => { unsafe { melee_world::tu::grcorneria::grCorneria_801E2B80() != 0 } },
        }
    }
    pub static mut GRCORNERIA_801E2C34: Option<fn() -> bool> = None;
    #[allow(non_snake_case)]
    pub fn grCorneria_801E2C34() -> bool {
        match unsafe { GRCORNERIA_801E2C34 } {
            Some(f) => f(),
            None => { unsafe { melee_world::tu::grcorneria::grCorneria_801E2C34() != 0 } },
        }
    }
    pub static mut GRCORNERIA_801E2CE8: Option<fn() -> bool> = None;
    #[allow(non_snake_case)]
    pub fn grCorneria_801E2CE8() -> bool {
        match unsafe { GRCORNERIA_801E2CE8 } {
            Some(f) => f(),
            None => { unsafe { melee_world::tu::grcorneria::grCorneria_801E2CE8() != 0 } },
        }
    }
    pub static mut GRCORNERIA_801E2D14: Option<fn() -> bool> = None;
    #[allow(non_snake_case)]
    pub fn grCorneria_801E2D14() -> bool {
        match unsafe { GRCORNERIA_801E2D14 } {
            Some(f) => f(),
            None => { unsafe { melee_world::tu::grcorneria::grCorneria_801E2D14() != 0 } },
        }
    }
    hook_or!(FT_8008A324, ft_8008A324, melee_fighter::tu::ft_08a1::ft_8008A324, (gobj: *mut HSD_GObj));
    hook_or!(FT_80084280, ft_80084280, melee_fighter::tu::ft_081b::ft_80084280, (gobj: *mut HSD_GObj));
}

/// `FTFOX_APPEALS_ATTACKID`.
pub const FTFOX_APPEALS_ATTACKID: i32 = 0x72;

/// `ASID_AppealS` (static): [facing][animCount] -> motion state.
pub const ASID_AppealS: [[i32; 3]; 2] = [
    [ftFx_MS_AppealSStartR as i32, ftFx_MS_AppealSR as i32, ftFx_MS_AppealSEndR as i32],
    [ftFx_MS_AppealSStartL as i32, ftFx_MS_AppealSL as i32, ftFx_MS_AppealSEndL as i32],
];

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// `ftGetTemplateKind` (ft/inlines.h, TARGET_PC): the base kind of an m-ex / Geno fighter.
fn ftGetTemplateKind(fp: &Fighter) -> i32 {
    let kind = fp.kind as i32;
    if kind >= Ft_Kind_Mex0 as i32 && fh::geno_define_template_skeleton(kind) != 0 {
        return fh::geno_define_base_kind(kind);
    }
    kind
}

/// `ftCheckThrowB3` (ft/inlines.h): test and clear the flag.
fn ftCheckThrowB3(fp: &mut Fighter) -> bool {
    unsafe {
        if fp.anon_2210.anon_0.throw_flags_b3() != 0 {
            fp.anon_2210.anon_0.set_throw_flags_b3(0);
            true
        } else {
            false
        }
    }
}

/// C `bool`: this player has used the taunt and the stage's transmission is ready.
pub fn ftFx_AppealS_CheckIfUsed(fp: *mut Fighter) -> bool {
    melee_core::ct!("ftFx_AppealS_CheckIfUsed");
    unsafe {
        let attack_count = ext::Player_GetActionStats((*fp).player_id as i32);
        ext::pl_800386D8(attack_count, FTFOX_APPEALS_ATTACKID) != 0 && ext::grCorneria_801E2D14()
    }
}

/// `ftFox_CheckAppealSCount` (static inline): any of the six slots has used the taunt.
pub fn ftFox_CheckAppealSCount() -> bool {
    melee_core::ct!("ftFox_CheckAppealSCount");
    for i in 0..6 {
        let attack_count = ext::Player_GetActionStats(i);
        if ext::pl_800386D8(attack_count, 0x72) != 0 {
            return true;
        }
    }
    false
}

/// C `bool`: start the taunt when Fox / Falco is on Corneria and nobody used it yet.
pub fn ftFx_AppealS_CheckInput(gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("ftFx_AppealS_CheckInput");
    unsafe {
        let fp = fpp(gobj);
        let ft_kind = ftGetTemplateKind(&*fp);
        if (ft_kind == Ft_Kind_Fox as i32 || ft_kind == Ft_Kind_Falco as i32)
            && ext::grCorneria_801E2CE8()
            && ((*fp).input.held_buttons[0] & (HSD_PAD_DPADDOWN as u32)) == 0
            && (*fp).x682 == 1
            && !ftFox_CheckAppealSCount()
        {
            ftFx_AppealS_Enter(gobj);
            ext::pl_80040120((*fp).player_id as i32, (*fp).is_sub_fighter() as i32);
            return true;
        }
        false
    }
}

/// `ftFox_AppealS_GetLR` (static inline).
pub fn ftFox_AppealS_GetLR(x1: f32, x2: f32) -> bool {
    melee_core::ct!("ftFox_AppealS_GetLR");
    x1 != x2
}

pub fn ftFx_AppealS_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_AppealS_Enter");
    unsafe {
        let fp = fpp(gobj);
        (*fp).mv.fx.AppealS.animCount = 0;
        let facing_dir = ftFox_AppealS_GetLR(1.0, (*fp).facing_dir) as i32;
        (*fp).mv.fx.AppealS.facingDir = facing_dir;
        (*fp).anon_2210.throw_flags = 0;
        let action_dir = (*fp).mv.fx.AppealS.facingDir;
        let anim_count = (*fp).mv.fx.AppealS.animCount;
        st::Fighter_ChangeMotionState(
            gobj,
            ASID_AppealS[action_dir as usize][anim_count as usize],
            0,
            0.0,
            1.0,
            0.0,
            core::ptr::null_mut(),
        );
    }
}

/// `ftFx_AppealS_OnTakeDamage` as a Fighter callback entry (`death1_cb`).
pub unsafe extern "C" fn cb_ftFx_AppealS_OnTakeDamage(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftFx_AppealS_OnTakeDamage");
    ftFx_AppealS_OnTakeDamage(g);
}

pub fn ftFx_AppealS_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_AppealS_Anim");
    unsafe {
        let fp = fpp(gobj);
        if ftCheckThrowB3(&mut *fp) {
            match ftGetTemplateKind(&*fp) {
                k if k == Ft_Kind_Fox as i32 => {
                    if ext::grCorneria_801E2B80() {
                        (*fp).death1_cb = Some(cb_ftFx_AppealS_OnTakeDamage);
                    }
                }
                k if k == Ft_Kind_Falco as i32 => {
                    if ext::grCorneria_801E2C34() {
                        (*fp).death1_cb = Some(cb_ftFx_AppealS_OnTakeDamage);
                    }
                }
                _ => {}
            }
        }
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            (*fp).mv.fx.AppealS.animCount += 1;
            if (*fp).mv.fx.AppealS.animCount >= 3 {
                ext::ft_8008A324(gobj);
                return;
            }
            st::Fighter_ChangeMotionState(
                gobj,
                ASID_AppealS[(*fp).mv.fx.AppealS.facingDir as usize][(*fp).mv.fx.AppealS.animCount as usize],
                0,
                0.0,
                1.0,
                0.0,
                core::ptr::null_mut(),
            );
        }
    }
}

pub fn ftFx_AppealS_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_AppealS_Coll");
    ext::ft_80084280(gobj);
}

/// Fox & Falco's Special Taunt OnTakeDamage / OnDeath callback (static).
pub fn ftFx_AppealS_OnTakeDamage(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_AppealS_OnTakeDamage");
    unsafe {
        let fp = fpp(gobj);
        ext::grCorneria_801E2AF4();
        (*fp).death1_cb = None;
    }
}
