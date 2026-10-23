//! tu::grshrineroute against the C TU compiled natively with shipping contraction (state oracle:
//! gen/turef/tu_grshrineroute_ref.c, data tests/data/tu_grshrineroute.txt; regenerate with
//! `python rust/melee_world/gen/turef/tu_shard585.py grshrineroute`).
#![allow(static_mut_refs, non_snake_case, unused)]
mod common;
use common::gr::*;
use common::*;
use core::ffi::c_void;
use melee_types::*;
use melee_world::tu::grshrineroute as t;
use melee_world::tu_ext as ext;

static mut GNULL: bool = false;
static mut JNULL: bool = false;
static mut LDNULL: bool = false;
static mut FPOS: bool = false;
static mut SCALE: f32 = 0.0;
static mut PGOBJ: *mut HSD_GObj = core::ptr::null_mut();
static mut CB: std::vec::Vec<*mut u8> = std::vec::Vec::new();

unsafe fn mkjobj2() -> *mut HSD_JObj {
    let j = mkjobj();
    (*j).flags &= !(JOBJ_USE_QUATERNION as u32);
    j
}
unsafe fn mklobj() -> *mut HSD_LObj {
    let l = az_t::<HSD_LObj>();
    (*l).flags = rnd() as u16;
    (*l).next = core::ptr::null_mut();
    l
}
unsafe fn mkfgobj() -> *mut HSD_GObj {
    az_t::<HSD_GObj>()
}
unsafe fn mkyp() -> *mut t::grShrineRoute_YakumonoParam {
    let p = az_t::<t::grShrineRoute_YakumonoParam>();
    (*p).x0 = az(16) as *mut c_void;
    (*p).x4 = az(16) as *mut c_void;
    (*p).x8 = az(16) as *mut c_void;
    (*p).xC = az(16) as *mut c_void;
    (*p).x10 = (rnd() % 50) as i32;
    (*p).x14 = (rnd() % 100) as f32 / 50.0;
    (*p).x18 = (rnd() % 100) as f32 / 50.0;
    (*p).x1C = (rnd() % 100) as f32 / 5.0;
    (*p).x20 = (rnd() % 100) as f32 / 5.0;
    (*p).x24 = if rnd() % 3 == 0 { 0 } else { (1 + rnd() % 4) as i32 };
    (*p).spawn_desc.kind = rnd() as u16;
    (*p).spawn_desc.x2 = rnd() as u8;
    (*p).spawn_desc.respawn = rnd() as u8;
    p
}
include!("stubs/grshrineroute.rs");

fn t_efSync_Spawn(gfx_id: i32, g: *mut HSD_GObj, j: *mut HSD_JObj, sc: *mut f32) -> *mut c_void {
    unsafe {
        out("efSync_Spawn");
        tri(gfx_id as i64);
        trp(g as *const u8);
        trp(j as *const u8);
        trf(*sc);
        out("\n");
    }
    core::ptr::null_mut()
}

/// The stage's light gobjs on GX plink 3: n decoys (wrong classifier) ahead of the real one.
unsafe fn setup_plink() {
    let pl = az(8 * core::mem::size_of::<*mut HSD_GObj>()) as *mut *mut HSD_GObj;
    let nd = (rnd() % 3) as i32;
    let mut head: *mut HSD_GObj = core::ptr::null_mut();
    let mut tail: *mut HSD_GObj = core::ptr::null_mut();
    ext::HSD_GObjPLinkHead = pl;
    for d in 0..=nd {
        let l = mklights(16 + (rnd() % 5) as i32);
        if d < nd {
            (*l).classifier = 7;
        }
        if !tail.is_null() {
            (*tail).next = l;
        } else {
            head = l;
        }
        tail = l;
    }
    *pl.add(HSD_GOBJ_PLINK_LIGHT as usize) = head;
}

unsafe fn greset() {
    gr_reset();
    CB.clear();
    GNULL = false;
    JNULL = false;
    LDNULL = false;
    FPOS = false;
    SCALE = 0.0;
    PGOBJ = mkfgobj();
    (*PGOBJ).hsd_obj = mkjobj2() as *mut c_void;
    setup_plink();
}

unsafe fn state() {
    out(&format!("rs {}\n", ORS));
    out(&format!(
        "si {} {} {} {}\n",
        ext::stage_info.unk8C.b4(),
        ext::stage_info.unk8C.b5(),
        ext::stage_info.flags,
        ext::stage_info.x6DC
    ));
    dump_t("yp", &t::yakumono_param);
    gnd_canon_begin();
    for i in 0..CB.len() {
        canon_add(CB[i], 1);
    }
    dump_arena();
    canon_restore();
    gnd_canon_end();
    state_end();
}

unsafe fn mkcoll(v: i32) -> *mut CollData {
    let c = az_t::<CollData>();
    // b1234 is MSB-first at GC bits 1..4 here, LSB-first in the native C
    *((&raw mut (*c).x34_flags) as *mut u8) = (v << 3) as u8;
    out(&format!("cf {}\n", (*c).x34_flags.b1234()));
    CB.push((&raw mut (*c).x34_flags) as *mut u8);
    c
}

unsafe fn mksym() -> *mut HSD_GObj {
    let g = mkgobj();
    (*((*g).user_data as *mut Ground)).map_id = rnd() % 5;
    (*((*g).hsd_obj as *mut HSD_JObj)).flags &= !(JOBJ_USE_QUATERNION as u32);
    g
}
unsafe fn mkhub(randomize: bool) -> *mut HSD_GObj {
    let g = mkgobj();
    let gp = (*g).user_data as *mut Ground;
    (*((*g).hsd_obj as *mut HSD_JObj)).flags &= !(JOBJ_USE_QUATERNION as u32);
    for i in 0..6usize {
        (*gp).u.map.symbol[i] = if !randomize && rnd() % 5 == 0 { core::ptr::null_mut() } else { mksym() };
    }
    for i in 0..3usize {
        (*gp).u.shrineroute.platforms[i].jobj = if rnd() % 4 == 0 { core::ptr::null_mut() } else { mkjobj2() };
        (*gp).u.shrineroute.platforms[i].offset.x = rf();
        (*gp).u.shrineroute.platforms[i].offset.y = rf();
        (*gp).u.shrineroute.platforms[i].offset.z = rf();
    }
    if !randomize {
        return g;
    }
    (*gp).u.shrineroute.xC4 = (rnd() % 8) as u16;
    (*gp).u.shrineroute.xC6 = (rnd() % 64) as u16;
    (*gp).u.shrineroute.xC8 = (0xBD + rnd() % 6) as u16;
    (*gp).u.shrineroute.xCA = (rnd() % 2) as u16;
    (*gp).u.shrineroute.xCC = (rnd() % 2) as u16;
    (*gp).u.shrineroute.xCE = (if rnd() % 3 == 0 { 0 } else { rnd() % 70 }) as u16;
    (*gp).u.shrineroute.xD0 = rnd() as u16;
    (*gp).u.shrineroute.xD4 = if rnd() % 3 == 0 { 0 } else { mksym() as usize as u32 };
    g
}
unsafe fn mklights(n: i32) -> *mut HSD_GObj {
    let g = az_t::<HSD_GObj>();
    let mut prev: *mut HSD_LObj = core::ptr::null_mut();
    for _ in 0..n {
        let l = mklobj();
        if !prev.is_null() {
            (*prev).next = l;
        } else {
            (*g).hsd_obj = l as *mut c_void;
        }
        prev = l;
    }
    (*g).classifier = 12;
    g
}

macro_rules! fnr {
    ($($p:path),* $(,)?) => { $( fnreg(stringify!($p).rsplit("::").next().unwrap(), $p as *const () as usize); )* };
}

#[test]
fn matches_c() {
    unsafe {
        reset_all();
        gen_stubs::install();
        install_custom();
        ext::HOOK_efSync_Spawn = Some(t_efSync_Spawn);
        size_t::<Ground>("Ground");
        size_t::<HSD_GObj>("HSD_GObj");
        size_t::<HSD_JObj>("HSD_JObj");
        size_t::<HSD_LObj>("HSD_LObj");
        size_t::<CollData>("CollData");
        size_t::<StageCallbacks>("StageCallbacks");
        size_t::<StageData>("StageData");
        size_t::<HSD_LightDesc>("HSD_LightDesc");
        size_t::<t::grShrineRoute_YakumonoParam>("struct grShrineRoute_YakumonoParam");
        size_t::<t::grSh_Route_LightConfig>("struct grSh_Route_LightConfig");
        reg("cb", t::stage_callbacks.as_ptr(), core::mem::size_of_val(&t::stage_callbacks));
        reg("sd", &t::grSh_Route_StageData, core::mem::size_of::<StageData>());
        reg("dat", t::grSh_Route_StageData.data1, 16);
        reg("lc", t::light_config.as_ptr(), core::mem::size_of_val(&t::light_config));
        reg("l0", &t::lobj0, core::mem::size_of::<HSD_LightDesc>());
        reg("l1", &t::lobj1, core::mem::size_of::<HSD_LightDesc>());
        reg("s0", &t::lobj0_shininess, 4);
        reg("s1", &t::lobj1_shininess, 4);
        fnr!(t::grShrineRoute_OnDemoInit, t::grShrineRoute_OnInit, t::grShrineRoute_OnLoad, t::grShrineRoute_OnStart,
             t::grShrineRoute_802088B8, t::grShrineRoute_802088C0, t::stageGObj0_OnInit, t::grShrineRoute_802089D8,
             t::grShrineRoute_802089E0, t::grShrineRoute_802089E4, t::grShrineRoute_802089E8, t::grShrineRoute_80208A28,
             t::grShrineRoute_80208A30, t::grShrineRoute_80208A34, t::fn_80208A38, t::grShrineRoute_80208D14,
             t::grShrineRoute_80208F0C, t::grShrineRoute_80208F14, t::grShrineRoute_80208F70, t::grShrineRoute_80209AEC,
             t::grShrineRoute_80209AF0, t::grShrineRoute_80209BE4, t::grShrineRoute_80209BEC, t::grShrineRoute_8020A100,
             t::grShrineRoute_8020A104, t::grShrineRoute_8020A214, t::grShrineRoute_8020A21C, t::grShrineRoute_8020A864,
             t::stageGObj1_3_OnInit, t::grShrineRoute_8020A894, t::grShrineRoute_8020A89C, t::grShrineRoute_8020A8A0,
             t::grShrineRoute_8020A8A4, t::grShrineRoute_8020AA40, t::grShrineRoute_8020AB58, t::grShrineRoute_8020AC44,
             t::grShrineRoute_8020AD24, t::onJointCollision, t::grShrineRoute_8020AE08, t::grShrineRoute_8020AF38,
             t::grShrineRoute_8020B020, t::grShrineRoute_8020B0AC, t::grShrineRoute_OnTouchLine,
             t::grShrineRoute_OnCheckShadowRender, t::grShrineRoute_ScriptQuery);
        fnreg("grDisplay_801C5DB0", ext::grDisplay_801C5DB0 as *const () as usize);
        dump_t("cb", &t::stage_callbacks);
        dump_t("sd", &t::grSh_Route_StageData);
        dump_t("lc", &t::light_config);
        dump_t("tb", &t::grSh_Route_803E58E0);
        dump_t("l0", &t::lobj0);
        dump_t("l1", &t::lobj1);
        dump_t("s0", &t::lobj0_shininess);
        dump_t("s1", &t::lobj1_shininess);
        state_end();

        for c in 0..40usize {
            greset();
            case(c);
            rseed(100 + c as u32);
            t::yakumono_param = core::ptr::null_mut();
            GNULL = c % 2 != 0;
            t::grShrineRoute_OnInit();
            state();
            t::grShrineRoute_OnLoad();
            state();
            t::yakumono_param = mkyp();
            t::grShrineRoute_OnStart();
            t::grShrineRoute_OnDemoInit(rnd() as i32);
            out(&format!("r {}\n", t::grShrineRoute_802088B8()));
            state();
        }
        for c in 0..28usize {
            greset();
            case(1000 + c);
            rseed(2000 + c as u32);
            t::yakumono_param = mkyp();
            GNULL = c % 2 != 0;
            out("r");
            trp(t::grShrineRoute_802088C0((c % 7) as i32) as *const u8);
            out("\n");
            state();
        }
        for c in 0..20usize {
            greset();
            case(2000 + c);
            rseed(3000 + c as u32);
            let g = mkgobj();
            t::stageGObj0_OnInit(g);
            t::stageGObj1_3_OnInit(g);
            t::grShrineRoute_802089E8(g);
            state();
            t::grShrineRoute_802089E0(g);
            t::grShrineRoute_802089E4(g);
            t::grShrineRoute_80208A30(g);
            t::grShrineRoute_80208A34(g);
            t::grShrineRoute_80209AEC(g);
            t::grShrineRoute_8020A100(g);
            t::grShrineRoute_8020A864(g);
            t::grShrineRoute_8020A89C(g);
            t::grShrineRoute_8020A8A0(g);
            let a = t::grShrineRoute_802089D8(g);
            let b = t::grShrineRoute_80208A28(g);
            let cc = t::grShrineRoute_80208F0C(g);
            let d = t::grShrineRoute_80209BE4(g);
            let e = t::grShrineRoute_8020A214(g);
            let f = t::grShrineRoute_8020A894(g);
            out(&format!("r {} {} {} {} {} {}\n", a, b, cc, d, e, f));
            out(&format!("r {}\n", t::grShrineRoute_OnTouchLine(0).is_null() as i32));
            state();
        }
        for c in 0..60usize {
            greset();
            case(3000 + c);
            rseed(4000 + c as u32);
            t::yakumono_param = mkyp();
            let mut g = mkgobj();
            t::fn_80208A38(g);
            state();
            g = mkgobj();
            JNULL = c % 2 != 0;
            t::grShrineRoute_80208D14(g);
            state();
        }
        for c in 0..80usize {
            greset();
            case(4000 + c);
            rseed(5000 + c as u32);
            let g = mkgobj();
            let gp = (*g).user_data as *mut Ground;
            (*gp).u.shrineroute.xCA = (rnd() % 2) as u16;
            (*gp).u.shrineroute.xCC = (rnd() % 2) as u16;
            out(&format!("r {}\n", t::grShrineRoute_80208F14()));
            state();
        }
        for c in 0..700usize {
            greset();
            case(5000 + c);
            rseed(6000 + c as u32);
            t::yakumono_param = mkyp();
            let g = mkhub(true);
            for k in 0..4 {
                let hp = (*g).user_data as *mut Ground;
                if (*hp).u.shrineroute.xC4 == 4 && (*hp).u.shrineroute.xD4 == 0 {
                    (*hp).u.shrineroute.xD4 = mksym() as usize as u32;
                }
                if (*hp).u.shrineroute.xC4 == 0 {
                    for si in 0..6usize {
                        if (*hp).u.map.symbol[si].is_null() {
                            (*hp).u.map.symbol[si] = mksym();
                        }
                    }
                }
                t::grShrineRoute_80208F70(g);
                state();
                if k == 1 {
                    let gp = (*g).user_data as *mut Ground;
                    (*gp).u.shrineroute.xC4 = (rnd() % 7) as u16;
                    (*gp).u.shrineroute.xCA = (rnd() % 2) as u16;
                    (*gp).u.shrineroute.xCC = (rnd() % 2) as u16;
                }
            }
        }
        for c in 0..60usize {
            greset();
            case(6000 + c);
            rseed(7000 + c as u32);
            let g = mkgobj();
            let gj = (*g).hsd_obj as *mut HSD_JObj;
            (*gj).flags &= !(JOBJ_USE_QUATERNION as u32);
            (*gj).rotate.x = if rnd() % 2 != 0 { 0.25 } else { (rnd() % 600) as f32 / 1000.0 - 0.3 };
            (*gj).rotate.y = if rnd() % 2 != 0 { 0.17 } else { (rnd() % 400) as f32 / 1000.0 - 0.2 };
            t::grShrineRoute_80209AF0(g);
            let gp = (*g).user_data as *mut Ground;
            state();
            for k in 0..25 {
                if k % 7 == 3 {
                    (*gp).u.shrineroute3.xC8 = (rnd() % 200) as f32 / 100000.0 - 0.001;
                    (*gp).u.shrineroute3.xCC = (rnd() % 200) as f32 / 100000.0 - 0.001;
                    (*gj).rotate.x = if rnd() % 2 != 0 { 0.2615 } else { -0.2615 };
                    (*gj).rotate.y = if rnd() % 2 != 0 { 0.1744 } else { -0.1744 };
                }
                if k % 5 == 2 && !(*gp).u.shrineroute3.xC4.is_null() {
                    let c4 = (*gp).u.shrineroute3.xC4;
                    (*c4).rotate.x = if rnd() % 2 != 0 { 6.28 } else { -6.28 };
                    (*c4).rotate.y = (rnd() % 700) as f32 / 100.0 - 3.5;
                    (*gp).u.shrineroute3.xD8 = (rnd() % 100) as f32 / 100.0;
                    (*gp).u.shrineroute3.xDC = -((rnd() % 100) as f32) / 100.0;
                }
                t::grShrineRoute_80209BEC(g);
                state();
            }
        }
        for c in 0..40usize {
            greset();
            case(7000 + c);
            rseed(8000 + c as u32);
            let g = mkgobj();
            (*((*g).user_data as *mut Ground)).u.shrineroute3.xC4 =
                if c % 4 == 0 { core::ptr::null_mut() } else { mkjobj2() };
            t::grShrineRoute_8020A8A4(g);
            state();
        }
        for c in 0..80usize {
            greset();
            case(8000 + c);
            rseed(9000 + c as u32);
            t::yakumono_param = mkyp();
            let g = mkgobj();
            t::grShrineRoute_8020A104(g);
            state();
            for k in 0..3 {
                t::grShrineRoute_8020A21C(g);
                state();
            }
        }
        for c in 0..200usize {
            greset();
            case(9000 + c);
            rseed(10000 + c as u32);
            let g = mkgobj();
            let gp = (*g).user_data as *mut Ground;
            let n = 3 + (rnd() % 18) as i32;
            (*gp).u.shrineroute2.xC4 = if c % 11 == 0 { core::ptr::null_mut() } else { mkfgobj() };
            (*gp).u.shrineroute2.x168 = n as u32;
            for i in 0..n as usize {
                (*gp).u.shrineroute2.xC8[i] = mklobj();
                (*gp).u.shrineroute2.x118[i] = rnd();
            }
            (*gp).u.shrineroute2.x16C = mklobj();
            (*gp).u.shrineroute2.x170 = mklobj();
            t::grShrineRoute_8020A21C(g);
            state();
        }
        for c in 0..40usize {
            greset();
            case(10000 + c);
            rseed(11000 + c as u32);
            let mut g = mklights(16 + (rnd() % 5) as i32);
            t::grShrineRoute_8020AA40(g);
            t::grShrineRoute_8020AA40(core::ptr::null_mut());
            out("r");
            trp(t::grShrineRoute_8020AB58(g) as *const u8);
            out("\n");
            out("r");
            trp(t::grShrineRoute_8020AC44(g) as *const u8);
            out("\n");
            LDNULL = true;
            g = if c % 5 == 0 {
                core::ptr::null_mut()
            } else if c % 5 == 1 {
                mkfgobj()
            } else {
                mklights(1 + (rnd() % 4) as i32)
            };
            out("r");
            trp(t::grShrineRoute_8020AB58(g) as *const u8);
            out("\n");
            out("r");
            trp(t::grShrineRoute_8020AC44(g) as *const u8);
            out("\n");
            state();
        }
        for c in 0..80usize {
            greset();
            case(11000 + c);
            rseed(12000 + c as u32);
            t::yakumono_param = mkyp();
            FPOS = c % 2 != 0;
            SCALE = if c % 2 != 0 { 1.0 } else { 0.0 };
            let a = t::grShrineRoute_8020AD24(1);
            let b = t::grShrineRoute_8020AD24((rnd() % 3) as i32);
            out(&format!("r {} {}\n", a, b));
            let g = mkgobj();
            let gp = (*g).user_data as *mut Ground;
            for k in 0..6 {
                let v = (rnd() % 3) as i32;
                let gk = (rnd() % 4);
                let cd = mkcoll(v);
                (*cd).x0_gobj = if rnd() % 3 == 0 { mkfgobj() } else { PGOBJ };
                t::onJointCollision(gp as *mut c_void, 8, cd, 0, gk, 0.0);
                out(&format!("cc {} {}\n", (*gp).u.shrineroute.xCA, (*gp).u.shrineroute.xCC));
            }
            (*gp).u.shrineroute.xC4 = (rnd() % 5) as u16;
            let o = az(8) as *mut i32;
            out(&format!("r {}\n", t::grShrineRoute_8020AE08(g, PGOBJ, o)));
            out(&format!("out {}\n", *o));
            state();
        }
        for c in 0..60usize {
            greset();
            case(12000 + c);
            rseed(13000 + c as u32);
            t::yakumono_param = mkyp();
            let g = mkhub(false);
            t::grShrineRoute_8020AF38(g, 0xBD + (rnd() % 6) as i32);
            state();
            let a1 = if c % 4 == 0 { -1 } else { 0xBD + (rnd() % 6) as i32 };
            let a2 = (rnd() % 2) as i32;
            t::grShrineRoute_8020B020(g, a1, a2);
            state();
            t::grShrineRoute_8020B0AC();
        }
        for c in 0..40usize {
            greset();
            case(13000 + c);
            rseed(14000 + c as u32);
            let v = az_t::<Vec3>();
            (*v).x = rf();
            (*v).y = rf();
            (*v).z = rf();
            let a1 = rnd() as i32;
            let j = mkjobj2();
            out(&format!("r {}\n", t::grShrineRoute_OnCheckShadowRender(v, a1, j)));
            for i in 0..6 {
                let idx = (rnd() % 8) as i32 - 1;
                out(&format!("q {}\n", t::grShrineRoute_ScriptQuery(i % 5, idx)));
            }
            state();
        }
    }
    check("tu_grshrineroute");
}
