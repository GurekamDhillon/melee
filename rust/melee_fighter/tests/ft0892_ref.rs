//! ft0892.rs + pltrick.rs vs src/melee/ft/ft_0892.c and pltrick.c's attack-stats functions
//! (gen/cref/ft0892_ref.c, built with the shipping build's contraction + gwtool lowering;
//! expected values in tests/data/ft0892.txt; regenerate with
//! `python rust/melee_fighter/gen/cref/run.py ft0892`). One fighter, random fields and action
//! stats, one op per case. Fill order and the stubbed callees (trace word `TR`) match the C.
//! The two Struct2070 int literals of ft_800895E0 hash as tags A1/A2 on both sides (the C lays
//! them out little-endian, the port big-endian as the game).
#![allow(static_mut_refs, non_upper_case_globals)]

use melee_fighter::ft0892 as m;
use melee_fighter::hooks as h;
use melee_fighter::pltrick as p;
use melee_fighter::stale;
use melee_fighter::types::*;
use std::collections::BTreeSet;

static mut TR: u32 = 0;
fn t(v: u32) {
    unsafe { TR = TR.wrapping_mul(31).wrapping_add(v) }
}
fn tr() -> u32 {
    unsafe { TR }
}
static mut FP: *mut Fighter = core::ptr::null_mut();
static mut ITP: *mut HSD_GObj = core::ptr::null_mut();

struct Rng(u32);
impl Rng {
    fn rnd(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        self.0
    }
    fn b(&mut self) -> u8 {
        (self.rnd() & 1) as u8
    }
}

fn install_hooks() {
    unsafe {
        h::IT_8026B6C8 = Some(|g| {
            t(0x500 + (g == ITP) as u32);
            ((tr() >> 3) % 3) as i32
        });
        h::FTFX_APPEALS_CHECK_IF_USED = Some(|fp| {
            t(0x510 + core::ptr::eq(fp, FP) as u32);
            (tr() >> 4) & 1 != 0
        });
        h::PL_8003FE40 = Some(|a, b| {
            t(0x520);
            t(a as u32);
            t(b as u32);
        });
        h::PL_8003DFF4 = Some(|a, b, c| {
            t(0x530);
            t(a as u32);
            t(b as u32);
            t(c as u32);
        });
        h::FTDATA_80085FD4_X8 = Some(|fp, ms| {
            t(0x540 + core::ptr::eq(fp, FP) as u32);
            t(ms as u32);
            ((tr() >> 5) & 1) as usize
        });
        h::GENO_DEFINE_TEMPLATE_SKELETON = Some(|k| {
            t(0x550u32.wrapping_add(k as u32));
            ((tr() >> 2) & 1) as i32
        });
        h::GENO_DEFINE_BASE_KIND = Some(|k| {
            t(0x560u32.wrapping_add(k as u32));
            if (tr() >> 6) & 1 != 0 { 1 } else { 22 }
        });
    }
}

const KINDS: [u32; 8] = [1, 22, 16, 17, 33, 34, 2, 9];
const NOPS: u32 = 19;

fn r2070(rg: &mut Rng, u: &mut Struct2070) {
    let e = unsafe { &mut u.anon_0 };
    e.x2070 = (1 + rg.rnd() % 0x30) as i8;
    e.set_x2071_b0_3((rg.rnd() & 15) as u8);
    e.set_x2071_b4(rg.b());
    e.set_x2071_b5(rg.b());
    e.set_x2071_b6(rg.b());
    e.set_x2071_b7(rg.b());
    e.set_x2072_b0(rg.b());
    e.set_x2072_b1(rg.b());
    e.set_x2072_b2(rg.b());
    e.set_count_x1A4(rg.b());
    e.set_count_thrown_items(rg.b());
    e.set_count_aerials(rg.b());
    e.set_count_x1A0(rg.b());
    e.set_count_specials(rg.b());
    e.x2073 = match rg.rnd() % 6 {
        0 => 0,
        1 => (0x60 + rg.rnd() % 0x14) as u8,
        2 => (100 + rg.rnd() % 16) as u8,
        _ => (rg.rnd() % 100) as u8,
    };
}
fn r18d8(rg: &mut Rng, e: &mut ft_800898B4_t) {
    e.x0 = rg.rnd() as i32;
    e.x4 = rg.rnd() as i32;
    e.kb_applied1 = (rg.rnd() % 2001) as i32 as f32 / 7.0;
    e.xC = (rg.rnd() % 20) as i32;
    e.set_x10_b0(rg.b());
    e.set_x10_b1(rg.b());
    e.set_x10_b2(rg.b());
    e.set_x10_b3(rg.b());
    e.set_x10_b4(rg.b());
    e.set_x10_b5(rg.b());
    e.set_x10_b6(rg.b());
    e.set_x10_b7(rg.b());
    e.set_x11_b0(rg.b());
    e.set_x11_b1(rg.b());
    e.set_x11_b2(rg.b());
    e.set_x11_b3(rg.b());
    e.set_x11_b4(rg.b());
}
fn mix(t: &mut u32, v: u32) {
    *t = t.wrapping_mul(31).wrapping_add(v);
}
fn h2070(t: &mut u32, u: &Struct2070) {
    let e = unsafe { &u.anon_0 };
    for v in [
        e.x2070 as u8, e.x2071_b0_3(), e.x2071_b4(), e.x2071_b5(), e.x2071_b6(), e.x2071_b7(), e.x2072_b0(),
        e.x2072_b1(), e.x2072_b2(), e.count_x1A4(), e.count_thrown_items(), e.count_aerials(), e.count_x1A0(),
        e.count_specials(), e.x2073,
    ] {
        mix(t, v as u32);
    }
}
fn h18d8(t: &mut u32, e: &ft_800898B4_t) {
    mix(t, e.x0 as u32);
    mix(t, e.x4 as u32);
    mix(t, e.kb_applied1.to_bits());
    mix(t, e.xC as u32);
    for v in [
        e.x10_b0(), e.x10_b1(), e.x10_b2(), e.x10_b3(), e.x10_b4(), e.x10_b5(), e.x10_b6(), e.x10_b7(), e.x11_b0(),
        e.x11_b1(), e.x11_b2(), e.x11_b3(), e.x11_b4(),
    ] {
        mix(t, v as u32);
    }
}
fn hatk(t: &mut u32, a: &plAttackStats) {
    mix(t, a.total);
    for v in a.by_attack_counts {
        mix(t, v);
    }
    for v in [a.thrown_item_count, a.aerials_count, a.specials_count, a.x1A0_count, a.x1A4_count, a.x1A8] {
        mix(t, v);
    }
}
fn ratk(rg: &mut Rng, a: &mut plAttackStats) {
    a.total = rg.rnd() % 50;
    for v in a.by_attack_counts.iter_mut() {
        *v = rg.rnd() % 4;
    }
    a.thrown_item_count = rg.rnd() % 9;
    a.aerials_count = rg.rnd() % 9;
    a.specials_count = rg.rnd() % 9;
    a.x1A0_count = rg.rnd() % 9;
    a.x1A4_count = rg.rnd() % 9;
    a.x1A8 = rg.rnd() % 9;
}
fn hstats(t: &mut u32, s: &plActionStats) {
    hatk(t, &s.attacks);
    hatk(t, &s.hits);
    hatk(t, unsafe { &s.anon_358.x358_hits });
    for v in s.x504 {
        mix(t, v as u32);
    }
    for v in [s.x568, s.x56C, s.x570, s.x574, s.x578, s.x57C, s.x580, s.x584, s.x588, s.x58C, s.x590, s.x594] {
        mix(t, v);
    }
    for v in s.x598 {
        mix(t, v);
    }
    for v in [s.x5BC_b0(), s.x5BC_b1(), s.x5BC_b2(), s.x5BC_b3()] {
        mix(t, v as u32);
    }
}

#[test]
fn ft0892_matches_c() {
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/ft0892.txt")).unwrap();
    let mut lines = text.lines();
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut go = Box::new(HSD_GObj::zeroed());
    let mut it = Box::new(HSD_GObj::zeroed());
    unsafe {
        FP = &mut *f;
        ITP = &mut *it;
    }
    install_hooks();
    let mut seen = BTreeSet::new();
    let mut checked = 0;
    for op in 0..NOPS {
        for k in 0..300u32 {
            *f = Fighter::zeroed();
            *go = HSD_GObj::zeroed();
            *it = HSD_GObj::zeroed();
            unsafe { stale::PLAYER_SLOTS = core::mem::zeroed() };
            let mut rg = Rng(0x2468Bu32.wrapping_add(op.wrapping_mul(7919)).wrapping_add(k.wrapping_mul(104729)));
            unsafe { TR = rg.rnd() };
            let c = if rg.rnd() % 4 == 0 { 0xFFFF } else { rg.rnd() as u16 };
            unsafe { stale::ATTACK_COUNTER = c };
            let fpp: *mut Fighter = &mut *f;
            let g: *mut HSD_GObj = &mut *go;
            let itp: *mut HSD_GObj = &mut *it;
            go.user_data = fpp as *mut _;
            f.gobj = g;
            f.kind = KINDS[(rg.rnd() % 8) as usize];
            f.player_id = (rg.rnd() % 6) as _;
            f.set_is_sub_fighter(rg.b());
            f.set_is_metal(rg.b());
            f.set_x221D_b6(rg.b() as _);
            f.set_x2226_b4(rg.b());
            f.set_x2220_b5(rg.b());
            f.set_x2220_b6(rg.b());
            f.x197C = if rg.rnd() & 1 != 0 { itp } else { core::ptr::null_mut() };
            f.item_gobj = if rg.rnd() & 1 != 0 { itp } else { core::ptr::null_mut() };
            f.motion_id = (rg.rnd() % 400) as _;
            f.dmg.x1830_percent = (rg.rnd() % 4001) as i32 as f32 / 13.0;
            r2070(&mut rg, &mut f.x2070);
            f.x2074.x2074_vec.x = f32::from_bits(rg.rnd());
            f.x2074.x2074_vec.y = f32::from_bits(rg.rnd());
            f.x2074.x207C.x = rg.rnd() as i32;
            f.x2074.x207C.y = rg.rnd() as i32;
            {
                let b = unsafe { &mut f.x2074.anon_10.anon_0 };
                b.set_x2084_b0(rg.b());
                b.set_x2084_b1(rg.b());
                b.set_x2084_b2(rg.b());
                b.set_x2084_b3(rg.b());
                b.set_x2084_b4(rg.b());
                b.set_x2084_b5(rg.b());
                b.set_x2084_b6(rg.b());
                b.set_x2084_b7(rg.b());
                b.set_x2085_b0(rg.b());
                b.set_x2085_b1(rg.b());
                b.set_x2085_b2(rg.b());
                b.set_x2085_b3(rg.b());
                b.set_x2085_b4(rg.b());
                b.set_x2085_b5(rg.b());
                b.set_x2085_b6(rg.b());
                b.set_x2085_b7(rg.b());
            }
            f.x2074.x2088 = rg.rnd() as u16;
            f.dmg.x18d4.x0 = rg.rnd() as u8;
            f.dmg.x18d4.x1 = rg.rnd() as u8;
            f.dmg.x18d4.set_x2_b0(rg.b());
            f.dmg.x18d4.set_x2_b7(rg.b());
            f.dmg.x18d4.x3 = rg.rnd() as u8;
            r18d8(&mut rg, &mut f.dmg.x18d8);
            f.dmg.x18ec_instancehitby = rg.rnd() as u16;
            let st: *mut plActionStats = stale::action_stats(f.player_id as i32);
            let st = unsafe { &mut *st };
            ratk(&mut rg, &mut st.attacks);
            ratk(&mut rg, &mut st.hits);
            ratk(&mut rg, unsafe { &mut st.anon_358.x358_hits });
            for v in st.x504.iter_mut() {
                *v = rg.rnd() as u8;
            }
            st.set_x5BC_b0(rg.b());
            st.set_x5BC_b1(rg.b());
            st.set_x5BC_b2(rg.b());
            st.set_x5BC_b3(rg.b());
            let mut arg = Struct2070::zeroed();
            r2070(&mut rg, &mut arg);
            if rg.rnd() % 3 == 0 {
                arg.anon_0.x2073 = unsafe { f.x2070.anon_0.x2073 };
            }
            if rg.rnd() % 4 == 0 {
                arg.anon_0.x2073 = [0x71, 0x62, 0x63][(rg.rnd() % 3) as usize];
            }
            let mut evd = ft_800898B4_t::zeroed();
            r18d8(&mut rg, &mut evd);
            let fr = unsafe { &mut *fpp };
            let ret: u32 = match op {
                0 => { m::ft_800892d4(fr); 0 }
                1 => { m::ft_80089460(fr); 0 }
                2 => { m::ft_800895e0(fr, arg); 0 }
                3 => { m::ft_80089768(&mut fr.x2074); 0 }
                4 => { m::ft_80089824(g); 0 }
                5 => (m::ft_80089884(g) == &mut fr.x2070 as *mut _) as u32,
                6 => (m::ft_80089890(g) == &mut fr.dmg.x18d4 as *mut _) as u32,
                7 => m::ft_8008989c(g) as u32,
                8 => (m::ft_800898a8(g) == &mut fr.x2074 as *mut _) as u32,
                9 => (m::ft_800898b4(g) == &mut fr.dmg.x18d8 as *mut _) as u32,
                10 => m::ft_800898c0(g) as u32,
                11 => {
                    let ms = 45 + (rg.rnd() % 22) as i32;
                    m::ft_80089914(g, ms) as u32
                }
                12 => {
                    let hp = (rg.rnd() % 8) as i32;
                    let idx = (rg.rnd() % 100) as i32;
                    p::pl_80037b2c(st, hp, idx) as u32
                }
                13 => {
                    let a = if rg.rnd() & 1 != 0 { &mut st.attacks } else { &mut st.hits };
                    p::pl_80037bc0(a, &arg);
                    0
                }
                14 => { p::pl_80037c60(g, arg); 0 }
                15 => { p::pl_80037df4(g, &arg); 0 }
                16 => { p::pl_80037ecc(g); 0 }
                17 => { p::pl_800384dc(g, arg, &mut evd); 0 }
                18 => {
                    let kd = 1 + (rg.rnd() % 16) as i32;
                    p::pl_80038628(g, kd) as u32
                }
                _ => unreachable!(),
            };
            let f = unsafe { &*fpp };
            let mut th: u32 = 0;
            match m::s2070_gc_int(&f.x2070) {
                0x240063 => mix(&mut th, 0xA1),
                0x44003D => mix(&mut th, 0xA2),
                _ => h2070(&mut th, &f.x2070),
            }
            mix(&mut th, f.x2074.x2074_vec.x.to_bits());
            mix(&mut th, f.x2074.x2074_vec.y.to_bits());
            mix(&mut th, f.x2074.x207C.x as u32);
            mix(&mut th, f.x2074.x207C.y as u32);
            {
                let b = unsafe { &f.x2074.anon_10.anon_0 };
                for v in [
                    b.x2084_b0(), b.x2084_b1(), b.x2084_b2(), b.x2084_b3(), b.x2084_b4(), b.x2084_b5(), b.x2084_b6(),
                    b.x2084_b7(), b.x2085_b0(), b.x2085_b1(), b.x2085_b2(), b.x2085_b3(), b.x2085_b4(), b.x2085_b5(),
                    b.x2085_b6(), b.x2085_b7(),
                ] {
                    mix(&mut th, v as u32);
                }
            }
            mix(&mut th, f.x2074.x2088 as u32);
            let d = &f.dmg.x18d4;
            for v in [d.x0, d.x1, d.x2_b0(), d.x2_b7(), d.x3] {
                mix(&mut th, v as u32);
            }
            h18d8(&mut th, &f.dmg.x18d8);
            mix(&mut th, f.dmg.x18ec_instancehitby as u32);
            hstats(&mut th, stale::action_stats(f.player_id as i32));
            mix(&mut th, unsafe { stale::ATTACK_COUNTER } as u32);
            let got = format!("{} {} {:08X} {:08X} {:08X}", op, k, ret, th, tr());
            let want = lines.next().expect("expected file too short");
            assert_eq!(got, want, "op {op} case {k}");
            seen.insert(op);
            checked += 1;
        }
    }
    assert!(lines.next().is_none(), "expected file has extra lines");
    assert_eq!(seen.len(), NOPS as usize);
    assert_eq!(checked, 5700);
}
