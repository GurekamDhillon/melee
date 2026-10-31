//! Group 6 round 3: the pointer-taking leaf functions against the C, bit for bit.
//! tests/data/vectors_ptr.txt is written by oracle/build_ptr.sh (gen_oracle_ptr.py): each C
//! function cut verbatim from its TU, compiled with contraction on + gwtool's lowering.
//!
//! A line is `name tok... = ret hash`. `s:<hex>` is a scalar (or a pointer the function only
//! stores/compares); `b:<seed>:<size>:<sizeof>` a buffer filled by `fill` (seed 0 = NULL);
//! `f:<buf>:<off>:<size>:<hex>` a little-endian store into a buffer after filling. `hash` is
//! FNV-1a 64 over every non-NULL buffer after the call. The C views a buffer as its own struct
//! type, the Rust as the melee_types record: their native layouts are identical (gen_types.py
//! checks every offset against clang), so equal hashes mean equal field values.
#![allow(non_snake_case)]

use melee_modes::{gm_ptr::*, hud_ptr::*, lb_ptr::*, mn_ptr::*};
use melee_types as mt;
use std::collections::BTreeMap;

fn sm(x: &mut u64) -> u64 {
    *x = x.wrapping_add(0x9E3779B97F4A7C15);
    let mut z = *x;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
}

/// One 4-byte word of buffer fill; the same generator as main.c's `word`. Floats never
/// come out NaN or infinite (bit 30 is cleared in the raw-bits case).
fn word(st: &mut u64) -> u32 {
    let r = sm(st);
    let v = (r >> 32) as u32;
    match r & 7 {
        0 | 1 => 0,
        2 => v % 3 + 1,
        3 => v & 0xFF,
        4 => (((v % 1000001) as i32 - 500000) as f32 / 1000.0).to_bits(),
        5 => (((v % 4001) as i32 - 2000) as f32 / 1000.0).to_bits(),
        _ => v & 0xBFFF_FFFF,
    }
}

struct Buf {
    w: Vec<u64>,
    sz: usize,
    null: bool,
    /// sizeof(*arg) in the C, checked against the Rust type each view uses
    csize: usize,
}

impl Buf {
    fn new(seed: u64, sz: usize, csize: usize) -> Buf {
        let mut w = vec![0u64; sz / 8 + 2];
        if seed != 0 {
            let mut st = seed;
            let bytes = unsafe { std::slice::from_raw_parts_mut(w.as_mut_ptr() as *mut u8, sz) };
            for i in 0..sz / 4 {
                bytes[4 * i..4 * i + 4].copy_from_slice(&word(&mut st).to_le_bytes());
            }
        }
        Buf { w, sz, null: seed == 0, csize }
    }
    fn bytes(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.w.as_mut_ptr() as *mut u8, self.sz) }
    }
}

/// A buffer as the Rust function sees it: pointer, byte size, C sizeof of the pointee.
#[derive(Clone, Copy)]
struct P(*mut u8, usize, usize);

impl P {
    fn check<T>(self) {
        // the Rust record must have the C struct's size (layouts are checked by gen_types.py)
        assert_eq!(std::mem::size_of::<T>(), self.2, "sizeof {} vs C", std::any::type_name::<T>());
    }
    fn r<'a, T>(self) -> &'a mut T {
        self.check::<T>();
        assert!(!self.0.is_null() && std::mem::size_of::<T>() <= self.1);
        unsafe { &mut *(self.0 as *mut T) }
    }
    fn o<'a, T>(self) -> Option<&'a mut T> {
        self.check::<T>();
        if self.0.is_null() { None } else { Some(self.r()) }
    }
    fn s<'a, T>(self) -> &'a mut [T] {
        self.check::<T>();
        assert!(!self.0.is_null());
        unsafe { std::slice::from_raw_parts_mut(self.0 as *mut T, self.1 / std::mem::size_of::<T>()) }
    }
}

fn f(b: u64) -> f32 {
    f32::from_bits(b as u32)
}
fn fb(x: f32) -> u64 {
    x.to_bits() as u64
}
fn i(x: i32) -> u64 {
    x as u32 as u64
}
fn b(x: bool) -> u64 {
    x as u64
}
fn color(v: u64) -> mt::_GXColor {
    let [r, g, b, a] = (v as u32).to_be_bytes();
    mt::_GXColor { r, g, b, a }
}
fn ptr<T>(v: u64) -> *mut T {
    v as usize as *mut T
}

fn eval(name: &str, s: &[u64], p: &[P]) -> u64 {
    let s32 = |k: usize| s[k] as u32 as i32;
    let u8_ = |k: usize| s[k] as u8;
    let s8 = |k: usize| s[k] as u8 as i8;
    match name {
        // ---- gm
        "gmCamera_801A2BF0_get_translate_x" => fb(gmCamera_801A2BF0_get_translate_x(p[0].r())),
        "gmClassic_GetMatchupCount" => i(gmClassic_GetMatchupCount(p[0].s())),
        "fn_801B9060" => { fn_801B9060(p[0].r(), ptr(s[0])); 0 }
        "fn_801BA7AC" => { fn_801BA7AC(p[0].r(), ptr(s[0])); 0 }
        "gmMainLib_AdjustNameTag" => { gmMainLib_AdjustNameTag(p[0].r(), u8_(0)); 0 }
        "gmMainLib_ClearNameTag" => { gmMainLib_ClearNameTag(p[0].r(), u8_(0)); 0 }
        "gmMainLib_8015EF30" => { gmMainLib_8015EF30(p[0].r()); 0 }
        "gm_8017BE8C" => i(gm_8017BE8C(p[0].s())),
        "gm_801A9094_get_entry" => i(gm_801A9094_get_entry(p[0].s(), s32(0))),
        "gmResultFindNth" => i(gmResultFindNth(p[0].s(), s32(0))),
        "matchWasSkipped" => b(matchWasSkipped(p[0].r())),
        "fn_801BA5B4" => { fn_801BA5B4(p[0].r(), ptr(s[0])); 0 }
        "gm_801AB200_SetAppSRT" => { gm_801AB200_SetAppSRT(p[0].r(), p[1].r()); 0 }
        "fn_801B8C5C" => { fn_801B8C5C(p[0].r(), ptr(s[0])); 0 }
        "fn_801B8E50" => { fn_801B8E50(p[0].r(), ptr(s[0])); 0 }
        "gmTournament_SetBracketByes" => { gmTournament_SetBracketByes(p[0].s(), s32(0)); 0 }
        "fn_8018B090_inline4" => i(fn_8018B090_inline4(p[0].r())),
        "fn_8018F62C" => fn_8018F62C(p[0].r()) as u64,
        "fn_80191240_save_root_jobj" => { fn_80191240_save_root_jobj(ptr(s[0]), p[0].r()); 0 }
        "fn_80191FD4_is_selected" => b(fn_80191FD4_is_selected(s32(0), s32(1), p[0].r())),
        "fn_80195CCC_IsUniqueEntry" => i(fn_80195CCC_IsUniqueEntry(p[0].r(), s32(0), s32(1), s32(2))),
        "fn_80196564" => b(fn_80196564(p[0].r())),
        "fn_80196594" => b(fn_80196594(p[0].r())),
        "getPort" => i(getPort(p[0].r(), s32(0))),
        "gmVsMelee_WasAnyPlayerHuman" => b(gmVsMelee_WasAnyPlayerHuman(p[0].r())),
        "fn_80161C90_count_players" => fn_80161C90_count_players(p[0].r()) as u64,
        "is_character_unlocked" => b(is_character_unlocked(u8_(0), p[0].r())),
        "fn_80165418" => i(fn_80165418(p[0].r())),
        "fn_801654A0" => i(fn_801654A0(p[0].r())),
        "fn_80165548" => i(fn_80165548(p[0].r(), s32(0), s32(1))),
        "fn_801656A8" => fn_801656A8(p[0].r(), s[0] as u32) as u64,
        "fn_80166CBC" => fn_80166CBC(p[0].r(), s32(0)) as u64,
        "gm_80166CCC" => { gm_80166CCC(p[0].r(), p[1].r()); 0 }
        "fn_80167194" => i(fn_80167194(p[0].r())),
        "gm_801677C0" => { gm_801677C0(p[0].r()); 0 }
        "gm_80168940" => i(gm_80168940(p[0].r())),
        "get_big_loser" => i(get_big_loser(s32(0), p[0].r())),
        "gm_8017CE34_CountEnemies" => i(gm_8017CE34_CountEnemies(p[0].s())),
        "gm_8017CE34_GetCpuLevel" => gm_8017CE34_GetCpuLevel(p[0].r()) as u64,
        "gm_8017DB58" => { gm_8017DB58(p[0].s()); 0 }
        "gm_8017DB6C" => gm_8017DB6C(p[0].s(), s32(0)) as u64,
        "gm_8017DB78" => gm_8017DB78(p[0].s(), s32(0)) as u64,
        "fn_80181E18_ComputeRemaining100" => i(fn_80181E18_ComputeRemaining100(p[0].r(), s32(0))),
        "gm_80182578_GetIndexFromPointer" => i(gm_80182578_GetIndexFromPointer(p[0].r())),
        "resetText" => { resetText(p[0].r()); 0 }
        "firstState" => firstState(p[0].s(), u8_(0)) as u64,
        "player_standings_inline" => {
            player_standings_inline(p[0].r(), p[1].r(), s32(0), s[1] as u32, s32(2));
            0
        }
        "gm_801B0474_inline" => i(gm_801B0474_inline(p[0].r(), s32(0))),
        "gm_801B05F4" => { gm_801B05F4(p[0].r(), s32(0)); 0 }
        "gm_801B0730" => { gm_801B0730(p[0].r(), p[1].o(), p[2].o(), p[3].o(), p[4].o(), p[5].o()); 0 }
        "gm_801B07B4" => { gm_801B07B4(p[0].r(), s8(0), s8(1), s8(2), u8_(3), u8_(4), u8_(5)); 0 }
        "gm_801B07E8" => { gm_801B07E8(p[0].r(), p[1].o(), p[2].o(), p[3].o(), p[4].o(), p[5].o()); 0 }
        // ---- if, ty
        "un_802FE918_update_x3" => { un_802FE918_update_x3(p[0].r(), p[1].r()); 0 }
        "ifStatus_GetStoredTranslationX" => fb(ifStatus_GetStoredTranslationX(p[0].r(), s32(0))),
        "ifStock_802F8298_step_anim" => { ifStock_802F8298_step_anim(p[0].r()); 0 }
        "ifStock_ClearGObjSlots" => { ifStock_ClearGObjSlots(p[0].s(), s32(0)); 0 }
        "un_802FF884" => b(un_802FF884(ptr(s[0]))),
        "DevText_SetCursorXY" => { DevText_SetCursorXY(p[0].r(), s32(0), s32(1)); 0 }
        "DevText_HideCursor" => { DevText_HideCursor(p[0].r()); 0 }
        "DevText_80302AC0" => { DevText_80302AC0(p[0].r()); 0 }
        "DevText_ShowBackground" => { DevText_ShowBackground(p[0].r()); 0 }
        "DevText_HideBackground" => { DevText_HideBackground(p[0].r()); 0 }
        "DevText_ShowText" => { DevText_ShowText(p[0].r()); 0 }
        "DevText_HideText" => { DevText_HideText(p[0].r()); 0 }
        "DevText_SetScale" => { DevText_SetScale(p[0].r(), f(s[0]), f(s[1])); 0 }
        "DevText_SetXY" => { DevText_SetXY(p[0].r(), s32(0), s32(1)); 0 }
        "DevText_SetBGColor" => {
            let c = DevText_SetBGColor(p[0].r(), color(s[0]));
            u32::from_be_bytes([c.r, c.g, c.b, c.a]) as u64
        }
        "un_80302DF8" => {
            // SAFETY: an Option<extern fn> is a nullable pointer; the value is only stored.
            let cb: mt::soundtest_callback = unsafe { std::mem::transmute::<usize, mt::soundtest_callback>(s[0] as usize) };
            un_80302DF8(p[0].r(), cb);
            0
        }
        "un_80303FD4_first" => i(un_80303FD4_first(p[0].s())),
        "un_80303FD4_count" => i(un_80303FD4_count(p[0].s())),
        "un_80304334" => { un_80304334(p[0].r()); 0 }
        "tyDisplay_SetGridSize" => { tyDisplay_SetGridSize(p[0].r(), p[1].r()); 0 }
        "tyFigupon_StoreDigits" => { tyFigupon_StoreDigits(p[0].s(), s32(0)); 0 }
        "tyFigupon_GetBetCount" => i(tyFigupon_GetBetCount(p[0].r())),
        // ---- lb
        "lbAnim_8001E8F8" => fb(lbAnim_8001E8F8(p[0].o::<FigaTree>().map(|t| &*t))),
        "readCardFileSize" => i(readCardFileSize(p[0].r())),
        "lbColl_80005EBC" => fb(lbColl_80005EBC(p[0].r(), p[1].r(), p[2].r(), p[3].r())),
        "lbColl_80005FC0" => fb(lbColl_80005FC0(p[0].r(), p[1].r(), p[2].r(), p[3].r())),
        "end" => b(end(p[0].r(), p[1].r(), f(s[0]))),
        "lbColl_GetY" => fb(lbColl_GetY(p[0].r())),
        "lbColl_DifferenceY" => fb(lbColl_DifferenceY(p[0].r(), p[1].r())),
        "sqrDistance" => fb(sqrDistance(p[0].r(), p[1].r())),
        "lbColl_80008428" => { lbColl_80008428(p[0].r()); 0 }
        "lbColl_80008434" => { lbColl_80008434(p[0].r()); 0 }
        "lbColl_80008440" => { lbColl_80008440(p[0].r()); 0 }
        "lbColl_800089B8" => { lbColl_800089B8(p[0].r(), ptr(s[0])); 0 }
        "lbColl_80008A5C" => { lbColl_80008A5C(p[0].r()); 0 }
        "lbColl_80008D30" => { lbColl_80008D30(p[0].r(), p[1].r()); 0 }
        "Command_00" => { Command_00(p[0].r()); 0 }
        "Command_06" => { Command_06(p[0].r()); 0 }
        "Command_08" => { Command_08(p[0].r()); 0 }
        "splGetCardinalTangent" => { splGetCardinalTangent(p[0].r(), p[1].s(), f(s[0]), f(s[1])); 0 }
        "splGetBSplineTangent" => { splGetBSplineTangent(p[0].r(), p[1].s(), f(s[0])); 0 }
        "splGetBezierTangent" => { splGetBezierTangent(p[0].r(), p[1].s(), f(s[0])); 0 }
        "lbSnap_ClearText" => { lbSnap_ClearText(p[0].s()); 0 }
        "checkJObjFlags" | "checkJObjFlags_lb_00F9" => b(checkJObjFlags(p[0].r())),
        "lbVector_Mirror" => { lbVector_Mirror(p[0].r(), p[1].r()); 0 }
        "lb_8000CE30" => { lb_8000CE30(p[0].o(), ptr(s[0])); 0 }
        "lb_8000CE40" => { lb_8000CE40(p[0].o(), ptr(s[0])); 0 }
        "lb_800103D8" => b(lb_800103D8(p[0].r(), f(s[0]), f(s[1]), f(s[2]), f(s[3]), f(s[4]))),
        "lb_80013BB0" => b(lb_80013BB0(ptr(s[0]))),
        "lb_80014638" => b(lb_80014638(p[0].r(), p[1].r())),
        // ---- mn
        "loadStickValue" => fb(loadStickValue(p[0].r())),
        "mnCharSel_MexPackRows" => i(mnCharSel_MexPackRows(p[0].s(), s32(0), s32(1), p[1].s())),
        "mnDiagram_TextSetPos" => { mnDiagram_TextSetPos(p[0].r(), f(s[0]), f(s[1]), f(s[2])); 0 }
        "mnDiagram3_GetRowStat" => mnDiagram3_GetRowStat(p[0].r(), u8_(0), u8_(1)) as u64,
        "float_order_helper" => { float_order_helper(p[0].r()); 0 }
        "mnInfo_80251AFC_inline_3" => i(mnInfo_80251AFC_inline_3(p[0].s(), s32(0))),
        "mnInfo_80252720" => { mnInfo_80252720(p[0].r()); 0 }
        "textSize" => { textSize(p[0].r()); 0 }
        "mnInfoBonus_80252ADC_inline" => { mnInfoBonus_80252ADC_inline(p[0].r()); 0 }
        "GetAnimStartFrame" => fb(GetAnimStartFrame(p[0].r())),
        "GetAnimEndFrame" => fb(GetAnimEndFrame(p[0].r())),
        "readNameTerminator" => readNameTerminator(p[0].r()) as u8 as u64,
        "checkStringRest" => b(checkStringRest(p[0].s(), s8(0))),
        "mnName_80238C34_inline" => fb(mnName_80238C34_inline(p[0].r())),
        "mnNameNew_8023BAA8" => i(mnNameNew_8023BAA8(p[0].r(), s32(0), u8_(1))),
        "GlyphVariantCount" => { GlyphVariantCount(s[0] as u16, p[0].r()); 0 }
        "getVolumePosition" => fb(getVolumePosition(u8_(0), p[0].r(), p[1].r())),
        "mnStageSw_CountEnabled" => i(mnStageSw_CountEnabled(p[0].s())),
        "mn_8022F410" => i(mn_8022F410(p[0].r(), p[1].r(), f(s[0]))),
        "mn_8022F470" => i(mn_8022F470(p[0].r(), p[1].r(), s32(0))),
        _ => panic!("no port for {name}"),
    }
}

#[test]
fn group6_pointer_leaf_functions_match_c() {
    let text = include_str!("data/vectors_ptr.txt");
    let mut per: BTreeMap<&str, (usize, usize)> = BTreeMap::new();
    let mut first_bad = Vec::new();
    for line in text.lines().filter(|l| !l.is_empty()) {
        let (lhs, rhs) = line.split_once(" = ").expect(line);
        let (want_r, want_h) = rhs.split_once(' ').expect(line);
        let (want_r, want_h) = (u64::from_str_radix(want_r, 16).unwrap(), u64::from_str_radix(want_h, 16).unwrap());
        let mut it = lhs.split(' ');
        let name = it.next().unwrap();
        // lbcommand.c's Command_xx walk raw 4-byte command words (CmdUnion is 4 bytes); the C oracle's native union is 8 bytes
        // (tests/lbcommand_raw.rs in melee_runtime covers them)
        if name.starts_with("Command_") { continue; }
        let (mut s, mut bufs) = (Vec::new(), Vec::new());
        for t in it {
            let v: Vec<u64> = t[2..].split(':').map(|x| u64::from_str_radix(x, 16).unwrap()).collect();
            match &t[..2] {
                "s:" => s.push(v[0]),
                "b:" => bufs.push(Buf::new(v[0], v[1] as usize, v[2] as usize)),
                "f:" => {
                    let bytes = bufs[v[0] as usize].bytes();
                    for k in 0..v[2] as usize {
                        bytes[v[1] as usize + k] = (v[3] >> (8 * k)) as u8;
                    }
                }
                _ => panic!("{t}"),
            }
        }
        let ps: Vec<P> = bufs
            .iter_mut()
            .map(|b| P(if b.null { std::ptr::null_mut() } else { b.w.as_mut_ptr() as *mut u8 }, b.sz, b.csize))
            .collect();
        let got_r = eval(name, &s, &ps);
        let mut h: u64 = 0xCBF29CE484222325;
        for b in bufs.iter_mut().filter(|b| !b.null) {
            for &x in b.bytes().iter() {
                h ^= x as u64;
                h = h.wrapping_mul(0x100000001B3);
            }
        }
        let e = per.entry(name).or_default();
        e.0 += 1;
        if got_r != want_r || h != want_h {
            e.1 += 1;
            if first_bad.len() < 20 {
                first_bad.push(format!("{line}  (rust {got_r:x} {h:x})"));
            }
        }
    }
    let bad: Vec<_> = per.iter().filter(|(_, v)| v.1 > 0).collect();
    assert!(bad.is_empty(), "mismatches {bad:?}\n{}", first_bad.join("\n"));
    assert_eq!(per.len(), 120, "functions covered (round 3, minus the three Command_xx)");
}
