//! src/melee/gm/gmtou_1.c: tournament bracket scene (player-setup widgets, bracket result
//! screens, per-frame handlers `fn_8019A71C`/`fn_8019A86C`/`fn_8019AF50`, scene OnEnter/OnExit).
//!
//! Fused float sites (tools/rust-port/fma_sites.py): the player-X helpers `13*x-19.5` /
//! `6.5+(13*x-19.5)` (inlined `gmTournament_GetPlayerX`/`SetPlayerX`, all inline sites), the
//! `73+(240*x+152)` family and `0.99*(pos-320)-261` of `fn_80198EBC` are `fmadds` below.
//!
//! GameCube-layout notes. The C reaches `lbl_804799D8` and the BracketEntry slots through raw
//! byte offsets (stride 0x2C, `x4E` text at 0x4E) that hold on the GameCube; they are written
//! as the named fields here (same bytes on the GC). The `TmPlayerAnimFrames` table is initialised
//! as big-endian words and read as u16 triplets, so it is stored decoded.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, clippy::all)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types as mt;
#[allow(unused_imports)]
use mt::TmData as TmData_gm_ptr;
use core::ffi::{c_char, c_void};
use core::ptr::{addr_of_mut, null_mut};

use melee_core::fp::fmadds;
use melee_runtime::cabi::sdiv;
use melee_types::inline::{HSD_JObjGetChild, HSD_JObjGetNext, HSD_JObjGetScale, HSD_JObjGetTranslation, HSD_JObjGetTranslationY};
use melee_types::*;

use super::gmtoulib::*;

pub mod ext {
    pub use super::super::gmtou_0::ext::*;
    use super::*;

    crate::gmtou_ext! {
        fn fn_80160400(arg0: CharacterKind) -> u32;
        fn fn_801642A0() -> i32;
        fn gm_80167858(port: i32, arg1: i32, arg2: i32, arg3: i32);
        fn lbAudioAx_8002305C(arg0: i32, arg1: i32) -> i32;
        fn lbAudioAx_80023730() -> i32;
        fn lbAudioAx_80026E84(arg0: CharacterKind) -> u64;
        fn lbAudioAx_80026EBC(arg0: StKind) -> u64;
        fn lbDvd_SetupVsPreloadCache();
        fn lbDvd_GetPreloadCacheScene() -> *mut PreloadedGameModeState;
        fn lbDvd_80018254();
        fn HSD_JObjGetDObj(jobj: *mut HSD_JObj) -> *mut HSD_DObj;
        fn HSD_SisLib_803A7548(arg0: *mut HSD_Text, arg1: i32, arg2: f32, arg3: f32);
        fn HSD_MObjRemoveAnimByFlags(mobj: *mut HSD_MObj, flags: u32);
        fn HSD_DObjGetFlags(dobj: *mut HSD_DObj) -> u32;
    }
    /// `HSD_SisLib_803A70A0(text, entry_idx, fmt, ...)`: `va` are the variadic words.
    pub unsafe extern "C" fn HSD_SisLib_803A70A0(arg0: *mut HSD_Text, entry_idx: i32, fmt: *mut c_char) -> i32 {
        use super::super::gmtoulib::hook::{self, Word};
        <i32 as Word>::from_w(hook::call("HSD_SisLib_803A70A0", &[arg0.to_w(), entry_idx.to_w(), fmt.to_w()]))
    }
}

// ---------------------------------------------------------------------------------------------
// data
// ---------------------------------------------------------------------------------------------

pub static mut lbl_804D6660: *mut HSD_Archive = null_mut();
pub static mut lbl_804D6664: *mut HSD_Archive = null_mut();
pub static mut lbl_804D6668: *mut HSD_Archive = null_mut();
pub static mut lbl_804D666C: *mut SceneDesc = null_mut();
pub static mut lbl_804D6670: *mut SceneDesc = null_mut();
pub static mut lbl_804D6674: *mut SceneDesc = null_mut();
pub static mut lbl_804D6678: i32 = 0;
pub static mut lbl_804D667C: i32 = 0;
pub static mut lbl_804D6680: [u8; 3] = [0; 3];
/// `lbl_804799D8`: bracket-scene state (counters, per-player anim records, text buffer).
pub static mut lbl_804799D8: Lbl804799D8_t = unsafe { core::mem::zeroed() };
pub static mut lbl_804D4190: i32 = -1;
pub static mut lbl_804D4194: i32 = -1;

/// `lbl_803DA0D0`: icon -> model map and rank thresholds (the flat C initializer: 24 + 6 + 192
/// bytes); `bounce_y` is filled by `fn_80198EBC`.
pub static mut lbl_803DA0D0: lbl_803DA0D0_t = lbl_803DA0D0_t {
    icon_model_map: [0, 1, 2, 3, 5, 6, 12, 6, 6, 7, 9, 8, 6, 9, 4, 9, 10, 11, 12, 2, 12, 6, 7, 9],
    pad_0x18: [12, 0, 0, 0, 0, 0],
    rank_thresholds: [
        [2, 0, 0, 0, 0, 3], [3, 0, 0, 0, 0, 5], [6, 4, 0, 0, 0, 6], [7, 6, 10, 0, 0, 0], [12, 13, 8, 12, 0, 0],
        [14, 15, 12, 18, 21, 0], [23, 24, 16, 24, 28, 0], [30, 31, 24, 36, 42, 46], [48, 49, 32, 48, 56, 60],
        [62, 63, 2, 0, 0, 0], [0, 3, 3, 0, 0, 0], [0, 4, 4, 0, 0, 0], [6, 7, 6, 0, 0, 0], [8, 9, 9, 0, 0, 0],
        [12, 13, 2, 0, 0, 0], [0, 0, 3, 3, 0, 0], [0, 0, 4, 4, 0, 0], [0, 0, 5, 4, 0, 0], [0, 0, 6, 7, 5, 0],
        [0, 0, 7, 8, 6, 0], [0, 0, 8, 9, 7, 0], [0, 0, 9, 10, 8, 0], [0, 0, 10, 11, 8, 12], [0, 0, 14, 15, 9, 0],
        [0, 0, 12, 13, 10, 0], [0, 0, 14, 15, 11, 0], [0, 0, 15, 16, 12, 0], [0, 0, 16, 17, 13, 0],
        [0, 0, 17, 18, 14, 0], [0, 0, 18, 19, 15, 0], [0, 0, 19, 20, 16, 0], [0, 0, 20, 21, 16, 20],
    ],
    pad_0xDE: [0, 0],
    bounce_y: [0.0; 41],
};

/// `lbl_803B7CE0` decoded: (start, end, loop) per animation state.
const lbl_803B7CE0: [[u16; 3]; 6] = [[0, 30, 0], [50, 59, 0], [60, 70, 1], [90, 100, 0], [110, 130, 0], [150, 160, 0]];

/// `lbl_803B7D04`: 20 zero bytes.
const lbl_803B7D04: [u8; 20] = [0; 20];

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
unsafe fn d8() -> *mut Lbl804799D8_t {
    addr_of_mut!(lbl_804799D8)
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
unsafe fn pad_err(i: i32) -> i8 {
    ext::HSD_PadMasterStatus[i as u8 as usize].err
}

/// inlines.h `gmTournament_GetPlayerX`: `13*p-19.5` (4 players), `6.5+(13*p-19.5)` (3),
/// `6.5+(13*(2*p)-19.5)` (else); the inner `a*b+c` pairs are fused.
#[inline]
pub fn gmTournament_GetPlayerX(player_count: u8, player: i32) -> f32 {
    melee_core::ct!("gmTournament_GetPlayerX");
    if player_count as i32 == 4 {
        return fmadds(13.0, player as f32, -19.5);
    }
    if player_count as i32 == 3 {
        return 6.5f32 + fmadds(13.0, player as f32, -19.5);
    }
    6.5f32 + fmadds(13.0, 2.0f32 * player as f32, -19.5)
}
#[inline]
pub fn gmTournament_SetPlayerX(x: &mut f32, player_count: u8, player: i32) {
    melee_core::ct!("gmTournament_SetPlayerX");
    *x = gmTournament_GetPlayerX(player_count, player);
}

/// `gmTournament_IsPlayerSetupOption`: `0x1B <= cur_option <= 0x1E`.
#[inline]
pub unsafe fn gmTournament_IsPlayerSetupOption() -> i32 {
    melee_core::ct!("gmTournament_IsPlayerSetupOption");
    let o = (*gm_GetTournamentData()).cur_option;
    if o >= 0x1B && (*gm_GetTournamentData()).cur_option <= 0x1E { 1 } else { 0 }
}

#[inline]
unsafe fn in_range_13_1a() -> bool {
    (*gm_GetTournamentData()).cur_option >= 0x13 && (*gm_GetTournamentData()).cur_option <= 0x1A
}

// ---------------------------------------------------------------------------------------------
// functions
// ---------------------------------------------------------------------------------------------

pub unsafe extern "C" fn fn_80196510() {
    melee_core::ct!("fn_80196510");
    gm_GetTournamentData();

    loop {
        let stage = fn_8018F4A0();
        lbl_804D4190 = stage;
        if lbl_804D4194 != lbl_804D4190 {
            break;
        }
        if ext::fn_801642A0() != 0 {
            break;
        }
    }

    lbl_804D4194 = lbl_804D4190;
}

pub unsafe extern "C" fn fn_8019655C() -> i32 {
    melee_core::ct!("fn_8019655C");
    lbl_804D4190
}

pub unsafe extern "C" fn fn_80196564(arg0: *mut TmData) -> i32 {
    melee_core::ct!("fn_80196564");
    ((((*arg0).stage_selection_type == 0 && (*arg0).x32 == 0) || (*arg0).stage_selection_type == 1) as i32)
}

pub unsafe extern "C" fn fn_80196594(arg0: *mut TmData) -> i32 {
    melee_core::ct!("fn_80196594");
    ((((*arg0).stage_selection_type == 2 && (*arg0).x32 == 0) || (*arg0).stage_selection_type == 3) as i32)
}

pub unsafe extern "C" fn fn_801965C4() {
    melee_core::ct!("fn_801965C4");
    let temp_r3 = gm_GetTournamentData();
    (*temp_r3).x2D = 1;

    if fn_80196594(temp_r3) != 0 {
        (*temp_r3).x32 = 1;
        ext::gm_SetNextGameModeStateId(3);
        ext::gm_801A4B60();
        return;
    }
    if fn_80196564(temp_r3) != 0 {
        (*temp_r3).x32 = 1;
        (*temp_r3).x28 = lbl_804D4190 as u32;
    }
    ext::gm_SetNextGameModeStateId(4);
    ext::gm_801A4B60();
}

#[inline]
unsafe fn x51(b: i32, s: usize) -> i32 {
    lbl_80473AB8[b as usize].slots[s].x51 as i32
}
#[inline]
unsafe fn set4c(b: i32, s: usize, v: u8) {
    lbl_80473AB8[b as usize].slots[s].x4C = v;
}

/// Randomly assigns bracket positions for tournament seeding (three entrants).
pub unsafe extern "C" fn fn_80196684(bracket_idx: i32) {
    melee_core::ct!("fn_80196684");
    let b = bracket_idx;
    let rand_val = ext::HSD_Randi(x51(b, 0) + x51(b, 1) + x51(b, 2));

    if rand_val < x51(b, 0) {
        set4c(b, 0, 0);
        if ext::HSD_Randi(x51(b, 1) + x51(b, 2)) < x51(b, 1) {
            set4c(b, 1, 1);
            set4c(b, 2, 2);
            return;
        }
        set4c(b, 1, 2);
        set4c(b, 2, 1);
        return;
    }
    if rand_val < x51(b, 0) + x51(b, 1) {
        set4c(b, 1, 0);
        if ext::HSD_Randi(x51(b, 0) + x51(b, 2)) < x51(b, 0) {
            set4c(b, 0, 1);
            set4c(b, 2, 2);
            return;
        }
        set4c(b, 0, 2);
        set4c(b, 2, 1);
        return;
    }
    set4c(b, 2, 0);
    if ext::HSD_Randi(x51(b, 0) + x51(b, 1)) < x51(b, 0) {
        set4c(b, 0, 1);
        set4c(b, 1, 2);
        return;
    }
    set4c(b, 0, 2);
    set4c(b, 1, 1);
}

/// Four-entrant seeding: pick the winner by weight, then the rest recursively.
pub unsafe extern "C" fn fn_801967E0(arg0: i32) {
    melee_core::ct!("fn_801967E0");
    let a = arg0;
    let mut rand = ext::HSD_Randi(x51(a, 0) + x51(a, 1) + x51(a, 2) + x51(a, 3));

    if rand < x51(a, 0) {
        set4c(a, 0, 0);
        rand = ext::HSD_Randi(x51(a, 1) + x51(a, 2) + x51(a, 3));
        if rand < x51(a, 1) {
            set4c(a, 1, 1);
            rand = ext::HSD_Randi(x51(a, 2) + x51(a, 3));
            if rand < x51(a, 2) {
                set4c(a, 2, 2);
                set4c(a, 3, 3);
                return;
            }
            set4c(a, 2, 3);
            set4c(a, 3, 2);
            return;
        }
        if rand < x51(a, 1) + x51(a, 2) {
            set4c(a, 2, 1);
            rand = ext::HSD_Randi(x51(a, 1) + x51(a, 3));
            if rand < x51(a, 1) {
                set4c(a, 1, 2);
                set4c(a, 3, 3);
                return;
            }
            set4c(a, 1, 3);
            set4c(a, 3, 2);
            return;
        }
        set4c(a, 3, 1);
        rand = ext::HSD_Randi(x51(a, 1) + x51(a, 2));
        if rand < x51(a, 2) {
            set4c(a, 1, 2);
            set4c(a, 2, 3);
            return;
        }
        set4c(a, 1, 3);
        set4c(a, 2, 2);
        return;
    }

    if rand < x51(a, 0) + x51(a, 1) {
        set4c(a, 1, 0);
        rand = ext::HSD_Randi(x51(a, 0) + x51(a, 2) + x51(a, 3));
        if rand < x51(a, 0) {
            set4c(a, 0, 1);
            rand = ext::HSD_Randi(x51(a, 2) + x51(a, 3));
            if rand < x51(a, 2) {
                set4c(a, 2, 2);
                set4c(a, 3, 3);
                return;
            }
            set4c(a, 2, 3);
            set4c(a, 3, 2);
            return;
        }
        if rand < x51(a, 0) + x51(a, 2) {
            set4c(a, 2, 1);
            rand = ext::HSD_Randi(x51(a, 0) + x51(a, 3));
            if rand < x51(a, 0) {
                set4c(a, 0, 2);
                set4c(a, 3, 3);
                return;
            }
            set4c(a, 0, 3);
            set4c(a, 3, 2);
            return;
        }
        set4c(a, 3, 1);
        rand = ext::HSD_Randi(x51(a, 0) + x51(a, 2));
        if rand < x51(a, 0) {
            set4c(a, 0, 2);
            set4c(a, 2, 3);
            return;
        }
        set4c(a, 0, 3);
        set4c(a, 2, 2);
        return;
    }

    if rand < x51(a, 2) + (x51(a, 0) + x51(a, 1)) {
        set4c(a, 2, 0);
        rand = ext::HSD_Randi(x51(a, 0) + x51(a, 1) + x51(a, 3));
        if rand < x51(a, 0) {
            set4c(a, 0, 1);
            rand = ext::HSD_Randi(x51(a, 1) + x51(a, 3));
            if rand < x51(a, 1) {
                set4c(a, 1, 2);
                set4c(a, 3, 3);
                return;
            }
            set4c(a, 1, 3);
            set4c(a, 3, 2);
            return;
        }
        if rand < x51(a, 0) + x51(a, 1) {
            set4c(a, 1, 1);
            rand = ext::HSD_Randi(x51(a, 0) + x51(a, 3));
            if rand < x51(a, 0) {
                set4c(a, 0, 2);
                set4c(a, 3, 3);
                return;
            }
            set4c(a, 0, 3);
            set4c(a, 3, 2);
            return;
        }
        set4c(a, 3, 1);
        rand = ext::HSD_Randi(x51(a, 0) + x51(a, 1));
        if rand < x51(a, 0) {
            set4c(a, 0, 2);
            set4c(a, 1, 3);
            return;
        }
        set4c(a, 0, 3);
        set4c(a, 1, 2);
        return;
    }

    set4c(a, 3, 0);
    rand = ext::HSD_Randi(x51(a, 0) + x51(a, 1) + x51(a, 2));
    if rand < x51(a, 0) {
        set4c(a, 0, 1);
        rand = ext::HSD_Randi(x51(a, 1) + x51(a, 2));
        if rand < x51(a, 1) {
            set4c(a, 1, 2);
            set4c(a, 2, 3);
            return;
        }
        set4c(a, 1, 3);
        set4c(a, 2, 2);
        return;
    }
    if rand < x51(a, 0) + x51(a, 1) {
        set4c(a, 1, 1);
        rand = ext::HSD_Randi(x51(a, 0) + x51(a, 2));
        if rand < x51(a, 0) {
            set4c(a, 0, 2);
            set4c(a, 2, 3);
            return;
        }
        set4c(a, 0, 3);
        set4c(a, 2, 2);
        return;
    }
    set4c(a, 2, 1);
    rand = ext::HSD_Randi(x51(a, 0) + x51(a, 1));
    if rand < x51(a, 0) {
        set4c(a, 0, 2);
        set4c(a, 1, 3);
        return;
    }
    set4c(a, 0, 3);
    set4c(a, 1, 2);
}

pub unsafe extern "C" fn fn_80196CF8() -> i32 {
    melee_core::ct!("fn_80196CF8");
    let tmdata = gm_GetTournamentData();
    let entrants = (*tmdata).entrants;
    let x24 = (*tmdata).x24;

    let mut result = 0;
    let th = &lbl_803DA0D0.rank_thresholds;
    if x24 > th[entrants as usize][5] as i32 {
        return 6;
    }

    let mut i: i32 = 5;
    while i >= 0 {
        if x24 <= th[entrants as usize][i as usize] as i32 {
            result = i;
        }
        i -= 1;
    }

    result
}

/// Cycles animation frame counter and updates JObj animation.
pub unsafe extern "C" fn fn_80196DBC(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_80196DBC");
    let counter = addr_of_mut!(lbl_804799D8.x10);
    let jobj = gobj_jobj(gobj);
    if *counter as u32 > 0x320 {
        *counter = 0;
    }
    fn_8019044C(jobj, *counter as f32);
    *counter = (*counter).wrapping_add(1);
}

/// Updates the tournament UI animation frame counter.
pub unsafe extern "C" fn fn_80196E30(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_80196E30");
    let state = d8();
    let cur_option = (*gm_GetTournamentData()).cur_option;
    let jobj = gobj_jobj(gobj);

    if cur_option <= 0x1A || cur_option >= 0x1F {
        if (*state).x1A as u32 > 0x77 {
            (*state).x1A = 0x5A;
        }
    } else if cur_option <= 0x1E {
        if (*state).x1A as u32 > 0x9F {
            (*state).x1A = 0x82;
        }
    }

    let frame = addr_of_mut!((*state).x1A);
    fn_8019044C(jobj, *frame as f32);
    *frame = (*frame).wrapping_add(1);
}

/// Tournament mode JObj visibility/animation callback.
pub unsafe extern "C" fn fn_80196EEC(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_80196EEC");
    let tmdata = gm_GetTournamentData();
    let jobj = gobj_jobj(gobj);

    let in_range = in_range_13_1a();

    if !in_range {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        return;
    }

    ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
    let counter = addr_of_mut!(lbl_804799D8.x1B);

    if *counter != 0 {
        *counter -= 1;
        ext::HSD_JObjAnimAll(jobj);
        if (*tmdata).x33 == (*tmdata).pad_x34[0] {
            if *counter > 2 {
                *counter = 2;
            }
            ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        }
    } else {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        if (*tmdata).cur_option == 0x14 {
            (*tmdata).cur_option = 0x15;
        }
    }
}

pub unsafe extern "C" fn fn_80196FFC(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_80196FFC");
    let tm = gm_GetTournamentData();
    let pnum = fn_8018F62C(&*gobj) as i32;
    let jobj = gobj_jobj(gobj);
    let table = lbl_803B7CE0;

    let in_range = gmTournament_IsPlayerSetupOption();

    if in_range == 0 {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        return;
    }

    ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);

    if pad_err(pnum) != 0 && (*tm).x4B8[pnum as usize].x0 != 1 {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        ext::HSD_SisLib_803A7548((*tm).x524[2], pnum, 0.0, 0.0);
        return;
    }

    let state = lbl_804799D8.x2A[pnum as usize].state;
    if state == 4 {
        ext::HSD_SisLib_803A7548((*tm).x524[2], pnum, 0.0, 0.0);
    } else {
        ext::HSD_SisLib_803A7548((*tm).x524[2], pnum, 1.0, 1.0);
    }

    let players = (*tm).x30;
    let mut x: f32 = 0.0;
    gmTournament_SetPlayerX(&mut x, players, pnum);

    set_tx(jobj, x);

    (*(*tm).x524[2]).hidden = 0;

    let rec = &mut lbl_804799D8.x2A[pnum as usize];
    rec.start = table[rec.state as usize][0] as u8;
    rec.end = table[rec.state as usize][1] as u8;
    rec.r#loop = table[rec.state as usize][2] as u8;

    if rec.cur < rec.start {
        rec.cur = rec.start;
    }

    if rec.cur < rec.end {
        rec.cur += 1;
    } else {
        rec.done = 1;
        if rec.r#loop != 0 {
            rec.cur = rec.start;
        } else {
            rec.cur = rec.end;
        }
    }

    let state = lbl_804799D8.x2A[pnum as usize].state;
    if state == 0 && lbl_804799D8.x44[pnum as usize] == 6 {
        if lbl_804799D8.x2A[pnum as usize].done != 0 && (*tm).x4B8[pnum as usize].x0 != 0 {
            lbl_804799D8.x2A[pnum as usize].state = 1;
            lbl_804799D8.x2A[pnum as usize].done = 0;
        }
    }

    let state = lbl_804799D8.x2A[pnum as usize].state;
    if state == 1 {
        if lbl_804799D8.x2A[pnum as usize].done != 0 {
            lbl_804799D8.x2A[pnum as usize].state = 2;
            lbl_804799D8.x2A[pnum as usize].done = 0;
        }
    }

    fn_8019044C(jobj, lbl_804799D8.x2A[pnum as usize].cur as f32);
}

/// Updates visibility and position of a tournament menu JObj.
pub unsafe extern "C" fn fn_801973F8(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_801973F8");
    let tm = gm_GetTournamentData();
    let pnum = fn_8018F62C(&*gobj) as i32;
    let jobj = gobj_jobj(gobj);

    let cond = gmTournament_IsPlayerSetupOption();

    if cond == 0 {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        return;
    }

    ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);

    if pad_err(pnum) != 0 && (*tm).x4B8[pnum as usize].x0 != 1 {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        return;
    }

    let player_count = (*tm).x30;
    let x = gmTournament_GetPlayerX(player_count, pnum);

    fn_8018FDC4(jobj, 4.5f32 + x, 5.5, 666.0);

    if lbl_804799D8.x2A[pnum as usize].state == 4 {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
    }
}

/// Updates visibility of a tournament menu JObj based on current menu state.
pub unsafe extern "C" fn fn_801975C8(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_801975C8");
    let tmdata = gm_GetTournamentData();
    let idx = fn_8018F62C(&*gobj);
    let jobj = gobj_jobj(gobj);

    let in_range = gmTournament_IsPlayerSetupOption();

    if in_range == 0 {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        return;
    }

    ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);

    if ext::HSD_PadMasterStatus[idx as u8 as usize].err != 0 && (*tmdata).x4B8[idx as usize].x0 != 1 {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        return;
    }

    if lbl_804799D8.x2A[idx as usize].state == 4 {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
    }
}

/// Updates visibility and animation of tournament menu JObj.
pub unsafe extern "C" fn fn_801976D4(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_801976D4");
    gm_GetTournamentData();
    let jobj = gobj_jobj(gobj);
    let is_in_range = gmTournament_IsPlayerSetupOption();
    if is_in_range == 0 {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
    } else {
        ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
        let counter = addr_of_mut!(lbl_804799D8.x1C);
        if *counter > 0x3C {
            *counter = 0x3C;
        }
        fn_8019044C(jobj, *counter as f32);
        *counter = (*counter).wrapping_add(1);
    }
}

pub unsafe extern "C" fn fn_801977AC(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_801977AC");
    let tm = gm_GetTournamentData();
    let pnum = fn_8018F62C(&*gobj) as i32;
    let jobj = gobj_jobj(gobj);

    let in_range = gmTournament_IsPlayerSetupOption();

    if in_range == 0 {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        return;
    }

    ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);

    let players = (*tm).x30;
    let x = gmTournament_GetPlayerX(players, pnum);

    fn_8018FDC4(jobj, 0.3f32 + x, 12.6, 666.0);

    if lbl_804799D8.x2A[pnum as usize].state == 4 {
        if lbl_804799D8.x1D[pnum as usize] < 0x28 {
            lbl_804799D8.x1D[pnum as usize] += 1;
        }
        set_ty(jobj, lbl_803DA0D0.bounce_y[lbl_804799D8.x1D[pnum as usize] as usize]);
    } else {
        lbl_804799D8.x1D[pnum as usize] = 0;
    }

    if pad_err(pnum) != 0 && (*tm).x4B8[pnum as usize].x0 != 1 {
        ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);

        let players = (*tm).x30;
        let x = gmTournament_GetPlayerX(players, pnum);

        fn_8018FDC4(jobj, 0.3f32 + x, 12.6, 666.0);
    }
}

pub unsafe extern "C" fn fn_80197AF0(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_80197AF0");
    let tm = gm_GetTournamentData();
    let pnum = fn_8018F62C(&*gobj) as i32;
    let jobj = gobj_jobj(gobj);

    let in_range = gmTournament_IsPlayerSetupOption();

    if in_range == 0 {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        return;
    }

    ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);

    if ext::HSD_PadMasterStatus[pnum as u8 as usize].err != 0 && (*tm).x4B8[pnum as usize].x0 != 1 {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        return;
    }

    if (*tm).x4B8[pnum as usize].x0 == 1 {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
    }

    let players = (*tm).x30;
    let mut x: f32 = 0.0;
    gmTournament_SetPlayerX(&mut x, players, pnum);

    fn_8018FDC4(jobj, x, 666.0, 666.0);

    if lbl_804799D8.x44[pnum as usize] != 6
        || lbl_804799D8.x2A[pnum as usize].state == 1
        || lbl_804799D8.x2A[pnum as usize].state == 2
        || lbl_804799D8.x2A[pnum as usize].state == 4
    {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
    }

    if (lbl_804799D8.x12[pnum as usize] as u32) < 0x258 {
        lbl_804799D8.x12[pnum as usize] += 1;
    } else {
        lbl_804799D8.x12[pnum as usize] = 0;
    }
    fn_8019044C(jobj, lbl_804799D8.x12[pnum as usize] as f32);
}

/// Updates visibility of a menu JObj based on current menu option.
pub unsafe extern "C" fn fn_80197D4C(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_80197D4C");
    gm_GetTournamentData();
    let jobj = gobj_jobj(gobj);
    let cond = gmTournament_IsPlayerSetupOption();
    if cond == 0 {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        return;
    }
    ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
    let counter = addr_of_mut!(lbl_804799D8.x29);
    fn_8019044C(jobj, lbl_804799D8.x29 as f32);
    if (*counter as u32) < 10 {
        *counter += 1;
    }
}

/// Updates player HUD element visibility and position.
pub unsafe extern "C" fn fn_80197E18(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_80197E18");
    let data = gm_GetTournamentData();
    let pnum = fn_8018F62C(&*gobj) as i32;
    let jobj = gobj_jobj(gobj);

    let in_range = gmTournament_IsPlayerSetupOption();

    if in_range == 0 {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        return;
    }

    ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);

    if ext::HSD_PadMasterStatus[pnum as u8 as usize].err != 0 && (*data).x4B8[pnum as usize].x0 != 1 {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        return;
    }

    if (*data).x4B8[pnum as usize].x0 != 1 {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
    }

    let mut x_pos: f32 = 0.0;
    gmTournament_SetPlayerX(&mut x_pos, (*data).x30, pnum);
    fn_8018FDC4(jobj, x_pos, 666.0, 666.0);
}

/// Updates JObj visibility based on menu option and player state.
pub unsafe extern "C" fn fn_80197FD8(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_80197FD8");
    let tm_data = gm_GetTournamentData();
    let port = fn_8018F62C(&*gobj) as i32;
    let jobj = gobj_jobj(gobj);

    let option_in_range = gmTournament_IsPlayerSetupOption();

    if option_in_range == 0 {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        return;
    }

    ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);

    let port_u8 = port as u8;
    if ext::HSD_PadMasterStatus[port_u8 as usize].err != 0 {
        if (*tm_data).x4B8[port as usize].x0 != 1 {
            ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
            return;
        }
    } else if lbl_804799D8.x2A[port as usize].state == 4 {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
    }

    let num_players = (*tm_data).x30;
    let mut x_position: f32 = 0.0;
    gmTournament_SetPlayerX(&mut x_position, num_players, port);

    fn_8018FDC4(jobj, x_position, 666.0, 666.0);
}

/// Updates player cursor animation and position.
pub unsafe extern "C" fn fn_801981A0(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_801981A0");
    let state = d8();
    let data = gm_GetTournamentData();
    let pnum = fn_8018F62C(&*gobj) as i32;
    let jobj = gobj_jobj(gobj);

    let in_range = gmTournament_IsPlayerSetupOption();

    if in_range == 0 {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        return;
    }

    ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);

    if ext::HSD_PadMasterStatus[pnum as u8 as usize].err != 0 && (*state).x2A[pnum as usize].state != 4 {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        return;
    }

    if (*state).x44[pnum as usize] == 6 {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
    }

    let mut x_pos: f32 = 0.0;
    gmTournament_SetPlayerX(&mut x_pos, (*data).x30, pnum);

    fn_8018FDC4(jobj, x_pos, 666.0, 0.01);

    let mut counter = (*state).x21[pnum as usize];
    counter = ((counter as i32 + 1) % 11) as u8;
    (*state).x21[pnum as usize] = counter;

    if (*state).x44[pnum as usize] == 7 {
        fn_8019044C(jobj, (*state).x21[pnum as usize] as f32);
        return;
    }

    fn_8019044C(jobj, ((*state).x21[pnum as usize] as i32 + 0x14) as f32);
}

/// Updates the visibility and position of a player's controller indicator.
pub unsafe extern "C" fn fn_801983E4(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_801983E4");
    let tm = gm_GetTournamentData();
    let pnum = fn_8018F62C(&*gobj) as i32;
    let jobj = gobj_jobj(gobj);

    let cond = gmTournament_IsPlayerSetupOption();

    if cond == 0 {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        return;
    }

    ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);

    if pad_err(pnum) == 0 || (*tm).x4B8[pnum as usize].x0 == 1 {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        return;
    }

    let player_count = (*tm).x30;
    let mut x: f32 = 0.0;
    gmTournament_SetPlayerX(&mut x, player_count, pnum);

    fn_8018FDC4(jobj, x, 666.0, 0.01);
}

pub unsafe extern "C" fn fn_80198584(results: *mut ResultsData) {
    melee_core::ct!("fn_80198584");
    let cur_option = (*gm_GetTournamentData()).cur_option;
    let jobj = (*results).x28;
    if cur_option == 0x1D {
        ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
    } else {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
    }
}

pub unsafe extern "C" fn fn_801985D4(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_801985D4");
    let d = d8();
    let jobj = gobj_jobj(gobj);

    gm_GetTournamentData();
    let in_range = (*gm_GetTournamentData()).cur_option >= 0x1F && (*gm_GetTournamentData()).cur_option <= 0x27;

    if !in_range {
        set_tz(jobj, 10000.0);
    } else {
        set_tz(jobj, 0.0);

        if (*d).x0 < 0xA {
            lbl_804D667C = 1;
            return;
        }

        ext::HSD_JObjAnimAll(jobj);
        let child = jchild(jobj);

        if ext::HSD_DObjGetFlags(ext::HSD_JObjGetDObj(child)) & 1 != 0 {
            lbl_804D667C = 1;
        } else {
            lbl_804D667C = 0;
        }

        HSD_JObjGetTranslation(&*child, &mut (*d).x64);
        HSD_JObjGetScale(&*child, &mut (*d).x70);
    }
}

/// Animates and positions the tournament bracket results overlay JObj.
pub unsafe extern "C" fn fn_80198824(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_80198824");
    let d = d8();
    let jobj = gobj_jobj(gobj);

    gm_GetTournamentData();
    let in_range = (*gm_GetTournamentData()).cur_option >= 0x1F && (*gm_GetTournamentData()).cur_option <= 0x27;

    if !in_range {
        set_tz(jobj, 10000.0);
    } else {
        set_tz(jobj, 0.0);

        if lbl_804D667C != 0 {
            set_tz(jobj, 10000.0);
        } else {
            let frame_ctr = addr_of_mut!((*d).x4);
            let min_frame = (*d).x8 as u32;
            if *frame_ctr < min_frame {
                *frame_ctr = min_frame;
            }
            if *frame_ctr < (*d).xC as u32 {
                *frame_ctr = (*frame_ctr).wrapping_add(1);
            }

            ext::HSD_JObjReqAnimAll(jobj, *frame_ctr as f32);
            ext::HSD_JObjAnimAll(jobj);

            // HSD_JObjSetTranslate(jobj, &d8->x64) / HSD_JObjSetScale(jobj, &d8->x70)
            struct D;
            impl melee_types::inline::JObjDirty for D {
                fn set_mtx_dirty(&mut self, j: &mut HSD_JObj) {
                    if !melee_types::inline::HSD_JObjMtxIsDirty(j) {
                        unsafe { ext::HSD_JObjSetMtxDirtySub(j) };
                    }
                }
                fn set_mtx_dirty_sub(&mut self, j: &mut HSD_JObj) {
                    unsafe { ext::HSD_JObjSetMtxDirtySub(j) };
                }
            }
            melee_types::inline::HSD_JObjSetTranslate(&mut *jobj, &(*d).x64, &mut D);
            melee_types::inline::HSD_JObjSetScale(&mut *jobj, &(*d).x70, &mut D);
        }
    }
}

/// Initializes tournament mode match data (per-player counters and anim records).
pub unsafe extern "C" fn fn_80198BA0() {
    melee_core::ct!("fn_80198BA0");
    let d = d8();
    let td = gm_GetTournamentData();

    (*d).x0 = 0;
    (*d).x10 = 0;

    if (*td).x2C != 0 {
        (*d).x1A = 0x50;
    } else {
        (*d).x1A = 0x5A;
    }

    (*d).x1C = 0;

    let mut i: i32 = 0;
    while i < (*td).x30 as i32 {
        let iu = i as usize;
        (*d).x25[iu] = 0;
        (*d).x12[iu] = 0;
        (*d).x1D[iu] = 0;
        (*d).x2A[iu].start = 0;
        (*d).x2A[iu].cur = 0;
        (*d).x2A[iu].end = 0x1E;
        (*d).x2A[iu].done = 0;
        (*d).x2A[iu].r#loop = 0;
        (*d).x2A[iu].state = 0;
        (*d).x44[iu] = 6;
        i += 1;
    }
}

/// Initializes tournament mode text displays.
pub unsafe extern "C" fn fn_80198C60() {
    melee_core::ct!("fn_80198C60");
    let td = gm_GetTournamentData();
    (*td).x524[2] = ext::HSD_SisLib_803A6754(0, lbl_804D663C);
    let text = (*td).x524[2];
    (*text).font_size.x = 0.054945;
    (*text).font_size.y = 0.08;
    (*(*td).x524[2]).default_alignment = 1;
    (*(*td).x524[2]).default_kerning = 1;

    (*td).x524[3] = ext::HSD_SisLib_803A6754(0, lbl_804D663C);
    (*(*td).x524[3]).default_alignment = 1;
    (*(*td).x524[3]).default_kerning = 1;
    ext::HSD_SisLib_803A6B98((*td).x524[3], 320.0, 250.0, b"    \0".as_ptr() as *const c_char);
    ext::HSD_SisLib_803A7548((*td).x524[3], 0, 1.5, 1.5);
}

/// Initializes the scene rendering components for the gm_18A5 game mode.
pub unsafe extern "C" fn fn_80198D18() {
    melee_core::ct!("fn_80198D18");
    gm_GetTournamentData();
    let gobj = fn_80190174((*(*lbl_804D666C).cameras).desc);
    fn_801901F8((*(*lbl_804D666C).cameras).desc);
    fn_801902F0(gobj as usize as i32);
    fn_8019027C((*lbl_804D666C).lights as *mut c_void);
    fn_8019035C(0, *(*lbl_804D666C).models.add(5), 0, 0x1A, 2, 1, Some(fn_80196DBC), 0.0);
    fn_8019035C(0, *(*lbl_804D666C).models.add(4), 0, 0x1A, 2, 1, Some(fn_80196E30), 80.0);

    let td = gm_GetTournamentData();
    (*td).x524[2] = ext::HSD_SisLib_803A6754(0, lbl_804D663C);
    let text = (*td).x524[2];
    (*text).font_size.x = 0.054945;
    (*text).font_size.y = 0.08;
    (*(*td).x524[2]).default_alignment = 1;
    (*(*td).x524[2]).default_kerning = 1;

    (*td).x524[3] = ext::HSD_SisLib_803A6754(0, lbl_804D663C);
    (*(*td).x524[3]).default_alignment = 1;
    (*(*td).x524[3]).default_kerning = 1;
    ext::HSD_SisLib_803A6B98((*td).x524[3], 320.0, 250.0, b"    \0".as_ptr() as *const c_char);
    ext::HSD_SisLib_803A7548((*td).x524[3], 0, 1.5, 1.5);

    let gobj = ext::GObj_Create(0xE, 0x1A, 0);
    let fog = ext::HSD_FogLoadDesc((*(*lbl_804D666C).fogs).desc);
    ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_FogKind as u8, fog as *mut c_void);
    ext::GObj_SetupGXLink(gobj, Some(ext::HSD_GObj_FogCallback), 0, 0);
    fn_80198BA0();
}

/// Hides every model from the icon child down except the selected character's (`map` index).
unsafe fn select_icon_child(jobj2: &mut *mut HSD_JObj, map: i32) {
    set_tz(*jobj2, 10000.0);

    if map == 0 {
        set_tz(*jobj2, 0.0);

        let mut j = 1;
        while j <= 12 {
            *jobj2 = jnext(*jobj2);
            set_tz(*jobj2, 10000.0);
            j += 1;
        }
    } else {
        let mut j = 1;
        while j <= 12 {
            *jobj2 = jnext(*jobj2);
            set_tz(*jobj2, 10000.0);

            if map == j {
                set_tz(*jobj2, 0.0);

                j += 1;
                while j <= 12 {
                    *jobj2 = jnext(*jobj2);
                    set_tz(*jobj2, 10000.0);
                    j += 1;
                }
                break;
            }
            j += 1;
        }
    }
}

pub unsafe extern "C" fn fn_80198EBC() {
    melee_core::ct!("fn_80198EBC");
    let td = gm_GetTournamentData();
    gm_GetTournamentData();

    let gobj = fn_80190174((*(*lbl_804D666C).cameras).desc);
    fn_801901F8((*(*lbl_804D666C).cameras).desc);
    fn_801902F0(gobj as usize as i32);
    fn_8019027C((*lbl_804D666C).lights as *mut c_void);

    fn_8019035C(0, *(*lbl_804D666C).models.add(5), 0, 0x1A, 2, 1, Some(fn_80196DBC), 0.0);
    fn_8019035C(0, *(*lbl_804D666C).models.add(4), 0, 0x1A, 2, 1, Some(fn_80196E30), 80.0);
    fn_80198C60();

    let gobj = ext::GObj_Create(0xE, 0x1A, 0);
    let fog = ext::HSD_FogLoadDesc((*(*lbl_804D666C).fogs).desc);
    ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_FogKind as u8, fog as *mut c_void);
    ext::GObj_SetupGXLink(gobj, Some(ext::HSD_GObj_FogCallback), 0, 0);
    fn_80198BA0();

    let m = (*lbl_804D6674).models;
    let mut i: i32 = 0;
    while i < (*td).x30 as i32 {
        let iu = i as usize;
        let anim_rate: f32;
        if (*td).x4B8[iu].x0 == 0 {
            anim_rate = i as f32;
        } else if (*td).x4B8[iu].x0 == 1 {
            anim_rate = 4.0;
        } else {
            anim_rate = i as f32;
            (*td).x4B8[iu].x0 = 0;
            lbl_804799D8.x44[iu] = 6;
            lbl_804799D8.x2A[iu].state = 4;
        }

        let gobj = fn_8019035C(0, *m.add(12), 0, 0x1A, 2, 1, Some(fn_80196FFC), anim_rate);
        let jobj = gobj_jobj(gobj);
        fn_8018FBD8(&mut *gobj, i);
        let c = jchild(jobj);
        ext::HSD_MObjRemoveAnimByFlags((*(*c).u.dobj).mobj, 4);
        let c = jchild(jobj);
        let c = jnext(c);
        ext::HSD_MObjRemoveAnimByFlags((*(*c).u.dobj).mobj, 4);
        let c = jchild(jobj);
        let c = jnext(c);
        let c = jnext(c);
        ext::HSD_MObjRemoveAnimByFlags((*(*c).u.dobj).mobj, 4);

        let pos: f32;
        if (*td).x30 == 4 {
            pos = 73.0f32 + fmadds(240.0, i as f32, 152.0);
        } else if (*td).x30 == 3 {
            pos = 73.0f32 + fmadds(240.0, i as f32, 270.0);
        } else {
            pos = 73.0f32 + fmadds(240.0, 2.0f32 * i as f32, 270.0);
        }
        fn_8018ECA8(
            (*td).x4B8[iu].x6 as i32,
            (*td).x4B8[iu].x0 as i32,
            5,
            fmadds(0.99, pos - 320.0, -261.0),
            -42.0,
            5,
        );

        (*(*td).x524[2]).hidden = 1;

        let gobj = fn_8019035C(0, *m.add(10), 0, 0x1A, 2, 1, Some(fn_801973F8), anim_rate);
        let jobj = gobj_jobj(gobj);
        fn_8018FBD8(&mut *gobj, i);

        let mut jobj2 = jchild(jobj);
        select_icon_child(&mut jobj2, lbl_803DA0D0.icon_model_map[(*td).x4B8[iu].x1 as usize] as i32);

        let gobj = fn_8019035C(0, *m.add(4), 0, 0x1A, 2, 1, Some(fn_801975C8), anim_rate);
        let jobj = gobj_jobj(gobj);
        fn_8018FBD8(&mut *gobj, i);

        let pos: f32;
        if (*td).x30 as i32 == 4 {
            pos = fmadds(13.0, i as f32, -19.5);
        } else if (*td).x30 as i32 == 3 {
            pos = 6.5f32 + fmadds(13.0, i as f32, -19.5);
        } else {
            pos = 6.5f32 + fmadds(13.0, 2.0f32 * i as f32, -19.5);
        }
        // 2.8f - 1 ulp (0x4033332F)
        fn_8018FDC4(jobj, pos - 2.7999989986419678f64 as f32, 6.8, 0.001);

        let c = jchild(jobj);
        let jobj = jnext(c);
        fn_8019044C(jobj, fn_8018F71C((*td).x4B8[iu].x1 as i32, (*td).x4B8[iu].x3 as i32));

        let gobj = fn_8019035C(0, *m.add(9), 0, 0x1A, 2, 1, Some(fn_801977AC), anim_rate);
        let jobj = gobj_jobj(gobj);
        fn_8018FBD8(&mut *gobj, i);
        fn_8019044C(jobj, anim_rate);

        let gobj = fn_8019035C(0, *m.add(8), 0, 0x1A, 2, 1, Some(fn_80197AF0), 0.0);
        fn_8018FBD8(&mut *gobj, i);

        let gobj = fn_8019035C(0, *m.add(13), 0, 0x1A, 2, 1, Some(fn_80197E18), anim_rate);
        let jobj = gobj_jobj(gobj);
        fn_8018FBD8(&mut *gobj, i);

        if (*td).x4B8[iu].x0 == 1 {
            fn_8019044C(jobj, (*td).x4B8[iu].x4 as f32);
        } else {
            ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        }

        if (*td).x31 != 0 {
            let gobj = fn_8019035C(0, *m.add(11), 0, 0x1A, 2, 1, Some(fn_80197FD8), anim_rate);
            let jobj = gobj_jobj(gobj);
            fn_8018FBD8(&mut *gobj, i);
            fn_8019044C(jobj, (*td).x4B8[iu].x5 as f32);
        }

        let gobj = fn_8019035C(0, *m.add(7), 0, 0x1A, 2, 1, Some(fn_801981A0), anim_rate);
        let jobj = gobj_jobj(gobj);
        fn_8018FBD8(&mut *gobj, i);
        fn_8018FDC4(jobj, -2.5, 666.0, 0.02);

        let gobj = fn_8019035C(0, *m.add(14), 0, 0x1A, 2, 1, Some(fn_801983E4), anim_rate);
        fn_8018FBD8(&mut *gobj, i);
        i += 1;
    }

    fn_8019035C(0, *m.add(5), 0, 0x1A, 2, 1, Some(fn_801976D4), 0.0);

    let gobj = fn_8019035C(0, *m.add(0), 0, 0x1A, 2, 1, None, 0.0);
    let mut j16 = gobj_jobj(gobj);

    j16 = jchild(j16);
    j16 = jnext(j16);
    j16 = jnext(j16);
    j16 = jnext(j16);
    j16 = jnext(j16);
    j16 = jnext(j16);
    let mut j: i32 = 0;
    while j <= 40 {
        fn_8019044C(j16, (j + 10) as f32);
        lbl_803DA0D0.bounce_y[j as usize] = HSD_JObjGetTranslationY(&*j16);
        j += 1;
    }

    fn_8019035C(0, *m.add(6), 0, 0x1A, 2, 1, Some(fn_80197D4C), 0.0);
    fn_8019035C(
        0,
        *m.add(2),
        0,
        0x1A,
        3,
        1,
        Some(core::mem::transmute::<unsafe extern "C" fn(*mut ResultsData), unsafe extern "C" fn(*mut HSD_GObj)>(fn_80198584)),
        0.0,
    );
}

pub unsafe extern "C" fn fn_80199AF0() {
    melee_core::ct!("fn_80199AF0");
    let td1 = gm_GetTournamentData();
    let td2 = gm_GetTournamentData();
    let mut local1: i32 = 0;
    let mut local2: i32 = 0;

    let mut result = fn_8018F508(&mut local1);
    if result == 1 {
        result = 1;
    } else if (*td2).x33 == 5 {
        result = 2;
    } else {
        result = 0;
    }
    let mut mode = result;

    result = fn_8018F508(&mut local2);
    let mut slot: i32;
    if result == 1 {
        slot = local2;
    } else {
        slot = -1;
        let mut i = 0;
        while i < 4 {
            let p = &gm_80477738.player_standings[i];
            if p.pkind != 3 && p.is_small_loser == 0 {
                slot = i as i32;
                break;
            }
            i += 1;
        }
    }

    let bracket_idx = fn_8018F74C();
    gm_GetTournamentData();

    let gobj = fn_80190174((*(*lbl_804D666C).cameras).desc);
    fn_801901F8((*(*lbl_804D666C).cameras).desc);
    fn_801902F0(gobj as usize as i32);
    fn_8019027C((*lbl_804D666C).lights as *mut c_void);
    fn_8019035C(0, *(*lbl_804D666C).models.add(5), 0, 0x1A, 2, 1, Some(fn_80196DBC), 0.0);
    fn_8019035C(0, *(*lbl_804D666C).models.add(4), 0, 0x1A, 2, 1, Some(fn_80196E30), 80.0);

    fn_80198C60();

    let gobj = ext::GObj_Create(0xE, 0x1A, 0);
    let fog = ext::HSD_FogLoadDesc((*(*lbl_804D666C).fogs).desc);
    ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_FogKind as u8, fog as *mut c_void);
    ext::GObj_SetupGXLink(gobj, Some(ext::HSD_GObj_FogCallback), 0, 0);
    fn_80198BA0();

    if (*td1).x33 == 5 {
        mode = 2;
    }

    if mode != 2 {
        if lbl_80473AB8[bracket_idx as usize].x18 == 0 {
            return;
        }
    }

    fn_8019035C(0, *(*lbl_804D6670).models.add(0), mode, 0x1A, 3, 1, Some(fn_801985D4), 0.0);
    let gobj = fn_8019035C(0, *(*lbl_804D6670).models.add(2), 0, 0x1A, 3, 1, Some(fn_80198824), 0.0);

    let mut jobj = jchild(gobj_jobj(gobj));

    lbl_804799D8.x8 = mode * 0x14;
    lbl_804799D8.xC = if mode == 2 { 0x96 } else { mode * 0x14 + 0x13 };

    // slot index is also reused as the inner loop counter by the C (dead afterwards)
    let map = lbl_803DA0D0.icon_model_map[(*td1).x4B8[slot as usize].x1 as usize] as i32;
    select_icon_child(&mut jobj, map);
}

pub unsafe extern "C" fn fn_8019A158() {
    melee_core::ct!("fn_8019A158");
    let mut local1: i32 = 0;
    let mut local2: i32 = 0;
    let td1 = gm_GetTournamentData();
    let base = d8();
    (*base).x48 = addr_of_mut!(gm_80477738);
    let x48_ptr = addr_of_mut!((*base).x48);
    let mut mode: i32 = 0;
    (*base).x0 = 0;

    let td2 = gm_GetTournamentData();

    let mut result = fn_8018F508(&mut local1);
    if result == 1 {
        mode = 1;
    } else if (*td2).x33 == 5 {
        mode = 2;
    }

    let me = *x48_ptr;
    result = fn_8018F508(&mut local2);
    let mut state_slot: i32;
    if result == 1 {
        state_slot = local2;
    } else {
        state_slot = -1;
        let mut i = 0;
        while i < 4 {
            if (*me).player_standings[i].pkind != 3 && (*me).player_standings[i].is_small_loser == 0 {
                state_slot = i as i32;
                break;
            }
            i += 1;
        }
    }
    let mut sel = state_slot;

    let bracket_idx = fn_8018F74C();

    let mut k = 0;
    while k < 20 {
        (*base).x4E[k] = 0;
        k += 1;
    }

    if mode == 1 {
        let bracket = &mut lbl_80473AB8[bracket_idx as usize];
        let mut i = 0;
        while i < 4 {
            if i == state_slot {
                bracket.slots[state_slot as usize].x4C = 0;
            } else {
                bracket.slots[i as usize].x4C = 3;
            }
            i += 1;
        }
    } else if (*td1).x2D == 1 {
        let bracket = &mut lbl_80473AB8[bracket_idx as usize];
        let mut i = 0;
        while i < 4 {
            if bracket.slots[i].x4E == 3 {
                bracket.slots[i].x4C = 3;
            } else {
                let standing = &mut (**x48_ptr).player_standings[i];
                let v = standing.is_small_loser;
                standing.is_big_loser = v;
                bracket.slots[i].x4C = v;
                if (**x48_ptr).player_standings[i].is_small_loser == 0 {
                    sel = i as i32;
                }
            }
            i += 1;
        }
    } else {
        let mut counter: i32 = 0;
        let mut i = 0;
        while i < 4 {
            if lbl_80473AB8[bracket_idx as usize].slots[i].x4E == 3 {
                lbl_80473AB8[bracket_idx as usize].slots[i].x4C = 4;
            } else {
                lbl_80473AB8[bracket_idx as usize].slots[i].x4C = counter as u8;
                counter += 1;
            }
            i += 1;
        }

        match counter {
            2 => {
                let rand_val = ext::HSD_Randi(x51(bracket_idx, 0) + x51(bracket_idx, 1));
                if rand_val < x51(bracket_idx, 0) {
                    lbl_80473AB8[bracket_idx as usize].slots[0].x4C = 0;
                    lbl_80473AB8[bracket_idx as usize].slots[1].x4C = 1;
                } else {
                    lbl_80473AB8[bracket_idx as usize].slots[0].x4C = 1;
                    lbl_80473AB8[bracket_idx as usize].slots[1].x4C = 0;
                }
            }
            3 => fn_80196684(bracket_idx),
            4 => fn_801967E0(bracket_idx),
            _ => {}
        }

        let mut p = 0;
        while p < 4 {
            let v = lbl_80473AB8[bracket_idx as usize].slots[p].x4C;
            (**x48_ptr).player_standings[p].is_big_loser = v;
            (**x48_ptr).player_standings[p].is_small_loser = v;
            p += 1;
        }

        let mut i = 0;
        while i < 4 {
            if lbl_80473AB8[bracket_idx as usize].slots[i].x4C == 0 {
                sel = i as i32;
            }
            i += 1;
        }
    }

    (*base).x4C = sel as u8;
    (*base).x4D = lbl_80473AB8[bracket_idx as usize].slots[sel as usize].x4E;

    if (*base).x4D == 0 && lbl_80473AB8[bracket_idx as usize].x18 != 0 {
        let s = (*base).x4C;
        let val = (*td1).x4B8[s as usize].x6;
        if val <= 0x78 {
            ext::gm_80167858(s as i32, val as i32, 0x1F, 0x78);
        } else {
            ext::gm_80167858(s as i32, 0x78, 0x1F, 0x78);
        }
    }

    // cursor = (u8*) &lbl_80473AB8[bracket_idx] + sel * 0x2C; model_idx = cursor[0x50]
    let model_idx = lbl_80473AB8[bracket_idx as usize].slots[sel as usize].x50;
    fn_8018F00C((*base).x4E.as_mut_ptr() as *mut c_char, (*x37(td1, model_idx as i32)).x9 as i32);
}

pub unsafe extern "C" fn fn_8019A71C(state: *mut i32, _unused1: u32, _unused2: u32) {
    melee_core::ct!("fn_8019A71C");
    let counter = addr_of_mut!(lbl_804799D8.x0);

    match *state {
        0x13 => fn_8019B458(state),
        _ => {}
    }

    if *state > 0x14 {
        *counter = (*counter).wrapping_add(1);
    }
    if *counter >= 0x14 {
        *state = 0x15;
    }

    if *counter == 0x1E {
        *state = 0x16;
    }

    if *counter > 0x1E {
        *state = 0x17;
    }

    if *counter >= 0x3C {
        *state = 0x18;
    }

    if *counter >= 0x46 {
        *state = 0x19;
    }

    if *counter >= 0x5A {
        fn_8018EC48();
        *state = 0x1A;
        fn_8018EC7C();
        if fn_8018F508(null_mut()) == 1 {
            *state = 0x1F;
            return;
        }
        *state = 0x1B;
    }
}

pub unsafe extern "C" fn gm_8019A828() {
    melee_core::ct!("gm_8019A828");
    (*gm_GetTournamentData()).x32 = 0;
    lbl_804D6678 = 1;
    (*gm_GetTournamentData()).cur_option = 0x1B;
}

#[inline]
unsafe fn get_pad_error(i: i32) -> u8 {
    ext::HSD_PadMasterStatus[i as u8 as usize].err as u8
}

/// The stage-selection start sequence shared by the confirm paths of `fn_8019A86C`.
unsafe fn start_stage_sequence(t: *mut TmData) -> bool {
    (*t).x2D = 1;
    let stype = (*t).stage_selection_type;
    let mut cond = if (stype == 2 && (*t).x32 == 0) || stype == 3 { 1 } else { 0 };
    if cond != 0 {
        (*t).x32 = 1;
        ext::gm_SetNextGameModeStateId(3);
        ext::gm_801A4B60();
        return true;
    }
    cond = if (stype == 0 && (*t).x32 == 0) || stype == 1 { 1 } else { 0 };
    if cond != 0 {
        (*t).x32 = 1;
        (*t).x28 = lbl_804D4190 as u32;
    }
    ext::gm_SetNextGameModeStateId(4);
    ext::gm_801A4B60();
    false
}

pub unsafe extern "C" fn fn_8019A86C(tm: *mut TmData, _arg1: u32, arg2: u32) {
    melee_core::ct!("fn_8019A86C");
    let mut ready_count: i32 = 0;
    let mut pad_err_flag: i32 = 0;

    match (*tm).cur_option {
        0x1B => fn_8019B81C(tm as *mut i32),
        _ => {}
    }

    if (*tm).cur_option == 0x1D {
        lbl_804799D8.x0 += 1;
        if (arg2 & 0x600) != 0 || lbl_804799D8.x0 >= 0x12C {
            sfxBack();
            fn_8018EC48();
            (*tm).x2D = 0;
            (*tm).cur_option = 0x1F;
        }
        if arg2 & 0x1100 != 0 {
            sfxForward();
            let t = gm_GetTournamentData();
            start_stage_sequence(t);
        }
    } else {
        let mut i: i32 = 0;
        while i < (*tm).x30 as i32 {
            if (*tm).x4B8[i as usize].x0 == 1 {
                ready_count += 1;
            } else {
                let err = get_pad_error(i);
                if err as i8 != 0 {
                    pad_err_flag = 1;
                }
                let state = lbl_804799D8.x2A[i as usize].state;
                if ((state == 2 && lbl_804799D8.x2A[i as usize].cur as u32 >= 0x3C)
                    || (state == 4 && lbl_804799D8.x2A[i as usize].cur as u32 == 0x82))
                    && err as i8 == 0
                {
                    ready_count += 1;
                }
            }
            i += 1;
        }

        let cond3 = if (fn_8018F640(4) & 0x1160) != 0 && (fn_8018F6A8(4) & 0x1160) == 0x1160 { 1 } else { 0 };
        if cond3 != 0 && pad_err_flag != 0 {
            sfxBack();
            ext::HSD_SisLib_803A5E70();
            ext::mn_8022F138(0x19, 0x1C);
            ext::mn_8022F138(0x12, 0x15);
            ext::mn_8022F268();
            ext::gm_801A4B60();
            ext::gm_ChangeGameModeAfterCurrentScene(GM_MENU as i32);
            return;
        }

        if ready_count == (*tm).x30 as i32 {
            lbl_804799D8.x0 += 1;
            if lbl_804799D8.x0 >= 0x1E {
                let mut i: i32 = 0;
                while i < (*tm).x30 as i32 {
                    if (*tm).x4B8[i as usize].x0 == 0 && lbl_804799D8.x2A[i as usize].state == 4 {
                        (*tm).x4B8[i as usize].x0 = 3;
                    }
                    i += 1;
                }

                let t2 = gm_GetTournamentData();
                let bi = fn_8018F74C();
                let mut hmn_count: i32 = 0;
                let mut active_count: i32 = 0;

                let mut i: i32 = 0;
                while i < 4 {
                    lbl_80473AB8[bi as usize].slots[i as usize].x4E = (*t2).x4B8[i as usize].x0;
                    i += 1;
                }

                let mut i: i32 = 0;
                while i < 4 {
                    let st = (*tm).x4B8[i as usize].x0;
                    if st == 0 {
                        hmn_count += 1;
                    }
                    if st != 3 {
                        active_count += 1;
                    }
                    i += 1;
                }

                if active_count <= 1 {
                    fn_8018EC48();
                    (*tm).x2D = 0;
                    (*tm).cur_option = 0x1F;
                    return;
                }
                if hmn_count == 0 {
                    (*tm).cur_option = 0x1D;
                    return;
                }

                let t3 = gm_GetTournamentData();
                start_stage_sequence(t3);
            }
        } else {
            lbl_804799D8.x0 = 0;

            let mut i: i32 = 0;
            while i < (*tm).x30 as i32 {
                let iu = i as usize;
                if ext::HSD_PadMasterStatus[i as u8 as usize].err == 0 && (*tm).x4B8[iu].x0 == 0 {
                    let buttons = fn_8018F640(i);

                    if buttons & (PAD_BUTTON_A | PAD_BUTTON_START) != 0 {
                        sfxForward();
                        if lbl_804799D8.x44[iu] == 7 {
                            lbl_804799D8.x44[iu] = 6;
                        } else if lbl_804799D8.x44[iu] == 8 {
                            let np = (*gm_GetTournamentData()).x30;
                            let mut count4: i32 = 0;
                            let mut j: i32 = 0;
                            while j < np as i32 {
                                if lbl_804799D8.x2A[j as usize].state == 4 {
                                    count4 += 1;
                                }
                                j += 1;
                            }
                            if count4 < (*tm).x30 as i32 - 1 {
                                lbl_804799D8.x44[iu] = 6;
                                lbl_804799D8.x2A[iu].state = 4;
                            }
                        } else {
                            let pstate = lbl_804799D8.x2A[iu].state;
                            if pstate == 4 {
                                lbl_804799D8.x2A[iu].state = 5;
                            } else if pstate == 0 || pstate == 3 || pstate == 5 {
                                let val = (*tm).x4B8[iu].x6;
                                if val as u32 <= 0x78 {
                                    ext::gm_80167858(i, val as i32, 0xB, 0x14);
                                } else {
                                    ext::gm_80167858(i, 0x78, 0xB, 0x14);
                                }
                                lbl_804799D8.x2A[iu].state = 1;
                            }
                        }
                    } else if buttons & PAD_BUTTON_X != 0 {
                        if lbl_804799D8.x44[iu] != 6 {
                            sfxBack();
                            lbl_804799D8.x44[iu] = 6;
                        } else {
                            let pstate2 = lbl_804799D8.x2A[iu].state;
                            if pstate2 == 0 || pstate2 == 3 || pstate2 == 5 {
                                let np2 = (*gm_GetTournamentData()).x30;
                                let mut count5: i32 = 0;
                                let mut k: i32 = 0;
                                while k < np2 as i32 {
                                    if lbl_804799D8.x2A[k as usize].state == 4 {
                                        count5 += 1;
                                    }
                                    k += 1;
                                }
                                if count5 < (*tm).x30 as i32 - 1 {
                                    sfxBack();
                                    lbl_804799D8.x44[iu] = 7;
                                }
                            } else if pstate2 == 2 {
                                sfxBack();
                                lbl_804799D8.x2A[iu].state = 3;
                                lbl_804799D8.x2A[iu].done = 0;
                            }
                        }
                    } else if (buttons & PAD_STICK_UP) != 0 || (buttons & PAD_BUTTON_UP) != 0 {
                        if lbl_804799D8.x44[iu] == 8 {
                            sfxMove();
                            lbl_804799D8.x44[iu] = 7;
                        }
                    } else if ((buttons & PAD_STICK_DOWN) != 0 || (buttons & PAD_BUTTON_DOWN) != 0) && lbl_804799D8.x44[iu] == 7 {
                        sfxMove();
                        lbl_804799D8.x44[iu] = 8;
                    }
                }
                i += 1;
            }
        }
    }
}

/// Tournament match timer display/audio state machine.
pub unsafe extern "C" fn fn_8019AF50(arg0: *mut i32, _arg1: u32, _arg2: u32) {
    melee_core::ct!("fn_8019AF50");
    let tm = arg0 as *mut TmData;
    let mut sp_buf: [u8; 20] = lbl_803B7D04;
    let bracket_idx = fn_8018F74C();

    match *arg0 {
        0x1F => fn_8019B860(tm),
        _ => {}
    }

    if (*tm).x33 == 6 {
        if lbl_804D6680[0] == 2 {
            if ext::lbAudioAx_80023730() == 0 {
                ext::lbAudioAx_80023F28(ext::lbAudioAx_8002305C(fn_8018F6FC(lbl_804D6680[1] as i32), 0));
                lbl_804D6680[0] = 3;
            }
        } else if lbl_804D6680[0] == 0 {
            let mut j: i32 = 0;
            let mut n = 4;
            while n != 0 {
                let s = &lbl_80473AB8[bracket_idx as usize].slots[j as usize];
                if s.x30 != 0 && s.x4C == 0 {
                    lbl_804D6680[1] = s.x4D;
                    break;
                }
                j += 1;
                n -= 1;
            }

            ext::lbAudioAx_80023F28(ext::fn_80160400(fn_8018F6FC(lbl_804D6680[1] as i32) as CharacterKind) as i32);
            lbl_804D6680[0] = 1;
        } else if lbl_804D6680[0] == 1 {
            let timer_ptr = addr_of_mut!(lbl_804D6680[2]);
            if *timer_ptr > 0x3C {
                lbl_804D6680[0] = 2;
            }
            *timer_ptr = (*timer_ptr).wrapping_add(1);
        }
    }

    let buttons: u32 = if lbl_804799D8.x4D != 1 {
        ext::gm_GetButtonsTriggered(lbl_804799D8.x4C) as u32
    } else {
        ext::gm_GetButtonsTriggered(4) as u32
    };

    if lbl_80473AB8[bracket_idx as usize].x18 != 0 {
        if lbl_804799D8.x0 < 0xFA {
            lbl_804799D8.x0 += 1;
            if lbl_804799D8.x0 >= 0x64 {
                let count = (lbl_804799D8.x0 - 0x64) / 15;
                let mut i: u32 = 0;
                while i < count {
                    sp_buf[(i * 2) as usize] = lbl_804799D8.x4E[(i * 2) as usize];
                    sp_buf[(i * 2 + 1) as usize] = lbl_804799D8.x4E[(i * 2 + 1) as usize];
                    i += 1;
                }
            }
            ext::HSD_SisLib_803A70A0((*tm).x524[3], 0, sp_buf.as_mut_ptr() as *mut c_char);
        } else {
            lbl_804799D8.x0 += 2;
            if lbl_804799D8.x4D != 1 {
                if lbl_804799D8.x0 > 0xFA {
                    lbl_804799D8.x0 = 0xFA;
                }
            }
            ext::HSD_SisLib_803A70A0((*tm).x524[3], 0, lbl_804799D8.x4E.as_mut_ptr() as *mut c_char);
        }
    } else if lbl_804799D8.x0 < 0xFA {
        lbl_804799D8.x0 = 0xFA;
    }

    if *arg0 == 0x27 {
        if lbl_804799D8.x0 >= 0xFA {
            if (*tm).x33 == 6 {
                if lbl_804799D8.x0 >= 0x1C20 || (buttons & (PAD_BUTTON_A | PAD_BUTTON_START)) != 0 {
                    ext::gm_ChangeGameModeAfterCurrentScene(GM_MENU as i32);
                    ext::gm_801A4B60();
                }
            } else {
                fn_8018EC48();
                *arg0 = 0x13;
            }
        }
    }
}

/// Per-frame update for tournament mode menu.
pub unsafe extern "C" fn gm_Scene_TouBracket_OnFrame() {
    melee_core::ct!("gm_Scene_TouBracket_OnFrame");
    let mut sp: [i32; 13] = [0; 13];

    let data = gm_GetTournamentData();
    let arg1 = fn_8018F674(4);
    let arg2 = fn_8018F640(4);

    if (*gm_GetTournamentData()).cur_option >= 0x13 && (*gm_GetTournamentData()).cur_option <= 0x1A {
        fn_8019A71C(data as *mut i32, arg1, arg2);
    } else {
        let cond = gmTournament_IsPlayerSetupOption();
        if cond != 0 {
            fn_8019A86C(data, arg1, arg2);
        } else {
            fn_8019AF50(data as *mut i32, arg1, arg2);
        }
    }

    let val = (*data).stage_selection_type;
    let cond = if (val == 0 && (*data).x32 == 0) || val == 1 { 1 } else { 0 };
    let stage = if cond != 0 { lbl_804D4190 } else { (*data).x28 as i32 };
    sp[0] = stage;

    let mut i: usize = 0;
    while i < 4 {
        sp[1 + i] = (*data).x4B8[i].x0 as i32;
        sp[5 + i] = fn_8018F6FC((*data).x4B8[i].x1 as i32);
        sp[9 + i] = (*data).x4B8[i].x3 as i32;
        i += 1;
    }

    fn_8019BA04(sp.as_mut_ptr());
}

#[inline]
unsafe fn fn_8019B458_UpdateRank(tm: *mut TmData, d: *mut Lbl804799D8_t) {
    let mut rank: i32 = 0;
    (*d).x0 = 0;
    (*tm).pad_x34[0] = (*tm).x33;

    let td = gm_GetTournamentData();
    let entrants = (*td).entrants;
    let x24 = (*td).x24;

    let th = &lbl_803DA0D0.rank_thresholds;
    if x24 > th[entrants as usize][5] as i32 {
        rank = 6;
    } else {
        let mut j: i32 = 5;
        while j >= 0 {
            if x24 <= th[entrants as usize][j as usize] as i32 {
                rank = j;
            }
            j -= 1;
        }
    }

    (*tm).x33 = rank as u8;
}

/// Transitions to results screen after a tournament match: ranks players, preloads stage and
/// character data, and starts audio.
pub unsafe extern "C" fn fn_8019B458(arg0: *mut i32) {
    melee_core::ct!("fn_8019B458");
    // struct Preload { s32 stage; s32 char_ids[4]; s32 costumes[4]; } (zero-initialised)
    let mut req_stage: i32 = 0;
    let mut req_char_ids: [i32; 4] = [0; 4];
    let mut req_costumes: [i32; 4] = [0; 4];
    let tm = arg0 as *mut TmData;
    let d = d8();

    (*tm).x24 += 1;
    fn_8019B458_UpdateRank(tm, d);

    let match_ = fn_80196CF8();
    let td2 = gm_GetTournamentData();
    fn_80198D18();

    {
        let gobj = fn_8019035C(0, *(*lbl_804D6670).models.add(3), match_, 0x1A, 3, 1, Some(fn_80196EEC), 0.0);

        if (*td2).pad_x34[0] as i32 == match_ {
            ext::HSD_JObjSetFlagsAll(gobj_jobj(gobj), JOBJ_HIDDEN as u32);
        }
    }

    if match_ < 4 {
        (*d).x1B = 0x50;
    } else if match_ == 4 {
        (*d).x1B = 0x5F;
    } else {
        (*d).x1B = 0x61;
    }

    fn_80198BA0();
    fn_8018E618((*tm).entrants, 4.5, (*tm).x2C as i32);
    fn_8018E85C(*(*lbl_804D6670).models.add(4), (*tm).x2C as i32);
    fn_8018FA24();

    (*tm).cur_option = 0x14;
    (*tm).x2C = 0;

    let mut i: usize = 0;
    while i < 4 {
        if (*tm).x4B8[i].x0 != 3 {
            req_char_ids[i] = fn_8018F6FC((*tm).x4B8[i].x1 as i32);
            req_costumes[i] = (*tm).x4B8[i].x3 as i32;
        }
        i += 1;
    }

    let use_random_stage = if ((*tm).stage_selection_type == 0 && (*tm).x32 == 0) || (*tm).stage_selection_type == 1 { 1 } else { 0 };

    if use_random_stage != 0 {
        gm_GetTournamentData();
        loop {
            lbl_804D4190 = fn_8018F4A0();
            if lbl_804D4194 == lbl_804D4190 {
                if ext::fn_801642A0() != 0 {
                    break;
                }
            } else {
                break;
            }
        }
        lbl_804D4194 = lbl_804D4190;
        req_stage = lbl_804D4190;
    } else {
        req_stage = (*tm).x28 as i32;
    }

    {
        let scene = ext::lbDvd_GetPreloadCacheScene();
        let gc = addr_of_mut!((*scene).game_cache);
        ext::lbDvd_SetupVsPreloadCache();

        let mut i: usize = 0;
        while i < 4 {
            if (*tm).x4B8[i].x0 != 3 {
                (*gc).entries[i].char_id = req_char_ids[i] as _;
                (*gc).entries[i].color = req_costumes[i] as _;
            }
            i += 1;
        }

        let skip_stage_cache = if ((*tm).stage_selection_type == 2 && (*tm).x32 == 0) || (*tm).stage_selection_type == 3 { 1 } else { 0 };
        if skip_stage_cache == 0 {
            (*gc).stkind = req_stage as _;
        }

        ext::lbDvd_80018254();
    }

    let mut audio_mask: u64 = 0;
    let mut i: usize = 0;
    while i < 4 {
        if (*tm).x4B8[i].x0 != 3 {
            audio_mask |= ext::lbAudioAx_80026E84(req_char_ids[i] as _);
        }
        i += 1;
    }
    audio_mask |= ext::lbAudioAx_80026EBC(req_stage as _);
    ext::lbAudioAx_80026F2C(0x1C);
    ext::lbAudioAx_8002702C(0xC, audio_mask);
    ext::lbAudioAx_80027168();
}

/// Initializes match data and transitions to next state.
pub unsafe extern "C" fn fn_8019B81C(state: *mut i32) {
    melee_core::ct!("fn_8019B81C");
    fn_80198BA0();
    fn_80198EBC();
    *state = 0x1C;
    lbl_804799D8.x0 = 0;
}

pub unsafe extern "C" fn fn_8019B860(tm: *mut TmData) {
    melee_core::ct!("fn_8019B860");
    fn_8019A158();
    fn_80199AF0();
    fn_80198BA0();
    fn_8018F888();
    fn_8018E618((*tm).entrants, 4.5, (*tm).x2C as i32);
    fn_8018E85C(*(*lbl_804D6670).models.add(4), (*tm).x2C as i32);
    (*tm).cur_option = 0x20;
}

pub unsafe extern "C" fn gm_Scene_TouBracket_OnEnter(_arg0: *mut c_void) {
    melee_core::ct!("gm_Scene_TouBracket_OnEnter");
    lbl_804D6668 = null_mut();
    lbl_804D6664 = null_mut();
    lbl_804D6680[0] = 0;
    lbl_804D6680[2] = 0;
    gm_GetTournamentData();
    lbl_804D6660 = ext::lbArchive_80016DBC(
        b"GmTou1p\0".as_ptr() as *const c_char,
        addr_of_mut!(lbl_804D666C) as *mut c_void,
        &[b"ScGamTour_scene_data\0".as_ptr() as usize as u64, 0],
    );
    lbl_804D6638 = ext::lbArchive_80016DBC(
        b"TmBox.dat\0".as_ptr() as *const c_char,
        addr_of_mut!(lbl_804771B8.box2) as *mut c_void,
        &[
            b"tournament_box2_array\0".as_ptr() as usize as u64,
            addr_of_mut!(lbl_804771B8.box3) as usize as u64,
            b"tournament_box3_array\0".as_ptr() as usize as u64,
            addr_of_mut!(lbl_804771B8.box4) as usize as u64,
            b"tournament_box4_array\0".as_ptr() as usize as u64,
            0,
        ],
    );
    lbl_804D6664 = ext::lbArchive_80016DBC(
        b"GmTou3p\0".as_ptr() as *const c_char,
        addr_of_mut!(lbl_804D6670) as *mut c_void,
        &[b"ScGamTour_scene_data\0".as_ptr() as usize as u64, 0],
    );
    lbl_804D6668 = ext::lbArchive_80016DBC(
        b"GmTou4p\0".as_ptr() as *const c_char,
        addr_of_mut!(lbl_804D6674) as *mut c_void,
        &[b"ScGamTour_scene_data\0".as_ptr() as usize as u64, 0],
    );
    ext::HSD_SisLib_803A62A0(0, fn_8018F5F0(), b"SIS_TournamentData\0".as_ptr() as *mut c_char);
    if ext::HSD_Randi(2) != 0 {
        ext::lbAudioAx_80023F28(0x5D);
        return;
    }
    ext::lbAudioAx_80023F28(0x5E);
}

pub unsafe extern "C" fn gm_Scene_TouBracket_OnExit(_arg0: *mut c_void) {
    melee_core::ct!("gm_Scene_TouBracket_OnExit");
    ext::lbArchive_80016EFC(lbl_804D6660);
    ext::lbArchive_80016EFC(lbl_804D6638);
    ext::lbArchive_80016EFC(lbl_804D6664);
    ext::lbArchive_80016EFC(lbl_804D6668);
}

pub unsafe extern "C" fn fn_8019BA04(_state: *mut i32) { melee_core::ct!("fn_8019BA04");}

// ---- folded from gm_ptr.rs (consolidation steps 21/22) ----
/// gmtou_1.c `fn_80196564_fp`.
pub fn fn_80196564_fp(arg0: &TmData_gm_ptr) -> bool {
    melee_core::ct!("fn_80196564_fp");
    (arg0.stage_selection_type == 0 && arg0.x32 == 0) || arg0.stage_selection_type == 1
}

/// gmtou_1.c `fn_80196594_fp`.
pub fn fn_80196594_fp(arg0: &TmData_gm_ptr) -> bool {
    melee_core::ct!("fn_80196594_fp");
    (arg0.stage_selection_type == 2 && arg0.x32 == 0) || arg0.stage_selection_type == 3
}
