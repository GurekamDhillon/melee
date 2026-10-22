//! tu::ftmariospecialn vs the C (gen/turef/tu_ftmariospecialn_ref.c; data in
//! tests/data/tu_ftmariospecialn.txt; regenerate with
//! `python rust/melee_chara/gen/turef/tu_shard345.py tu_ftmariospecialn`): the Megavitamin colour
//! pick (both sides draw from the same deterministic HSD_Randi stream) and ItemFireSpawn's
//! Mario / Dr. Mario dispatch. Plus a smoke test of the IASA callbacks.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftmariospecialn as t;
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
}

static mut RS: Rng = Rng(0x13572468);
static mut RANDI_N: i32 = 0;
static mut LOGB: String = String::new();

fn install_cref_hooks() {
    unsafe {
        melee_fighter::hooks::FTPARTS_GET_BONE_INDEX = Some(|_, i| i);
        t::ext::HSD_RANDI = Some(|n| {
            RANDI_N = n;
            if n > 0 {
                (RS.rnd() % n as u32) as i32
            } else {
                0
            }
        });
        t::ext::GENO_DEFINE_BASE_KIND = Some(|k| if k == 40 { Ft_Kind_Mario as i32 } else { k });
        t::ext::LB_8000B1CC = Some(|_, _, out| unsafe { *out = Vec3 { x: 1.0, y: 2.0, z: 3.0 } });
        t::ext::IT_8029B6F8 = Some(|_, _, kind, facing| LOGB += &format!("fire{},{},{};", kind, facing as i32, 0));
        t::ext::EFSYNC_SPAWN = Some(|id, _, _| {
            LOGB += &format!("ef{},{},{};", id, 0, 0);
            core::ptr::null_mut()
        });
        t::ext::ITDRMARIOPILL_SPAWN = Some(|_, _, color, kind, facing| LOGB += &format!("pill{},{},{};", color, kind, facing as i32));
    }
}

#[test]
fn matches_c() {
    let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let want = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/tu_ftmariospecialn.txt")).unwrap();
    let mut lines = want.lines();
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let mut parts: std::vec::Vec<FighterBone> = (0..80).map(|_| unsafe { core::mem::zeroed() }).collect();
    f.parts = parts.as_mut_ptr();
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    install_cref_hooks();
    let mut r = Rng(0x5E1CA7);
    for i in 0..3000 {
        unsafe {
            f.u.mr.x222C_vitaminCurr = (r.rnd() % 11) as i32;
            f.u.mr.x2230_vitaminPrev = (r.rnd() % 11) as i32;
        }
        let v = t::ftMr_SpecialN_VitaminRandom(gp);
        let got = unsafe { format!("V {} {} {} {} {}", i, v, f.u.mr.x222C_vitaminCurr, f.u.mr.x2230_vitaminPrev, RANDI_N) };
        assert_eq!(got, lines.next().unwrap(), "case {i} V");
        unsafe { f.anon_2210.anon_0.set_throw_flags_b0((r.rnd() & 1) as u8) };
        f.kind = if r.rnd() % 3 == 0 {
            Ft_Kind_Mario
        } else if r.rnd() & 1 != 0 {
            40
        } else {
            Ft_Kind_DrMario
        };
        f.facing_dir = if r.rnd() & 1 != 0 { 1.0 } else { -1.0 };
        unsafe { LOGB.clear() };
        t::ftMr_SpecialN_ItemFireSpawn(gp);
        let got = unsafe { format!("F {} {} {}", i, f.anon_2210.anon_0.throw_flags_b0(), LOGB) };
        assert_eq!(got.trim_end(), lines.next().unwrap().trim_end(), "case {i} F");
    }
    assert!(lines.next().is_none());
}

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}

#[test]
fn smoke_iasa() {
    let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        st::FTCO_WAIT_IASA = Some(|_| log("wait".into()));
        t::ext::FTCO_FALL_IASA_INNER = Some(|_| log("fall".into()));
    }
    f.cmd_vars[0] = 0;
    t::ftMr_SpecialN_IASA(gp);
    t::ftMr_SpecialAirN_IASA(gp);
    f.cmd_vars[0] = 1;
    t::ftMr_SpecialN_IASA(gp);
    t::ftMr_SpecialAirN_IASA(gp);
    assert_eq!(unsafe { LOG.join(",") }, "wait,fall");
}
