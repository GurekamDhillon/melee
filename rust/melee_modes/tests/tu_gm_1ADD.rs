//! tu::gm_1ADD vs the C (state oracle): gen/turef/tu_gm_1ADD_ref.c is the game's gm_1ADD.c compiled natively
//! (shipping contraction) with the allocator and the gobj / joint / text engine stubbed and logged; the expected
//! lines are tests/data/tu_gm_1ADD.txt (regenerate with
//! `python rust/melee_modes/gen/turef/tu_shard537_gm.py tu_gm_1ADD`). Each line is the callee log (float arguments
//! as bit patterns, so the fused `2.2F * x - 1.0F` sites are compared exactly), the return value and the whole
//! object graph: the list, the gobjs, the joint trees and the texts.
#![allow(static_mut_refs)]

mod tu537;
use core::ffi::c_void;
use std::vec::Vec;
use melee_modes::tu::gm_1ADD as t;
use melee_types::*;
use tu537::Rng;

const NCASES: usize = 300;
const FUNC_NAMES: [&str; 12] = [
    "gm_801ADDD8",
    "gm_801ADE1C",
    "gm_801AE050",
    "gm_801AE44C",
    "gm_801AE544",
    "gm_801AE640",
    "gm_801AE74C",
    "gm_801AE848",
    "fn_801AE948",
    "gm_801AEBB0",
    "gm_801AECC4",
    "gm_801AEDC8",
];

#[repr(C)]
struct Tree {
    j: [HSD_JObj; 10],
    d: HSD_DObj,
    m: HSD_MObj,
    mat: HSD_Material,
}

const ASLOT: usize = 64;
#[repr(C, align(8))]
struct Arena([u8; ASLOT * 64]);

static mut LOG: String = String::new();
static mut ARENA: Arena = Arena([0; ASLOT * 64]);
static mut NALLOC: usize = 0;
static mut GP: Vec<HSD_GObj> = Vec::new();
static mut NG: usize = 0;
static mut TREES: Vec<Tree> = Vec::new();
static mut NT: usize = 0;
static mut TX: Vec<HSD_Text> = Vec::new();
static mut NTX: usize = 0;
static mut US: bool = false;
static mut N611: i32 = 0;
static mut CO: [u8; 8] = [0; 8];
static mut JOINTS: Vec<HSD_Joint> = Vec::new();
static mut MODELS: Vec<DynamicModelDesc> = Vec::new();
static mut MODELPTRS: [*mut DynamicModelDesc; 2] = [core::ptr::null_mut(); 2];
static mut CAMS: Vec<SceneCameraDesc> = Vec::new();
static mut CAMDESC: Vec<HSD_CObjDesc> = Vec::new();
static mut SD: Option<SceneDesc> = None;
static mut MASTER_ERR: [i8; 4] = [0; 4];
static mut COPY_TRIG: [u32; 4] = [0; 4];

unsafe fn log(s: &str) {
    LOG.push_str(s);
}
unsafe fn gid(p: *mut HSD_GObj) -> i32 {
    p.offset_from(GP.as_mut_ptr()) as i32
}
unsafe fn txid(p: *mut HSD_Text) -> i32 {
    p.offset_from(TX.as_mut_ptr()) as i32
}
unsafe fn jid(j: *mut HSD_JObj) -> String {
    if j.is_null() {
        return "null".into();
    }
    for ti in 0..NT {
        for n in 0..10 {
            if j == &mut TREES[ti].j[n] as *mut HSD_JObj {
                return format!("t{:02}j{}", ti, n);
            }
        }
    }
    "?".into()
}
unsafe fn nid(p: *mut c_void) -> i32 {
    if p.is_null() {
        return -1;
    }
    if p == &raw mut t::gm_803DD4D0 as *mut c_void {
        return -2;
    }
    ((p as usize - ARENA.0.as_ptr() as usize) / ASLOT) as i32
}
fn bits(f: f32) -> String {
    format!("{:08x}", f.to_bits())
}

fn install_hooks() {
    unsafe {
        t::ext::HSD_MEMALLOC = Some(|size| unsafe {
            let n = NALLOC;
            NALLOC += 1;
            log(&format!("alloc({})=m{};", size, n));
            core::ptr::write_bytes(ARENA.0.as_mut_ptr().add(n * ASLOT), 0xA5, ASLOT);
            ARENA.0.as_mut_ptr().add(n * ASLOT) as *mut c_void
        });
        t::ext::HSD_FREE = Some(|p| unsafe { log(&format!("free(m{});", nid(p))) });
        t::ext::GOBJ_CREATE = Some(|c, pl, pr| unsafe {
            let n = NG;
            NG += 1;
            GP[n] = HSD_GObj::zeroed();
            log(&format!("gcreate({},{},{})=g{};", c, pl, pr, n));
            &mut GP[n] as *mut HSD_GObj
        });
        t::ext::HSD_GOBJOBJECT_80390A70 = Some(|g, kind, obj| unsafe {
            (*g).hsd_obj = obj;
            let what = if kind == 3 { "cobj".to_string() } else { jid(obj as *mut HSD_JObj) };
            log(&format!("gobj(g{},{},{});", gid(g), kind, what));
        });
        t::ext::GOBJ_SETUPGXLINK = Some(|g, cb, gx, prio| unsafe {
            let who = if cb == t::ext::HSD_GObj_JObjCallback as usize { "jobjcb" } else { "?" };
            log(&format!("gxlink(g{},{},{},{});", gid(g), who, gx, prio));
        });
        t::ext::GOBJ_SETUPGXLINKMAX = Some(|g, cb, prio| unsafe {
            let who = if cb == t::ext::HSD_GObj_803910D8 as usize { "910D8" } else { "?" };
            log(&format!("gxlinkmax(g{},{},{});", gid(g), who, prio));
        });
        t::ext::HSD_GOBJ_SETUPPROC = Some(|g, proc, prio| unsafe {
            let who = if proc == t::fn_801AE948 as usize { "948" } else { "?" };
            log(&format!("proc(g{},{},{});", gid(g), who, prio));
        });
        t::ext::HSD_GOBJFREE = Some(|g| unsafe { log(&format!("gfree(g{});", gid(g))) });
        t::ext::HSD_GOBJ_JOBJKIND = Some(|| 1);
        t::ext::HSD_GOBJ_CAMERAKIND = Some(|| 3);
        t::ext::HSD_JOBJLOADJOINT = Some(|j| unsafe {
            let n = NT;
            NT += 1;
            let tr = &mut TREES[n];
            *tr = core::mem::zeroed();
            let jp: *mut HSD_JObj = tr.j.as_mut_ptr();
            (*jp.add(0)).child = jp.add(1);
            (*jp.add(1)).child = jp.add(2);
            (*jp.add(2)).child = jp.add(3);
            (*jp.add(2)).next = jp.add(4);
            (*jp.add(4)).child = jp.add(5);
            (*jp.add(5)).next = jp.add(6);
            (*jp.add(4)).next = jp.add(7);
            (*jp.add(3)).next = jp.add(8);
            (*jp.add(8)).child = jp.add(9);
            (*jp.add(2)).u.dobj = &mut tr.d;
            tr.d.mobj = &mut tr.m;
            tr.m.mat = &mut tr.mat;
            for k in 0..10usize {
                let m = (n + k) % 3;
                tr.j[k].translate.x = ((n * 3 + k) as f32) * 0.25;
                tr.j[k].translate.y = ((n + k * 5) as f32) * 0.5;
                tr.j[k].flags = if m == 0 {
                    JOBJ_MTX_DIRTY as u32
                } else if m == 1 {
                    JOBJ_MTX_INDEP_SRT as u32
                } else {
                    0
                };
            }
            let who = if j == &mut JOINTS[0] as *mut HSD_Joint {
                "jt0"
            } else if j == &mut JOINTS[1] as *mut HSD_Joint {
                "jt1"
            } else {
                "?"
            };
            log(&format!("loadjoint({})=t{};", who, n));
            jp
        });
        t::ext::HSD_JOBJSETMTXDIRTYSUB = Some(|j| unsafe {
            log(&format!("dirty({});", jid(j)));
            (*j).flags |= JOBJ_MTX_DIRTY as u32;
        });
        t::ext::HSD_JOBJSETFLAGSALL = Some(|j, f| unsafe {
            log(&format!("setflagsall({},{:x});", jid(j), f));
            (*j).flags |= f;
        });
        t::ext::GM_8016895C = Some(|j, m, idx| unsafe {
            let who = if m == &mut MODELS[0] as *mut DynamicModelDesc {
                "md0"
            } else if m == &mut MODELS[1] as *mut DynamicModelDesc {
                "md1"
            } else {
                "?"
            };
            log(&format!("8016895C({},{},{});", jid(j), who, idx));
        });
        t::ext::HSD_JOBJREQANIMALL = Some(|j, f| unsafe { log(&format!("reqanim({},{});", jid(j), bits(f))) });
        t::ext::HSD_JOBJANIMALL = Some(|j| unsafe { log(&format!("anim({});", jid(j))) });
        t::ext::HSD_COBJLOADDESC = Some(|d| unsafe {
            log(&format!("cobjload({});", if d == &mut CAMDESC[0] as *mut HSD_CObjDesc { "camdesc" } else { "?" }));
            CO.as_mut_ptr() as *mut HSD_CObj
        });
        t::ext::HSD_SISLIB_803A5ACC = Some(|a, b, x, y, z, w, h| unsafe {
            let n = NTX;
            NTX += 1;
            TX[n] = HSD_Text::zeroed();
            log(&format!("text({},{},{},{},{},{},{})=x{};", a, b, bits(x), bits(y), bits(z), bits(w), bits(h), n));
            &mut TX[n] as *mut HSD_Text
        });
        t::ext::HSD_SISLIB_803A6368 = Some(|tx, v| unsafe { log(&format!("setText(x{},{});", txid(tx), v)) });
        t::ext::HSD_SISLIB_803A62A0 = Some(|a, f, s| unsafe { log(&format!("sisload({},{},{});", a, f, s)) });
        t::ext::HSD_SISLIB_803A611C = Some(|a, g, b, c, d, e, f, h| unsafe {
            let r = 77 + N611;
            N611 += 1;
            log(&format!("sis611C({},g{},{},{},{},{},{},{})={};", a, gid(g), b, c, d, e, f, h, r));
            r
        });
        t::ext::HSD_SISLIB_803A5CC4 = Some(|tx| unsafe { log(&format!("textfree(x{});", txid(tx))) });
        t::ext::LBARCHIVE_80016DBC = Some(|filename, symbols, sym, end| unsafe {
            log(&format!("archive({},{},{});", filename, sym, end));
            *symbols = SD.as_mut().unwrap() as *mut SceneDesc;
        });
        t::ext::LBLANG_ISSAVEDLANGUAGEUS = Some(|| unsafe {
            log("isUS;");
            US
        });
        t::ext::HSD_PADMASTERSTATUS_ERR = Some(|i| unsafe { MASTER_ERR[i] });
        t::ext::HSD_PADCOPYSTATUS_TRIGGER = Some(|i| unsafe { COPY_TRIG[i] });
    }
}

unsafe fn reset_world(r: &mut Rng) {
    ARENA.0 = [0; ASLOT * 64];
    NALLOC = 0;
    NG = 0;
    NT = 0;
    NTX = 0;
    N611 = 0;
    GP = (0..64).map(|_| HSD_GObj::zeroed()).collect();
    TX = (0..64).map(|_| HSD_Text::zeroed()).collect();
    TREES = (0..64).map(|_| core::mem::zeroed()).collect();
    t::gm_803DD4D0 = core::mem::zeroed();
    t::gm_804D686C = 0;
    t::gm_804D6868 = core::ptr::null_mut();
    JOINTS = (0..2).map(|_| core::mem::zeroed()).collect();
    MODELS = (0..2).map(|_| DynamicModelDesc::zeroed()).collect();
    MODELS[0].joint = &mut JOINTS[0];
    MODELS[1].joint = &mut JOINTS[1];
    MODELPTRS[0] = &mut MODELS[0];
    MODELPTRS[1] = &mut MODELS[1];
    CAMDESC = (0..1).map(|_| core::mem::zeroed()).collect();
    CAMS = (0..1).map(|_| SceneCameraDesc::zeroed()).collect();
    CAMS[0].desc = &mut CAMDESC[0];
    let mut sd = SceneDesc::zeroed();
    sd.models = MODELPTRS.as_mut_ptr();
    sd.cameras = CAMS.as_mut_ptr();
    SD = Some(sd);
    for i in 0..4 {
        MASTER_ERR[i] = if r.rnd() % 3 == 0 { -1 } else { 0 };
        COPY_TRIG[i] = r.rnd();
    }
}

unsafe fn dump_graph() -> String {
    let mut o = format!(
        "|g686C={},head={},{}",
        t::gm_804D686C,
        nid(t::gm_803DD4D0.next as *mut c_void),
        t::gm_803DD4D0.unkC
    );
    let mut b = t::gm_803DD4D0.next;
    while !b.is_null() {
        o += &format!(
            "|B{}:{},{},{},{},g{},x{},p{}",
            nid(b as *mut c_void),
            (*b).unkC,
            (*b).unk10,
            bits((*b).unk14),
            bits((*b).unk18),
            if (*b).unk1C.is_null() { -1 } else { gid((*b).unk1C) },
            if (*b).unk20.is_null() { -1 } else { txid((*b).unk20) },
            nid((*b).unk4 as *mut c_void)
        );
        let mut bt = (*b).unk8;
        while !bt.is_null() {
            o += &format!(
                ";b{}:{},{},{},{},g{},x{},p{}",
                nid(bt as *mut c_void),
                (*bt).unk8,
                (*bt).unkC,
                bits((*bt).unk10),
                bits((*bt).unk14),
                if (*bt).unk20.is_null() { -1 } else { gid((*bt).unk20) },
                if (*bt).unk24.is_null() { -1 } else { txid((*bt).unk24) },
                nid((*bt).unk4 as *mut c_void)
            );
            bt = (*bt).next;
        }
        b = (*b).next;
    }
    for g in 0..NG {
        o += &format!("|G{}:{:x},{}", g, GP[g].gxlink_prios, if GP[g].hsd_obj.is_null() { -1 } else { 1 });
    }
    for ti in 0..NT {
        o += &format!("|T{}:", ti);
        for n in 0..10 {
            let j = &TREES[ti].j[n];
            o += &format!("{},{},{:x};", bits(j.translate.x), bits(j.translate.y), j.flags);
        }
        let d = TREES[ti].mat.diffuse;
        o += &format!("{},{},{}", d.r, d.g, d.b);
    }
    for x in 0..NTX {
        let tx = &TX[x];
        o += &format!(
            "|X{}:{},{},{},{},{},{},{}",
            x,
            tx.text_color.r,
            tx.text_color.g,
            tx.text_color.b,
            bits(tx.x34.x),
            bits(tx.x34.y),
            tx.default_alignment,
            tx.default_fitting
        );
    }
    o
}

#[test]
fn matches_c() {
    let want = tu537::data("tu_gm_1ADD");
    install_hooks();
    let mut r = Rng::new();
    let mut lines = want.lines();
    for f in 0..FUNC_NAMES.len() {
        for i in 0..NCASES {
            unsafe {
                reset_world(&mut r);
                US = r.rnd() & 1 != 0;
                LOG.clear();
                t::gm_801AEBB0();
                let nbox = r.rnd() % 4;
                for _ in 0..nbox {
                    let a = r.rnd();
                    let b = r.rnd();
                    let x = r.rf();
                    let y = r.rf();
                    t::gm_801ADE1C((a % 3) as i32, (b % 30) as i32, x, y);
                }
                let nbtn = r.rnd() % 5;
                for _ in 0..nbtn {
                    let a = r.rnd();
                    let b = r.rnd();
                    let c = r.rnd();
                    let x = r.rf();
                    let y = r.rf();
                    t::gm_801AE050((a % 3) as i32, (b % 2) as i32, (c % 10) as i32, x, y);
                }
                LOG.clear();
                let a = r.rnd();
                let b = r.rnd();
                let x = r.rf();
                let _y = r.rf();
                let c = r.rnd();
                let mut ret = 0u64;
                match f {
                    0 => t::gm_801ADDD8(),
                    1 => ret = t::gm_801ADE1C((a % 3) as i32, (b % 30) as i32, x, _y) as u64,
                    2 => ret = t::gm_801AE050((a % 3) as i32, (b % 2) as i32, (c % 10) as i32, x, _y) as u64,
                    3 => t::gm_801AE44C((a % 3) as i32, x),
                    4 => t::gm_801AE544((a % 3) as i32, x),
                    5 => t::gm_801AE640((a % 3) as i32, b % 2 != 0),
                    6 => t::gm_801AE74C((a % 3) as i32, b % 2 != 0),
                    7 => t::gm_801AE848((a % 3) as i32),
                    8 => t::fn_801AE948(core::ptr::null_mut()),
                    9 => t::gm_801AEBB0(),
                    10 => ret = gid(t::gm_801AECC4((a % 2) as i32)) as u64,
                    _ => ret = t::gm_801AEDC8() as u64,
                }
                let mut got = format!("{}.{} {}|log={}|ret={:x}", f, i, FUNC_NAMES[f], LOG, ret);
                got += &dump_graph();
                let line = lines.next().expect("fewer lines than cases");
                assert_eq!(got, line, "function {} case {}", FUNC_NAMES[f], i);
            }
        }
    }
    assert!(lines.next().is_none(), "more lines than cases");
}
