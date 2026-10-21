//! Round 10 (night 3 round 6): gm leaves over MatchEnd, HSD_SObj chains and BracketEntry,
//! verified bit for bit against the C by `tests/oracle_r10.rs`.
use melee_types::{BracketEntry, HSD_GObj, HSD_SObj, MatchEnd};

/// gm_1601.c `fn_80161004`: the largest `is_big_loser` among active teams (team mode) or
/// non-absent players (pkind != 3), floor 0.
pub fn fn_80161004(match_end: &MatchEnd) -> i32 {
    let mut max = 0i32;
    if match_end.is_teams == 1 {
        for t in &match_end.team_standings {
            if t.active != 0 && t.is_big_loser as i32 > max {
                max = t.is_big_loser as i32;
            }
        }
    } else {
        for p in &match_end.player_standings[..4] {
            if p.pkind != 3 && p.is_big_loser as i32 > max {
                max = p.is_big_loser as i32;
            }
        }
    }
    max
}

/// gm_1601.c `gm_801688AC`: the sum of `xE` over the six players with pkind 0.
pub fn gm_801688AC(arg0: &MatchEnd) -> i32 {
    let mut sum = 0u32;
    for p in &arg0.player_standings {
        if p.pkind == 0 {
            sum = sum.wrapping_add(p.xE as u32);
        }
    }
    sum as i32
}

/// gm_17C0.c `fn_8017DE54` (TARGET_PC branch: the null id is `ChKind_None`, 0x21 either way).
pub fn fn_8017DE54(arg0: u8, arg1: &[u8]) -> i32 {
    const NONE: i32 = melee_types::ChKind_None as i32;
    if arg0 & 0x20 != 0 {
        return 0;
    }
    if arg0 & 0x10 != 0 {
        let count = arg1[..3].iter().filter(|&&k| k as i32 != NONE).count() as i32;
        return 3 - count;
    }
    if arg0 & 2 != 0 {
        return 2;
    }
    0
}

/// gm_1832.c `fn_801859C8`: counts every SObj's `x48` timer down, clearing bit 0 of `x40`
/// when it reaches 0.
///
/// # Safety
/// `gobj.hsd_obj` must be NULL or a valid NULL-terminated HSD_SObj chain.
pub unsafe fn fn_801859C8(gobj: &HSD_GObj) {
    let mut sobj = gobj.hsd_obj as *mut HSD_SObj;
    while let Some(s) = sobj.as_mut() {
        if s.x48 != 0 {
            s.x48 = s.x48.wrapping_sub(1);
            if s.x48 == 0 {
                s.x40 &= !1;
            }
        }
        sobj = s.next;
    }
}

/// gmtoulib.c `gmTournament_SetTripleRightCoords` (static inline).
pub fn gmTournament_SetTripleRightCoords(
    entry: &BracketEntry,
    p3C: &mut i32,
    p44: &mut i32,
    p34: &mut i32,
    p40: &mut i32,
    p48: &mut i32,
    p38: &mut i32,
) {
    let val1 = entry.xC.wrapping_add(entry.x14);
    *p3C = val1;
    *p44 = val1;
    *p34 = val1;
    let val = entry.x10.wrapping_add(entry.x18);
    *p48 = val;
    *p38 = val;
    *p40 = val.wrapping_sub(entry.x18 / 3);
}
