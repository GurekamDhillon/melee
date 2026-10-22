//! tu::ftmariospecials vs the C (gen/turef/tu_ftmariospecials_ref.c, shipping contraction; data in
//! tests/data/tu_ftmariospecials.txt; regenerate with
//! `python rust/melee_chara/gen/turef/tu_shard345.py tu_ftmariospecials`): the wind puff handed to
//! lb_800119DC (fused `3 * facing + x`), the cape's Phys velocities, reflect and AirS Enter. Plus a
//! smoke test of the cape item callbacks through recording hooks.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftmariospecials as t;
use melee_types::*;

/// The hooks are process globals: the tests of this file run one at a time.
static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

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
            _ => ((self.rnd() % 4001) as i32 - 2000) as f32 / 97.0,
        }
    }
}

static mut HIP: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
static mut PUFF_N: u32 = 0;
static mut PUFF_POS: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
static mut PUFF_ARGS: (i32, f32, f32, f32) = (0, 0.0, 0.0, 0.0);
static mut REFLECT_N: u32 = 0;

fn install_cref_hooks() {
    unsafe {
        melee_fighter::hooks::FTPARTS_GET_BONE_INDEX = Some(|_, i| i);
        t::ext::LB_8000B1CC = Some(|_, _, out| *out = HIP);
        t::ext::LB_800119DC = Some(|pos, a, b, c, d| {
            PUFF_N += 1;
            PUFF_POS = *pos;
            PUFF_ARGS = (a, b, c, d);
        });
        t::ext::FTCOLL_CREATEREFLECTHIT = Some(|_, _, _| REFLECT_N += 1);
        st::FTCOLL_8007AEF8 = Some(|_| {});
        st::FT_80084F3C = Some(|_| {});
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, _, _, _, _, _, _| {});
        st::FTANIM_8006EBA4 = Some(|_| {});
    }
}

#[test]
fn matches_c() {
    let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let want = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/tu_ftmariospecials.txt")).unwrap();
    let mut lines = want.lines();
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let a: &'static mut ftMario_DatAttrs = Box::leak(Box::new(ftMario_DatAttrs::zeroed()));
    let mut parts: std::vec::Vec<FighterBone> = (0..80).map(|_| unsafe { core::mem::zeroed() }).collect();
    f.dat_attrs = a as *mut _ as *mut _;
    f.parts = parts.as_mut_ptr();
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    install_cref_hooks();
    let mut r = Rng(0x77C1E5);
    for i in 0..4000 {
        unsafe {
            HIP = Vec3 { x: r.rf(), y: r.rf(), z: r.rf() };
        }
        f.facing_dir = if r.rnd() & 1 != 0 { 1.0 } else { -1.0 };
        if i & 1 != 0 {
            unsafe { HIP.x = ((r.rnd() % 200001) as i32 - 100000) as f32 / 3.0 };
        }
        f.cmd_vars[0] = r.rnd() % 3;
        unsafe { PUFF_N = 0 };
        t::ftMr_SpecialS_Phys(gp);
        let got = unsafe {
            format!(
                "S {} {} {} {:08x} {:08x} {:08x} {} {:08x} {:08x} {:08x} {}",
                i,
                PUFF_N,
                PUFF_ARGS.0,
                PUFF_POS.x.to_bits(),
                PUFF_POS.y.to_bits(),
                PUFF_POS.z.to_bits(),
                0,
                PUFF_ARGS.1.to_bits(),
                PUFF_ARGS.2.to_bits(),
                PUFF_ARGS.3.to_bits(),
                f.cmd_vars[0]
            )
        };
        assert_eq!(got, lines.next().unwrap(), "case {i} S");
        // AirS
        a.specials.vel.x = r.rf();
        a.specials.vel.y = r.rf();
        a.specials.grav = r.rf();
        a.specials.terminal_vel = r.rf();
        f.co_attrs.gravity = r.rf();
        f.co_attrs.terminal_velocity = r.rf();
        unsafe { f.u.mr.x2238_isCapeBoost = (r.rnd() & 1) as i32 };
        f.cmd_vars[0] = r.rnd() % 3;
        f.self_vel.x = r.rf();
        f.self_vel.y = r.rf();
        unsafe { PUFF_N = 0 };
        t::ftMr_SpecialAirS_Phys(gp);
        let got = unsafe {
            format!(
                "A {} {} {:08x} {:08x} {:08x} {:08x} {:08x} {:08x} {:08x} {:08x} {} {}",
                i,
                PUFF_N,
                PUFF_POS.x.to_bits(),
                PUFF_POS.y.to_bits(),
                PUFF_POS.z.to_bits(),
                PUFF_ARGS.1.to_bits(),
                PUFF_ARGS.2.to_bits(),
                PUFF_ARGS.3.to_bits(),
                f.self_vel.x.to_bits(),
                f.self_vel.y.to_bits(),
                f.u.mr.x2238_isCapeBoost,
                f.cmd_vars[0]
            )
        };
        assert_eq!(got, lines.next().unwrap(), "case {i} A");
        // reflect + Enter
        f.cmd_vars[1] = r.rnd() % 3;
        unsafe { f.mv.mr.SpecialS.reflecting = (r.rnd() & 1) as i32 };
        f.set_reflecting(0);
        unsafe { REFLECT_N = 0 };
        f.cmd_vars[0] = 0;
        t::ftMr_SpecialS_Phys(gp);
        let got = unsafe { format!("R {} {} {} {}", i, REFLECT_N, f.mv.mr.SpecialS.reflecting, f.reflecting()) };
        assert_eq!(got, lines.next().unwrap(), "case {i} R");
        f.self_vel.x = r.rf();
        a.specials.vel_x_decay = r.rf();
        t::ftMr_SpecialAirS_Enter(gp);
        let got = format!("E {} {:08x}", i, f.self_vel.x.to_bits());
        assert_eq!(got, lines.next().unwrap(), "case {i} E");
    }
    assert!(lines.next().is_none());
}

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}

#[test]
fn smoke_cape() {
    let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let mut cape: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let a: &'static mut ftMario_DatAttrs = Box::leak(Box::new(ftMario_DatAttrs::zeroed()));
    let mut parts: std::vec::Vec<FighterBone> = (0..80).map(|_| unsafe { core::mem::zeroed() }).collect();
    f.dat_attrs = a as *mut _ as *mut _;
    f.parts = parts.as_mut_ptr();
    f.facing_dir = 1.0;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    let cp: *mut HSD_GObj = &mut *cape;
    static mut CAPE: *mut HSD_GObj = core::ptr::null_mut();
    unsafe {
        CAPE = cp;
        melee_fighter::hooks::FTPARTS_GET_BONE_INDEX = Some(|_, i| i);
        t::ext::LB_8000B1CC = Some(|_, _, out| *out = Vec3 { x: 1.0, y: 2.0, z: 3.0 });
        t::ext::IT_802B2560 = Some(|_, _, _, bone, kind| {
            log(format!("create {bone} {kind}"));
            CAPE
        });
        t::ext::IT_802B2674 = Some(|_| log("remove".into()));
        t::ext::IT_802B26C0 = Some(|_| log("enter".into()));
        t::ext::IT_802B26E0 = Some(|_| log("exit".into()));
    }
    a.specials.cape_kind = 77;
    // Creating the cape once: stores it, holds it, arms the damage + hitlag callbacks.
    t::ftMr_SpecialS_CreateCape(gp);
    assert_eq!(unsafe { f.u.mr.x223C_capeGObj }, cp);
    assert_eq!(f.x1984_heldItemSpec, cp);
    assert_eq!(f.cmd_vars[2], 1);
    assert!(f.take_dmg_cb.is_some() && f.death2_cb.is_some());
    assert!(f.pre_hitlag_cb.is_some() && f.post_hitlag_cb.is_some());
    assert!(f.accessory4_cb.is_none());
    t::ftMr_SpecialS_CreateCape(gp); // cmd_vars[2] set: no second cape
    // Hitlag callbacks reach the cape; taking damage removes it and resets the callbacks.
    unsafe { f.pre_hitlag_cb.unwrap()(gp) };
    unsafe { f.post_hitlag_cb.unwrap()(gp) };
    t::ftMr_SpecialS_RemoveCape(gp);
    assert!(unsafe { f.u.mr.x223C_capeGObj }.is_null());
    assert!(f.take_dmg_cb.is_none() && f.death2_cb.is_none());
    t::ftMr_SpecialS_RemoveCape(gp); // no cape: nothing
    assert_eq!(unsafe { LOG.join(",") }, "create 49 77,enter,exit,remove,exit");
    // Item removal check (ported helper): motion ids 345..=346 keep the cape.
    f.motion_id = 345;
    assert!(!t::ftMr_SpecialS_CheckItemCapeRemove(&f));
    f.motion_id = 347;
    assert!(t::ftMr_SpecialS_CheckItemCapeRemove(&f));
}
