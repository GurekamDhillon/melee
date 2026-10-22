//! tu::gm_18A1 against the C TU compiled natively with shipping contraction (state oracle:
//! gen/turef/tu_gm_18A1_ref.c, data tests/data/tu_gm_18A1.txt; regenerate with
//! `python rust/melee_modes/gen/turef/tu_shard529.py gm_18A1`).
#![allow(static_mut_refs, non_snake_case, unused)]
mod common;
use common::*;
use melee_modes::tu::gm_18A1 as t;
use melee_modes::tu_ext as ext;
use melee_types::*;

static mut G_RULES: StartMeleeRules = unsafe { core::mem::zeroed() };
include!("stubs/gm_18A1.rs");

#[test]
fn matches_c() {
    unsafe {
        reset_all();
        gen_stubs::install();
        size_t::<MatchEnd>("MatchEnd");
        size_t::<UnkAllstarData>("UnkAllstarData");
        size_t::<gmm_x0_528_t>("gmm_x0_528_t");
        size_t::<t::ClassicStageEntry>("ClassicStageEntry");
        reg("rules", &G_RULES, core::mem::size_of::<StartMeleeRules>());
        reg("allstar", &t::gm_80473A18, core::mem::size_of::<UnkAllstarData>());
        dump("tbl", t::gmTraining_ClassicStages.as_ptr() as *const u8, core::mem::size_of_val(&t::gmTraining_ClassicStages));
        state_end();
        for d in 0..13u8 {
            for s in 0..5u8 {
                let e = s as i32 + d as i32 * 5;
                if e > 64 {
                    continue;
                }
                out(&format!(
                    "row {} {} {} {:08x} {:08x} {:08x} {:08x}",
                    d,
                    s,
                    t::gm_8018A160(d, s),
                    t::gm_8018A188(d, s).to_bits(),
                    t::gm_8018A1D8(d, s).to_bits(),
                    t::gm_8018A2C4(d, s).to_bits(),
                    t::gm_8018A314(d, s).to_bits()
                ));
                for k in 0..4u8 {
                    if k == 3 && e == 64 {
                        continue;
                    }
                    out(&format!(" {} {} {}", t::gm_8018A228(d, s, k), t::gm_8018A25C(d, s, k), t::gm_8018A290(d, s, k)));
                }
                out("\n");
            }
        }
        for c in 0..3000usize {
            case(c);
            rseed(40 + c as u32);
            arena_reset();
            rfill_t(&mut t::gm_80473A18);
            t::gm_80473A18.x0.x0.cpu_level = (rnd() % 6) as _;
            t::gm_80473A18.x0.xC.x20 = rnd() % 0xA000;
            t::gm_80473A18.x0.xC.x1C = (rnd() % 4) as i32;
            t::gm_80473A18.x0.xC.xE = (rnd() % 2) as u8;
            t::gm_80473A18.x0.xC.xF = (rnd() % 2) as u8;
            t::gm_80473A18.x0.xC.xD = (rnd() % 2) as u8;
            G_RULES.set_x4_5(rnd() & 1);
            let me = azr(core::mem::size_of::<MatchEnd>()) as *mut MatchEnd;
            (*me).player_standings[0].x44 = rnd() % 4;
            (*me).player_standings[0].stocks = ((rnd() % 6) as i32 - 1) as i8;
            for k in 0..4 {
                (*me)._x448[k] = (rnd() >> 7) as u8;
            }
            t::fn_8018A364(me);
            let b = &(*me)._x448;
            out(&format!("x448 {:02x} {:02x} {:02x} {:02x}\n", b[0], b[1], b[2], b[3]));
            // the C driver keeps the bytes bit-reversed in memory (LSB-first bitfields)
            for k in 0..4 {
                (*me)._x448[k] = (*me)._x448[k].reverse_bits();
            }
            let off448 = core::mem::offset_of!(MatchEnd, _x448);
            let off44c = core::mem::offset_of!(MatchEnd, x44C);
            dump("me0", me as *const u8, off448);
            dump("me1", (me as *const u8).add(off44c), core::mem::size_of::<MatchEnd>() - off44c);
            dump_t("allstar", &t::gm_80473A18);
            dump_arena();
            state_end();
        }
    }
    check("tu_gm_18A1");
}
