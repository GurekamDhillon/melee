//! ft0877.rs vs src/melee/ft/ft_0877.c (gen/cref/ft0877_ref.c, built with the shipping build's
//! contraction + gwtool lowering; expected values in tests/data/ft0877.txt; regenerate with
//! `python rust/melee_fighter/gen/cref/run.py ft0877`). Six fighters, a random fighter p-link
//! list, one op per case. Fill order and the stubbed callees (trace word `TR`) match the C.
#![allow(static_mut_refs, non_upper_case_globals)]

use melee_fighter::ft0877 as m;
use melee_fighter::hooks as h;
use melee_fighter::types::*;
use std::collections::BTreeSet;

const NF: usize = 6;
static mut TR: u32 = 0;
fn t(v: u32) {
    unsafe { TR = TR.wrapping_mul(31).wrapping_add(v) }
}
fn tr() -> u32 {
    unsafe { TR }
}

struct World {
    f: std::vec::Vec<Fighter>,
    go: std::vec::Vec<HSD_GObj>,
    it: std::vec::Vec<HSD_GObj>,
    plinks: [*mut HSD_GObj; 64],
}
static mut W: *mut World = core::ptr::null_mut();
fn w() -> &'static mut World {
    unsafe { &mut *W }
}
fn e(p: *mut HSD_GObj) -> u32 {
    if p.is_null() {
        return 0;
    }
    let w = w();
    for i in 0..NF {
        if p == &mut w.go[i] as *mut _ {
            return 1 + i as u32;
        }
    }
    for i in 0..3 {
        if p == &mut w.it[i] as *mut _ {
            return 10 + i as u32;
        }
    }
    99
}
fn fi(fp: *const Fighter) -> u32 {
    (0..NF).find(|&i| core::ptr::eq(fp, &w().f[i])).map_or(9, |i| i as u32)
}

const SFXM: [i32; 25] = [
    407, 467, 503, 515, 290051, 509, 485, 491, 479, 473, 437, 455, 461, 449, 443, 425, 431, 419,
    413, 377, 395, 401, 389, 383, 371,
];
const SFXS: [i32; 13] = [359, 368, 365, 362, 344, 353, 356, 350, 347, 338, 341, 335, 332];

struct Rng(u32);
impl Rng {
    fn rnd(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        self.0
    }
    fn rf(&mut self) -> f32 {
        match self.rnd() % 16 {
            0 => 0.0,
            1 => -0.0,
            2 => 1.0,
            3 => -1.0,
            _ => ((self.rnd() % 2001) as i32 - 1000) as f32 / 7.0,
        }
    }
}

fn install_hooks() {
    unsafe {
        h::PLINK_HEAD = Some(|link| w().plinks[link as usize]);
        h::IT_IS_HEAVY = Some(|g| {
            t(0x300 + e(g));
            (tr() >> 3) & 1 != 0
        });
        h::IT_GET_KIND = Some(|g| {
            t(0x310 + e(g));
            match (tr() >> 2) & 3 {
                0 => 30,
                1 => 7,
                k => k as i32,
            }
        });
        h::FT_GET_PARASOL_STATUS = Some(|g| {
            t(0x320 + e(g));
            3 + ((tr() >> 1) % 4) as i32
        });
        h::FTCO_800C06B4 = Some(|fp| {
            t(0x330 + fi(fp));
            match (tr() >> 4) % 3 {
                0 => 0x7B,
                1 => 0x80,
                _ => 5,
            }
        });
        h::FTCO_800A101C = Some(|fp, a, b, c| {
            t(0x340 + fi(fp));
            t(a as u32);
            t(b as u32);
            t(c as u32);
        });
        h::LBAUDIOAX_800233EC = Some(|id| {
            t(0x350);
            t(id as u32);
            match tr() % 4 {
                0 => id,
                1 => SFXM[((tr() >> 3) % 25) as usize],
                2 => SFXS[((tr() >> 3) % 13) as usize],
                _ => 0x1FBFD + ((tr() >> 3) % 0xD0) as i32,
            }
        });
        h::LBAUDIOAX_80023130 = Some(|s| {
            t(0x360);
            t(s as u32);
            match (tr() >> 5) % 5 {
                0 | 4 => 0,
                1 => 13,
                2 => 6 + ((tr() >> 8) % 28) as i32,
                _ => {
                    if (tr() >> 8) & 1 != 0 {
                        32
                    } else {
                        34
                    }
                }
            }
        });
        h::LBAUDIOAX_800230C8 = Some(|ssm, off, out| {
            t(0x370u32.wrapping_add(ssm as u32));
            *off = (tr() % 0x20000) as i32;
            *out = 1;
            ((tr() >> 9) & 3 == 0) as i32
        });
        h::LBAUDIOAX_80023220 = Some(|ssm| {
            t(0x380u32.wrapping_add(ssm as u32));
            ((tr() >> 3) % 0x10000) as i32
        });
        h::LBAUDIOAX_80024DC4 = Some(|s| {
            t(0x390);
            t(s as u32)
        });
        h::LBAUDIOAX_80024FDC = Some(|| t(0x3A0));
        h::LBAUDIOAX_80024FF4 = Some(|| t(0x3B0));
        h::LBAUDIOAX_8002500C = Some(|n| {
            t(0x3C0);
            t(n as u32)
        });
        h::LBAUDIOAX_80025038 = Some(|n| {
            t(0x3D0);
            t(n as u32)
        });
        h::LBAUDIOAX_800237A8 = Some(|s, v, p| {
            t(0x3E0);
            t(s as u32);
            t(v as u32);
            t(p as u32);
            (tr() >> 4) as i32
        });
        h::LBAUDIOAX_80024B94 = Some(|hh, v| {
            t(0x3F0);
            t(hh as u32);
            t(v as u32);
            0
        });
        h::MEX_SSM_FOR_PORT_KIND = Some(|k| {
            t(0x270u32.wrapping_add(k as u32));
            (tr() % 5) as i32 - 1
        });
        h::GENO_DEFINE_VOICE_PLAY_OWNER = Some(|k, s, _| {
            t(0x400u32.wrapping_add(k as u32));
            t(s as u32);
            ((tr() >> 3) & 1) as i32
        });
        h::HSD_RANDI = Some(|n| {
            t(0x160u32.wrapping_add(n as u32));
            ((tr() >> 7) % n as u32) as i32
        });
    }
}

const KINDS: [u32; 12] = [0, 2, 8, 10, 11, 14, 20, 27, 28, 33, 34, 40];
const NOPS: u32 = 34;

#[test]
fn ft0877_matches_c() {
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/ft0877.txt")).unwrap();
    let mut lines = text.lines();
    let mut world = Box::new(World {
        f: (0..NF).map(|_| Fighter::zeroed()).collect(),
        go: (0..NF).map(|_| HSD_GObj::zeroed()).collect(),
        it: (0..3).map(|_| HSD_GObj::zeroed()).collect(),
        plinks: [core::ptr::null_mut(); 64],
    });
    unsafe { W = &mut *world };
    install_hooks();
    let w = w();
    let mut seen = BTreeSet::new();
    let mut checked = 0;
    for op in 0..NOPS {
        for k in 0..300u32 {
            for i in 0..NF {
                w.f[i] = Fighter::zeroed();
                w.go[i] = HSD_GObj::zeroed();
            }
            for i in 0..3 {
                w.it[i] = HSD_GObj::zeroed();
            }
            w.plinks = [core::ptr::null_mut(); 64];
            let mut rg = Rng(0x2468Au32.wrapping_add(op.wrapping_mul(7919)).wrapping_add(k.wrapping_mul(104729)));
            unsafe { TR = rg.rnd() };
            for i in 0..NF {
                let fpp: *mut Fighter = &mut w.f[i];
                w.go[i].user_data = fpp as *mut _;
                let gp: *mut HSD_GObj = &mut w.go[i];
                let itp: *mut HSD_GObj = w.it.as_mut_ptr();
                let f = &mut w.f[i];
                f.gobj = gp;
                f.kind = KINDS[(rg.rnd() % 12) as usize];
                f.costume_id = (rg.rnd() % 4) as u8;
                f.input.held_buttons[0] = rg.rnd();
                f.input.pressed_buttons = rg.rnd();
                f.x1968_jumpsUsed = (rg.rnd() % 4) as u8;
                f.motion_id = if rg.rnd() % 3 != 0 { 0 } else { (rg.rnd() % 340) as i32 };
                f.item_gobj = if rg.rnd() & 1 != 0 { unsafe { itp.add((rg.rnd() % 3) as usize) } } else { core::ptr::null_mut() };
                f.x197C = if rg.rnd() % 4 == 0 { unsafe { itp.add((rg.rnd() % 3) as usize) } } else { core::ptr::null_mut() };
                f.x1980 = if rg.rnd() & 1 != 0 { unsafe { itp.add((rg.rnd() % 3) as usize) } } else { core::ptr::null_mut() };
                f.smash_attrs.x2134_vibrateFrame = rg.rnd() as u8;
                f.smash_attrs.x2138_smashSinceHitbox = rg.rf();
                f.x213C = rg.rnd() as i32;
                f.x34_scale.x = rg.rf();
                f.x34_scale.y = if rg.rnd() & 1 != 0 { f.x34_scale.x } else { rg.rf() };
                f.self_vel.x = rg.rf();
                f.self_vel.y = rg.rf();
                f.x8c_kb_vel.x = rg.rf();
                f.x8c_kb_vel.y = rg.rf();
                f.x98_atk_shield_kb.x = rg.rf();
                f.x98_atk_shield_kb.y = rg.rf();
                match rg.rnd() % 4 {
                    1 => {
                        f.x8c_kb_vel.x = f.self_vel.y;
                        f.x8c_kb_vel.y = f.self_vel.x;
                    }
                    2 => {
                        f.x98_atk_shield_kb.x = f.self_vel.y;
                        f.x98_atk_shield_kb.y = f.self_vel.x;
                    }
                    3 => {
                        f.x98_atk_shield_kb.x = f.x8c_kb_vel.y;
                        f.x98_atk_shield_kb.y = f.x8c_kb_vel.x;
                    }
                    _ => {}
                }
                f.cpu.kind = rg.rnd() % 30;
                f.cpu.level = (rg.rnd() % 10) as i32;
                f.cpu.x14 = rg.rnd() as i32;
                f.x1988 = rg.rnd() as i32;
                f.x2164 = (rg.rnd() % 100) as i32;
                f.x2168 = (rg.rnd() % 100) as i32;
                f.x2160 = rg.rnd() as i32;
                f.set_x221D_b6((rg.rnd() & 1) as u16);
                f.set_x221D_b7((rg.rnd() & 1) as u16);
                f.set_invisible((rg.rnd() & 1) as u8);
                f.set_x2220_b5((rg.rnd() & 1) as u8);
                f.set_x2220_b6((rg.rnd() & 1) as u8);
                f.set_is_metal((rg.rnd() & 1) as u8);
                f.set_x2226_b4((rg.rnd() & 1) as u8);
                f.set_x2226_b5((rg.rnd() & 1) as u8);
                f.set_x2227_b1((rg.rnd() & 1) as u8);
                f.set_x2227_b5((rg.rnd() & 1) as u8);
                f.set_x2228_b3((rg.rnd() % 4) as u8);
                f.set_x2228_b5((rg.rnd() & 1) as u8);
            }
            let n = (rg.rnd() % (NF as u32 + 1)) as usize;
            let mut used = 0u32;
            let mut prev: *mut HSD_GObj = core::ptr::null_mut();
            for _ in 0..n {
                let mut j = (rg.rnd() % NF as u32) as usize;
                while used & (1 << j) != 0 {
                    j = (j + 1) % NF;
                }
                used |= 1 << j;
                let gj: *mut HSD_GObj = &mut w.go[j];
                if prev.is_null() {
                    w.plinks[HSD_GOBJ_PLINK_FIGHTER as usize] = gj;
                } else {
                    unsafe { (*prev).next = gj };
                }
                prev = gj;
            }
            let gi = (rg.rnd() % NF as u32) as usize;
            let g: *mut HSD_GObj = &mut w.go[gi];
            let f0: *mut Fighter = &mut w.f[gi];
            let f0 = unsafe { &mut *f0 };
            let ia = rg.rnd() as i32;
            let ib = rg.rnd() as i32;
            let sa = match rg.rnd() % 3 {
                0 => (rg.rnd() % 12000) as i32,
                1 => 5000 + (rg.rnd() % 5000) as i32,
                _ => 0x1FBFD + (rg.rnd() % 0xD0) as i32,
            };
            let ub = ib as u32;
            let bu = |b: bool| b as u32;
            let mut r: u32 = 0;
            match op {
                0 => r = bu(m::ft_800877f8(g, ia)),
                1 => r = bu(m::ft_80087818(g, ia)),
                2 => r = bu(m::ft_80087838(g)),
                3 => r = bu(m::ft_80087858(g)),
                4 => r = bu(m::ft_80087878(g, 0x22 + (ub % 5) as i32)),
                5 => r = bu(m::ft_800878bc(g)),
                6 => r = bu(m::ft_80087900(g)),
                7 => r = bu(m::ft_80087944(g)),
                8 => r = bu(m::ft_80087988(g)),
                9 => r = bu(m::ft_800879d8(g)),
                10 => r = bu(m::ft_800879f8(g)),
                11 => r = bu(m::ft_80087a18(g)),
                12 => r = m::ft_80087a80(g) as u32,
                13 => r = m::ft_80087a8c(g).to_bits(),
                14 => r = m::ft_80087a98(g) as u32,
                15 => r = bu(m::ft_80087aa4(g)),
                16 => r = m::ft_80087ab4(g) as u32,
                17 => m::ft_80087ac0(g, (ub % 3) as i32),
                18 => r = bu(m::ft_80087aec(g)),
                19 => r = m::ft_80087b34(g) as u32,
                20 => m::ft_80087bac(g, (ub % 3) as i32),
                21 => m::ft_80087bc0(g, ib),
                22 => m::ft_80087bec(g, ib),
                23 => r = m::get_ft_kind_mask() as u32,
                24 => r = m::ft_80087c58(g) as u32,
                25 => m::ft_80087c64(g, ib),
                26 => r = m::ft_80087c70(f0, (ub % 1000) as i32) as u32,
                27 => r = m::ft_80087d0c(f0, sa) as u32,
                28 => m::ft_8008805c(f0, sa),
                29 => m::ft_80088080(f0),
                30 => m::ft_800880ac(f0),
                31 => m::ft_800880d8(f0),
                32 => m::ft_80088110(f0),
                33 => m::play_sfx(f0, sa, ia as u8, ib as u8),
                _ => unreachable!(),
            }
            let mut th: u32 = 0;
            for i in 0..NF {
                let f = &w.f[i];
                th = th.wrapping_mul(31).wrapping_add((f.invisible() | f.x2228_b5() << 1) as u32);
                th = th.wrapping_mul(31).wrapping_add(f.x1988 as u32);
                th = th
                    .wrapping_mul(31)
                    .wrapping_add((f.x2164 as u32).wrapping_mul(3))
                    .wrapping_add((f.x2168 as u32).wrapping_mul(7));
                th = th.wrapping_mul(31).wrapping_add(f.x2160 as u32);
            }
            let got = format!("{} {} {:08X} {:08X} {:08X}", op, k, r, th, tr());
            let want = lines.next().expect("expected file too short");
            assert_eq!(got, want, "op {op} case {k}");
            seen.insert(op);
            checked += 1;
        }
    }
    assert!(lines.next().is_none(), "expected file has extra lines");
    assert_eq!(seen.len(), NOPS as usize);
    assert_eq!(checked, 10200);
}
