//! tu::grshrine against the C TU compiled natively with shipping contraction (state oracle:
//! gen/turef/tu_grshrine_ref.c, data tests/data/tu_grshrine.txt; regenerate with
//! `python rust/melee_world/gen/turef/tu_shard585.py grshrine`).
#![allow(static_mut_refs, non_snake_case, unused)]
mod common;
use common::gr::*;
use common::*;
use melee_types::*;
use melee_world::tu::grshrine as t;
use melee_world::tu_ext as ext;

include!("stubs/grshrine.rs");

unsafe fn state() {
    out(&format!("rs {}\n", ORS));
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
        size_t::<StageCallbacks>("StageCallbacks");
        size_t::<StageData>("StageData");
        reg("cb", t::grSh_StageCallbacks.as_ptr(), core::mem::size_of_val(&t::grSh_StageCallbacks));
        reg("sd", &t::grSh_StageData, core::mem::size_of::<StageData>());
        reg("dat", t::grSh_StageData.data1, 16);
        fnr!(t::grShrine_80201C60, t::grShrine_80201C64, t::grShrine_UnkStage0_OnLoad, t::grShrine_UnkStage0_OnStart,
             t::grShrine_80201D18, t::grShrine_80201D20, t::stageGObj0_OnInit, t::grShrine_80201E34,
             t::grShrine_80201E3C, t::grShrine_80201E40, t::grShrine_80201E44, t::grShrine_80201E88,
             t::grShrine_80201E90, t::grShrine_80201E94, t::grShrine_80201E98, t::grShrine_80201E9C,
             t::grShrine_80201F14, t::grShrine_80201F1C, t::grShrine_80201F40, t::grShrine_80201F44,
             t::grShrine_80201F4C);
        fnreg("grDisplay_801C5DB0", ext::grDisplay_801C5DB0 as usize);
        dump_t("cb", &t::grSh_StageCallbacks);
        dump_t("sd", &t::grSh_StageData);
        state_end();

        for c in 0..60usize {
            case(c);
            rseed(100 + c as u32);
            gr_reset();
            out("r");
            trp(t::grShrine_80201D20((c % 3) as i32) as *const u8);
            out("\n");
            state();
        }
        for c in 0..20usize {
            case(1000 + c);
            rseed(2000 + c as u32);
            gr_reset();
            t::grShrine_80201C64();
            state();
            t::grShrine_UnkStage0_OnStart();
            t::grShrine_UnkStage0_OnLoad();
            t::grShrine_80201C60(rnd() as i32);
            state();
        }
        for c in 0..24usize {
            case(2000 + c);
            rseed(3000 + c as u32);
            gr_reset();
            let g = mkgobj();
            {
                let gp = (*g).user_data as *mut Ground;
                (*gp).x10_flags.set_b0((rnd() & 1) as u8);
                (*gp).x10_flags.set_b3((rnd() & 1) as u8);
                (*gp).x10_flags.set_b5((rnd() & 1) as u8);
                (*gp).x11_flags.set_b012((rnd() & 7) as u8);
                (*gp).x11_flags.set_b4((rnd() & 1) as u8);
                (*gp).x11_flags.set_b7((rnd() & 1) as u8);
            }
            t::stageGObj0_OnInit(g);
            state();
            out(&format!("r {}\n", t::grShrine_80201E34(g)));
            t::grShrine_80201E3C(g);
            t::grShrine_80201E40(g);
            t::grShrine_80201E44(g);
            state();
            out(&format!("r {}\n", t::grShrine_80201E88(g)));
            t::grShrine_80201E90(g);
            t::grShrine_80201E94(g);
            t::grShrine_80201E98(g);
            t::grShrine_80201E9C(g);
            state();
            out(&format!("r {}\n", t::grShrine_80201F14(g)));
            t::grShrine_80201F1C(g);
            t::grShrine_80201F40(g);
            state();
            out(&format!("r {}\n", t::grShrine_80201D18()));
            out("r");
            trp(t::grShrine_80201F44(rnd() as i32) as *const u8);
            out("\n");
            {
                let mut v = Vec3 { x: rf(), y: rf(), z: rf() };
                let a1 = rnd() as i32;
                let j = mkjobj();
                out(&format!("r {}\n", t::grShrine_80201F4C(&mut v, a1, j)));
            }
            state();
        }
    }
    check("tu_grshrine");
}
