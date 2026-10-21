//! src/plbonus3.rs (plbonus.c primitives) against the C (gen/cref/plbonusc_ref.c, the game code
//! cut verbatim): the same xorshift-driven stale tables, tracing gmDecisionGetType /
//! gm_GetFrameCount stubs, and per op the return value, out-params, trace word and a hash of
//! x904[] plus the fields pl_80038F10 writes (tests/data/plbonusc.txt).
#![allow(static_mut_refs)]

use melee_fighter::hooks as h;
use melee_fighter::plbonus3 as p3;
use melee_fighter::player::get_stale_move_table_index_ptr2;
use melee_fighter::types::*;

static mut TR: u32 = 0;
static mut RS: u32 = 0;
static mut FC: u32 = 0;
static mut TAB: pl_804D6470_t = unsafe { core::mem::zeroed() };

fn t(v: u32) {
    unsafe { TR = TR.wrapping_mul(31).wrapping_add(v) }
}
fn rnd() -> u32 {
    unsafe {
        RS ^= RS << 13;
        RS ^= RS >> 17;
        RS ^= RS << 5;
        RS
    }
}
fn rf() -> f32 {
    match rnd() % 16 {
        0 => 0.0,
        1 => -0.0,
        2 => 1.0,
        3 => -1.0,
        4 => 5.0e9,
        5 => -3.0,
        _ => ((rnd() % 2001) as i32 - 1000) as f32 / 7.0,
    }
}
fn ext(slot: i32) -> &'static mut pl_StaleMoveTableExt_t {
    unsafe { &mut *get_stale_move_table_index_ptr2(slot) }
}

fn init_slot(slot: i32) {
    let x = ext(slot);
    for i in 0..215 {
        x.x0_staleMoveTable.x904[i] = if rnd() % 2 != 0 { rnd() % 3 } else { rnd() };
    }
    for i in 0..65 {
        unsafe { x.x0_staleMoveTable.total_attack_count_struct.anon_358.anon_0.by_attack_hi[i] = rnd() };
    }
    x.x0_staleMoveTable.xCAC = rnd() % 8;
    x.xD50 = rnd();
    x.xD54 = rnd();
    x.xD60 = rnd();
    x.xD64 = (rnd() % 8) as i32;
    x.xD6C = (rnd() % 8) as i32;
}

static mut HH: u32 = 0;
fn hh(v: u32) {
    unsafe { HH = HH.wrapping_mul(0x01000193) ^ v }
}
fn hash_slot(slot: i32) {
    let x = ext(slot);
    for i in 0..215 {
        hh(x.x0_staleMoveTable.x904[i]);
    }
    hh(x.x0_staleMoveTable.xCAC);
    hh(x.xD50);
    hh(x.xD54);
    hh(x.xD60);
    hh(x.xD64 as u32);
    hh(x.xD6C as u32);
}

const NOPS: u32 = 15;

#[test]
fn plbonusc_matches_c() {
    unsafe {
        h::PL_804D6470 = &raw const TAB;
        h::GM_GET_FRAME_COUNT = Some(|| {
            t(0x170);
            FC
        });
        h::GM_DECISION_GET_TYPE = Some(|k| {
            t(0x500u32.wrapping_add(k as u32));
            if k % 3 == 0 { p3::GM_DEC_TYPE_POINT } else { p3::GM_DEC_TYPE_FLAG }
        });
    }
    let want = include_str!("data/plbonusc.txt");
    let mut lines = want.lines();
    let mut seen = [false; NOPS as usize];
    let mut arr = [0i32; 40];
    let mut a6e8 = pl_800386E8_arg0_t::zeroed();
    let mut n = 0;
    for k in 0..300u32 {
        unsafe {
            RS = 0xC0B05u32.wrapping_add(k.wrapping_mul(2654435761));
            TR = 0;
        }
        for i in 0..6 {
            init_slot(i);
        }
        for step in 0..50 {
            let slot = (rnd() % 6) as i32;
            let op = rnd() % NOPS;
            let pk = (rnd() % 72) as i32 * 3;
            let mut fk = pk + 1 + (rnd() % 2) as i32;
            let anyk = (rnd() % 215) as i32;
            let kk = if rnd() % 3 == 0 { -1 } else { pk };
            let c = rnd();
            let fsel = rnd() % 6;
            let a3 = 1 + rnd() % 20;
            let a4 = rnd() % 8;
            let a5 = rnd() % 16;
            let mut uo = rnd() % 3;
            let mut io = (rnd() % 16) as i32;
            let mut fo = rf();
            let f6 = rf();
            let f7 = rf();
            let lo = (rnd() % 22) as i32 - 1;
            let hi = (rnd() % 40) as i32 - 2;
            let (mut ret, mut o1, mut o2) = (0u32, 0u32, 0u32);
            if fk > 214 {
                fk = 214 - (214 % 3 == 0) as i32;
            }
            unsafe {
                FC = match fsel {
                    0 => 0,
                    1 => (-2i32) as u32,
                    5 => rnd(),
                    _ => 1 + rnd() % 40,
                };
            }
            for a in arr.iter_mut() {
                *a = rnd() as i32;
            }
            a6e8.unk_190 = rnd() as i32;
            a6e8.unk_5AC = rnd() as i32;
            a6e8.unk_5B0 = rnd() as i32;
            match op {
                0 => ret = p3::pl_800386d8(&ext(slot).x0_staleMoveTable.total_attack_count_struct, (c % 65) as i32),
                1 => ret = p3::pl_800386e8(&a6e8) as u32,
                2 => ret = p3::fn_80038700(&arr, lo, hi.min(38)) as u32,
                3 => p3::pl_80038788(slot, pk, c as i32),
                4 => p3::pl_80038824(slot, pk),
                5 => p3::pl_80038898(slot, pk),
                6 => ret = (p3::pl_80038914() == &raw const TAB) as u32,
                7 => p3::pl_80038f10(slot),
                8 => p3::fn_80038fb8(slot),
                9 => {
                    ret = p3::pl_8003906c(slot, kk, &mut uo, a3, a4, a5, &mut io) as u32;
                    o1 = uo;
                    o2 = io as u32;
                }
                10 => {
                    let mut iv = uo as i32;
                    ret = p3::pl_80039238(slot, kk, &mut iv, a3, &mut fo, f6, f7) as u32;
                    o1 = iv as u32;
                    o2 = fo.to_bits();
                }
                11 => ret = p3::pl_80039418(slot, anyk),
                12 => p3::set_flag(slot, fk),
                13 => p3::set_point_value(slot, pk, c),
                14 => p3::reset_bonuses(slot),
                _ => unreachable!(),
            }
            seen[op as usize] = true;
            unsafe { HH = 0x811C9DC5 };
            for i in 0..6 {
                hash_slot(i);
            }
            let got = format!("{} {} {} {:x} {:x} {:x} {:x} {:x}", k, step, op, ret, o1, o2, unsafe { TR }, unsafe { HH });
            let exp = lines.next().expect("C output ended early");
            assert_eq!(got, exp, "case {k} step {step} op {op}");
            n += 1;
        }
    }
    assert!(lines.next().is_none());
    assert!(seen.iter().all(|&s| s), "every op exercised");
    assert_eq!(n, 15000);
}
