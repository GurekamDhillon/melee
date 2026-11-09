//! tu::ftkirbyspecialmewtwo vs the C (gen/turef/tu_ftkirbyspecialmewtwo_ref.c, shipping contraction; data in
//! tests/data/tu_ftkirbyspecialmewtwo.txt; regenerate with
//! `python rust/melee_chara/gen/turef/tu_shard321.py tu_ftkirbyspecialmewtwo`): recoil, launch, the charge sfx
//! ladder and the Start / Loop anims against recording stand-ins, plus a smoke test of the cancel path.
//! One #[test]: the hooks are process globals.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftkirbyspecialmewtwo as t;
use melee_fighter::hooks as fh;
use melee_types::*;

struct Rng(u32);
impl Rng {
    fn rnd(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        self.0
    }
    fn rf(&mut self) -> f32 {
        match self.rnd() % 8 {
            0 => 0.0,
            1 => -0.0,
            2 => 1.0,
            3 => -1.0,
            _ => ((self.rnd() % 4001) as i32 - 2000) as f32 / 997.0,
        }
    }
}

static mut EV: std::vec::Vec<u32> = std::vec::Vec::new();
static mut LB: [f32; 3] = [0.0; 3];
static mut BALL: *mut HSD_GObj = core::ptr::null_mut();
static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();

fn ev(v: u32) {
    unsafe { EV.push(v) }
}
fn evf(v: f32) {
    ev(v.to_bits())
}

fn dump(tag: &str, i: usize, f: &Fighter) -> String {
    let mut s = unsafe {
        format!(
            "{} {} {:08x} {:08x} {} {} {:08x} {:08x} {} {} {} {} |",
            tag,
            i,
            f.self_vel.x.to_bits(),
            f.gr_vel.to_bits(),
            f.u.kb.x9C,
            f.mv.kb.specialhi.xC,
            f.mv.kb.specialhi.x10.f.to_bits(),
            f.mv.kb.specialhi.x8.f.to_bits(),
            f.mv.kb.specialhi.x4,
            f.mv.kb.specialhi.x8.i,
            f.cmd_vars[1],
            (!f.u.kb.x98.is_null()) as i32
        )
    };
    for e in unsafe { std::mem::take(&mut EV) } {
        s += &format!(" {:x}", e);
    }
    s
}

#[test]
fn matches_c_and_smoke() {
    let want = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/tu_ftkirbyspecialmewtwo.txt")).unwrap();
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let mut ball: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let mut dummy: Box<[u8; 64]> = Box::new([0; 64]);
    let mut parts: std::vec::Vec<FighterBone> = (0..100).map(|_| unsafe { core::mem::zeroed() }).collect();
    for p in parts.iter_mut() {
        p.joint = dummy.as_mut_ptr() as *mut _;
    }
    let da: &'static mut ftKb_DatAttrs = Box::leak(Box::new(ftKb_DatAttrs::zeroed()));
    f.dat_attrs = da as *mut _ as *mut _;
    f.parts = parts.as_mut_ptr();
    g.user_data = &mut *f as *mut Fighter as *mut _;
    f.gobj = &mut *g;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        BALL = &mut *ball;
        t::ext::LB_8000B1CC = Some(|_, a, out| unsafe {
            ev(0x10);
            if !a.is_null() {
                evf((*a).x);
                evf((*a).y);
                evf((*a).z);
            }
            (*out).x = LB[0];
            (*out).y = LB[1];
            (*out).z = LB[2];
        });
        t::ext::IT_802C53F0 = Some(|_, p, ang, ch, mx| unsafe {
            ev(0x11);
            evf((*p).x);
            evf((*p).y);
            evf((*p).z);
            evf(ang);
            evf(ch);
            evf(mx);
        });
        t::ext::IT_802C5000 = Some(|_, p, part, kind, dir| unsafe {
            ev(0x12);
            evf((*p).x);
            evf((*p).y);
            evf((*p).z);
            ev(part as u32);
            ev(kind as u32);
            evf(dir);
            BALL
        });
        fh::PLAY_SFX = Some(|_, id, vol, pan| {
            ev(0x13);
            ev(id as u32);
            ev(vol as u32);
            ev(pan as u32);
        });
        t::ext::FT_80088510 = Some(|_, id, vol, pan| {
            ev(0x14);
            ev(id);
            ev(vol as u32);
            ev(pan as u32);
        });
        st::EFLIB_DESTROYALL = Some(|_| ev(0x15));
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, a, sp, bl, _| {
            ev(0x16);
            ev(ms as u32);
            ev(fl);
            evf(a);
            evf(sp);
            evf(bl);
        });
        st::FTCOMMON_8007EBAC = Some(|_, a, c| {
            ev(0x17);
            ev(a);
            ev(c);
        });
        t::ext::FTCO_800BFFD0 = Some(|_, a, c| {
            ev(0x18);
            ev(a as u32);
            ev(c as u32);
            false
        });
        t::ext::EFSYNC_SPAWN = Some(|id, _, a| unsafe {
            let p = a[0] as *const Vec3;
            ev(0x19);
            ev(id as u32);
            evf((*p).x);
            evf((*p).y);
            evf((*p).z);
            core::ptr::null_mut()
        });
        st::FTANIM_ISFRAMESREMAINING = Some(|_| 0);
    }
    let mut r = Rng(0x3E27);
    let mut lines = want.lines();
    for i in 0..1500 {
        da.specialn_mt_charge_time = 1.0 + (r.rnd() % 60) as f32;
        da.specialn_mt_ground_horizontal_momentum = r.rf();
        da.specialn_mt_air_horizontal_momentum = r.rf();
        da.specialn_mt_loops_to_full_charge = r.rnd() % 4;
        unsafe {
            LB[0] = r.rf();
            LB[1] = r.rf();
            LB[2] = r.rf();
        }
        f.facing_dir = if r.rnd() & 1 != 0 { 1.0 } else { -1.0 };
        f.ground_or_air = (r.rnd() & 1) as _;
        f.motion_id = if r.rnd() & 1 != 0 {
            ftKb_MS_MtSpecialAirNEnd as _
        } else if r.rnd() & 1 != 0 {
            ftKb_MS_MtSpecialNEnd as _
        } else {
            0
        };
        unsafe {
            f.u.kb.x9C = (r.rnd() % 70) as i32;
        }
        f.self_vel.x = r.rf();
        f.gr_vel = r.rf();
        f.cur_anim_frame = r.rf();
        unsafe {
            f.mv.kb.specialhi.xC = (r.rnd() % 4) as i32 - 1;
            f.mv.kb.specialhi.x4 = (r.rnd() % 5) as i32;
            f.mv.kb.specialhi.x8.i = (r.rnd() & 1) as i32;
            f.mv.kb.specialhi.x10.f = if r.rnd() & 3 != 0 {
                if r.rnd() & 1 != 0 {
                    r.rf() * 0.5
                } else {
                    0.0
                }
            } else {
                0.0001
            };
        }
        f.cmd_vars[1] = r.rnd() % 3;
        f.cmd_vars[2] = r.rnd() % 2;
        f.cmd_vars[3] = r.rnd() % 2;
        unsafe {
            f.u.kb.x98 = if r.rnd() & 1 != 0 { BALL } else { core::ptr::null_mut() };
        }
        f.item_gobj = core::ptr::null_mut();
        t::ftKb_SpecialNMt_SetRecoil(gp);
        assert_eq!(dump("A", i, &f), lines.next().unwrap(), "A case {i}");
        t::ftKb_SpecialNMt_80107410(gp);
        assert_eq!(dump("B", i, &f), lines.next().unwrap(), "B case {i}");
        t::ftKb_SpecialNMt_801071FC(gp);
        assert_eq!(dump("C", i, &f), lines.next().unwrap(), "C case {i}");
        t::ftKb_MtSpecialNLoop_Anim(gp);
        assert_eq!(dump("D", i, &f), lines.next().unwrap(), "D case {i}");
        unsafe { f.u.kb.x98 = core::ptr::null_mut() };
        t::ftKb_MtSpecialNStart_Anim(gp);
        assert_eq!(dump("E", i, &f), lines.next().unwrap(), "E case {i}");
    }
    assert!(lines.next().is_none());

    // smoke: the cancel path releases the ball and the status helpers read the motion.
    unsafe {
        f.u.kb.x98 = BALL;
        t::ext::IT_802C573C = Some(|_| unsafe { LOG.push("release".into()) });
        st::EFLIB_DESTROYALL = Some(|_| unsafe { LOG.push("destroy".into()) });
        st::FT_8008A2BC = Some(|_| unsafe { LOG.push("wait".into()) });
    }
    t::ftKb_MtSpecialNCancel_Anim(gp);
    assert!(unsafe { f.u.kb.x98 }.is_null());
    assert_eq!(unsafe { LOG.join(",") }, "release,destroy,wait");
    f.motion_id = ftKb_MS_MtSpecialNLoop as _;
    unsafe { f.x2070.x0.set_x2071_b6(0) };
    assert!(!t::ftKb_SpecialNMt_80106F9C(gp));
    assert!(!t::ftKb_SpecialNMt_80106FEC(gp));
    f.motion_id = 0;
    assert!(t::ftKb_SpecialNMt_80106F9C(gp) && t::ftKb_SpecialNMt_80106FEC(gp));
    f.motion_id = ftKb_MS_MtSpecialNCancel as _;
    assert!(t::ftKb_SpecialNMt_80106FEC(gp));
    drop(parts);
}
