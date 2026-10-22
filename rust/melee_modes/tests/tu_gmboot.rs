//! tu::gmboot vs the C (state oracle): gen/turef/tu_gmboot_ref.c is the game's gmboot.c compiled
//! natively (shipping contraction) with its callees stubbed and logged; the expected lines are
//! tests/data/tu_gmboot.txt (regenerate with `python rust/melee_modes/gen/turef/tu_shard457_gm.py
//! tu_gmboot`). Each line is the callee log, the boot-scene data the functions write and the skip flag.
#![allow(static_mut_refs)]

use core::ffi::c_void;
use melee_modes::tu::gmboot as t;
use melee_types::GameModeState;

struct Rng(u32);
impl Rng {
    fn rnd(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        self.0
    }
}

const NCASES: usize = 200;
const FUNC_NAMES: [&str; 3] = ["bootOnLoad", "bootOnLeave", "memcardOnLoad"];

// Hook controls and the call log (the test is single-threaded; the hooks are plain fn pointers).
static mut LOG: String = String::new();
static mut SKIP: bool = false;
static mut TOY_OWNED: i32 = 0;
static mut PIKMIN_RET: i32 = 0;
static mut LANG: i32 = 0;
static mut LAUNCH: i32 = -1;
static mut PROBE_PRESENT: bool = false;
static mut PROBE_OK: bool = false;
static mut SENTINEL: u8 = 0;

fn log(s: &str) {
    unsafe { LOG.push_str(s) }
}

fn install_hooks() {
    unsafe {
        t::ext::GM_MAIN_LIB_SKIP_INTRO = Some(|| SKIP);
        t::ext::GM_801BF708 = Some(|a| log(&format!("801BF708({})", a)));
        t::ext::TOY_803048C0 = Some(|id| {
            log(&format!("toy48C0({})", id));
            TOY_OWNED
        });
        t::ext::TOY_803124BC = Some(|| log("toy124BC"));
        t::ext::TOY_SETUNLOCKSTATE = Some(|id, v| log(&format!("unlock({},{})", id, v as i32)));
        t::ext::LB_8001C2D8 = Some(|chan, company, game, file| {
            log(&format!("cardprobe({},{},{},{})", chan, company, game, file));
            PIKMIN_RET
        });
        t::ext::LBLANG_GETLANGUAGESETTING = Some(|| {
            log("lang");
            LANG
        });
        t::ext::GM_CHANGEGAMEMODEAFTERCURRENTSCENE = Some(|mode| log(&format!("chg({})", mode)));
        t::ext::GM_SETGAMEMODEOVERRIDE = Some(|f| {
            let who = if f as usize == t::ext::lbCardGame_DecideGameMode as usize { "dec" } else { "?" };
            log(&format!("override({})", who));
        });
        t::ext::GM_SETPENDINGGAMEMODE = Some(|mode| log(&format!("pend({})", mode)));
        t::ext::SCENELAUNCH_BOOTGAMEMODE = Some(|| {
            log("launch");
            LAUNCH
        });
        t::ext::CONTENTPROBENAME = Some(|| {
            log("probename");
            if PROBE_PRESENT {
                Some("probe.dat")
            } else {
                None
            }
        });
        t::ext::CONTENTPROBERESULT = Some(|name, archive| {
            log(&format!("probeRes({},{})", name, (!archive.is_null()) as i32));
        });
        t::ext::LBARCHIVE_LOADARCHIVE = Some(|name| {
            log(&format!("probeload({})", name));
            if PROBE_OK {
                core::ptr::addr_of_mut!(SENTINEL) as *mut c_void
            } else {
                core::ptr::null_mut()
            }
        });
    }
}

#[test]
fn matches_c() {
    let want = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/tu_gmboot.txt")).unwrap();
    install_hooks();
    let mut load = t::loadData::default();
    let mut leave = t::exitData::default();
    let mut scene = GameModeState::zeroed();
    let mut r = Rng(0x2468ACE);
    let mut lines = want.lines();
    for fi in 0..FUNC_NAMES.len() {
        for i in 0..NCASES {
            // setup_case(), in the C driver's order.
            load.x0 = r.rnd();
            load.x4 = r.rnd() as u8;
            load.mode_id = r.rnd() as u8;
            leave.x0 = r.rnd();
            leave.mode_id = r.rnd() as u8;
            unsafe {
                SKIP = r.rnd() & 1 != 0;
                TOY_OWNED = (r.rnd() & 1) as i32;
                PIKMIN_RET = (r.rnd() & 1) as i32;
                LANG = (r.rnd() % 3) as i32;
                LAUNCH = (r.rnd() % 4) as i32 - 1;
                PROBE_PRESENT = r.rnd() & 1 != 0;
                PROBE_OK = r.rnd() & 1 != 0;
            }
            scene = GameModeState::zeroed();
            scene.info.enter_data = &mut load as *mut t::loadData as *mut c_void;
            scene.info.exit_data = &mut leave as *mut t::exitData as *mut c_void;

            unsafe { LOG.clear() };
            let s: *mut GameModeState = &mut scene;
            match fi {
                0 => t::bootOnLoad(s),
                1 => t::bootOnLeave(s),
                _ => t::memcardOnLoad(s),
            }
            let got = unsafe {
                format!(
                    "{}.{} {}|log={}|load={},{},{}|leave={},{}|skip={}",
                    fi,
                    i,
                    FUNC_NAMES[fi],
                    LOG,
                    load.x0,
                    load.x4,
                    load.mode_id,
                    leave.x0,
                    leave.mode_id,
                    SKIP as i32,
                )
            };
            let line = lines.next().expect("fewer lines than cases");
            assert_eq!(got, line, "function {} case {}", FUNC_NAMES[fi], i);
        }
    }
    assert!(lines.next().is_none(), "more lines than cases");
}
