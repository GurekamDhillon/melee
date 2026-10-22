//! tu::gmcamera vs the C (state oracle): gen/turef/tu_gmcamera_ref.c is the game's gmcamera.c compiled
//! natively (shipping contraction) with its externals stubbed and logged; the expected lines are
//! tests/data/tu_gmcamera.txt (regenerate with `python rust/melee_modes/gen/turef/tu_shard457_gm.py
//! tu_gmcamera`). Each line is the callee log, the VS camera UI state, the pause/snap state, the joint
//! and text state, the return value and the digit buffer.
#![allow(static_mut_refs)]

use core::ffi::c_void;
use melee_modes::tu::gmcamera as t;
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
const FUNC_NAMES: [&str; 23] = [
    "gmCamera_801A2224",
    "gmCamera_801A2334",
    "gmCamera_801A253C",
    "gmCamera_801A25C8",
    "gmCamera_801A2640",
    "gmCamera_801A2650",
    "gmCamera_801A26C0",
    "gmCamera_801A2798",
    "gmCamera_801A2800",
    "gmCamera_801A28AC",
    "gmCamera_801A292C",
    "gmCamera_801A2AAC",
    "gmCamera_801A2BB0",
    "gmCamera_801A2BF0_get_translate_x",
    "gmCamera_801A2BF0",
    "gmCamera_801A2D44",
    "gmCamera_801A2FBC",
    "gmCamera_801A2FFC",
    "gmCamera_801A3048",
    "gmCamera_801A3098",
    "gmCamera_801A30E4",
    "fn_801A31D8",
    "gmCamera_801A31FC",
];

const JSTRIDE: usize = core::mem::size_of::<HSD_JObj>();
const GSTRIDE: usize = core::mem::size_of::<HSD_GObj>();
const TSTRIDE: usize = core::mem::size_of::<HSD_Text>();

// Driver-owned objects and hook controls (the test is single-threaded; the hooks are plain fn pointers).
static mut LOG: String = String::new();
static mut JOBJS: usize = 0; // base of 16 HSD_JObj
static mut GOBJS: usize = 0; // base of 4 HSD_GObj
static mut TEXTS: usize = 0; // base of 3 HSD_Text
static mut ARCH: usize = 0;
static mut IMG: usize = 0;
static mut DIGITS: [u8; 16] = [0; 16];
static mut SC: usize = 0;
static mut JSLOTPTR: usize = 0;
static mut MDESC: usize = 0;
static mut SLARR: [u8; 4096] = [0; 4096];
static mut PAUSE: [i32; 4] = [0; 4];
static mut SNAP40C: [i32; 2] = [0; 2];
static mut SNAP3B0: [i32; 2] = [0; 2];
static mut SNAP3CC: [i32; 2] = [0; 2];
static mut SNAP338: [i32; 2] = [0; 2];
static mut SNAPDC0C: i32 = 0;
static mut SNAPDF20: i32 = 0;
static mut SIS611C_RET: i32 = 0;
static mut CARDTASK: i32 = 0;
static mut CMSNAP_RET: usize = 0;
static mut PAD3: u32 = 0;
static mut LOADCOUNT: i32 = 0;

fn log(s: &str) {
    unsafe { LOG.push_str(s) }
}

fn bits(f: f32) -> String {
    format!("{:08x}", f.to_bits())
}

fn jname(p: usize) -> String {
    unsafe {
        if p == 0 {
            return "null".into();
        }
        for k in 0..16 {
            if p == JOBJS + k * JSTRIDE {
                return format!("k{}", (b'a' + k as u8) as char);
            }
        }
        "?".into()
    }
}

fn gname(p: usize) -> &'static str {
    unsafe {
        for k in 0..4 {
            if p == GOBJS + k * GSTRIDE {
                return ["G0", "G1", "G2", "G3"][k];
            }
        }
        "?"
    }
}

fn tname(p: usize) -> &'static str {
    unsafe {
        if p == 0 {
            "null"
        } else if p == TEXTS {
            "T0"
        } else if p == TEXTS + TSTRIDE {
            "T1"
        } else if p == TEXTS + 2 * TSTRIDE {
            "T2"
        } else {
            "?"
        }
    }
}

fn install_hooks() {
    unsafe {
        t::ext::HSD_SISLIB_804D1124_ARR = Some(|| SLARR.as_ptr());
        t::ext::HSD_SISLIB_803A5ACC = Some(|a, b, x, y, _w, _h, _s| {
            log(&format!("sis5ACC({},{},{},{})", a, b, bits(x), bits(y)));
            TEXTS as *mut HSD_Text
        });
        t::ext::HSD_SISLIB_803A6530 = Some(|a, b, c| {
            log(&format!("dig({},{},{})", a, b, c));
            core::ptr::addr_of_mut!(DIGITS) as *mut u8
        });
        t::ext::HSD_SISLIB_803A660C = Some(|a, b, c| log(&format!("label({},{},{})", a, b, c)));
        t::ext::HSD_SISLIB_803A6368 = Some(|text, v| log(&format!("text6368({},{})", tname(text as usize), v)));
        t::ext::HSD_SISLIB_803A62A0 = Some(|a, b, c| log(&format!("sis62A0({},{},{})", a, b, c)));
        t::ext::HSD_SISLIB_803A611C = Some(|a, g, b, c, d, e, f, h| {
            let gs = if g.is_null() { "null" } else { "g" };
            log(&format!("sis611C({},{},{},{},{},{},{},{})", a, gs, b, c, d, e, f, h));
            SIS611C_RET
        });
        t::ext::HSD_SISLIB_803A5CC4 = Some(|text| log(&format!("sis5CC4({})", tname(text as usize))));
        t::ext::LBSNAP_8001D40C = Some(|i| {
            log(&format!("snap40C({})", i));
            SNAP40C[(i & 1) as usize]
        });
        t::ext::LBSNAP_8001D3B0 = Some(|i| {
            log(&format!("snap3B0({})", i));
            SNAP3B0[(i & 1) as usize]
        });
        t::ext::LBSNAP_8001D3CC = Some(|i| {
            log(&format!("snap3CC({})", i));
            SNAP3CC[(i & 1) as usize]
        });
        t::ext::LBSNAP_8001D338 = Some(|i| {
            log(&format!("snap338({})", i));
            SNAP338[(i & 1) as usize]
        });
        t::ext::LBSNAP_8001DC0C = Some(|img| {
            let who = if img as usize == IMG { "img" } else { "?" };
            log(&format!("snapDC0C({})", who));
            SNAPDC0C
        });
        t::ext::LBSNAP_8001DF20 = Some(|| {
            log("snapDF20");
            SNAPDF20
        });
        t::ext::LBSNAP_8001DF6C = Some(|chan| {
            log(&format!("snapDF6C({})", chan));
            0
        });
        t::ext::GMVS_GETSCENECONTROLLER = Some(|| {
            log("vsctrl");
            SC as *mut VsSceneController
        });
        t::ext::GM_GETDBPAUSEFLAG = Some(|bit| {
            log(&format!("dbg({})", bit));
            PAUSE[(bit & 3) as usize] != 0
        });
        t::ext::GM_CLEARDBPAUSEFLAG = Some(|bit| {
            log(&format!("dbclr({})", bit));
            PAUSE[(bit & 3) as usize] = 0;
        });
        t::ext::GM_SETDBPAUSEFLAG = Some(|bit| {
            log(&format!("dbset({})", bit));
            PAUSE[(bit & 3) as usize] = 1;
        });
        t::ext::GM_801A0FEC = Some(|a, b| log(&format!("a0fec({},{})", a, b)));
        t::ext::GM_801A10FC = Some(|a| log(&format!("a10fc({})", a)));
        t::ext::LBAUDIOAX_80024E84 = Some(|v| log(&format!("audio84({})", v as i32)));
        t::ext::LBAUDIOAX_80024030 = Some(|id| log(&format!("audio30({})", id)));
        t::ext::IFALL_802F33CC = Some(|| log("if33CC"));
        t::ext::IFALL_802F3394 = Some(|| log("if3394"));
        t::ext::HSD_PADRUMBLEUNPAUSEALL = Some(|| log("rumbleUnp"));
        t::ext::HSD_PADRUMBLEPAUSEALL = Some(|| log("rumblePau"));
        t::ext::CMSNAP_800315C8 = Some(|| log("cm15C8"));
        t::ext::CMSNAP_80031618 = Some(|| {
            log("cm1618");
            CMSNAP_RET as *mut c_void
        });
        t::ext::CMSNAP_800316B4 = Some(|| log("cm16B4"));
        t::ext::LB_80011E24 = Some(|src, out, idx, last| {
            log(&format!("lb11E24({},{},{})", jname(src as usize), idx, last));
            *out = (JOBJS + ((idx & 15) as usize) * JSTRIDE) as *mut HSD_JObj;
            0
        });
        t::ext::HSD_JOBJSETFLAGSALL = Some(|j, f| log(&format!("setflags({},{:08x})", jname(j as usize), f)));
        t::ext::HSD_JOBJCLEARFLAGSALL = Some(|j, f| log(&format!("clrflags({},{:08x})", jname(j as usize), f)));
        t::ext::HSD_JOBJSETMTXDIRTYSUB = Some(|j| log(&format!("dirty({})", jname(j as usize))));
        t::ext::HSD_FOREACHANIM = Some(|_obj, ty, mask, func, _arg, _a, _b| {
            log(&format!("foreach({},{:08x},{})", ty, mask, if func == 0 { "n" } else { "f" }));
        });
        t::ext::LBCARDNEW_COMPLETENEXTTASK = Some(|| {
            log("cardtask");
            CARDTASK
        });
        t::ext::LBARCHIVE_LOADARCHIVE = Some(|name| {
            log(&format!("arch({})", name));
            ARCH as *mut HSD_Archive
        });
        t::ext::HSD_ARCHIVEGETPUBLICADDRESS = Some(|_a, name| {
            log(&format!("pub({})", name));
            if name.starts_with("IfCameraInfo") {
                core::ptr::addr_of_mut!(JSLOTPTR) as *mut c_void
            } else {
                MDESC as *mut c_void
            }
        });
        t::ext::GOBJ_CREATE = Some(|c, p, pr| {
            log(&format!("gobj({},{},{})", c, p, pr));
            (GOBJS + (((LOADCOUNT + 1) & 3) as usize) * GSTRIDE) as *mut HSD_GObj
        });
        t::ext::HSD_JOBJLOADJOINT = Some(|joint| {
            let who = if joint.is_null() { "null" } else { "j" };
            log(&format!("jload({},{})", who, LOADCOUNT));
            let r = (JOBJS + (8 + (LOADCOUNT & 7) as usize) * JSTRIDE) as *mut HSD_JObj;
            LOADCOUNT += 1;
            r
        });
        t::ext::HSD_GOBJOBJECT_80390A70 = Some(|gobj, kind, obj| {
            log(&format!("setobj({},{},{})", gname(gobj as usize), kind, jname(obj as usize)));
        });
        t::ext::GOBJ_SETUPGXLINK = Some(|gobj, render, gx_link, prio| {
            let who = if render == t::ext::HSD_GObj_JObjCallback as usize { "jobjcb" } else { "?" };
            log(&format!("gx({},{},{},{})", gname(gobj as usize), who, gx_link, prio));
        });
        t::ext::GM_8016895C = Some(|jobj, _mdl, idx| log(&format!("m895C({},{})", jname(jobj as usize), idx)));
        t::ext::HSD_JOBJREQANIMALL = Some(|jobj, frame| log(&format!("reqanim({},{})", jname(jobj as usize), bits(frame))));
        t::ext::HSD_JOBJANIMALL = Some(|jobj| log(&format!("anim({})", jname(jobj as usize))));
        t::ext::HSD_GOBJ_SETUPPROC = Some(|gobj, proc, prio| {
            let who = if proc == t::fn_801A31D8 as usize { "p31D8" } else { "?" };
            log(&format!("proc({},{},{})", gname(gobj as usize), who, prio));
        });
        t::ext::HSD_GOBJ_JOBJKIND = Some(|| 1);
        t::ext::HSD_PADCOPYSTATUS_TRIGGER = Some(|idx| if idx == 3 { PAD3 } else { 0 });
    }
}

fn set_rng_rf_array(r: &mut Rng, out: &mut [f32; 4]) {
    for v in out.iter_mut() {
        *v = r.rf();
    }
}

#[test]
fn matches_c() {
    let want = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/tu_gmcamera.txt")).unwrap();

    // Driver-owned storage. JOBJS / TEXTS persist across cases (as the C statics do).
    let mut jobjs: std::boxed::Box<[HSD_JObj; 16]> = std::boxed::Box::new(unsafe { core::mem::zeroed() });
    let mut gobjs: std::boxed::Box<[HSD_GObj; 4]> = std::boxed::Box::new(unsafe { core::mem::zeroed() });
    let mut texts: std::boxed::Box<[HSD_Text; 3]> = std::boxed::Box::new(unsafe { core::mem::zeroed() });
    let mut arch: std::boxed::Box<u8> = std::boxed::Box::new(0);
    let mut img: std::boxed::Box<[u8; 4]> = std::boxed::Box::new([0; 4]);
    let mut sc: std::boxed::Box<VsSceneController> = std::boxed::Box::new(VsSceneController::zeroed());
    let mut jslot: std::boxed::Box<[HSD_Joint; 2]> = std::boxed::Box::new(unsafe { core::mem::zeroed() });
    let mut mdesc: std::boxed::Box<DynamicModelDesc> = std::boxed::Box::new(unsafe { core::mem::zeroed() });

    unsafe {
        JOBJS = jobjs.as_mut_ptr() as usize;
        GOBJS = gobjs.as_mut_ptr() as usize;
        TEXTS = texts.as_mut_ptr() as usize;
        ARCH = &mut *arch as *mut u8 as usize;
        IMG = img.as_mut_ptr() as usize;
        SC = &mut *sc as *mut VsSceneController as usize;
        MDESC = &mut *mdesc as *mut DynamicModelDesc as usize;
    }
    // The C driver's `JSLOTPTR = &JSLOT[0]` (set each case below) and `MDESC.joint = &JSLOT[1]`.
    let jslot0 = jslot.as_mut_ptr();
    mdesc.joint = unsafe { jslot0.add(1) };
    install_hooks();

    let mut r = Rng(0x2468ACE);
    let mut lines = want.lines();
    for fi in 0..FUNC_NAMES.len() {
        for i in 0..NCASES {
            // setup_case(), in the C driver's order.
            let state = unsafe { &mut *core::ptr::addr_of_mut!(t::gmCamera_VsCamUiState) };
            state.ifvscam = if r.rnd() & 1 != 0 { unsafe { ARCH as *mut c_void } } else { core::ptr::null_mut() };
            state.x4 = if r.rnd() % 17 == 16 {
                core::ptr::null_mut()
            } else {
                unsafe { (JOBJS + (r.rnd() % 16) as usize * JSTRIDE) as *mut HSD_JObj }
            };
            state.x8 = if r.rnd() % 17 == 16 {
                core::ptr::null_mut()
            } else {
                unsafe { (JOBJS + (r.rnd() % 16) as usize * JSTRIDE) as *mut HSD_JObj }
            };
            state.xC = (r.rnd() % 9) as i32;
            state.x10 = (r.rnd() % 3) as i32;
            state.x14 = (r.rnd() & 1) as i32;
            state.x18 = (r.rnd() & 1) as i32;
            state.snap_image = if r.rnd() & 1 != 0 { unsafe { IMG as *mut c_void } } else { core::ptr::null_mut() };
            state.x20 = (r.rnd() % 4) as i32;
            for i2 in 0..2 {
                state.x24[i2].x0 = r.rnd() & 1 != 0;
                state.x24[i2].x4 = (r.rnd() % 7) as i32;
                state.x24[i2].x8 = (r.rnd() % 4) as i32;
                state.x24[i2].xC = (r.rnd() % 6) as i32;
            }
            state.x44 = (r.rnd() % 6) as i32;
            for i2 in 0..3 {
                state.x48[i2] = if r.rnd() & 1 != 0 {
                    unsafe { (TEXTS + i2 * TSTRIDE) as *mut HSD_Text }
                } else {
                    core::ptr::null_mut()
                };
            }
            state.x54 = (r.rnd() % 50) as i32;
            unsafe {
                for k in 0..4 {
                    PAUSE[k] = (r.rnd() & 1) as i32;
                }
                for k in 0..2 {
                    SNAP40C[k] = (r.rnd() % 2) as i32;
                }
                for k in 0..2 {
                    SNAP3B0[k] = (r.rnd() % 2) as i32;
                }
                for k in 0..2 {
                    SNAP3CC[k] = (r.rnd() % 2) as i32;
                }
                for k in 0..2 {
                    SNAP338[k] = (r.rnd() % 2) as i32;
                }
                SNAPDC0C = (r.rnd() % 2) as i32;
                SNAPDF20 = (r.rnd() % 5) as i32;
                CMSNAP_RET = if r.rnd() & 1 != 0 { IMG } else { 0 };
                let k = r.rnd() % 3;
                CARDTASK = [0, 11, 3][k as usize];
                SIS611C_RET = (r.rnd() % 40) as i32;
                PAD3 = r.rnd();
                (*(SC as *mut VsSceneController)).state.pauser = (r.rnd() % 3) as i8;
                (*(SC as *mut VsSceneController)).state.hud_enabled = (r.rnd() & 1) as u8;
                (*(SC as *mut VsSceneController)).state.unk_3 = (r.rnd() & 1) as u8;
            }
            let arg_u = r.rnd() % 12000;
            let arg_i = r.rnd() % 9;
            let arg_p0 = r.rnd() % 3;
            let arg_p1 = r.rnd() % 3;
            let mut arg_f = [0.0f32; 4];
            set_rng_rf_array(&mut r, &mut arg_f);
            unsafe {
                JSLOTPTR = jslot0 as usize;
                LOADCOUNT = 0;
                *gobjs = core::mem::zeroed();
                for k in 0..4096 {
                    SLARR[k] = r.rnd() as u8;
                }
            }

            // call_fn + fingerprint.
            unsafe {
                LOG.clear();
                DIGITS = [0; 16];
            }
            let mut ret: i32 = 0;
            match fi {
                0 => {
                    let mut buf = [0u8; 16];
                    unsafe {
                        let rp = t::gmCamera_801A2224(buf.as_mut_ptr(), arg_u);
                        ret = rp.offset_from(buf.as_ptr()) as i32;
                        DIGITS[..8].copy_from_slice(&buf[..8]);
                    }
                }
                1 => unsafe {
                    let tp = t::gmCamera_801A2334((arg_p0 & 1) as i32, arg_f[0], arg_f[1], arg_f[2], arg_f[3]);
                    ret = (!tp.is_null()) as i32;
                },
                2 => unsafe {
                    let mut a0: i32 = 0;
                    let mut a1: i32 = 0;
                    t::gmCamera_801A253C(
                        if arg_p0 == 0 { core::ptr::null_mut() } else { &mut a0 },
                        if arg_p1 == 0 { core::ptr::null_mut() } else { &mut a1 },
                    );
                    ret = a0 + a1;
                },
                3 => unsafe { t::gmCamera_801A25C8() },
                4 => unsafe { ret = t::gmCamera_801A2640() },
                5 => unsafe { t::gmCamera_801A2650() },
                6 => unsafe { t::gmCamera_801A26C0() },
                7 => unsafe { t::gmCamera_801A2798() },
                8 => unsafe { t::gmCamera_801A2800() },
                9 => unsafe { t::gmCamera_801A28AC() },
                10 => unsafe { t::gmCamera_801A292C() },
                11 => unsafe { t::gmCamera_801A2AAC() },
                12 => unsafe { t::gmCamera_801A2BB0() },
                13 => unsafe {
                    let v: i32 = (arg_p0 & 1) as i32;
                    let rv = t::gmCamera_801A2BF0_get_translate_x(&v);
                    ret = rv.to_bits() as i32;
                },
                14 => unsafe { t::gmCamera_801A2BF0() },
                15 => unsafe { t::gmCamera_801A2D44() },
                16 => unsafe { t::gmCamera_801A2FBC() },
                17 => unsafe { t::gmCamera_801A2FFC() },
                18 => unsafe { t::gmCamera_801A3048(arg_i as i32) },
                19 => unsafe { t::gmCamera_801A3098() },
                20 => unsafe { t::gmCamera_801A30E4() },
                21 => unsafe {
                    let mut g: HSD_GObj = core::mem::zeroed();
                    g.hsd_obj = (JOBJS + (arg_i & 15) as usize * JSTRIDE) as *mut c_void;
                    t::fn_801A31D8(&mut g);
                },
                22 => unsafe { t::gmCamera_801A31FC() },
                _ => unreachable!(),
            }

            let st = unsafe { &*core::ptr::addr_of!(t::gmCamera_VsCamUiState) };
            let got = unsafe {
                let mut tx = String::new();
                for k in 0..3 {
                    let tp = (TEXTS + k * TSTRIDE) as *const HSD_Text;
                    tx.push_str(&format!(
                        "{},{},{};",
                        bits((*tp).font_size.x),
                        bits((*tp).font_size.y),
                        (*tp).default_kerning as i32
                    ));
                }
                let mut joints = String::new();
                for k in 0..16 {
                    joints.push_str(&format!("{},", bits((*((JOBJS + k * JSTRIDE) as *const HSD_JObj)).translate.x)));
                }
                let d = core::ptr::addr_of!(DIGITS) as *const u8;
                format!(
                    "{}.{} {}|log={}|ifv={}|x4={}|x8={}|xC={}|x10={}|x14={}|x18={}|snap={}|x20={}|p0={},{},{},{}|p1={},{},{},{}|x44={}|x48={},{},{}|x54={}|pause={},{},{},{}|sc={},{},{}|tx={}|txt={}|ret={}|buf={},{},{},{},{},{},{},{}",
                    fi,
                    i,
                    FUNC_NAMES[fi],
                    LOG,
                    if st.ifvscam.is_null() { "null" } else { "arch" },
                    jname(st.x4 as usize),
                    jname(st.x8 as usize),
                    st.xC,
                    st.x10,
                    st.x14,
                    st.x18,
                    if st.snap_image.is_null() { "null" } else { "img" },
                    st.x20,
                    st.x24[0].x0 as i32,
                    st.x24[0].x4,
                    st.x24[0].x8,
                    st.x24[0].xC,
                    st.x24[1].x0 as i32,
                    st.x24[1].x4,
                    st.x24[1].x8,
                    st.x24[1].xC,
                    st.x44,
                    tname(st.x48[0] as usize),
                    tname(st.x48[1] as usize),
                    tname(st.x48[2] as usize),
                    st.x54,
                    PAUSE[0],
                    PAUSE[1],
                    PAUSE[2],
                    PAUSE[3],
                    (*(SC as *const VsSceneController)).state.pauser as i32,
                    (*(SC as *const VsSceneController)).state.hud_enabled as i32,
                    (*(SC as *const VsSceneController)).state.unk_3 as i32,
                    joints,
                    tx,
                    ret,
                    *d,
                    *d.add(1),
                    *d.add(2),
                    *d.add(3),
                    *d.add(4),
                    *d.add(5),
                    *d.add(6),
                    *d.add(7),
                )
            };
            let line = lines.next().expect("fewer lines than cases");
            assert_eq!(got, line, "function {} case {}", FUNC_NAMES[fi], i);
        }
    }
    assert!(lines.next().is_none(), "more lines than cases");
    let _ = (&mut jobjs, &mut gobjs, &mut texts, &mut arch, &mut img, &mut sc, &mut jslot, &mut mdesc);
}
