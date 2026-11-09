//! tu::gm_1A33 vs the C (state oracle): gen/turef/tu_gm_1A33_ref.c is the game's gm_1A33.c compiled natively
//! (shipping contraction) with the camera UI, the SIS text engine and the snap/menu/pad queries stubbed and
//! logged; the expected lines are tests/data/tu_gm_1A33.txt (regenerate with
//! `python rust/melee_modes/gen/turef/tu_shard537_gm.py tu_gm_1A33`). Each line is the callee log (float
//! arguments as bit patterns), the scene's result word and text slots, and the texts' alignment/size.
#![allow(static_mut_refs)]

mod tu537;
use core::ffi::c_void;
use melee_modes::tu::gm_1A33 as t;
use melee_types::*;
use std::vec::Vec;
use tu537::Rng;

const FUNC_NAMES: [&str; 4] = ["gmCamera_801A33BC", "OnFrame", "OnEnter", "OnExit"];
const NCASES: [usize; 4] = [300, 1500, 300, 50];

static mut LOG: String = String::new();
static mut TX: Vec<HSD_Text> = Vec::new();
static mut NTX: usize = 0;
static mut X0V: u32 = 0;
static mut SNAP0: i32 = 0;
static mut SNAP1: i32 = 0;
static mut MN: bool = false;
static mut SP10: i32 = 0;
static mut SPC: i32 = 0;
static mut V2640: i32 = 0;
static mut TRIG: u64 = 0;

fn bits(f: f32) -> String {
    format!("{:08x}", f.to_bits())
}
fn log(s: &str) {
    unsafe { LOG.push_str(s) }
}
unsafe fn txid(p: *mut HSD_Text) -> i32 {
    if p.is_null() {
        -1
    } else {
        p.offset_from(TX.as_mut_ptr()) as i32
    }
}

fn install_hooks() {
    use t::ext::*;
    unsafe {
        GMCAMERA_801A25C8 = Some(|| log("25C8;"));
        GMCAMERA_801A2334 = Some(|a, x, y, w, h| unsafe {
            let n = NTX;
            NTX += 1;
            TX[n] = HSD_Text::zeroed();
            log(&format!("2334({},{},{},{},{})=x{};", a, bits(x), bits(y), bits(w), bits(h), n));
            &mut TX[n] as *mut HSD_Text
        });
        GMCAMERA_801A253C = Some(|a, b| unsafe {
            log("253C;");
            *a = SP10;
            *b = SPC;
        });
        GMCAMERA_801A2640 = Some(|| {
            log("2640;");
            unsafe { V2640 }
        });
        GMCAMERA_801A2650 = Some(|| log("2650;"));
        GMCAMERA_801A2224 = Some(|p, v| unsafe {
            log(&format!("2224({},{});", if p == &mut TX[15] as *mut HSD_Text as *mut u8 { "digits" } else { "?" }, v));
            p
        });
        HSD_SISLIB_803A5ACC = Some(|a, b, x, y, z, w, h| unsafe {
            let n = NTX;
            NTX += 1;
            TX[n] = HSD_Text::zeroed();
            log(&format!("5ACC({},{},{},{},{},{},{})=x{};", a, b, bits(x), bits(y), bits(z), bits(w), bits(h), n));
            &mut TX[n] as *mut HSD_Text
        });
        HSD_SISLIB_803A6368 = Some(|tx, v| unsafe { log(&format!("6368(x{},{});", txid(tx), v)) });
        HSD_SISLIB_803A6530 = Some(|a, b, c| unsafe {
            log(&format!("6530({},{},{});", a, b, c));
            &mut TX[15] as *mut HSD_Text as *mut u8
        });
        HSD_SISLIB_803A660C = Some(|a, b, c| log(&format!("660C({},{},{});", a, b, c)));
        HSD_SISLIB_803A5CC4 = Some(|tx| unsafe { log(&format!("5CC4(x{});", txid(tx))) });
        LBSNAP_8001D338 = Some(|i| unsafe {
            log(&format!("snap({});", i));
            if i == 0 {
                SNAP0
            } else {
                SNAP1
            }
        });
        MN_8022F218 = Some(|| {
            log("mnF218;");
            unsafe { MN }
        });
        MN_8022F268 = Some(|| log("mnF268;"));
        GM_801A4B60 = Some(|| log("4B60;"));
        LBAUDIOAX_80024030 = Some(|id| log(&format!("sfx({});", id)));
        GM_GETBUTTONSTRIGGERED = Some(|idx| unsafe {
            log(&format!("trig({});", idx));
            TRIG
        });
    }
}

#[test]
fn matches_c() {
    let want = tu537::data("tu_gm_1A33");
    install_hooks();
    let mut r = Rng::new();
    let mut lines = want.lines();
    for f in 0..FUNC_NAMES.len() {
        for i in 0..NCASES[f] {
            unsafe {
                // setup_case(), in the C driver's order.
                TX = (0..16).map(|_| HSD_Text::zeroed()).collect();
                NTX = 0;
                for k in 0..3usize {
                    t::gmCamera_VsCameraTextLayout.x4.text[k] =
                        if r.rnd() % 2 != 0 { &mut TX[10 + k] as *mut HSD_Text } else { core::ptr::null_mut() };
                }
                X0V = r.rnd();
                t::gmCamera_VsCameraTextLayout.x0 = &raw mut X0V;
                SNAP0 = (r.rnd() % 4 == 0) as i32;
                SNAP1 = (r.rnd() % 4 == 0) as i32;
                MN = r.rnd() % 3 == 0;
                SP10 = (r.rnd() % 5) as i32;
                SPC = (r.rnd() % 20000) as i32;
                V2640 = (r.rnd() % 50) as i32;
                TRIG = 0;
                if r.rnd() % 2 != 0 {
                    TRIG |= t::HSD_PAD_A;
                }
                if r.rnd() % 3 == 0 {
                    TRIG |= t::HSD_PAD_START;
                }
                if r.rnd() % 2 != 0 {
                    TRIG |= t::PAD_BUTTON_B;
                }
                if r.rnd() % 2 != 0 {
                    TRIG |= 1u64 << 40;
                }
                let use_x0 = r.rnd() % 4 != 0;
                LOG.clear();
                match f {
                    0 => t::gmCamera_801A33BC(),
                    1 => t::gm_Scene_CameraVs_OnFrame(),
                    2 => t::gm_Scene_CameraVs_OnEnter(if use_x0 { &raw mut X0V as *mut c_void } else { core::ptr::null_mut() }),
                    _ => t::gm_Scene_CameraVs_OnExit(core::ptr::null_mut()),
                }
                let d = &t::gmCamera_VsCameraTextLayout;
                let x0 = if d.x0 == &raw mut X0V {
                    1
                } else if d.x0.is_null() {
                    0
                } else {
                    2
                };
                let mut got = format!(
                    "{}.{} {}|log={}|x0={},{}|slots={},{},{}",
                    f,
                    i,
                    FUNC_NAMES[f],
                    LOG,
                    x0,
                    X0V,
                    txid(d.x4.x0.slot_a),
                    txid(d.x4.x0.slot_b),
                    txid(d.x4.x0.bottom_text)
                );
                for k in 0..NTX {
                    let x = &TX[k];
                    got += &format!(
                        "|X{}:{},{},{},{},{}",
                        k,
                        x.default_alignment,
                        x.default_kerning,
                        x.default_fitting,
                        bits(x.font_size.x),
                        bits(x.font_size.y)
                    );
                }
                let line = lines.next().expect("fewer lines than cases");
                assert_eq!(got, line, "function {} case {}", FUNC_NAMES[f], i);
            }
        }
    }
    assert!(lines.next().is_none(), "more lines than cases");
}
