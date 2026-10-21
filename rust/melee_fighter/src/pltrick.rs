//! Hand port, round 7: the attack-statistics half of src/melee/pl/pltrick.c (7 of its 10
//! definitions): pl_80037B2C, 80037BC0, 80037C60, 80037DF4, 80037ECC, 800384DC, 80038628 plus the
//! table `pl_803BCE70`. Not ported: fn_80037F00 / pl_80038144 (hit bookkeeping; need
//! ifMagnify, gm and the rest of plbonuslib).
//!
//! Fused sites (`tools/rust-port/fma_sites.py src/melee/pl/pltrick.c`): none in the ported set.
//! `union Struct2070` int arguments are taken as the `Struct2070` they pun (see `ft0892`).
//! Checked against the C (cut verbatim, shipping contraction) by tests/ft0892_ref.rs.

use crate::ft0892::G;
use crate::hooks as h;
use crate::stale::action_stats;
use crate::types::*;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

pub static PL_803BCE70: [i32; 16] = [-1, 0x2F, 0x30, 0x31, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1];

/// `pl_80037B2C`. The C has no return for `idx >= 100` (undefined); this returns 0 there.
pub fn pl_80037b2c(s: &plActionStats, h_player: i32, idx: i32) -> i32 {
    assert!((0..8).contains(&h_player), "pltrick.c:0x89: 0 <= h_player && h_player < 8");
    if idx < 0x64 {
        return s.x504[idx as usize] as i32 & (1i32 << (h_player as u8));
    }
    0
}

/// `pl_80037BC0`. As the C, `by_attack_counts[x2073]` is not bounds-checked: an id >= 100
/// increments the u32 `4 + id*4` bytes into `*st` (the following plAttackStats fields / the next
/// plAttackStats of the plActionStats). That matches the game wherever the target is a u32
/// field: every id for `attacks`, id <= 212 for `hits`, id <= 106 for `x358_hits`; past that it
/// lands on `x504`'s bytes (the game's big-endian increment differs) or outside plActionStats.
/// The pointer leaves `st`'s own borrow there (the caller's plActionStats owns the memory).
pub fn pl_80037bc0(st: &mut plAttackStats, ev: &Struct2070) {
    let e = unsafe { &ev.anon_0 };
    st.total = st.total.wrapping_add(1);
    let k = e.x2073 as usize;
    let c = unsafe { st.by_attack_counts.as_mut_ptr().add(k) };
    unsafe { *c = (*c).wrapping_add(1) };
    if e.count_thrown_items() != 0 {
        st.thrown_item_count = st.thrown_item_count.wrapping_add(1);
    }
    if e.count_aerials() != 0 {
        st.aerials_count = st.aerials_count.wrapping_add(1);
    }
    if e.count_specials() != 0 {
        st.specials_count = st.specials_count.wrapping_add(1);
    }
    if e.count_x1A0() != 0 {
        st.x1A0_count = st.x1A0_count.wrapping_add(1);
    }
    if e.count_x1A4() != 0 {
        st.x1A4_count = st.x1A4_count.wrapping_add(1);
    }
}

/// `pl_80037C60(gobj, prev2070_int)`: `prev` is the punned `Struct2070`.
pub fn pl_80037c60(g: G, prev: Struct2070) {
    let f = fp(g);
    let acp = action_stats(f.player_id as i32);
    let e = unsafe { f.x2070.anon_0 };
    if e.x2072_b2() != 0 {
        acp.set_x5BC_b0(1);
    }
    if e.x2072_b1() != 0 {
        acp.set_x5BC_b1(1);
    }
    if e.x2072_b0() != 0 {
        acp.set_x5BC_b2(1);
    }
    if e.x2071_b7() != 0 {
        acp.set_x5BC_b3(1);
    }
    let id = e.x2073;
    if id != 0 && id != unsafe { prev.anon_0.x2073 } {
        if id as i32 >= StatsAttack_Count as i32 {
            if f.is_sub_fighter() == 0 {
                by_attack_hi_inc(acp, id);
            }
        } else {
            let ev = f.x2070;
            pl_80037bc0(&mut acp.attacks, &ev);
        }
    }
}

/// `acp->by_attack_hi[id]++` for id >= 100. The C indexes past the 65-entry array: the u32 is
/// at plActionStats +0x3E8 + id*4. id 100..=115 is a u32 field (x578..x594, x598[0..8]) and is
/// incremented as one. 116/117 land on `x5B8[4]` / the `x5BC` bit byte: the game does a
/// big-endian u32 increment over those bytes, done here byte-wise (the native C differs).
/// id >= 118 runs past plActionStats into the rest of the GameCube StaticPlayer, whose native
/// layout differs: not ported (panics).
fn by_attack_hi_inc(acp: &mut plActionStats, id: u8) {
    match id {
        100..=115 => {
            let hi = unsafe { acp.anon_358.anon_0.by_attack_hi.as_mut_ptr().add(id as usize) };
            unsafe { *hi = (*hi).wrapping_add(1) };
        }
        116 | 117 => {
            let base = acp as *mut plActionStats as *mut u8;
            let p = unsafe { base.add(0x3E8 + id as usize * 4) };
            let mut b = [0u8; 4];
            unsafe { core::ptr::copy_nonoverlapping(p, b.as_mut_ptr(), 4) };
            b = u32::from_be_bytes(b).wrapping_add(1).to_be_bytes();
            unsafe { core::ptr::copy_nonoverlapping(b.as_ptr(), p, 4) };
        }
        _ => panic!("TODO(group3 pltrick) pl_80037C60: by_attack_hi[{id}] is past plActionStats (GC StaticPlayer layout)"),
    }
}

/// `pl_80037DF4`.
pub fn pl_80037df4(g: G, ev: &Struct2070) {
    let f = fp(g);
    let local = *ev;
    pl_80037bc0(&mut action_stats(f.player_id as i32).attacks, &local);
}

/// `pl_80037ECC`.
pub fn pl_80037ecc(g: G) {
    let f = fp(g);
    let a = &mut action_stats(f.player_id as i32).attacks;
    a.x1A8 = a.x1A8.wrapping_add(1);
}

/// `pl_800384DC(gobj, arg1, ev_data)`: `arg1` is the punned `Struct2070`. `ev_data` is
/// dereferenced unconditionally for `x10_b7` (as the C), so it must not be null when the
/// attack id is 1..99.
pub fn pl_800384dc(g: G, ev: Struct2070, ev_data: *mut ft_800898B4_t) {
    let f = fp(g);
    let id = unsafe { ev.anon_0.x2073 };
    if id != 0 && id < 0x64 {
        let (pid, sub) = (f.player_id as i32, f.is_sub_fighter() as i32);
        let acp = action_stats(pid);
        pl_80037bc0(&mut acp.hits, &ev);
        if !ev_data.is_null() && unsafe { (*ev_data).x10_b0() } != 0 {
            acp.hits.x1A8 = acp.hits.x1A8.wrapping_add(1);
        }
        if id == 0x63 {
            h::pl_8003fe40(pid, sub);
        }
        let best = unsafe { &mut acp.anon_358.x358_hits };
        if unsafe { (*ev_data).x10_b7() } == 0
            && acp.attacks.by_attack_counts[id as usize] > best.by_attack_counts[id as usize]
        {
            pl_80037bc0(best, &ev);
            if !ev_data.is_null() && unsafe { (*ev_data).x10_b0() } != 0 {
                best.x1A8 = best.x1A8.wrapping_add(1);
            }
            h::pl_8003dff4(pid, sub, id as i32);
        }
    }
    unsafe { f.x2074.anon_10.anon_0.set_x2084_b7(1) };
}

/// `pl_80038628`.
pub fn pl_80038628(g: G, kind: i32) -> bool {
    assert!((PlATK_AttackNormal_Start as i32..=PlATK_AttackNormal_End as i32).contains(&kind),
        "pltrick.c:0x1A1: PlATK_AttackNormal_Start <= kind <= End");
    let f = fp(g);
    let v = PL_803BCE70[(kind - 1) as usize];
    if v == -1 {
        return true;
    }
    h::ftdata_80085fd4_x8(f, v) != 0
}
