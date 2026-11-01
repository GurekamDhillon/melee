
//! Hand port, round 3: src/melee/ft/ftlib.c, the fighter accessor library other code calls
#![allow(non_upper_case_globals)]

//! with a fighter `HSD_GObj*`. Functions keep the C signature shape: `g` is the raw gobj
//! (`G = *mut HSD_GObj`), `gobj->user_data` is the `Fighter`. Callers must pass a live fighter
//! gobj, as in C (the C dereferences without checks too).
//!
//! The fighter list (`HSD_GObjPLinkHead[HSD_GOBJ_PLINK_FIGHTER]`) comes from
//! `hooks::plink_head`. Calls out of the ported set go through `crate::hooks`.
//! Fused sites (`tools/rust-port/fma_sites.py src/melee/ft/ftlib.c`): ftLib_8008627C and
//! ftLib_80086368, one f32 fmuladd each (`dx*dx + dy*dy` -> `fmadds(dx, dx, dy*dy)`).
//! TARGET_PC paths are the ones ported (80086A8C returns true; Slippi Whispy fix in 800864A8;
//! Geno template kinds in GetKind; m-ex crowd sfx in 8008746C; UpdateLogicOffCamera).
//!
//! Not ported from ftlib.c: ftLib_800871A8 (metal box: ftCo/it/camera), ftLib_80087508 /
//! 80087574 / 80087610 (archive preloading), ftLib_8008777C (atan2f; depends on group 1's MSL).
//! Checked against the C (cut verbatim, shipping contraction) by tests/ftlib_ref.rs.

use crate::ftcommon2 as c2;

use crate::hooks as h;

use crate::types::*;

use melee_core::fp::fmadds;

pub type G = *mut HSD_GObj;

#[inline(always)]
pub(crate) fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

/// Iterate the fighter p-link list like the C `for (cur = head; cur; cur = cur->next)`.
/// `next` is read before calling `f` (the C reads it after; no ported body unlinks).
pub(crate) fn each(mut f: impl FnMut(G) -> bool) -> G {
    let mut cur = h::plink_head(HSD_GOBJ_PLINK_FIGHTER);
    while !cur.is_null() {
        if f(cur) {
            return cur;
        }
        cur = unsafe { (*cur).next };
    }
    core::ptr::null_mut()
}

pub fn is_master_hand_present() -> bool {
    !each(|c| get_kind(c) == Ft_Kind_MasterH).is_null()
}

pub fn is_crazy_hand_present() -> bool {
    !each(|c| get_kind(c) == Ft_Kind_CrezyH).is_null()
}

#[inline]
pub(crate) fn skip_opponent(fp_own: Option<&Fighter>, cur: &Fighter) -> bool {
    cur.x221F_b3() != 0 || (h::gm_8016b168() && fp_own.is_some_and(|f| cur.team == f.team))
}

pub fn set_scale(g: G, val: f32) {
    fp(g).x34_scale.y = val;
    h::fighter_update_model_scale(g);
    c2::ftcommon_80080174(fp(g));
}

pub fn get_item(g: G) -> G { fp(g).item_gobj }

/// `ftLib_UpdateLogicOffCamera` (TARGET_PC).
pub fn update_logic_off_camera(g: G) {
    let f = fp(g);
    if f.x221F_b3() != 0 {
        return;
    }
    if f.x2229_b3() == 0 && f.x2220_b7() == 0 && !h::camera_80030a50().is_null() {
        let b = !h::camera_logic_to_screen(f.x890_cameraBox, &mut f.x2188);
        f.set_x221F_b0(b as _);
    } else {
        f.set_x221F_b0(0);
    }
}

pub fn get_motion_id(g: G) -> i32 { fp(g).motion_id as i32 }

#[inline]
pub(crate) fn rumble_helper(g: G, arg1: i32, arg2: i32, val: i32) {
    let f = fp(g);
    if !h::player_8003544c(f.player_id as i32, f.is_sub_fighter() as i32) {
        return;
    }
    if f.x221F_b3() == 0 && f.x2224_b2() == 0 {
        h::lb_rumble_start(f.x618_player_id, val, arg1, arg2);
    }
}

/// `ftLib_GetKind` (TARGET_PC): a Geno define on a template skeleton answers as its base kind.
pub fn get_kind(g: G) -> FighterKind {
    let k = fp(g).kind;
    if k >= Ft_Kind_Mex0 && h::geno_define_template_skeleton(k as i32) != 0 {
        return h::geno_define_base_kind(k as i32) as FighterKind;
    }
    k
}

#[inline]
pub(crate) fn msid_in(g: G, lo: i32, hi: i32) -> bool {
    let m = get_motion_id(g);
    m >= lo && m <= hi
}

pub use crate::tu::ftlib::ftlib_800860c4;
pub use crate::tu::ftlib::ftlib_80086198;
pub use crate::tu::ftlib::ftlib_8008627c;
pub use crate::tu::ftlib::ftlib_80086368;
pub use crate::tu::ftlib::ftlib_800864a8;
pub use crate::tu::ftlib::ftlib_800865c0;
pub use crate::tu::ftlib::ftlib_800865cc;
pub use crate::tu::ftlib::ftlib_800865d8;
pub use crate::tu::ftlib::ftlib_800865f0;
pub use crate::tu::ftlib::ftlib_80086630;
pub use crate::tu::ftlib::ftlib_80086644;
pub use crate::tu::ftlib::ftlib_80086664;
pub use crate::tu::ftlib::ftlib_80086684;
pub use crate::tu::ftlib::ftlib_800866dc;
pub use crate::tu::ftlib::ftlib_80086724;
pub use crate::tu::ftlib::ftlib_80086764;
pub use crate::tu::ftlib::ftlib_800867a0;
pub use crate::tu::ftlib::ftlib_800867cc;
pub use crate::tu::ftlib::ftlib_800867d8;
pub use crate::tu::ftlib::ftlib_800867e8;
pub use crate::tu::ftlib::ftlib_80086824;
pub use crate::tu::ftlib::ftlib_8008688c;
pub use crate::tu::ftlib::ftlib_800868a4;
pub use crate::tu::ftlib::ftlib_800868d4;
pub use crate::tu::ftlib::ftlib_80086960;
pub use crate::tu::ftlib::ftlib_80086984;
pub use crate::tu::ftlib::ftlib_80086990;
pub use crate::tu::ftlib::ftlib_800869d4;
pub use crate::tu::ftlib::ftlib_800869f8;
pub use crate::tu::ftlib::ftlib_80086a0c;
pub use crate::tu::ftlib::ftlib_80086a18;
pub use crate::tu::ftlib::ftlib_80086a4c;
pub use crate::tu::ftlib::ftlib_80086a58;
pub use crate::tu::ftlib::ftlib_80086a8c;
pub use crate::tu::ftlib::ftlib_80086b64;
pub use crate::tu::ftlib::ftlib_80086b74;
pub use crate::tu::ftlib::ftlib_80086b80;
pub use crate::tu::ftlib::ftlib_80086b90;
pub use crate::tu::ftlib::ftlib_80086bb4;
pub use crate::tu::ftlib::ftlib_80086be0;
pub use crate::tu::ftlib::ftlib_80086bec;
pub use crate::tu::ftlib::ftlib_80086c18;
pub use crate::tu::ftlib::ftlib_80086c9c;
pub use crate::tu::ftlib::ftlib_80086d40;
pub use crate::tu::ftlib::ftlib_80086dc4;
pub use crate::tu::ftlib::ftlib_80086e68;
pub use crate::tu::ftlib::ftlib_80086eb4;
pub use crate::tu::ftlib::ftlib_80086ec0;
pub use crate::tu::ftlib::ftlib_80086ed0;
pub use crate::tu::ftlib::ftlib_80086f4c;
pub use crate::tu::ftlib::ftlib_80086f80;
pub use crate::tu::ftlib::ftlib_80086fa8;
pub use crate::tu::ftlib::ftlib_80086fd4;
pub use crate::tu::ftlib::ftlib_8008701c;
pub use crate::tu::ftlib::ftlib_8008702c;
pub use crate::tu::ftlib::ftlib_80087050;
pub use crate::tu::ftlib::ftlib_80087074;
pub use crate::tu::ftlib::ftlib_800870bc;
pub use crate::tu::ftlib::ftlib_800870f0;
pub use crate::tu::ftlib::ftlib_80087120;
pub use crate::tu::ftlib::ftlib_80087140;
pub use crate::tu::ftlib::ftlib_80087284;
pub use crate::tu::ftlib::ftlib_800872b0;
pub use crate::tu::ftlib::ftlib_800872bc;
pub use crate::tu::ftlib::ftlib_80087300;
pub use crate::tu::ftlib::ftlib_8008730c;
pub use crate::tu::ftlib::ftlib_8008731c;
pub use crate::tu::ftlib::ftlib_8008732c;
pub use crate::tu::ftlib::ftlib_80087354;
pub use crate::tu::ftlib::ftlib_8008737c;
pub use crate::tu::ftlib::ftlib_800873a4;
pub use crate::tu::ftlib::ftlib_800873cc;
pub use crate::tu::ftlib::ftlib_800873f4;
pub use crate::tu::ftlib::ftlib_8008741c;
pub use crate::tu::ftlib::ftlib_80087454;
pub use crate::tu::ftlib::ftlib_80087460;
pub use crate::tu::ftlib::ftlib_8008746c;
pub use crate::tu::ftlib::ftlib_800874bc;
pub use crate::tu::ftlib::ftlib_800874cc;
pub use crate::tu::ftlib::ftlib_800876b4;
pub use crate::tu::ftlib::ftlib_800876d4;
pub use crate::tu::ftlib::ftlib_800876f4;
pub use crate::tu::ftlib::ftlib_80087700;
pub use crate::tu::ftlib::ftlib_8008770c;
pub use crate::tu::ftlib::ftlib_80087744;
pub use crate::tu::ftlib::ftlib_800877d4;
