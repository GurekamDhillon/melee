//! Group 6 round 1: every ported leaf function against the C, bit for bit.
//! tests/data/vectors.txt is written by oracle/build.sh: the C functions cut verbatim from their
//! game TUs, compiled with contraction on + gwtool's lowering (melee_runtime/oracle/lower.py).
//! Each line: `name a0 a1 ... = r`, every value the raw bits of its C type in hex.
use melee_core::vec::Vec3;
use melee_modes::{gm::*, hud::*, lb::*, mn::*, GXColor};
use std::collections::BTreeMap;

fn f(b: u64) -> f32 {
    f32::from_bits(b as u32)
}
fn fb(x: f32) -> u64 {
    x.to_bits() as u64
}
fn b(x: bool) -> u64 {
    x as u64
}
fn i(x: i32) -> u64 {
    x as u32 as u64
}

fn eval(name: &str, a: &[u64]) -> u64 {
    let s32 = |k: usize| a[k] as u32 as i32;
    let u32_ = |k: usize| a[k] as u32;
    let u8_ = |k: usize| a[k] as u8;
    let s8 = |k: usize| a[k] as u8 as i8;
    match name {
        "gm_GetNextColor" => gm_GetNextColor(u8_(0)) as u64,
        "bitset_mask" => bitset_mask(u32_(0)) as u64,
        "bitset64_mask" => bitset64_mask(u32_(0)) as u64,
        "selkind_mask" => i(selkind_mask(u8_(0))),
        "fe_np_zero" => i(fe_np_zero()),
        "fe_rgba" => fe_rgba(u8_(0), u8_(1), u8_(2), u8_(3)).to_u32() as u64,
        "fe_ease" => fb(fe_ease(f(a[0]))),
        "fe_row_y" => fb(fe_row_y(s32(0))),
        "gm_8017BE84" => i(gm_8017BE84(u32_(0))),
        "gm_WasMatchCanceled" => b(gm_WasMatchCanceled(u8_(0))),
        "gmRst_IsMex" => b(gmRst_IsMex(s32(0))),
        "fn_8018F3D0" => i(fn_8018F3D0(s32(0))),
        "fn_8018F6DC" => i(fn_8018F6DC(s32(0))),
        "fn_8018F6FC" => i(fn_8018F6FC(s32(0))),
        "fn_8018F71C" => fb(fn_8018F71C(s32(0), s32(1))),
        "fn_8018FA24_inline0" => i(fn_8018FA24_inline0(s32(0))),
        "gm_801905F0_inline0" => i(gm_801905F0_inline0(s32(0))),
        "fn_801953C8_GetPreviousPosition" => i(fn_801953C8_GetPreviousPosition(s32(0), s32(1))),
        "direction" => fb(direction(f(a[0]))),
        "gm_801601C4" => i(gm_801601C4(s8(0))),
        "gm_80160244" => i(gm_80160244(s8(0))),
        "fn_80160840" => fn_80160840(u8_(0)) as u64,
        "gm_80160854" => gm_80160854(u8_(0), u8_(1), u8_(2), u8_(3)) as u64,
        "fn_801653E8" => fb(fn_801653E8(u8_(0))),
        "fn_8016588C_clamp" => i(fn_8016588C_clamp(s32(0))),
        "fn_801724C8" => b(fn_801724C8()),
        "fn_8017AD04" => i(fn_8017AD04(s32(0), s32(1))),
        "fn_8017AD28" => i(fn_8017AD28(s32(0))),
        "fn_8017AD48" => i(fn_8017AD48(s32(0))),
        "fn_8017AD78" => i(fn_8017AD78(s32(0))),
        "BossHook_Kind" => i(BossHook_Kind(s32(0))),
        "gm_Is1PMode" => b(gm_Is1PMode(u8_(0))),
        "inlineA1" => fb(inlineA1(f(a[0]))),
        "ifStock_802F89F8_count" => i(ifStock_802F89F8_count(s32(0))),
        "ifStock_802F89F8_divisor" => i(ifStock_802F89F8_divisor(s32(0))),
        "ifStock_802F89F8_digit" => i(ifStock_802F89F8_digit(s32(0), s32(1))),
        "fn_802FFCC8" => b(fn_802FFCC8(s32(0))),
        "DevText_Clamp" => i(DevText_Clamp(s32(0), s32(1))),
        "adjust" => adjust(GXColor::from_u32(u32_(0))).to_u32() as u64,
        "calcPan" => i(calcPan(s32(0), s32(1), s32(2), s32(3))),
        "convertSdkResult" => i(convertSdkResult(s32(0))),
        "isSmall" => b(isSmall(f(a[0]))),
        "same" => i(same(s32(0), s32(1))),
        "RGB565_TO_RGB5A3" => RGB565_TO_RGB5A3(a[0] as u16) as u64,
        "lbSnap_8001E204" => i(lbSnap_8001E204()),
        "lbSnap_8001E210" => i(lbSnap_8001E210()),
        "lbTime_8000AEC8" => lbTime_8000AEC8(u32_(0), u32_(1)) as u64,
        "lbTime_8000AEE4" => lbTime_8000AEE4(u32_(0), s32(1)) as u64,
        "lbTime_8000AF24" => lbTime_8000AF24(u32_(0), s32(1)) as u64,
        "lbTime_8000AF74" => lbTime_8000AF74(u32_(0), s32(1)) as u64,
        "expf" => fb(expf(f(a[0]))),
        "powi" => i(powi(s32(0), s32(1))),
        "approximatelyZeroVec3" => b(approximatelyZeroVec3(Vec3 { x: f(a[0]), y: f(a[1]), z: f(a[2]) })),
        "absf" => fb(absf(f(a[0]))),
        "toMeters" => fb(toMeters(f(a[0]))),
        "toFeet" => fb(toFeet(f(a[0]))),
        "equalU8" => b(equalU8(u8_(0), u8_(1))),
        "getIconOffset" => i(getIconOffset(u32_(0))),
        "inline_is_row_time" => b(inline_is_row_time(s32(0))),
        "inline_is_row_char" => b(inline_is_row_char(s32(0))),
        "mnDiagram2_IsTimeStat" => b(mnDiagram2_IsTimeStat(u8_(0))),
        "mnDiagram2_IsDistanceStat" => b(mnDiagram2_IsDistanceStat(u8_(0))),
        "mnDiagram2_IsPercentageStat" => b(mnDiagram2_IsPercentageStat(u8_(0))),
        "mnDiagram2_IsIconOnlyStat" => b(mnDiagram2_IsIconOnlyStat(u8_(0))),
        "GetSelectionFrameOffset" => i(GetSelectionFrameOffset(s32(0))),
        "mn_8022C010" => i(mn_8022C010(s32(0), s32(1))),
        "signedCharactersEqual" => b(signedCharactersEqual(s8(0), u8_(1))),
        "unsignedCharacter" => unsignedCharacter(s32(0)) as u64,
        "mnVibration_GetPreviousCursorRow" => i(mnVibration_GetPreviousCursorRow(s32(0))),
        "mnVibration_GetNextCursorRow" => i(mnVibration_GetNextCursorRow(s32(0))),
        "mnVibration_GetPadIndex" => mnVibration_GetPadIndex(s32(0)) as u64,
        "_tyDisplay_80318CB4_calc_dist_sq" => fb(_tyDisplay_80318CB4_calc_dist_sq(f(a[0]), f(a[1]))),
        _ => panic!("no port for {name}"),
    }
}

const F32_RET: &[&str] = &["fe_ease", "fe_row_y", "fn_8018F71C", "direction", "fn_801653E8", "inlineA1",
    "expf", "absf", "toMeters", "toFeet", "_tyDisplay_80318CB4_calc_dist_sq"];

#[test]
fn group6_leaf_functions_match_c() {
    let text = include_str!("data/vectors.txt");
    let mut per: BTreeMap<&str, (usize, usize)> = BTreeMap::new();
    let mut first_bad = Vec::new();
    for line in text.lines().filter(|l| !l.is_empty()) {
        let (lhs, rhs) = line.split_once(" = ").expect(line);
        let mut it = lhs.split(' ');
        let name = it.next().unwrap();
        let args: Vec<u64> = it.map(|t| u64::from_str_radix(t, 16).unwrap()).collect();
        let want = u64::from_str_radix(rhs, 16).unwrap();
        let got = eval(name, &args);
        // NaN sign/payload is host-codegen dependent (the native C picked a different mulsd
        // operand order than rustc for fe_ease(NaN)); neither is the Gekko's. Any NaN == any NaN
        // for float results; every other bit must match.
        let got = if F32_RET.contains(&name) && f(got).is_nan() && f(want).is_nan() { want } else { got };
        let e = per.entry(name).or_default();
        e.0 += 1;
        if got != want {
            e.1 += 1;
            if first_bad.len() < 20 {
                first_bad.push(format!("{line}  (rust {got:x})"));
            }
        }
    }
    let bad: Vec<_> = per.iter().filter(|(_, v)| v.1 > 0).collect();
    assert!(bad.is_empty(), "mismatches {bad:?}\n{}", first_bad.join("\n"));
    assert_eq!(per.len(), 72, "functions covered");
}
