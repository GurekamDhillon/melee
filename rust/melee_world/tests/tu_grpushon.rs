//! tu::grpushon against the C TU compiled natively with shipping contraction (state oracle:
//! gen/turef/tu_grpushon_ref.c, data tests/data/tu_grpushon.txt; regenerate with
//! `python rust/melee_world/gen/turef/tu_shard585.py grpushon`).
#![allow(static_mut_refs, non_snake_case, unused)]
mod common;
use common::gr::*;
use common::*;
use core::ffi::c_void;
use melee_types::*;
use melee_world::tu::grpushon as t;
use melee_world::tu_ext as ext;

static mut SCALE: f32 = 0.0;
static mut FMODE: i32 = 0;
static mut LDNULL: bool = false;
static mut GKIND: [u8; 16] = [0; 16];

unsafe fn mklobj() -> *mut HSD_LObj {
    let l = az_t::<HSD_LObj>();
    (*l).flags = rnd() as u16;
    (*l).next = core::ptr::null_mut();
    l
}
unsafe fn mkfgobj() -> *mut HSD_GObj {
    az_t::<HSD_GObj>()
}
unsafe fn mkmapgobj() -> *mut HSD_GObj {
    let g = mkgobj();
    GKIND[GPS.len() - 1] = 1;
    if rnd() % 4 == 0 {
        (*g).user_data = core::ptr::null_mut();
    } else {
        (*((*g).user_data as *mut Ground)).u.map.set_xC4_b0(rnd() & 1);
    }
    g
}
unsafe fn mkstagegobj(id: i32) -> *mut HSD_GObj {
    let g = mkgobj();
    GKIND[GPS.len() - 1] = if id == 2 { 2 } else { 1 };
    g
}
unsafe fn mkypp() -> *mut t::grPushon_YakumonoParam {
    let p = az_t::<t::grPushon_YakumonoParam>();
    (*p).x0 = (rnd() % 100) as i32;
    (*p).x4 = az(16) as *mut DynamicsDesc;
    (*p).x8 = az(16) as *mut DynamicsDesc;
    (*p).xC = az(16) as *mut DynamicsDesc;
    (*p).x10 = az(16) as *mut DynamicsDesc;
    (*p).x14 = az(16) as *mut DynamicsDesc;
    (*p).x18 = (rnd() & 1) as i32;
    for i in 0..0x1E {
        (*p).x1c[i].x0 = rnd() as i32;
        (*p).x1c[i].x4 = rnd() as i16;
        (*p).x1c[i].x6 = rnd() as i16;
    }
    for i in 0..0x21 {
        (*p).x10c[i].key = if i < 8 {
            i as i32
        } else if i == 8 {
            -1
        } else {
            (rnd() % 20) as i32
        };
        (*p).x10c[i].value = (rnd() % 1000) as i32;
    }
    p
}
include!("stubs/grpushon.rs");

unsafe fn mklightgobj(n: usize) -> *mut HSD_GObj {
    let g = az_t::<HSD_GObj>();
    let mut prev: *mut HSD_LObj = core::ptr::null_mut();
    for i in 0..n {
        let l = mklobj();
        if i > 0 {
            (*l).flags = (((*l).flags as u32 & !3) | 2) as u16;
        }
        if !prev.is_null() {
            (*prev).next = l;
        } else {
            (*g).hsd_obj = l as *mut c_void;
        }
        prev = l;
    }
    (*g).classifier = 12;
    (*g).obj_kind = ext::HSD_GObj_LightKind as u8;
    g
}

unsafe fn mkgx(c: usize) {
    ext::HSD_GObj_LightKind = (rnd() % 8) as i8;
    let gx = az(5 * core::mem::size_of::<*mut HSD_GObj>()) as *mut *mut HSD_GObj;
    ext::HSD_GObjGXLinkHead = gx;
    let nd = (rnd() % 3) as usize;
    let mut head: *mut HSD_GObj = core::ptr::null_mut();
    let mut tail: *mut HSD_GObj = core::ptr::null_mut();
    for d in 0..=nd {
        let l = mklightgobj(3 + (rnd() % 6) as usize);
        if d < nd || c % 5 == 0 {
            let w = rnd() % 3;
            if w == 0 {
                (*l).classifier = 7;
            }
            if w == 1 {
                (*l).obj_kind = (ext::HSD_GObj_LightKind as u8).wrapping_add(1);
            }
            if w == 2 {
                (*l).classifier = 12;
                (*l).obj_kind = ext::HSD_GObj_LightKind as u8;
            }
        }
        if !tail.is_null() {
            (*tail).next_gx = l;
        } else {
            head = l;
        }
        tail = l;
    }
    *gx.add(4) = head;
}

unsafe fn greset() {
    gr_reset();
    GKIND = [0; 16];
    SCALE = 0.0;
    FMODE = 0;
    LDNULL = false;
}

unsafe fn state() {
    out(&format!("rs {}\n", ORS));
    out(&format!("si {} {}\n", ext::stage_info.unk8C.b4(), ext::stage_info.unk8C.b5()));
    dump_t("yp", &t::yakumono_param);
    dump_t("gx", &ext::HSD_GObjGXLinkHead);
    gnd_canon_begin();
    for i in 0..GPS.len() {
        if GKIND[i] != 1 {
            continue;
        }
        out(&format!("m{} {}\n", i, (*GPS[i]).u.map.xC4_b0()));
        canon_add((&raw mut (*GPS[i]).u) as *mut u8, 4);
    }
    dump_arena();
    canon_restore();
    gnd_canon_end();
    state_end();
}

macro_rules! fnr {
    ($($p:path),* $(,)?) => { $( fnreg(stringify!($p).rsplit("::").next().unwrap(), $p as usize); )* };
}

#[test]
fn matches_c() {
    unsafe {
        reset_all();
        gen_stubs::install();
        install_custom();
        size_t::<Ground>("Ground");
        size_t::<HSD_GObj>("HSD_GObj");
        size_t::<HSD_LObj>("HSD_LObj");
        size_t::<CollData>("CollData");
        size_t::<StageCallbacks>("StageCallbacks");
        size_t::<StageData>("StageData");
        size_t::<t::grPushon_YakumonoParam>("struct grPushon_YakumonoParam");
        size_t::<t::grPushOn_LightConfig>("struct grPushOn_LightConfig");
        reg("cb", t::grPushOn_StageCallbacks.as_ptr(), core::mem::size_of_val(&t::grPushOn_StageCallbacks));
        reg("sd", &t::grPushOn_StageData, core::mem::size_of::<StageData>());
        reg("dat", t::grPushOn_StageData.data1, 16);
        reg("ld1", &t::grPushOn_803E7B74, core::mem::size_of::<HSD_LightDesc>());
        reg("ld2", &t::grPushOn_803E7B90, core::mem::size_of::<HSD_LightDesc>());
        reg("f34", &t::grPushOn_804D4934, 4);
        reg("f48", &t::grPushOn_804D4948, 4);
        fnr!(t::grPushOn_802182C4, t::grPushOn_802182C8, t::grPushOn_80218330, t::grPushOn_80218378,
             t::grPushOn_802183DC, t::grPushOn_802183E4, t::grPushOn_802184CC, t::grPushOn_80218590,
             t::grPushOn_80218598, t::grPushOn_8021859C, t::grPushOn_802185A0, t::grPushOn_80218670,
             t::fn_80218678, t::grPushOn_802186C8, t::grPushOn_802187A4, t::grPushOn_802187A8,
             t::grPushOn_80218880, t::grPushOn_80218888, t::grPushOn_80218ED0, t::grPushOn_80218ED4,
             t::grPushOn_80218FC0, t::fn_802190A0, t::grPushOn_802190D0, t::grPushOn_80219204,
             t::grPushOn_80219230, t::fn_802192A4, t::grPushOn_80219458, t::grPushOn_80219528);
        fnreg("grDisplay_801C5DB0", ext::grDisplay_801C5DB0 as usize);
        fnreg("HSD_AObjSetFlags", ext::HSD_AObjSetFlags as usize);
        dump_t("cb", &t::grPushOn_StageCallbacks);
        dump_t("sd", &t::grPushOn_StageData);
        dump_t("ld1", &t::grPushOn_803E7B74);
        dump_t("ld2", &t::grPushOn_803E7B90);
        dump_t("cfg", &t::light_configs);
        dump_t("tbl", &t::grPushOn_803E7CCC);
        dump_t("v40", &t::grPushOn_803B8440);
        dump_t("v4c", &t::grPushOn_803B844C);
        dump_t("v58", &t::grPushOn_803B8458);
        dump_t("v64", &t::grPushOn_803B8464);
        dump_t("f34", &t::grPushOn_804D4934);
        dump_t("f48", &t::grPushOn_804D4948);
        state_end();

        for c in 0..60usize {
            greset();
            case(c);
            rseed(100 + c as u32);
            t::yakumono_param = mkypp();
            ext::stage_info.unk8C.set_b4((rnd() & 1) as u8);
            ext::stage_info.unk8C.set_b5((rnd() & 1) as u8);
            mkgx(c);
            out("r");
            trp(t::grPushOn_802183E4((c % 3) as i32) as *const u8);
            out("\n");
            state();
        }
        for c in 0..40usize {
            greset();
            case(1000 + c);
            rseed(2000 + c as u32);
            mkgx(c);
            t::yakumono_param = if c % 2 != 0 { mkypp() } else { core::ptr::null_mut() };
            t::grPushOn_802182C8();
            state();
            t::grPushOn_80218330();
            state();
            t::yakumono_param = mkypp();
            t::grPushOn_80218378();
            state();
            t::grPushOn_802182C4(rnd() as i32);
            out(&format!("r {}\n", t::grPushOn_802183DC()));
            state();
        }
        for c in 0..40usize {
            greset();
            case(2000 + c);
            rseed(3000 + c as u32);
            let g = mkgobj();
            GKIND[GPS.len() - 1] = 1;
            t::grPushOn_802184CC(g);
            state();
            out(&format!("r {}\n", t::grPushOn_80218590(g)));
            t::grPushOn_80218598(g);
            t::grPushOn_8021859C(g);
            t::grPushOn_802185A0(g);
            state();
            out(&format!("r {}\n", t::grPushOn_80218670(g)));
            t::grPushOn_802187A4(g);
            out(&format!("r {}\n", t::grPushOn_80218880(g)));
            t::grPushOn_80218ED0(g);
            {
                let mut v = Vec3 { x: rf(), y: rf(), z: rf() };
                let a1 = rnd() as i32;
                let j = mkjobj();
                out(&format!("r {}\n", t::grPushOn_80219528(&mut v, a1, j)));
            }
            state();
        }
        for c in 0..60usize {
            greset();
            case(3000 + c);
            rseed(4000 + c as u32);
            let g = mkgobj();
            GKIND[GPS.len() - 1] = 1;
            (*((*g).user_data as *mut Ground)).u.map.set_xC4_b0(rnd() & 1);
            out(&format!("r {}\n", t::fn_80218678()));
            t::grPushOn_802186C8(g);
            state();
            out(&format!("r {}\n", t::fn_80218678()));
        }
        for c in 0..40usize {
            greset();
            case(4000 + c);
            rseed(5000 + c as u32);
            let g = mkgobj();
            GKIND[GPS.len() - 1] = 1;
            let coll = az_t::<CollData>();
            *((&raw mut (*coll).x34_flags) as *mut u8) = rnd() as u8;
            if c % 2 != 0 {
                let b = *((&raw mut (*coll).x34_flags) as *mut u8);
                *((&raw mut (*coll).x34_flags) as *mut u8) = (b & !0x78) | 0x08;
            }
            let jt = rnd() % 5;
            let a1 = rnd() as i32;
            let a2 = rnd() as i32;
            let df = rf();
            t::fn_802190A0((*g).user_data, a1, coll, a2, jt, df);
            out(&format!("raw {}\n", *((&raw mut (*coll).x34_flags) as *mut u8)));
            state();
        }
        for c in 0..50usize {
            greset();
            case(5000 + c);
            rseed(6000 + c as u32);
            let g = mklightgobj(1 + (rnd() % 12) as usize);
            SCALE = if c % 3 == 0 { 1.0 } else { 0.0 };
            t::grPushOn_802190D0(g);
            state();
        }
        for c in 0..80usize {
            greset();
            case(6000 + c);
            rseed(7000 + c as u32);
            LDNULL = c % 4 == 1;
            let g = if c % 10 == 0 {
                core::ptr::null_mut()
            } else if c % 10 == 1 {
                mklightgobj(0)
            } else {
                mklightgobj(1 + (rnd() % 4) as usize)
            };
            out("r");
            trp(t::grPushOn_80218ED4(g) as *const u8);
            out("\n");
            out("r");
            trp(t::grPushOn_80218FC0(g) as *const u8);
            out("\n");
            state();
        }
        for c in 0..120usize {
            greset();
            case(7000 + c);
            rseed(8000 + c as u32);
            let g = mkgobj();
            GKIND[GPS.len() - 1] = 1;
            GKIND[GPS.len() - 1] = 2;
            mkgx(c);
            t::grPushOn_802187A8(g);
            state();
        }
        for c in 0..200usize {
            greset();
            case(8000 + c);
            rseed(9000 + c as u32);
            let g = mkgobj();
            GKIND[GPS.len() - 1] = 1;
            let gp = (*g).user_data as *mut Ground;
            GKIND[GPS.len() - 1] = 2;
            let n = 1 + (rnd() % 20) as usize;
            (*gp).u.pushon.gobj = if c % 9 == 0 { core::ptr::null_mut() } else { mkfgobj() as *mut c_void };
            (*gp).u.pushon.count = n as i32;
            for i in 0..n {
                (*gp).u.pushon.lobjs[i] = mklobj();
                (*gp).u.pushon.lobj_flags[i] = rnd();
            }
            (*gp).u.pushon.point_light = mklobj();
            (*gp).u.pushon.spot_light = mklobj();
            t::grPushOn_80218888(g);
            state();
        }
        for c in 0..20usize {
            greset();
            case(9000 + c);
            rseed(10000 + c as u32);
            t::yakumono_param = mkypp();
            let mut o1 = 0i32;
            let mut o2 = 0i32;
            t::grPushOn_80219204(0x99 + (rnd() % 30) as i32, &mut o1, &mut o2);
            out(&format!("r {} {}\n", o1, o2));
            state();
        }
        for c in 0..60usize {
            greset();
            case(10000 + c);
            rseed(11000 + c as u32);
            t::yakumono_param = mkypp();
            out(&format!("r {}\n", t::grPushOn_80219230((rnd() % 8) as i32)));
            state();
        }
        for c in 0..200usize {
            greset();
            case(11000 + c);
            rseed(12000 + c as u32);
            t::yakumono_param = mkypp();
            FMODE = (c % 2) as i32;
            SCALE = if c % 4 < 2 { 1.0 } else { 0.0 };
            let g = mkfgobj();
            let res = az(8) as *mut i32;
            out(&format!("r {}\n", t::fn_802192A4(core::ptr::null_mut(), g, res)));
            out(&format!("res {}\n", *res));
            state();
        }
        for c in 0..100usize {
            greset();
            case(12000 + c);
            rseed(13000 + c as u32);
            t::yakumono_param = mkypp();
            out("r");
            let a = if c % 7 == 0 { -1 } else { rnd() as i32 };
            trp(t::grPushOn_80219458(a) as *const u8);
            out("\n");
            state();
        }
    }
    check("tu_grpushon");
}
