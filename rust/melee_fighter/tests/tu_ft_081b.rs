//! src/tu/ft_081b.rs vs the C of src/melee/ft/ft_081B.c (gen/cref/tu_ft_081B_ref.c, expected log
//! in tests/data/tu_ft_081B.txt; regenerate with
//! `python rust/melee_fighter/gen/cref/tu_run_s57.py tu_ft_081B`). Fill order matches the C.
//! Plus a smoke test of the stateful wrappers with stubbed callees.
#![allow(static_mut_refs)]

use melee_fighter::hooks;
use melee_fighter::tu::ft_081b::{self as m, ext};
use melee_fighter::types::*;

static mut LOG: String = String::new();
static mut RET: bool = false;
static mut A00: bool = false;
static mut JOINTS: *const u8 = core::ptr::null();
static mut F0: *const Fighter = core::ptr::null();

fn log(s: String) {
    unsafe {
        LOG.push_str(&s);
        LOG.push('\n');
    }
}
fn b(f: f32) -> String {
    format!("{:08X}", f.to_bits())
}
fn ji(p: *mut HSD_JObj) -> i32 {
    if p.is_null() { -1 } else { (p as usize - unsafe { JOINTS } as usize) as i32 }
}

struct Rng(u32);
impl Rng {
    fn rnd(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        self.0
    }
    fn rf(&mut self) -> f32 {
        let k = self.rnd() % 16;
        if k == 0 {
            return 0.0;
        }
        if k == 1 {
            return -0.0;
        }
        ((self.rnd() % 2001) as i32 - 1000) as f32 / 7.0
    }
}

fn l(n: &str, c: *mut CollData) -> bool {
    log(format!("{} {}", n, b(unsafe { (*c).ledge_snap_height })));
    unsafe { RET }
}

fn install() {
    unsafe {
        ext::MPCOLL_80041EE4 = Some(|c| log(format!("1EE4 {}", (c as *const CollData == &(*F0).coll_data) as i32)));
        ext::MPCOLL_SET_ECB_SOURCE_JOBJ = Some(|_, g, a, bb, c, d, e, f, h, s| {
            log(format!("ECB {} {} {} {} {} {} {} {} {}", ((*g).user_data as *const Fighter == F0) as i32,
                ji(a), ji(bb), ji(c), ji(d), ji(e), ji(f), ji(h), b(s)))
        });
        ext::MPCOLL_SET_LEDGE_SNAP = Some(|_, x, y, h| log(format!("SNAP {} {} {}", b(x), b(y), b(h))));
        ext::MPCOLL_800477E0 = Some(|c| l("77E0", c));
        ext::MPCOLL_800471F8 = Some(|c| l("71F8", c));
        ext::MPCOLL_800473CC = Some(|c| l("73CC", c));
        ext::MPCOLL_80048274 = Some(|c| l("8274", c));
        ext::MPCOLL_80048160 = Some(|c| l("8160", c));
        ext::MPCOLL_80048464 = Some(|c| l("8464", c));
        ext::MPCOLL_80048388 = Some(|c| l("8388", c));
        ext::MPCOLL_80048768 = Some(|c| l("8768", c));
        ext::MPCOLL_80048578 = Some(|c| l("8578", c));
        ext::FT_80081A00 = Some(|_| {
            log("1A00".into());
            A00
        });
        ext::MP_CHECK_RIGHT_WALL = Some(|ax, ay, bx, by, _, _, _, _, e, f| {
            log(format!("RW {} {} {} {} {}", b(ax), b(ay), b(bx), b(by), e + f));
            RET
        });
        ext::MP_CHECK_LEFT_WALL = Some(|ax, ay, bx, by, _, _, _, _, e, f| {
            log(format!("LW {} {} {} {} {}", b(ax), b(ay), b(bx), b(by), e + f));
            RET
        });
    }
}

#[test]
fn ft_081b_matches_c() {
    install();
    let joints = [0u8; 8];
    let mut bones: [FighterBone; 8] = core::array::from_fn(|_| FighterBone::zeroed());
    for (i, bn) in bones.iter_mut().enumerate() {
        bn.joint = &joints[i] as *const u8 as *mut HSD_JObj;
    }
    let mut x44 = ftData_x44_t::zeroed();
    let mut fd = ftData::zeroed();
    let mut cd = ftCommonData::zeroed();
    let mut fs: std::vec::Vec<Fighter> = (0..2).map(|_| Fighter::zeroed()).collect();
    let mut go: std::vec::Vec<HSD_GObj> = (0..2).map(|_| HSD_GObj::zeroed()).collect();
    unsafe {
        JOINTS = joints.as_ptr();
        F0 = &fs[0];
        hooks::P_FT_COMMON_DATA = &cd;
    }
    for op in 0..7u32 {
        for k in 0..1000u32 {
            let mut r = 0i32;
            for f in fs.iter_mut() {
                *f = Fighter::zeroed();
            }
            let mut rg = Rng(0x81B38u32.wrapping_add(op.wrapping_mul(7919)).wrapping_add(k.wrapping_mul(104729)));
            for i in 0..2 {
                go[i] = HSD_GObj::zeroed();
                go[i].user_data = &mut fs[i] as *mut Fighter as *mut _;
                let f = &mut fs[i];
                f.ft_data = &mut fd;
                f.parts = bones.as_mut_ptr();
                f.ecb_lock = rg.rnd() as i32;
                f.set_allow_sdi((rg.rnd() & 1) as u8);
                f.set_x2224_b2((rg.rnd() % 3 == 0) as u8);
                f.x2064_ledgeCooldown = if rg.rnd() % 3 == 0 { 5 } else { 0 };
                f.cur_pos.x = rg.rf();
                f.cur_pos.y = rg.rf();
                f.cur_pos.z = rg.rf();
                f.x34_scale.y = rg.rf();
                f.co_attrs.weight = rg.rf();
                f.facing_dir = rg.rf();
                let c = &mut f.coll_data;
                c.cur_pos.x = rg.rf();
                c.cur_pos.y = rg.rf();
                c.cur_pos.z = rg.rf();
                c.ecb_source.kind = (rg.rnd() % 3) as _;
                c.ecb_source.x124 = rg.rf();
                c.ecb_source.x128 = rg.rf();
                c.ecb_source.x12C = rg.rf();
                c.ledge_snap_x = rg.rf();
                c.ledge_snap_y = rg.rf();
                c.ledge_snap_height = rg.rf();
                c.ecb.top.y = rg.rf();
                c.ecb.bottom.y = rg.rf();
            }
            x44.unk0 = (rg.rnd() % 8) as i16;
            x44.unk2 = (rg.rnd() % 8) as i16;
            x44.unk4 = (rg.rnd() % 8) as i16;
            x44.unk6 = (rg.rnd() % 8) as i16;
            x44.unk8 = (rg.rnd() % 8) as i16;
            x44.unkA = (rg.rnd() % 8) as i16;
            x44.unkC = rg.rf();
            x44.ledge_snap_x = rg.rf();
            x44.ledge_snap_y = rg.rf();
            x44.ledge_snap_height = rg.rf();
            fd.x44 = &mut x44;
            cd.x1CC = rg.rf();
            unsafe {
                RET = rg.rnd() & 1 != 0;
                A00 = rg.rnd() % 4 == 0;
            }
            log(format!("OP {} {}", op, k));
            let g0: *mut HSD_GObj = &mut go[0];
            match op {
                0 => m::ft_80081b38(g0),
                1 => m::ft_80081c88(g0, rg.rf()),
                2 => r = m::ft_80081dd4(g0) as i32,
                3 => r = m::ft_80081f2c(g0) as i32,
                4 => r = m::ft_80082084(g0) as i32,
                5 => {
                    let mut a = ftCollisionBox::zeroed();
                    a.top = rg.rf();
                    a.bottom = rg.rf();
                    a.left.x = rg.rf();
                    a.left.y = rg.rf();
                    a.right.x = rg.rf();
                    a.right.y = rg.rf();
                    let mut bx = ftCollisionBox::zeroed();
                    let p = m::ft_80082838(&mut bx, &mut a, fs[0].facing_dir);
                    r = (p == &mut bx as *mut _) as i32;
                    let p = unsafe { &*p };
                    log(format!("BOX {} {} {} {} {} {}", b(p.top), b(p.bottom), b(p.left.x), b(p.left.y), b(p.right.x), b(p.right.y)));
                }
                _ => r = m::ft_80084ce4(&fs[0], &fs[1]) as i32,
            }
            let f = &fs[0];
            let c = &f.coll_data;
            log(format!("R {} {} {} | {} {} {} | {} {} {} | {} {} {}", r, f.ecb_lock, c.x34_flags.b1234(),
                b(c.ecb_source.x124), b(c.ecb_source.x128), b(c.ecb_source.x12C),
                b(c.ledge_snap_x), b(c.ledge_snap_y), b(c.ledge_snap_height),
                b(c.x50), b(f.cur_pos.y), b(c.last_pos.x)));
        }
    }
    let want = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/tu_ft_081B.txt")).unwrap();
    let got = unsafe { LOG.clone() };
    for (i, (w, g)) in want.lines().zip(got.lines()).enumerate() {
        assert_eq!(w, g, "line {}", i + 1);
    }
    assert_eq!(want.lines().count(), got.lines().count());
    ft_081b_smoke();
}

static mut CALLS: std::vec::Vec<&'static str> = std::vec::Vec::new();

/// Run after the C comparison (same thread: the ext hooks are global).
fn ft_081b_smoke() {
    let mut f = Fighter::zeroed();
    let mut g = HSD_GObj::zeroed();
    g.user_data = &mut f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut g;
    f.cur_pos.x = 3.0;
    f.facing_dir = -1.0;
    unsafe {
        ext::FT_80081A00 = Some(|_| false);
        ext::MPCOLL_8004B108 = Some(|_| true);
        ext::MPCOLL_8004B2DC = Some(|_| false);
        ext::FTCO_FALL_ENTER = Some(|_| CALLS.push("fall"));
        ext::FTCO_8009F39C = Some(|_| CALLS.push("slip"));
        ext::FTCO_8009EDA4 = Some(|_| {
            CALLS.push("edge");
            true
        });
        ext::MPCOLL_80047AC8 = Some(|_, cb, g| cb(g, 7));
        ext::MPCOLL_80047E14 = Some(|_, _, _| false);
        ext::MPCOLL_SET_FACING_DIR = Some(|c, d| (*c).facing_dir = d as i16);
        ext::FTWALLJUMP_8008169C = Some(|_| false);
        ext::FTCLIFFCOMMON_80081298 = Some(|_| {
            CALLS.push("cliff");
            false
        });
        ext::FTCO_8009EF68 = Some(|_| {
            CALLS.push("ef68");
            false
        });
        ext::MP_LINE_GET_FLAGS = Some(|i| i as u32 * 2);
        ext::MPLIB_80056AFC = Some(|_, _| {
            static mut A: [i32; 4] = [4, 5, 6, 7];
            A.as_mut_ptr()
        });
        ext::MPLIB_80056B34 = Some(|_, _| -1);
        ext::MPCOLL_8004CA6C = Some(|_| 0.25);
    }
    // air after ledge: ft_800844EC calls the edge handler; cur_pos copied through coll.
    m::ft_800844ec(gp);
    assert_eq!(f.coll_data.cur_pos.x, 3.0);
    m::ft_80084104(gp);
    unsafe { ext::MPCOLL_8004B108 = Some(|_| false) };
    m::ft_80083f88(gp);
    f.coll_data.env_flags = Collide_RightLedgeSlip;
    m::ft_800845b4(gp);
    assert_eq!(unsafe { CALLS.clone() }, ["edge", "fall", "fall", "slip"]);
    // callback collision: cooldown set -> the mpColl gets our callback.
    unsafe { CALLS.clear() };
    f.x2064_ledgeCooldown = 1;
    m::ft_800835b0(gp, |_, n| n == 7, |_| unsafe { CALLS.push("cb") });
    f.x2064_ledgeCooldown = 0;
    m::ft_800835b0(gp, |_, _| true, |_| unsafe { CALLS.push("cb") });
    assert_eq!(f.coll_data.facing_dir, -1);
    assert_eq!(unsafe { CALLS.clone() }, ["cb", "cliff", "ef68"]);
    // floor-line lookup
    f.coll_data.floor.index = 3;
    let (mut a1, mut a2, mut a3) = ([0i32; 4], 0i32, 0i32);
    assert!(m::ft_80084c74(gp, a1.as_mut_ptr(), &mut a2, &mut a3));
    assert_eq!((a1, a2, a3), ([4, 5, 6, 7], 0, -1));
    f.ground_or_air = GA_Air;
    assert!(!m::ft_80084c74(gp, a1.as_mut_ptr(), &mut a2, &mut a3));
    f.kind = Ft_Kind_Nana;
    assert_eq!(m::ft_get_ground_friction_multiplier(&mut f), 1.0);
    f.kind = 0;
    assert_eq!(m::ft_get_ground_friction_multiplier(&mut f), 0.25);
}
