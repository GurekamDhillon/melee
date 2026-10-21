//! src/plbonus.rs (pl_040D.c + plbonuslib.c tail) against the C (gen/cref/plbonus_ref.c, the game
//! code cut verbatim): the same xorshift-driven player tables, stale-table extensions and
//! pl_804D6470 constants, the same tracing stubs for every call out, and per op the return value,
//! trace word and a hash of every written field compared (tests/data/plbonus.txt).
#![allow(static_mut_refs)]

use melee_fighter::hooks as h;
use melee_fighter::plbonus as pb;
use melee_fighter::plbonus2 as p2;
use melee_fighter::player::get_stale_move_table_index_ptr2;
use melee_fighter::stale::PLAYER_SLOTS;
use melee_fighter::types::*;

static mut TR: u32 = 0;
static mut RS: u32 = 0;
static mut GA: HSD_GObj = unsafe { core::mem::zeroed() };
static mut GB: HSD_GObj = unsafe { core::mem::zeroed() };
static mut KB: ft_800898B4_t = unsafe { core::mem::zeroed() };
static mut TAB: pl_804D6470_t = unsafe { core::mem::zeroed() };
static mut GI: HSD_GObj = unsafe { core::mem::zeroed() };
static mut S2074: Struct2074 = unsafe { core::mem::zeroed() };
const IKINDS: [i32; 12] = [0, 7, 6, 16, 34, 35, 3, 38, 205, 225, 226, 40];
const MKINDS: [i32; 10] = [0, 7, 6, 16, 34, 3, 205, 225, 226, 1];

fn t(v: u32) {
    unsafe { TR = TR.wrapping_mul(31).wrapping_add(v) }
}
fn tr() -> u32 {
    unsafe { TR }
}
fn b(f: f32) -> u32 {
    f.to_bits()
}
fn e(g: *mut HSD_GObj) -> u32 {
    if g.is_null() { 0 } else if g == &raw mut GA { 1 } else if g == &raw mut GB { 2 } else { 3 }
}
fn ent3(k: u32) -> *mut HSD_GObj {
    match k {
        0 => core::ptr::null_mut(),
        1 => &raw mut GA,
        _ => &raw mut GB,
    }
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
fn rent() -> *mut HSD_GObj {
    match rnd() % 3 {
        0 => core::ptr::null_mut(),
        1 => &raw mut GA,
        _ => &raw mut GB,
    }
}
fn z() -> u32 {
    if rnd() % 2 != 0 { 0 } else { rnd() }
}

fn install() {
    unsafe {
        h::PL_804D6470 = &raw const TAB;
        h::FTLIB_8008701C = Some(|g| {
            t(0x40 + e(g));
            (tr() >> 3) & 1 != 0
        });
        h::FTLIB_80087300 = Some(|g| {
            t(0x1C0 + e(g));
            (tr() % 7) as i32
        });
        h::GM_8016B168 = Some(|| {
            t(0x140);
            (tr() >> 4) & 1 != 0
        });
        h::GM_GET_FRAME_COUNT = Some(|| {
            t(0x170);
            tr()
        });
        h::PL_80038788 = Some(|s, k, n| {
            t(0x300 + s as u32);
            t(k as u32);
            t(n as u32)
        });
        h::PL_80038824 = Some(|s, k| {
            t(0x310 + s as u32);
            t(k as u32)
        });
        h::PL_80038898 = Some(|s, k| {
            t(0x320 + s as u32);
            t(k as u32)
        });
        h::PL_80039418 = Some(|s, k| {
            t(0x330 + s as u32);
            t(k as u32);
            (tr() % 5 == 0) as u32
        });
        h::PL_800386D8 = Some(|st, n| {
            t(0x340);
            t(st.attacks.total);
            t(n as u32);
            tr() % 1000
        });
        h::FT_80087A8C = Some(|g| {
            t(0x350 + e(g));
            ((tr() % 2001) as i32 - 1000) as f32 / 7.0
        });
        h::FT_80087A98 = Some(|g| {
            t(0x360 + e(g));
            (tr() % 4) as i32
        });
        h::FT_80087AA4 = Some(|g| {
            t(0x370 + e(g));
            (tr() >> 2) & 1 != 0
        });
        h::FT_80087AB4 = Some(|g| {
            t(0x380 + e(g));
            (tr() % 3) as u8
        });
        h::FT_800898B4 = Some(|g| {
            t(0x390 + e(g));
            KB.kb_applied1 = (tr() % 400) as i32 as f32 / 2.0;
            &raw mut KB
        });
        h::GM_8016F2F8 = Some(|k, s| {
            t(0x3A0);
            t(k as u32);
            t(s as u32);
            (tr() % 50) as i32
        });
        h::GM_8016B6E8 = Some(|s, v| {
            t(0x3B0 + s as u32);
            t(v as u32)
        });
        // round 10
        h::IT_GET_KIND = Some(|g| {
            t(0x4C0 + e(g));
            IKINDS[(tr() % 12) as usize]
        });
        h::IT_8026B7E8 = Some(|g| {
            t(0x420 + e(g));
            (tr() % 3 == 0) as i32
        });
        h::IT_8026B774 = Some(|g, a| {
            t(0x410 + e(g));
            t(a as u32);
            (tr() >> 3) & 1 != 0
        });
        h::IT_GET_OWNER = Some(|g| {
            t(0x430 + e(g));
            ent3(tr() % 3)
        });
        h::FTLIB_80086960 = Some(|g| {
            t(0x440 + e(g));
            (tr() >> 1) & 1 != 0
        });
        h::FTLIB_80086BE0 = Some(|g| {
            t(0x450 + e(g));
            (tr() % 6) as u8
        });
        h::FTLIB_800874BC = Some(|g| {
            t(0x460 + e(g));
            ((tr() >> 4) & 1) as i32
        });
        h::IT_8026B7BC = Some(|g| {
            t(0x470 + e(g));
            ((tr() >> 2) & 1) as i32
        });
        h::IT_8026B960 = Some(|g| {
            t(0x480 + e(g));
            ((tr() % 2001) as i32 - 1000) as f32 / 7.0
        });
        h::IT_8026B7CC = Some(|g| {
            t(0x490 + e(g));
            (tr() % 8) as i32
        });
        h::IT_8026C1D4 = Some(|| {
            t(0x4A0);
            tr() % 4
        });
        h::FT_800898A8 = Some(|g| {
            t(0x4B0 + e(g));
            &raw mut S2074
        });
    }
}

fn init_slot(slot: usize) {
    let x = unsafe { &mut *get_stale_move_table_index_ptr2(slot as i32) };
    unsafe { core::ptr::write_bytes(x as *mut pl_StaleMoveTableExt_t as *mut u8, 0, 0xDD4) };
    let p = unsafe { &mut PLAYER_SLOTS[slot] };
    p.player_state = (rnd() % 4) as i32;
    p.team = (rnd() % 3) as u8;
    p.transformed[0] = (rnd() % 2) as u8;
    p.transformed[1] = (rnd() % 2) as u8;
    p.player_entity[0] = rent();
    p.player_entity[1] = rent();
    let s = &mut x.x0_staleMoveTable;
    s.x5EC.x0 = (rnd() % 6) as u8;
    s.x5EC.x4 = rf();
    s.x5EC.x8 = rnd() % 4;
    s.x5EC.xC = rnd() % 4;
    for i in 0..6 {
        let q = &mut s.x5EC.x10[i];
        q.x0 = rf();
        q.x4 = (rnd() % 3) as i16;
        q.x8 = (rnd() % 5) as i16;
        q.xC = if rnd() % 3 == 0 { 0x5A } else { (rnd() % 0x62) as i32 };
        q.x10 = (rnd() % 6) as i16;
        q.set_x12_b0((rnd() & 1) as u16);
    }
    s.x8C0.x0 = (rnd() % 5) as i32;
    for i in 0..5 {
        let q = &mut s.x8C0.x4[i];
        q.x0 = (rnd() % 8) as u16;
        q.x2 = (rnd() % 7) as u8;
        q.set_x3_b0((rnd() & 1) as u8);
    }
    s.x8D8.x0 = (rnd() % 5) as i32;
    for i in 0..5 {
        let q = &mut s.x8D8.x4[i];
        q.x0 = rnd() % 8;
        q.set_x4_b0((rnd() & 1) as u8);
        q.set_x4_b1((rnd() & 1) as u8);
    }
    s.xC60 = rf();
    s.xC64 = rf();
    s.xC68 = rnd() as i32;
    s.xC6C = rf();
    s.xC90 = rnd() % 5;
    s.xCAC = rnd();
    s.xCB0 = rnd() as i32;
    s.xCB4 = rnd() as i32;
    s.xCB8 = (rnd() % 7) as i32;
    s.xCBC.x3 = (0x30 + rnd() % 16) as u8;
    s.xCC0.set_x10_b0((rnd() & 1) as u8);
    s.xCDC = rf();
    s.xCE0 = rf();
    s.xCE4 = rnd();
    s.xCE8 = rnd();
    s.xCEC = z() as i32;
    s.xCF0 = rnd() as i32;
    x.xCF8 = rf();
    x.xCFC = rnd() as i32;
    x.xD00 = rnd() as i32;
    x.xD04 = rnd();
    x.xD20 = rnd();
    x.xD24 = rnd();
    x.xD40 = rnd();
    x.xD48 = rnd();
    x.xD4C = z();
    x.xD50 = z();
    x.xD54 = z();
    x.xD60 = rnd();
    x.xD64 = if rnd() % 2 != 0 { 6 } else { rnd() as i32 };
    x.xD68 = rnd() as i32;
    x.xD70 = rnd() as i32;
    x.xD74 = rf();
    x.xD78 = rf();
    x.xD7C = rf();
    x.xD80 = rf();
    x.xD84 = rf();
    x.xD88 = rf();
    unsafe {
        x.xDD0.anon_0.set_bit0((rnd() & 1) as u8);
        x.xDD0.anon_0.set_bit5((rnd() & 1) as u8);
        x.xDD1.anon_0.set_bit4((rnd() & 1) as u8);
        x.xDD1.anon_0.set_bit6((rnd() & 1) as u8);
    }
    let a = &mut x.x0_staleMoveTable.total_attack_count_struct;
    a.attacks.total = z();
    a.attacks.aerials_count = rnd();
    a.attacks.thrown_item_count = rnd();
    a.attacks.specials_count = rnd();
    for i in 30..70 {
        a.attacks.by_attack_counts[i] = rnd();
    }
    a.anon_358.x358_hits.total = rnd();
    let s = &mut x.x0_staleMoveTable;
    for i in 0..39 {
        s.x674[i] = rnd() % 4;
        s.x710[i] = (rnd() % 4) as i32;
        s.x7AC[i] = (rnd() % 4) as i32;
    }
    s.xCD8 = rnd() as i32;
    s.xCA8 = rnd() % 4;
    s.xC70 = rf();
    s.xC74 = rf();
    for i in 0..6 {
        s.xC78[i] = rf();
    }
    s.xC98 = rf();
    s.xCA0 = rf();
    s.xCA4 = (rnd() % 100) as i32;
    x.xD08 = rnd();
    x.xD10 = rnd() as i32;
    x.xD44 = rnd();
    x.xD6C = if rnd() % 2 != 0 { 6 } else { (rnd() % 6) as i32 };
}

static mut H: u32 = 0;
fn hh(v: u32) {
    unsafe { H = H.wrapping_mul(0x01000193) ^ v }
}
fn hash_slot(slot: i32) {
    let x = unsafe { &*get_stale_move_table_index_ptr2(slot) };
    let s = &x.x0_staleMoveTable;
    hh(s.x5EC.x0 as u32);
    hh(b(s.x5EC.x4));
    hh(s.x5EC.x8);
    hh(s.x5EC.xC);
    for q in s.x5EC.x10.iter() {
        hh(b(q.x0));
        hh(q.x4 as i32 as u32);
        hh(q.x8 as i32 as u32);
        hh(q.xC as u32);
        hh(q.x10 as i32 as u32);
        hh(q.x12_b0() as u32);
    }
    hh(s.x8C0.x0 as u32);
    for q in s.x8C0.x4.iter() {
        hh(q.x0 as u32);
        hh(q.x2 as u32);
        hh(q.x3_b0() as u32);
    }
    hh(s.x8D8.x0 as u32);
    for q in s.x8D8.x4.iter() {
        hh(q.x0);
        hh(q.x4_b0() as u32);
        hh(q.x4_b1() as u32);
    }
    for v in [s.xC68 as u32, s.xC90, s.xCAC, s.xCB0 as u32, b(s.xCDC), b(s.xCE0), s.xCE4, s.xCE8, s.xCEC as u32,
        s.xCF0 as u32, b(x.xCF8), x.xD4C, x.xD50, x.xD60, x.xD70 as u32]
    {
        hh(v);
    }
    unsafe {
        hh(x.xDD0.anon_0.bit0() as u32);
        hh(x.xDD0.anon_0.bit5() as u32);
        hh(x.xDD1.anon_0.bit4() as u32);
        hh(x.xDD1.anon_0.bit6() as u32);
    }
    for i in 0..39 {
        hh(s.x674[i]);
        hh(s.x710[i] as u32);
        hh(s.x7AC[i] as u32);
    }
    for v in [s.xCD8 as u32, s.xCA8, b(s.xC6C), b(s.xC70), b(s.xC74)] {
        hh(v);
    }
    for f in s.xC78 {
        hh(b(f));
    }
    for v in [b(s.xC98), b(s.xC60), b(s.xC64), b(s.xCA0), s.xCA4 as u32, x.xD08, x.xD10 as u32, x.xD44, x.xD54, x.xD6C as u32] {
        hh(v);
    }
}

const ITEMS: [i32; 14] = [
    It_Kind_Old_Kuri as i32, It_Kind_Kuriboh as i32, It_Kind_Heiho as i32, It_Kind_Nokonoko as i32,
    It_Kind_Patapata as i32, It_Kind_Likelike as i32, It_Kind_Old_Lead as i32, It_Kind_Leadead as i32,
    It_Kind_Old_Octa as i32, It_Kind_Octarock as i32, It_Kind_Old_Otto as i32, It_Kind_Ottosea as i32,
    It_Kind_Whitebea as i32, 0,
];
const NOPS: u32 = 104;

#[test]
fn plbonus_matches_c() {
    install();
    let want = include_str!("data/plbonus.txt");
    let mut lines = want.lines();
    let mut seen = [false; NOPS as usize];
    let mut n = 0;
    for k in 0..300u32 {
        unsafe {
            RS = 0xB0B05u32.wrapping_add(k.wrapping_mul(2654435761));
            TR = 0;
        }
        for i in 0..6 {
            init_slot(i);
        }
        unsafe {
            TAB.x4 = (rnd() % 6) as i32;
            TAB.x17C = (rnd() % 5) as i32;
            TAB.xD4 = rf();
            TAB.x38 = rf();
            TAB.x98 = rf();
            TAB.xA0 = rf();
            TAB.xA4 = rnd();
            TAB.xB0 = rnd();
            TAB.x124 = rnd();
            TAB.x4C = rf();
            TAB.xEC = rnd() as i32;
            TAB.x84 = rf();
            TAB.x138 = rnd() % 4;
            TAB.xB4 = rnd();
            TAB.x180 = rf();
            S2074.x207C.y = 0;
        }
        for step in 0..50 {
            let slot = (rnd() % 6) as i32;
            let op = rnd() % NOPS;
            let a = (rnd() % 6) as i32;
            let bb = (rnd() % 3) as i32;
            let c = rnd() as i32;
            let w = (rnd() % 0x62) as i32;
            let z7 = (rnd() % 7) as i32;
            let it = ITEMS[(rnd() % 14) as usize];
            let d = rf();
            let ee = rf();
            let w8 = (w % 8) as u16;
            let q = if rnd() % 3 == 0 { 0x59 } else { w % 20 };
            let a5 = match w % 4 { 0 => 0x34, 1 => 0x5C, 2 => 0x5D, _ => w };
            let gi = &raw mut GI;
            let mut ret: u32 = 0;
            seen[op as usize] = true;
            match op {
                0 => pb::pl_80040ddc(slot),
                1 => pb::pl_80040ed4(slot, c, a, bb, w, d),
                2 => pb::pl_80040fbc(slot, c, a, bb, w),
                3 => pb::pl_800410f4(slot, c, a, bb, w),
                4 => pb::pl_800411c4(slot, bb),
                5 => pb::pl_80041280(slot, bb),
                6 => ret = pb::pl_800412d0(slot) as u32,
                7 => ret = pb::pl_80041300(slot) as u32,
                8 => ret = pb::pl_8004134c(slot) as u32,
                9 => pb::fn_8004138c(slot, bb),
                10 => ret = b(pb::pl_800414c0(slot)),
                11 => pb::pl_80041524(slot),
                12 => pb::fn_800415b0(slot, c, w8, (bb & 1) as u32),
                13 => pb::pl_80041720(slot, c, w8),
                14 => pb::pl_80041744(slot, bb, w8),
                15 => pb::fn_80041770(slot, (w % 8) as u8, bb as u32),
                16 => pb::pl_8004182c(slot),
                17 => pb::pl_800418f4(slot),
                18 => pb::pl_800419ac(slot, c, w8),
                19 => pb::pl_80041b08(slot, core::ptr::null_mut(), w8),
                20 => pb::fn_80041bc8(slot, (bb & 1) as u32, (a & 1) as u32),
                21 => pb::pl_80041bfc(slot),
                22 => ret = pb::pl_8003fbfc(slot) as u32,
                23 => ret = pb::pl_8003fc20(slot) as u32,
                24 => pb::pl_8003fc44(slot, bb),
                25 => pb::pl_8003fc88(slot, bb, w % 50),
                26 => pb::pl_8003fda0(slot, c),
                27 => pb::pl_8003fdc8(slot, c),
                28 => pb::pl_8003fdf4(slot, c),
                29 => pb::pl_8003fe1c(slot, c),
                30 => pb::pl_8003fe40(slot, c),
                31 => pb::pl_8003fe64(slot, c),
                32 => pb::pl_8003fed0(slot, bb),
                33 => pb::pl_8003ff44(slot, bb & 1, w % 20),
                34 => pb::pl_8003ffdc(slot, c, a, bb & 1, w % 4),
                35 => pb::pl_80040048(slot, bb & 1),
                36 => pb::pl_80040120(slot, bb),
                37 => pb::pl_800401f0(slot, c, d, ee),
                38 => pb::pl_80040270(slot, bb, d),
                39 => pb::pl_800402d0(slot, bb, a & 1),
                40 => pb::pl_80040330(slot, bb, d),
                41 => pb::pl_80040374(slot, bb),
                42 => pb::pl_800403c0(slot, c),
                43 => pb::pl_800403fc(c, bb, slot, bb, 0x50 + w % 10),
                44 => pb::pl_80040460(slot, bb),
                45 => pb::pl_8004049c(z7, it as ItemKind),
                46 => pb::pl_80040614(slot, bb != 0, d),
                47 => pb::pl_8004065c(slot, bb),
                48 => pb::pl_80040688(slot, z7, bb),
                49 => ret = b(pb::pl_800407c8(slot)),
                50 => ret = b(pb::pl_80040870(slot)),
                51 => ret = pb::pl_80040894(slot),
                52 => ret = pb::pl_800408b8(slot),
                53 => ret = pb::pl_800408dc(slot) as u32,
                54 => ret = pb::pl_80040900(slot) as u32,
                55 => ret = pb::pl_80040924(slot) as u32,
                56 => ret = b(pb::pl_80040948(slot)),
                57 => ret = pb::pl_80040a04(slot) as u32,
                58 => ret = pb::pl_80040a30(slot) as u32,
                59 => ret = pb::pl_80040a54(slot) as u32,
                60 => ret = pb::pl_80040a78(slot) as u32,
                61 => ret = pb::pl_80040a9c(slot) as u32,
                62 => ret = pb::pl_80040af0(slot),
                63 => ret = pb::pl_80040b18(slot) as u32,
                64 => ret = pb::pl_80040b3c(slot),
                65 => ret = pb::pl_80040b64(slot),
                66 => pb::pl_80040b8c(slot, bb, c),
                67 => ret = pb::pl_80040bd8(slot) as u32,
                68 => ret = pb::pl_80040bfc(slot),
                69 => ret = b(pb::pl_80040c24(slot)),
                70 => ret = b(pb::pl_80040c48(slot)),
                71 => ret = b(pb::pl_80040c6c(slot)),
                72 => ret = b(pb::pl_80040c90(slot)),
                73 => ret = pb::pl_80040cb4(slot) as u32,
                74 => ret = pb::pl_80040cd8(slot) as u32,
                75 => ret = pb::pl_80040cfc(slot),
                76 => ret = pb::pl_80040d20(slot) as u32,
                77 => ret = b(pb::pl_80040d44(slot)),
                78 => ret = b(pb::pl_80040d68(slot)),
                79 => pb::pl_80040d8c(slot, c),
                80 => ret = pb::pl_80040db8(slot) as u32,
                81 => p2::pl_8003df44(slot, bb & 1),
                82 => p2::pl_8003dff4(slot, bb & 1, q),
                83 => p2::pl_8003e058(z7, c, a, bb),
                84 => p2::pl_8003e0e8(slot, bb),
                85 => p2::pl_8003e114(slot, c, d),
                86 => p2::pl_8003e150(slot, c),
                87 => p2::pl_8003e17c(slot, c, gi),
                88 => ret = p2::pl_8003e2cc(slot, (w % 39) as u32),
                89 => ret = p2::pl_8003e334(slot, (w % 39) as u32),
                90 => ret = p2::pl_8003e39c(slot) as u32,
                91 => ret = p2::pl_8003e420(slot),
                92 => {
                    let mut moves = [0i32; 6];
                    for m in moves.iter_mut() {
                        *m = MKINDS[(rnd() % 10) as usize];
                    }
                    p2::pl_8003e4a4(slot, bb & 1 != 0, &moves[..a as usize]);
                }
                93 => p2::pl_8003e854(slot, bb & 1, gi),
                94 => p2::pl_8003e978(slot, bb & 1 != 0, gi),
                95 => p2::fn_8003e998(slot, bb & 1),
                96 => p2::pl_8003ea08(slot, bb),
                97 => p2::pl_8003ea40(slot, c, a, bb & 1, z7),
                98 => p2::pl_8003ea74(slot, bb),
                99 => p2::pl_8003eaac(slot, bb & 1, q),
                100 => p2::pl_8003eb30(d, z7, c, if bb == 2 { 6 } else { a }, bb & 1, a5),
                101 => p2::pl_8003ec30(slot, bb, w % 4, d),
                102 => p2::pl_8003ec9c(slot, bb, d, ee),
                103 => p2::pl_8003ed0c(slot, c, a, bb, d),
                _ => unreachable!(),
            }
            unsafe { H = 0x811C9DC5 };
            for i in 0..6 {
                hash_slot(i);
            }
            hh(unsafe { S2074.x207C.y } as u32);
            let got = format!("{} {} {} {:x} {:x} {:x}", k, step, op, ret, tr(), unsafe { H });
            assert_eq!(Some(got.as_str()), lines.next(), "case {k} step {step} op {op}");
            n += 1;
        }
    }
    assert_eq!(lines.next(), None);
    assert!(seen.iter().all(|&s| s), "every op exercised");
    assert_eq!(n, 15000);
}
