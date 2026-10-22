//! tu::ftzeldaspeciallw vs the C (state oracle): gen/turef/tu_ftzeldaspeciallw_ref.c is the game's
//! ftzeldaspeciallw.c compiled natively (shipping contraction) on a Fighter filled from the same
//! xorshift stream as here; the expected lines are tests/data/tu_ftzeldaspeciallw.txt (regenerate
//! with `python rust/melee_chara/gen/turef/tu_shard457.py tu_ftzeldaspeciallw`). Each line is the
//! callee log, the callback identities, the Fighter fields the functions write and the return value.
#![allow(static_mut_refs)]

use melee_chara::chars::cb;
use melee_chara::stubs as st;
use melee_chara::tu::ftzeldaspeciallw as t;
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
const JSTRIDE: usize = 64;
const FUNC_NAMES: [&str; 9] = [
    "ftZd_SpecialLw_8013ADB4",
    "ftZd_SpecialLw_8013AE30",
    "ftZd_SpecialLw_8013AEAC",
    "ftZd_SpecialLw_Enter",
    "ftZd_SpecialAirLw_Enter",
    "ftZd_SpecialLw_8013B4D8",
    "ftZd_SpecialLw_8013B540",
    "ftZd_SpecialLw_8013B574",
    "ftZd_SpecialLw_8013B5EC",
];

// Hook controls and the call log (the test is single-threaded; the hooks are plain fn pointers).
static mut G_FRAMES: i32 = 0;
static mut G_GROUND: i32 = 0;
static mut G_AIR: i32 = 0;
static mut LOG: String = String::new();
static mut JOINT_BASE: usize = 0;
static mut ITEM_A: usize = 0;
static mut ITEM_B: usize = 0;

fn log(s: &str) {
    unsafe { LOG.push_str(s) }
}

fn bits(f: f32) -> String {
    format!("{:08x}", f.to_bits())
}

fn cb_name(c: Option<usize>) -> &'static str {
    let Some(a) = c else { return "0" };
    let table: [(usize, &'static str); 8] = [
        (t::ftZd_SpecialLw_8013ADB4 as usize, "lADB4"),
        (t::ftZd_SpecialLw_8013AE30 as usize, "lAE30"),
        (t::ftZd_SpecialLw_8013AEAC as usize, "lAEAC"),
        (t::ftZd_Init_801393AC as usize, "init"),
        (cb::ext__efLib_PauseAll as usize, "pause"),
        (cb::ext__efLib_ResumeAll as usize, "resume"),
        (t::ext::ftSk_SpecialLw_80114758 as usize, "skLw"),
        (sk_callback() as usize, "skLw"),
    ];
    table.iter().find(|(p, _)| *p == a).map(|(_, n)| *n).unwrap_or("?")
}

fn item_name(p: usize) -> &'static str {
    unsafe {
        if p == 0 {
            "0"
        } else if p == ITEM_A {
            "A"
        } else if p == ITEM_B {
            "B"
        } else {
            "?"
        }
    }
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
        st::FTANIM_ISFRAMESREMAINING = Some(|_| {
            log("framesrem");
            G_FRAMES
        });
        st::FT_8008A2BC = Some(|_| log("8008A2BC"));
        st::FTCO_FALL_ENTER = Some(|_| log("cofall"));
        st::FT_80081D0C = Some(|_| {
            log("81D0C");
            G_AIR
        });
        st::FT_80082708 = Some(|_| {
            log("82708");
            G_GROUND
        });
        st::FTCOLL_8007AEF8 = Some(|_| log("coll8007AEF8"));
        t::ext::EFSYNC_SPAWN = Some(|id, _, _| {
            log(&format!("efsync({})", id));
            core::ptr::null_mut()
        });
        t::ext::LB_8000B1CC = Some(|j, _pos0, out| {
            let i = ((j as usize - JOINT_BASE) / JSTRIDE) as i32;
            (*out).x = i as f32;
            (*out).y = i as f32 * 2.0;
            (*out).z = 3.0;
            log(&format!("lb8000B1CC(j{})", i));
        });
        t::ext::LB_800119DC = Some(|v, n, a, b, c| {
            log(&format!(
                "lb119DC({},{},{},{},{},{},{})",
                bits((*v).x),
                bits((*v).y),
                bits((*v).z),
                n,
                bits(a),
                bits(b),
                bits(c)
            ));
        });
        t::ext::IT_802C3D44 = Some(|item| log(&format!("it3D44({})", item_name(item as usize))));
        t::ext::FTCOMMON_8007EFC8 = Some(|_, cb| log(&format!("8007EFC8({})", cb_name(Some(cb as usize)))));
    }
}

/// The C driver's callback table (tu_ftzeldaspeciallw_ref.c CB_TABLE), same order.
fn cb_table() -> [HSD_GObjEvent; 8] {
    [
        None,
        Some(t::ftZd_SpecialLw_8013ADB4),
        Some(t::ftZd_SpecialLw_8013AE30),
        Some(t::ftZd_SpecialLw_8013AEAC),
        Some(t::ftZd_Init_801393AC),
        Some(cb::ext__efLib_PauseAll),
        Some(cb::ext__efLib_ResumeAll),
        Some(sk_callback()),
    ]
}

/// `ftSk_SpecialLw_80114758` as a callback value (its address is all the port stores).
fn sk_callback() -> unsafe extern "C" fn(*mut HSD_GObj) {
    unsafe extern "C" fn sk(g: *mut HSD_GObj) {
        t::ext::ftSk_SpecialLw_80114758(g)
    }
    sk
}

/// Runs function `f`, returning the C-style return value (0 for void functions).
fn call(f: usize, g: *mut HSD_GObj) -> i32 {
    unsafe {
        match f {
            0 => t::ftZd_SpecialLw_8013ADB4(g),
            1 => t::ftZd_SpecialLw_8013AE30(g),
            2 => t::ftZd_SpecialLw_8013AEAC(g),
            3 => t::ftZd_SpecialLw_Enter(g),
            4 => t::ftZd_SpecialAirLw_Enter(g),
            5 => t::ftZd_SpecialLw_8013B4D8(g),
            6 => return t::ftZd_SpecialLw_8013B540(g),
            7 => return t::ftZd_SpecialLw_8013B574(g) as i32,
            8 => t::ftZd_SpecialLw_8013B5EC(g),
            _ => unreachable!(),
        }
    }
    0
}

#[test]
fn matches_c() {
    let want = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/tu_ftzeldaspeciallw.txt"))
        .unwrap();
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let mut da: Box<ftZelda_DatAttrs> = Box::new(ftZelda_DatAttrs::zeroed());
    let mut parts: std::vec::Vec<FighterBone> = (0..128).map(|_| unsafe { core::mem::zeroed() }).collect();
    let mut cdata: Box<ftCommonData> = Box::new(unsafe { core::mem::zeroed() });
    let mut items_a: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let mut items_b: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let mut joints: std::vec::Vec<u8> = vec![0; 128 * JSTRIDE];
    let joint_base = joints.as_mut_ptr();
    let da_ptr: *mut ftZelda_DatAttrs = &mut *da;
    let fighter_ptr: *mut Fighter = &mut *f;
    let self_obj: *mut HSD_GObj = &mut *g;
    let ia: *mut HSD_GObj = &mut *items_a;
    let ib: *mut HSD_GObj = &mut *items_b;
    unsafe {
        JOINT_BASE = joint_base as usize;
        ITEM_A = ia as usize;
        ITEM_B = ib as usize;
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
                    parts[k].joint = joint_base.add(k * JSTRIDE) as *mut HSD_JObj;
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
            // The C draws G_ITEM and G_OWNER too (its item stubs are unused by these functions).
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
            let ret = call(fi, self_obj);
            let got = unsafe {
                let u = item_name(f.u.zd.x222C as usize);
                format!(
                    "{}.{} {}|log={}|acc={}|death={}|take={}|pre={}|post={}|x2219={}|cmd={},{},{},{}|sv={},{}|\
                     gr={}|fd={}|cur={}|sp={},{},{},{}|refl={}|u={}|jumps={}|held={}|acc74={},{}|ret={}",
                    fi,
                    i,
                    FUNC_NAMES[fi],
                    LOG,
                    cb_name(f.accessory4_cb.map(|c| c as usize)),
                    cb_name(f.death2_cb.map(|c| c as usize)),
                    cb_name(f.take_dmg_cb.map(|c| c as usize)),
                    cb_name(f.pre_hitlag_cb.map(|c| c as usize)),
                    cb_name(f.post_hitlag_cb.map(|c| c as usize)),
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
                    ret,
                )
            };
            let line = lines.next().expect("fewer lines than cases");
            assert_eq!(got, line, "function {} case {}", FUNC_NAMES[fi], i);
        }
    }
    assert!(lines.next().is_none(), "more lines than cases");
}
