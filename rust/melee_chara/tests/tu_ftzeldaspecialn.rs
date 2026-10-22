//! tu::ftzeldaspecialn vs the C (state oracle): gen/turef/tu_ftzeldaspecialn_ref.c is the game's
//! ftzeldaspecialn.c compiled natively (shipping contraction) on a Fighter filled from the same
//! xorshift stream as here; the expected lines are tests/data/tu_ftzeldaspecialn.txt (regenerate
//! with `python rust/melee_chara/gen/turef/tu_shard457.py tu_ftzeldaspecialn`). Each line is the
//! callee log, the callback identities and the Fighter fields the functions write.
#![allow(static_mut_refs)]

use melee_chara::chars::cb;
use melee_chara::stubs as st;
use melee_chara::tu::ftzeldaspecialn as t;
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

const NCASES: usize = 200;
const FUNC_NAMES: [&str; 11] = [
    "ftZd_SpecialN_8013A830",
    "ftZd_SpecialN_8013A8AC",
    "ftZd_SpecialN_Enter",
    "ftZd_SpecialAirN_Enter",
    "ftZd_SpecialN_Anim",
    "ftZd_SpecialAirN_Anim",
    "ftZd_SpecialN_Coll",
    "ftZd_SpecialAirN_Coll",
    "ftZd_SpecialN_8013AC88",
    "ftZd_SpecialN_8013AD1C",
    "ftZd_SpecialAirN_Phys",
];

// Hook controls and the call log (the test is single-threaded; the hooks are plain fn pointers).
static mut G_FRAMES: i32 = 0;
static mut G_GROUND: i32 = 0;
static mut G_AIR: i32 = 0;
static mut LOG: String = String::new();
static mut DAT: *const ftZelda_DatAttrs = core::ptr::null();

fn log(s: &str) {
    unsafe { LOG.push_str(s) }
}

fn bits(f: f32) -> String {
    format!("{:08x}", f.to_bits())
}

fn cb_name(c: HSD_GObjEvent) -> &'static str {
    let Some(f) = c else { return "0" };
    let a = f as usize;
    let table: [(usize, &'static str); 6] = [
        (t::ftZd_SpecialN_8013A830 as usize, "nA830"),
        (t::ftZd_SpecialN_8013A8AC as usize, "nA8AC"),
        (t::ftZd_SpecialN_8013ADB0_cb as usize, "nADB0"),
        (t::ftZd_Init_801393AC as usize, "init"),
        (cb::ext__efLib_PauseAll as usize, "pause"),
        (cb::ext__efLib_ResumeAll as usize, "resume"),
    ];
    table.iter().find(|(p, _)| *p == a).map(|(_, n)| *n).unwrap_or("?")
}

fn hook_frames() -> i32 {
    log("framesrem");
    unsafe { G_FRAMES }
}

fn hook_air() -> i32 {
    log("81D0C");
    unsafe { G_AIR }
}

fn hook_ground() -> i32 {
    log("82708");
    unsafe { G_GROUND }
}

fn install_hooks() {
    unsafe {
    st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, msid, flags, start, speed, blend, arg3| {
        log(&format!(
            "cms({},{},{},{},{},{})",
            msid,
            flags,
            bits(start),
            bits(speed),
            bits(blend),
            if arg3.is_null() { "0" } else { "g" }
        ));
    });
    st::FTANIM_8006EBA4 = Some(|_| log("anim8006EBA4"));
    st::FTANIM_ISFRAMESREMAINING = Some(|_| hook_frames());
    st::FT_8008A2BC = Some(|_| log("8008A2BC"));
    st::FTCO_FALL_ENTER = Some(|_| log("cofall"));
    st::FT_80081D0C = Some(|_| hook_air());
    st::FT_80082708 = Some(|_| hook_ground());
    st::FTCOLL_8007AEF8 = Some(|_| log("coll8007AEF8"));
    t::ext::FTCOLL_CREATEREFLECTHIT = Some(|_, desc, c| {
        let off = desc as isize - unsafe { DAT } as isize;
        log(&format!("reflect({},{})", off, cb_name(c)));
    });
    t::ext::EFSYNC_SPAWN = Some(|id, _, _| {
        log(&format!("efsync({})", id));
        core::ptr::null_mut()
    });
    t::ext::FTCOMMON_FALL = Some(|_, g, tv| log(&format!("fall({},{})", bits(g), bits(tv))));
    t::ext::FTCOMMON_8007D5D4 = Some(|_| log("d5d4"));
    t::ext::FTCOMMON_8007D7FC = Some(|_| log("d7fc"));
    }
}

/// The C driver's callback table (tu_ftzeldaspecialn_ref.c CB_TABLE), same order.
fn cb_table() -> [HSD_GObjEvent; 7] {
    [
        None,
        Some(t::ftZd_SpecialN_8013A830),
        Some(t::ftZd_SpecialN_8013A8AC),
        Some(t::ftZd_SpecialN_8013ADB0_cb),
        Some(t::ftZd_Init_801393AC),
        Some(cb::ext__efLib_PauseAll),
        Some(cb::ext__efLib_ResumeAll),
    ]
}

fn call(f: usize, g: *mut HSD_GObj) {
    unsafe {
        match f {
            0 => t::ftZd_SpecialN_8013A830(g),
            1 => t::ftZd_SpecialN_8013A8AC(g),
            2 => t::ftZd_SpecialN_Enter(g),
            3 => t::ftZd_SpecialAirN_Enter(g),
            4 => t::ftZd_SpecialN_Anim(g),
            5 => t::ftZd_SpecialAirN_Anim(g),
            6 => t::ftZd_SpecialN_Coll(g),
            7 => t::ftZd_SpecialAirN_Coll(g),
            8 => t::ftZd_SpecialN_8013AC88(g),
            9 => t::ftZd_SpecialN_8013AD1C(g),
            10 => t::ftZd_SpecialAirN_Phys(g),
            _ => unreachable!(),
        }
    }
}

#[test]
fn matches_c() {
    let want = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/tu_ftzeldaspecialn.txt"))
        .unwrap();
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let mut da: Box<ftZelda_DatAttrs> = Box::new(ftZelda_DatAttrs::zeroed());
    let mut parts: std::vec::Vec<FighterBone> = (0..128).map(|_| unsafe { core::mem::zeroed() }).collect();
    let mut cdata: Box<ftCommonData> = Box::new(unsafe { core::mem::zeroed() });
    let mut items_a: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let mut items_b: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let mut joints: std::vec::Vec<u8> = vec![0; 128 * 64];
    let joint_base = joints.as_mut_ptr();
    let da_ptr: *mut ftZelda_DatAttrs = &mut *da;
    let fighter_ptr: *mut Fighter = &mut *f;
    let self_obj: *mut HSD_GObj = &mut *g;
    let ia: *mut HSD_GObj = &mut *items_a;
    let ib: *mut HSD_GObj = &mut *items_b;
    unsafe {
        DAT = da_ptr;
        melee_fighter::hooks::P_FT_COMMON_DATA = &*cdata as *const ftCommonData;
    }
    install_hooks();

    let cbs = cb_table();
    let mut r = Rng(0x2468ACE);
    let mut lines = want.lines();
    for fi in 0..FUNC_NAMES.len() {
        for i in 0..NCASES {
            // setup_case(), in the C driver's order.
            unsafe {
                *fighter_ptr = Fighter::zeroed();
                (*self_obj).user_data = fighter_ptr as *mut core::ffi::c_void;
                (*fighter_ptr).dat_attrs = da_ptr as *mut core::ffi::c_void;
                let w = da_ptr as *mut u32;
                for k in 0..core::mem::size_of::<ftZelda_DatAttrs>() / 4 {
                    *w.add(k) = r.rf().to_bits();
                }
                (*fighter_ptr).parts = parts.as_mut_ptr();
                for k in 0..128 {
                    parts[k].joint = joint_base.add(k * 64) as *mut HSD_JObj;
                }
            }
            let f = &mut *f;
            f.facing_dir = r.rf();
            f.self_vel.x = r.rf();
            f.self_vel.y = r.rf();
            f.gr_vel = r.rf();
            f.cur_anim_frame = r.rf();
            for k in 0..4 {
                f.cmd_vars[k] = r.rnd() % 4;
            }
            f.set_x2219_b0((r.rnd() & 1) as u8);
            f.set_reflecting((r.rnd() & 1) as u8);
            f.x1968_jumpsUsed = (r.rnd() % 4) as u8;
            f.motion_id = (340 + r.rnd() % 12) as _;
            f.ground_or_air = (r.rnd() % 2) as _;
            f.input.held_buttons[0] = r.rnd();
            f.co_attrs.aerial_friction = r.rf();
            f.co_attrs.terminal_velocity = r.rf();
            let choice = r.rnd() % 3;
            f.u.zd.x222C = match choice {
                0 => core::ptr::null_mut(),
                1 => ia,
                _ => ib,
            };
            f.mv.zd.specials.x0 = (r.rnd() % 5) as i32;
            f.mv.zd.specials.x4 = (r.rnd() % 5) as i32;
            f.mv.zd.specials.x8 = (r.rnd() % 5) as i32;
            f.mv.zd.specials.xC = (r.rnd() % 5) as i32;
            unsafe {
                G_FRAMES = (r.rnd() & 1) as i32;
                G_GROUND = (r.rnd() & 1) as i32;
                G_AIR = (r.rnd() & 1) as i32;
            }
            // The C draws G_ITEM and G_OWNER too (its item stubs are unused by this TU).
            r.rnd();
            r.rnd();
            f.accessory4_cb = cbs[r.rnd() as usize % cbs.len()];
            f.death2_cb = cbs[r.rnd() as usize % cbs.len()];
            f.take_dmg_cb = cbs[r.rnd() as usize % cbs.len()];
            f.pre_hitlag_cb = cbs[r.rnd() as usize % cbs.len()];
            f.post_hitlag_cb = cbs[r.rnd() as usize % cbs.len()];
            f.x74_self_accel.x = r.rf();
            f.x74_self_accel.y = r.rf();
            cdata.aerial_friction_oob = r.rf();

            unsafe { LOG.clear() };
            call(fi, self_obj);
            let got = unsafe {
                let u = if f.u.zd.x222C.is_null() {
                    "0"
                } else if f.u.zd.x222C == ia {
                    "A"
                } else if f.u.zd.x222C == ib {
                    "B"
                } else if f.u.zd.x222C == self_obj {
                    "self"
                } else {
                    "?"
                };
                format!(
                    "{}.{} {}|log={}|acc={}|death={}|take={}|pre={}|post={}|x2219={}|cmd={},{},{},{}|sv={},{}|\
                     gr={}|fd={}|cur={}|sp={},{},{},{}|refl={}|u={}|jumps={}|held={}|acc74={},{}|ret=0",
                    fi,
                    i,
                    FUNC_NAMES[fi],
                    LOG,
                    cb_name(f.accessory4_cb),
                    cb_name(f.death2_cb),
                    cb_name(f.take_dmg_cb),
                    cb_name(f.pre_hitlag_cb),
                    cb_name(f.post_hitlag_cb),
                    f.x2219_b0(),
                    f.cmd_vars[0],
                    f.cmd_vars[1],
                    f.cmd_vars[2],
                    f.cmd_vars[3],
                    bits(f.self_vel.x),
                    bits(f.self_vel.y),
                    bits(f.gr_vel),
                    bits(f.facing_dir),
                    bits(f.cur_anim_frame),
                    f.mv.zd.specials.x0,
                    f.mv.zd.specials.x4,
                    f.mv.zd.specials.x8,
                    f.mv.zd.specials.xC,
                    f.reflecting(),
                    u,
                    f.x1968_jumpsUsed,
                    f.input.held_buttons[0],
                    bits(f.x74_self_accel.x),
                    bits(f.x74_self_accel.y),
                )
            };
            let line = lines.next().expect("fewer lines than cases");
            assert_eq!(got, line, "function {} case {}", FUNC_NAMES[fi], i);
        }
    }
    assert!(lines.next().is_none(), "more lines than cases");
}
