//! Group 3 round 10: the middle of src/melee/pl/plbonuslib.c, `pl_8003DF44` .. `pl_8003ED0C`
//! (24 C definitions incl. the static inline `match_item_kind`), field logic over
//! `pl_StaleMoveTableExt_t`. Calls out go through `crate::hooks` (plbonus.c, items, ftLib).
//! One fused site: `pl_8003ED0C`'s `xCA0 * (n - 1) + arg2` (fmadds, fma_sites.py).
//! Checked against the C (cut verbatim) by tests/plbonus_ref.rs (gen/cref/plbonus_ref.c).
//!
//! Not ported here: pl_8003E70C / pl_8003E7D4 (index the 8-entry `x598` with item kinds
//! 161..190, i.e. past the array into neighbouring fields), the fn_8003EE2C.. tail of the range.
#![allow(non_upper_case_globals)]

use crate::hooks as h;
use crate::plbonus::{check_if_same_team, calculate_average, pl_8003fc88, pl_8003ff44, pl_800419ac, pl_80040ed4};
use crate::player::get_entity_at_index;
use crate::types::*;
use melee_core::fp::fmadds;

type I = *mut Item_GObj;

#[inline]
fn ext(slot: i32) -> &'static mut pl_StaleMoveTableExt_t {
    unsafe { &mut *crate::player::get_stale_move_table_index_ptr2(slot) }
}

/// `pl_8003DF44`.
pub fn pl_8003df44(arg0: i32, arg1: i32) {
    let r3 = h::ftlib_80087300(get_entity_at_index(arg0, arg1));
    if r3 == 6 || r3 == arg0 || check_if_same_team(arg0, r3) {
        return;
    }
    ext(r3).xD54 = h::pl_804d6470().xB4;
}

/// `pl_8003DFF4`.
pub fn pl_8003dff4(arg0: i32, arg1: i32, arg2: i32) {
    pl_8003fc88(arg0, arg1, arg2);
    pl_8003ff44(arg0, arg1, arg2);
    pl_8003eaac(arg0, arg1, arg2);
}

/// `pl_8003E058`.
pub fn pl_8003e058(arg0: i32, _arg1: i32, arg2: i32, _arg3: i32) {
    h::pl_80038824(arg2, 0x3F);
    if arg0 == 6 || check_if_same_team(arg2, arg0) {
        return;
    }
    h::pl_80038824(arg0, 0x3E);
}

/// `pl_8003E0E8`.
pub fn pl_8003e0e8(arg0: i32, arg1: i32) {
    if arg1 != 0 {
        return;
    }
    h::pl_80038824(arg0, 0x40);
}

/// `pl_8003E114`.
pub fn pl_8003e114(arg0: i32, _arg1: i32, arg2: f32) {
    if arg2 <= h::pl_804d6470().x84 {
        h::pl_80038788(arg0, 0x41, 1);
    }
}

/// `pl_8003E150`.
pub fn pl_8003e150(slot: i32, _arg1: i32) {
    let s = &mut ext(slot).x0_staleMoveTable;
    s.xCD8 = s.xCD8.wrapping_add(1);
}

/// `pl_8003E17C`.
pub fn pl_8003e17c(player_id: i32, _arg1: i32, item: I) {
    let t = ext(player_id);
    let kind = h::it_get_kind(item);
    let kind2 = if kind >= It_Kind_Capsule as i32 && kind < It_Kind_L_Gun_Ray as i32 {
        kind
    } else {
        match kind {
            It_Kind_Lucky_Egg => It_Kind_L_Gun_Ray as i32,
            It_Kind_WhispyApple => It_Kind_StarRod_Star as i32,
            It_Kind_WhispyHealApple => It_Kind_LipStick_Spore as i32,
            It_Kind_Hammer_Head => It_Kind_S_Scope_Beam as i32,
            _ => -1,
        }
    };
    if kind2 != -1
        && (kind2 == It_Kind_L_Gun_Ray as i32
            || kind2 == It_Kind_Egg as i32
            || kind2 == It_Kind_S_Scope_Beam as i32
            || h::it_8026b7e8(item) == 1)
    {
        let s = &mut t.x0_staleMoveTable;
        if !h::it_8026b774(item, player_id as u8) {
            s.x674[kind2 as usize] = s.x674[kind2 as usize].wrapping_add(1);
        }
        for cnt in 0..6 {
            if h::it_8026b774(item, cnt) {
                return;
            }
        }
        s.x710[kind2 as usize] = s.x710[kind2 as usize].wrapping_add(1);
    }
}

/// `pl_8003E2CC` (HSD_ASSERT kind < Pl_ItemLog_Terminate -> panic).
pub fn pl_8003e2cc(arg0: i32, kind: u32) -> u32 {
    assert!(kind < Pl_ItemLog_Terminate as u32, "plbonuslib.c:555");
    ext(arg0).x0_staleMoveTable.x674[kind as usize]
}

/// `pl_8003E334`.
pub fn pl_8003e334(arg0: i32, kind: u32) -> u32 {
    assert!(kind < Pl_ItemLog_Terminate as u32, "plbonuslib.c:564");
    ext(arg0).x0_staleMoveTable.x710[kind as usize] as u32
}

/// `pl_8003E39C`.
pub fn pl_8003e39c(arg0: i32) -> i32 {
    let s = &ext(arg0).x0_staleMoveTable;
    s.x674.iter().fold(0i32, |a, &v| a.wrapping_add(v as i32))
}

/// `pl_8003E420`.
pub fn pl_8003e420(arg0: i32) -> u32 {
    let s = &ext(arg0).x0_staleMoveTable;
    s.x710.iter().fold(0i32, |a, &v| a.wrapping_add(v)) as u32
}

/// plbonuslib.c static inline `match_item_kind`.
#[inline]
pub fn match_item_kind(kind: i32) -> i32 {
    if kind >= It_Kind_Capsule as i32 && kind < It_Kind_L_Gun_Ray as i32 {
        return kind;
    }
    match kind {
        It_Kind_Lucky_Egg => Pl_ItemLog_Unk35 as i32,
        It_Kind_WhispyApple => Pl_ItemLog_Unk36 as i32,
        It_Kind_WhispyHealApple => Pl_ItemLog_Unk37 as i32,
        It_Kind_Hammer_Head => Pl_ItemLog_Unk38 as i32,
        _ => -1,
    }
}

/// `pl_8003E4A4`. C quirk not reproduced: an unknown kind gives -1, which passes the C's signed
/// `< Pl_ItemLog_Terminate` test and stores to `seen[-1]` (out of bounds on the stack); here it
/// is skipped. The oracle only drives kinds that map.
pub fn pl_8003e4a4(slot: i32, _arg1: bool, moves: &[i32]) {
    let t = ext(slot);
    let mut seen = [0u32; Pl_ItemLog_Terminate as usize];
    for &m in moves {
        let k = match_item_kind(m);
        if k >= 0 && k < Pl_ItemLog_Terminate as i32 {
            seen[k as usize] = 1;
        }
    }
    let lim = h::pl_804d6470().x138;
    for i in 0..Pl_ItemLog_Terminate as usize {
        let s = &mut t.x0_staleMoveTable;
        if seen[i] == 1 {
            s.x7AC[i] = s.x7AC[i].wrapping_add(1);
            if s.x7AC[i] as u32 == lim {
                h::pl_80038788(slot, 0x9D, 1);
                s.x7AC[i] = 0;
            }
        } else {
            s.x7AC[i] = 0;
        }
    }
}

/// `pl_8003E854`.
pub fn pl_8003e854(arg0: i32, arg1: i32, arg2: I) {
    let r31 = get_entity_at_index(arg0, arg1);
    let r3 = h::it_get_owner(arg2);
    if r31 != r3 && h::it_8026b7bc(arg2) != 0 {
        h::pl_80038824(arg0, 0x9A);
        if h::it_get_kind(arg2) == It_Kind_Dosei as i32 {
            h::pl_80038788(arg0, 0xB6, 1);
        }
        if arg1 == 0 && h::ftlib_80086960(r3) && h::ftlib_800874bc(r3) == 0 {
            let id = h::ftlib_80086be0(r3);
            ext(arg0).xD6C = id as i32;
        }
    }
    if h::it_get_kind(arg2) == It_Kind_BombHei as i32 && h::it_8026b960(arg2) <= h::pl_804d6470().x180 {
        pl_800419ac(arg0, arg1, h::it_8026b7cc(arg2) as u16);
    }
    if arg1 != 0 {
        return;
    }
    ext(arg0).x0_staleMoveTable.xCA8 = 0;
}

/// `pl_8003E978`.
pub fn pl_8003e978(slot: i32, b4: bool, _item: I) {
    fn_8003e998(slot, b4 as i32);
}

/// `fn_8003E998`.
pub fn fn_8003e998(arg0: i32, index: i32) {
    let t = ext(arg0);
    if index == 0 && t.xD6C != 6 {
        let g = get_entity_at_index(arg0, index);
        unsafe { (*h::ft_800898a8(g)).x207C.y = t.xD6C };
    }
}

/// `pl_8003EA08`.
pub fn pl_8003ea08(slot: i32, arg1: i32) {
    if arg1 != 0 {
        return;
    }
    ext(slot).xD6C = 6;
}

/// `pl_8003EA40`.
pub fn pl_8003ea40(arg0: i32, _arg1: i32, arg2: i32, arg3: i32, arg4: i32) {
    if arg3 != 0 || arg2 != arg4 {
        return;
    }
    h::pl_80038824(arg0, 0x9B);
}

/// `pl_8003EA74`.
pub fn pl_8003ea74(arg0: i32, arg1: i32) {
    if arg1 != 0 {
        return;
    }
    ext(arg0).xD44 = 0;
}

/// `pl_8003EAAC`.
pub fn pl_8003eaac(arg0: i32, arg1: i32, arg2: i32) {
    let s = &mut ext(arg0).x0_staleMoveTable;
    if arg1 == 0 && arg2 == 0x59 {
        s.xCA8 = s.xCA8.wrapping_add(1);
        if s.xCA8 == h::it_8026c1d4() {
            h::pl_80038788(arg0, 0xC2, 1);
        }
    }
}

/// `pl_8003EB30`.
pub fn pl_8003eb30(arg0: f32, arg1: i32, arg2: i32, arg3: i32, arg4: i32, arg5: i32) {
    if arg1 != 6 && arg1 != arg3 {
        {
            let s = &mut ext(arg1).x0_staleMoveTable;
            s.xC6C += arg0;
            if arg5 == 0x34 {
                s.xC70 += arg0;
            }
            if (arg5.wrapping_sub(0x5C) as u32) <= 1 {
                s.xC74 += arg0;
            }
        }
        if arg3 != 6 {
            if arg4 == 0 {
                let v = &mut ext(arg3).x0_staleMoveTable;
                v.xC78[arg1 as usize] += arg0;
            }
            pl_80040ed4(arg1, arg2, arg3, arg4, arg5, arg0);
        }
        ext(arg1).xD08 = 0;
    }
}

/// `pl_8003EC30`.
pub fn pl_8003ec30(slot: i32, arg1: i32, arg2: i32, arg3: f32) {
    if arg1 != 0 {
        return;
    }
    let s = &mut ext(slot).x0_staleMoveTable;
    if arg2 == 1 || arg2 == 3 {
        s.xC98 += arg3;
    }
}

/// `pl_8003EC9C`.
pub fn pl_8003ec9c(arg0: i32, arg1: i32, arg2: f32, arg3: f32) {
    let t = ext(arg0);
    if arg1 != 0 {
        return;
    }
    t.x0_staleMoveTable.xC60 += arg3;
    if t.x0_staleMoveTable.xC64 < arg2 {
        t.x0_staleMoveTable.xC64 = arg2;
    }
    t.xD10 = 0;
}

/// `pl_8003ED0C`: `xCA0 * (n - 1) + arg2` is one fmadds in the shipping build.
pub fn pl_8003ed0c(arg0: i32, _arg1: i32, r5: i32, _arg3: i32, arg2: f32) {
    if arg0 == r5 || check_if_same_team(r5, arg0) {
        return;
    }
    let s = &mut ext(arg0).x0_staleMoveTable;
    s.xCA4 = s.xCA4.wrapping_add(1);
    let n = s.xCA4;
    let f30 = fmadds(s.xCA0, n.wrapping_sub(1) as f32, arg2);
    s.xCA0 = calculate_average(f30, n as f32);
}
