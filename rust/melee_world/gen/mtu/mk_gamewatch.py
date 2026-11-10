#!/usr/bin/env python3
"""Writes the C oracle driver and the Rust test of the eight GameWatch item TUs (shard 689-696):

    python rust/melee_world/gen/mtu/mk_gamewatch.py        (from the worktree root)

  rust/melee_world/gen/mtu/tu/itgamewatch<x>.c      C driver (the game's .c included verbatim)
  rust/melee_items/tests/tu_itgamewatch<x>.rs       Rust test, draw for draw the same

then `python rust/melee_world/gen/mtu/run_tu.py itgamewatch<x>` writes tests/data/mtu_itgamewatch<x>.txt.

State oracle. One item (ITEM, user_data of G) and one spawned item (NEWITEM, user_data of NEWG, what
the stubbed Item_80268B18 returns) are filled byte for byte from the seeded generator, then the
fields the TUs read are set (owner / xDD8 chosen among {NULL, OWN, PAR}, anim frame and speed, cmd
vars, a 3-joint chain for the jobj walks). After each call the whole Item is hashed as bytes with
its pointer fields replaced by region ids and the one bitfield byte (xDCC_flag, LSB-first in the C
build, MSB-first in melee_types) masked and compared by value, plus the joints and every callee
trace word (arguments, order, the SpawnItem the TUs build). Each line: op case ret state trace.
"""
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
CDIR = ROOT / "rust/melee_world/gen/mtu/tu"
TDIR = ROOT / "rust/melee_items/tests"

# kinds: g (void f(g)), gi (int f(g)), gg (void f(g, g2)), s4 (spawn parent,pos,part,dir), jd (judge spawn),
#        s6 (rescue spawn), gm (f(g, int))
STEMS = {
    "fire": dict(table=("it_803F78E8", 1), part="u32", ops=[
        ("itGamewatchFire_Spawn", "s4"), ("itGamewatchFire_Destroyed", "g"), ("itGamewatchFire_802C6A2C", "g"),
        ("itGamewatchFire_802C6A78", "g"), ("itGamewatchFire_802C6A98", "g"), ("itGamewatchFire_PickedUp", "g"),
        ("itGamewatchFire_Motion0_Anim", "gi"), ("itGamewatchFire_EvtUnk", "gg")]),
    "greenhouse": dict(table=("it_803F7898", 4), part="u32", ops=[
        ("itGamewatchGreenhouse_Spawn", "s4"), ("itGamewatchGreenhouse_Destroyed", "g"), ("itGamewatchGreenhouse_802C6328", "g"),
        ("itGamewatchGreenhouse_802C6374", "g"), ("itGamewatchGreenhouse_802C6394", "g"), ("itGamewatchGreenhouse_PickedUp", "g"),
        ("itGamewatchGreenhouse_802C6430", "g"), ("itGamewatchGreenhouse_802C6458", "g"), ("itGamewatchGreenhouse_802C6480", "g"),
        ("itGamewatchGreenhouse_802C64A8", "g"), ("itGamewatchGreenhouse_Motion3_Anim", "gi"),
        ("itGamewatchGreenhouse_Motion2_Anim", "gi"), ("itGamewatchGreenhouse_EvtUnk", "gg")]),
    "judge": dict(table=("it_803F7968", 1), part="u32", ops=[
        ("it_802C7774", "jd"), ("it_802C78B8", "g"), ("itGameWatchJudge_Logic77_Destroyed", "g"), ("it_802C7A84", "g"),
        ("it_802C7AD0", "g"), ("it_802C7AF0", "g"), ("it_2725_Logic77_PickedUp", "g"),
        ("itGamewatchjudge_UnkMotion0_Anim", "gi"), ("itGameWatchJudge_Logic77_EvtUnk", "gg")]),
    "manhole": dict(table=("it_803F78D8", 1), part="u32", ops=[
        ("it_802C65E4", "s4"), ("itGameWatchManhole_Logic72_Destroyed", "g"), ("it_802C6718", "g"), ("it_802C6764", "g"),
        ("it_802C6784", "g"), ("itGameWatchManhole_Logic72_PickedUp", "g"), ("itGamewatchmanhole_UnkMotion0_Anim", "gi"),
        ("itGameWatchManhole_Logic72_EvtUnk", "gg")]),
    "panic": dict(table=("it_803F79A0", 2), part="u32", ops=[
        ("it_802C7D60", "s4"), ("itGameWatchPanic_Logic78_Destroyed", "g"), ("it_802C7E94", "g"), ("it_802C7EE0", "g"),
        ("it_802C7F00", "g"), ("itGameWatchPanic_Logic78_PickedUp", "g"), ("itGamewatchpanic_UnkMotion1_Anim", "gi"),
        ("itGameWatchPanic_Logic78_EvtUnk", "gg")]),
    "parachute": dict(table=("it_803F78F8", 2), part="i32", ops=[
        ("it_802C6C38", "s4"), ("itGameWatchParachute_Logic74_Destroyed", "g"), ("it_802C6D6C", "g"), ("it_802C6DB8", "g"),
        ("it_802C6DD8", "g"), ("itGameWatchParachute_Logic74_PickedUp", "g"), ("it_802C6E50", "g"),
        ("itGamewatchparachute_UnkMotion1_Anim", "gi"), ("itGameWatchParachute_Logic74_EvtUnk", "gg")]),
    "rescue": dict(table=("it_803F79C0", 2), part="u32", ops=[
        ("it_802C8038", "s6"), ("it_802C8158", "g"), ("it_802C81C8", "g"), ("it_802C81E8", "g"), ("it_802C8208", "gm"),
        ("itGamewatchrescue_UnkMotion1_Anim", "gi"), ("itGamewatchrescue_UnkMotion1_Phys", "g"),
        ("itGamewatchrescue_UnkMotion1_Coll", "gi"), ("itGameWatchRescue_Logic81_EvtUnk", "gg")]),
    "turtle": dict(table=("it_803F7918", 2), part="u32", ops=[
        ("it_802C6F40", "s4"), ("itGameWatchTurtle_Logic75_Destroyed", "g"), ("it_802C7074", "g"), ("it_802C70C0", "g"),
        ("it_802C70E0", "g"), ("itGameWatchTurtle_Logic75_PickedUp", "g"), ("it_802C7158", "g"),
        ("itGamewatchturtle_UnkMotion1_Anim", "gi"), ("itGameWatchTurtle_Logic75_EvtUnk", "gg")]),
}

C_CALL = {
    "g": "{ {f}(&G); }",
    "gi": "{ ret = (u32) {f}(&G); }",
    "gg": "{ HSD_GObj* o = pickg(); {f}(&G, o); }",
    "gm": "{ int m = (int) fv(); {f}(&G, m); }",
    "s4": "{ HSD_GObj* pa = pickg(); float d = rw(); u32 part = fv(); ret = PI({f}(pa, &POS, part, d)); }",
    "jd": "{ float d = rw(); HSD_GObj* pa = pickg(); int part = (int) fv(); u32 a4 = fv(); ret = PI({f}(d, pa, &POS, part, a4)); }",
    "s6": "{ HSD_GObj* pa = pickg(); int a2 = (int) fv(); int a3 = (int) fv(); float f0 = rw(); float f1 = rw(); "
          "ret = PI({f}(pa, &POS, a2, a3, f0, f1)); }",
}
R_CALL = {
    "g": "m::{f}(&raw mut G);",
    "gi": "ret = m::{f}(&raw mut G) as u32;",
    "gg": "let o = pickg(); m::{f}(&raw mut G, o);",
    "gm": "let mm = fv() as i32; m::{f}(&raw mut G, mm);",
    "s4": "let pa = pickg(); let d = rw(); let part = fv(); ret = pi(m::{f}(pa, &raw mut POS, part as {pt}, d));",
    "jd": "let d = rw(); let pa = pickg(); let part = fv() as i32; let a4 = fv(); ret = pi(m::{f}(d, pa, &raw mut POS, part, a4));",
    "s6": "let pa = pickg(); let a2 = fv() as i32; let a3 = fv() as i32; let f0 = rw(); let f1 = rw(); "
          "ret = pi(m::{f}(pa, &raw mut POS, a2, a3, f0, f1));",
}
NCASE = 2000


def c_driver(stem, d):
    tbl, tn = d["table"]
    ops = d["ops"]
    L = []
    L.append(f"/* TU: src/melee/it/kinds/itgamewatch{stem}.c (written by gen/mtu/mk_gamewatch.py: do not edit) */")
    L.append("#include TU_SRC\n#include \"mtu_common.h\"")
    L.append("""static HSD_GObj G, NEWG, OWN, PAR;
static Item ITEM, NEWITEM;
static HSD_JObj JT[3];
static Article ART;
static void* ATTRS[2];
static int ATTR0[4];
static Vec POS;
#include "itgamewatch%s_gen.h"

static HSD_GObj* pickg(void)
{
    switch (rnd() %% 3) {
    case 0: return 0;
    case 1: return &OWN;
    default: return &PAR;
    }
}
static void fillitem(Item* ip)
{
    unsigned i;
    unsigned char* b = (unsigned char*) ip;
    for (i = 0; i < sizeof *ip; i++) b[i] = (unsigned char) rnd();
}
static void setup(void)
{
    int i;
    float f;
    fillitem(&ITEM);
    fillitem(&NEWITEM);
    *(unsigned char*) &ITEM.xDCC_flag = (rnd() & 1) ? 0xFF : 0;
    *(unsigned char*) &NEWITEM.xDCC_flag = (rnd() & 1) ? 0xFF : 0;
    zero(&G, sizeof G); zero(&NEWG, sizeof NEWG); zero(&OWN, sizeof OWN); zero(&PAR, sizeof PAR);
    zero(&ART, sizeof ART); zero(ATTRS, sizeof ATTRS); zero(ATTR0, sizeof ATTR0);
    for (i = 0; i < 3; i++) {
        zero(&JT[i], sizeof JT[i]);
        JT[i].child = i < 2 ? &JT[i + 1] : 0;
        JT[i].flags = rnd() & ~(u32) JOBJ_USE_QUATERNION;
        JT[i].rotate.x = rw();
        JT[i].rotate.y = rw();
    }
    G.user_data = &ITEM; G.hsd_obj = &JT[0];
    NEWG.user_data = &NEWITEM; NEWG.hsd_obj = &JT[0];
    ATTRS[0] = &ATTR0[0];
    ART.x4_specialAttributes = (union ItemSpecialAttributes*) ATTRS;
    ITEM.xC4_article_data = &ART; NEWITEM.xC4_article_data = &ART;
    ITEM.owner = pickg(); NEWITEM.owner = pickg();
    ITEM.xDD4_itemVar.gamewatchrescue.xDD8 = pickg();
    NEWITEM.xDD4_itemVar.gamewatchrescue.xDD8 = pickg();
    switch (rnd() %% 4) { case 0: f = 3.0f; break; case 1: f = 30.0f; break; case 2: f = rw(); break; default: f = 0.0f; break; }
    ITEM.x5CC_currentAnimFrame = f;
    switch (rnd() %% 4) { case 0: f = 0.0f; break; case 1: f = 1.0f; break; case 2: f = rw(); break; default: f = -0.0f; break; }
    ITEM.x5D0_animFrameSpeed = f;
    ITEM.xDAC_itcmd_var0 = fv(); ITEM.xDB0_itcmd_var1 = fv(); ITEM.xDB4_itcmd_var2 = fv(); ITEM.xDB8_itcmd_var3 = fv();
    NEWITEM.xDAC_itcmd_var0 = fv(); NEWITEM.xDB0_itcmd_var1 = fv(); NEWITEM.xDB4_itcmd_var2 = fv(); NEWITEM.xDB8_itcmd_var3 = fv();
    ITEM.ground_or_air = fv() %% 3;
    POS.x = rw(); POS.y = rw(); POS.z = rw();
}
static void hash_item(const Item* ip)
{
    Item t = *ip;
    t.owner = (HSD_GObj*) (unsigned long long) PI(ip->owner);
    t.xDD4_itemVar.gamewatchrescue.xDD8 = (HSD_GObj*) (unsigned long long) PI(ip->xDD4_itemVar.gamewatchrescue.xDD8);
    t.xC4_article_data = (Article*) (unsigned long long) PI(ip->xC4_article_data);
    HM((u32) ip->xDCC_flag.b3);
    *(unsigned char*) &t.xDCC_flag = 0;
    HM(RH(&t, sizeof t));
}
static void hash_all(void)
{
    int i;
    hash_item(&ITEM); hash_item(&NEWITEM);
    for (i = 0; i < 3; i++) {
        HM(JT[i].flags); HM(B(JT[i].rotate.x)); HM(B(JT[i].rotate.y)); HM(B(JT[i].rotate.z)); HM(B(JT[i].rotate.w));
        HM(PI(JT[i].child));
    }
    HM(B(POS.x)); HM(B(POS.y)); HM(B(POS.z));
}

int main(void)
{
    int op, k, e;
    preg(&G, sizeof G); preg(&NEWG, sizeof NEWG); preg(&OWN, sizeof OWN); preg(&PAR, sizeof PAR);
    preg(&ITEM, sizeof ITEM); preg(&NEWITEM, sizeof NEWITEM); preg(JT, sizeof JT); preg(&ART, sizeof ART);
    preg(ATTRS, sizeof ATTRS); preg(ATTR0, sizeof ATTR0); preg(&POS, sizeof POS);
    freg_all();
    for (op = 0; op < %d; op++) {
        for (k = 0; k < %d; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup();
            switch (op) {""" % (stem, len(ops) + 1, NCASE))
    for i, (f, kind) in enumerate(ops):
        L.append(f"            case {i}: " + C_CALL[kind].replace("{f}", f) + " break;")
    L.append(f"""            case {len(ops)}:
                for (e = 0; e < {tn}; e++) {{
                    HM((u32) {tbl}[e].anim_id); HM(FI((const void*) {tbl}[e].animated));
                    HM(FI((const void*) {tbl}[e].physics_updated)); HM(FI((const void*) {tbl}[e].collided));
                }}
                break;
            }}
            hash_all();
            printf("%d %d %08X %08X %08X\\n", op, k, ret, H, TR);
        }}
    }}
    return 0;
}}""")
    return "\n".join(L) + "\n"


def r_test(stem, d):
    tbl, tn = d["table"]
    ops = d["ops"]
    L = [f"""//! melee_items src/tu/itgamewatch{stem}.rs vs the C (gen/mtu/tu/itgamewatch{stem}.c: the whole TU, shipping
//! contraction). Written by rust/melee_world/gen/mtu/mk_gamewatch.py from the same description as the C
//! driver; regenerate the expected file with `python rust/melee_world/gen/mtu/run_tu.py itgamewatch{stem}`.
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments)]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_items::tu::itgamewatch{stem} as m;
use melee_types::*;
use mtu_harness::*;
#[path = "mtu/itgamewatch{stem}_gen.rs"]
mod g;

static mut G: HSD_GObj = unsafe {{ core::mem::zeroed() }};
static mut NEWG: HSD_GObj = unsafe {{ core::mem::zeroed() }};
static mut OWN: HSD_GObj = unsafe {{ core::mem::zeroed() }};
static mut PAR: HSD_GObj = unsafe {{ core::mem::zeroed() }};
static mut ITEM: Item = unsafe {{ core::mem::zeroed() }};
static mut NEWITEM: Item = unsafe {{ core::mem::zeroed() }};
static mut JT: [HSD_JObj; 3] = unsafe {{ core::mem::zeroed() }};
static mut ART: Article = unsafe {{ core::mem::zeroed() }};
static mut ATTRS: [*mut c_void; 2] = [null_mut(); 2];
static mut ATTR0: [i32; 4] = [0; 4];
static mut POS: Vec = Vec {{ x: 0.0, y: 0.0, z: 0.0 }};

unsafe fn pickg() -> *mut HSD_GObj {{
    match rnd() % 3 {{
        0 => null_mut(),
        1 => &raw mut OWN,
        _ => &raw mut PAR,
    }}
}}

unsafe fn fillitem(ip: *mut Item) {{
    let b = ip as *mut u8;
    for i in 0..core::mem::size_of::<Item>() {{
        *b.add(i) = rnd() as u8;
    }}
}}

unsafe fn setup() {{
    fillitem(&raw mut ITEM);
    fillitem(&raw mut NEWITEM);
    ITEM.xDCC_flag.bf_0 = [if rnd() & 1 != 0 {{ 0xFF }} else {{ 0 }}];
    NEWITEM.xDCC_flag.bf_0 = [if rnd() & 1 != 0 {{ 0xFF }} else {{ 0 }}];
    G = core::mem::zeroed();
    NEWG = core::mem::zeroed();
    OWN = core::mem::zeroed();
    PAR = core::mem::zeroed();
    ART = core::mem::zeroed();
    ATTRS = [null_mut(); 2];
    ATTR0 = [0; 4];
    for i in 0..3usize {{
        JT[i] = core::mem::zeroed();
        JT[i].child = if i < 2 {{ &raw mut JT[i + 1] }} else {{ null_mut() }};
        JT[i].flags = rnd() & !(JOBJ_USE_QUATERNION as u32);
        JT[i].rotate.x = rw();
        JT[i].rotate.y = rw();
    }}
    G.user_data = &raw mut ITEM as *mut c_void;
    G.hsd_obj = &raw mut JT[0] as *mut c_void;
    NEWG.user_data = &raw mut NEWITEM as *mut c_void;
    NEWG.hsd_obj = &raw mut JT[0] as *mut c_void;
    ATTRS[0] = &raw mut ATTR0[0] as *mut c_void;
    ART.x4_specialAttributes = &raw mut ATTRS as *mut ItemSpecialAttributes;
    ITEM.xC4_article_data = &raw mut ART;
    NEWITEM.xC4_article_data = &raw mut ART;
    ITEM.owner = pickg();
    NEWITEM.owner = pickg();
    ITEM.xDD4_itemVar.gamewatchrescue.xDD8 = pickg();
    NEWITEM.xDD4_itemVar.gamewatchrescue.xDD8 = pickg();
    ITEM.x5CC_currentAnimFrame = match rnd() % 4 {{
        0 => 3.0,
        1 => 30.0,
        2 => rw(),
        _ => 0.0,
    }};
    ITEM.x5D0_animFrameSpeed = match rnd() % 4 {{
        0 => 0.0,
        1 => 1.0,
        2 => rw(),
        _ => -0.0,
    }};
    ITEM.xDAC_itcmd_var0 = fv();
    ITEM.xDB0_itcmd_var1 = fv();
    ITEM.xDB4_itcmd_var2 = fv();
    ITEM.xDB8_itcmd_var3 = fv();
    NEWITEM.xDAC_itcmd_var0 = fv();
    NEWITEM.xDB0_itcmd_var1 = fv();
    NEWITEM.xDB4_itcmd_var2 = fv();
    NEWITEM.xDB8_itcmd_var3 = fv();
    ITEM.ground_or_air = fv() % 3;
    POS.x = rw();
    POS.y = rw();
    POS.z = rw();
}}

unsafe fn hash_item(ip: *const Item) {{
    let mut t: Item = core::ptr::read(ip);
    t.owner = pi((*ip).owner) as usize as *mut HSD_GObj;
    t.xDD4_itemVar.gamewatchrescue.xDD8 = pi((*ip).xDD4_itemVar.gamewatchrescue.xDD8) as usize as *mut HSD_GObj;
    t.xC4_article_data = pi((*ip).xC4_article_data) as usize as *mut Article;
    hm((*ip).xDCC_flag.b3() as u32);
    t.xDCC_flag.bf_0 = [0];
    hm(rh(&t));
}}

unsafe fn hash_all() {{
    hash_item(&raw const ITEM);
    hash_item(&raw const NEWITEM);
    for i in 0..3usize {{
        hm(JT[i].flags);
        hm(JT[i].rotate.x.to_bits());
        hm(JT[i].rotate.y.to_bits());
        hm(JT[i].rotate.z.to_bits());
        hm(JT[i].rotate.w.to_bits());
        hm(pi(JT[i].child));
    }}
    hm(POS.x.to_bits());
    hm(POS.y.to_bits());
    hm(POS.z.to_bits());
}}

#[test]
fn itgamewatch{stem}_matches_c() {{
    unsafe {{
        g::install();
        preg(&raw const G, core::mem::size_of::<HSD_GObj>());
        preg(&raw const NEWG, core::mem::size_of::<HSD_GObj>());
        preg(&raw const OWN, core::mem::size_of::<HSD_GObj>());
        preg(&raw const PAR, core::mem::size_of::<HSD_GObj>());
        preg(&raw const ITEM, core::mem::size_of::<Item>());
        preg(&raw const NEWITEM, core::mem::size_of::<Item>());
        preg(&raw const JT, core::mem::size_of_val(&JT));
        preg(&raw const ART, core::mem::size_of::<Article>());
        preg(&raw const ATTRS, core::mem::size_of_val(&ATTRS));
        preg(&raw const ATTR0, core::mem::size_of_val(&ATTR0));
        preg(&raw const POS, core::mem::size_of::<Vec>());
        g::freg_all();
        let mut c = Check::new(include_str!("data/mtu_itgamewatch{stem}.txt"));
        for op in 0..{len(ops) + 1}u32 {{
            for k in 0..{NCASE}u32 {{
                case(op, k);
                setup();
                let mut ret = 0u32;
                match op {{"""]
    for i, (f, kind) in enumerate(ops):
        body = R_CALL[kind].replace("{f}", f).replace("{pt}", d["part"])
        L.append(f"                    {i} => {{ {body} }}")
    L.append(f"""                    _ => {{
                        for e in 0..{tn}usize {{
                            let t = &(*(&raw const m::{tbl}))[e];
                            hm(t.anim_id as u32);
                            hm(fi(t.animated));
                            hm(fi(t.physics_updated));
                            hm(fi(t.collided));
                        }}
                    }}
                }}
                hash_all();
                c.line(op, k, ret);
            }}
        }}
        assert_eq!(c.done(), {(len(ops) + 1) * NCASE});
    }}
}}""")
    return "\n".join(L) + "\n"


for stem, d in STEMS.items():
    (CDIR / f"itgamewatch{stem}.c").write_text(c_driver(stem, d), encoding="utf-8", newline="\n")
    (TDIR / f"tu_itgamewatch{stem}.rs").write_text(r_test(stem, d), encoding="utf-8", newline="\n")
print("ok")
