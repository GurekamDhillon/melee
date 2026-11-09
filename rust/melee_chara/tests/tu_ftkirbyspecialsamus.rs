//! tu::ftkirbyspecialsamus vs the C (gen/turef/tu_ftkirbyspecialsamus_ref.c, shipping contraction; data in
//! tests/data/tu_ftkirbyspecialsamus.txt; regenerate with
//! `python rust/melee_chara/gen/turef/tu_shard321.py tu_ftkirbyspecialsamus`): launch, recoil, charge level
//! sfx and the Start / Hold anims against recording stand-ins, plus a smoke test of the HUD helpers.
//! One #[test]: the hooks are process globals.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftkirbyspecialsamus as t;
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
static mut SHOT: *mut HSD_GObj = core::ptr::null_mut();
static mut FRAMES: i32 = 0;

fn ev(v: u32) {
    unsafe { EV.push(v) }
}
fn evf(v: f32) {
    ev(v.to_bits())
}

fn dump(tag: &str, i: usize, f: &Fighter) -> String {
    let mut s = unsafe {
        format!(
            "{} {} {:08x} {} {} {} {} {} {} {} {} {} |",
            tag,
            i,
            f.self_vel.x.to_bits(),
            f.u.kb.xA8,
            f.mv.kb.specialhi.x4,
            f.mv.kb.specialhi.x0,
            f.cmd_vars[0],
            f.cmd_vars[1],
            f.cmd_vars[2],
            (!f.u.kb.xA4.is_null()) as i32,
            (!f.u.kb.xAC.is_null()) as i32,
            (!f.item_gobj.is_null()) as i32
        )
    };
    for e in unsafe { std::mem::take(&mut EV) } {
        s += &format!(" {:x}", e);
    }
    s
}

#[test]
fn matches_c_and_smoke() {
    let want = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/tu_ftkirbyspecialsamus.txt")).unwrap();
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let mut shot: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
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
        SHOT = &mut *shot;
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
        t::ext::IT_802B56E4 = Some(|_, p, ang, ch, mx| unsafe {
            ev(0x11);
            evf((*p).x);
            evf((*p).y);
            evf((*p).z);
            evf(ang);
            evf(ch);
            evf(mx);
        });
        t::ext::IT_802B55C8 = Some(|_, p, part, kind, dir| unsafe {
            ev(0x12);
            evf((*p).x);
            evf((*p).y);
            evf((*p).z);
            ev(part as u32);
            ev(kind as u32);
            evf(dir);
            SHOT as *mut core::ffi::c_void
        });
        t::ext::IT_802B5974 = Some(|_| ev(0x13));
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
        t::ext::FTCO_800BFFD0 = Some(|_, a, c| {
            ev(0x18);
            ev(a as u32);
            ev(c as u32);
            false
        });
        t::ext::EFSYNC_SPAWN = Some(|id, _, a| unsafe {
            let p = a[0] as *const Vec3;
            let q = a[1] as *const f32;
            ev(0x19);
            ev(id as u32);
            evf((*p).x);
            evf((*p).y);
            evf((*p).z);
            evf(*q);
            core::ptr::null_mut()
        });
        st::FTANIM_ISFRAMESREMAINING = Some(|_| unsafe { FRAMES });
    }
    let mut r = Rng(0x55A5);
    let mut lines = want.lines();
    for i in 0..2000 {
        da.specialn_ss_charge_time = 1.0 + (r.rnd() % 100) as f32;
        da.specialn_ss_aerial_shot_recoil = r.rf();
        da.specialn_ss_frames_per_charge_level = (r.rnd() % 4) as i32;
        unsafe {
            LB[0] = r.rf();
            LB[1] = r.rf();
            LB[2] = r.rf();
        }
        f.facing_dir = if r.rnd() & 1 != 0 { 1.0 } else { -1.0 };
        f.ground_or_air = (r.rnd() & 1) as _;
        f.motion_id = if r.rnd() & 1 != 0 { ftKb_MS_SsSpecialAirN as _ } else { 0 };
        unsafe {
            f.u.kb.xA8 = (r.rnd() % (da.specialn_ss_charge_time as i32 + 1) as u32) as i32;
        }
        f.self_vel.x = r.rf();
        unsafe {
            f.mv.kb.specialhi.x4 = (r.rnd() % 5) as i32;
            f.mv.kb.specialhi.x0 = (r.rnd() & 1) as i32;
        }
        f.cmd_vars[0] = r.rnd() % 3;
        f.cmd_vars[1] = r.rnd() % 3;
        f.cmd_vars[2] = r.rnd() % 2;
        unsafe {
            f.u.kb.xA4 = if r.rnd() & 1 != 0 { SHOT as *mut _ } else { core::ptr::null_mut() };
            f.u.kb.xAC = if r.rnd() & 1 != 0 { SHOT as *mut _ } else { core::ptr::null_mut() };
        }
        f.item_gobj = core::ptr::null_mut();
        unsafe { FRAMES = (r.rnd() & 1) as i32 };
        t::ftKb_SpecialNSs_800FCDE0(gp);
        assert_eq!(dump("A", i, &f), lines.next().unwrap(), "A case {i}");
        t::ftKb_SsSpecialNStart_Anim(gp);
        assert_eq!(dump("B", i, &f), lines.next().unwrap(), "B case {i}");
        t::ftKb_SsSpecialNHold_Anim(gp);
        assert_eq!(dump("C", i, &f), lines.next().unwrap(), "C case {i}");
        t::ftKb_SsSpecialAirNStart_Anim(gp);
        assert_eq!(dump("D", i, &f), lines.next().unwrap(), "D case {i}");
    }
    assert!(lines.next().is_none());

    // smoke: HUD helpers and the motion-range status checks.
    unsafe {
        f.u.kb.xA4 = SHOT as *mut _;
        f.u.kb.xA8 = 17;
    }
    da.specialn_ss_charge_time = 60.0;
    let (mut a, mut b) = (0, 0);
    assert_eq!(t::ftKb_SpecialNSs_800FCC14(gp, &mut a, &mut b), 0);
    assert_eq!((a, b), (17, 60));
    unsafe { f.u.kb.xA4 = core::ptr::null_mut() };
    assert_eq!(t::ftKb_SpecialNSs_800FCC14(gp, &mut a, &mut b), -1);
    f.motion_id = 0x199;
    unsafe { f.x2070.x0.set_x2071_b6(1) };
    assert!(t::ftKb_SpecialNSs_800FCC6C(gp));
    unsafe { f.x2070.x0.set_x2071_b6(0) };
    assert!(!t::ftKb_SpecialNSs_800FCC6C(gp));
    assert!(t::ftKb_SpecialNSs_800FCCBC(gp));
    f.motion_id = 0x19A;
    assert!(!t::ftKb_SpecialNSs_800FCCBC(gp));
    assert_eq!(t::charge_sfx_index(60, 60.0), 5);
    assert_eq!(t::charge_sfx_index(0, 60.0), 0);
    drop(parts);
}
