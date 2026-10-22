//! tu::gmapproach vs the C (state oracle): gen/turef/tu_gmapproach_ref.c is the game's gmapproach.c
//! compiled natively (shipping contraction) with its engine calls stubbed and logged; the expected
//! lines are tests/data/tu_gmapproach.txt (regenerate with `python rust/melee_modes/gen/turef/
//! tu_shard457_gm.py tu_gmapproach`). Each line is the callee log, the scene state, the gobj pool's
//! link priorities and the two arguments.
#![allow(static_mut_refs)]

use core::ffi::c_void;
use melee_modes::tu::gmapproach as t;
use melee_types::*;

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
const FUNC_NAMES: [&str; 5] = [
    "fn_801AD920",
    "gm_801ADB04",
    "gm_Scene_Approach_OnFrame",
    "gm_Scene_Approach_OnEnter",
    "gm_Scene_Approach_OnExit",
];

// Hook controls and the call log (single-threaded test; the hooks are plain fn pointers).
static mut LOG: String = String::new();
static mut PAD_TRIGGER: [u32; 4] = [0; 4];
static mut POOL_NEXT: usize = 0;
static mut POOL_BASE: usize = 0;
static mut GOBJ_P: usize = 0;
static mut ARCH: u8 = 0;
static mut ARCH_P: usize = 0;
static mut SD_P: usize = 0;
static mut CAMDESC_P: usize = 0;
static mut LIGHTPTR_P: usize = 0;
static mut MODEL_P: usize = 0;
static mut MJOINT_P: usize = 0;
static mut LOADED_P: usize = 0;
static mut J_BASE: usize = 0;
static mut D_P: usize = 0;
static mut M_P: usize = 0;
static mut T_P: usize = 0;
static mut A_P: usize = 0;

fn log(s: &str) {
    unsafe { LOG.push_str(s) }
}

fn bits(f: f32) -> String {
    format!("{:08x}", f.to_bits())
}

fn gobjid(p: usize) -> &'static str {
    unsafe {
        for k in 0..3 {
            if p == POOL_BASE + k * core::mem::size_of::<HSD_GObj>() {
                return ["g0", "g1", "g2"][k];
            }
        }
        if p == GOBJ_P {
            return "gP";
        }
        "?"
    }
}

fn jid(p: usize) -> &'static str {
    unsafe {
        if p == 0 {
            "null"
        } else if p == J_BASE {
            "j0"
        } else if p == J_BASE + core::mem::size_of::<HSD_JObj>() {
            "j1"
        } else if p == J_BASE + 2 * core::mem::size_of::<HSD_JObj>() {
            "j2"
        } else if p == J_BASE + 3 * core::mem::size_of::<HSD_JObj>() {
            "j3"
        } else if p == LOADED_P {
            "loaded"
        } else {
            "?"
        }
    }
}

fn install_hooks() {
    unsafe {
        t::ext::LBARCHIVE_80016DBC = Some(|filename, symbols, _name, _flag| {
            log(&format!("archive({})", filename));
            *symbols = SD_P as *mut SceneDesc;
            ARCH_P as *mut HSD_Archive
        });
        t::ext::LBARCHIVE_80016EFC = Some(|a| log(&format!("archclose({})", if a as usize == ARCH_P { "arch" } else { "?" })));
        t::ext::GOBJ_CREATE = Some(|classifier, p_link, priority| {
            let k = POOL_NEXT.min(2);
            POOL_NEXT += 1;
            log(&format!("gobj({},{},{})", classifier, p_link, priority));
            (POOL_BASE + k * core::mem::size_of::<HSD_GObj>()) as *mut HSD_GObj
        });
        t::ext::HSD_COBJLOADDESC = Some(|desc| {
            log(&format!("cobj({})", if desc as usize == CAMDESC_P { "cam" } else { "?" }));
            0x7001 as *mut HSD_CObj
        });
        t::ext::HSD_GOBJOBJECT_80390A70 = Some(|gobj, kind, _obj| {
            log(&format!("setobj({},{})", gobjid(gobj as usize), kind));
        });
        t::ext::GOBJ_SETUPGXLINKMAX = Some(|gobj, render, priority| {
            let who = if render == t::ext::HSD_GObj_803910D8 as usize { "cb803910D8" } else { "?" };
            log(&format!("gxmax({},{},{})", gobjid(gobj as usize), who, priority));
        });
        t::ext::LB_80011AC4 = Some(|lights| {
            log(&format!("lobj({})", if lights as usize == LIGHTPTR_P { "lights" } else { "?" }));
            0x7002 as *mut HSD_LObj
        });
        t::ext::GOBJ_SETUPGXLINK = Some(|gobj, render, gx_link, priority| {
            let who = if render == t::ext::HSD_GObj_LObjCallback as usize {
                "lobjcb"
            } else if render == t::ext::HSD_GObj_JObjCallback as usize {
                "jobjcb"
            } else {
                "?"
            };
            log(&format!("gx({},{},{},{})", gobjid(gobj as usize), who, gx_link, priority));
        });
        t::ext::HSD_JOBJLOADJOINT = Some(|joint| {
            log(&format!("jload({})", if joint as usize == MJOINT_P { "mjoint" } else { "?" }));
            LOADED_P as *mut HSD_JObj
        });
        t::ext::GM_8016895C = Some(|jobj, model, idx| {
            log(&format!("m895C({},{},{})", jid(jobj as usize), if model as usize == MODEL_P { "m0" } else { "?" }, idx));
        });
        t::ext::HSD_JOBJREQANIMALL = Some(|jobj, frame| log(&format!("reqanim({},{})", jid(jobj as usize), bits(frame))));
        t::ext::HSD_JOBJANIMALL = Some(|jobj| log(&format!("anim({})", jid(jobj as usize))));
        t::ext::HSD_GOBJ_SETUPPROC = Some(|gobj, proc, prio| {
            let who = if proc == fn_801ad920_addr() { "p920" } else { "?" };
            log(&format!("proc({},{},{})", gobjid(gobj as usize), who, prio));
        });
        t::ext::HSD_TOBJREQANIMALL = Some(|tobj, start| {
            let who = if tobj as usize == T_P { "t" } else { "?" };
            log(&format!("treq({},{})", who, bits(start)));
        });
        t::ext::HSD_AOBJSETRATE = Some(|aobj, rate| {
            let who = if aobj as usize == A_P { "a" } else { "?" };
            log(&format!("arate({},{})", who, bits(rate)));
        });
        t::ext::GM_801A4B60 = Some(|| log("gm4B60"));
        t::ext::LBAUDIOAX_80023F28 = Some(|id| {
            log(&format!("audio({})", id));
            0
        });
        t::ext::LBAUDIOAX_800236DC = Some(|| {
            log("audioclose");
            0
        });
        t::ext::HSD_PADCOPYSTATUS_TRIGGER = Some(|idx| PAD_TRIGGER[idx]);
        t::ext::HSD_GOBJ_JOBJKIND = Some(|| 1);
        t::ext::HSD_GOBJ_LIGHTKIND = Some(|| 2);
        t::ext::HSD_GOBJ_CAMERAKIND = Some(|| 3);
    }
}

fn fn_801ad920_addr() -> usize {
    t::fn_801AD920 as usize
}

#[test]
fn matches_c() {
    let want = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/tu_gmapproach.txt")).unwrap();

    // The C driver's storage, at fixed addresses.
    let mut pool: std::boxed::Box<[HSD_GObj; 3]> = std::boxed::Box::new(unsafe { core::mem::zeroed() });
    let mut gobj: std::boxed::Box<HSD_GObj> = std::boxed::Box::new(unsafe { core::mem::zeroed() });
    let mut jnt: std::boxed::Box<[HSD_JObj; 4]> = std::boxed::Box::new(unsafe { core::mem::zeroed() });
    let mut dobj: std::boxed::Box<HSD_DObj> = std::boxed::Box::new(unsafe { core::mem::zeroed() });
    let mut mobj: std::boxed::Box<HSD_MObj> = std::boxed::Box::new(unsafe { core::mem::zeroed() });
    let mut tobj: std::boxed::Box<HSD_TObj> = std::boxed::Box::new(unsafe { core::mem::zeroed() });
    let mut aobj: std::boxed::Box<HSD_AObj> = std::boxed::Box::new(unsafe { core::mem::zeroed() });
    let mut camdesc: std::boxed::Box<HSD_CObjDesc> = std::boxed::Box::new(unsafe { core::mem::zeroed() });
    let mut lights: std::boxed::Box<LightList> = std::boxed::Box::new(unsafe { core::mem::zeroed() });
    let mut lightptr: std::boxed::Box<[*mut LightList; 1]> = std::boxed::Box::new([core::ptr::null_mut()]);
    let mut model: std::boxed::Box<DynamicModelDesc> = std::boxed::Box::new(unsafe { core::mem::zeroed() });
    let mut mjoint: std::boxed::Box<HSD_Joint> = std::boxed::Box::new(unsafe { core::mem::zeroed() });
    let mut modelptr: std::boxed::Box<[*mut DynamicModelDesc; 1]> = std::boxed::Box::new([core::ptr::null_mut()]);
    let mut cams: std::boxed::Box<[SceneCameraDesc; 1]> = std::boxed::Box::new(unsafe { core::mem::zeroed() });
    let mut sd: std::boxed::Box<SceneDesc> = std::boxed::Box::new(SceneDesc::zeroed());
    let mut loaded: std::boxed::Box<HSD_JObj> = std::boxed::Box::new(unsafe { core::mem::zeroed() });

    unsafe {
        POOL_BASE = pool.as_mut_ptr() as usize;
        GOBJ_P = &mut *gobj as *mut HSD_GObj as usize;
        ARCH_P = &mut ARCH as *mut u8 as usize;
        SD_P = &mut *sd as *mut SceneDesc as usize;
        CAMDESC_P = &mut *camdesc as *mut HSD_CObjDesc as usize;
        LIGHTPTR_P = lightptr.as_mut_ptr() as usize;
        MODEL_P = &mut *model as *mut DynamicModelDesc as usize;
        MJOINT_P = &mut *mjoint as *mut HSD_Joint as usize;
        LOADED_P = &mut *loaded as *mut HSD_JObj as usize;
        J_BASE = jnt.as_mut_ptr() as usize;
        D_P = &mut *dobj as *mut HSD_DObj as usize;
        M_P = &mut *mobj as *mut HSD_MObj as usize;
        T_P = &mut *tobj as *mut HSD_TObj as usize;
        A_P = &mut *aobj as *mut HSD_AObj as usize;
    }
    install_hooks();

    let mut r = Rng(0x2468ACE);
    let mut lines = want.lines();
    for fi in 0..FUNC_NAMES.len() {
        for i in 0..NCASES {
            // setup_case(), in the C driver's order.
            unsafe {
                t::GM_80480D98.x0 = if r.rnd() & 1 != 0 { &mut ARCH as *mut u8 as *mut HSD_Archive } else { core::ptr::null_mut() };
                t::GM_80480D98.x4 = (r.rnd() % 30) as i32;
                t::GM_80480D98.x8 = r.rnd() as u16;
                t::GM_80480D98.xA = r.rnd() as u16;
                t::GM_80480D98.xC = r.rnd() as u16;
                t::GM_80480D98.xE = (r.rnd() % 4) as u8;
                for k in 0..4 {
                    PAD_TRIGGER[k] = r.rnd();
                }
                POOL_NEXT = 0;
                *pool = core::mem::zeroed();
                // The joint chain: gobj->hsd_obj = J0; child J1; J1.next = J2; J2.next = J3 (dobj chain).
                *jnt = [core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed()];
                jnt[0].child = &mut jnt[1];
                jnt[1].next = &mut jnt[2];
                jnt[2].next = &mut jnt[3];
                jnt[3].u.dobj = &mut *dobj;
                *dobj = core::mem::zeroed();
                dobj.mobj = &mut *mobj;
                *mobj = core::mem::zeroed();
                mobj.tobj = &mut *tobj;
                *tobj = core::mem::zeroed();
                tobj.aobj = &mut *aobj;
                *aobj = core::mem::zeroed();
                *gobj = core::mem::zeroed();
                gobj.hsd_obj = &mut jnt[0] as *mut HSD_JObj as *mut c_void;
                camdesc_zero(&mut camdesc);
                cams[0].desc = &mut *camdesc;
                lightptr[0] = &mut *lights;
                modelptr[0] = &mut *model;
                model.joint = &mut *mjoint;
                sd.cameras = cams.as_mut_ptr();
                sd.lights = lightptr.as_mut_ptr();
                sd.models = modelptr.as_mut_ptr();
            }
            let arg0 = (r.rnd() % 40) as u8;
            let arg1 = (r.rnd() % 4) as u8;
            let mut arg = [arg0, arg1];

            unsafe { LOG.clear() };
            match fi {
                0 => unsafe { t::fn_801AD920(&mut *gobj) },
                1 => t::gm_801ADB04(),
                2 => t::gm_Scene_Approach_OnFrame(),
                3 => t::gm_Scene_Approach_OnEnter(arg.as_mut_ptr() as *mut c_void),
                _ => t::gm_Scene_Approach_OnExit(core::ptr::null_mut()),
            }
            let got = unsafe {
                let g = &t::GM_80480D98;
                format!(
                    "{}.{} {}|log={}|x0={}|x4={}|x8={}|xA={}|xC={}|xE={}|gx={},{},{}|arg={},{}",
                    fi,
                    i,
                    FUNC_NAMES[fi],
                    LOG,
                    (!g.x0.is_null()) as i32,
                    g.x4,
                    g.x8,
                    g.xA,
                    g.xC,
                    g.xE,
                    pool[0].gxlink_prios as u32,
                    pool[1].gxlink_prios as u32,
                    pool[2].gxlink_prios as u32,
                    arg[0] as i8 as i32,
                    arg[1] as i8 as i32,
                )
            };
            let line = lines.next().expect("fewer lines than cases");
            assert_eq!(got, line, "function {} case {}", FUNC_NAMES[fi], i);
        }
    }
    assert!(lines.next().is_none(), "more lines than cases");
    let _ = (&mut loaded, &mut lights, &mut mjoint, &mut aobj, &mut tobj, &mut mobj, &mut dobj, &mut gobj, &mut pool);
}

fn camdesc_zero(d: &mut HSD_CObjDesc) {
    unsafe { *d = core::mem::zeroed() };
}
