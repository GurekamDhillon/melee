#!/usr/bin/env python3
"""Writes the two halves of a per-TU state-oracle driver from one description, so the C driver
(gen/mtu/tu/<stem>.c) and the Rust test (tests/tu_<stem>.rs) cannot drift apart:

  python rust/melee_world/gen/mtu/mk_stage_driver.py <stem> [...]      (worktree root)

Each description (CONFIGS below) gives the statics both sides declare, how a case's synthetic
state is built (`setup`, same draws on both sides), what is hashed afterwards (`hash`), and one
(C statement, Rust statement) pair per operation. The statements may assign `ret` (u32), which is
the value printed / compared per case. Then run `run_tu.py <stem>` and `cargo test -p melee_world
--test tu_<stem>`.
"""
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
N_CASES = 500


def gobj_fn(n):
    return (n, f"{n}(&SGO);", f"m::{n}(&raw mut SGO);")


def pred_fn(n):
    return (n, f"ret = (u32) {n}(&SGO);", f"ret = m::{n}(&raw mut SGO) as u32;")


def void_fn(n):
    return (n, f"{n}();", f"m::{n}();")


def pred0_fn(n):
    return (n, f"ret = (u32) {n}();", f"ret = m::{n}() as u32;")


# ---------------------------------------------------------------- the stage-gobj fixture
STD_C_DECL = """static HSD_GObj SGO;
static Ground SGP;
static char SJO[16];
static char YPB[64];
static Vec3 SV;
"""
STD_RS_DECL = """static mut SGO: HSD_GObj = unsafe { core::mem::zeroed() };
static mut SGP: Ground = unsafe { core::mem::zeroed() };
static mut SJO: [u8; 16] = [0; 16];
static mut YPB: [u8; 64] = [0; 64];
static mut SV: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
"""
STD_PREG = [("&SGO", "sizeof SGO", "&raw const SGO", "core::mem::size_of::<HSD_GObj>()"),
            ("&SGP", "sizeof SGP", "&raw const SGP", "core::mem::size_of::<Ground>()"),
            ("SJO", "sizeof SJO", "&raw const SJO", "16"),
            ("YPB", "sizeof YPB", "&raw const YPB", "64"),
            ("&SV", "sizeof SV", "&raw const SV", "core::mem::size_of::<Vec3>()")]
STD_SETUP_C = """    zero(&SGO, sizeof SGO); zero(&SGP, sizeof SGP);
    fill_HSD_GObj(&SGO); fill_Ground(&SGP); fill_StageInfo(&stage_info);
    SGO.user_data = &SGP;
    SGO.hsd_obj = SJO;
    SV.x = rw(); SV.y = rw(); SV.z = rw();
"""
STD_SETUP_RS = """    SGO = core::mem::zeroed();
    SGP = core::mem::zeroed();
    g::fill_HSD_GObj(&mut SGO);
    g::fill_Ground(&mut SGP);
    g::fill_StageInfo(&mut m::ext::stage_info);
    SGO.user_data = &raw mut SGP as *mut _;
    SGO.hsd_obj = &raw mut SJO as *mut _;
    SV.x = rw();
    SV.y = rw();
    SV.z = rw();
"""
STD_HASH_C = """    hash_HSD_GObj(&SGO); hash_Ground(&SGP); hash_StageInfo(&stage_info);
    HM(B(SV.x)); HM(B(SV.y)); HM(B(SV.z));
"""
STD_HASH_RS = """    g::hash_HSD_GObj(&SGO);
    g::hash_Ground(&SGP);
    g::hash_StageInfo(&m::ext::stage_info);
    hm(SV.x.to_bits());
    hm(SV.y.to_bits());
    hm(SV.z.to_bits());
"""


def table_op(callbacks, n, stagedata, joints=None):
    c = (f"{{ int i; for (i = 0; i < {n}; i++) hash_StageCallbacks(&{callbacks}[i]); hash_StageData(&{stagedata}); "
         f"HM(SH({stagedata}.data1)); HM((u32) {stagedata}.joint_count); }}")
    r = (f"{{ for i in 0..{n} {{ g::hash_StageCallbacks(&m::{callbacks}[i]); }} g::hash_StageData(&m::{stagedata}); "
         f"hm(sh(m::{stagedata}.data1 as *const u8)); hm(m::{stagedata}.joint_count as u32); }}")
    return ("tables", c, r)


def std_stage_ops(prefix_ops):
    return prefix_ops


# Standard fixture parts for the stage TUs; a config supplies ops and any extras.
def std(stem, ops, callbacks, ncb, stagedata, extra=None):
    cfg = dict(
        stem=stem, c_decl=STD_C_DECL, rs_decl=STD_RS_DECL, pregs=list(STD_PREG) + [
            (callbacks, f"sizeof {callbacks}", f"&raw const m::{callbacks}", f"core::mem::size_of_val(&m::{callbacks})"),
            (f"&{stagedata}", f"sizeof {stagedata}", f"&raw const m::{stagedata}", f"core::mem::size_of_val(&m::{stagedata})")],
        c_extra="", rs_extra="", setup_c=STD_SETUP_C, setup_rs=STD_SETUP_RS,
        hash_c=STD_HASH_C, hash_rs=STD_HASH_RS, ops=list(ops) + [table_op(callbacks, ncb, stagedata)],
        extra_freg=[])
    if extra:
        cfg.update(extra)
    return cfg


def figure_ops(stem, pfx, ids):
    """ids: dict role -> function suffix for the three figure TUs (grfigure1/2/3)."""
    P = lambda s: f"{pfx}_{s}"
    ops = [
        ("demoinit", f"{ids['demoinit']}((int) fv());", f"m::{ids['demoinit']}(fv() as i32);"),
        void_fn(ids["init"]), void_fn(ids["load"]), void_fn(ids["start"]),
        pred0_fn(ids["pred4"]),
        ("setup_gobj", f"ret = PI({ids['setup']}((int) (rnd() % 3)));", f"ret = pi(m::{ids['setup']}((rnd() % 3) as i32));"),
        gobj_fn("stageGObj0_OnInit"),
        pred_fn(ids["p0"]), gobj_fn(ids["g0b"]), gobj_fn(ids["g0c"]),
        gobj_fn(ids["g1a"]), pred_fn(ids["p1"]), gobj_fn(ids["g1b"]), gobj_fn(ids["g1c"]),
        gobj_fn("stageGObj2_OnInit"),
        pred_fn(ids["p2"]), gobj_fn(ids["g2b"]), gobj_fn(ids["g2c"]),
        ("touchline", f"ret = PI({ids['touch']}((enum_t) fv()));", f"ret = pi(m::{ids['touch']}(fv() as _));"),
        ("shadow", f"ret = (u32) {ids['shadow']}(&SV, (int) fv(), (HSD_JObj*) SJO);",
         f"ret = m::{ids['shadow']}(&raw mut SV, fv() as i32, &raw mut SJO as *mut _) as u32;"),
    ]
    return ops


F1 = dict(demoinit="grFigure1_OnDemoInit", init="grFigure1_OnInit", load="grFigure1_OnLoad", start="grFigure1_OnStart",
          pred4="grFigure1_8020E078", setup="grFigure1_8020E080", p0="grFigure1_8020E194", g0b="grFigure1_8020E19C",
          g0c="grFigure1_8020E1A0", g1a="grFigure1_8020E1A4", p1="grFigure1_8020E1D0", g1b="grFigure1_8020E1D8",
          g1c="grFigure1_8020E1FC", p2="grFigure1_8020E250", g2b="grFigure1_8020E258", g2c="grFigure1_8020E25C",
          touch="grFigure1_OnTouchLine", shadow="grFigure1_OnCheckShadowRender")
F2 = dict(demoinit="grFigure1_8020E270", init="grFigure2_8020E274", load="grFigure2_OnLoad", start="grFigure2_OnStart",
          pred4="grFigure2_8020E30C", setup="grFigure2_8020E314", p0="grFigure2_8020E428", g0b="grFigure2_8020E430",
          g0c="grFigure2_8020E434", g1a="grFigure2_8020E438", p1="grFigure2_8020E464", g1b="grFigure2_8020E46C",
          g1c="grFigure2_8020E490", p2="grFigure2_8020E4E4", g2b="grFigure2_8020E4EC", g2c="grFigure2_8020E4F0",
          touch="grFigure2_8020E4F4", shadow="grFigure2_8020E4FC")
F3 = dict(demoinit="grFigure3_8020E504", init="grFigure3_8020E508", load="grFigure3_OnLoad", start="grFigure3_OnStart",
          pred4="grFigure3_8020E5A0", setup="grFigure3_8020E5A8", p0="grFigure3_8020E6BC", g0b="grFigure3_8020E6C4",
          g0c="grFigure3_8020E6C8", g1a="grFigure3_8020E6CC", p1="grFigure3_8020E6F8", g1b="grFigure3_8020E700",
          g1c="grFigure3_8020E724", p2="grFigure3_8020E778", g2b="grFigure3_8020E780", g2c="grFigure3_8020E784",
          touch="grFigure3_8020E788", shadow="grFigure3_8020E790")

CONFIGS = {}


def fig_cfg(stem, ids, callbacks, stagedata):
    ops = figure_ops(stem, None, ids)
    if stem == "grfigure1":
        ops = [o if o[0] != "demoinit" else ("demoinit", "grFigure1_OnDemoInit((int) fv());", "m::grFigure1_OnDemoInit(fv() as i32);")
               for o in ops]
        ops.append(("fig1_270", "grFigure1_8020E270((int) fv());", "m::grFigure1_8020E270(fv() as i32);"))
    if stem == "grfigure2":  # its demo-init is grfigure1's function (a stub in this TU's C oracle)
        ops = [o for o in ops if o[0] != "demoinit"]
    cfg = std(stem, ops, callbacks, 3, stagedata)
    # yakumono_param is overwritten by OnInit: start from a random value
    cfg["setup_c"] += "    yakumono_param = (rnd() & 1) ? (void*) YPB : 0;\n"
    cfg["setup_rs"] += "    m::yakumono_param = if rnd() & 1 != 0 { &raw mut YPB as *mut _ } else { null_mut() };\n"
    cfg["hash_c"] += "    HM(PI(yakumono_param));\n"
    cfg["hash_rs"] += "    hm(pi(m::yakumono_param));\n"
    return cfg


CONFIGS["grfigure1"] = fig_cfg("grfigure1", F1, "grEF1_StageCallbacks", "grEF1_StageData")
CONFIGS["grfigure2"] = fig_cfg("grfigure2", F2, "grEF2_StageCallbacks", "grEF2_StageData")
CONFIGS["grfigure3"] = fig_cfg("grfigure3", F3, "grEF2_StageCallbacks", "grEF3_StageData")

# ---------------------------------------------------------------- emit
def emit(stem):
    cfg = CONFIGS[stem]
    ops = cfg["ops"]
    nops = len(ops)
    n = cfg.get("n_cases", N_CASES)
    c = [f"/* TU: {cfg.get('src', 'src/melee/gr/' + stem + '.c')} */",
         "/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a",
         " * recording stub. Written by gen/mtu/mk_stage_driver.py. Each line: op case ret state-hash trace-hash. */",
         "#include TU_SRC", '#include "mtu_common.h"', cfg["c_decl"], f'#include "{stem}_gen.h"', "", cfg["c_extra"], "",
         "static void setup(void)", "{", "    int i, n;", "    (void) i; (void) n;", cfg["setup_c"], "}", "",
         "static void hash_all(void)", "{", "    int i;", "    (void) i;", cfg["hash_c"], "}", "",
         "int main(void)", "{", "    int op, k;"]
    for ca, cs, _, _ in cfg["pregs"]:
        c.append(f"    preg({ca}, {cs});")
    c.append("    freg_all();")
    for ce, _ in cfg["extra_freg"]:
        c.append(f"    freg((const void*) {ce});")
    c += [f"    for (op = 0; op < {nops}; op++) {{", f"        for (k = 0; k < {n}; k++) {{", "            u32 ret = 0;",
          "            tu_case(op, k);", "            setup();", "            switch (op) {"]
    for i, (_, cs, _) in enumerate(ops):
        c.append(f"            case {i}: {{ {cs} }} break;")
    c += ["            }", "            hash_all();", '            printf("%d %d %08X %08X %08X\\n", op, k, ret, H, TR);',
          "        }", "    }", "    return 0;", "}", ""]
    (HERE / "tu" / f"{stem}.c").write_text("\n".join(c), encoding="utf-8", newline="\n")

    r = ["//! Written by gen/mtu/mk_stage_driver.py from the same description as gen/mtu/tu/" + stem + ".c.",
         f"//! src/tu/{stem}.rs vs the C (the whole TU, shipping contraction; regenerate the expected file",
         f"//! with `python rust/melee_world/gen/mtu/run_tu.py {stem}`).",
         "#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables)]",
         "mod mtu_harness;", "use core::ptr::null_mut;", "use melee_types::*;", f"use melee_world::tu::{stem} as m;",
         "use mtu_harness::*;", f'#[path = "mtu/{stem}_gen.rs"]', "mod g;", "", cfg["rs_decl"], "", cfg["rs_extra"], "",
         "unsafe fn setup() {", "    let mut n: u32;", cfg["setup_rs"], "}", "", "unsafe fn hash_all() {", cfg["hash_rs"], "}", "",
         "#[test]", f"fn {stem}_matches_c() {{", "    unsafe {", "        g::install();"]
    for _, _, ra, rs in cfg["pregs"]:
        r.append(f"        preg({ra}, {rs});")
    r.append("        g::freg_all();")
    for _, re_ in cfg["extra_freg"]:
        r.append(f"        freg({re_} as *const () as usize);")
    r += [f'        let mut c = Check::new(include_str!("data/mtu_{stem}.txt"));', f"        for op in 0..{nops}u32 {{",
          f"            for k in 0..{n}u32 {{", "                case(op, k);", "                setup();",
          "                let mut ret = 0u32;", "                match op {"]
    for i, (_, _, rs) in enumerate(ops):
        r.append(f"                    {i} => {{ {rs} }}")
    r += ["                    _ => unreachable!(),", "                }", "                hash_all();", "                c.line(op, k, ret);",
          "            }", "        }", f"        assert_eq!(c.done(), {nops * n});", "    }", "}", ""]
    (ROOT / "rust/melee_world/tests" / f"tu_{stem}.rs").write_text("\n".join(r), encoding="utf-8", newline="\n")
    print(f"mk_stage_driver: {stem}: {nops} ops x {n} cases")


if __name__ == "__main__":
    sys.path.insert(0, str(HERE))
    import mk_stage_cfgs
    CONFIGS.update(mk_stage_cfgs.CONFIGS)
    for s in sys.argv[1:]:
        emit(s)
