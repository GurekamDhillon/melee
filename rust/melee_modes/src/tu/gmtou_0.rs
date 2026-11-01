//! src/melee/gm/gmtou_0.c: tournament setup scene (menu text/model updaters, per-option input
//! handlers `fn_801937C4`..`fn_8019610C`, scene OnFrame/OnEnter/OnExit).
//!
//! Fused float sites (tools/rust-port/fma_sites.py): `fn_80190ABC` (`149*x+537`, `47*x+409`,
//! `48.6*x+183`), `fn_80193308` (`35.7*x+113`), `fn_80191D38`/`fn_80191E9C` (`2.62*x-12.8`),
//! `fn_8019237C` (`7.19*x-1.8`, `2.3*x-2.7`), `fn_80192BB0` (`3.5*x-11.5` x3), `fn_80192E6C`
//! (`6*x-3`, `4.5*x-2.3`) are `fmadds` below.
//!
//! Byte-level accesses of the C stay byte-level where the GC and the native layout agree (the
//! `Lbl804799B8_t` state, TmData below 0x4E0, the `s32*` view of TmData's first ints). The
//! setting table `lbl_803D9F80` is read big-endian (`(u16*) &lbl_803D9F80`: game memory is
//! big-endian in the shipping build).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, clippy::all)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types as mt;
#[allow(unused_imports)]
use mt::HSD_JObj as HSD_JObj_gm_ptr;
#[allow(unused_imports)]
use mt::TmData as TmData_gm_ptr;
use core::ffi::{c_char, c_void};
use core::ptr::{addr_of_mut, null_mut};

use melee_core::fp::fmadds;
use melee_runtime::cabi::{sdiv, srem};
use melee_types::*;

use super::gmtoulib::*;

/// Callees owned by other (unported) TUs; the shared ones come from gmtoulib's `ext`.
pub mod ext {
    pub use super::super::gmtoulib::ext::*;
    use super::*;

    crate::gmtou_ext! {
        fn HSD_GObj_FogCallback(gobj: *mut HSD_GObj, unused: i32);
        fn gm_IsCKindUnlocked(ckind: u8) -> i32;
        fn gm_SetNextGameModeStateId(pending_scene: u8);
        fn gm_ChangeGameModeAfterCurrentScene(pending_mode: i32);
        fn gm_801A4B60();
        fn lbArchive_LoadArchive(filename: *const c_char) -> *mut HSD_Archive;
        fn lbArchive_80016EFC(arg0: *mut HSD_Archive);
        fn gmMainLib_8015ECB0() -> i32;
        fn lbAudioAx_800237A8(sfx_id: u32, sfx_vol: i32, sfx_pan: i32) -> i32;
        fn lbAudioAx_80023F28(arg0: i32) -> i32;
        fn lbAudioAx_80024030(arg0: i32);
        fn lbAudioAx_80026F2C(arg0: u32);
        fn lbAudioAx_8002702C(flags: u32, mask: u64);
        fn lbAudioAx_80027168();
        fn lbAudioAx_80027648();
        fn mn_8022F218() -> i32;
        fn mn_8022F268();
        fn HSD_JObjSetFlags(arg0: *mut HSD_JObj, flags: u32);
        fn HSD_JObjClearFlags(arg0: *mut HSD_JObj, flags: u32);
        fn HSD_JObjClearFlagsAll(arg0: *mut HSD_JObj, flags: u32);
        fn HSD_SisLib_803A7664(arg0: *mut HSD_Text);
        fn HSD_SisLib_803A6754(arg0: i32, arg1: i32) -> *mut HSD_Text;
        fn HSD_SisLib_803A6368(arg0: *mut HSD_Text, arg1: i32);
        fn HSD_SisLib_803A62A0(arg0: i32, arg1: *mut c_char, arg2: *mut c_char);
        fn HSD_SisLib_803A5ACC(a0: i32, a1: i32, a2: f32, a3: f32, a4: f32, a5: f32, a6: f32) -> *mut HSD_Text;
        fn mn_802317E4(arg0: *mut HSD_Archive, arg1: i32);
        fn GetNameCount() -> i32;
        fn IsNameListFull() -> i32;
        fn mnNameNew_8023EA08(arg0: *mut c_void);
        fn HSD_MObjReqAnim(mobj: *mut HSD_MObj, startframe: f32);
        fn HSD_MObjAnim(mobj: *mut HSD_MObj);
        fn HSD_FogLoadDesc(desc: *mut HSD_FogDesc) -> *mut HSD_Fog;
    }
    /// `lbArchive_80016DBC(filename, symbols, ...)`: `va` are the variadic words.
    pub unsafe extern "C" fn lbArchive_80016DBC(filename: *const c_char, symbols: *mut c_void, va: &[u64]) -> *mut HSD_Archive {
        use super::super::gmtoulib::hook::{self, Word};
        let mut a = vec![filename.to_w(), symbols.to_w()];
        a.extend_from_slice(va);
        <*mut HSD_Archive as Word>::from_w(hook::call("lbArchive_80016DBC", &a))
    }
}

// ---------------------------------------------------------------------------------------------
// data
// ---------------------------------------------------------------------------------------------

/// `lbl_804799B8`: 0x20 bytes of menu state (cursor rows, flash timers, anim counters).
pub static mut lbl_804799B8: Lbl804799B8_t = unsafe { core::mem::zeroed() };
pub static mut lbl_804D6640: *mut HSD_Archive = null_mut();
pub static mut lbl_804D6644: *mut HSD_Archive = null_mut();
pub static mut lbl_804D6648: *mut HSD_Archive = null_mut();
pub static mut lbl_804D664C: *mut SceneDesc = null_mut();
pub static mut lbl_804D6650: *mut SceneDesc = null_mut();
pub static mut lbl_804D6654: u32 = 0;
pub static mut lbl_804D6658: i32 = 0;
pub static mut lbl_804D665C: i32 = 0;

pub const lbl_804DA78C: GXColor = GXColor { r: 0x46, g: 0xDC, b: 0x46, a: 0xFF };

/// `lbl_803D9F80` (`TmSettingTable`, 0x58 bytes): 32 big-endian u16 text ids, then `min[6][2]`
/// at 0x40 and `max[6][2]` at 0x4C.
pub static lbl_803D9F80: [u8; 0x58] = [
    0, 74, 0, 74, 0, 77, 0, 75, 0, 75, 0, 77, 0, 80, 0, 78, 0, 79, 0, 79, 0, 81, 0, 0, 0, 82, 0, 82, 0, 92, 0, 92, 0, 96, 0,
    93, 0, 93, 0, 96, 0, 95, 0, 97, 0, 98, 0, 98, 0, 100, 0, 0, 0, 83, 0, 111, 0, 111, 0, 111, 0, 88, 0, 101, 0, 0, 2, 3, 0,
    2, 0, 1, 0, 0, 0, 0, 2, 2, 4, 16, 31, 3, 63, 3, 3, 3, 9, 0,
];
const MIN_OFF: usize = 0x40;
const MAX_OFF: usize = 0x4C;

/// `((u16*) &lbl_803D9F80)[i]` with big-endian memory.
#[inline]
fn tbl_u16(i: usize) -> i32 {
    u16::from_be_bytes([lbl_803D9F80[i * 2], lbl_803D9F80[i * 2 + 1]]) as i32
}
/// `table->min[idx][!!mt]`.
#[inline]
fn tbl_min(idx: i32, m: i32) -> i32 {
    lbl_803D9F80[MIN_OFF + idx as usize * 2 + (m != 0) as usize] as i32
}
/// `table->max[idx][!!mt]`.
#[inline]
fn tbl_max(idx: i32, m: i32) -> i32 {
    lbl_803D9F80[MAX_OFF + idx as usize * 2 + (m != 0) as usize] as i32
}

type TmFn = unsafe extern "C" fn(*mut i32, u32, u32);

/// `lbl_803D9FD8`: per-`cur_option` input handlers.
pub static lbl_803D9FD8: [Option<TmFn>; 19] = [
    Some(fn_801937C4),
    Some(fn_80193B58),
    Some(fn_80193FCC),
    Some(fn_80194658),
    Some(fn_801949B4),
    Some(fn_80194BC4),
    Some(fn_80194D84),
    Some(fn_80194D84),
    Some(fn_80194D84),
    None,
    Some(fn_80194F30),
    Some(fn_801953C8),
    Some(fn_80194F30),
    Some(fn_80195AF0),
    Some(fn_80195AF0),
    Some(fn_80195CCC),
    Some(fn_80194F30),
    Some(fn_8019610C),
    Some(fn_8019610C),
];

// ---------------------------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------------------------

#[inline]
unsafe fn st() -> *mut Lbl804799B8_t {
    addr_of_mut!(lbl_804799B8)
}
/// `((u8*) &lbl_804799B8) + off`.
#[inline]
unsafe fn st_byte(off: usize) -> *mut u8 {
    (addr_of_mut!(lbl_804799B8) as *mut u8).add(off)
}
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
unsafe fn child_of(j: *mut HSD_JObj) -> *mut HSD_JObj {
    if j.is_null() { null_mut() } else { (*j).child }
}
#[inline]
unsafe fn next_of(j: *mut HSD_JObj) -> *mut HSD_JObj {
    if j.is_null() { null_mut() } else { (*j).next }
}
#[inline]
unsafe fn gobj_jobj(g: *mut HSD_GObj) -> *mut HSD_JObj {
    (*g).hsd_obj as *mut HSD_JObj
}
/// `(&tm->match_type)[i]`: the int run starting at match_type.
#[inline]
unsafe fn mt_ptr(tm: *mut TmData, i: i32) -> *mut i32 {
    (addr_of_mut!((*tm).match_type)).offset(i as isize)
}
#[inline]
unsafe fn x37(tm: *mut TmData, i: i32) -> *mut TmUnkMenuData {
    (addr_of_mut!((*tm).x37) as *mut TmUnkMenuData).offset(i as isize)
}
/// Pointer to the `i`-th `HSD_Text*` of the contiguous x4E8/x500/x518/x524 runs (indices past
/// an array's end alias the next array, as in the C).
#[inline]
unsafe fn t4e8(tm: *mut TmData, i: i32) -> *mut HSD_Text {
    *(addr_of_mut!((*tm).x4E8) as *mut *mut HSD_Text).offset(i as isize)
}
#[inline]
unsafe fn t500(tm: *mut TmData, i: i32) -> *mut HSD_Text {
    *(addr_of_mut!((*tm).x500) as *mut *mut HSD_Text).offset(i as isize)
}

// ---------------------------------------------------------------------------------------------
// functions
// ---------------------------------------------------------------------------------------------

pub unsafe extern "C" fn fn_80190ABC(mode: i32) {
    melee_core::ct!("fn_80190ABC");
    let mut mode = mode;
    let state = st();
    let tm = gm_GetTournamentData();
    let cur_opt = (*tm).cur_option;
    let opt = core::cmp::min(cur_opt, 6);
    let mt = (gm_804771C4.match_type != 0) as i32;

    if opt == 6 && mode != 0 && cur_opt < 9 {
        return;
    }

    match mode {
        0 => {
            ext::HSD_SisLib_803A6368((*tm).x4E0, tbl_u16((opt * 2 + mt) as usize));
        }
        2 => {
            ext::HSD_SisLib_803A6368(t4e8(tm, opt), tbl_u16((opt * 2 + mt + 0xE) as usize));
        }
        3 => {
            let display_val: i32;
            if opt == 2 && gm_804771C4.match_type == 0 {
                let val = *mt_ptr(tm, opt);
                match val {
                    16 | 17 => display_val = val + 0xE1,
                    21 | 22 => display_val = val + 0xDE,
                    30 | 31 => display_val = val + 0xD7,
                    _ => {
                        display_val = lbl_803D9D20_x0(val) as i32 + tbl_u16((opt + 0x1A) as usize);
                    }
                }
            } else {
                let mut dv = tbl_u16((opt + 0x1A) as usize);
                dv += *mt_ptr(tm, opt);
                display_val = dv;
            }
            ext::HSD_SisLib_803A6368(t500(tm, opt), display_val);
        }
        1 => {
            let display_val = lbl_803D9D20_x0((*tm).entrants) as i32 + 0xB0 - (*tm).hmn_cpu_count;
            ext::HSD_SisLib_803A6368((*tm).x4E4, display_val);
        }
        4 => {
            ext::HSD_SisLib_803A7664((*tm).x518[0]);
            mode = 0;
            while mode < 0x24 {
                fn_8018ECA8(
                    mode + (*state).x6 as i32 * 4,
                    0xFF,
                    0,
                    fmadds(149.0, (mode % 4) as f32, 537.0),
                    fmadds(47.0, (mode / 4) as f32, 409.0),
                    0,
                );
                mode += 1;
            }
        }
        6 => {
            ext::HSD_SisLib_803A7664((*tm).x518[1]);
            ext::HSD_SisLib_803A7664((*tm).x518[2]);
            let start = addr_of_mut!((*state).x3);
            let mut i = 0;
            while i < 12 {
                let idx = *start as i32 + i;
                if idx > (*state).x4 as i32 {
                    i += 1;
                    continue;
                }
                let pos_y = fmadds(48.5999985, i as f32, 183.0);
                fn_8018ECA8((*x37(tm, idx)).x9 as i32, (*x37(tm, idx)).x0 as i32, 1, 143.0, pos_y, 2);
                let idx = *start as i32 + i;
                fn_8018ECA8((*x37(tm, idx)).x9 as i32, (*x37(tm, idx)).x0 as i32, 1, 143.0, pos_y, 2);
                i += 1;
            }
        }
        5 => {
            ext::HSD_SisLib_803A7664((*tm).x524[0]);
            ext::HSD_SisLib_803A7664((*tm).x524[1]);
            let table = addr_of_mut!((*state).x2);
            let start = addr_of_mut!((*state).x3);
            let idx = *table as i32 + *start as i32;
            fn_8018ECA8((*x37(tm, idx)).x9 as i32, (*x37(tm, idx)).x0 as i32, 3, 514.0, 87.0, 4);
            let idx = *table as i32 + *start as i32;
            fn_8018ECA8((*x37(tm, idx)).x9 as i32, (*x37(tm, idx)).x0 as i32, 3, 514.0, 87.0, 4);
        }
        _ => {}
    }
}

/// `lbl_803D9D20.x0[i]` (the byte at offset `i` of the table; callers index small values).
#[inline]
unsafe fn lbl_803D9D20_x0(i: i32) -> u8 {
    *(addr_of_mut!(lbl_803D9D20) as *const u8).offset(i as isize)
}

pub unsafe extern "C" fn gm_80190EA4() {
    melee_core::ct!("gm_80190EA4");
    let tmdata = gm_GetTournamentData();

    fn_80190174((*(*lbl_804D664C).cameras).desc);
    fn_801902F0(0);
    fn_80193308();
    fn_8019027C((*lbl_804D664C).lights as *mut c_void);
    fn_80193230();
    (*tmdata).cur_option = 0;

    while (*tmdata).cur_option < 7 {
        fn_80190ABC(0);
        fn_80190ABC(2);
        fn_80190ABC(3);
        fn_80190ABC(1);
        (*tmdata).cur_option += 1;
    }
    if gm_804771C4.match_type == 0 {
        let tmdata2 = gm_GetTournamentData();
        fn_8018EC7C();
        fn_8018E618((*tmdata2).entrants, 4.5, 1);
        fn_80190480(130.0);
        fn_80190520(-278.0, 255.0, 0.0);
    }
    (*tmdata).x20 = 0;

    let mut i = 0;
    while i < 64 {
        if (*ext::gmMainLib_GetGameRules()).handicap == 1 {
            (*x37(tmdata, i)).x2 = 5;
        } else {
            (*x37(tmdata, i)).x2 = 9;
        }
        i += 1;
    }
}

/// Initializes the time menu state when entering the time selection screen.
pub unsafe extern "C" fn gm_80190FE4(arg0: i32) {
    melee_core::ct!("gm_80190FE4");
    let tm = gm_GetTournamentData();

    if ext::mn_8022F218() == 0 {
        fn_80190174((*(*lbl_804D664C).cameras).desc);
        fn_801902F0(0);
        fn_80193308();
        fn_8019027C((*lbl_804D664C).lights as *mut c_void);
        fn_80193230();

        (*tm).cur_option = 0;
        while (*tm).cur_option < 7 {
            fn_80190ABC(0);
            fn_80190ABC(2);
            fn_80190ABC(3);
            fn_80190ABC(1);
            (*tm).cur_option += 1;
        }

        (*tm).x20 = 0;
        (*tm).cur_option = 0xC;

        if arg0 != 0x78 {
            (*x37(tm, lbl_804799B8.x2 as i32 + lbl_804799B8.x3 as i32)).x9 = arg0 as u16;
        }

        fn_80190ABC(5);
        fn_80190ABC(6);
    }
}

pub unsafe extern "C" fn fn_801910E0(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_801910E0");
    let counter = addr_of_mut!(lbl_804799B8.xC_counter);
    let jobj = gobj_jobj(gobj);

    if *counter as u32 > 0x320 {
        *counter = 0;
    }
    fn_8019044C(jobj, *counter as f32);
    *counter = (*counter).wrapping_add(1);
}

pub unsafe extern "C" fn fn_80191154(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_80191154");
    let state = st();
    let tm = gm_GetTournamentData();
    let jobj = gobj_jobj(gobj);

    let frame = addr_of_mut!((*state).xE);
    if (*tm).cur_option < 9 {
        if *frame as u32 > 0x27 {
            *frame = 0xA;
        }
    } else {
        if *frame == 0x31 {
            fn_80190ABC(6);
            fn_80190ABC(5);
            (*tm).cur_option = 0xA;
        }
        if *frame as u32 > 0x4E {
            *frame = 0x32;
        }
    }
    let counter = addr_of_mut!((*state).xE);
    fn_8019044C(jobj, *counter as f32);
    *counter = (*counter).wrapping_add(1);
}

#[inline]
unsafe fn fn_80191240_dec_flash_timer() -> u8 {
    lbl_804799B8.x7.wrapping_sub(1)
}

/// Updates the visibility and flash animation for a Tournament Mode option icon.
pub unsafe extern "C" fn fn_80191240(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_80191240");
    let tm = gm_GetTournamentData();
    let val = fn_8018F62C(&*gobj) as i32;
    let mut jobj = gobj_jobj(gobj);
    let jobj_copy = jobj;
    if (*tm).cur_option >= 9 {
        ext::HSD_JObjSetFlagsAll(jobj_copy, JOBJ_HIDDEN as u32);
        return;
    }
    // fn_80191240_show_children
    ext::HSD_JObjClearFlagsAll(jobj_copy, JOBJ_HIDDEN as u32);
    let first_child = if jobj.is_null() { null_mut() } else { (*jobj).child };
    jobj = if first_child.is_null() { null_mut() } else { (*first_child).next };

    if val == (*tm).cur_option {
        if lbl_804799B8.x7 != 0 {
            fn_8019044C(first_child, 1.0);
            lbl_804799B8.x7 = fn_80191240_dec_flash_timer();
        } else {
            fn_8019044C(first_child, 0.0);
        }
        if lbl_804799B8.x8 != 0 {
            fn_8019044C(jobj, 1.0);
            lbl_804799B8.x8 = lbl_804799B8.x8.wrapping_sub(1);
            return;
        }
        fn_8019044C(jobj, 0.0);
        return;
    }
    if val > (*tm).cur_option {
        ext::HSD_JObjSetFlagsAll(jobj_copy, JOBJ_HIDDEN as u32);
        return;
    }
    if val == 5 && gm_804771C4.match_type != 0 {
        ext::HSD_JObjSetFlagsAll(jobj_copy, JOBJ_HIDDEN as u32);
    }
    fn_8019044C(jobj_copy, 0.0);
}

/// Updates text visibility and menu state for tournament bracket options.
pub unsafe extern "C" fn fn_801913BC(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_801913BC");
    let tm = gm_GetTournamentData();
    let idx = fn_8018F62C(&*gobj);
    let jobj = gobj_jobj(gobj);
    let ii = idx as i32;

    if (*tm).cur_option < 9 {
        ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);

        if ii == 0 {
            (*(*tm).x518[0]).hidden = 1;
            (*(*tm).x518[1]).hidden = 1;
            (*(*tm).x518[2]).hidden = 1;
            (*(*tm).x524[0]).hidden = 1;
            (*(*tm).x524[1]).hidden = 1;
            (*(*tm).x4E0).hidden = 0;
        }

        if ii <= (*tm).cur_option {
            (*t4e8(tm, ii)).hidden = 0;
            (*t500(tm, ii)).hidden = 0;
            if ii < 3 && gm_804771C4.match_type == 0 {
                (*(*tm).x4E4).hidden = 1;
            } else {
                (*(*tm).x4E4).hidden = 0;
            }
            if ii == 5 && gm_804771C4.match_type != 0 {
                (*t4e8(tm, ii)).hidden = 1;
                (*t500(tm, ii)).hidden = 1;
                ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
            }
        } else {
            (*t4e8(tm, ii)).hidden = 1;
            (*t500(tm, ii)).hidden = 1;
        }

        if (*tm).cur_option >= 3 && gm_804771C4.match_type == 0 {
            (*(*tm).x4E4).hidden = 0;
        } else {
            (*(*tm).x4E4).hidden = 1;
        }

        let cur = (*tm).cur_option;
        if ii == cur {
            // counter_ptr = &lbl_804799B8.x0 + idx; counter_ptr += 0x10
            let counter_ptr = st_byte(ii as usize + 0x10);
            if *counter_ptr as u32 >= 0x14 {
                *counter_ptr = 0xA;
            }
            fn_8019044C(jobj, *counter_ptr as f32);
            *counter_ptr = (*counter_ptr).wrapping_add(1);
            return;
        }
        if ii > cur {
            ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
            return;
        }
        fn_8019044C(jobj, 0.0);
        return;
    }

    ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
    (*t4e8(tm, ii)).hidden = 1;
    (*t500(tm, ii)).hidden = 1;
    if ii == 0 {
        if (*tm).cur_option > 9 {
            (*(*tm).x518[1]).hidden = 0;
            (*(*tm).x518[2]).hidden = 0;
            (*(*tm).x524[0]).hidden = 0;
            (*(*tm).x524[1]).hidden = 0;
            if (*tm).cur_option != 0xF {
                (*(*tm).x518[0]).hidden = 1;
            } else {
                (*(*tm).x518[0]).hidden = 0;
            }
        }
        (*(*tm).x4E4).hidden = 1;
        (*(*tm).x4E0).hidden = 1;
    }
}

/// Updates the visibility and animation state of a Tournament Mode menu option.
pub unsafe extern "C" fn fn_80191678(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_80191678");
    let tmdata = gm_GetTournamentData();
    let idx = fn_8018F62C(&*gobj) as i32;
    let jobj = gobj_jobj(gobj);

    if (*tmdata).cur_option >= 9 {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        return;
    }

    ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);

    if idx == (*tmdata).cur_option {
        let ptr = st_byte(idx as usize + 0x16);
        if *ptr > 0x64 {
            *ptr = 0;
        }
        fn_8019044C(jobj, *ptr as f32);
        *ptr = (*ptr).wrapping_add(1);
        return;
    }

    ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
}

/// Updates menu option selection animation.
pub unsafe extern "C" fn fn_8019175C(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_8019175C");
    let tm = gm_GetTournamentData();
    fn_8018F62C(&*gobj);
    let root_jobj = gobj_jobj(gobj);
    let jobj = root_jobj;
    let mut jobjs: [*mut HSD_JObj; 3] = [null_mut(); 3];

    if (*tm).cur_option >= 9 {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        return;
    }

    ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);

    let first_child = child_of(root_jobj);
    jobjs[2] = first_child;

    let mut child = next_of(jobjs[2]);
    child = next_of(child);
    let option_a = next_of(child);
    jobjs[0] = option_a;
    let option_b = next_of(jobjs[0]);
    jobjs[1] = option_b;

    if (*tm).cur_option >= 6 {
        let counter = addr_of_mut!(lbl_804799B8.xF);
        if *counter >= 0x14 {
            *counter = 0;
        }
        let mut i = 0;
        while i <= 2 {
            if i != (*tm).cur_option - 6 {
                fn_8019044C(jobjs[i as usize], 0.0);
            } else {
                fn_8019044C(jobjs[i as usize], *counter as f32);
            }
            i += 1;
        }
        *counter = (*counter).wrapping_add(1);
        return;
    }

    ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
}

/// Updates visibility and animation frame of a Training Mode HUD element.
pub unsafe extern "C" fn fn_801918F0(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_801918F0");
    let tm = gm_GetTournamentData();
    let jobj = gobj_jobj(gobj);

    if (*tm).cur_option <= 9 {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        return;
    }

    ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);

    match (*tm).cur_option {
        0xB => {
            let counter = addr_of_mut!(lbl_804799B8.pad2[0xC]);
            if *counter > 5 {
                *counter = 0;
            }
            fn_8019044C(jobj, *counter as f32);
            if *counter < 5 {
                *counter += 1;
            }
        }
        0xD | 0xE => {
            let counter = addr_of_mut!(lbl_804799B8.pad2[0xC]);
            if *counter < 10 {
                *counter = 10;
            }
            fn_8019044C(jobj, *counter as f32);
            if *counter < 15 {
                *counter += 1;
            }
        }
        0xF => fn_8019044C(jobj, 16.0),
        _ => fn_8019044C(jobj, 0.0),
    }
}

/// Updates the tournament bracket display visibility and animation.
pub unsafe extern "C" fn fn_80191A54(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_80191A54");
    let td = gm_GetTournamentData();
    let jobj = gobj_jobj(gobj);
    let jobj_copy = jobj;

    if (*td).cur_option <= 9 {
        ext::HSD_JObjSetFlagsAll(jobj_copy, JOBJ_HIDDEN as u32);
        return;
    }

    ext::HSD_JObjClearFlagsAll(jobj_copy, JOBJ_HIDDEN as u32);

    let child = child_of(jobj);

    let cur_option = (*td).cur_option;
    if cur_option == 0xA || cur_option == 0xC || cur_option == 0x10 {
        fn_8019044C(child, lbl_804799B8.x2 as f32);
        ext::HSD_MObjReqAnim((*(*child).u.dobj).mobj, lbl_804799B8.xA as f32);
    } else {
        ext::HSD_MObjReqAnim((*(*child).u.dobj).mobj, 6.0);
    }
    ext::HSD_MObjAnim((*(*child).u.dobj).mobj);
}

/// Updates button highlight animation based on current menu option.
pub unsafe extern "C" fn fn_80191B5C(gobj: *mut c_void) {
    melee_core::ct!("fn_80191B5C");
    let timers = st();
    let tm = gm_GetTournamentData();
    let jobj = (*(gobj as *mut HSD_GObj)).hsd_obj as *mut HSD_JObj;

    if (*tm).cur_option <= 9 {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
    } else {
        ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
        match (*tm).cur_option {
            10 => fn_8019044C(jobj, (*timers).xA as f32),
            11 => fn_8019044C(jobj, 5.0),
            12 => fn_8019044C(jobj, ((*timers).xA as i32 + 0x14) as f32),
            13 | 14 | 15 => fn_8019044C(jobj, 25.0),
            16 => fn_8019044C(jobj, ((*timers).xA as i32 + 0x28) as f32),
            _ => {}
        }
    }
}

/// Updates JObj visibility based on current menu option selection.
pub unsafe extern "C" fn fn_80191CA4(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_80191CA4");
    let tm = gm_GetTournamentData();
    let jobj = gobj_jobj(gobj);
    if (*tm).cur_option <= 9 {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        return;
    }
    if lbl_804D6658 < 10 {
        lbl_804D6658 += 1;
    }
    ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
    fn_8019044C(jobj, lbl_804D6658 as f32);
}

/// GObj callback for tournament bracket slot UI elements.
pub unsafe extern "C" fn fn_80191D38(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_80191D38");
    let tm = gm_GetTournamentData();
    let idx = fn_8018F62C(&*gobj) as i32;
    let jobj = gobj_jobj(gobj);

    if (*tm).cur_option <= 9 {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        return;
    }

    ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);

    if idx < lbl_804799B8.x3 as i32 || idx >= (*tm).x2E as i32 || idx > lbl_804799B8.x3 as i32 + 0xB {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        return;
    }

    fn_8018FDC4(
        jobj,
        666.0,
        -fmadds(2.6200008, (idx - lbl_804799B8.x3 as i32) as f32, -12.800008),
        0.1,
    );

    let m = x37(tm, idx);
    if (*m).x5 != 0 {
        fn_8019044C(jobj, 201.0);
    } else {
        fn_8019044C(jobj, fn_8018F71C((*m).x3 as i32, (*m).x7 as i32));
    }

    if (*addr_of_mut!(lbl_803D9D20)).x72[(*m).x3 as usize] != 0 {
        ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
    } else {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
    }
}

/// Updates visibility and position of a tournament bracket player entry JObj.
pub unsafe extern "C" fn fn_80191E9C(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_80191E9C");
    let tm = gm_GetTournamentData();
    let idx = fn_8018F62C(&*gobj) as i32;
    let jobj = gobj_jobj(gobj);

    if (*tm).cur_option <= 9 {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        return;
    }

    ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);

    if lbl_804799B8.x0 == 0 {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        return;
    }

    if idx < lbl_804799B8.x3 as i32 || idx >= (*tm).x2E as i32 || idx > lbl_804799B8.x3 as i32 + 0xB {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        return;
    }

    fn_8018FDC4(
        jobj,
        666.0,
        -fmadds(2.6200008, (idx - lbl_804799B8.x3 as i32) as f32, -12.800008),
        0.1,
    );
    fn_8019044C(jobj, (*x37(tm, idx)).x2 as f32);
}

#[inline]
unsafe fn fn_80191FD4_is_selected(hud: i32, slot: i32, tm: *mut TmData) -> bool {
    (*x37(tm, slot)).x3 as i32 == hud
}

pub unsafe extern "C" fn fn_80191FD4(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_80191FD4");
    let state = st();
    let tm = gm_GetTournamentData();
    let idx = fn_8018F62C(&*gobj);
    let jobj = gobj_jobj(gobj);

    if (*tm).cur_option <= 9 {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        return;
    }

    ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);

    if idx as i32 == 0x19 {
        let child = child_of(jobj);
        let frame: f32 = 0.0;
        fn_8019044C(child, frame);

        let sibling = next_of(child);
        let x2_ptr = addr_of_mut!((*state).x2);
        let x3_ptr = addr_of_mut!((*state).x3);
        let slot = *x2_ptr as i32 + *x3_ptr as i32;

        if (*x37(tm, slot)).x5 != 0 {
            fn_8019044C(sibling, 201.0);
            return;
        }

        fn_8019044C(sibling, fn_8018F71C((*x37(tm, slot)).x3 as i32, (*x37(tm, slot)).x7 as i32));
        if (*addr_of_mut!(lbl_803D9D20)).x72[(*x37(tm, *x2_ptr as i32 + *x3_ptr as i32)).x3 as usize] != 0 {
            ext::HSD_JObjClearFlagsAll(sibling, JOBJ_HIDDEN as u32);
            return;
        }
        if fn_8018F3D0(fn_8018F310(fn_8018F6FC((*x37(tm, *x2_ptr as i32 + *x3_ptr as i32)).x3 as i32))) == 1 {
            fn_8019044C(sibling, 200.0);
            return;
        }
        ext::HSD_JObjSetFlagsAll(sibling, JOBJ_HIDDEN as u32);
        return;
    }

    if (*tm).cur_option != 0xB {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        return;
    }

    fn_8018FF9C(jobj, 0.9, 0.9, 666.0);
    let child2 = child_of(jobj);

    let mut hud = fn_8018F6DC(fn_8018F3BC(idx as i32));
    let x2b = addr_of_mut!((*state).x2);
    let x3b = addr_of_mut!((*state).x3);
    let slot = *x2b as i32 + *x3b as i32;
    if fn_80191FD4_is_selected(hud, slot, tm) {
        fn_8019044C(child2, ((*state).xA as i32 + 0xA) as f32);
    } else {
        hud = fn_8018F6DC(fn_8018F3BC(idx as i32));
        let flag: i32;
        if (*addr_of_mut!(lbl_803D9D20)).x72[hud as usize] != 0 || fn_8018F3D0(fn_8018F310(fn_8018F6FC(hud))) != 2 {
            flag = 1;
        } else {
            flag = 0;
        }
        if flag == 0 {
            ext::HSD_JObjSetFlagsAll(child2, JOBJ_HIDDEN as u32);
        } else {
            fn_8019044C(child2, 0.0);
        }
    }

    let sibling2 = next_of(child2);
    ext::HSD_JObjClearFlagsAll(sibling2, JOBJ_HIDDEN as u32);

    hud = fn_8018F6DC(fn_8018F3BC(idx as i32));
    if (*addr_of_mut!(lbl_803D9D20)).x72[hud as usize] != 0 {
        let m = x37(tm, *x2b as i32 + *x3b as i32);
        if (*m).x3 as i32 == fn_8018F6DC(fn_8018F3BC(idx as i32)) {
            fn_8019044C(
                sibling2,
                (((*m).x7 as i32 * 0x1E) + fn_8018F6DC(fn_8018F3BC(idx as i32))) as f32,
            );
            return;
        }
        fn_8019044C(sibling2, fn_8018F6DC(fn_8018F3BC(idx as i32)) as f32);
        return;
    }

    if fn_8018F3D0(fn_8018F310(fn_8018F3BC(idx as i32))) == 1 {
        fn_8019044C(sibling2, 200.0);
        return;
    }
    ext::HSD_JObjSetFlagsAll(sibling2, JOBJ_HIDDEN as u32);
}

/// Updates tournament menu cursor JObj visibility and position.
pub unsafe extern "C" fn fn_8019237C(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_8019237C");
    let tm = gm_GetTournamentData();
    let jobj = gobj_jobj(gobj);

    if (*tm).cur_option <= 9 {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        return;
    }

    ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);

    if (*tm).cur_option != 0xF {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        return;
    }

    fn_8019044C(jobj, lbl_804799B8.xA as f32);
    fn_8018FDC4(
        jobj,
        fmadds(7.18999958, (lbl_804799B8.x5 % 4) as f32, -1.8),
        -fmadds(2.3, (lbl_804799B8.x5 as i32 / 4) as f32, -2.7),
        666.0,
    );
}

pub unsafe extern "C" fn fn_8019249C(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_8019249C");
    let tmdata = gm_GetTournamentData();
    let mut cur_option = (*tmdata).cur_option;
    let jobj = gobj_jobj(gobj);

    if cur_option <= 9 {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        return;
    }

    ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
    cur_option = (*tmdata).cur_option;
    if cur_option != 0xD && cur_option != 0xE {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        return;
    }

    let mut child = child_of(jobj);
    child = next_of(child);
    child = next_of(child);
    child = next_of(child);
    child = next_of(child);
    child = next_of(child);
    child = next_of(child);
    let mut child2 = next_of(child);
    child2 = next_of(child2);
    child2 = next_of(child2);
    let child3 = if child2.is_null() { null_mut() } else { (*child2).next };
    child2 = child3;

    if cur_option == 0xD {
        fn_8019044C(child, 0.0);
        fn_8019044C(child2, (lbl_804799B8.xA as i32 + 10) as f32);
    } else {
        fn_8019044C(child, (lbl_804799B8.xA as i32 + 10) as f32);
        fn_8019044C(child2, 0.0);
    }
}

pub unsafe extern "C" fn fn_80192690(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_80192690");
    let tmdata = gm_GetTournamentData();
    let jobj = gobj_jobj(gobj);

    if (*tmdata).cur_option <= 9 {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        return;
    }
    ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
    if (*tmdata).cur_option < 0x11 || (*tmdata).cur_option > 0x12 {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        return;
    }
    fn_8018FDC4(jobj, 666.0, 666.0, 0.3);
    fn_8019044C(jobj, ((*tmdata).cur_option - 0x11) as f32);
}

pub unsafe extern "C" fn fn_80192758(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_80192758");
    let data = st();
    let tmdata = gm_GetTournamentData();
    let jobj = gobj_jobj(gobj);

    if (*tmdata).cur_option <= 9 {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        return;
    }

    ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);

    if (*data).x0 == 0 {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        return;
    }

    fn_8019044C(jobj, (*x37(tmdata, lbl_804799B8.x2 as i32 + lbl_804799B8.x3 as i32)).x2 as f32);

    if (*data).x1 != 0 {
        let mut child = child_of(jobj);
        ext::HSD_JObjClearFlags(child, JOBJ_HIDDEN as u32);

        child = child_of(jobj);
        child = if child.is_null() { null_mut() } else { (*child).next };
        ext::HSD_JObjClearFlags(child, JOBJ_HIDDEN as u32);

        let val = (*x37(tmdata, lbl_804799B8.x2 as i32 + lbl_804799B8.x3 as i32)).x2;
        if val == 9 {
            child = child_of(jobj);
            ext::HSD_JObjSetFlags(child, JOBJ_HIDDEN as u32);
            return;
        }
        if val == 1 {
            child = child_of(jobj);
            child = if child.is_null() { null_mut() } else { (*child).next };
            ext::HSD_JObjSetFlags(child, JOBJ_HIDDEN as u32);
            return;
        }
    } else {
        let mut child = child_of(jobj);
        ext::HSD_JObjSetFlags(child, JOBJ_HIDDEN as u32);

        child = child_of(jobj);
        child = if child.is_null() { null_mut() } else { (*child).next };
        ext::HSD_JObjSetFlags(child, JOBJ_HIDDEN as u32);
    }
}

pub unsafe extern "C" fn fn_80192938() {
    melee_core::ct!("fn_80192938");
    let state = st();
    let tm = gm_GetTournamentData();
    lbl_804D6658 = 0;
    (*tm).cur_option = 9;

    if gm_804771C4.match_type == 0 {
        (*tm).x2E = lbl_803D9D20_x0((*tm).entrants);
        (*tm).x2F = (*tm).hmn_cpu_count as u8;
    } else {
        let comb = (*tm).combatants as u8;
        (*tm).x2E = comb;
        (*tm).x2F = comb;
        (*tm).x30 = (*tm).entrants as u8;
    }

    let mut i: i32 = 0;
    while i < 0x40 {
        let m = x37(tm, i);
        (*m).x5 = 0;
        (*m).xD = i as u8;

        if (*ext::gmMainLib_GetGameRules()).handicap == 1 {
            if gm_804771C4.match_type == 0 {
                (*ext::gmMainLib_GetGameRules()).handicap = 0;
                (*m).x2 = 9;
            } else {
                (*m).x2 = 5;
            }
        } else {
            (*m).x2 = 9;
        }

        (*m).x3 = fn_8018F410() as u8;
        (*m).x7 = ext::HSD_Randi(ext::gm_GetNumCostumesForCKind(fn_8018F6FC((*m).x3 as i32) as u8) as i32) as u8;

        // ((u8*) &tm->x37[i])[-1]
        let before = (m as *mut u8).sub(1);
        if i < (*tm).x2E as i32 {
            *before = 1;
            if i < (*tm).x2F as i32 {
                (*m).x0 = 0;
                (*m).x9 = (i + 0x320) as u16;
            } else {
                (*m).x0 = 1;
                (*m).x9 = ((i + 0x384) - (*tm).x2F as i32) as u16;
                if gm_804771C4.match_type == 0 && (*tm).cpu_level == 0 {
                    (*m).x1 = (ext::HSD_Randi(9) + 1) as u8;
                } else {
                    (*m).x1 = (*tm).cpu_level as u8;
                }
            }
        } else {
            *before = 0;
            (*m).x0 = 3;
            (*m).x9 = 0x3E7;
        }
        i += 1;
    }

    (*tm).x4B8[0].x2 = 0;
    (*tm).x4B8[1].x2 = 0;
    let mut start: i32 = 0;
    (*tm).x4B8[2].x2 = 0;
    (*tm).x4B8[3].x2 = 0;

    if gm_804771C4.match_type == 0 {
        if lbl_804D665C < 2 {
            start = 1;
        }
        let mut j = 0;
        while j < 0x3E8 {
            let a = start + ext::HSD_Randi((*tm).x2E as i32 - start);
            let b = start + ext::HSD_Randi((*tm).x2E as i32 - start);
            let tmp = (*x37(tm, a)).xD;
            (*x37(tm, a)).xD = (*x37(tm, b)).xD;
            (*x37(tm, b)).xD = tmp;
            j += 1;
        }
    }

    (*state).x2 = 0;
    (*state).x3 = 0;
    (*state).x4 = 0xB;
    (*state).x5 = 0;
    (*state).x6 = 0;
    (*state).x0 = (*ext::gmMainLib_GetGameRules()).handicap;
    (*tm).x24 = 0;
    (*tm).pad_x34[0] = 0xFF;
    (*tm).x33 = 0xFF;
}

pub unsafe extern "C" fn fn_80192BB0() {
    melee_core::ct!("fn_80192BB0");
    fn_8019035C(1, *(*lbl_804D664C).models.add(2), 0, 0x1A, 2, 1, Some(fn_8019175C), 0.0);

    let mut i: i32 = 0;
    while i < 6 {
        let gobj = fn_8019035C(1, *(*lbl_804D664C).models.add(3), 0, 0x1A, 2, 1, Some(fn_80191240), 0.0);
        let jobj = gobj_jobj(gobj);
        fn_8018FBD8(&mut *gobj, i);
        set_ty(jobj, -fmadds(3.5, i as f32, -11.5));

        let gobj = fn_8019035C(1, *(*lbl_804D664C).models.add(0), 0, 0x1A, 2, 1, Some(fn_801913BC), 0.0);
        let jobj = gobj_jobj(gobj);
        fn_8018FBD8(&mut *gobj, i);
        set_ty(jobj, -fmadds(3.5, i as f32, -11.5));

        let gobj = fn_8019035C(1, *(*lbl_804D664C).models.add(1), 0, 0x1A, 2, 1, Some(fn_80191678), 0.0);
        let jobj = gobj_jobj(gobj);
        fn_8018FBD8(&mut *gobj, i);
        set_ty(jobj, -fmadds(3.5, i as f32, -11.5));
        i += 1;
    }
}

pub unsafe extern "C" fn fn_80192E6C() {
    melee_core::ct!("fn_80192E6C");
    let m = (*lbl_804D6650).models;
    fn_8019035C(1, *m.add(4), 0, 0x1A, 2, 1, Some(fn_801918F0), 0.0);
    fn_8019035C(1, *m.add(7), 0, 0x1A, 2, 1, Some(fn_80191A54), 0.0);
    fn_8019035C(1, *m.add(5), 0, 0x1A, 2, 1, Some(core::mem::transmute::<unsafe extern "C" fn(*mut c_void), unsafe extern "C" fn(*mut HSD_GObj)>(fn_80191B5C)), 0.0);
    fn_8019035C(1, *m.add(0), 0, 0x1A, 2, 1, Some(fn_80191CA4), 0.0);
    fn_8019035C(1, *m.add(6), 0, 0x1A, 2, 1, Some(fn_8019237C), 0.0);
    fn_8019035C(1, *m.add(10), 0, 0x1A, 2, 1, Some(fn_8019249C), 0.0);
    let g = fn_8019035C(1, *m.add(2), 0, 0x1A, 2, 1, Some(fn_80192758), 0.0);
    ext::HSD_JObjSetFlagsAll(gobj_jobj(g), JOBJ_HIDDEN as u32);

    let mut i: i32 = 0;
    while i < 0x40 {
        let gobj = fn_8019035C(1, *m.add(8), 0, 0x1A, 2, 1, Some(fn_80191D38), 0.0);
        fn_8018FDC4(gobj_jobj(gobj), -12.300001, 12.800008, 666.0);
        fn_8018FBD8(&mut *gobj, i);

        let gobj = fn_8019035C(1, *m.add(3), 0, 0x1A, 2, 1, Some(fn_80191E9C), 0.0);
        let jobj = gobj_jobj(gobj);
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        fn_8018FDC4(jobj, -12.300001, 666.0, 666.0);
        fn_8018FBD8(&mut *gobj, i);
        i += 1;
    }

    let mut j: i32 = 0;
    while j <= 0x19 {
        let new_var2 = j as f32;
        let gobj = fn_8019035C(1, *m.add(9), 0, 0x1A, 2, 1, Some(fn_80191FD4), new_var2);
        let jobj = gobj_jobj(gobj);
        if j != 0x19 {
            fn_8018FF9C(jobj, 0.65, 0.66, 666.0);
            let q = j / 5;
            fn_8018FDC4(jobj, fmadds(6.0, (j % 5) as f32, -3.0), -fmadds(4.500006, q as f32, -2.3), 0.2);
        } else {
            fn_8018FDC4(jobj, -1.0, 10.099993, 0.2);
        }
        fn_8018FBD8(&mut *gobj, j);
        j += 1;
    }

    fn_8019035C(1, *m.add(1), 0, 0x1A, 2, 1, Some(fn_80192690), 0.0);
}

pub unsafe extern "C" fn fn_80193230() {
    melee_core::ct!("fn_80193230");
    fn_8019035C(0, *(*lbl_804D664C).models.add(5), 0, 0x1A, 2, 1, Some(fn_801910E0), 0.0);
    fn_8019035C(0, *(*lbl_804D664C).models.add(4), 0, 0x1A, 2, 1, Some(fn_80191154), 0.0);
    fn_80192BB0();
    fn_80192E6C();
    let gobj = ext::GObj_Create(0xE, 0x1A, 0);
    let fog = ext::HSD_FogLoadDesc((*(*lbl_804D664C).fogs).desc);
    ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_FogKind as u8, fog as *mut c_void);
    ext::GObj_SetupGXLink(gobj, Some(ext::HSD_GObj_FogCallback), 0, 0);
}

pub unsafe extern "C" fn fn_80193308() {
    melee_core::ct!("fn_80193308");
    let color = lbl_804DA78C;
    let tm = gm_GetTournamentData();
    let lbl = lbl_804D663C;

    (*tm).x4E0 = ext::HSD_SisLib_803A5ACC(0, lbl, 124.5, 45.0, 0.0, 391.0, 30.0);
    (*(*tm).x4E0).default_fitting = 1;
    ext::HSD_SisLib_803A6368((*tm).x4E0, 0x4A);

    (*tm).x4E4 = ext::HSD_SisLib_803A5ACC(0, lbl, 393.0, 221.7, 0.0, 360.0, 40.0);
    (*(*tm).x4E4).hidden = 1;

    let mut i: i32 = 0;
    while i < 6 {
        (*tm).x4E8[i as usize] = ext::HSD_SisLib_803A5ACC(0, lbl, 160.0, 48.0, 0.0, 360.0, 40.0);
        let text = (*tm).x4E8[i as usize];
        (*text).pos_x = 123.0;
        let y = fmadds(35.7, i as f32, 113.0);
        (*text).pos_y = y;
        (*text).pos_z = 0.0;
        (*(*tm).x4E8[i as usize]).hidden = 1;
        ext::HSD_SisLib_803A6368((*tm).x4E8[i as usize], 0x5C);

        (*tm).x500[i as usize] = ext::HSD_SisLib_803A5ACC(0, lbl, 160.0, 48.0, 0.0, 360.0, 40.0);
        let text = (*tm).x500[i as usize];
        (*text).pos_x = 343.0;
        (*text).pos_y = y;
        (*text).pos_z = 0.0;
        (*(*tm).x500[i as usize]).hidden = 1;
        ext::HSD_SisLib_803A6368((*tm).x500[i as usize], 0x53);
        i += 1;
    }

    (*tm).x518[0] = ext::HSD_SisLib_803A6754(0, lbl);
    let text = (*tm).x518[0];
    (*text).font_size.x = 0.5;
    (*text).font_size.y = 0.5;
    (*(*tm).x518[0]).default_kerning = 1;

    let x518 = (*tm).x518.as_mut_ptr();
    let mut count = 0;
    let mut idx: usize = 1;
    loop {
        let created = ext::HSD_SisLib_803A6754(0, lbl);
        let ptr = x518.add(idx);
        *ptr = created;
        let text = *ptr;
        (*text).font_size.x = 0.58;
        (*text).font_size.y = 0.55;
        (**ptr).default_kerning = 1;
        if count != 0 {
            (**ptr).text_color = color;
        }
        count += 1;
        idx = 2;
        if !(count < 2) {
            break;
        }
    }

    count = 0;
    idx = 3;
    loop {
        let created = ext::HSD_SisLib_803A6754(0, lbl);
        // x518[3], x518[4] alias x524[0], x524[1]
        let ptr = x518.add(idx);
        *ptr = created;
        let text = *ptr;
        (*text).font_size.x = 0.85;
        (*text).font_size.y = 1.35;
        (**ptr).default_kerning = 1;
        (**ptr).default_alignment = 1;
        if count != 0 {
            (**ptr).text_color = color;
        }
        count += 1;
        idx = 4;
        if !(count < 2) {
            break;
        }
    }
}

pub unsafe extern "C" fn fn_801935B8() {
    melee_core::ct!("fn_801935B8");
    let state = st();
    let tm = gm_GetTournamentData();
    fn_8018FBE0(0, 0, 0, 5, 5, 0x3e7, 3);
    fn_801902F0(fn_80190174((*(*lbl_804D664C).cameras).desc) as usize as i32);
    fn_80193308();
    fn_8019027C((*lbl_804D664C).lights as *mut c_void);
    fn_8019035C(0, *(*lbl_804D664C).models.add(5), 0, 0x1A, 2, 1, Some(fn_801910E0), 0.0);
    fn_8019035C(0, *(*lbl_804D664C).models.add(4), 0, 0x1A, 2, 1, Some(fn_80191154), 0.0);
    fn_80192BB0();
    fn_80192E6C();
    let gobj = ext::GObj_Create(0xE, 0x1A, 0);
    let fog = ext::HSD_FogLoadDesc((*(*lbl_804D664C).fogs).desc);
    ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_FogKind as u8, fog as *mut c_void);
    ext::GObj_SetupGXLink(gobj, Some(ext::HSD_GObj_FogCallback), 0, 0);

    lbl_804D6658 = 0;
    (*state).xC_counter = 0;
    (*state).xE = 0;
    (*state).xF = 0;
    (*state).pad2[12] = 0;

    let mut i: i32 = 0;
    while i < 6 {
        (*state).pad2[i as usize] = 0xa;
        (*state).pad2[i as usize + 6] = 0;
        *mt_ptr(tm, i) = lbl_803D9F80[MIN_OFF + i as usize * 2] as i32;
        i += 1;
    }

    (*state).x1 = 0;
    (*state).x0 = (*ext::gmMainLib_GetGameRules()).handicap;

    (*tm).x32 = 0;
    let mut i: i32 = 0;
    while i < 25 {
        (*addr_of_mut!(lbl_803D9D20)).x72[i as usize] = ext::gm_IsCKindUnlocked(fn_8018F6FC(i) as u8) as u8;
        i += 1;
    }
}

/// Step the selected setting down, wrapping to its maximum.
unsafe fn tmSettings_StepDown(menu: *mut i32, mt: *const i32) {
    sfxMove();
    lbl_804799B8.x7 = 5;
    let idx = *menu;
    let ptr = menu.offset(idx as isize + 1);
    let val = *ptr;
    if val > tbl_min(idx, *mt) {
        *ptr = val - 1;
    } else {
        *ptr = tbl_max(idx, *mt);
    }
}

/// Step the selected setting up, wrapping to its minimum.
unsafe fn tmSettings_StepUp(menu: *mut i32, mt: *const i32) {
    sfxMove();
    lbl_804799B8.x8 = 5;
    let idx = *menu;
    let ptr = menu.offset(idx as isize + 1);
    let val = *ptr;
    if val < tbl_max(idx, *mt) {
        *ptr = val + 1;
    } else {
        *ptr = tbl_min(idx, *mt);
    }
}

/// Redraw the settings HUD after input.
pub unsafe fn tmSettings_Refresh(mask: u32) {
    melee_core::ct!("tmSettings_Refresh");
    if mask & 0x300 != 0 {
        fn_80190ABC(0);
        fn_80190ABC(2);
    }
    fn_80190ABC(3);
}

#[inline]
unsafe fn mt_val() -> i32 {
    gm_804771C4.match_type
}
#[inline]
unsafe fn mt_p() -> *const i32 {
    addr_of_mut!(gm_804771C4.match_type)
}

pub unsafe extern "C" fn fn_801937C4(arg0: *mut i32, arg1: u32, arg2: u32) {
    melee_core::ct!("fn_801937C4");
    if arg1 & 0x40001 != 0 {
        tmSettings_StepDown(arg0, mt_p());
    } else if arg1 & 0x80002 != 0 {
        tmSettings_StepUp(arg0, mt_p());
    }

    if arg1 != 0 {
        tmSettings_Refresh(arg1 | arg2);
    }

    if arg2 & 0x100 != 0 {
        sfxForward();
        let mut idx = *arg0;
        *arg0 = idx + 1;
        idx = *arg0;
        let mut dp = arg0.offset(idx as isize);
        while idx < 6 {
            dp = dp.add(1);
            *dp = lbl_803D9F80[(idx as usize) * 2 + (mt_val() != 0) as usize + MIN_OFF] as i32;
            idx += 1;
        }
        if mt_val() == 0 {
            if fn_8018F808() < 2 {
                let idx = *arg0;
                *arg0.offset(idx as isize + 1) = 2;
            } else {
                let count = fn_8018F808();
                let idx = *arg0;
                *arg0.offset(idx as isize + 1) = count;
            }
        }
    } else if arg2 & 0x200 != 0 {
        sfxBack();
        ext::gm_ChangeGameModeAfterCurrentScene(GM_MENU as i32);
        ext::gm_801A4B60();
    }

    if arg2 != 0 {
        tmSettings_Refresh(arg1 | arg2);
    }
}

pub unsafe extern "C" fn fn_80193B58(arg0: *mut i32, arg1: u32, arg2: u32) {
    melee_core::ct!("fn_80193B58");
    let state = st();
    let mt = mt_p();
    let mut clamp_val: i32 = 0;

    if *mt == 0 {
        lbl_804D665C = fn_8018F808();
    }

    if *mt == 0 {
        clamp_val = core::cmp::max(2, lbl_804D665C);
        let idx = *arg0;
        let ptr = arg0.offset(idx as isize + 1);
        let val = *ptr;
        if val > clamp_val {
            *ptr = clamp_val;
            fn_80190ABC(3);
        }
    }

    if arg1 & 0x40001 != 0 {
        if *mt == 0 {
            let idx = *arg0;
            let ptr = arg0.offset(idx as isize + 1);
            if *ptr > tbl_min(idx, *mt) {
                *ptr = *ptr - 1;
                sfxMove();
                (*state).x7 = 5;
            } else {
                let max = tbl_max(idx, *mt);
                if clamp_val < max {
                    *ptr = clamp_val;
                } else {
                    *ptr = max;
                }
                let idx = *arg0;
                let val = *arg0.offset(idx as isize + 1);
                if val != tbl_min(idx, *mt) {
                    sfxMove();
                    (*state).x7 = 5;
                }
            }
        } else {
            tmSettings_StepDown(arg0, mt);
        }
    } else if arg1 & 0x80002 != 0 {
        if *mt == 0 {
            let idx = *arg0;
            let ptr = arg0.offset(idx as isize + 1);
            if *ptr < tbl_max(idx, *mt) {
                if *ptr + 1 <= clamp_val {
                    *ptr = *ptr + 1;
                    sfxMove();
                    (*state).x8 = 5;
                } else {
                    *ptr = tbl_min(idx, *mt);
                    let idx = *arg0;
                    let val = *arg0.offset(idx as isize + 1);
                    if val != clamp_val {
                        sfxMove();
                        (*state).x8 = 5;
                    }
                }
            } else {
                *ptr = tbl_min(idx, *mt);
                let idx = *arg0;
                let val = *arg0.offset(idx as isize + 1);
                if val != tbl_max(idx, *mt) {
                    sfxMove();
                    (*state).x8 = 5;
                }
            }
        } else {
            tmSettings_StepUp(arg0, mt);
        }
    }

    if *mt == 0 {
        (*(arg0 as *mut TmData)).x30 = *arg0.add(2) as u8;
    } else if *arg0.add(3) > *arg0.add(2) - 1 {
        *arg0.add(3) = *arg0.add(2) - 1;
    }

    if arg1 != 0 {
        tmSettings_Refresh(arg1 | arg2);
    }

    if arg2 & 0x100 != 0 {
        sfxForward();
        *arg0 = *arg0 + 1;
        if *mt == 0 {
            let x30 = (*(arg0 as *mut TmData)).x30;
            if x30 == 2 {
                *arg0.add(3) = 0;
            } else if x30 == 3 {
                *arg0.add(3) = 9;
            } else {
                *arg0.add(3) = 0xE;
            }
            let tm = gm_GetTournamentData();
            fn_8018EC7C();
            fn_8018E618((*tm).entrants, 4.5, 1);
            fn_80190480(130.0);
            fn_80190520(-278.0, 255.0, 0.0);
        }
    } else if arg2 & 0x200 != 0 {
        sfxBack();
        *arg0 = *arg0 - 1;
    }

    if arg2 != 0 {
        tmSettings_Refresh(arg1 | arg2);
    }
}

pub unsafe extern "C" fn fn_80193FCC(arg0: *mut i32, arg1: u32, arg2: u32) {
    melee_core::ct!("fn_80193FCC");
    let state = st();
    let mt = mt_p();
    let mut clamp_val: i32 = 0;

    if *mt != 0 {
        lbl_804D665C = fn_8018F808();
    }

    if *mt != 0 {
        let clamp_expr = core::cmp::max(2, fn_8018F808());
        clamp_val = clamp_expr;
        let idx = *arg0;
        let ptr = arg0.offset(idx as isize + 1);
        let val = *ptr;
        if val > clamp_expr {
            *ptr = clamp_expr;
            fn_80190ABC(3);
        }
    }

    if arg1 & 0x40001 != 0 {
        if *mt != 0 {
            let idx = *arg0;
            let ptr = arg0.offset(idx as isize + 1);
            if *ptr > tbl_min(idx, *mt) {
                *ptr = *ptr - 1;
                sfxMove();
                (*state).x7 = 5;
            } else {
                let max_plus1 = tbl_max(idx, *mt) + 1;
                if clamp_val < max_plus1 {
                    *ptr = clamp_val;
                } else {
                    *ptr = max_plus1;
                }
                let idx = *arg0;
                let val = *arg0.offset(idx as isize + 1);
                if val != tbl_min(idx, *mt) {
                    sfxMove();
                    (*state).x7 = 5;
                }
            }
        } else {
            tmSettings_StepDown(arg0, mt);
        }
        if *mt == 0 {
            let x30 = (*(arg0 as *mut TmData)).x30;
            match x30 {
                2 => {
                    if *arg0.add(3) == 0x1F {
                        *arg0.add(3) = 8;
                    }
                }
                3 => {
                    if *arg0.add(3) == 8 {
                        *arg0.add(3) = 0xD;
                    }
                }
                4 => {
                    if *arg0.add(3) == 0xD {
                        *arg0.add(3) = 0x1F;
                    }
                }
                _ => {}
            }
            let tm = gm_GetTournamentData();
            fn_8018EC7C();
            fn_8018E618((*tm).entrants, 4.5, 1);
            fn_80190480(130.0);
            fn_80190520(-278.0, 255.0, 0.0);
        }
    } else if arg1 & 0x80002 != 0 {
        if *mt != 0 {
            let idx = *arg0;
            let p = arg0.offset(idx as isize + 1);
            if *p < tbl_max(idx, *mt) + 1 {
                if *p + 1 <= clamp_val && *p + 1 < *arg0.add(2) {
                    *p = *p + 1;
                    sfxMove();
                    (*state).x8 = 5;
                } else {
                    *p = tbl_min(idx, *mt);
                    let idx = *arg0;
                    let val = *arg0.offset(idx as isize + 1);
                    if val != clamp_val {
                        sfxMove();
                        (*state).x8 = 5;
                    }
                }
            } else {
                *p = tbl_min(idx, *mt);
                let idx = *arg0;
                let val = *arg0.offset(idx as isize + 1);
                if val != tbl_max(idx, *mt) + 1 {
                    sfxMove();
                    (*state).x8 = 5;
                }
            }
        } else {
            sfxMove();
            (*state).x8 = 5;
            let idx = *arg0;
            let ptr = arg0.offset(idx as isize + 1);
            let val = *ptr;
            if val < tbl_max(idx, *mt) {
                if *mt != 0 {
                    if val == *arg0.add(2) - 1 {
                        *ptr = tbl_min(idx, *mt);
                    } else {
                        *ptr = val + 1;
                    }
                } else {
                    *ptr = val + 1;
                }
            } else {
                *ptr = tbl_min(idx, *mt);
            }
        }
        if *mt == 0 {
            let x30 = (*(arg0 as *mut TmData)).x30;
            match x30 {
                2 => {
                    if *arg0.add(3) == 9 {
                        *arg0.add(3) = 0;
                    }
                }
                3 => {
                    if *arg0.add(3) == 0xE {
                        *arg0.add(3) = 9;
                    }
                }
                4 => {
                    if *arg0.add(3) == 0 {
                        *arg0.add(3) = 0xE;
                    }
                }
                _ => {}
            }
            let tm = gm_GetTournamentData();
            fn_8018EC7C();
            fn_8018E618((*tm).entrants, 4.5, 1);
            fn_80190480(130.0);
            fn_80190520(-278.0, 255.0, 0.0);
        }
    }

    if *mt != 0 {
        if *arg0.add(3) > *arg0.add(2) - 1 {
            *arg0.add(3) = *arg0.add(2) - 1;
        }
        if *arg0.add(4) > *arg0.add(3) - 1 {
            *arg0.add(4) = *arg0.add(3) - 1;
        }
        if *arg0.add(4) > *arg0.add(2) - *arg0.add(3) {
            *arg0.add(4) = *arg0.add(2) - *arg0.add(3);
        }
    }

    if *mt != 0 {
        (*(arg0 as *mut TmData)).x30 = *arg0.add(3) as u8;
    }

    if arg1 != 0 {
        if (arg1 | arg2) & 0x300 != 0 {
            fn_80190ABC(0);
            fn_80190ABC(2);
            if *mt == 0 {
                if arg2 & 0x100 != 0 {
                    if lbl_804D665C < 2 {
                        *arg0.add(4) = 1;
                    } else {
                        *arg0.add(4) = lbl_803D9D20_x0(*arg0.add(3)) as i32;
                    }
                    fn_80190ABC(1);
                }
            }
        }
        fn_80190ABC(3);
    }

    if arg2 & 0x100 != 0 {
        sfxForward();
        *arg0 = *arg0 + 1;
    } else if arg2 & 0x200 != 0 {
        sfxBack();
        *arg0 = *arg0 - 1;
        if *mt == 0 {
            fn_8018EC7C();
        }
    }

    if arg2 != 0 {
        if (arg1 | arg2) & 0x300 != 0 {
            fn_80190ABC(0);
            fn_80190ABC(2);
            if *mt == 0 {
                if arg2 & 0x100 != 0 {
                    if lbl_804D665C < 2 {
                        *arg0.add(4) = 1;
                    } else {
                        *arg0.add(4) = lbl_803D9D20_x0(*arg0.add(3)) as i32;
                    }
                    fn_80190ABC(1);
                }
            }
        }
        fn_80190ABC(3);
    }
}

pub unsafe extern "C" fn fn_80194658(arg0: *mut i32, arg1: u32, arg2: u32) {
    melee_core::ct!("fn_80194658");
    let mut changed: i32 = 0;

    let idx = *arg0;
    let dir = gm_804771C4.match_type;

    let val = *arg0.offset(idx as isize + 1);
    let ptr = arg0.offset(idx as isize + 1);

    if 0 == dir && lbl_804D665C < 2 {
        if arg1 & 0x40001 != 0 {
            *ptr = if val != 0 { 0 } else { 1 };
        } else if arg1 & 0x80002 != 0 {
            changed = 1;
            *ptr = if val != 0 { 0 } else { 1 };
        }
    } else if arg1 & 0x40001 != 0 {
        if val > tbl_min(idx, dir) {
            *ptr = *ptr - 1;
        } else if dir == 0 {
            *ptr = lbl_803D9D20_x0(*arg0.add(3)) as i32;
        } else {
            *ptr = tbl_max(idx, dir);
        }
    } else if arg1 & 0x80002 != 0 {
        changed = 1;
        if val < tbl_max(idx, dir) {
            *ptr = *ptr + 1;
            if gm_804771C4.match_type != 0 {
                if *arg0.add(4) > *arg0.add(3) - 1 || *arg0.add(4) > *arg0.add(2) - *arg0.add(3) {
                    *arg0.offset(*arg0 as isize + 1) = 1;
                }
            }
        } else if dir == 0 {
            *ptr = tbl_min(idx, dir);
        } else {
            *ptr = 1;
        }
    }
    if gm_804771C4.match_type == 0 {
        if *arg0.add(4) > lbl_803D9D20_x0(*arg0.add(3)) as i32 {
            *arg0.offset(*arg0 as isize + 1) = 0;
        }
    } else {
        if *arg0.add(4) > *arg0.add(3) - 1 {
            *arg0.add(4) = *arg0.add(3) - 1;
        }
        if *arg0.add(4) > *arg0.add(2) - *arg0.add(3) {
            *arg0.add(4) = *arg0.add(2) - *arg0.add(3);
        }
    }

    if val != *arg0.add(4) {
        sfxMove();
        if changed == 0 {
            lbl_804799B8.x7 = 5;
        } else {
            lbl_804799B8.x8 = 5;
        }
    }

    if arg1 != 0 {
        tmSettings_Refresh(arg1 | arg2);
        fn_80190ABC(1);
    }

    if arg2 & 0x100 != 0 {
        sfxForward();
        *arg0 = *arg0 + 1;
    } else if arg2 & 0x200 != 0 {
        sfxBack();
        *arg0 = *arg0 - 1;
    }

    if arg2 != 0 {
        tmSettings_Refresh(arg1 | arg2);
        fn_80190ABC(1);
    }
}

/// Handles tournament rules menu input.
pub unsafe extern "C" fn fn_801949B4(arg0: *mut i32, arg1: u32, arg2: u32) {
    melee_core::ct!("fn_801949B4");
    if arg1 & 0x40001 != 0 {
        tmSettings_StepDown(arg0, mt_p());
    } else if arg1 & 0x80002 != 0 {
        tmSettings_StepUp(arg0, mt_p());
    }

    if arg1 != 0 {
        if (arg1 | arg2) & 0x300 != 0 {
            fn_80190ABC(0);
            fn_80190ABC(2);
            fn_80190ABC(1);
        }
        fn_80190ABC(3);
    }

    if arg2 & 0x100 != 0 {
        sfxForward();
        if mt_val() == 0 {
            *arg0 = *arg0 + 1;
        } else {
            *arg0 = 6;
        }
    } else if arg2 & 0x200 != 0 {
        sfxBack();
        *arg0 = *arg0 - 1;
    }

    if arg2 != 0 {
        if (arg1 | arg2) & 0x300 != 0 {
            fn_80190ABC(0);
            fn_80190ABC(2);
            fn_80190ABC(1);
        }
        fn_80190ABC(3);
    }
}

pub unsafe extern "C" fn fn_80194BC4(arg0: *mut i32, arg1: u32, arg2: u32) {
    melee_core::ct!("fn_80194BC4");
    if *mt_p() != 0 {
        return;
    }

    if arg1 & 0x40001 != 0 {
        tmSettings_StepDown(arg0, mt_p());
    } else if arg1 & 0x80002 != 0 {
        tmSettings_StepUp(arg0, mt_p());
    }

    if arg1 != 0 {
        if (arg1 | arg2) & 0x300 != 0 {
            fn_80190ABC(0);
        }
        fn_80190ABC(3);
    }

    if arg2 & 0x100 != 0 {
        sfxForward();
        *arg0 = *arg0 + 1;
    } else if arg2 & 0x200 != 0 {
        sfxBack();
        *arg0 = *arg0 - 1;
    }

    if arg2 != 0 {
        if (arg1 | arg2) & 0x300 != 0 {
            fn_80190ABC(0);
        }
        fn_80190ABC(3);
    }
}

/// Handles button input for tournament mode menu navigation.
pub unsafe extern "C" fn fn_80194D84(state: *mut i32, buttons: u32, trigger: u32) {
    melee_core::ct!("fn_80194D84");
    if buttons & (PAD_BUTTON_LEFT | PAD_STICK_LEFT) != 0 {
        if *state > 6 {
            sfxMove();
            *state -= 1;
        }
    } else if (buttons & (PAD_BUTTON_RIGHT | PAD_STICK_RIGHT)) != 0 && *state < 8 {
        sfxMove();
        *state += 1;
    }

    if buttons != 0 && ((buttons | trigger) & 0x300) != 0 {
        fn_80190ABC(0);
    }

    if trigger & (PAD_BUTTON_A | PAD_BUTTON_START) != 0 {
        sfxForward();
        fn_8018EC7C();
        let cur_state = *state;
        if cur_state == 6 {
            fn_80192938();
        } else if cur_state == 7 {
            let tm = gm_GetTournamentData();
            (*tm).x20 = 1;
            ext::HSD_SisLib_803A5E70();
            ext::mn_8022F138(0x19, 0x1C);
            ext::mn_8022F138(0x12, 0x15);
            if gm_804771C4.match_type == 0 {
                ext::mn_802317E4(lbl_804D6648, 0);
            } else {
                ext::mn_802317E4(lbl_804D6648, 1);
            }
            return;
        } else {
            *state = 0;
        }
    } else if trigger & PAD_BUTTON_B != 0 {
        sfxBack();
        if gm_804771C4.match_type == 0 {
            *state = 5;
        } else {
            *state = 4;
        }
    }

    if trigger != 0 && ((buttons | trigger) & 0x300) != 0 {
        fn_80190ABC(0);
    }
}

#[inline]
unsafe fn tm_x37(base: *mut i32, idx: i32) -> *mut TmUnkMenuData {
    (((base as *mut u8).add(0x37)) as *mut TmUnkMenuData).offset(idx as isize)
}

/// Handles tournament settings menu input (entrant configuration).
pub unsafe extern "C" fn fn_80194F30(state_ptr: *mut i32, buttons: u32, trigger: u32) {
    melee_core::ct!("fn_80194F30");
    let state = st();
    let tm = state_ptr as *mut TmData;

    if trigger & PAD_BUTTON_START != 0 {
        sfxForward();
        lbl_804D6654 = *state_ptr as u32;
        *state_ptr = 0x11;
        return;
    }

    if buttons & (PAD_BUTTON_LEFT | PAD_STICK_LEFT) != 0 {
        match *state_ptr {
            12 => {
                sfxMove();
                *state_ptr = 0xA;
            }
            16 => {
                sfxMove();
                *state_ptr = 0xC;
                (*state).x1 = 0;
            }
            _ => {}
        }
    } else if buttons & (PAD_BUTTON_RIGHT | PAD_STICK_RIGHT) != 0 {
        match *state_ptr {
            11 => {}
            10 => {
                sfxMove();
                *state_ptr = 0xC;
            }
            12 => {
                if (*state).x0 != 0 {
                    sfxMove();
                    *state_ptr = 0x10;
                }
            }
            _ => {}
        }
    } else if buttons & (PAD_BUTTON_UP | PAD_STICK_UP) != 0 {
        if *state_ptr != 0x10 || (*state).x1 == 0 {
            if (*state).x2 != 0 {
                sfxMove();
                (*state).x2 -= 1;
                fn_80190ABC(5);
            } else if (*state).x3 != 0 {
                sfxMove();
                (*state).x3 -= 1;
                (*state).x4 = (*state).x4.wrapping_sub(1);
                fn_80190ABC(5);
                fn_80190ABC(6);
            }
        } else {
            let idx = (*state).x2 as i32 + (*state).x3 as i32;
            if (*x37(tm, idx)).x2 > 1 {
                sfxMove();
                let idx = (*state).x2 as i32 + (*state).x3 as i32;
                (*x37(tm, idx)).x2 -= 1;
            }
        }
    } else if buttons & (PAD_BUTTON_DOWN | PAD_STICK_DOWN) != 0 {
        if *state_ptr != 0x10 || (*state).x1 == 0 {
            let pos = (*state).x2;
            if pos < 0xB {
                if (pos as i32 + (*state).x3 as i32) < (*tm).x2E as i32 - 1 {
                    sfxMove();
                    (*state).x2 += 1;
                    fn_80190ABC(5);
                }
            } else if (pos as i32 + (*state).x3 as i32) < (*tm).x2E as i32 - 1 {
                sfxMove();
                (*state).x3 += 1;
                (*state).x4 = (*state).x4.wrapping_add(1);
                fn_80190ABC(5);
                fn_80190ABC(6);
            }
        } else {
            let idx = (*state).x2 as i32 + (*state).x3 as i32;
            if (*x37(tm, idx)).x2 < 9 {
                sfxMove();
                let idx = (*state).x2 as i32 + (*state).x3 as i32;
                (*x37(tm, idx)).x2 += 1;
            }
        }
    }

    if trigger & PAD_BUTTON_A != 0 {
        match *state_ptr {
            12 | 10 => {
                sfxForward();
                let idx = (*state).x2 as i32 + (*state).x3 as i32;
                (*x37(tm, idx)).x4 = (*x37(tm, idx)).x3;
                let idx = (*state).x2 as i32 + (*state).x3 as i32;
                (*x37(tm, idx)).x8 = (*x37(tm, idx)).x7;
                let idx = (*state).x2 as i32 + (*state).x3 as i32;
                // tm_alt->x37[idx].xB (u16 at +0xB) = x9
                core::ptr::write_unaligned(
                    (x37(tm, idx) as *mut u8).add(0xB) as *mut u16,
                    (*x37(tm, idx)).x9,
                );
                let idx = (*state).x2 as i32 + (*state).x3 as i32;
                (*x37(tm, idx)).x6 = (*x37(tm, idx)).x5;
                *state_ptr += 1;
                return;
            }
            16 => {
                if (*state).x0 != 1 {
                    if (*state).x1 != 1 {
                        sfxForward();
                    }
                    (*state).x1 = 1;
                    return;
                }
            }
            _ => {}
        }
    } else if trigger & PAD_BUTTON_B != 0 {
        sfxBack();
        if *state_ptr != 0x10 || (*state).x1 == 0 {
            *state_ptr = 6;
            (*state).xE = 0;
            if gm_804771C4.match_type == 0 {
                let tmdata = gm_GetTournamentData();
                fn_8018EC7C();
                fn_8018E618((*tmdata).entrants, 4.5, 1);
                fn_80190480(130.0);
                fn_80190520(-278.0, 255.0, 0.0);
            }
        } else {
            (*state).x1 = 0;
        }
    }
}

#[inline]
unsafe fn fn_801953C8_GetPreviousPosition(offset: i32, position: i32) -> i32 {
    position - offset
}

#[inline]
unsafe fn fn_801953C8_GetPlayerIndex() -> i32 {
    lbl_804799B8.x2 as i32 + lbl_804799B8.x3 as i32
}

#[inline]
unsafe fn slot_idx() -> i32 {
    lbl_804799B8.x2 as i32 + lbl_804799B8.x3 as i32
}

/// The `occupied` test shared by the four direction scans of `fn_801953C8`.
#[inline]
unsafe fn fn_801953C8_occupied(adj: i32) -> bool {
    (*addr_of_mut!(lbl_803D9D20)).x72[adj as usize] != 0 || fn_8018F3D0(fn_8018F310(fn_8018F6FC(adj))) != 2
}

/// Applies the chosen candidate icon to the current slot (clears costume and the x5 flag).
#[inline]
unsafe fn fn_801953C8_apply(tm: *mut TmData, candidate: i32) {
    sfxMove();
    let idx = slot_idx();
    (*x37(tm, idx)).x7 = 0;
    let idx = slot_idx();
    (*x37(tm, idx)).x5 = 0;
    let adj = fn_8018F6DC(fn_8018F3BC(candidate));
    let idx = slot_idx();
    (*x37(tm, idx)).x3 = adj as u8;
}

pub unsafe extern "C" fn fn_801953C8(state_ptr: *mut i32, buttons: u32, trigger: u32) {
    melee_core::ct!("fn_801953C8");
    let tm = state_ptr as *mut TmData;

    let idx = slot_idx();
    let cur_pos = fn_8018F310(fn_8018F6FC((*x37(tm, idx)).x3 as i32));

    if trigger & PAD_BUTTON_START != 0 {
        sfxBack();
        let idx = slot_idx();
        (*x37(tm, idx)).x3 = (*x37(tm, idx)).x4;
        let idx = slot_idx();
        (*x37(tm, idx)).x7 = (*x37(tm, idx)).x8;
        let idx = slot_idx();
        (*x37(tm, idx)).x5 = (*x37(tm, idx)).x6;
        lbl_804D6654 = *state_ptr as u32;
        *state_ptr = 0x11;
        return;
    }

    if buttons & (PAD_BUTTON_LEFT | PAD_STICK_LEFT) != 0 {
        let group = sdiv(cur_pos, 5) * 5;
        let mut i = 1;
        while i < 5 {
            let step = fn_801953C8_GetPreviousPosition(i, cur_pos);
            let mut candidate = step;
            if step < group {
                candidate += 5;
            }
            let adj = fn_8018F6DC(fn_8018F3BC(candidate));
            if fn_801953C8_occupied(adj) {
                fn_801953C8_apply(tm, candidate);
                break;
            }
            i += 1;
        }
    } else if buttons & (PAD_BUTTON_RIGHT | PAD_STICK_RIGHT) != 0 {
        let group = (sdiv(cur_pos, 5) + 1) * 5;
        let mut i = 1;
        while i < 5 {
            let step = cur_pos + i;
            let mut candidate = step;
            if step >= group {
                candidate -= 5;
            }
            let adj = fn_8018F6DC(fn_8018F3BC(candidate));
            if fn_801953C8_occupied(adj) {
                fn_801953C8_apply(tm, candidate);
                break;
            }
            i += 1;
        }
    } else if buttons & (PAD_BUTTON_UP | PAD_STICK_UP) != 0 {
        let mut i = 1;
        while i < 5 {
            let temp = cur_pos - i * 5;
            let mut candidate = temp;
            if temp < 0 {
                candidate += 25;
            }
            let adj = fn_8018F6DC(fn_8018F3BC(candidate));
            if fn_801953C8_occupied(adj) {
                fn_801953C8_apply(tm, candidate);
                break;
            }
            i += 1;
        }
    } else if buttons & (PAD_BUTTON_DOWN | PAD_STICK_DOWN) != 0 {
        let mut i = 1;
        while i < 5 {
            let mut next_candidate = cur_pos + i * 5;
            if 25 <= next_candidate {
                next_candidate -= 25;
            }
            let adj = fn_8018F6DC(fn_8018F3BC(next_candidate));
            if fn_801953C8_occupied(adj) {
                fn_801953C8_apply(tm, next_candidate);
                break;
            }
            i += 1;
        }
    }

    if trigger & PAD_BUTTON_A != 0 {
        if (fn_8018F6A8(4) & 0x40) != 0 && (fn_8018F6A8(4) & 0x20) != 0 && gm_804771C4.match_type != 0 {
            let idx = slot_idx();
            if (*addr_of_mut!(lbl_803D9D20)).x72[(*x37(tm, idx)).x3 as usize] == 0 {
                ext::lbAudioAx_80024030(3);
                return;
            }
            sfxForward();
            let idx = slot_idx();
            (*x37(tm, idx)).x5 = 1;
            *state_ptr -= 1;
            return;
        }
        let idx = slot_idx();
        if (*addr_of_mut!(lbl_803D9D20)).x72[(*x37(tm, idx)).x3 as usize] == 0 {
            ext::lbAudioAx_80024030(3);
            return;
        }
        sfxForward();
        *state_ptr -= 1;
        let idx = fn_801953C8_GetPlayerIndex();
        if (*x37(tm, idx)).x5 != 0 {
            (*x37(tm, idx)).x5 = 0;
        }
    } else if trigger & PAD_BUTTON_B != 0 {
        sfxBack();
        let idx = slot_idx();
        (*x37(tm, idx)).x3 = (*x37(tm, idx)).x4;
        let idx = slot_idx();
        (*x37(tm, idx)).x7 = (*x37(tm, idx)).x8;
        let idx = slot_idx();
        (*x37(tm, idx)).x5 = (*x37(tm, idx)).x6;
        *state_ptr -= 1;
    } else if trigger & PAD_BUTTON_X != 0 {
        let cur = x37(tm, slot_idx());
        if ((*cur).x7 as i32)
            < ext::gm_GetNumCostumesForCKind(fn_8018F6FC((*x37(tm, slot_idx())).x3 as i32) as u8) as i32 - 1
        {
            sfxMove();
            let idx = slot_idx();
            (*x37(tm, idx)).x7 = (*x37(tm, idx)).x7.wrapping_add(1);
        }
    } else if trigger & PAD_BUTTON_Y != 0 {
        let idx = slot_idx();
        if (*x37(tm, idx)).x7 != 0 {
            sfxMove();
            let idx = slot_idx();
            (*x37(tm, idx)).x7 = (*x37(tm, idx)).x7.wrapping_sub(1);
        }
    }
}

/// Handles name entry/selection input for tournament mode.
pub unsafe extern "C" fn fn_80195AF0(state_ptr: *mut i32, buttons: u32, trigger: u32) {
    melee_core::ct!("fn_80195AF0");
    let menu = st();

    if trigger & PAD_BUTTON_START != 0 {
        lbl_804D6654 = *state_ptr as u32;
        *state_ptr = 0x11;
        return;
    }

    if buttons & (PAD_BUTTON_LEFT | PAD_STICK_LEFT) != 0 {
        if *state_ptr == 0xE {
            sfxMove();
            *state_ptr = 0xD;
        }
    } else if (buttons & (PAD_BUTTON_RIGHT | PAD_STICK_RIGHT)) != 0 && *state_ptr == 0xD {
        sfxMove();
        *state_ptr = 0xE;
    }

    if trigger & PAD_BUTTON_A != 0 {
        sfxForward();
        let tm_alt = state_ptr as *mut TmData;
        let x2 = addr_of_mut!((*menu).x2);
        let x3 = addr_of_mut!((*menu).x3);
        let idx = *x2 as i32 + *x3 as i32;
        // tm_alt->x37[idx].xB (u16 at +0xB) = x37[idx].x9
        core::ptr::write_unaligned((x37(tm_alt, idx) as *mut u8).add(0xB) as *mut u16, (*x37(tm_alt, idx)).x9);
        match *state_ptr {
            0xD => {
                if ext::IsNameListFull() == 0 {
                    let tm = gm_GetTournamentData();
                    (*tm).x20 = 1;
                    ext::HSD_SisLib_803A5E70();
                    ext::mn_8022F138(0x19, 0x1C);
                    ext::mn_8022F138(0x12, 0x15);
                    ext::mnNameNew_8023EA08(lbl_804D6648 as *mut c_void);
                }
                return;
            }
            0xE => {
                let x6 = addr_of_mut!((*menu).x6);
                let x5 = addr_of_mut!((*menu).x5);
                if !ext::GetNameText(*x5 as i32 + (*x6 as i32 * 4)).is_null() {
                    let idx = *x2 as i32 + *x3 as i32;
                    (*x37(tm_alt, idx)).x9 = (*x5 as i32 + (*x6 as i32 * 4)) as u16;
                }
                *state_ptr = 0xF;
                fn_80190ABC(4);
                return;
            }
            _ => {}
        }
    } else if trigger & PAD_BUTTON_B != 0 {
        sfxBack();
        let tm_alt = state_ptr as *mut TmData;
        let idx = (*menu).x2 as i32 + (*menu).x3 as i32;
        // x9 = u16 at +0xB (xB)
        (*x37(tm_alt, idx)).x9 = core::ptr::read_unaligned((x37(tm_alt, idx) as *mut u8).add(0xB) as *const u16);
        fn_80190ABC(5);
        fn_80190ABC(6);
        *state_ptr = 0xC;
    }
}

/// Returns 1 when `selected` is not already taken by another entrant slot.
#[inline]
unsafe fn fn_80195CCC_IsUniqueEntry(tm: *mut TmData, count: i32, slot: i32, selected: i32) -> i32 {
    let mut i = 0;
    while i < count {
        if i != slot && selected == (*x37(tm, i)).x9 as i32 {
            return 0;
        }
        i += 1;
    }
    1
}

pub unsafe extern "C" fn fn_80195CCC(arg: *mut i32, buttons: u32, trigger: u32) {
    melee_core::ct!("fn_80195CCC");
    let mut menu: *mut u8 = st_byte(0);
    let tm_alt = arg as *mut TmData;

    if buttons & (PAD_BUTTON_LEFT | PAD_STICK_LEFT) != 0 {
        sfxMove();
        let ptr = menu.add(5);
        if *ptr % 4 != 0 {
            *ptr -= 1;
            let slot = lbl_804799B8.x2 as i32 + lbl_804799B8.x3 as i32;
            (*x37(tm_alt, slot)).x9 = (*ptr as i32 + (lbl_804799B8.x6 as i32 * 4)) as u16;
            fn_80190ABC(5);
            fn_80190ABC(4);
        }
    } else if buttons & (PAD_BUTTON_RIGHT | PAD_STICK_RIGHT) != 0 {
        sfxMove();
        let right_x5 = menu.add(5);
        let x6_tmp = menu.add(6);
        let i = (*x6_tmp as i32 * 4) + 1;
        if ((*right_x5 % 4) as i32) < 3 && (*right_x5 as i32 + i) < ext::GetNameCount() {
            *right_x5 += 1;
            let slot = lbl_804799B8.x2 as i32 + lbl_804799B8.x3 as i32;
            (*x37(tm_alt, slot)).x9 = (*right_x5 as i32 + (*x6_tmp as i32 * 4)) as u16;
            fn_80190ABC(5);
            fn_80190ABC(4);
        }
    } else if buttons & (PAD_BUTTON_UP | PAD_STICK_UP) != 0 {
        sfxMove();
        if (lbl_804799B8.x5 as i32 / 4) != 0 {
            lbl_804799B8.x5 -= 4;
            let slot = lbl_804799B8.x2 as i32 + lbl_804799B8.x3 as i32;
            (*x37(tm_alt, slot)).x9 = (lbl_804799B8.x5 as i32 + (lbl_804799B8.x6 as i32 * 4)) as u16;
            fn_80190ABC(5);
            fn_80190ABC(4);
        } else if lbl_804799B8.x6 != 0 {
            lbl_804799B8.x6 -= 1;
            let slot = lbl_804799B8.x2 as i32 + lbl_804799B8.x3 as i32;
            (*x37(tm_alt, slot)).x9 = (lbl_804799B8.x5 as i32 + (lbl_804799B8.x6 as i32 * 4)) as u16;
            fn_80190ABC(5);
            fn_80190ABC(4);
        }
    } else if buttons & (PAD_BUTTON_DOWN | PAD_STICK_DOWN) != 0 {
        sfxMove();
        let ptr = menu.add(5);
        if (*ptr as i32 / 4) < 8 {
            let x6_tmp = menu.add(6);
            let i = (*x6_tmp as i32 * 4) + 4;
            if (*ptr as i32 + i) < ext::GetNameCount() {
                *ptr += 4;
                let slot = lbl_804799B8.x2 as i32 + lbl_804799B8.x3 as i32;
                (*x37(tm_alt, slot)).x9 = (*ptr as i32 + (*x6_tmp as i32 * 4)) as u16;
            }
            fn_80190ABC(5);
            fn_80190ABC(4);
        } else {
            let base = *ptr;
            let page_ptr = menu.add(6);
            let i = (*page_ptr as i32 * 4) + 4;
            if (base as i32 + i) < ext::GetNameCount() {
                *page_ptr += 1;
                let slot = lbl_804799B8.x2 as i32 + lbl_804799B8.x3 as i32;
                (*x37(tm_alt, slot)).x9 = (*ptr as i32 + (*page_ptr as i32 * 4)) as u16;
                fn_80190ABC(5);
                fn_80190ABC(4);
            }
        }
    }

    if trigger & PAD_BUTTON_A != 0 {
        let x6_ptr = menu.add(6);
        let x5 = menu.add(5);
        if !ext::GetNameText(*x5 as i32 + (*x6_ptr as i32 * 4)).is_null() {
            let selected = *x5 as i32 + (*x6_ptr as i32 * 4);
            let x2_ptr = menu.add(2);
            menu = menu.add(3);
            let slot = *x2_ptr as i32 + *menu as i32;
            let tm = gm_GetTournamentData();
            let count = (*tm).x2E;
            let unique = fn_80195CCC_IsUniqueEntry(tm, count as i32, slot, selected);
            if unique != 0 {
                sfxForward();
                let slot = *x2_ptr as i32 + *menu as i32;
                (*x37(tm_alt, slot)).x9 = (*x5 as i32 + (*x6_ptr as i32 * 4)) as u16;
                fn_80190ABC(5);
                fn_80190ABC(6);
                *arg = 0xC;
                return;
            }
        }
        ext::lbAudioAx_80024030(3);
        return;
    }

    if trigger & PAD_BUTTON_B != 0 {
        sfxBack();
        // menu is still &x0 here only when the A path did not run (it returns before)
        let slot = *menu.add(2) as i32 + *menu.add(3) as i32;
        (*x37(tm_alt, slot)).x9 = core::ptr::read_unaligned((x37(tm_alt, slot) as *mut u8).add(0xB) as *const u16);
        fn_80190ABC(5);
        fn_80190ABC(6);
        *arg = 0xE;
    }
}

pub unsafe extern "C" fn fn_8019610C(state: *mut i32, buttons: u32, trigger: u32) {
    melee_core::ct!("fn_8019610C");
    let mut i: i32 = 1;

    if buttons & (PAD_BUTTON_UP | PAD_STICK_UP) != 0 {
        if *state == 0x12 {
            sfxMove();
            *state = 0x11;
        }
    } else if (buttons & (PAD_BUTTON_DOWN | PAD_STICK_DOWN)) != 0 {
        if *state == 0x11 {
            sfxMove();
            *state = 0x12;
        }
    }

    if (trigger & (PAD_BUTTON_A | PAD_BUTTON_START)) != 0 {
        if *state == 0x11 {
            ext::lbAudioAx_80024030(i);
            if gm_804771C4.match_type == 0 {
                ext::gm_SetNextGameModeStateId(1);
            } else {
                let td = gm_GetTournamentData();
                i = 0;
                let mut src_ptr = td as *mut u8;
                let mut dst_ptr = td as *mut u8;
                let base_val = i;
                while i < (*td).x2E as i32 {
                    *src_ptr.add(0x45) = *src_ptr.add(0x44);
                    *src_ptr.add(0x46) = base_val as u8;
                    if i < (*td).x30 as i32 {
                        core::ptr::write_unaligned(
                            dst_ptr.add(0x4BE) as *mut u16,
                            core::ptr::read_unaligned(src_ptr.add(0x40) as *const u16),
                        );
                        *dst_ptr.add(0x4BD) = *src_ptr.add(0x39);
                        *dst_ptr.add(0x4B9) = *src_ptr.add(0x3A);
                        *dst_ptr.add(0x4B8) = *src_ptr.add(0x37);
                        *dst_ptr.add(0x4BB) = *src_ptr.add(0x3E);
                        *dst_ptr.add(0x4BA) = *src_ptr.add(0x3C);
                    }
                    src_ptr = src_ptr.add(0x12);
                    dst_ptr = dst_ptr.add(0xA);
                    i += 1;
                }
                ext::gm_SetNextGameModeStateId(2);
            }
            ext::gm_801A4B60();
        } else {
            sfxBack();
            *state = lbl_804D6654 as i32;
        }
    } else if (trigger & PAD_BUTTON_B) != 0 {
        sfxBack();
        *state = lbl_804D6654 as i32;
    }
}

/// Tournament Mode per-frame update handler.
pub unsafe extern "C" fn gm_Scene_TouSetup_OnFrame() {
    melee_core::ct!("gm_Scene_TouSetup_OnFrame");
    let tm = gm_GetTournamentData();
    let r30 = fn_8018F674(4);
    let r29 = fn_8018F640(4);

    if ext::mn_8022F218() != 0 {
        sfxBack();
        ext::HSD_SisLib_803A5E70();
        ext::mn_8022F138(1, 8);
        ext::mn_8022F138(0x19, 0x1C);
        ext::mn_8022F138(0x12, 0x15);
        ext::mn_8022F268();
        ext::gm_801A4B60();
        ext::gm_ChangeGameModeAfterCurrentScene(GM_MENU as i32);
        return;
    }

    if (*tm).x20 != 0 {
        return;
    }

    lbl_804799B8.xA = srem(lbl_804799B8.xA as i32 + 1, 11) as u8;

    let cur_option = (*tm).cur_option;
    if cur_option >= 9 && cur_option <= 9 {
        return;
    }
    if cur_option >= 0x13 {
        return;
    }
    (lbl_803D9FD8[cur_option as usize].unwrap())(tm as *mut i32, r30, r29);
}

pub unsafe extern "C" fn gm_Scene_TouSetup_OnEnter(_arg0: *mut c_void) {
    melee_core::ct!("gm_Scene_TouSetup_OnEnter");
    ext::lbAudioAx_80026F2C(0x12);
    ext::lbAudioAx_8002702C(2, 8);
    ext::lbAudioAx_80027168();
    ext::lbAudioAx_80027648();
    lbl_804D6640 = ext::lbArchive_80016DBC(
        b"GmTou1p\0".as_ptr() as *const c_char,
        addr_of_mut!(lbl_804D664C) as *mut c_void,
        &[b"ScGamTour_scene_data\0".as_ptr() as usize as u64, 0],
    );
    lbl_804D6644 = ext::lbArchive_80016DBC(
        b"GmTou2p\0".as_ptr() as *const c_char,
        addr_of_mut!(lbl_804D6650) as *mut c_void,
        &[b"ScGamTour_scene_data\0".as_ptr() as usize as u64, 0],
    );
    lbl_804D6648 = ext::lbArchive_LoadArchive(b"MnExtAll\0".as_ptr() as *const c_char);
    let filename = b"TmBox.dat\0";
    lbl_804D6638 = ext::lbArchive_80016DBC(
        filename.as_ptr() as *const c_char,
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
    ext::HSD_SisLib_803A62A0(0, fn_8018F5F0(), b"SIS_TournamentData\0".as_ptr() as *mut c_char);
    fn_801935B8();
    ext::lbAudioAx_800237A8(0x7539, 0x7F, 0x40);
    ext::lbAudioAx_80023F28(ext::gmMainLib_8015ECB0());
}

/// Scene OnLeave callback for gm_18A5. Frees archive resources.
pub unsafe extern "C" fn gm_Scene_TouSetup_OnExit(_arg: *mut c_void) {
    melee_core::ct!("gm_Scene_TouSetup_OnExit");
    ext::lbArchive_80016EFC(lbl_804D6640);
    ext::lbArchive_80016EFC(lbl_804D6644);
    ext::lbArchive_80016EFC(lbl_804D6648);
    ext::lbArchive_80016EFC(lbl_804D6638);
    (*gm_GetTournamentData()).cur_option = 0x13;
    (*gm_GetTournamentData()).x2C = 1;
    (*gm_GetTournamentData()).x31 = lbl_804799B8.x0;
}

// ---- folded from gm.rs (consolidation steps 21/22) ----
/// gmtou_0.c `fn_801953C8_GetPreviousPosition_fp`.
pub fn fn_801953C8_GetPreviousPosition_fp(offset: i32, position: i32) -> i32 {
    melee_core::ct!("fn_801953C8_GetPreviousPosition_fp");
    position.wrapping_sub(offset)
}

// ---- folded from gm_ptr.rs (consolidation steps 21/22) ----
/// gmtou_0.c `fn_80191240_save_root_jobj`.
pub fn fn_80191240_save_root_jobj(jobj: *mut HSD_JObj_gm_ptr, jobj_copy: &mut *mut HSD_JObj_gm_ptr) {
    melee_core::ct!("fn_80191240_save_root_jobj");
    *jobj_copy = jobj;
}

/// gmtou_0.c `fn_80191FD4_is_selected_fp` (`hud` is an `enum CSSIconHud`).
pub fn fn_80191FD4_is_selected_fp(hud: i32, slot: i32, tm: &TmData_gm_ptr) -> bool {
    melee_core::ct!("fn_80191FD4_is_selected_fp");
    tm.x37[slot as usize].x3 as u32 == hud as u32
}

/// gmtou_0.c `fn_80195CCC_IsUniqueEntry_fp`.
pub fn fn_80195CCC_IsUniqueEntry_fp(tm: &TmData_gm_ptr, count: i32, slot: i32, selected: i32) -> i32 {
    melee_core::ct!("fn_80195CCC_IsUniqueEntry_fp");
    for i in 0..count.max(0) {
        if i != slot && selected == tm.x37[i as usize].x9 as i32 {
            return 0;
        }
    }
    1
}

// ---- folded from gm_r6.rs (consolidation steps 21/22) ----
/// gmtou_0.c `fn_80194658_get_value` (static inline): `*++ptr`.
pub fn fn_80194658_get_value(ptr: &[i32]) -> i32 {
    melee_core::ct!("fn_80194658_get_value");
    ptr[1]
}
