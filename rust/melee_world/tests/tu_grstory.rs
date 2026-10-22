//! tu::grstory against the C TU compiled natively with shipping contraction (state oracle:
//! gen/turef/tu_grstory_ref.c, data tests/data/tu_grstory.txt; regenerate with
//! `python rust/melee_world/gen/turef/tu_shard585.py grstory`).
#![allow(static_mut_refs, non_snake_case, unused)]
mod common;
use common::gr::*;
use common::*;
use melee_types::*;
use melee_world::tu::grstory as t;
use melee_world::tu_ext as ext;

unsafe fn mkyp() -> *mut t::grStory_YakumonoParam {
    let p = az_t::<t::grStory_YakumonoParam>();
    (*p).timer_min = rf();
    (*p).timer_rand = (rnd() % 20) as f32;
    (*p).spawnmany_rarity = (rnd() % 5) as f32;
    for i in 0..6 {
        (*p).vpos[i] = rf();
    }
    p
}
include!("stubs/grstory.rs");

unsafe fn state() {
    out(&format!("rs {}\n", ORS));
    out(&format!("si {} {}\n", ext::stage_info.unk8C.b4(), ext::stage_info.unk8C.b5()));
    dump_t("yp", &t::yakumono_param);
    gnd_canon_begin();
    dump_arena();
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
        size_t::<StageCallbacks>("StageCallbacks");
        size_t::<StageData>("StageData");
        size_t::<t::grStory_YakumonoParam>("struct grStory_YakumonoParam");
        reg("cb", t::grSt_StageCallbacks.as_ptr(), core::mem::size_of_val(&t::grSt_StageCallbacks));
        reg("sd", &t::grSt_StageData, core::mem::size_of::<StageData>());
        reg("dat", t::grSt_StageData.data1, 16);
        fnr!(t::grStory_801E302C, t::grStory_801E3030, t::grStory_UnkStage0_OnLoad, t::grStory_UnkStage0_OnStart,
             t::grStory_801E30D0, t::grStory_801E30D8, t::grStory_801E31C0, t::grStory_801E3224,
             t::grStory_801E322C, t::grStory_801E3230, t::grStory_801E3234, t::grStory_801E332C,
             t::grStory_801E3334, t::grStory_801E336C, t::grStory_801E3370, t::grStory_801E33D8,
             t::grStory_801E33E0, t::grStory_801E3414, t::grStory_801E3418, t::grStory_801E366C,
             t::grStory_801E36D0, t::grStory_801E36D8, t::grStory_StageSlotParamSwap);
        fnreg("grDisplay_801C5DB0", ext::grDisplay_801C5DB0 as usize);
        dump_t("cb", &t::grSt_StageCallbacks);
        dump_t("sd", &t::grSt_StageData);
        state_end();

        for c in 0..80usize {
            case(c);
            rseed(100 + c as u32);
            gr_reset();
            t::yakumono_param = mkyp();
            ext::stage_info.unk8C.set_b4((rnd() & 1) as u8);
            ext::stage_info.unk8C.set_b5((rnd() & 1) as u8);
            out("r");
            trp(t::grStory_801E30D8((c % 4) as i32) as *const u8);
            out("\n");
            state();
        }
        for c in 0..20usize {
            case(1000 + c);
            rseed(2000 + c as u32);
            gr_reset();
            t::yakumono_param = core::ptr::null_mut();
            t::grStory_801E3030();
            state();
            t::grStory_UnkStage0_OnStart();
            t::grStory_UnkStage0_OnLoad();
            t::grStory_801E302C(rnd() as i32);
            state();
        }
        for c in 0..30usize {
            case(2000 + c);
            rseed(3000 + c as u32);
            gr_reset();
            t::yakumono_param = mkyp();
            let g = mkgobj();
            {
                let gp = (*g).user_data as *mut Ground;
                (*gp).x10_flags.set_b0((rnd() & 1) as u8);
                (*gp).x10_flags.set_b5((rnd() & 1) as u8);
                (*gp).x11_flags.set_b012((rnd() & 7) as u8);
                (*gp).x11_flags.set_b7((rnd() & 1) as u8);
            }
            t::grStory_801E31C0(g);
            state();
            out(&format!("r {}\n", t::grStory_801E3224(g)));
            t::grStory_801E322C(g);
            t::grStory_801E3230(g);
            t::grStory_801E3234(g);
            state();
            out(&format!("r {}\n", t::grStory_801E332C(g)));
            t::grStory_801E336C(g);
            t::grStory_801E3370(g);
            state();
            out(&format!("r {}\n", t::grStory_801E33D8(g)));
            t::grStory_801E3414(g);
            out(&format!("r {}\n", t::grStory_801E30D0()));
            out("r");
            trp(t::grStory_801E36D0(rnd() as i32) as *const u8);
            out("\n");
            {
                let mut v = Vec3 { x: rf(), y: rf(), z: rf() };
                let a1 = rnd() as i32;
                let j = mkjobj();
                out(&format!("r {}\n", t::grStory_801E36D8(&mut v, a1, j)));
            }
            state();
        }
        for c in 0..400usize {
            case(3000 + c);
            rseed(4000 + c as u32);
            gr_reset();
            t::yakumono_param = mkyp();
            let g = mkgobj();
            let gp = (*g).user_data as *mut Ground;
            (*gp).u.shyguys.timer = if c % 3 == 0 { (rnd() % 4) as i32 } else { 0 };
            (*gp).u.shyguys.count = (rnd() % 8) as i8;
            (*gp).u.shyguys.pattern = ((rnd() % 8) as i32 - 1) as i8;
            if c % 4 == 3 {
                t::grStory_801E3334(g);
            } else {
                t::grStory_801E3418(g);
            }
            state();
        }
        for c in 0..300usize {
            case(5000 + c);
            rseed(6000 + c as u32);
            gr_reset();
            t::yakumono_param = mkyp();
            let g = mkgobj();
            let gp = (*g).user_data as *mut Ground;
            (*gp).u.randall.timer = ((rnd() % 8) as i32 - 4) as i16;
            (*gp).u.randall.jobj = if c % 5 == 0 { core::ptr::null_mut() } else { mkjobj() };
            if c % 3 == 0 {
                t::grStory_801E33E0(g);
            } else {
                t::grStory_801E366C(g);
            }
            state();
        }
        for c in 0..6usize {
            case(7000 + c);
            rseed(8000 + c as u32);
            gr_reset();
            t::yakumono_param = if c % 2 != 0 { core::ptr::null_mut() } else { mkyp() };
            let a = az(32);
            let prev = t::grStory_StageSlotParamSwap(a as *mut _);
            out("r");
            trp(prev as *const u8);
            out("\n");
            state();
        }
    }
    check("tu_grstory");
}
