//! Per-function argument recipes: how to call each function on the world so that every pointer
//! it follows is valid and every index stays in range (the C has no bounds checks). A recipe may
//! also adjust the world (e.g. force `cur_option` into the range a handler expects).
use std::ptr::addr_of_mut;

use melee_modes::tu::{gmtou_0, gmtou_1, gmtou_2, gmtoulib, gmtoumode};
use melee_types::*;
use std::vec::Vec;

use super::ext::Rng;
use super::world::{World, N_GOBJ};

#[derive(Clone, Copy, Debug)]
pub enum Arg {
    I(i64),
    F(f32),
    P(usize),
    /// callback chosen by index (Rust passes the index, C the address of the same-named function)
    Cb(u32),
}

impl Arg {
    pub fn word(&self) -> u64 {
        match *self {
            Arg::I(v) => v as u64,
            Arg::F(f) => f.to_bits() as u64,
            Arg::P(p) => p as u64,
            Arg::Cb(k) => k as u64,
        }
    }
}

pub const CB_NAMES: [&str; 3] = ["", "fn_801910E0", "fn_80196DBC"];

fn tm() -> &'static mut TmData {
    unsafe { &mut *addr_of_mut!(gmtoulib::gm_804771C4) }
}

fn pad(rng: &mut Rng) -> i64 {
    const BITS: [u32; 17] = [1, 2, 4, 8, 0x10, 0x20, 0x40, 0x100, 0x200, 0x400, 0x800, 0x1000, 0x10000, 0x20000, 0x40000, 0x80000, 0x600];
    if rng.chance(1, 3) {
        return 0;
    }
    let mut w = 0u32;
    for _ in 0..rng.range(1, 3) {
        w |= BITS[rng.below(17) as usize];
    }
    w as i64
}

fn gobj(w: &mut World, lo: i64, hi: i64) -> Arg {
    let k = w.rng.below(N_GOBJ as u64) as usize;
    let g = w.gobjs[k];
    unsafe { (*g).user_data = w.rng.range(lo, hi) as usize as *mut core::ffi::c_void };
    Arg::P(g as usize)
}

fn jobj(w: &mut World) -> Arg {
    let k = w.rng.below(N_GOBJ as u64) as usize;
    Arg::P(w.jroots[k] as usize)
}

fn int(w: &mut World, lo: i64, hi: i64) -> Arg {
    Arg::I(w.rng.range(lo, hi))
}

fn flt(w: &mut World, lo: f32, hi: f32) -> Arg {
    Arg::F(w.rng.f32r(lo, hi))
}

/// float that is sometimes the 666.0 "leave unchanged" marker
fn flt666(w: &mut World) -> Arg {
    if w.rng.chance(1, 3) { Arg::F(666.0) } else { Arg::F(w.rng.f32r(-30.0, 30.0)) }
}

fn tmp() -> Arg {
    Arg::P(addr_of_mut!(gmtoulib::gm_804771C4) as usize)
}

fn mode_state(w: &mut World, enter: usize, exit: usize) -> Arg {
    let s = w.sp.alloc_ty("GameModeState", 1) as *mut GameModeState;
    unsafe {
        (*s).info.enter_data = enter as *mut core::ffi::c_void;
        (*s).info.exit_data = exit as *mut core::ffi::c_void;
    }
    Arg::P(s as usize)
}

fn raw_fill(w: &mut World, p: *mut u8, n: usize, modulus: u64) {
    for i in 0..n {
        unsafe { *p.add(i) = w.rng.below(modulus) as u8 };
    }
}

fn set_cur(w: &mut World, lo: i64, hi: i64) {
    tm().cur_option = w.rng.range(lo, hi) as i32;
}

fn some_cur(w: &mut World, set: &[i32]) {
    tm().cur_option = set[w.rng.below(set.len() as u64) as usize];
}

pub fn build_args(name: &str, spell: &[&str], w: &mut World) -> Option<Vec<Arg>> {
    let t = tm();
    let cur = w.cur_entry as i64;
    // entrants spread so the bye / odd-bracket branches are exercised
    if w.rng.chance(1, 3) {
        t.entrants = [1, 3, 5, 7][w.rng.below(4) as usize];
    }
    // entries need a non-zero divisor (the C divides by x3 when x3 != 0, and by constants only when x4 != 0)
    unsafe {
        for e in (*addr_of_mut!(gmtoulib::lbl_80473AB8)).iter_mut() {
            if e.x3 == 0 && e.x4 == 0 {
                e.x4 = 1;
            }
            // the winner's destination (entry x5, slot x6) is a real bracket slot
            e.x5 = w.rng.below(64) as u8;
            e.x6 = w.rng.below(4) as u8;
            // and one slot is the current winner (x4C == 0)
            let wi = w.rng.below(4) as usize;
            e.slots[wi].x4C = 0;
            // every bracket entry keeps one slot that is still in (x4E == 3: eliminated)
            if e.slots[0].x4E == 3 {
                e.slots[0].x4E = w.rng.below(3) as u8;
            }
        }
        // icon_model_map has 24 entries, indexed by the entrant's character
        for e in (*addr_of_mut!(gmtoulib::gm_804771C4)).x4B8.iter_mut() {
            e.x1 = w.rng.below(24) as u8;
        }
        if matches!(name, "fn_80199AF0" | "fn_8019A158" | "fn_8019AF50" | "fn_8019B860" | "gm_Scene_TouBracket_OnFrame") {
            // an absent player (pkind 3) can never be the one the scene picks as the slot
            for p in (*addr_of_mut!(gmtoulib::gm_80477738)).player_standings.iter_mut() {
                if p.pkind == 3 {
                    p.pkind = w.rng.below(3) as u8;
                }
            }
        }
        // the bracket scenes look for a standing that is neither 'not playing' nor a small loser (else slot -1)
        let me = &mut *addr_of_mut!(gmtoulib::gm_80477738);
        for k in 0..4 {
            if k == 0 || w.rng.chance(1, 2) {
                me.player_standings[k].pkind = w.rng.below(3) as u8;
                me.player_standings[k].is_small_loser = 0;
            }
        }
        // fn_8018F410 needs a few unlocked characters
        let x72 = &mut (*addr_of_mut!(gmtoulib::lbl_803D9D20)).x72;
        for k in 0..8 {
            x72[k] = 1;
        }
    }
    let a = match name {
        // ------------------------------------------------------------------ gmtitle
        "gmTitle_801A12C4" | "gmTitle_801A165C" | "gmTitle_801A185C" | "gmTitle_801A1944" | "gmTitle_801A19AC"
        | "gmTitle_801A1A3C" | "gmTitle_801A1AC0" | "fn_801A1498_inline" | "gm_Scene_Title_OnFrame" => vec![],
        "gmTitle_801A146C" | "fn_801A1498" | "gmTitle_801A1630" | "gmTitle_801A1A18" => vec![gobj(w, 0, 3)],
        "gmTitle_801A1814" | "gmTitle_801A18D4" => vec![gobj(w, 0, 3), int(w, 0, 3)],
        "gmTitle_801A1D38" => {
            let src = w.sp.alloc_raw(64);
            let n = w.rng.range(0, 24) as usize;
            for i in 0..n {
                unsafe { *src.add(i) = [b' ', b'0', b'9', b'A', b'Z', b'a', b'z', b'-', b':', 0x7F][w.rng.below(10) as usize] };
            }
            let dst = w.sp.alloc_raw(160);
            vec![Arg::P(src as usize), Arg::P(dst as usize)]
        }
        "gm_Scene_Title_OnEnter" => vec![Arg::P(0)],
        // ------------------------------------------------------------------ gmtitlemode / toy
        "gmTitleMode_OnEnter" | "gm_801BED14" => vec![mode_state(w, 0, 0)],
        "onExit" => {
            let e = w.sp.alloc_raw(16);
            let b = pad(&mut w.rng) | if w.rng.chance(1, 2) { 0 } else { w.rng.range(0, 0xFFFFF) };
            unsafe { *(e as *mut i32) = b as i32 };
            vec![mode_state(w, 0, e as usize)]
        }
        // ------------------------------------------------------------------ gmtoumode
        "gm_801B1724" => vec![mode_state(w, 0, 0)],
        "gm_801B174C" => {
            let s = w.sp.alloc_ty("SSSData", 1);
            raw_fill(w, s, std::mem::size_of::<SSSData>(), 4);
            vec![mode_state(w, s as usize, 0)]
        }
        "gm_801B1788" => {
            let s = w.sp.alloc_ty("SSSData", 1) as *mut SSSData;
            unsafe {
                raw_fill(w, s as *mut u8, std::mem::size_of::<SSSData>(), 4);
                (*s).start_game = w.rng.below(2) as u8;
            }
            vec![mode_state(w, 0, s as usize)]
        }
        "gm_801905F0" => {
            let s = w.sp.alloc_ty("StartMeleeData", 1) as *mut StartMeleeData;
            unsafe { raw_fill(w, s as *mut u8, std::mem::size_of::<StartMeleeData>(), 5) };
            let t = tm();
            t.x30 = w.rng.range(0, 4) as u8;
            for e in t.x4B8.iter_mut() {
                e.x2 = w.rng.below(2) as u8;
                e.x0 = w.rng.below(3) as u8;
                e.x6 = w.rng.range(0, 200) as u16;
                e.x5 = w.rng.range(1, 9) as u8;
            }
            vec![Arg::P(s as usize)]
        }
        "gm_801B1810" | "gm_801B18D4" => {
            let s = w.sp.alloc_ty("StartMeleeData", 1) as *mut StartMeleeData;
            unsafe { std::ptr::write_bytes(s as *mut u8, 0, std::mem::size_of::<StartMeleeData>()) };
            vec![mode_state(w, s as usize, 0)]
        }
        "gm_801B1834" | "gm_801B1A2C" => {
            let s = w.sp.alloc_ty("MatchExitInfo", 1) as *mut MatchExitInfo;
            unsafe {
                raw_fill(w, s as *mut u8, std::mem::size_of::<MatchExitInfo>(), 6);
                (*s).match_end.n_winners = w.rng.range(0, 4) as u8;
                (*s).match_end.is_teams = w.rng.below(2) as u8;
                (*s).match_end.frame_count = w.rng.below(60 * 400) as u32;
                for p in (*s).match_end.player_standings.iter_mut() {
                    p.pkind = w.rng.below(4) as u8;
                }
            }
            vec![mode_state(w, 0, s as usize)]
        }
        "gm_801B1A84" | "gm_801B1AD4" => {
            let s = w.sp.alloc_ty("ResultsMatchInfo", 1) as *mut ResultsMatchInfo;
            unsafe {
                raw_fill(w, s as *mut u8, std::mem::size_of::<ResultsMatchInfo>(), 6);
                for p in (*s).match_end.player_standings.iter_mut() {
                    p.pkind = w.rng.below(4) as u8;
                }
            }
            vec![mode_state(w, s as usize, 0)]
        }
        // ------------------------------------------------------------------ gmtoulib
        "fn_8018A514" => vec![int(w, 1, 31), flt(w, 0.5, 6.0)],
        "fn_8018A970" => vec![int(w, 1, 31)],
        "fn_8018AA74" => vec![jobj(w), int(w, 0, 63), int(w, 0, 3)],
        "fn_8018B090" => {
            some_cur(w, &[20, 21, 22, 23, 24, 25, 32, 33, 34, 35, 36, 37, 38, 99]);
            vec![gobj(w, 0, 3)]
        }
        "fn_8018C8D4" | "fn_8018D50C" | "fn_8018DC18" | "fn_8018DF68" => {
            let k = w.rng.below(64) as usize;
            let e = unsafe { (addr_of_mut!(gmtoulib::lbl_80473AB8) as *mut BracketEntry).add(k) };
            unsafe {
                if w.rng.chance(1, 2) {
                    (*e).x4 = w.rng.below(4) as u8;
                }
            }
            vec![Arg::P(e as usize), int(w, -60, 640), int(w, -60, 480), int(w, 8, 600), int(w, -300, 300), int(w, -300, 300)]
        }
        "fn_8018E46C" => {
            let g = w.gobjs[w.rng.below(N_GOBJ as u64) as usize];
            let k = w.rng.below(64) as usize;
            let e = unsafe { (addr_of_mut!(gmtoulib::lbl_80473AB8) as *mut BracketEntry).add(k) };
            unsafe {
                (*g).user_data = e as *mut core::ffi::c_void;
                if w.rng.chance(1, 2) {
                    (*e).x3 = w.rng.below(4) as u8;
                }
            }
            vec![Arg::P(g as usize), int(w, 0, 3)]
        }
        "fn_8018E618" => vec![int(w, 1, 31), flt(w, 0.0, 9.0), int(w, 0, 1)],
        "fn_8018E85C" => {
            if w.rng.chance(1, 2) {
                tm().cur_option = w.rng.range(0, 0x30) as i32;
            }
            let sd = unsafe { gmtou_0::lbl_804D664C };
            let m = unsafe { *(*sd).models.add(w.rng.below(16) as usize) };
            vec![Arg::P(m as usize), int(w, 0, 1)]
        }
        "fn_8018EC48" | "fn_8018EC7C" | "fn_8018F410" | "fn_8018F4A0" | "fn_8018F5F0" | "gm_GetTournamentData"
        | "fn_8018F74C" | "fn_8018F808" | "fn_8018F888" | "fn_8018FA24" | "fn_801904D0" => {
            if name == "fn_8018FA24" {
                tm().x4B8[0].x0 = tm().x4B8[0].x0;
            }
            vec![]
        }
        "fn_8018ECA8" => {
            let ids = [w.rng.range(0, 40), 0x320 + w.rng.range(0, 99), 0x384 + w.rng.range(0, 99), 0x3E7, w.rng.range(0, 0x500)];
            let id = ids[w.rng.below(5) as usize];
            let nt = [0i64, 1, 0xFF, 2][w.rng.below(4) as usize];
            vec![Arg::I(id), Arg::I(nt), int(w, 0, 5), flt(w, 0.0, 600.0), flt(w, 0.0, 400.0), int(w, 0, 5)]
        }
        "fn_8018F00C" => {
            let dst = w.sp.alloc_raw(64);
            let ids = [w.rng.range(0, 40), 0x320 + w.rng.range(0, 99), 0x384 + w.rng.range(0, 99), 0x3E7, w.rng.range(0, 0x400)];
            // GetNameText is NULL for id % 7 == 3 (the callee stub); the C strcpy()s it unchecked
            let id = ids[w.rng.below(5) as usize];
            let id = if id < 0x320 && (id & 0xFF) % 7 == 3 { id + 1 } else { id };
            vec![Arg::P(dst as usize), Arg::I(id)]
        }
        "gm_8018F1B0" => {
            if w.rng.chance(1, 2) {
                let t = tm();
                t.match_type = w.rng.range(0, 2) as i32;
                t.hmn_cpu_count = w.rng.range(1, 3) as i32;
                t.x30 = w.rng.range(2, 4) as u8;
            }
            vec![Arg::P(addr_of_mut!(gmtoulib::gm_80477738) as usize)]
        }
        "fn_8018F310" => vec![int(w, 0, 30)],
        "fn_8018F3BC" => vec![int(w, 0, 24)],
        "fn_8018F508" => {
            let o = w.sp.alloc_raw(16);
            vec![if w.rng.chance(1, 4) { Arg::P(0) } else { Arg::P(o as usize) }]
        }
        "fn_8018F640" | "fn_8018F674" | "fn_8018F6A8" => vec![int(w, 0, 8)],
        "fn_8018FBE0" => vec![int(w, 0, 0x30), int(w, 0, 1), int(w, 0, 1), int(w, 0, 9), int(w, 0, 9), int(w, 0, 0x3E7), int(w, 0, 3)],
        "fn_8018FDC4" | "fn_8018FF9C" => vec![jobj(w), flt666(w), flt666(w), flt666(w)],
        "fn_80190174" | "fn_801901F8" => {
            let d = w.sp.alloc_ty("HSD_CObjDesc", 1);
            vec![Arg::P(d as usize)]
        }
        "fn_8019027C" => {
            let arr = w.sp.alloc(8, super::space::PTRS, 2) as *mut *mut LightList;
            unsafe { *arr = w.sp.alloc_ty("LightList", 1) as *mut LightList };
            vec![Arg::P(arr as usize)]
        }
        "fn_801902F0" => vec![int(w, 0, 5)],
        "fn_8019035C" => {
            let sd = unsafe { gmtou_0::lbl_804D664C };
            let m = unsafe { *(*sd).models.add(w.rng.below(16) as usize) };
            vec![int(w, 0, 1), Arg::P(m as usize), int(w, 0, 3), int(w, 0, 0x30), int(w, 0, 5), int(w, 0, 1), Arg::Cb(w.rng.below(3) as u32), flt(w, 0.0, 90.0)]
        }
        "fn_8019044C" => vec![jobj(w), flt(w, 0.0, 200.0)],
        "fn_80190480" => vec![if w.rng.chance(1, 3) { Arg::F(0.0) } else { flt(w, 1.0, 140.0) }],
        "fn_80190520" => {
            let z = |w: &mut World| if w.rng.chance(1, 3) { Arg::F(0.0) } else { Arg::F(w.rng.f32r(-300.0, 300.0)) };
            vec![z(w), z(w), z(w)]
        }
        // ------------------------------------------------------------------ gmtou_0
        "fn_80190ABC" => vec![int(w, 0, 6)],
        "gm_80190EA4" | "fn_80192938" | "fn_80192BB0" | "fn_80192E6C" | "fn_80193230" | "fn_80193308" | "fn_801935B8"
        | "gm_Scene_TouSetup_OnFrame" | "gm_Scene_TouSetup_OnExit" | "gm_Scene_TouSetup_OnEnter" => {
            if name == "gm_Scene_TouSetup_OnFrame" {
                tm().cur_option = w.rng.range(0, 0x14) as i32;
                tm().x20 = w.rng.below(2) as i32;
                // the dispatch table has a hole at 9 and nothing past 0x12
                if tm().cur_option >= 6 && tm().cur_option <= 8 {
                    tm().cur_option = 6 + w.rng.below(3) as i32;
                }
            }
            if name == "gm_Scene_TouSetup_OnEnter" || name == "gm_Scene_TouSetup_OnExit" {
                return Some(vec![Arg::P(0)]);
            }
            vec![]
        }
        "gm_80190FE4" => vec![if w.rng.chance(1, 2) { Arg::I(0x78) } else { int(w, 0, 0x3E7) }],
        "fn_801910E0" | "fn_80191154" | "fn_8019175C" | "fn_801918F0" | "fn_80191A54" | "fn_80191CA4" | "fn_8019237C"
        | "fn_8019249C" | "fn_80192690" | "fn_80192758" => vec![gobj(w, 0, 5)],
        "fn_80191B5C" => vec![gobj(w, 0, 5)],
        "fn_80191240" | "fn_801913BC" | "fn_80191678" => vec![gobj(w, 0, 5)],
        "fn_80191D38" | "fn_80191E9C" => vec![gobj(w, 0, 63)],
        "fn_80191FD4" => vec![gobj(w, 0, 25)],
        "fn_801937C4" | "fn_80193B58" | "fn_80193FCC" | "fn_80194658" | "fn_801949B4" | "fn_80194BC4" | "fn_80194D84"
        | "fn_80194F30" | "fn_801953C8" | "fn_80195AF0" | "fn_80195CCC" | "fn_8019610C" => {
            let opts: &[i32] = match name {
                "fn_801937C4" => &[0],
                "fn_80193B58" => &[1],
                "fn_80193FCC" => &[2],
                "fn_80194658" => &[3],
                "fn_801949B4" => &[4],
                "fn_80194BC4" => &[5],
                "fn_80194D84" => &[6, 7, 8],
                "fn_80194F30" => &[10, 12, 16],
                "fn_801953C8" => &[11],
                "fn_80195AF0" => &[13, 14],
                "fn_80195CCC" => &[15],
                _ => &[17, 18],
            };
            some_cur(w, opts);
            // settings handlers keep their values inside the table's ranges
            let t = tm();
            if opts[0] <= 6 {
                t.combatants = w.rng.range(2, 6) as i32;
                t.entrants = w.rng.range(1, 31) as i32;
                t.hmn_cpu_count = w.rng.range(1, 4) as i32;
                t.stage_selection_type = w.rng.range(0, 3) as i32;
                t.cpu_level = w.rng.range(0, 9) as i32;
            }
            // x37[idx] reached through state.x2 + state.x3 stays in range (world already ensures < 64)
            let b = pad(&mut w.rng);
            let tr = pad(&mut w.rng);
            vec![tmp(), Arg::I(b), Arg::I(tr)]
        }
        // ------------------------------------------------------------------ gmtou_1
        "fn_80196510" | "fn_8019655C" | "fn_801965C4" | "fn_80196CF8" | "fn_80198BA0" | "fn_80198C60" | "fn_80198D18"
        | "fn_80198EBC" | "fn_80199AF0" | "fn_8019A158" | "gm_8019A828" | "gm_Scene_TouBracket_OnFrame"
        | "gm_Scene_TouBracket_OnEnter" | "gm_Scene_TouBracket_OnExit" => {
            match name {
                "gm_Scene_TouBracket_OnFrame" => some_cur(w, &[0x13, 0x14, 0x15, 0x17, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F, 0x20, 0x22, 0x26, 0x27]),
                "fn_80199AF0" | "fn_8019A158" => {
                    if name == "fn_8019A158" {
                        t.x2D = w.rng.below(2) as u8;
                    }
                    for e in unsafe { (*addr_of_mut!(gmtoulib::lbl_80473AB8)).iter_mut() } {
                        for s in e.slots.iter_mut() {
                            s.x51 = w.rng.range(1, 9) as u8;
                        }
                    }
                }
                "gm_Scene_TouBracket_OnEnter" | "gm_Scene_TouBracket_OnExit" => return Some(vec![Arg::P(0)]),
                _ => {}
            }
            vec![]
        }
        "fn_80196564" | "fn_80196594" => vec![tmp()],
        "fn_80196684" => vec![Arg::I(cur)],
        "fn_801967E0" => vec![Arg::I(cur)],
        "fn_80196DBC" | "fn_80196E30" | "fn_80196EEC" | "fn_801976D4" | "fn_80197D4C" | "fn_801985D4" | "fn_80198824" => {
            if matches!(name, "fn_801985D4" | "fn_80198824") {
                some_cur(w, &[0x1E, 0x1F, 0x20, 0x25, 0x27, 0x28]);
            } else if name == "fn_80196EEC" {
                some_cur(w, &[0x13, 0x14, 0x15, 0x1A, 0x1B]);
            } else {
                some_cur(w, &[0x13, 0x14, 0x19, 0x1A, 0x1B, 0x1C, 0x1E, 0x1F, 0x23]);
            }
            vec![gobj(w, 0, 3)]
        }
        "fn_80196FFC" | "fn_801973F8" | "fn_801975C8" | "fn_801977AC" | "fn_80197AF0" | "fn_80197E18" | "fn_80197FD8"
        | "fn_801981A0" | "fn_801983E4" => {
            some_cur(w, &[0x1B, 0x1C, 0x1D, 0x1E, 0x1B, 0x1C, 0x13]);
            vec![gobj(w, 0, 3)]
        }
        "fn_80198584" => {
            let r = w.sp.alloc_ty("ResultsData", 1) as *mut ResultsData;
            unsafe { (*r).x28 = w.jroots[w.rng.below(N_GOBJ as u64) as usize] };
            some_cur(w, &[0x1D, 0x1C, 0x1E, 0x13]);
            vec![Arg::P(r as usize)]
        }
        "fn_8019A71C" => {
            some_cur(w, &[0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B]);
            unsafe { (*addr_of_mut!(gmtou_1::lbl_804799D8)).x0 = w.rng.range(0, 0x70) as u32 };
            vec![tmp(), Arg::I(0), Arg::I(0)]
        }
        "fn_8019A86C" => {
            some_cur(w, &[0x1B, 0x1C, 0x1D, 0x1E, 0x1D, 0x1C]);
            unsafe { (*addr_of_mut!(gmtou_1::lbl_804799D8)).x0 = w.rng.range(0, 0x140) as u32 };
            let tr = pad(&mut w.rng) | if w.rng.chance(1, 4) { 0x600 } else { 0 };
            vec![tmp(), Arg::I(pad(&mut w.rng)), Arg::I(tr)]
        }
        "fn_8019AF50" => {
            some_cur(w, &[0x1F, 0x20, 0x26, 0x27, 0x27]);
            unsafe { (*addr_of_mut!(gmtou_1::lbl_804799D8)).x0 = w.rng.range(0, 0x1D00) as u32 };
            t.x33 = w.rng.below(8) as u8;
            vec![tmp(), Arg::I(0), Arg::I(0)]
        }
        "fn_8019B458" => {
            t.entrants = w.rng.range(0, 31) as i32;
            vec![tmp()]
        }
        "fn_8019B81C" | "fn_8019BA04" => {
            let s = w.sp.alloc_raw(16);
            vec![Arg::P(s as usize)]
        }
        "fn_8019B860" => vec![tmp()],
        // ------------------------------------------------------------------ gmtou_2
        "fn_8019BA08" => vec![int(w, 0, 3), jobj(w)],
        "fn_8019BF18" | "fn_8019BF8C" | "fn_8019CBFC" | "fn_8019C6AC" | "fn_8019CFA4" => {
            if name == "fn_8019BF8C" {
                tm().cur_option = w.rng.range(0x10, 0x28) as i32;
            }
            vec![gobj(w, 0, 3)]
        }
        "fn_8019C048" | "fn_8019C3EC" | "fn_8019C570" | "fn_8019C744" | "fn_8019CA38" | "fn_8019CC74" | "fn_8019D074" => {
            vec![gobj(w, 0, 3)]
        }
        "fn_8019CDBC" => {
            t.x2E = w.rng.range(1, 3) as u8;
            let hi = t.x2E as i64 - 1;
            vec![gobj(w, 0, hi)]
        }
        "fn_8019D1BC" | "fn_8019DD60" | "gm_Scene_TouAlt_OnEnter" => {
            t.x2E = w.rng.range(1, 3) as u8;
            if name == "gm_Scene_TouAlt_OnEnter" {
                return Some(vec![Arg::P(0)]);
            }
            vec![]
        }
        "gm_Scene_TouAlt_OnFrame" => {
            t.x2E = w.rng.range(1, 3) as u8;
            vec![]
        }
        "gm_8019ECAC_OnEnter_inline" | "gm_Scene_TouAlt_OnExit" => {
            if name == "gm_Scene_TouAlt_OnExit" {
                return Some(vec![Arg::P(0)]);
            }
            vec![]
        }
        "gm_8019E634" => {
            t.x2E = w.rng.range(4, 12) as u8;
            t.x30 = w.rng.range(2, 4) as u8;
            t.hmn_cpu_count = w.rng.range(1, t.x30 as i64) as i32;
            vec![]
        }
        "fn_8019EE80" | "fn_8019EF08" => {
            let v = w.sp.alloc_ty("TmVsData", 1) as *mut TmVsData;
            unsafe {
                (*v).stkind = w.rng.below(30) as _;
                for i in 0..4 {
                    (*v).slot_type[i] = w.rng.below(4) as _;
                    (*v).char_id[i] = w.rng.below(26) as _;
                    (*v).color[i] = w.rng.below(6) as _;
                }
            }
            vec![Arg::P(v as usize)]
        }
        _ => {
            let _ = spell;
            return None;
        }
    };
    Some(a)
}
