//! ft0881.rs vs src/melee/ft/ft_0881.c ft_800881D8..ft_80088C5C (gen/cref/ft0881_ref.c, built
//! with the shipping build's contraction + gwtool lowering; expected values in
//! tests/data/ft0881.txt; regenerate with `python rust/melee_fighter/gen/cref/run.py ft0881`).
//! Six fighters with random sfx channels, one op per case. Fill order and the stubbed callees (trace word `TR`) match the C.
#![allow(static_mut_refs, non_upper_case_globals)]

use melee_fighter::ft0881 as m;
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
        h::LBAUDIOAX_80023870 = Some(|s, v, p, c| {
            t(0x410);
            t(s as u32);
            t(v as u32);
            t(p as u32);
            t(c as u32);
            (tr() >> 6) as i32 - 3
        });
        h::LBAUDIOAX_80026510 = Some(|g| {
            t(0x420 + e(g));
            (tr() >> 2) & 1 != 0
        });
        // C returns 0/1/2 as bool; the caller tests `== 1`: model with `% 3 == 1`.
        h::LBAUDIOAX_800265C4 = Some(|g, s| {
            t(0x430 + e(g));
            t(s as u32);
            (tr() >> 5) % 3 == 1
        });
        h::HSD_RANDI = Some(|n| {
            t(0x160u32.wrapping_add(n as u32));
            ((tr() >> 7) % n as u32) as i32
        });
    }
}

const KINDS: [u32; 8] = [0, 2, 13, 24, 20, 33, 34, 8];
const NOPS: u32 = 17;

#[test]
fn ft0881_matches_c() {
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/ft0881.txt")).unwrap();
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
    let mut ids = [0i32; 5];
    let mut seen = BTreeSet::new();
    let mut checked = 0;
    for op in 0..NOPS {
        for k in 0..300u32 {
            for i in 0..NF {
                w.f[i] = Fighter::zeroed();
                w.go[i] = HSD_GObj::zeroed();
            }
            let mut rg = Rng(0x13579u32.wrapping_add(op.wrapping_mul(7919)).wrapping_add(k.wrapping_mul(104729)));
            unsafe { TR = rg.rnd() };
            for id in ids.iter_mut() {
                *id = if rg.rnd() % 4 == 0 { 0x83D61 } else { (rg.rnd() % 12000) as i32 };
            }
            let mut arr = FtSFXArr::zeroed();
            arr.num = 1 + (rg.rnd() % 5) as i32;
            arr.sfx_ids = ids.as_mut_ptr();
            let rch = |rg: &mut Rng| if rg.rnd() % 3 == 0 { -1 } else { (rg.rnd() % 1000) as i32 };
            for i in 0..NF {
                let fpp: *mut Fighter = &mut w.f[i];
                w.go[i].user_data = fpp as *mut _;
                let gp: *mut HSD_GObj = &mut w.go[i];
                let f = &mut w.f[i];
                f.gobj = gp;
                f.kind = KINDS[(rg.rnd() % 8) as usize];
                f.costume_id = (rg.rnd() % 4) as u8;
                f.player_idx = (rg.rnd() % 6) as u8;
                f.set_is_sub_fighter((rg.rnd() & 1) as u8);
                f.set_x2225_b6((rg.rnd() & 1) as u8);
                f.set_is_metal((rg.rnd() & 1) as u8);
                f.set_x2220_b5((rg.rnd() & 1) as u8);
                f.set_x2220_b6((rg.rnd() & 1) as u8);
                f.x34_scale.x = 1.0;
                f.x34_scale.y = if rg.rnd() & 1 != 0 { 1.0 } else { 1.5 };
                f.x2144 = rch(&mut rg);
                f.x2148 = rch(&mut rg);
                f.x214C = rch(&mut rg);
                f.x2150 = rch(&mut rg);
                f.x2154 = rch(&mut rg);
                f.x2158 = rch(&mut rg);
                f.x215C = rch(&mut rg);
                f.x2160 = rch(&mut rg);
                f.x2164 = (rg.rnd() % 100) as i32;
                f.x2168 = (rg.rnd() % 100) as i32;
            }
            let gi = (rg.rnd() % NF as u32) as usize;
            let g: *mut HSD_GObj = &mut w.go[gi];
            let f0: *mut Fighter = &mut w.f[gi];
            let f0 = unsafe { &mut *f0 };
            let sa = match rg.rnd() % 4 {
                0 => 0x83D60,
                1 => 0x83D61,
                2 => 5000 + (rg.rnd() % 5000) as i32,
                _ => (rg.rnd() % 12000) as i32,
            };
            let v = rg.rnd() as u8;
            let p = rg.rnd() as u8;
            match op {
                0..=6 => {
                    [m::ft_800881d8, m::ft_80088328, m::ft_80088478, m::ft_80088510, m::ft_800885a8, m::ft_80088640, m::ftco_800886d8]
                        [op as usize](f0, sa, v, p)
                }
                7 => m::ft_80088770(f0),
                8 => m::ft_800887cc(f0),
                9 => m::ft_80088828(f0),
                10 => m::ft_80088884(f0),
                11 => m::ft_800888e0(f0),
                12 => m::ft_8008893c(f0),
                13 => m::ft_80088998(f0),
                14 => {
                    let a: *const FtSFXArr = if rg.rnd() % 4 == 0 { core::ptr::null() } else { &arr };
                    m::ft_800889f4(f0, a)
                }
                15 => m::ft_80088a50(f0),
                16 => m::ft_80088c5c(g),
                _ => unreachable!(),
            }
            let mut th: u32 = 0;
            for i in 0..NF {
                let f = &w.f[i];
                for v in [f.x2144, f.x2148, f.x214C, f.x2150, f.x2154, f.x2158, f.x215C, f.x2160, f.x2164, f.x2168] {
                    th = th.wrapping_mul(31).wrapping_add(v as u32);
                }
            }
            let got = format!("{} {} {:08X} {:08X}", op, k, th, tr());
            let want = lines.next().expect("expected file too short");
            assert_eq!(got, want, "op {op} case {k}");
            seen.insert(op);
            checked += 1;
        }
    }
    assert!(lines.next().is_none(), "expected file has extra lines");
    assert_eq!(seen.len(), NOPS as usize);
    assert_eq!(checked, 5100);
}
