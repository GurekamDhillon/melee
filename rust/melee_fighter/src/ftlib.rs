
//! Hand port, round 3: src/melee/ft/ftlib.c, the fighter accessor library other code calls
#![allow(non_upper_case_globals)]

//! with a fighter `HSD_GObj*`. Functions keep the C signature shape: `g` is the raw gobj
//! (`G = *mut HSD_GObj`), `gobj->user_data` is the `Fighter`. Callers must pass a live fighter
//! gobj, as in C (the C dereferences without checks too).
//!
//! The fighter list (`HSD_GObjPLinkHead[HSD_GOBJ_PLINK_FIGHTER]`) comes from
//! `hooks::plink_head`. Calls out of the ported set go through `crate::hooks`.
//! Fused sites (`tools/rust-port/fma_sites.py src/melee/ft/ftlib.c`): ftLib_FindNearestOpponent and
//! ftLib_FindNearestOpponentInDir, one f32 fmuladd each (`dx*dx + dy*dy` -> `fmadds(dx, dx, dy*dy)`).
//! TARGET_PC paths are the ones ported (80086A8C returns true; Slippi Whispy fix in 800864A8;
//! Geno template kinds in GetKind; m-ex crowd sfx in 8008746C; UpdateLogicOffCamera).
//!
//! Not ported from ftlib.c: ftLib_ApplyMetalBox (metal box: ftCo/it/camera), ftLib_LoadFighterCostume /
//! 80087574 / 80087610 (archive preloading), ftLib_GetGroundSlopeAngle (atan2f; depends on group 1's MSL).
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

pub use crate::tu::ftlib::ftlib_countfighters;
pub use crate::tu::ftlib::ftlib_findlowestpercentopponent;
pub use crate::tu::ftlib::ftlib_findnearestopponent;
pub use crate::tu::ftlib::ftlib_findnearestopponentindir;
pub use crate::tu::ftlib::ftlib_getopponentsdir;
pub use crate::tu::ftlib::ftlib_getfacingdir;
pub use crate::tu::ftlib::ftlib_getgroundair;
pub use crate::tu::ftlib::ftlib_getlstick;
pub use crate::tu::ftlib::ftlib_gethipjobj;
pub use crate::tu::ftlib::ftlib_getpartjobj;
pub use crate::tu::ftlib::ftlib_getpos;
pub use crate::tu::ftlib::ftlib_setpos;
pub use crate::tu::ftlib::ftlib_getprevpos;
pub use crate::tu::ftlib::ftlib_getcamerabonepos;
pub use crate::tu::ftlib::ftlib_releaseitem;
pub use crate::tu::ftlib::ftlib_clearheldspecialitem;
pub use crate::tu::ftlib::ftlib_isholdingitem;
pub use crate::tu::ftlib::ftlib_getheldspecialitem;
pub use crate::tu::ftlib::ftlib_isinputdisabled;
pub use crate::tu::ftlib::ftlib_disableinput;
pub use crate::tu::ftlib::ftlib_disableallinput;
pub use crate::tu::ftlib::ftlib_enableinput;
pub use crate::tu::ftlib::ftlib_enableallinput;
pub use crate::tu::ftlib::ftlib_isitemvisible;
pub use crate::tu::ftlib::ftlib_isfighter;
pub use crate::tu::ftlib::ftlib_getcolldata;
pub use crate::tu::ftlib::ftlib_getecbcenter;
pub use crate::tu::ftlib::ftlib_getmodelscale;
pub use crate::tu::ftlib::ftlib_800869f8;
pub use crate::tu::ftlib::ftlib_getscale;
pub use crate::tu::ftlib::ftlib_isshielding;
pub use crate::tu::ftlib::ftlib_80086a4c;
pub use crate::tu::ftlib::ftlib_getscreenpos;
pub use crate::tu::ftlib::ftlib_updatescreenvisibility;
pub use crate::tu::ftlib::ftlib_isoffscreen;
pub use crate::tu::ftlib::ftlib_getcamerasubject;
pub use crate::tu::ftlib::ftlib_80086b80;
pub use crate::tu::ftlib::ftlib_getcamerasubjectbonepos;
pub use crate::tu::ftlib::ftlib_iscamerasubjectinbounds;
pub use crate::tu::ftlib::ftlib_getplayerindex;
pub use crate::tu::ftlib::ftlib_getposdelta;
pub use crate::tu::ftlib::ftlib_startrumble;
pub use crate::tu::ftlib::ftlib_startrumbleall;
pub use crate::tu::ftlib::ftlib_startitemrumble;
pub use crate::tu::ftlib::ftlib_startitemrumbleall;
pub use crate::tu::ftlib::ftlib_stopitemrumble;
pub use crate::tu::ftlib::ftlib_getteam;
pub use crate::tu::ftlib::ftlib_isinhitstun;
pub use crate::tu::ftlib::ftlib_canbemagnified;
pub use crate::tu::ftlib::ftlib_80086f4c;
pub use crate::tu::ftlib::ftlib_getnametagheight;
pub use crate::tu::ftlib::ftlib_isswordswing;
pub use crate::tu::ftlib::ftlib_issameplayer;
pub use crate::tu::ftlib::ftlib_issleeping;
pub use crate::tu::ftlib::ftlib_incfighterrefcount;
pub use crate::tu::ftlib::ftlib_decfighterrefcount;
pub use crate::tu::ftlib::ftlib_getshakeoffset;
pub use crate::tu::ftlib::ftlib_getsubcolor;
pub use crate::tu::ftlib::ftlib_setpercent;
pub use crate::tu::ftlib::ftlib_getpercent;
pub use crate::tu::ftlib::ftlib_80087140;
pub use crate::tu::ftlib::ftlib_80087284;
pub use crate::tu::ftlib::ftlib_getshadow;
pub use crate::tu::ftlib::ftlib_castsshadow;
pub use crate::tu::ftlib::ftlib_getlastattackerslot;
pub use crate::tu::ftlib::ftlib_islastattackersubfighter;
pub use crate::tu::ftlib::ftlib_issleeping_8008731c;
pub use crate::tu::ftlib::ftlib_isdead;
pub use crate::tu::ftlib::ftlib_isdeadup;
pub use crate::tu::ftlib::ftlib_isdeadupstar;
pub use crate::tu::ftlib::ftlib_isdeadupfall;
pub use crate::tu::ftlib::ftlib_isrebirth;
pub use crate::tu::ftlib::ftlib_isentry;
pub use crate::tu::ftlib::ftlib_findbyspawnnum;
pub use crate::tu::ftlib::ftlib_getcrowdknockback;
pub use crate::tu::ftlib::ftlib_getspawnnum;
pub use crate::tu::ftlib::ftlib_8008746c;
pub use crate::tu::ftlib::ftlib_issubfighter;
pub use crate::tu::ftlib::ftlib_800874cc;
pub use crate::tu::ftlib::ftlib_isframesremaining;
pub use crate::tu::ftlib::ftlib_ischargingsmash;
pub use crate::tu::ftlib::ftlib_getlasthitsourcetype;
pub use crate::tu::ftlib::ftlib_getlasthitsourcekind;
pub use crate::tu::ftlib::ftlib_getgamewatchcolor;
pub use crate::tu::ftlib::ftlib_getgamewatchoutlinecolor;
pub use crate::tu::ftlib::ftlib_isbeingnudged;
