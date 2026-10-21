//! src/tu/ftco_800c78b0.rs vs the C (gen/cref/tu_ftco_800c78b0_ref.c includes
//! src/melee/ft/ftCo_800C78B0.c whole, shipping contraction + gwtool lowering; expected values in
//! tests/data/tu_ftco_800c78b0.txt; regenerate with
//! `python rust/melee_fighter/gen/cref/run.py tu_ftco_800c78b0`). Same recorders, same inputs.
#![allow(static_mut_refs)]

use melee_fighter::tu::ftco_800c7070::ext as e70;
use melee_fighter::tu::ftco_800c78b0 as m;
use melee_fighter::tu::ftco_800c7590::ext as e;
use melee_fighter::tu::ftco_800c78b0::ext as e8b;
use melee_fighter::types::*;

static mut RS: u32 = 0;
fn rnd() -> u32 {
    unsafe {
        RS ^= RS << 13;
        RS ^= RS >> 17;
        RS ^= RS << 5;
        RS
    }
}
fn rg() -> f32 {
    match rnd() % 16 {
        0 => 0.0,
        1 => -0.0,
        2 => f32::from_bits(0x7FC0_0000),
        3 => 1.0,
        4 => -1.0,
        _ => ((rnd() % 41) as i32 - 20) as f32 / 20.0,
    }
}
fn rw() -> f32 {
    const K: [f32; 4] = [1.0, 13.0, 1000.0, 100000.0];
    let a = rg();
    a * K[(rnd() % 4) as usize]
}

static mut TR: u32 = 0;
fn t(v: u32) {
    unsafe { TR = TR.wrapping_mul(31).wrapping_add(v) }
}
fn b(f: f32) -> u32 {
    f.to_bits()
}
static mut GO: *mut HSD_GObj = core::ptr::null_mut();
static mut ITEM: *mut HSD_GObj = core::ptr::null_mut();
static mut FP: *mut Fighter = core::ptr::null_mut();
static mut RET_BB8: i32 = 0;
static mut RET_HC: i32 = 0;
static mut MASH: f32 = 0.0;
static mut HD: f32 = 0.0;
static mut KB: f32 = 0.0;
static mut HE: u32 = 0;
static mut HU: u32 = 0;
static mut IPOS: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
static mut ARG3: *mut lbColl_80008D30_arg1 = core::ptr::null_mut();
fn isg(g: *mut HSD_GObj) -> u32 {
    (g == unsafe { GO }) as u32
}
fn isf(f: &Fighter) -> u32 {
    (f as *const Fighter == unsafe { FP } as *const Fighter) as u32
}
fn isi(i: *mut HSD_GObj) -> u32 {
    (i == unsafe { ITEM }) as u32
}

fn install() {
    unsafe {
        e70::FIGHTER_CHANGE_MOTION_STATE = Some(|g, ms, fl, a, bb, c, o| {
            t(1); t(isg(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32);
        });
        e::FTCOMMON_8007DB58 = Some(|g| { t(2); t(isg(g)); });
        e::FTCO_8009750C = Some(|g| { t(3); t(isg(g)); });
        e::FTCO_800DD168 = Some(|g| { t(4); t(isg(g)); });
        e::FTCOMMON_8007E2FC = Some(|f| { t(7); t(isf(f)); });
        e::PLAYER_80033BB8 = Some(|s| { t(8); t(s as u32); RET_BB8 });
        e::PLAYER_GET_HANDICAP = Some(|s| { t(9); t(s as u32); RET_HC });
        e::FTCOMMON_INIT_GRAB = Some(|f, a, tm| { t(10); t(isf(f)); t(a as u32); t(b(tm)); });
        e::FTCOMMON_8007E2F4 = Some(|f, v| { t(11); t(isf(f)); t(v as i32 as u32); });
        e::FTCOMMON_GRAB_MASH = Some(|f, tm| { t(15); t(isf(f)); t(b(tm)); f.grab_timer -= MASH; false });
        e8b::FTANIM_8006EBA4 = Some(|g| { t(30); t(isg(g)); });
        e8b::IT_802DBA68 = Some(|i| { t(31); t(isi(i)); });
        e8b::IT_802DB9F4 = Some(|i| { t(32); t(isi(i)); });
        e8b::IT_8026B294 = Some(|i, p| { t(33); t(isi(i)); t((p as *mut Vec3 == &mut (*FP).cur_pos as *mut Vec3) as u32); *p = IPOS; });
        e8b::MPCOLL_80043670 = Some(|c| { t(34); t((c as *mut CollData == &mut (*FP).coll_data as *mut CollData) as u32); c.cur_pos.y = 7.0; });
        e8b::MPCOLL_800471F8 = Some(|c| { t(35); t((c as *mut CollData == &mut (*FP).coll_data as *mut CollData) as u32); c.cur_pos.x += 1.0; false });
        e8b::LBCOLL_80008D30 = Some(|h, a| { t(36); t((a == ARG3) as u32); h.damage = HD; h.element = HE; h.unk_count = HU; });
        e8b::FTCOLL_800788D4 = Some(|g| { t(37); t(isg(g)); });
        e8b::FTCOLL_80079EA8 = Some(|f, h, n| { t(38); t(isf(f)); t(b(h.damage)); t(n); KB });
        e8b::FIGHTER_UNK_TAKE_DAMAGE_8006CC30 = Some(|f, d| { t(39); t(isf(f)); t(b(d)); });
        e8b::FTCO_DAMAGE_CALC_KNOCKBACK = Some(|f| { t(40); t(isf(f)); });
        e8b::FTCO_8008E908 = Some(|g, f| { t(41); t(isg(g)); t(b(f)); });
        e8b::FIGHTER_TAKE_DAMAGE_8006CC7C = Some(|f, d| { t(42); t(isf(f)); t(b(d)); });
    }
}

struct Hs(u32);
impl Hs {
    fn m(&mut self, v: u32) {
        self.0 = self.0.wrapping_mul(0x0100_0193) ^ v;
    }
    fn v(&mut self, v: &Vec3) {
        self.m(b(v.x)); self.m(b(v.y)); self.m(b(v.z));
    }
}
fn fhash(f: &Fighter) -> u32 {
    let mut s = Hs(0x811C_9DC5);
    let mv = unsafe { f.mv.co.capturelikelike };
    s.m(b(f.grab_timer)); s.m(f.x221D_b5() as u32); s.m(f.invisible() as u32); s.m(f.x1988 as u32);
    s.m(isi(mv.x0)); s.m(mv.x4 as u32);
    s.m(f.take_dmg_cb.is_some() as u32); s.m(f.death2_cb.is_some() as u32); s.m(f.accessory1_cb.is_some() as u32);
    s.v(&f.cur_pos); s.v(&f.coll_data.cur_pos); s.v(&f.coll_data.last_pos);
    s.m(b(f.dmg.kb_applied)); s.m(f.dmg.x1848_kb_angle as u32); s.m(b(f.dmg.facing_dir_1));
    s.m(f.dmg.x184c_damaged_hurtbox as u32); s.v(&f.dmg.x1854_collpos); s.m(f.dmg.x1860_element);
    s.0
}

fn v3() -> Vec3 {
    Vec3 { x: rw(), y: rw(), z: rw() }
}

#[test]
fn matches_c() {
    let data = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/tu_ftco_800c78b0.txt")).unwrap();
    let mut go = Box::new(HSD_GObj::zeroed());
    let mut item = Box::new(HSD_GObj::zeroed());
    let mut f = Box::new(Fighter::zeroed());
    let mut cd = Box::new(ftCommonData::zeroed());
    let mut arg3 = Box::new(lbColl_80008D30_arg1::zeroed());
    go.user_data = &mut *f as *mut Fighter as *mut _;
    unsafe {
        GO = &mut *go;
        ITEM = &mut *item;
        FP = &mut *f;
        ARG3 = &mut *arg3;
        melee_fighter::hooks::P_FT_COMMON_DATA = &*cd;
    }
    install();
    let (mut n, mut bad) = (0, 0);
    for line in data.lines() {
        let w: std::vec::Vec<&str> = line.split_whitespace().collect();
        let (op, k): (u32, u32) = (w[0].parse().unwrap(), w[1].parse().unwrap());
        let ftr = unsafe { &mut *FP };
        *ftr = Fighter::zeroed();
        *cd = ftCommonData::zeroed();
        unsafe {
            TR = 0;
            RS = 0x78B0u32.wrapping_add(op.wrapping_mul(7919)).wrapping_add(k.wrapping_mul(104729));
        }
        cd.x740 = rw(); cd.x744 = rw(); cd.x748 = rw(); cd.x74C = rw(); cd.x750 = rw(); cd.x754 = rw();
        cd.x758 = rw(); cd.x75C = rw();
        ftr.player_id = (rnd() % 6) as u8;
        ftr.dmg.x1830_percent = rw();
        ftr.mv.co.capturelikelike.x0 = unsafe { ITEM };
        ftr.mv.co.capturelikelike.x4 = (rnd() % 2) as i32;
        ftr.grab_timer = rw();
        ftr.facing_dir = rg();
        ftr.dmg.x184c_damaged_hurtbox = 5;
        unsafe {
            RET_BB8 = (rnd() % 9) as i32 - 4;
            RET_HC = (rnd() % 12) as i32;
            MASH = rw(); HD = rw(); KB = rw(); HE = rnd() % 20; HU = rnd() % 4;
            IPOS = v3();
        }
        let a1 = v3();
        let a2 = v3();
        let ang = if rnd() % 3 == 0 { rw() * 10.0 } else { rw() };
        let dmg = (rnd() as i32) % 2000;
        let g = unsafe { GO };
        match op {
            0 => m::ftco_800c78b0(g, unsafe { ITEM }),
            1 => m::ftco_capture_likelike_anim(g),
            2 => m::ftco_800c7b0c(g, &a1, &a2, unsafe { ARG3 }, ang),
            _ => {
                m::ftco_800c7a30(g);
                unsafe { m::fn_800c7ae0(g) };
                m::ftco_800c7c60(g, dmg);
            }
        }
        let got = format!("{:08x} {:08x}", unsafe { TR }, fhash(unsafe { &*FP }));
        let want = format!("{} {}", w[2], w[3]);
        n += 1;
        if got != want {
            if bad < 5 {
                eprintln!("op {op} case {k}: got {got} want {want}");
            }
            bad += 1;
        }
    }
    assert_eq!(n, 12000);
    assert_eq!(bad, 0, "{bad} of {n} differ");
}
