//! src/melee/gm/gmtou_2.c: tournament "alt" (vs setup) scene: player-card widgets, the name/CPU
//! character select, scene OnFrame/OnEnter/OnExit, and the preload-cache fills `fn_8019EE80` /
//! `fn_8019EF08`.
//!
//! Fused float sites (tools/rust-port/fma_sites.py): `fn_8019C048` (`13*x-19.5`, three shapes),
//! `fn_8019CDBC` (`5.999997*x-21.5`), `fn_8019D1BC` (`240*x+152`, `240*x+270`, `480*x+270`,
//! `0.99*(pos-320)-261`, `5.999997*x-21.5` twice) are `fmadds` below. `lbl_803B7D18` is stored
//! decoded (u16 triplets, the C initializer is explicit).
//!
//! `tmd->x534[i]` / `name_buf[i]` index past their arrays when more than 3 / 16 entrants exist
//! (the C writes past `TmData::x534` and its stack buffer); the buffers here are as large as
//! the biggest bracket (64) so that stays defined.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, clippy::all)]

use core::ffi::{c_char, c_void};
use core::ptr::{addr_of_mut, null_mut};

use melee_core::fp::fmadds;
use melee_types::inline::{HSD_JObjGetChild, HSD_JObjGetNext, HSD_JObjGetTranslationX, HSD_JObjGetTranslationY};
use melee_types::*;

use super::gmtou_1::{
    fn_80196510, fn_8019655C, fn_80196564, fn_80196594, fn_801965C4, gmTournament_GetPlayerX, gmTournament_SetPlayerX,
};
use super::gmtoulib::*;

pub mod ext {
    pub use super::super::gmtou_1::ext::*;
    use super::*;

    crate::gmtou_ext! {
        fn fn_80169000(arg0: *mut MatchEnd, arg1: *mut u8);
        fn HSD_SisLib_803A746C(arg0: *mut HSD_Text, arg1: i32, arg2: f32, arg3: f32);
    }
    /// `OSReport(fmt, ...)` with one integer variadic.
    pub unsafe extern "C" fn OSReport(fmt: *mut c_char, v: i32) {
        use super::super::gmtoulib::hook::{self, Word};
        hook::call("OSReport", &[fmt.to_w(), v.to_w()]);
    }
}

// ---------------------------------------------------------------------------------------------
// data
// ---------------------------------------------------------------------------------------------

static mut lbl_804D6688: *mut HSD_Archive = null_mut();
/// `lbl_80479A58`: per-scene timers (anim counters, per-player state records).
pub static mut lbl_80479A58: TmAnimTimers = unsafe { core::mem::zeroed() };
static mut lbl_804D668C: *mut HSD_Archive = null_mut();
static mut lbl_804D6690: *mut SceneDesc = null_mut();
static mut lbl_804D6694: *mut SceneDesc = null_mut();

/// `lbl_803DA2E0`: icon -> model map (x0, 25 initialised bytes) and the bounce curve (x20,
/// filled by `fn_8019D1BC`).
pub static mut lbl_803DA2E0: lbl_803DA2E0_t = lbl_803DA2E0_t {
    x0: [
        0x00, 0x01, 0x02, 0x03, 0x05, 0x06, 0x0C, 0x06, 0x06, 0x07, 0x09, 0x08, 0x06, 0x09, 0x04, 0x09, 0x0A, 0x0B, 0x0C, 0x02,
        0x0C, 0x06, 0x07, 0x09, 0x0C, 0, 0, 0, 0, 0, 0, 0,
    ],
    x20: [0.0; 41],
};

/// `lbl_803B7D18`: (start, end, loop) per animation state.
const lbl_803B7D18: [[u16; 3]; 6] = [[0, 30, 0], [50, 59, 0], [60, 70, 1], [90, 100, 0], [110, 130, 0], [150, 160, 0]];

/// `lbl_803B7D3C`.
const lbl_803B7D3C: [i32; 5] = [0, 1, 2, 3, 0];

const PAD_TRIGGER_R: u32 = 1 << 5;
const PAD_TRIGGER_L: u32 = 1 << 6;

// ---------------------------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------------------------

#[inline]
unsafe fn sfxBack() {
    ext::lbAudioAx_80024030(0);
}
#[inline]
unsafe fn sfxForward() {
    ext::lbAudioAx_80024030(1);
}
#[inline]
unsafe fn sfxMove() {
    ext::lbAudioAx_80024030(2);
}
#[inline]
unsafe fn gobj_jobj(g: *mut HSD_GObj) -> *mut HSD_JObj {
    (*g).hsd_obj as *mut HSD_JObj
}
#[inline]
unsafe fn jchild(j: *mut HSD_JObj) -> *mut HSD_JObj {
    HSD_JObjGetChild(j.as_ref())
}
#[inline]
unsafe fn jnext(j: *mut HSD_JObj) -> *mut HSD_JObj {
    HSD_JObjGetNext(j.as_ref())
}
#[inline]
unsafe fn x37(tm: *mut TmData, i: i32) -> *mut TmUnkMenuData {
    (addr_of_mut!((*tm).x37) as *mut TmUnkMenuData).offset(i as isize)
}
#[inline]
unsafe fn t534(tm: *mut TmData, i: i32) -> *mut *mut HSD_Text {
    (*tm).x534.as_mut_ptr().offset(i as isize)
}
#[inline]
unsafe fn timers() -> *mut TmAnimTimers {
    addr_of_mut!(lbl_80479A58)
}

// ---------------------------------------------------------------------------------------------
// functions
// ---------------------------------------------------------------------------------------------

pub unsafe extern "C" fn fn_8019BA08(idx: u8, jobj: *mut HSD_JObj) {
    let mut jobj = jobj;
    let tmd = gm_GetTournamentData();

    if (*tmd).x4B8[idx as usize].x2 != 0 {
        jobj = jchild(jobj);
        set_tz(jobj, 10000.0);
        let mut i = 1;
        while i <= 0xc {
            jobj = jnext(jobj);
            set_tz(jobj, 10000.0);
            i += 1;
        }
        return;
    }

    jobj = jchild(jobj);
    set_tz(jobj, 10000.0);

    if lbl_803DA2E0.x0[(*tmd).x4B8[idx as usize].x1 as usize] == 0 {
        set_tz(jobj, 0.0);
        let mut i = 1;
        while i <= 0xc {
            jobj = jnext(jobj);
            set_tz(jobj, 10000.0);
            i += 1;
        }
        return;
    }

    let mut i = 1;
    while i <= 0xc {
        jobj = jnext(jobj);
        set_tz(jobj, 10000.0);
        if lbl_803DA2E0.x0[(*tmd).x4B8[idx as usize].x1 as usize] as i32 == i {
            set_tz(jobj, 0.0);
            let mut j = i + 1;
            while j <= 12 {
                jobj = jnext(jobj);
                set_tz(jobj, 10000.0);
                j += 1;
            }
            return;
        }
        i += 1;
    }
}

pub unsafe extern "C" fn fn_8019BF18(gobj: *mut HSD_GObj) {
    let jobj = gobj_jobj(gobj);

    if (*timers()).x4 > 800 {
        (*timers()).x4 = 0;
    }

    fn_8019044C(jobj, (*timers()).x4 as f32);
    (*timers()).x4 = (*timers()).x4.wrapping_add(1);
}

pub unsafe extern "C" fn fn_8019BF8C(gobj: *mut HSD_GObj) {
    let selection = (*gm_GetTournamentData()).cur_option;
    let jobj = gobj_jobj(gobj);
    if selection <= 0x1A || selection >= 0x1F {
        if (*timers()).xE as u32 > 0x77 {
            (*timers()).xE = 0x5A;
        }
    } else if selection <= 0x1E && (*timers()).xE as u32 > 0x9F {
        (*timers()).xE = 0x82;
    }

    fn_8019044C(jobj, (*timers()).xE as f32);
    (*timers()).xE = (*timers()).xE.wrapping_add(1);
}

pub unsafe extern "C" fn fn_8019C048(gobj: *mut HSD_GObj) {
    let tmd = gm_GetTournamentData();
    let idx = fn_8018F62C(&*gobj) as i32;
    let jobj = gobj_jobj(gobj);
    let table = lbl_803B7D18;
    let t = timers();
    let iu = idx as usize;

    if (*super::gmtou_1::ext::HSD_PadMasterStatus.as_ptr().add(idx as u8 as usize)).err != 0 {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        ext::HSD_SisLib_803A7548((*tmd).x524[2], idx, 0.0, 0.0);
        return;
    }

    ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);

    if (*t).x1D[iu].x0 == 4 {
        ext::HSD_SisLib_803A7548((*tmd).x524[2], idx, 0.0, 0.0);
    } else {
        ext::HSD_SisLib_803A7548((*tmd).x524[2], idx, 1.0, 1.0);
    }

    let players = (*tmd).x30;
    let mut x: f32 = 0.0;
    gmTournament_SetPlayerX(&mut x, players, idx);

    set_tx(jobj, x);

    (*(*tmd).x524[2]).hidden = 0;

    (*t).x1D[iu].a = table[(*t).x1D[iu].x0 as usize][0] as u8;
    (*t).x1D[iu].c = table[(*t).x1D[iu].x0 as usize][1] as u8;
    (*t).x1D[iu].x2 = table[(*t).x1D[iu].x0 as usize][2] as u8;

    if (*t).x1D[iu].b < (*t).x1D[iu].a {
        (*t).x1D[iu].b = (*t).x1D[iu].a;
    }

    if (*t).x1D[iu].b < (*t).x1D[iu].c {
        (*t).x1D[iu].b += 1;
    } else {
        (*t).x1D[iu].x1 = 1;
        if (*t).x1D[iu].x2 != 0 {
            (*t).x1D[iu].b = (*t).x1D[iu].a;
        } else {
            (*t).x1D[iu].b = (*t).x1D[iu].c;
        }
    }

    let state = (*t).x1D[iu].x0;
    if state == 0 && (*t).x38[iu] == 6 {
        if (*t).x1D[iu].x1 != 0 && (*tmd).x4B8[iu].x0 != 0 {
            (*t).x1D[iu].x0 = 1;
            (*t).x1D[iu].x1 = 0;
        }
    }

    let state = (*t).x1D[iu].x0;
    if state == 1 {
        if (*t).x1D[iu].x1 != 0 {
            (*t).x1D[iu].x0 = 2;
            (*t).x1D[iu].x1 = 0;
        }
    }

    fn_8019044C(jobj, (*t).x1D[iu].b as f32);
}

pub unsafe extern "C" fn fn_8019C3EC(gobj: *mut HSD_GObj) {
    let tmd = gm_GetTournamentData();
    let idx = fn_8018F62C(&*gobj) as i32;
    let jobj = gobj_jobj(gobj);
    if ext::HSD_PadMasterStatus[idx as u8 as usize].err != 0 {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        return;
    }
    ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);

    let var_f1 = gmTournament_GetPlayerX((*tmd).x30, idx);
    fn_8018FDC4(jobj, 4.5f32 + var_f1, 3.0, 666.0);
    if (*timers()).x1D[idx as usize].x0 == 4 {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
    }
    fn_8019BA08(idx as u8, jobj);
}

#[inline]
unsafe fn GetTmData() -> *mut TmData {
    gm_GetTournamentData()
}

pub unsafe extern "C" fn fn_8019C570(gobj: *mut HSD_GObj) {
    let tmd = GetTmData();

    let idx = fn_8018F62C(&*gobj) as u64;
    let mut jobj = gobj_jobj(gobj);
    if ext::HSD_PadMasterStatus[idx as u8 as usize].err != 0 {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        return;
    }
    ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
    if (*timers()).x1D[idx as usize].x0 == 4 {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
    }
    if (*tmd).x4B8[idx as usize].x2 != 0 {
        jobj = jchild(jobj);
        jobj = jnext(jobj);
        fn_8019044C(jobj, 201.0);
        return;
    }
    jobj = jchild(jobj);
    jobj = jnext(jobj);
    fn_8019044C(jobj, fn_8018F71C((*tmd).x4B8[idx as usize].x1 as i32, (*tmd).x4B8[idx as usize].x3 as i32));
}

pub unsafe extern "C" fn fn_8019C6AC(gobj: *mut HSD_GObj) {
    GetTmData();
    let jobj = gobj_jobj(gobj);

    if (*timers()).xF > 60 {
        (*timers()).xF = 60;
    }

    fn_8019044C(jobj, ((*timers()).xF as i32 + gm_804771C4.match_type * 100 + 100) as f32);
    (*timers()).xF = (*timers()).xF.wrapping_add(1);
}

pub unsafe extern "C" fn fn_8019C744(gobj: *mut HSD_GObj) {
    let tmd = gm_GetTournamentData();
    let idx = fn_8018F62C(&*gobj) as i32;
    let jobj = gobj_jobj(gobj);
    let t = timers();
    let iu = idx as usize;

    let mut var_f1 = gmTournament_GetPlayerX((*tmd).x30, idx);
    fn_8018FDC4(jobj, 0.3f32 + var_f1, 10.1, 666.0);

    if (*t).x1D[iu].x0 == 4 {
        if ((*t).x10[iu] as u32) < 0x28 {
            (*t).x10[iu] += 1;
        }
        set_ty(jobj, lbl_803DA2E0.x20[(*t).x10[iu] as usize]);
    } else {
        (*t).x10[iu] = 0;
    }

    if ext::HSD_PadMasterStatus[idx as u8 as usize].err != 0 && (*tmd).x4B8[iu].x0 != 1 {
        ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
        var_f1 = gmTournament_GetPlayerX((*tmd).x30, idx);
        fn_8018FDC4(jobj, 0.3f32 + var_f1, 10.1, 666.0);
    }
}

pub unsafe extern "C" fn fn_8019CA38(gobj: *mut HSD_GObj) {
    let tmd = gm_GetTournamentData();
    let idx = fn_8018F62C(&*gobj) as i32;
    let jobj = gobj_jobj(gobj);
    let t = timers();
    if ext::HSD_PadMasterStatus[idx as u8 as usize].err != 0 {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        return;
    }

    ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
    let mut var_f1: f32 = 0.0;
    gmTournament_SetPlayerX(&mut var_f1, (*tmd).x30, idx);
    fn_8018FDC4(jobj, var_f1, 666.0, 666.0);
    if (*t).x1D[idx as usize].x0 == 2 {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
    }

    if ((*t).x6[idx as usize] as u32) < 600 {
        (*t).x6[idx as usize] += 1;
    } else {
        (*t).x6[idx as usize] = 0;
    }
    fn_8019044C(jobj, (*t).x6[idx as usize] as f32);
}

pub unsafe extern "C" fn fn_8019CBFC(gobj: *mut HSD_GObj) {
    GetTmData();
    let jobj = gobj_jobj(gobj);

    fn_8019044C(jobj, ((*timers()).x1C as i32 + 0x14) as f32);
    if ((*timers()).x1C as u32) < 0xA {
        (*timers()).x1C += 1;
    }
}

pub unsafe extern "C" fn fn_8019CC74(gobj: *mut HSD_GObj) {
    let tmd = gm_GetTournamentData();
    let idx = fn_8018F62C(&*gobj) as i32;
    let jobj = gobj_jobj(gobj);
    if ext::HSD_PadMasterStatus[idx as u8 as usize].err == 0 {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        return;
    }
    ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);

    let mut var_f1: f32 = 0.0;
    gmTournament_SetPlayerX(&mut var_f1, (*tmd).x30, idx);
    fn_8018FDC4(jobj, var_f1, 666.0, 0.01);
}

pub unsafe extern "C" fn fn_8019CDBC(gobj: *mut HSD_GObj) {
    let tmd = gm_GetTournamentData();
    let jobj = gobj_jobj(gobj);
    let idx = fn_8018F62C(&*gobj);

    let mut sp28 = HSD_JObjGetTranslationX(&*jobj);
    let temp_r27 = (*x37(tmd, idx as i32)).xE;
    let sp24 = fmadds(5.999997, temp_r27 as f32, -21.5);
    crate::mn_ptr::mn_8022F410(&mut sp28, &sp24, 0.4);

    match crate::mn_ptr::mn_8022F410(&mut sp28, &sp24, 0.2) {
        1 => {
            fn_8018FDC4(jobj, sp28, 19.5, -6.0);
            ext::HSD_SisLib_803A746C(*t534(tmd, idx as i32), 0, 10.0f32 * sp28, -197.0);
            (**t534(tmd, idx as i32)).pos_z = -6.0;
        }
        -1 => {
            fn_8018FDC4(jobj, sp28, 15.0, 6.0);
            ext::HSD_SisLib_803A746C(*t534(tmd, idx as i32), 0, 10.0f32 * sp28, -152.0);
            (**t534(tmd, idx as i32)).pos_z = 6.0;
        }
        _ => {
            (*x37(tmd, idx as i32)).xF = (*x37(tmd, idx as i32)).xE;
            fn_8018FDC4(jobj, sp28, 17.0, 0.01);
            ext::HSD_SisLib_803A746C(*t534(tmd, idx as i32), 0, 10.0f32 * sp28, -172.0);
            (**t534(tmd, idx as i32)).pos_z = 0.01;
        }
    }
    if (*x37(tmd, idx as i32)).x5 != 0 {
        fn_8019044C(jobj, 201.0);
        return;
    }
    fn_8019044C(jobj, fn_8018F71C((*x37(tmd, idx as i32)).x3 as i32, (*x37(tmd, idx as i32)).x7 as i32));
}

pub unsafe extern "C" fn fn_8019CFA4(gobj: *mut HSD_GObj) {
    let data = GetTmData();
    let mut jobj = jnext(jnext(jchild(gobj_jobj(gobj))));

    fn_8019044C(jobj, (*data).x2E as f32);
    jobj = jnext(jobj);
    fn_8019044C(jobj, (*data).x30 as f32);
}

pub unsafe extern "C" fn fn_8019D074(gobj: *mut HSD_GObj) {
    let tmd = gm_GetTournamentData();
    let idx = fn_8018F62C(&*gobj) as i32;
    let jobj = gobj_jobj(gobj);
    if ext::HSD_PadMasterStatus[idx as u8 as usize].err != 0 {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        return;
    }
    ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
    let mut var_f1: f32 = 0.0;
    gmTournament_SetPlayerX(&mut var_f1, (*tmd).x30, idx);
    fn_8018FDC4(jobj, var_f1, 666.0, 666.0);
}

#[inline]
pub unsafe fn gmTournament_GetJObj(gobj: *mut HSD_GObj) -> *mut HSD_JObj {
    gobj_jobj(gobj)
}

pub unsafe extern "C" fn fn_8019D1BC() {
    let tmd = gm_GetTournamentData();
    let m = (*lbl_804D6694).models;
    // char name_buf[16][0x14]; sized for the largest bracket (see the module note)
    let mut name_buf: [[u8; 0x14]; 64] = [[0; 0x14]; 64];

    let mut i: i32 = 0;
    while i < (*tmd).x30 as i32 {
        let iu = i as usize;
        {
            let first_gobj = fn_8019035C(0, *m.add(12), 0, 0x1A, 2, 1, Some(fn_8019C048), i as f32);
            let first_c = gmTournament_GetJObj(first_gobj);
            let first_jobj = gobj_jobj(first_gobj);
            set_ty(first_c, -2.5);
            fn_8018FBD8(&mut *first_gobj, i);

            let c = jchild(first_jobj);
            ext::HSD_MObjRemoveAnimByFlags((*(*c).u.dobj).mobj, 4);
            let c = jchild(first_jobj);
            let c = jnext(c);
            ext::HSD_MObjRemoveAnimByFlags((*(*c).u.dobj).mobj, 4);
            let c = jchild(first_jobj);
            let c = jnext(c);
            let c = jnext(c);
            ext::HSD_MObjRemoveAnimByFlags((*(*c).u.dobj).mobj, 4);
        }

        let pos: f32;
        if (*tmd).x30 == 4 {
            pos = 73.0f32 + fmadds(240.0, i as f32, 152.0);
        } else if (*tmd).x30 == 3 {
            pos = 73.0f32 + fmadds(240.0, i as f32, 270.0);
        } else {
            pos = 73.0f32 + fmadds(480.0, i as f32, 270.0);
        }

        fn_8018ECA8(
            (*tmd).x4B8[iu].x6 as i32,
            (*tmd).x4B8[iu].x0 as i32,
            5,
            fmadds(0.99, pos - 320.0, -261.0),
            -11.0,
            5,
        );
        (*(*tmd).x524[2]).hidden = 1;

        let gobj = fn_8019035C(0, *m.add(10), 0, 0x1A, 2, 1, Some(fn_8019C3EC), pos);
        let jobj = gobj_jobj(gobj);
        set_ty(jobj, -2.5);
        fn_8018FBD8(&mut *gobj, i);
        fn_8019BA08(i as u8, jobj);

        let gobj = fn_8019035C(0, *m.add(4), 0, 0x1A, 2, 1, Some(fn_8019C570), pos);
        let jobj = gobj_jobj(gobj);
        fn_8018FBD8(&mut *gobj, i);

        let mut var_f1 = gmTournament_GetPlayerX((*tmd).x30, i);
        fn_8018FDC4(jobj, var_f1 - 2.799999f32, 4.3, 0.001);

        if (*tmd).x4B8[iu].x2 != 0 {
            let c = jchild(jobj);
            let c = jnext(c);
            fn_8019044C(c, 201.0);
        } else {
            let c = jchild(jobj);
            let c = jnext(c);
            fn_8019044C(c, fn_8018F71C((*tmd).x4B8[iu].x1 as i32, (*tmd).x4B8[iu].x3 as i32));
        }

        let gobj = fn_8019035C(0, *m.add(9), 0, 0x1A, 2, 1, Some(fn_8019C744), pos);
        let jobj = gobj_jobj(gobj);
        set_ty(jobj, -2.5);
        fn_8018FBD8(&mut *gobj, i);

        let anim: f32 = if (*tmd).x4B8[iu].x0 == 0 { i as f32 } else { 4.0 };
        fn_8019044C(jobj, anim);

        let gobj = fn_8019035C(0, *m.add(8), 0, 0x1A, 2, 1, Some(fn_8019CA38), 0.0);
        let jobj = gobj_jobj(gobj);
        set_ty(jobj, -2.5);
        fn_8018FBD8(&mut *gobj, i);

        let gobj = fn_8019035C(0, *m.add(14), 0, 0x1A, 2, 1, Some(fn_8019CC74), anim);
        let jobj = gobj_jobj(gobj);
        set_ty(jobj, -2.5);
        fn_8018FBD8(&mut *gobj, i);

        if (*tmd).x31 != 0 {
            let gobj = fn_8019035C(0, *m.add(11), 0, 0x1A, 2, 1, Some(fn_8019D074), anim);
            let jobj = gobj_jobj(gobj);
            set_ty(jobj, -2.0);
            fn_8018FBD8(&mut *gobj, i);
            fn_8019044C(jobj, (*tmd).x4B8[iu].x5 as f32);
        }
        let _ = &mut var_f1;
        i += 1;
    }

    fn_8019035C(0, *m.add(1), 0, 0x1A, 2, 1, Some(fn_8019CFA4), 0.0);
    fn_8019035C(0, *m.add(5), 0, 0x1A, 2, 1, Some(fn_8019C6AC), 0.0);

    {
        let mut j16 = gobj_jobj(fn_8019035C(0, *m.add(0), 0, 0x1A, 2, 1, None, 0.0));
        j16 = jchild(j16);
        j16 = jnext(j16);
        j16 = jnext(j16);
        j16 = jnext(j16);
        j16 = jnext(j16);
        j16 = jnext(j16);
        let mut i: i32 = 0;
        while i <= 0x28 {
            fn_8019044C(j16, (i + 10) as f32);
            lbl_803DA2E0.x20[i as usize] = HSD_JObjGetTranslationY(&*j16);
            i += 1;
        }
    }

    fn_8019035C(0, *m.add(6), 0, 0x1A, 2, 1, Some(fn_8019CBFC), 0.0);

    let mut i: i32 = 0;
    while i < (*tmd).x2E as i32 {
        let gobj = fn_8019035C(0, *m.add(3), 0, 0x1A, 2, 1, Some(fn_8019CDBC), 0.0);
        let jobj = gobj_jobj(gobj);
        fn_8018FDC4(jobj, fmadds(5.999997, (*x37(tmd, i)).xF as f32, -21.5), 17.0, 0.01);

        if (*x37(tmd, i)).x5 != 0 {
            fn_8019044C(jobj, 201.0);
        } else {
            fn_8019044C(jobj, fn_8018F71C((*x37(tmd, i)).x3 as i32, (*x37(tmd, i)).x7 as i32));
        }

        fn_8018FBD8(&mut *gobj, i);
        fn_8018F00C(name_buf[i as usize].as_mut_ptr() as *mut c_char, (*x37(tmd, i)).x9 as i32);

        let name_font_size = Vec2 { x: 0.1, y: 0.1 };

        *t534(tmd, i) = ext::HSD_SisLib_803A6754(0, lbl_804D663C);
        (**t534(tmd, i)).default_kerning = 1;
        (**t534(tmd, i)).x4C = 1;
        {
            let text = *t534(tmd, i);
            (*text).font_size.x = name_font_size.x;
            (*text).font_size.y = name_font_size.y;
        }
        (**t534(tmd, i)).default_alignment = 1;
        ext::HSD_SisLib_803A6B98(
            *t534(tmd, i),
            10.0f32 * fmadds(5.999997, i as f32, -21.5),
            -172.0,
            name_buf[i as usize].as_ptr() as *const c_char,
        );
        ext::HSD_SisLib_803A7548(*t534(tmd, i), 0, 0.35, 0.6);
        i += 1;
    }
}

pub unsafe extern "C" fn fn_8019DD60() {
    let tmd = gm_GetTournamentData();
    if (*tmd).x2C == 0 {
        if gm_804771C4.match_type == 1 {
            ext::lbAudioAx_800237A8(0x7545, 0x7F, 0x40);
        } else {
            ext::lbAudioAx_800237A8(0x7546, 0x7F, 0x40);
        }
    }
    (*tmd).x2C = 0;
    let gobj = fn_80190174((*(*lbl_804D6690).cameras).desc);
    fn_801901F8((*(*lbl_804D6690).cameras).desc);
    fn_801902F0(gobj as usize as i32);
    fn_8019027C((*lbl_804D6690).lights as *mut c_void);
    fn_8019035C(0, *(*lbl_804D6690).models.add(5), 0, 0x1A, 2, 1, Some(fn_8019BF18), 0.0);
    fn_8019035C(0, *(*lbl_804D6690).models.add(4), 0, 0x1A, 2, 1, Some(fn_8019BF8C), 0.0);
    (*tmd).x524[2] = ext::HSD_SisLib_803A6754(0, lbl_804D663C);
    (*(*tmd).x524[2]).default_kerning = 1;
    let text = (*tmd).x524[2];
    (*text).font_size.x = 0.054945;
    (*text).font_size.y = 0.08;
    (*(*tmd).x524[2]).default_alignment = 1;
    fn_8019D1BC();
    let gobj = ext::GObj_Create(0xE, 0x1A, 0);
    {
        let tmp = ext::HSD_FogLoadDesc((*(*lbl_804D6690).fogs).desc);
        ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_FogKind as u8, tmp as *mut c_void);
    }
    ext::GObj_SetupGXLink(gobj, Some(ext::HSD_GObj_FogCallback), 0, 0);

    let t = timers();
    (*t).x0 = 0;
    (*t).x4 = 0;
    (*t).xE = 0x50;
    (*t).xF = 0;
    let mut i: i32 = 0;
    while i < (*tmd).x30 as i32 {
        let iu = i as usize;
        (*t).x18[iu] = 0;
        (*t).x6[iu] = 0;
        (*t).x10[iu] = 0;
        (*t).x1D[iu].a = 0;
        (*t).x1D[iu].b = 0;
        (*t).x1D[iu].c = 0x1E;
        (*t).x1D[iu].x1 = 0;
        (*t).x1D[iu].x2 = 0;
        (*t).x1D[iu].x0 = 0;
        (*t).x38[iu] = 6;
        i += 1;
    }
}

#[inline]
unsafe fn get_match_player_index(player: i32) -> i32 {
    let tmd = gm_GetTournamentData();
    let mut i = 0;
    while i < (*tmd).x2E as i32 {
        if player == (*x37(tmd, i)).xE as i32 {
            return i;
        }
        i += 1;
    }
    0
}

#[inline]
unsafe fn get_match_player_index_xF(player: i32) -> i32 {
    let tmd = gm_GetTournamentData();
    let mut i = 0;
    while i < (*tmd).x2E as i32 {
        if player == (*x37(tmd, i)).xF as i32 {
            return i;
        }
        i += 1;
    }
    0
}

pub unsafe extern "C" fn gm_Scene_TouAlt_OnFrame() {
    let mut vsdata: TmVsData = core::mem::zeroed();
    let mut confirmed: i32 = 0;
    let tmd = gm_GetTournamentData();
    let t = timers();
    fn_8018F674(4);
    fn_8018F640(4);

    if ext::mn_8022F218() != 0 {
        sfxBack();
        ext::mn_8022F268();
        ext::gm_801A4B60();
        ext::gm_ChangeGameModeAfterCurrentScene(GM_MENU as i32);
        return;
    }

    let mut i: i32 = 0;
    while i < (*tmd).x30 as i32 {
        if (*t).x1D[i as usize].x0 == 2 && (*t).x1D[i as usize].b as u32 >= 0x3C && ext::HSD_PadMasterStatus[i as u8 as usize].err == 0 {
            confirmed += 1;
        }
        i += 1;
    }

    if confirmed == (*tmd).x30 as i32 {
        (*t).x0 += 1;
        if (*t).x0 >= 0x1E {
            let mut j: i32 = 0;
            while j < (*tmd).x2E as i32 {
                (*x37(tmd, j)).xF = (*x37(tmd, j)).xE;
                j += 1;
            }
            fn_801965C4();
        }
    } else {
        (*t).x0 = 0;

        let mut i: i32 = 0;
        while i < (*tmd).x30 as i32 {
            let iu = i as usize;
            if ext::HSD_PadMasterStatus[i as u8 as usize].err == 0 {
                let buttons = fn_8018F640(i);
                let pressed = fn_8018F674(i);

                // Abort combo check
                if fn_8018F6A8(i) & PAD_BUTTON_B != 0 {
                    (*t).x18[iu] = (*t).x18[iu].wrapping_add(1);
                    if (*t).x18[iu] > 0x5A {
                        sfxForward();
                        ext::gm_SetNextGameModeStateId(0);
                        ext::gm_801A4B60();
                        return;
                    }
                } else {
                    (*t).x18[iu] = 0;
                }

                {
                    let status = (*t).x1D[iu].x0;
                    if status != 2 && status != 1 {
                        if (fn_8018F6A8(i) & PAD_TRIGGER_L) != 0 && (fn_8018F6A8(i) & PAD_TRIGGER_R) != 0 {
                            let j = get_match_player_index(i);
                            (*x37(tmd, j)).x5 = 1;
                            (*tmd).x4B8[iu].x2 = 1;
                        }

                        if pressed & (PAD_BUTTON_LEFT | PAD_STICK_LEFT) != 0 {
                            sfxMove();

                            let j = get_match_player_index(i);
                            (*x37(tmd, j)).x5 = 0;
                            (*tmd).x4B8[iu].x2 = 0;

                            let mut chr = (*tmd).x4B8[iu].x1;
                            loop {
                                if chr != 0 {
                                    chr -= 1;
                                } else {
                                    chr = fn_8018F6DC(0x19) as u8;
                                }
                                if ext::gm_IsCKindUnlocked(fn_8018F6FC(chr as i32) as u8) != 0 {
                                    break;
                                }
                            }
                            (*tmd).x4B8[iu].x1 = chr;

                            let j = get_match_player_index(i);
                            (*x37(tmd, j)).x3 = (*tmd).x4B8[iu].x1;

                            let j = get_match_player_index(i);
                            (*tmd).x4B8[iu].x3 = 0;
                            (*x37(tmd, j)).x7 = 0;
                        } else if pressed & (PAD_BUTTON_RIGHT | PAD_STICK_RIGHT) != 0 {
                            sfxMove();

                            let j = get_match_player_index(i);
                            (*x37(tmd, j)).x5 = 0;
                            (*tmd).x4B8[iu].x2 = 0;

                            let mut chr = (*tmd).x4B8[iu].x1;
                            loop {
                                if (chr as i32) < fn_8018F6DC(0x19) {
                                    chr += 1;
                                } else {
                                    chr = fn_8018F6DC(0) as u8;
                                }
                                if ext::gm_IsCKindUnlocked(fn_8018F6FC(chr as i32) as u8) != 0 {
                                    break;
                                }
                            }
                            (*tmd).x4B8[iu].x1 = chr;

                            let j = get_match_player_index(i);
                            (*x37(tmd, j)).x3 = (*tmd).x4B8[iu].x1;

                            let j = get_match_player_index(i);
                            (*tmd).x4B8[iu].x3 = 0;
                            (*x37(tmd, j)).x7 = 0;
                        }
                    }
                }

                if buttons & (PAD_BUTTON_A | PAD_BUTTON_START) != 0 {
                    if (*t).x1D[iu].x0 != 2 {
                        sfxForward();
                        (*t).x1D[iu].x0 = 1;
                        let char_id = (*tmd).x4B8[iu].x6;
                        if char_id as u32 <= 0x78 {
                            ext::gm_80167858(i, char_id as i32, 0xB, 0x14);
                        } else {
                            ext::gm_80167858(i, 0x78, 0xB, 0x14);
                        }
                    }
                } else if buttons & PAD_BUTTON_B != 0 {
                    if (*t).x1D[iu].x0 == 2 {
                        sfxBack();
                        (*t).x1D[iu].x0 = 3;
                    }
                } else {
                    let color_status = (*t).x1D[iu].x0;
                    if color_status != 2 && color_status != 1 {
                        if buttons & PAD_BUTTON_X != 0 {
                            // Down: increment color
                            if ((*tmd).x4B8[iu].x3 as i32)
                                < (ext::gm_GetNumCostumesForCKind(fn_8018F6FC((*tmd).x4B8[iu].x1 as i32) as u8) as i32 - 1)
                            {
                                (*tmd).x4B8[iu].x3 += 1;
                            }

                            let j = get_match_player_index(i);
                            (*x37(tmd, j)).x7 = (*tmd).x4B8[iu].x3;
                        } else if buttons & PAD_BUTTON_Y != 0 {
                            // Up: decrement color
                            let color = (*tmd).x4B8[iu].x3;
                            if color != 0 {
                                (*tmd).x4B8[iu].x3 = color - 1;
                            }

                            let j = get_match_player_index(i);
                            (*x37(tmd, j)).x7 = (*tmd).x4B8[iu].x3;
                        }
                    }
                }
            }
            i += 1;
        }

        let stage: i32 = if fn_80196564(tmd) != 0 { fn_8019655C() } else { (*tmd).x28 as i32 };
        vsdata.stkind = stage as _;
        let mut i = 0;
        while i < 4 {
            vsdata.slot_type[i] = (*tmd).x4B8[i].x0 as _;
            vsdata.char_id[i] = fn_8018F6FC((*tmd).x4B8[i].x1 as i32) as _;
            vsdata.color[i] = (*tmd).x4B8[i].x3 as u32;
            i += 1;
        }
        fn_8019EE80(&mut vsdata);
    }
}

pub unsafe extern "C" fn gm_8019ECAC_OnEnter_inline() {
    lbl_804D6688 = ext::lbArchive_80016DBC(
        b"GmTou1p\0".as_ptr() as *const c_char,
        addr_of_mut!(lbl_804D6690) as *mut c_void,
        &[b"ScGamTour_scene_data\0".as_ptr() as usize as u64, 0],
    );
    lbl_804D668C = ext::lbArchive_80016DBC(
        b"GmTou4p\0".as_ptr() as *const c_char,
        addr_of_mut!(lbl_804D6694) as *mut c_void,
        &[b"ScGamTour_scene_data\0".as_ptr() as usize as u64, 0],
    );
    ext::HSD_SisLib_803A62A0(0, fn_8018F5F0(), b"SIS_TournamentData\0".as_ptr() as *mut c_char);
}

pub unsafe extern "C" fn gm_8019E634() {
    let tmd = gm_GetTournamentData();
    let hmn_cpu = (*tmd).hmn_cpu_count;
    let mut indices: [i32; 4] = [lbl_803B7D3C[0], lbl_803B7D3C[1], lbl_803B7D3C[2], lbl_803B7D3C[3]];
    let mut results: [i32; 4] = [0; 4];

    // Get match results per player
    let mut i: i32 = 0;
    while i < (*tmd).x30 as i32 {
        results[i as usize] = crate::gm_ptr::fn_80166CBC(&gm_80477738, i) as i32;
        i += 1;
    }

    // Bubble sort results, keeping indices in parallel
    let mut i: i32 = 0;
    while i < (*tmd).x30 as i32 - 1 {
        let mut j: i32 = 0;
        while j < ((*tmd).x30 as i32 - 1) - i {
            let ju = j as usize;
            if results[ju] > results[ju + 1] {
                let tr = results[ju];
                let ti = indices[ju];
                results[ju] = results[ju + 1];
                indices[ju] = indices[ju + 1];
                results[ju + 1] = tr;
                indices[ju + 1] = ti;
            }
            j += 1;
        }
        i += 1;
    }

    // Handicap adjustment
    if (*ext::gmMainLib_GetGameRules()).handicap == 1 {
        let mut hbuf: [u8; 4] = [0; 4];

        // Read handicap from x37 entries
        let mut i: i32 = 0;
        while i < 4 {
            if i < (*tmd).x30 as i32 {
                let j = get_match_player_index_xF(results[i as usize]);
                hbuf[i as usize] = (*x37(tmd, j)).x2;
            }
            i += 1;
        }

        ext::fn_80169000(addr_of_mut!(gm_80477738), hbuf.as_mut_ptr());

        // Write back adjusted handicap
        let mut i: i32 = 0;
        while i < 4 {
            if i < (*tmd).x30 as i32 {
                let id = results[i as usize];
                let j = get_match_player_index_xF(id);
                (*x37(tmd, j)).x2 = hbuf[i as usize];
            }
            i += 1;
        }
    }

    // Assign bracket positions
    if gm_804771C4.match_type == 1 {
        // Team mode
        let mut i: i32 = 0;
        while i < hmn_cpu {
            let id = indices[i as usize];
            let j = get_match_player_index_xF(id);
            (*x37(tmd, j)).xE = ((*tmd).x2E as i32 - 1 - i) as u8;

            let next_id = (*tmd).x30 as i32 + i;
            let j = get_match_player_index_xF(next_id);
            (*x37(tmd, j)).xE = indices[i as usize] as u8;
            i += 1;
        }
    } else {
        // FFA mode
        let mut i: i32 = 0;
        while i < hmn_cpu {
            let id = indices[((*tmd).x30 as i32 - 1 - i) as usize];
            let j = get_match_player_index_xF(id);
            (*x37(tmd, j)).xE = ((*tmd).x2E as i32 - 1 - i) as u8;

            let id = (*tmd).x30 as i32 + i;
            let j = get_match_player_index_xF(id);
            (*x37(tmd, j)).xE = indices[((*tmd).x30 as i32 - 1 - i) as usize] as u8;
            i += 1;
        }
    }

    // Adjust remaining bracket positions
    let mut i: i32 = (*tmd).x30 as i32 + hmn_cpu;
    while i < (*tmd).x2E as i32 {
        let j = get_match_player_index_xF(i);
        (*x37(tmd, j)).xE = (*x37(tmd, j)).xE.wrapping_sub(hmn_cpu as u8);
        i += 1;
    }

    // Copy x37 data to x4B8
    let mut i: i32 = 0;
    while i < (*tmd).x30 as i32 {
        let iu = i as usize;
        let j = get_match_player_index(i);
        (*tmd).x4B8[iu].x6 = (*x37(tmd, j)).x9;

        let j = get_match_player_index(i);
        (*tmd).x4B8[iu].x5 = (*x37(tmd, j)).x2;

        let j = get_match_player_index(i);
        (*tmd).x4B8[iu].x1 = (*x37(tmd, j)).x3;

        let j = get_match_player_index(i);
        (*tmd).x4B8[iu].x0 = (*x37(tmd, j)).x0;

        let j = get_match_player_index(i);
        (*tmd).x4B8[iu].x3 = (*x37(tmd, j)).x7;

        let j = get_match_player_index(i);
        (*tmd).x4B8[iu].x2 = (*x37(tmd, j)).x5;
        i += 1;
    }

    // Debug output + audio preloading
    {
        let mut audio_mask: u64 = 8;
        let mut k: i32 = 0;
        while k < (*tmd).x30 as i32 {
            audio_mask |= ext::lbAudioAx_80026E84((*tmd).x4B8[k as usize].x1 as CharacterKind);
            ext::OSReport(b"ckind:%d\n\0".as_ptr() as *mut c_char, (*tmd).x4B8[k as usize].x1 as i32);
            k += 1;
        }
        ext::lbAudioAx_80026F2C(0x16);
        ext::lbAudioAx_8002702C(6, audio_mask);
        ext::lbAudioAx_80027168();
    }
}

pub unsafe extern "C" fn gm_Scene_TouAlt_OnEnter(_arg0: *mut c_void) {
    // struct { StKind stkind; CharacterKind char_id[4]; u32 color[4]; } local (zero-initialised)
    let mut local_stkind: StKind = 0;
    let mut local_char_id: [CharacterKind; 4] = [0; 4];
    let mut local_color: [u32; 4] = [0; 4];

    let tmd = gm_GetTournamentData();
    ext::lbAudioAx_80026F2C(0x12);
    ext::lbAudioAx_8002702C(2, 8);
    ext::lbAudioAx_80027168();
    ext::lbAudioAx_80027648();
    gm_GetTournamentData();
    gm_8019ECAC_OnEnter_inline();
    fn_8019DD60();
    ext::lbAudioAx_80027648();
    ext::lbAudioAx_80023F28(ext::gmMainLib_8015ECB0());

    let mut i = 0;
    while i < 4 {
        if (*tmd).x4B8[i].x0 != 3 {
            local_char_id[i] = fn_8018F6FC((*tmd).x4B8[i].x1 as i32) as CharacterKind;
            local_color[i] = (*tmd).x4B8[i].x3 as u32;
        }
        i += 1;
    }

    fn_80196510();
    if fn_80196564(tmd) != 0 {
        local_stkind = fn_8019655C() as StKind;
    } else {
        local_stkind = (*tmd).x28 as StKind;
    }
    fn_80196594(tmd);
    ext::lbDvd_SetupVsPreloadCache();

    let mut audio_mask: u64 = 0;
    let mut j = 0;
    while j < 4 {
        if (*tmd).x4B8[j].x0 != 3 {
            audio_mask |= ext::lbAudioAx_80026E84(local_char_id[j]);
        }
        j += 1;
    }
    audio_mask |= ext::lbAudioAx_80026EBC(local_stkind);
    ext::lbAudioAx_80026F2C(0x1C);
    ext::lbAudioAx_8002702C(0xC, audio_mask);
    ext::lbAudioAx_80027168();
    ext::lbAudioAx_80023F28(ext::gmMainLib_8015ECB0());
    let _ = local_color;
}

pub unsafe extern "C" fn gm_Scene_TouAlt_OnExit(_arg0: *mut c_void) {
    ext::lbArchive_80016EFC(lbl_804D6688);
    ext::lbArchive_80016EFC(lbl_804D668C);
}

pub unsafe extern "C" fn fn_8019EE80(arg0: *mut TmVsData) {
    let game_cache = addr_of_mut!((*ext::lbDvd_GetPreloadCacheScene()).game_cache);
    let mut i: i32 = 0;
    while i < (*gm_GetTournamentData()).x30 as i32 {
        (*game_cache).entries[i as usize].char_id = (*arg0).char_id[i as usize] as _;
        (*game_cache).entries[i as usize].color = (*arg0).color[i as usize] as _;
        i += 1;
    }

    if fn_80196594(gm_GetTournamentData()) == 0 {
        (*game_cache).stkind = (*arg0).stkind;
    }

    ext::lbDvd_80018254();
}

pub unsafe extern "C" fn fn_8019EF08(arg0: *mut TmVsData) {
    let game_cache = addr_of_mut!((*ext::lbDvd_GetPreloadCacheScene()).game_cache);
    let mut i = 0;
    while i < 4 {
        if (*arg0).slot_type[i] != Gm_PKind_NA as _ {
            (*game_cache).entries[i].char_id = (*arg0).char_id[i] as _;
            (*game_cache).entries[i].color = (*arg0).color[i] as _;
        }
        i += 1;
    }

    ext::lbDvd_80018254();
}
