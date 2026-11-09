//! tu::grrcruise against the C TU compiled natively with shipping contraction (state oracle:
//! gen/turef/tu_grrcruise_ref.c, data tests/data/tu_grrcruise.txt; regenerate with
//! `python rust/melee_world/gen/turef/tu_shard585.py grrcruise`).
#![allow(static_mut_refs, non_snake_case, unused)]
mod common;
use common::gr::*;
use common::*;
use core::ffi::c_void;
use melee_types::*;
use melee_world::tu::grrcruise as t;
use melee_world::tu_ext as ext;

static mut GNULL: bool = false;
static mut PRSUB: bool = false;
static mut JNULL: bool = false;
static mut G3: *mut HSD_GObj = core::ptr::null_mut();
static mut ENT: std::vec::Vec<*mut grRCruise_Entry> = std::vec::Vec::new();
static mut CB: std::vec::Vec<*mut u8> = std::vec::Vec::new();

fn rnz() -> f32 {
    let m = (1 + rnd() % 2000) as f32 / 16.0;
    if rnd() & 1 != 0 { m } else { -m }
}
unsafe fn mkjtree(d: i32) -> *mut HSD_JObj {
    let j = mkjobj();
    (*j).flags &= !(JOBJ_USE_QUATERNION as u32);
    if d < 2 {
        if rnd() % 3 == 0 {
            (*j).child = mkjtree(d + 1);
        }
        if rnd() % 3 == 0 {
            (*j).next = mkjtree(d + 1);
        }
    }
    j
}
unsafe fn mkdobjs() -> *mut HSD_DObj {
    let n = (rnd() % 4) as i32;
    let mut h: *mut HSD_DObj = core::ptr::null_mut();
    for _ in 0..n {
        let d = az_t::<HSD_DObj>();
        (*d).next = h;
        h = d;
    }
    h
}
fn rsmall(n: u32, d: i32) -> f32 {
    (rnd() % n) as f32 / d as f32
}
unsafe fn mkyp() -> *mut t::grRCruise_YakumonoParam {
    let p = az_t::<t::grRCruise_YakumonoParam>();
    (*p).x0 = rsmall(100, 100) + 0.01;
    (*p).x4 = rsmall(300, 100) + 0.5;
    (*p).x8 = rsmall(100, 40);
    (*p).xC = (rnd() % 12) as i32;
    (*p).x10 = (rnd() % 14) as i32;
    (*p).x14 = (1 + rnd() % 5) as i32;
    (*p).x18 = (rnd() % 14) as i32;
    (*p).x1C = (1 + rnd() % 5) as i32;
    (*p).x20 = (rnd() % 8) as i32;
    (*p).x24 = (rnd() % 10) as i32;
    (*p).x28 = (1 + rnd() % 4) as i32;
    (*p).x2C = rsmall(100, 10);
    (*p).x30 = (*p).x2C + if rnd() % 4 == 0 { 0.0 } else { rsmall(50, 10) };
    (*p).x34 = rsmall(30, 10);
    (*p).x38 = rsmall(60, 10);
    (*p).x3C = (rnd() % 8) as i32;
    (*p).x40 = (rnd() % 10) as i32;
    (*p).x44 = (1 + rnd() % 4) as i32;
    p
}
include!("stubs/grrcruise.rs");

unsafe fn greset() {
    gr_reset();
    ENT.clear();
    CB.clear();
    PRSUB = false;
    GNULL = false;
    JNULL = false;
    G3 = core::ptr::null_mut();
}

unsafe fn state() {
    out(&format!("rs {}\n", ORS));
    out(&format!("si {} {}\n", ext::stage_info.unk8C.b4(), ext::stage_info.unk8C.b5()));
    dump_t("yp", &t::yakumono_param);
    gnd_canon_begin();
    for i in 0..GPS.len() {
        let g = GPS[i];
        out(&format!("u{} {} {}\n", i, (*g).u.rcruise.xC4.b0(), (*g).u.scroll.x0()));
        canon_add((&raw mut (*g).u) as *mut u8, 1);
        for k in 0..(if PRSUB { 3 } else { 0 }) {
            out(&format!("s{}.{} {}\n", i, k, (*g).u.rcruise.x3C[k].x1_b0()));
            canon_add(((&raw mut (*g).u.rcruise.x3C[k]) as *mut u8).add(1), 1);
        }
    }
    for i in 0..ENT.len() {
        for k in 0..17 {
            let e = ENT[i].add(k);
            out(&format!("e{}.{} {}\n", i, k, (*e).pad_01 >> 7));
            canon_add(&raw mut (*e).pad_01, 1);
        }
    }
    for i in 0..CB.len() {
        canon_add(CB[i], 1);
    }
    dump_arena();
    canon_restore();
    gnd_canon_end();
    state_end();
}

unsafe fn mkmain(randomize: bool) -> *mut HSD_GObj {
    let g = mkgobj();
    let gp = (*g).user_data as *mut Ground;
    (*gp).u.rcruise.entries = az(17 * core::mem::size_of::<grRCruise_Entry>()) as *mut grRCruise_Entry;
    (*gp).u.map.vanish = az(20 * core::mem::size_of::<Map_VanishEntry>()) as *mut Map_VanishEntry;
    (*gp).u.rcruise.x4 = az_t::<lb_80011A50_t>();
    ENT.push((*gp).u.rcruise.entries);
    (*(*gp).u.rcruise.x4).x4.x = rf();
    (*(*gp).u.rcruise.x4).x4.z = rf();
    (*(*gp).u.rcruise.x4).unk_angle_int = (rnd() % 600) as i32;
    (*(*gp).u.rcruise.x4).unk_scale = rf();
    (*gp).u.rcruise.x8 = rsmall(100, 1000) - 0.05;
    (*gp).u.rcruise.xC = (rnd() % 2) as i32;
    if !randomize {
        return g;
    }
    (*gp).u.rcruise.x10 = (rnd() % 2) as i32;
    (*gp).u.rcruise.x14 = rf();
    (*gp).u.rcruise.x18 = rsmall(40000, 20) - 1000.0;
    (*gp).u.rcruise.x1C = if rnd() % 2 != 0 { 1.0 } else { -1.0 };
    (*gp).u.rcruise.x20 = rsmall(200, 100) - 1.0;
    (*gp).u.rcruise.x24 = rsmall(100, 7);
    (*gp).u.rcruise.x28 = rsmall(100, 7);
    (*gp).u.rcruise.x2C = (rnd() % 3) as i32;
    (*gp).u.rcruise.x30 = (rnd() % 5) as i32;
    (*gp).u.rcruise.x34 = (rnd() % 3) as i32;
    (*gp).u.rcruise.x38 = (rnd() % 4) as i32;
    for i in 0..17 {
        let e = (*gp).u.rcruise.entries.add(i);
        (*e).x0 = (rnd() % 7) as u8;
        (*e).pad_01 = ((rnd() & 1) as u8) << 7;
        (*e).x2 = (rnd() % 50) as i16;
        (*e).x4 = (rnd() % 12) as i32;
        (*e).x8 = (rnd() % 3) as i32;
        (*e).xC = rf();
        (*e).x10 = rsmall(60, 10) - 3.0;
        (*e).x14 = if rnd() % 8 == 0 { core::ptr::null_mut() } else { mkjobj() };
    }
    for i in 0..20 {
        let v = (*gp).u.map.vanish.add(i);
        (*v).x0 = (rnd() % 5) as i16;
        (*v).x2 = (rnd() % 30) as i16;
        (*v).jobj = mkjobj();
    }
    for i in 0..3 {
        let s = &raw mut (*gp).u.rcruise.x3C[i];
        (*s).x0 = (rnd() % 7) as u8;
        (*s).set_x1_b0((rnd() & 1) as u8);
        (*s).x2 = (rnd() % 50) as i16;
        (*s).x4 = (rnd() % 12) as i32;
        (*s).x8 = (rnd() % 3) as i32;
        (*s).xC = mkjtree(1);
    }
    g
}
unsafe fn fixent(gp: *mut Ground) {
    for i in 0..17 {
        let e = (*gp).u.rcruise.entries.add(i);
        if (*e).x14.is_null() {
            (*e).x14 = mkjobj();
        }
    }
}
unsafe fn mkcoll(v: i32) -> *mut CollData {
    let c = az_t::<CollData>();
    // b1234 is MSB-first at GC bits 1..4 here, LSB-first in the native C
    *((&raw mut (*c).x34_flags) as *mut u8) = (v << 3) as u8;
    out(&format!("cf {}\n", (*c).x34_flags.b1234()));
    CB.push((&raw mut (*c).x34_flags) as *mut u8);
    (*c).cur_pos.x = rf();
    (*c).cur_pos.y = rf();
    (*c).cur_pos.z = rf();
    c
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
        size_t::<HSD_JObj>("HSD_JObj");
        size_t::<HSD_DObj>("HSD_DObj");
        size_t::<CollData>("CollData");
        size_t::<StageCallbacks>("StageCallbacks");
        size_t::<StageData>("StageData");
        size_t::<GrJoint>("GrJoint");
        size_t::<t::grRCruise_YakumonoParam>("struct grRCruise_YakumonoParam");
        size_t::<grRCruise_Entry>("struct grRCruise_Entry");
        size_t::<grRCruise_SubEntry>("struct grRCruise_SubEntry");
        size_t::<Map_VanishEntry>("struct Map_VanishEntry");
        size_t::<Map_VanishDesc>("struct Map_VanishDesc");
        size_t::<lb_80011A50_t>("struct lb_80011A50_t");
        reg("cb", t::grRc_StageCallbacks.as_ptr(), core::mem::size_of_val(&t::grRc_StageCallbacks));
        reg("sd", &t::grRc_StageData, core::mem::size_of::<StageData>());
        reg("dat", t::grRc_StageData.data1, 16);
        reg("jt", t::grRc_803E4DA8.as_ptr(), core::mem::size_of_val(&t::grRc_803E4DA8));
        fnr!(t::grRCruise_801FF164, t::grRCruise_801FF168, t::grRCruise_801FF298, t::grRCruise_801FF29C,
             t::grRCruise_801FF2C0, t::grRCruise_801FF2C8, t::grRCruise_801FF3B4, t::grRCruise_801FF3E0,
             t::grRCruise_801FF3E8, t::grRCruise_801FF3EC, t::grRCruise_801FF3F0, t::grRCruise_801FF434,
             t::grRCruise_801FF43C, t::grRCruise_801FF440, t::grRCruise_801FF444, t::grRCruise_801FF5B4,
             t::grRCruise_801FF6CC, t::grRCruise_801FF6D4, t::grRCruise_801FF738, t::grRCruise_801FF73C,
             t::grRCruise_801FF794, t::grRCruise_801FF79C, t::grRCruise_801FF7A0, t::grRCruise_801FF7A4,
             t::grRCruise_801FF8DC, t::grRCruise_801FF8E4, t::grRCruise_801FF920, t::grRCruise_801FF924,
             t::grRCruise_801FFAD4, t::grRCruise_801FFADC, t::grRCruise_80200070, t::grRCruise_80200074,
             t::grRCruise_8020014C, t::grRCruise_80200154, t::grRCruise_8020045C, t::fn_80200460,
             t::grRCruise_80200540, t::grRCruise_80200578, t::grRCruise_8020071C, t::grRCruise_80200B48,
             t::grRCruise_80200C04, t::grRCruise_802010A4, t::grRCruise_80201110, t::grRCruise_80201288,
             t::grRCruise_80201410, t::grRCruise_80201588, t::grRCruise_80201918, t::grRCruise_80201988,
             t::grRCruise_80201B60, t::fn_80201BE0, t::grRCruise_80201C50, t::grRCruise_80201C58);
        fnreg("grDisplay_801C5DB0", ext::grDisplay_801C5DB0 as usize);
        fnreg("HSD_AObjSetFlags", ext::HSD_AObjSetFlags as usize);
        fnreg("HSD_DObjSetFlags", ext::HSD_DObjSetFlags as usize);
        fnreg("HSD_DObjClearFlags", ext::HSD_DObjClearFlags as usize);
        dump_t("cb", &t::grRc_StageCallbacks);
        dump_t("sd", &t::grRc_StageData);
        dump_t("jt", &t::grRc_803E4DA8);
        dump_t("sl", &t::grRc_804D4790);
        dump_t("bl", &t::grRc_803E4FF0);
        dump_t("vd", &t::lbl_803E5014);
        state_end();

        for c in 0..28usize {
            greset();
            case(c);
            rseed(100 + c as u32);
            GNULL = c % 2 != 0;
            out("r");
            trp(t::grRCruise_801FF2C8((c % 7) as i32) as *const u8);
            out("\n");
            state();
        }
        for c in 0..20usize {
            greset();
            case(1000 + c);
            rseed(2000 + c as u32);
            t::yakumono_param = core::ptr::null_mut();
            t::grRCruise_801FF168();
            state();
            t::grRCruise_801FF164(rnd() as i32);
            t::grRCruise_801FF298();
            t::grRCruise_801FF29C();
            out(&format!("r {}\n", t::grRCruise_801FF2C0() as i32));
            state();
        }
        for c in 0..30usize {
            greset();
            case(2000 + c);
            rseed(3000 + c as u32);
            JNULL = c % 2 != 0;
            let g = mkgobj();
            t::grRCruise_801FF3B4(g);
            t::grRCruise_801FF3F0(g);
            state();
            t::grRCruise_801FF73C(g);
            t::grRCruise_801FF8E4(g);
            t::grRCruise_801FF444(g);
            state();
            let a = t::grRCruise_801FF3E0(g) as i32;
            let b = t::grRCruise_801FF434(g) as i32;
            let cc = t::grRCruise_801FF6CC(g) as i32;
            let d = t::grRCruise_801FF794(g) as i32;
            out(&format!("r {} {} {} {}\n", a, b, cc, d));
            t::grRCruise_801FF3E8(g);
            t::grRCruise_801FF3EC(g);
            t::grRCruise_801FF43C(g);
            t::grRCruise_801FF440(g);
            t::grRCruise_801FF738(g);
            t::grRCruise_801FF79C(g);
            t::grRCruise_801FF7A0(g);
            t::grRCruise_801FF920(g);
            t::grRCruise_80200070(g);
            t::grRCruise_8020045C(g);
            let a = t::grRCruise_801FF8DC(g) as i32;
            let b = t::grRCruise_801FFAD4(g) as i32;
            let cc = t::grRCruise_8020014C(g) as i32;
            out(&format!("r {} {} {}\n", a, b, cc));
            let a = t::grRCruise_80201C58(core::ptr::null_mut(), 0, core::ptr::null_mut()) as i32;
            let b = t::grRCruise_80201C50(0).is_null() as i32;
            out(&format!("r {} {}\n", a, b));
            state();
        }
        for c in 0..40usize {
            greset();
            case(3000 + c);
            rseed(4000 + c as u32);
            let g = mkgobj();
            JNULL = false;
            t::yakumono_param = mkyp();
            t::grRCruise_801FF5B4(g);
            ENT.push((*((*g).user_data as *mut Ground)).u.rcruise.entries);
            state();
            for k in 0..6 {
                let gp = (*g).user_data as *mut Ground;
                fixent(gp);
                (*gp).u.rcruise.x10 = (c % 3 == 0) as i32;
                t::grRCruise_801FF6D4(g);
                state();
            }
        }
        for c in 0..120usize {
            greset();
            case(4000 + c);
            PRSUB = true;
            rseed(5000 + c as u32);
            t::yakumono_param = mkyp();
            let g = mkmain(true);
            let gp = (*g).user_data as *mut Ground;
            fixent(gp);
            for k in 0..8 {
                if c % 4 == 0 {
                    t::grRCruise_80201588(g);
                }
                if c % 4 == 1 {
                    t::grRCruise_8020071C(g);
                }
                if c % 4 == 2 {
                    t::grRCruise_80200C04(g);
                }
                if c % 4 == 3 {
                    t::grRCruise_80201588(g);
                    t::grRCruise_8020071C(g);
                    t::grRCruise_80200C04(g);
                }
                for e in 0..3 {
                    let mut cd = mkcoll((rnd() % 4) as i32);
                    let a1 = (rnd() % 50) as i32;
                    let a3 = (rnd() % 3000) as i32;
                    let df = rf();
                    t::grRCruise_80200578(g as *mut c_void, a1, cd, a3, 0, df);
                    cd = mkcoll((rnd() % 3) as i32);
                    let jid = (*(*gp).u.rcruise.entries.add((rnd() % 17) as usize)).x2 as i32;
                    let df = rf();
                    t::grRCruise_802010A4(g as *mut c_void, jid, cd, 0, 0, df);
                }
                state();
            }
        }
        for c in 0..40usize {
            greset();
            case(5000 + c);
            PRSUB = true;
            rseed(6000 + c as u32);
            let g = mkmain(true);
            let gp = (*g).user_data as *mut Ground;
            t::grRCruise_80200540(g);
            state();
            t::grRCruise_80200B48(g);
            state();
            t::grRCruise_80201410(g);
            state();
            t::grRCruise_801FF6D4(g);
            state();
        }
        for c in 0..80usize {
            greset();
            case(6000 + c);
            PRSUB = true;
            rseed(7000 + c as u32);
            t::yakumono_param = mkyp();
            let g = mkgobj();
            let gp = (*g).user_data as *mut Ground;
            t::grRCruise_80200074(g);
            state();
            for k in 0..10 {
                if k % 3 == 1 {
                    for i in 0..3 {
                        (*gp).u.rcruise.x3C[i].x0 = (rnd() % 7) as u8;
                        (*gp).u.rcruise.x3C[i].set_x1_b0((rnd() & 1) as u8);
                        (*gp).u.rcruise.x3C[i].x4 = (rnd() % 14) as i32;
                        (*gp).u.rcruise.x3C[i].x8 = (rnd() % 2) as i32;
                    }
                }
                for e in 0..2 {
                    let cd = mkcoll((rnd() % 3) as i32);
                    let jid = (*gp).u.rcruise.x3C[(rnd() % 4 % 3) as usize].x2 as i32;
                    let df = rf();
                    t::fn_80200460(g as *mut c_void, jid, cd, 0, 0, df);
                }
                t::grRCruise_80200154(g);
                state();
            }
        }
        for c in 0..20usize {
            greset();
            case(7000 + c);
            rseed(8000 + c as u32);
            JNULL = c % 2 != 0;
            let g = mkgobj();
            t::grRCruise_801FF7A4(g);
            state();
        }
        for c in 0..20usize {
            greset();
            case(8000 + c);
            rseed(9000 + c as u32);
            let g = mkgobj();
            t::grRCruise_801FF924(g);
            state();
            t::fn_80201BE0(g, (rnd() % 4) as isize);
            t::fn_80201BE0(g, (rnd() % 4) as isize);
            state();
        }
        for c in 0..80usize {
            greset();
            case(9000 + c);
            rseed(10000 + c as u32);
            let g = mkgobj();
            let gp = (*g).user_data as *mut Ground;
            let ag = mkgobj();
            (*gp).u.scroll.anim_gobj = ag;
            (*((*ag).user_data as *mut Ground)).u.scroll.int_jobj = mkjobj();
            (*((*ag).user_data as *mut Ground)).u.scroll.cam_jobj = mkjobj();
            (*gp).u.scroll.set_x0((c % 4 == 0) as u8);
            (*gp).u.scroll.x10.x = rf();
            (*gp).u.scroll.x10.y = rf();
            (*gp).u.scroll.x10.z = rf();
            (*gp).u.scroll.x1C.x = rf();
            (*gp).u.scroll.x1C.y = rf();
            (*gp).u.scroll.x1C.z = rf();
            if c % 9 == 1 {
                (*g).hsd_obj = core::ptr::null_mut();
            }
            t::grRCruise_801FFADC(g);
            state();
            let v = az_t::<Vec3>();
            (*v).x = rf();
            t::grRCruise_80201918(v);
            out("v");
            trf((*v).x);
            trf((*v).y);
            trf((*v).z);
            out("\n");
            state();
        }
        for c in 0..60usize {
            greset();
            case(10000 + c);
            rseed(11000 + c as u32);
            let g = mkmain(false);
            let gp = (*g).user_data as *mut Ground;
            (*(*gp).u.rcruise.x4).unk_angle_int =
                (30 * (rnd() % 20) as i32) + if c % 3 == 0 { 0 } else { (rnd() % 2) as i32 };
            (*(*gp).u.rcruise.x4).x4.x = if c % 5 == 0 { 0.1 } else { rf() };
            (*(*gp).u.rcruise.x4).x4.z = if c % 7 == 0 { 5.0 } else { rf() };
            (*gp).u.rcruise.x8 = if c % 4 == 0 { 5.0 } else { rsmall(100, 100) - 0.5 };
            for k in 0..4 {
                t::grRCruise_80201110(g);
                out("l");
                trf((*(*gp).u.rcruise.x4).x4.x);
                trf((*(*gp).u.rcruise.x4).x4.z);
                trf((*(*gp).u.rcruise.x4).unk_scale);
                trf((*gp).u.rcruise.x8);
                out("\n");
            }
            state();
        }
        for c in 0..60usize {
            greset();
            case(11000 + c);
            rseed(12000 + c as u32);
            ext::stage_info.grkind = if c % 3 == 0 { 7 } else { Gr_Kind_RCruise };
            let a = if c % 11 == 0 { -1 } else { rnd() as i32 };
            out(&format!("r {}\n", t::grRCruise_80201988(a)));
            state();
        }
        for c in 0..40usize {
            greset();
            case(12000 + c);
            rseed(13000 + c as u32);
            let j = mkjtree(0);
            t::grRCruise_80201B60(j, (rnd() % 2) as i32);
            t::grRCruise_80201B60(core::ptr::null_mut(), (rnd() % 2) as i32);
            t::grRCruise_80201288(j, ext::HSD_DObjSetFlags, rnd());
            t::grRCruise_80201288(j, ext::HSD_DObjClearFlags, rnd());
            t::grRCruise_80201288(core::ptr::null_mut(), ext::HSD_DObjClearFlags, rnd());
            state();
        }
    }
    check("tu_grrcruise");
}
