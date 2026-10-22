//! tu::gm_1A9B vs the C (state oracle): gen/turef/tu_gm_1A9B_ref.c is the game's gm_1A9B.c compiled natively
//! (shipping contraction) with the gobj / audio / movie / scene-query calls stubbed and logged; the expected
//! lines are tests/data/tu_gm_1A9B.txt (regenerate with
//! `python rust/melee_modes/gen/turef/tu_shard537_gm.py tu_gm_1A9B`). OnEnter sweeps every (game-over mode,
//! character) pair, so each of the 78 movie names is compared through the logged file name.
#![allow(static_mut_refs)]

mod tu537;
use std::vec::Vec;
use melee_modes::tu::gm_1A9B as t;
use melee_types::*;
use tu537::Rng;

const FUNC_NAMES: [&str; 2] = ["OnEnter", "OnFrame"];
const NCASES: [usize; 2] = [520, 2000];

static mut LOG: String = String::new();
static mut GP: Vec<HSD_GObj> = Vec::new();
static mut NG: usize = 0;
static mut SOBJ: Option<HSD_SObj> = None;
static mut MODE: i32 = 0;
static mut BF050: i32 = 0;
static mut LEVEL: i32 = 0;
static mut CKIND: i32 = 0;
static mut REMAP: i32 = 0;
static mut PORT: i32 = 0;
static mut TRIG: u64 = 0;

fn bits(f: f32) -> String {
    format!("{:08x}", f.to_bits())
}
fn log(s: &str) {
    unsafe { LOG.push_str(s) }
}
unsafe fn gid(g: *mut HSD_GObj) -> i32 {
    g.offset_from(GP.as_mut_ptr()) as i32
}

fn install_hooks() {
    use t::ext::*;
    unsafe {
        GOBJ_CREATE = Some(|c, pl, pr| unsafe {
            let n = NG;
            NG += 1;
            GP[n] = HSD_GObj::zeroed();
            log(&format!("gcreate({},{},{})=g{};", c, pl, pr, n));
            &mut GP[n] as *mut HSD_GObj
        });
        HSD_SOBJLIB_803A55DC = Some(|g, w, h, n| unsafe { log(&format!("55DC(g{},{},{},{});", gid(g), w, h, n)) });
        HSD_GOBJOBJECT_80390A70 = Some(|g, kind, obj| unsafe {
            log(&format!("gobj(g{},{},{});", gid(g), kind, if obj.is_null() { "null" } else { "?" }));
        });
        GOBJ_SETUPGXLINK = Some(|g, cb, gx, prio| unsafe {
            let who = if cb == lbMthp8001F928 as usize { "mthp" } else { "?" };
            log(&format!("gxlink(g{},{},{},{});", gid(g), who, gx, prio));
        });
        HSD_SOBJLIB_804D7960 = Some(|| 9);
        LBAUDIOAX_80026F2C = Some(|v| log(&format!("6F2C({});", v)));
        LBAUDIOAX_8002702C = Some(|f, m| log(&format!("702C({},{});", f, m as u32)));
        LBAUDIOAX_80027168 = Some(|| log("7168;"));
        LBAUDIOAX_80027648 = Some(|| log("7648;"));
        LBAUDIOAX_80023694 = Some(|| {
            log("3694;");
            0
        });
        LBAUDIOAX_800237A8 = Some(|id, vol, pan| {
            log(&format!("37A8({:x},{},{});", id, vol, pan));
            0
        });
        LBAUDIOAX_80024030 = Some(|id| log(&format!("sfx({});", id)));
        LBBGFLASH_800209F4 = Some(|| log("09F4;"));
        LBBGFLASH_8002063C = Some(|n| log(&format!("063C({});", n)));
        LBMTHP_8001F890 = Some(|g| unsafe {
            log(&format!("F890(g{});", gid(g)));
            SOBJ.as_mut().unwrap() as *mut HSD_SObj
        });
        LBMTHP_8001FAA0 = Some(|f, w, h| log(&format!("FAA0({},{},{});", f, w, h)));
        GM_801BEFB0 = Some(|| {
            log("EFB0;");
            unsafe { CKIND }
        });
        GM_801BF010 = Some(|| {
            log("F010;");
            unsafe { PORT }
        });
        GM_801BF050 = Some(|| {
            log("F050;");
            unsafe { BF050 }
        });
        GM_GETCURRENTGAMEMODE = Some(|| {
            log("mode;");
            unsafe { MODE as u8 }
        });
        GM_8017DFF4 = Some(|v| {
            log(&format!("DFF4({});", v));
            unsafe { LEVEL }
        });
        GM_801A6630 = Some(|v| log(&format!("6630({});", v)));
        GM_MEXVANILLAKIND = Some(|kind, ck| {
            let r = (kind + unsafe { REMAP }) % 26;
            log(&format!("mex({},{})={};", kind, ck, r));
            r
        });
        GM_GETBUTTONSTRIGGERED = Some(|idx| unsafe {
            log(&format!("trig({});", idx));
            TRIG
        });
    }
}

#[test]
fn matches_c() {
    let want = tu537::data("tu_gm_1A9B");
    install_hooks();
    let modes = [GM_CLASSIC_GOVER as i32, GM_ADVENTURE_GOVER as i32, 23, GM_DEBUG_GOVER as i32];
    let mut r = Rng::new();
    let mut lines = want.lines();
    for f in 0..FUNC_NAMES.len() {
        for i in 0..NCASES[f] {
            unsafe {
                NG = 0;
                GP = (0..4).map(|_| HSD_GObj::zeroed()).collect();
                let mut s = HSD_SObj::zeroed();
                s.x40 = r.rnd();
                SOBJ = Some(s);
                REMAP = (r.rnd() % 3) as i32;
                BF050 = GM_CLASSIC_GOVER as i32 + (r.rnd() % 3) as i32;
                LEVEL = (r.rnd() % 6) as i32;
                PORT = (r.rnd() % 6) as i32;
                TRIG = 0;
                if r.rnd() % 2 != 0 {
                    TRIG |= t::PAD_BUTTON_A;
                }
                if r.rnd() % 3 == 0 {
                    TRIG |= t::PAD_BUTTON_START;
                }
                if r.rnd() % 2 != 0 {
                    TRIG |= 1u64 << 33;
                }
                if f == 0 {
                    CKIND = (i % 26) as i32;
                    MODE = modes[(i / 26) % 4];
                } else {
                    MODE = if r.rnd() % 2 != 0 { GM_DEBUG_GOVER as i32 } else { GM_CLASSIC_GOVER as i32 };
                    let a = r.rnd();
                    t::gmRegend_InputDelayTimer = (if a % 3 == 0 { 0 } else { a % 4 }) as u8;
                    let b = r.rnd();
                    t::gmRegend_ExitTimer = (if b % 3 == 0 { 0 } else { (b >> 4) % 3 }) as u8;
                    CKIND = 0;
                }
                LOG.clear();
                if f == 0 {
                    t::gm_Scene_Congrats_OnEnter(core::ptr::null_mut());
                } else {
                    t::gm_Scene_Congrats_OnFrame();
                }
                let so = SOBJ.as_ref().unwrap();
                let g0 = GP[0].gxlink_prios;
                let got = format!(
                    "{}.{} {}|log={}|t={},{}|sobj={},{},{:x}|g0={:x}",
                    f,
                    i,
                    FUNC_NAMES[f],
                    LOG,
                    t::gmRegend_InputDelayTimer,
                    t::gmRegend_ExitTimer,
                    bits(so.x10),
                    bits(so.x14),
                    so.x40,
                    g0
                );
                let line = lines.next().expect("fewer lines than cases");
                assert_eq!(got, line, "function {} case {}", FUNC_NAMES[f], i);
            }
        }
    }
    assert!(lines.next().is_none(), "more lines than cases");
}
