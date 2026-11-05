//! Native port of src/melee/pl/pltrick.c: per-player attack statistics bookkeeping (attacks
//! thrown, hits landed, who hit whom, hit-from-behind counts).
//!
//! 7 of the 9 definitions (`pl_80037B2C`, `pl_80037BC0`, `pl_80037C60`, `pl_80037DF4`,
//! `pl_80037ECC`, `pl_800384DC`, `pl_80038628`) and `pl_803BCE70` already have a port in
//! `crate::pltrick`; they are re-exported here under the C names. The remaining two, the static
//! `fn_80037F00` and `pl_80038144`, are ported below.
//!
//! `union Struct2070` int arguments are taken as the `Struct2070` they pun (see `crate::ft0892`).
//! Fused sites (`tools/rust-port/fma_sites.py src/melee/pl/pltrick.c`): none in this TU's float code
//! (`fn_80037F00` only compares floats).
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc, unused_unsafe)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::hooks as h;
#[allow(unused_imports)]
use crate::stale::action_stats;
#[allow(unused_imports)]
use crate::pltrick::{PL_803BCE70, by_attack_hi_inc, fp};
use crate::types::*;

pub use crate::pltrick::{
    pl_80037b2c as pl_80037B2C, pl_80037bc0 as pl_80037BC0, pl_80037c60 as pl_80037C60, pl_80037df4 as pl_80037DF4,
    pl_80037ecc as pl_80037ECC, pl_800384dc as pl_800384DC, pl_80038628 as pl_80038628, PL_803BCE70 as pl_803BCE70,
};

type G = *mut HSD_GObj;

/// Calls out of the TU. `hook::<name>` overrides; unset = the port, or a TODO panic.
pub mod ext {
    use super::*;
    macro_rules! ext_fns {
        ($($name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? = $def:expr;)*) => {
            pub mod hook {
                #[allow(unused_imports)] use super::*;
                $( #[allow(non_upper_case_globals)] pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( #[allow(non_snake_case)] pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } { Some(f) => f($($a),*), None => ($def)($($a),*) }
            } )*
        };
    }
    ext_fns! {
        ifMagnify_802FB6E8(slot: u8) -> i32 = |slot: u8| -> i32 { unsafe { melee_world::tu::ifmagnify::ifMagnify_802FB6E8(slot as _) } };
        gm_8016B0D4() -> bool = || todo!("TODO(port) gm_8016B0D4 (gm/gm_unsplit.c)");
        gm_8016B168() -> bool = crate::hooks::gm_8016b168;
        pl_80038914() -> *const pl_804D6470_t = crate::plbonus3::pl_80038914;
        ft_80089914(g: G, msid: i32) -> bool = crate::ft0892::ft_80089914;
        pl_800410F4(a: i32, b: i32, c: i32, d: i32, e: i32) = crate::plbonus::pl_800410f4;
        pl_80040D8C(slot: i32, sub: i32) = crate::plbonus::pl_80040d8c;
        pl_8003FE40(slot: i32, sub: i32) = crate::plbonus::pl_8003fe40;
        pl_8003FE64(slot: i32, sub: i32) = crate::plbonus::pl_8003fe64;
        pl_8003DFF4(slot: i32, sub: i32, id: i32) = crate::plbonus2::pl_8003dff4;
        pl_8003ED0C(a: i32, b: i32, c: i32, d: i32, pct: f32) = crate::plbonus2::pl_8003ed0c;
        pl_8003EA40(a: i32, b: i32, c: i32, d: i32, e: i32) = crate::plbonus2::pl_8003ea40;
        pl_800403FC(a: i32, b: i32, c: i32, d: i32, e: i32) = crate::plbonus::pl_800403fc;
        pl_80040FBC(a: i32, b: i32, c: i32, d: i32, e: i32) = crate::plbonus::pl_80040fbc;
    }
}

#[inline(always)]
unsafe fn get_fighter(g: G) -> *mut Fighter {
    (*g).user_data as *mut Fighter
}

/// `fn_80037F00` (static): fill `victim->dmg.x18d8` from the event `ev` (or a default) and flag
/// the hit. `gp` is, in the C, the attacker `Fighter*` cast to `HSD_GObj**` (the call site in
/// `pl_80038144` does `(HSD_GObj**) attacker_fp`), so `*gp` is `attacker_fp->gobj`
/// (Fighter offset 0); that is what `attacker_fp` stands for here.
unsafe fn fn_80037F00(attacker_fp: *mut Fighter, fp: *mut Fighter, ev: *mut ft_800898B4_t, arg3: i32, arg4: i32) {
    let fp = &mut *fp;
    let prev_kb: f32 = if fp.x221C_b6() != 0 { fp.dmg.x18d8.kb_applied1 } else { 0.0 };

    if !ev.is_null() {
        fp.dmg.x18d8 = *ev;
    } else {
        let d = &mut fp.dmg.x18d8;
        d.x0 = 0;
        d.x4 = 0;
        d.kb_applied1 = 0.0;
        d.xC = 6;
        d.set_x10_b0(0);
        d.set_x11_b3(0);
        d.set_x10_b1(0);
        d.set_x10_b2(0);
        d.set_x10_b3(0);
        d.set_x10_b4(0);
        d.set_x10_b5(0);
        d.set_x10_b6(0);
        d.set_x10_b7(0);
        d.set_x11_b0(0);
        d.set_x11_b1(0);
        d.set_x11_b2(0);
        d.set_x11_b4(0);
    }

    fp.dmg.x18d8.x4 = melee_core::ppc::fctiwz_f32(fp.dmg.x1830_percent);

    fp.dmg.x18d8.set_x11_b0(arg3 as u8);

    if ext::ifMagnify_802FB6E8(fp.player_id as u8) != 0 {
        fp.dmg.x18d8.set_x11_b1(1);
    }

    if fp.motion_id == 0xFD {
        fp.dmg.x18d8.set_x11_b2(1);
    }

    if fp.dmg.x18c4_source_ply != arg4 && fp.dmg.x18d8.x10_b0() != 0 {
        let table = ext::pl_80038914();
        if prev_kb >= (*table).x14 {
            fp.dmg.x18d8.set_x11_b3(1);
        }
    }

    if !attacker_fp.is_null() {
        let gobj = (*attacker_fp).gobj;
        if ext::ft_80089914(gobj, fp.dmg.x18d4.x3 as i32) && (fp.victim_gobj.is_null() || gobj != fp.victim_gobj) {
            fp.dmg.x18d8.set_x11_b4(1);
        }
        // The C stores through `ev` unconditionally here; a null `ev` with an attacker is a
        // caller bug (the C writes to address 0), so it is skipped.
        if !ev.is_null() {
            (*ev).set_x10_b7(1);
        }
    }
}

/// `pl_80038144`: record a landed hit for the attacker's stats and fill the victim's hit record.
/// `x18d4` is the punned `Struct2070` (the C's `s32 x18d4_int`).
pub unsafe fn pl_80038144(
    attacker_gobj: G,
    victim_gobj: G,
    x18d4: Struct2070,
    ev_data: *mut ft_800898B4_t,
    attack_instance: u16,
    arg5: i32,
    source_ply: i32,
) {
    melee_core::ct!("pl_80038144");
    let _ = arg5;
    let attacker_fp: *mut Fighter = if !attacker_gobj.is_null() { get_fighter(attacker_gobj) } else { core::ptr::null_mut() };
    let victim_fp = get_fighter(victim_gobj);
    let mut attacked_from_behind: i32 = 0;
    let ev = x18d4;
    let ev_id = ev.anon_0.x2073;

    if !attacker_fp.is_null() && ev_id != 0 {
        let facing_dir = (*victim_fp).facing_dir;

        if facing_dir * (*attacker_fp).cur_pos.x < facing_dir * (*victim_fp).cur_pos.x {
            let var_r0 = if ev_id == 0x60 || ev_id == 0x61 || (ev_id >= 0x33 && ev_id <= 0x3D) { 0 } else { 1 };
            if var_r0 != 0 {
                attacked_from_behind = 1;
            }
        }

        ext::pl_800410F4(
            (*attacker_fp).player_id as i32,
            (*attacker_fp).is_sub_fighter() as i32,
            (*victim_fp).player_id as i32,
            (*victim_fp).is_sub_fighter() as i32,
            ev_id as i32,
        );
    }

    if attack_instance == 0 || (*victim_fp).dmg.x18ec_instancehitby != attack_instance {
        // *(s32*) &victim_fp->dmg.x18d4 = ev.x2070_int: a 4-byte copy of the GC-ordered bytes.
        (*victim_fp).dmg.x18d4 = core::mem::transmute::<Struct2070, UnkPlBonusBits>(ev);
        (*victim_fp).dmg.x18ec_instancehitby = attack_instance;

        if !attacker_fp.is_null() && (*victim_fp).dmg.x18d4.x3 != 0 {
            if ext::gm_8016B168() && ext::gm_8016B0D4() && (*attacker_fp).team == (*victim_fp).team {
                ext::pl_80040D8C((*attacker_fp).player_id as i32, (*attacker_fp).is_sub_fighter() as i32);
            }

            let attack_id = ev_id;
            let mut attack_id2 = attack_id;
            if attack_id2 < 0x64 {
                let acp = crate::stale::action_stats((*attacker_fp).player_id as i32);
                let ev_reload: Struct2070 = core::mem::transmute::<UnkPlBonusBits, Struct2070>((*victim_fp).dmg.x18d4);
                let fp2 = get_fighter(attacker_gobj);
                let acp2 = crate::stale::action_stats((*get_fighter(attacker_gobj)).player_id as i32);
                let ev_hits = ev_reload;
                crate::pltrick::pl_80037bc0(&mut acp2.hits, &ev_hits);

                if !ev_data.is_null() && (*ev_data).x10_b0() != 0 {
                    acp2.hits.x1A8 = acp2.hits.x1A8.wrapping_add(1);
                }

                attack_id2 = ev_reload.anon_0.x2073;
                if attack_id2 == 0x63 {
                    ext::pl_8003FE40((*fp2).player_id as i32, (*fp2).is_sub_fighter() as i32);
                }

                // The C reads `ev_data->x10_b7` unconditionally here.
                if (*ev_data).x10_b7() == 0
                    && acp2.attacks.by_attack_counts[attack_id2 as usize]
                        > acp2.anon_358.x358_hits.by_attack_counts[attack_id2 as usize]
                {
                    let ev_best = ev_reload;
                    crate::pltrick::pl_80037bc0(&mut acp2.anon_358.x358_hits, &ev_best);

                    if !ev_data.is_null() && (*ev_data).x10_b0() != 0 {
                        acp2.anon_358.x358_hits.x1A8 = acp2.anon_358.x358_hits.x1A8.wrapping_add(1);
                    }

                    ext::pl_8003DFF4((*fp2).player_id as i32, (*fp2).is_sub_fighter() as i32, attack_id2 as i32);
                }

                if attacked_from_behind != 0 {
                    acp.x56C = acp.x56C.wrapping_add(1);
                } else {
                    acp.x568 = acp.x568.wrapping_add(1);
                }

                if attack_id == 8 || attack_id == 0xB {
                    acp.x574 = acp.x574.wrapping_add(1);
                } else {
                    acp.x570 = acp.x570.wrapping_add(1);
                }

                let h_player = (*victim_fp).player_id as i32;
                let x18d4_x3 = (*victim_fp).dmg.x18d4.x3 as i32;
                assert!((0..8).contains(&h_player), "pltrick.c:0x7E: 0 <= h_player && h_player < 8");
                if x18d4_x3 < 0x64 {
                    acp.x504[x18d4_x3 as usize] |= (<i32 as melee_core::ppc::Ppc>::pshl(1i32, ((h_player as u8)) as i64)) as u8;
                }

                ext::pl_8003FE64((*attacker_fp).player_id as i32, (*attacker_fp).is_sub_fighter() as i32);
                ext::pl_8003ED0C(
                    (*attacker_fp).player_id as i32,
                    (*attacker_fp).is_sub_fighter() as i32,
                    (*victim_fp).player_id as i32,
                    (*victim_fp).is_sub_fighter() as i32,
                    (*victim_fp).dmg.x1830_percent,
                );
                let xc = (*ev_data).xC;
                ext::pl_8003EA40(
                    (*attacker_fp).player_id as i32,
                    (*attacker_fp).is_sub_fighter() as i32,
                    (*victim_fp).player_id as i32,
                    (*victim_fp).is_sub_fighter() as i32,
                    xc,
                );
                ext::pl_800403FC(
                    (*attacker_fp).player_id as i32,
                    (*attacker_fp).is_sub_fighter() as i32,
                    (*victim_fp).player_id as i32,
                    (*victim_fp).is_sub_fighter() as i32,
                    (*victim_fp).dmg.x18d4.x3 as i32,
                );
                ext::pl_80040FBC(
                    (*attacker_fp).player_id as i32,
                    (*attacker_fp).is_sub_fighter() as i32,
                    (*victim_fp).player_id as i32,
                    (*victim_fp).is_sub_fighter() as i32,
                    (*victim_fp).dmg.x18d4.x3 as i32,
                );
            }
        }
    }

    fn_80037F00(attacker_fp, victim_fp, ev_data, attacked_from_behind, source_ply);
}

// ---- folded from pltrick.rs (consolidation steps 21/22) ----
/// `pl_80037B2C`. The C has no return for `idx >= 100` (undefined); this returns 0 there.
pub fn pl_80037b2c(s: &plActionStats, h_player: i32, idx: i32) -> i32 {
    melee_core::ct!("pl_80037b2c");
    assert!((0..8).contains(&h_player), "pltrick.c:0x89: 0 <= h_player && h_player < 8");
    if idx < 0x64 {
        return s.x504[idx as usize] as i32 & (<i32 as melee_core::ppc::Ppc>::pshl(1i32, ((h_player as u8)) as i64));
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
    melee_core::ct!("pl_80037bc0");
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
    melee_core::ct!("pl_80037c60");
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

/// `pl_80037DF4`.
pub fn pl_80037df4(g: G, ev: &Struct2070) {
    melee_core::ct!("pl_80037df4");
    let f = fp(g);
    let local = *ev;
    pl_80037bc0(&mut action_stats(f.player_id as i32).attacks, &local);
}

/// `pl_80037ECC`.
pub fn pl_80037ecc(g: G) {
    melee_core::ct!("pl_80037ecc");
    let f = fp(g);
    let a = &mut action_stats(f.player_id as i32).attacks;
    a.x1A8 = a.x1A8.wrapping_add(1);
}

/// `pl_800384DC(gobj, arg1, ev_data)`: `arg1` is the punned `Struct2070`. `ev_data` is
/// dereferenced unconditionally for `x10_b7` (as the C), so it must not be null when the
/// attack id is 1..99.
pub fn pl_800384dc(g: G, ev: Struct2070, ev_data: *mut ft_800898B4_t) {
    melee_core::ct!("pl_800384dc");
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

/// `pl_80038628_fp`.
pub fn pl_80038628_fp(g: G, kind: i32) -> bool {
    melee_core::ct!("pl_80038628_fp");
    assert!((PlATK_AttackNormal_Start as i32..=PlATK_AttackNormal_End as i32).contains(&kind),
        "pltrick.c:0x1A1: PlATK_AttackNormal_Start <= kind <= End");
    let f = fp(g);
    let v = PL_803BCE70[(kind - 1) as usize];
    if v == -1 {
        return true;
    }
    h::ftdata_80085fd4_x8(f, v) != 0
}

// Moved here from plbonus.rs (upstream moved them into src/melee/pl/pltrick.c).
/// `pl_800386D8`: `arg0->by_attack_hi[arg1]` (unchecked: indexes past the member into x504..)
pub fn pl_800386D8(arg0: *mut plActionStats, arg1: isize) -> u32 {
    melee_core::ct!("pl_800386D8");
    unsafe {
        let hi = (arg0 as *const u8).add(0x358 + 4 * (1 + 35)) as *const u32;
        *hi.offset(arg1)
    }
}

pub fn pl_800386E8(arg0: *mut pl_800386E8_arg0_t) -> i32 {
    melee_core::ct!("pl_800386E8");
    unsafe {
        let temp = (*arg0).unk_190.wrapping_add((*arg0).unk_5AC);
        temp.wrapping_add((*arg0).unk_5B0)
    }
}

pub fn fn_80038700(arg0: *const i32, arg1: i32, arg2: i32) -> i32 {
    melee_core::ct!("fn_80038700");
    let mut sum: i32 = 0;
    let mut i = arg1;
    while i <= arg2 {
        sum = sum.wrapping_add(unsafe { *arg0.offset((i + 1) as isize) });
        i += 1;
    }
    sum
}

/// `pl_800386D8`.
pub fn pl_800386d8(arg0: &plActionStats, arg1: i32) -> u32 {
    melee_core::ct!("pl_800386d8");
    // The C indexes `by_attack_hi[arg1]` with ids past its 65 entries (plbonus.c reads 0x65..0x72: the bonus checks),
    // so it reads the plActionStats words that follow the union (x504.., x568..). Same address arithmetic here.
    unsafe { core::ptr::addr_of!(arg0.anon_358.anon_0.by_attack_hi).cast::<u32>().add(arg1 as usize).read_unaligned() }
}

/// `pl_800386E8`.
pub fn pl_800386e8(arg0: &pl_800386E8_arg0_t) -> i32 {
    melee_core::ct!("pl_800386e8");
    let temp = arg0.unk_190.wrapping_add(arg0.unk_5AC);
    temp.wrapping_add(arg0.unk_5B0)
}
