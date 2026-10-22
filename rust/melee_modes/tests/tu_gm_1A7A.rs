//! tu::gm_1A7A vs the C (state oracle): gen/turef/tu_gm_1A7A_ref.c is the game's gm_1A7A.c compiled natively
//! (shipping contraction) with the gobj / joint / camera engine, the scene data of gmregtyfall.c and the toy
//! table stubbed and logged; the expected lines are tests/data/tu_gm_1A7A.txt (regenerate with
//! `python rust/melee_modes/gen/turef/tu_shard537_gm.py tu_gm_1A7A`). Each line is the callee log (float
//! arguments as bit patterns), the two static gobj pointers and the transform of the trophy's child joint.
#![allow(static_mut_refs)]

mod tu537;
use core::ffi::c_void;
use melee_modes::tu::gm_1A7A as t;
use melee_types::*;
use std::vec::Vec;
use tu537::Rng;

const FUNC_NAMES: [&str; 4] = ["fn_801A7A44", "fn_801A7A68", "fn_801A7A8C", "gm_801A7B00"];
const NCASES: [usize; 4] = [20, 20, 200, 600];

static mut LOG: String = String::new();
static mut GP: Vec<HSD_GObj> = Vec::new();
static mut NG: usize = 0;
static mut TREES: Vec<[HSD_JObj; 8]> = Vec::new();
static mut NT: usize = 0;
static mut CHAIN: usize = 0;
static mut JT_MAIN: Option<HSD_Joint> = None;
static mut JT_C: Option<HSD_Joint> = None;
static mut MD_A: Option<DynamicModelDesc> = None;
static mut MD_C: Option<DynamicModelDesc> = None;
static mut MDPTR_A: [*mut DynamicModelDesc; 1] = [core::ptr::null_mut()];
static mut MDPTR_C: [*mut DynamicModelDesc; 1] = [core::ptr::null_mut()];
static mut CAMS: Option<SceneCameraDesc> = None;
static mut CAMD: Option<HSD_CObjDesc> = None;
static mut ANIMS: [*mut HSD_CameraAnim; 1] = [core::ptr::null_mut()];
static mut CAMANIM: [u8; 8] = [0; 8];
static mut LIGHTPTR: [*mut LightList; 1] = [core::ptr::null_mut()];
static mut LO: [u8; 16] = [0; 16];
static mut SD_A: Option<SceneDesc> = None;
static mut SD_C: Option<SceneDesc> = None;
static mut CO: Option<HSD_CObj> = None;
static mut AO: Option<HSD_AObj> = None;
static mut TOY: [f32; 8] = [0.0; 8];
static mut CK: i32 = 0;

fn bits(f: f32) -> String {
    format!("{:08x}", f.to_bits())
}
fn log(s: &str) {
    unsafe { LOG.push_str(s) }
}
unsafe fn gid(g: *mut HSD_GObj) -> i32 {
    if g.is_null() {
        -1
    } else {
        g.offset_from(GP.as_mut_ptr()) as i32
    }
}
unsafe fn jid(j: *mut HSD_JObj) -> String {
    if j.is_null() {
        return "null".into();
    }
    for ti in 0..NT {
        for n in 0..8 {
            if j == &mut TREES[ti][n] as *mut HSD_JObj {
                return format!("t{}j{}", ti, n);
            }
        }
    }
    "?".into()
}

fn install_hooks() {
    use t::ext::*;
    unsafe {
        GM_804D6798 = Some(|| unsafe { JT_MAIN.as_mut().unwrap() as *mut HSD_Joint });
        GM_804D67A8 = Some(|| unsafe { SD_A.as_mut().unwrap() as *mut SceneDesc });
        GM_804D67AC = Some(|| unsafe { SD_C.as_mut().unwrap() as *mut SceneDesc });
        HSD_GOBJ_LIGHTKIND = Some(|| 2);
        HSD_GOBJ_JOBJKIND = Some(|| 1);
        HSD_GOBJ_CAMERAKIND = Some(|| 3);
        GOBJ_CREATE = Some(|c, pl, pr| unsafe {
            let n = NG;
            NG += 1;
            GP[n] = HSD_GObj::zeroed();
            log(&format!("gcreate({},{},{})=g{};", c, pl, pr, n));
            &mut GP[n] as *mut HSD_GObj
        });
        HSD_GOBJOBJECT_80390A70 = Some(|g, kind, obj| unsafe {
            (*g).hsd_obj = obj;
            let what = if obj == LO.as_mut_ptr() as *mut c_void {
                "lobj".to_string()
            } else if obj == CO.as_mut().unwrap() as *mut HSD_CObj as *mut c_void {
                "cobj".to_string()
            } else {
                jid(obj as *mut HSD_JObj)
            };
            log(&format!("gobj(g{},{},{});", gid(g), kind, what));
        });
        GOBJ_SETUPGXLINK = Some(|g, cb, gx, prio| unsafe {
            let who = if cb == HSD_GObj_LObjCallback as usize {
                "lobjcb"
            } else if cb == HSD_GObj_JObjCallback as usize {
                "jobjcb"
            } else {
                "?"
            };
            log(&format!("gxlink(g{},{},{},{});", gid(g), who, gx, prio));
        });
        GOBJ_SETUPGXLINKMAX = Some(|g, cb, prio| unsafe {
            let who = if cb == HSD_GObj_803910D8 as usize { "910D8" } else { "?" };
            log(&format!("gxlinkmax(g{},{},{});", gid(g), who, prio));
        });
        HSD_GOBJ_SETUPPROC = Some(|g, proc, prio| unsafe {
            let who = if proc == t::fn_801A7A8C as usize {
                "7A8C"
            } else if proc == t::fn_801A7A44 as usize {
                "7A44"
            } else if proc == t::fn_801A7A68 as usize {
                "7A68"
            } else {
                "?"
            };
            log(&format!("proc(g{},{},{});", gid(g), who, prio));
        });
        HSD_GOBJFREE = Some(|g| unsafe { log(&format!("gfree(g{});", gid(g))) });
        HSD_JOBJLOADJOINT = Some(|j| unsafe {
            let n = NT;
            NT += 1;
            let tr = &mut TREES[n];
            *tr = [HSD_JObj::zeroed(); 8];
            let jp = tr.as_mut_ptr();
            for k in 0..CHAIN {
                (*jp.add(k)).child = jp.add(k + 1);
            }
            (*jp.add(5)).next = jp.add(6);
            for k in 0..8usize {
                (*jp.add(k)).flags = if (n + k + CHAIN) % 3 == 0 { JOBJ_MTX_INDEP_SRT as u32 } else { 0 };
            }
            let who = if j == JT_MAIN.as_mut().unwrap() as *mut HSD_Joint {
                "main"
            } else if j == JT_C.as_mut().unwrap() as *mut HSD_Joint {
                "chara"
            } else {
                "?"
            };
            log(&format!("loadjoint({})=t{};", who, n));
            jp
        });
        HSD_JOBJSETMTXDIRTY = Some(|j| unsafe { log(&format!("dirty({});", jid(j))) });
        GM_8016895C = Some(|j, m, idx| unsafe {
            let who = if m == MD_A.as_mut().unwrap() as *mut DynamicModelDesc { "mdA" } else { "?" };
            log(&format!("8016895C({},{},{});", jid(j), who, idx));
        });
        HSD_JOBJREQANIMALL = Some(|j, f| unsafe { log(&format!("reqanim({},{});", jid(j), bits(f))) });
        HSD_JOBJANIMALL = Some(|j| unsafe { log(&format!("anim({});", jid(j))) });
        HSD_COBJADDANIM = Some(|c, a| unsafe {
            let cw = if c == CO.as_mut().unwrap() as *mut HSD_CObj { "cobj" } else { "?" };
            let aw = if a as *mut u8 == CAMANIM.as_mut_ptr() { "anim" } else { "?" };
            log(&format!("cobjaddanim({},{});", cw, aw));
        });
        HSD_COBJREQANIM = Some(|c, f| unsafe {
            let cw = if c == CO.as_mut().unwrap() as *mut HSD_CObj { "cobj" } else { "?" };
            log(&format!("cobjreq({},{});", cw, bits(f)));
        });
        HSD_COBJANIM = Some(|c| unsafe {
            let cw = if c == CO.as_mut().unwrap() as *mut HSD_CObj { "cobj" } else { "?" };
            log(&format!("cobjanim({});", cw));
        });
        LB_80011AC4 = Some(|l| unsafe {
            log(&format!("11AC4({});", if l == LIGHTPTR.as_mut_ptr() { "lights" } else { "?" }));
            LO.as_mut_ptr() as *mut HSD_LObj
        });
        LB_80013B14 = Some(|d| unsafe {
            log(&format!("13B14({});", if d as *mut c_void == CAMD.as_mut().unwrap() as *mut HSD_CObjDesc as *mut c_void { "camd" } else { "?" }));
            CO.as_mut().unwrap() as *mut HSD_CObj
        });
        LB_8000C1C0 = Some(|j, c| unsafe { log(&format!("C1C0({},{});", jid(j), jid(c))) });
        LB_8000C290 = Some(|j, c| unsafe { log(&format!("C290({},{});", jid(j), jid(c))) });
        GM_801BEFB0 = Some(|| {
            log("EFB0;");
            unsafe { CK }
        });
        GM_801A659C = Some(|i| {
            let r = (i + 3) % 7;
            log(&format!("659C({})={};", i, r));
            r
        });
        GM_GETCURRENTSCENEENTERDATA = Some(|| {
            log("enterdata;");
            core::ptr::null_mut()
        });
        GM_801A9630 = Some(|| log("9630;"));
        MN_8022F0F0 = Some(|v| log(&format!("F0F0({});", v)));
        TOY_803060BC = Some(|a, b| unsafe {
            let r = TOY[b as usize];
            log(&format!("toy({},{})={};", a, b, bits(r)));
            r
        });
    }
}

unsafe fn world(r: &mut Rng) {
    NG = 0;
    NT = 0;
    GP = (0..8).map(|_| HSD_GObj::zeroed()).collect();
    TREES = (0..4).map(|_| [HSD_JObj::zeroed(); 8]).collect();
    JT_MAIN = Some(core::mem::zeroed());
    JT_C = Some(core::mem::zeroed());
    MD_A = Some(DynamicModelDesc::zeroed());
    let mut mdc = DynamicModelDesc::zeroed();
    mdc.joint = JT_C.as_mut().unwrap();
    MD_C = Some(mdc);
    MDPTR_A[0] = MD_A.as_mut().unwrap();
    MDPTR_C[0] = MD_C.as_mut().unwrap();
    CAMD = Some(core::mem::zeroed());
    let mut cams = SceneCameraDesc::zeroed();
    cams.desc = CAMD.as_mut().unwrap();
    ANIMS[0] = CAMANIM.as_mut_ptr() as *mut HSD_CameraAnim;
    cams.anims = ANIMS.as_mut_ptr();
    CAMS = Some(cams);
    LIGHTPTR[0] = core::ptr::null_mut();
    let mut sda = SceneDesc::zeroed();
    sda.models = MDPTR_A.as_mut_ptr();
    sda.cameras = CAMS.as_mut().unwrap();
    sda.lights = LIGHTPTR.as_mut_ptr();
    SD_A = Some(sda);
    let mut sdc = SceneDesc::zeroed();
    sdc.models = MDPTR_C.as_mut_ptr();
    SD_C = Some(sdc);
    let mut co: HSD_CObj = core::mem::zeroed();
    AO = Some(HSD_AObj::zeroed());
    co.aobj = AO.as_mut().unwrap();
    CO = Some(co);
    AO.as_mut().unwrap().curr_frame = (r.rnd() % 5) as f32;
    AO.as_mut().unwrap().end_frame = (r.rnd() % 5) as f32;
    for k in 0..8 {
        TOY[k] = r.rf();
    }
    CK = (r.rnd() % 30) as i32;
    CHAIN = 1 + (r.rnd() % 7) as usize;
}

#[test]
fn matches_c() {
    let want = tu537::data("tu_gm_1A7A");
    install_hooks();
    let mut r = Rng::new();
    let mut lines = want.lines();
    for f in 0..FUNC_NAMES.len() {
        for i in 0..NCASES[f] {
            unsafe {
                world(&mut r);
                LOG.clear();
                t::gm_804D67B0 = core::ptr::null_mut();
                t::gm_804D67B4 = core::ptr::null_mut();
                if f == 0 || f == 1 {
                    let j = (HSD_JOBJLOADJOINT_call)(JT_MAIN.as_mut().unwrap());
                    GP[0].hsd_obj = j as *mut c_void;
                    NG = 1;
                    LOG.clear();
                    if f == 0 {
                        t::fn_801A7A44(&mut GP[0]);
                    } else {
                        t::fn_801A7A68(&mut GP[0]);
                    }
                } else if f == 2 {
                    NG = 4;
                    GP[0].hsd_obj = CO.as_mut().unwrap() as *mut HSD_CObj as *mut c_void;
                    t::gm_804D67B0 = &mut GP[1];
                    t::gm_804D67B4 = &mut GP[2];
                    t::fn_801A7A8C(&mut GP[0]);
                } else {
                    t::gm_801A7B00();
                }
                let mut got = format!("{}.{} {}|log={}|b0={},b4={}", f, i, FUNC_NAMES[f], LOG, gid(t::gm_804D67B0), gid(t::gm_804D67B4));
                for ti in 0..NT {
                    let c = &TREES[ti][1];
                    got += &format!(
                        "|T{}:{},{},{},{},{},{},{},{:x}",
                        ti,
                        bits(c.translate.x),
                        bits(c.translate.y),
                        bits(c.translate.z),
                        bits(c.rotate.y),
                        bits(c.scale.x),
                        bits(c.scale.y),
                        bits(c.scale.z),
                        c.flags
                    );
                }
                got += &format!("|gx={:x},{:x}", GP[0].gxlink_prios, GP[1].gxlink_prios);
                let line = lines.next().expect("fewer lines than cases");
                assert_eq!(got, line, "function {} case {}", FUNC_NAMES[f], i);
            }
        }
    }
    assert!(lines.next().is_none(), "more lines than cases");
}

/// `HSD_JObjLoadJoint` through the installed hook (the C driver calls its own stub the same way).
#[allow(non_snake_case)]
unsafe fn HSD_JOBJLOADJOINT_call(j: *mut HSD_Joint) -> *mut HSD_JObj {
    t::ext::HSD_JObjLoadJoint(j)
}
