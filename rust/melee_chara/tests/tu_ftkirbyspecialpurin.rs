//! tu::ftkirbyspecialpurin vs the C (gen/turef/tu_ftkirbyspecialpurin_ref.c: all of ftkirbyspecialpurin.c cut
//! verbatim, shipping contraction; data in tests/data/tu_ftkirbyspecialpurin.txt; regenerate with
//! `python rust/melee_chara/gen/turef/tu_shard321.py tu_ftkirbyspecialpurin`). Every callee in another TU
//! is a logging stand-in on both sides, so the compared state is the fighter's roll state plus the call log.
//! One #[test]: the hooks are process globals.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftkirbyspecialpurin as t;
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
    fn rg(&mut self) -> f32 {
        self.rf() * 3.0
    }
    fn sgn(&mut self) -> f32 {
        if self.rnd() & 1 != 0 {
            1.0
        } else {
            -1.0
        }
    }
}

fn fa(x: f32) -> f32 {
    if x < 0.0 {
        -x
    } else {
        x
    }
}

static mut EV: std::vec::Vec<u32> = std::vec::Vec::new();
static mut RET: [i32; 7] = [0; 7]; // 80082888, 80082978, 8008239C, 800824A0, 80081298, ft_80082708, ft_80081D0C
static mut MP: f32 = 0.0;

fn ev(v: u32) {
    unsafe { EV.push(v) }
}
fn evf(v: f32) {
    ev(v.to_bits())
}

struct Env {
    r: Rng,
    f: Box<Fighter>,
    g: Box<HSD_GObj>,
    da: &'static mut ftKb_DatAttrs,
    zz: i32,
    za: f32,
    ab: i32,
}

impl Env {
    fn randomize(&mut self) {
        let r = &mut self.r;
        let da = &mut *self.da;
        let f = &mut *self.f;
        da.specialn_pr_duration = r.rnd() % 30;
        da.specialn_pr_unk = r.rnd() % 5;
        da.specialn_pr_air_height_offset_at_start = r.rf();
        da.specialn_pr_bounciness = r.rf();
        da.specialn_pr_unk1 = r.rf();
        da.specialn_pr_base_speed = r.rg();
        da.specialn_pr_max_speed = r.rg();
        da.specialn_pr_unk2 = r.rf();
        da.specialn_pr_air_x_axis_momentum = r.rf();
        da.specialn_pr_air_y_axis_momentum = r.rf();
        da.specialn_pr_spinning_speed = r.rf();
        da.specialn_pr_spinning_speed_turn = r.rg();
        da.specialn_pr_delay_per_smoke = 1 + r.rnd() % 4;
        da.specialn_pr_unk3 = r.rf();
        da.specialn_pr_bounce1 = r.rf();
        da.specialn_pr_bounce2 = r.rf();
        da.specialn_pr_base_damage = r.rg();
        da.specialn_pr_damage_multiplier = r.rg();
        da.specialn_pr_horizontal_bounce_momentum_on_hit = r.rf();
        da.specialn_pr_vertical_bounce_momentum_on_hit = r.rf();
        da.specialn_pr_forward_momentum_from_stick = r.rf();
        da.specialn_pr_unk4 = r.rf();
        da.specialn_pr_unk5 = r.rg();
        da.specialn_pr_unk6 = 1 + r.rnd() % 3;
        da.specialn_pr_charge_rate1 = r.rg();
        da.specialn_pr_charge_time = fa(r.rg()) * 4.0;
        da.specialn_pr_charge_rate2 = r.rg();
        da.specialn_pr_charge_spin_animation = r.rg() * 10.0;
        da.specialn_pr_unk8 = r.rf();
        da.specialn_pr_some_speed_var = r.rf();
        da.specialn_pr_spin_anim_speed_after_collision = r.rf();
        da.specialn_pr_air_speed = r.rf();
        da.specialn_pr_turn_rate_related = r.rf();
        da.specialn_pr_unk9 = r.rf();
        da.specialn_pr_unk10 = r.rf();
        da.specialn_pr_unk11 = r.rf();
        da.specialn_pr_unk12 = r.rf();
        da.specialn_pr_freefall_toggle = if r.rnd() & 3 != 0 { r.rf() } else { 0.0 };
        f.cur_pos.x = r.rf();
        f.cur_pos.y = r.rf();
        f.cur_pos.z = r.rf();
        f.facing_dir = r.sgn();
        f.gr_vel = r.rg();
        f.self_vel.x = r.rg();
        f.self_vel.y = r.rg();
        f.self_vel.z = r.rf();
        f.xE4_ground_accel_1 = r.rf();
        f.x74_self_accel.x = r.rf();
        f.x74_self_accel.y = r.rf();
        f.cur_anim_frame = r.rf();
        f.ground_or_air = (r.rnd() & 1) as _;
        f.motion_id = (0x1DB + r.rnd() % 17) as _;
        unsafe {
            f.u.kb.x8C.x = r.rf();
            f.u.kb.x8C.y = r.rf();
            f.u.kb.x8C.z = r.rf();
        }
        f.coll_data.floor.normal.x = r.rf();
        f.coll_data.floor.normal.y = r.rf();
        f.coll_data.floor.flags = r.rnd();
        f.coll_data.left_facing_wall.normal.x = r.rf();
        f.coll_data.left_facing_wall.normal.y = r.rf();
        f.coll_data.ecb.top.y = r.rf();
        f.coll_data.ecb.bottom.y = r.rf();
        f.coll_data.ecb.left.x = r.rf();
        f.coll_data.ecb.right.x = r.rf();
        f.coll_data.env_flags = if r.rnd() & 1 != 0 { (r.rnd() & 0xFFF) as i32 } else { 0 };
        f.input.lstick[0].x = r.rf();
        f.input.held_buttons[0] = if r.rnd() & 1 != 0 { HSD_PAD_B as u32 } else { 0 };
        f.x914[0].state = r.rnd() % 3;
        f.x914[0].x4 = r.rnd() % 4;
        let sp = unsafe { &mut f.mv.pr.specialn };
        sp.x0 = (r.rnd() % 30) as i32 - 5;
        sp.x4 = (r.rnd() % 5) as i32 - 1;
        sp.x8 = (r.rnd() % 7) as i32 - 2;
        sp.xC = (r.rnd() % 4) as i32;
        sp.x10 = r.rg();
        sp.x14 = r.rg() * 4.0;
        sp.x18 = r.rg();
        sp.x1C = r.rf();
        sp.facing_dir = if r.rnd() & 3 != 0 { 0.0 } else { r.sgn() };
        sp.x24 = (r.rnd() % 20) as i32;
        sp.x2C = (r.rnd() % 400) as i32 - 20;
        sp.x30 = (r.rnd() & 1) as i32;
        sp.x34.x = r.sgn();
        sp.x34.y = if r.rnd() & 1 != 0 { 0.0 } else { r.rg() };
        unsafe {
            for k in 0..5 {
                RET[k] = (r.rnd() & 1) as i32;
            }
            RET[5] = (r.rnd() & 1) as i32;
            RET[6] = (r.rnd() & 1) as i32;
            MP = r.rf();
        }
        self.zz = (r.rnd() & 1) as i32;
        self.za = r.rf();
        self.ab = (r.rnd() & 1) as i32;
    }

    fn dump(&mut self, tag: &str, i: usize) -> String {
        let f = &*self.f;
        let sp = unsafe { &f.mv.pr.specialn };
        let mut s = format!(
            "{} {} {:08x} {:08x} {:08x} {:08x} {:08x} {:08x} {:08x} {} {} {} {} {} {} {:08x} {:08x} {:08x} {:08x} {} {} |",
            tag,
            i,
            f.gr_vel.to_bits(),
            f.self_vel.x.to_bits(),
            f.self_vel.y.to_bits(),
            f.facing_dir.to_bits(),
            sp.x14.to_bits(),
            sp.x10.to_bits(),
            sp.x18.to_bits(),
            sp.x0,
            sp.x2C,
            sp.x30,
            sp.x8,
            sp.x24,
            sp.xC,
            sp.x34.x.to_bits(),
            sp.x34.y.to_bits(),
            sp.x1C.to_bits(),
            sp.facing_dir.to_bits(),
            unsafe { f.u.kb.hat.kind },
            f.x914[0].state
        );
        for e in unsafe { std::mem::take(&mut EV) } {
            s += &format!(" {:x}", e);
        }
        s
    }
}

fn install() {
    unsafe {
        t::ext::HSD_JOBJSETSCALE = Some(|_, s| unsafe {
            ev(0x12);
            evf((*s).x);
            evf((*s).y);
            evf((*s).z);
        });
        t::ext::HSD_JOBJGETSCALE = Some(|_, s| unsafe {
            (*s).x = 1.5;
            (*s).y = 2.5;
            (*s).z = 3.5;
            ev(0x13);
        });
        t::ext::HSD_JOBJSETROTATIONY = Some(|_, v| {
            ev(0x14);
            evf(v);
        });
        t::ext::FTPARTSETROTX = Some(|_, i, v| {
            ev(0x10);
            ev(i as u32);
            evf(v);
        });
        t::ext::FTPARTSETROTY = Some(|_, i, v| {
            ev(0x11);
            ev(i as u32);
            evf(v);
        });
        t::ext::FT_80089824 = Some(|_| ev(0x15));
        t::ext::FT_800892A0 = Some(|_| ev(0x16));
        t::ext::FTCOLL_8007AFF8 = Some(|_| ev(0x17));
        t::ext::FTCOLL_8007ABD0 = Some(|_, dmg, _| {
            ev(0x18);
            ev(dmg);
        });
        t::ext::FTCO_800BFFAC = Some(|_| ev(0x19));
        t::ext::FTCO_800BFFD0 = Some(|_, a, c| {
            ev(0x1A);
            ev(a as u32);
            ev(c as u32);
            false
        });
        t::ext::FT_80088510 = Some(|_, id, v, p| {
            ev(0x1B);
            ev(id);
            ev(v as u32);
            ev(p as u32);
        });
        t::ext::CAMERA_REQUESTQUAKE = Some(|k, p| unsafe {
            ev(0x1C);
            ev(k);
            evf((*p).x);
            evf((*p).y);
            evf((*p).z);
        });
        t::ext::EFSYNC_SPAWN = Some(|id, _, a| unsafe {
            let p = a[0] as *const Vec3;
            let q = a[1] as *const f32;
            ev(0x1D);
            ev(id as u32);
            evf((*p).x);
            evf((*p).y);
            evf((*p).z);
            evf(*q);
            if id == 0x3FF {
                evf(*(a[2] as *const f32));
            }
            core::ptr::null_mut()
        });
        t::ext::FT_80082888 = Some(|_, _| {
            ev(0x1E);
            unsafe { RET[0] != 0 }
        });
        t::ext::FT_80082978 = Some(|_, _| {
            ev(0x1F);
            unsafe { RET[1] != 0 }
        });
        t::ext::FT_8008239C = Some(|_, d, _| {
            ev(0x20);
            ev(d as u32);
            unsafe { RET[2] != 0 }
        });
        t::ext::FT_800824A0 = Some(|_, _| {
            ev(0x21);
            unsafe { RET[3] != 0 }
        });
        st::FTCLIFFCOMMON_80081298 = Some(|_| unsafe { RET[4] });
        t::ext::FTCLIFFCOMMON_80081370 = Some(|_| ev(0x22));
        t::ext::MPLIB_800569EC = Some(|_| unsafe { MP });
        t::ext::FTCOMMON_8007D5D4 = Some(|_| ev(0x23));
        t::ext::FTCOMMON_8007D7FC = Some(|_| ev(0x24));
        t::ext::FTCOMMON_FALL = Some(|_, a, c| {
            ev(0x25);
            evf(a);
            evf(c);
        });
        t::ext::FTCOMMON_SETSELFMOVEMENT = Some(|_| ev(0x26));
        st::FTCOMMON_8007EBAC = Some(|_, a, c| {
            ev(0x27);
            ev(a);
            ev(c);
        });
        fh::PLAY_SFX = Some(|_, id, vol, pan| {
            ev(0x28);
            ev(id as u32);
            ev(vol as u32);
            ev(pan as u32);
        });
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, a, sp, bl, _| {
            ev(0x29);
            ev(ms as u32);
            ev(fl);
            evf(a);
            evf(sp);
            evf(bl);
        });
        st::FTANIM_8006EBA4 = Some(|_| ev(0x2A));
        st::FTANIM_SETANIMRATE = Some(|_, r| {
            ev(0x2B);
            evf(r);
        });
        st::FTANIM_ISFRAMESREMAINING = Some(|_| 0);
        st::FT_8008A2BC = Some(|_| ev(0x2C));
        st::FTCO_FALL_ENTER = Some(|_| ev(0x2D));
        st::FTCO_LANDINGFALLSPECIAL_ENTER = Some(|_, a, l| {
            ev(0x2E);
            ev(a as u32);
            evf(l);
        });
        st::FTCO_80096900 = Some(|_, a, c, d, e, f| {
            ev(0x2F);
            ev(a as u32);
            ev(c as u32);
            ev(d as u32);
            evf(e);
            evf(f);
        });
        st::FT_80082708 = Some(|_| unsafe { RET[5] });
        st::FT_80081D0C = Some(|_| unsafe { RET[6] });
    }
}

#[test]
fn matches_c() {
    let want = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/tu_ftkirbyspecialpurin.txt")).unwrap();
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let da: &'static mut ftKb_DatAttrs = Box::leak(Box::new(ftKb_DatAttrs::zeroed()));
    let mut dummy: Box<[u8; 64]> = Box::new([0; 64]);
    let mut parts: std::vec::Vec<FighterBone> = (0..100).map(|_| unsafe { core::mem::zeroed() }).collect();
    for p in parts.iter_mut() {
        p.joint = dummy.as_mut_ptr() as *mut _;
    }
    f.dat_attrs = da as *mut _ as *mut _;
    f.parts = parts.as_mut_ptr();
    g.user_data = &mut *f as *mut Fighter as *mut _;
    g.hsd_obj = dummy.as_mut_ptr() as *mut _;
    f.gobj = &mut *g;
    let gp: *mut HSD_GObj = &mut *g;
    install();
    let mut env = Env { r: Rng(0x9071F), f, g, da, zz: 0, za: 0.0, ab: 0 };
    let mut lines = want.lines();
    macro_rules! case {
        ($tag:expr, $i:expr, $call:expr) => {{
            env.randomize();
            $call;
            let got = env.dump($tag, $i);
            assert_eq!(got, lines.next().unwrap(), "{} case {}", $tag, $i);
        }};
    }
    for i in 0..200 {
        case!("a", i, t::ftKb_PrSpecialNLoop_Anim(gp));
        case!("b", i, t::ftKb_PrSpecialNFull_Anim(gp));
        case!("c", i, t::ftKb_PrSpecialAirNLoop_Anim(gp));
        case!("d", i, t::ftKb_PrSpecialAirNFull_Anim(gp));
        case!("e", i, t::ftKb_PrSpecialN1_Anim(gp));
        case!("f", i, t::ftKb_PrSpecialAirN_Anim(gp));
        case!("g", i, t::ftKb_PrSpecialNTurn_Anim(gp));
        case!("h", i, t::ftKb_PrSpecialN_Anim(gp));
        case!("i", i, t::ftKb_PrSpecialNHit_Anim(gp));
        case!("j", i, t::ftKb_PrSpecialNStart_Anim(gp));
        case!("k", i, t::ftKb_PrSpecialNEnd_Anim(gp));
        case!("l", i, t::ftKb_PrSpecialAirNEndR_Anim(gp));
        case!("m", i, t::ftKb_PrSpecialNLoop_IASA(gp));
        case!("n", i, t::ftKb_PrSpecialAirNFull_IASA(gp));
        case!("o", i, t::ftKb_PrSpecialN1_IASA(gp));
        case!("p", i, t::ftKb_PrSpecialN1_Phys(gp));
        case!("q", i, t::ftKb_PrSpecialNTurn_Phys(gp));
        case!("r", i, t::ftKb_PrSpecialAirN_Phys(gp));
        case!("s", i, t::ftKb_PrSpecialN_Phys(gp));
        case!("t", i, t::ftKb_PrSpecialN1_Coll(gp));
        case!("u", i, t::ftKb_PrSpecialAirN_Coll(gp));
        case!("v", i, t::ftKb_PrSpecialNTurn_Coll(gp));
        case!("w", i, t::ftKb_PrSpecialNHit_Coll(gp));
        case!("x", i, t::fn_80100E0C(gp));
        case!("y", i, t::ftKb_SpecialNPr_80100F94(gp));
        case!("z", i, {
            let (u, a) = (env.zz != 0, env.za);
            t::ftKb_SpecialNPr_801010D4(gp, u, 0x40012, a)
        });
        case!("A", i, {
            let a = env.ab != 0;
            t::ftKb_SpecialNPr_8010140C(gp, a)
        });
        case!("B", i, t::ftKb_SpecialNPr_80101560(gp));
        case!("C", i, t::ftKb_SpecialNPr_80101618(gp));
        case!("D", i, t::fn_80105978(gp));
        case!("E", i, t::ftKb_PrSpecialNStart_Coll(gp));
        case!("F", i, t::ftKb_PrSpecialNLoop_Coll(gp));
        case!("G", i, t::ftKb_PrSpecialNEnd_Coll(gp));
        case!("H", i, t::ftKb_PrSpecialAirNStart_Coll(gp));
    }
    assert!(lines.next().is_none());
    drop(parts);
}
