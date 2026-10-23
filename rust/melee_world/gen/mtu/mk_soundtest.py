#!/usr/bin/env python3
"""Writes the soundtest oracle driver pair from one description (like mk_itanimlist.py):

    python rust/melee_world/gen/mtu/mk_soundtest.py      (from the worktree root; run mk_soundtest_data.py first)

  rust/melee_world/gen/mtu/tu/soundtest.c    C driver (the game's soundtest.c included verbatim)
  rust/melee_world/tests/tu_soundtest.rs     Rust test, draw for draw the same

Ops: 0 = the data tables (every object of the C's .data / .sdata hashed in its initial state, pointers by
region id, function pointers by table id); 1.. = each function of the TU on seeded states of the TU's
statics (settings block, the loaded symbol table, the save-data window, the objects the stubs hand out).
Each line: op case ret state-hash trace-hash. Returns of the callbacks that fall off the end of the C
(undefined) are not compared.
"""
import json, re
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
SRC = (ROOT / "src/melee/if/soundtest.c").read_text(encoding="utf-8")
META = json.loads((HERE / "tu/soundtest_data.json").read_text(encoding="utf-8"))

CBS = re.findall(r"^bool (\w+)\(enum soundtest_callback_arg0", SRC, re.M)
# the C returns nothing on these paths (undefined value): not compared
NORET = {"un_802FF934", "un_80300758", "un_80300790", "fn_80300CC8", "fn_80300DE0", "fn_80300ED0", "fn_803011EC",
         "un_80301CE0"}
KCB = 250

# --- statics of the TU the ops start from: (name, C expr, Rust expr, kind) ; kind: i = int-like, f = float, p = pointer
S = lambda n, e, k="i": (n, e, k)
STATICS = [
    S("un_804D5850", "fv()"), S("un_804D5854", "fv()"), S("un_804D5858", "fv()"), S("un_804D585C", "fv()"),
    S("un_804D5908", "fv()"), S("un_804D590C", "fv()"), S("un_804D5910", "fv()"),
    S("un_804D6DAC", "fv()"), S("un_804D6DB0", "fv() % 56"), S("un_804D6DB4", "fv() % 200"), S("un_804D6DB8", "fv()"),
    S("un_804D6DBC", "fv()"), S("un_804D6DC0", "fv()"), S("un_804D6DC8", "fv()"), S("un_804D6DCC", "fv()"),
    S("un_804D6DD0", "(fv() % 5000) / 4.0f", "f"), S("un_804D6DD4", "fv()"), S("un_804D6DD8", "fv() % 5"),
    S("un_804D6DDC", "fv() % 1000"), S("un_804D6DE0", "fv()"), S("un_804D6DE4", "fv()"), S("un_804D6DE8", "fv()"),
    S("un_804D6DEC", "fv()"), S("un_804D6DF0", "fv()"), S("un_804D6DF4", "fv()"), S("un_804D6DF8", "fv()"),
    S("un_804D6DFC", "fv()"), S("un_804D6E00", "fv() % 4"), S("un_804D6E0C", "fv()"), S("un_804D6E10", "fv()"),
    S("un_804D6E14", "fv()"),
]
# (name, C pointer expr, Rust pointer expr)
PSTAT = [
    ("un_804D6E04", "((rnd() & 1) ? (void*) MEM : 0)", "if rnd() & 1 != 0 { &raw mut MEM as *mut c_void } else { null_mut() }"),
    ("un_804D6E08", "((rnd() & 1) ? &GO[rnd() % 4] : 0)", "if rnd() & 1 != 0 { &raw mut GO[(rnd() % 4) as usize] } else { null_mut() }"),
]
# un_803FA258 fields: (name, kind, count)  kind: i int, f float, u16, u8
F258 = ([("x0", "i", 1), ("x4", "i", 4), ("x14", "i", 4), ("x24", "i", 4), ("x34", "i", 1), ("x38", "i", 4),
         ("x48", "i", 4), ("x58", "i", 4), ("x68", "i", 4), ("x78", "f", 4), ("x88", "f", 4), ("x98", "f", 4),
         ("xA8", "i", 4), ("xB8", "i", 4), ("xC8", "i", 1), ("xCC", "i", 4), ("xDC", "i", 4), ("xEC", "f", 1),
         ("xF0", "u16", 1), ("xF4", "u8", 1), ("xF5", "u8", 1), ("xF6", "u8", 1), ("xF7", "u8", 1)]
        + [(f"x{h:X}", "i", 1) for h in range(0xF8, 0x138, 4)]
        + [("x138", "p", 1), ("x13C", "i", 1), ("x140", "i", 1), ("x144", "i", 17), ("x188", "i", 1), ("x18C", "i", 1)])
WIN = 0x1A00  # save-data window the ops touch (xF0 < 0x100, +0x186B)

CHARS_EXT = ["DbLevel", "db_804D4AF8", "db_804D6B88", "db_build_timestamp", "gmMainLib_8045A6C0"]


def f258_fill(lang):
    L = []
    for n, k, c in F258:
        for i in range(c):
            ix = f"[{i}]" if c > 1 else ""
            if lang == "c":
                tgt = f"un_803FA258.{n}{ix}"
                if n == "xF0":
                    L.append(f"{tgt} = (u16) (fv() & 0xFF);")
                elif n == "x138":
                    L.append(f"{tgt} = ((rnd() & 1) ? (void*) MEM : 0);")
                elif k == "i":
                    L.append(f"{tgt} = (int) fv();")
                elif k == "f":
                    L.append(f"{tgt} = rg();")
                else:
                    L.append(f"{tgt} = (u8) rnd();")
            else:
                tgt = f"m::un_803FA258.{n}{ix}"
                if n == "xF0":
                    L.append(f"{tgt} = (fv() & 0xFF) as u16;")
                elif n == "x138":
                    L.append(f"{tgt} = if rnd() & 1 != 0 {{ &raw mut MEM as *mut c_void }} else {{ null_mut() }};")
                elif k == "i":
                    L.append(f"{tgt} = fv() as i32;")
                elif k == "f":
                    L.append(f"{tgt} = rg();")
                else:
                    L.append(f"{tgt} = rnd() as u8;")
    return "\n    ".join(L)


def f258_hash(lang):
    L = []
    for n, k, c in F258:
        for i in range(c):
            ix = f"[{i}]" if c > 1 else ""
            if lang == "c":
                tgt = f"un_803FA258.{n}{ix}"
                L.append(f"HM(PI({tgt}));" if k == "p" else f"HM(B({tgt}));" if k == "f" else f"HM((u32) {tgt});")
            else:
                tgt = f"m::un_803FA258.{n}{ix}"
                L.append(f"hm(pi({tgt}));" if k == "p" else f"hm({tgt}.to_bits());" if k == "f" else f"hm({tgt} as u32);")
    return "\n    ".join(L)


def tables_hash(lang):
    L = []
    for e in META:
        n, k = e["name"], e["kind"]
        ct = e["ctype"]
        if lang == "c":
            ref = n
            if k == "chars" or k == "ints":
                L.append(f"HM(RH({ref}, sizeof {ref}));")
            elif k == "ptrs":
                L.append(f"for (i = 0; i < {e['n']}; i++) HM(PI({ref}[i]));")
            elif k == "rows":
                L.append(f"for (i = 0; i < {e['n']}; i++) hash_un_80304138_objalloc_t_x8(&{ref}[i]);")
            elif k == "scalar":
                if ct in ("int", "s32", "enum_t"):
                    L.append(f"HM((u32) {ref});")
                elif ct == "f32":
                    L.append(f"HM(B({ref}));")
                else:
                    L.append(f"HM(PI({ref}));")
            elif n == "un_803F9FA4":
                L.append(f"for (i = 0; i < 11; i++) hash_un_80304138_objalloc_t_x8(&{ref}.entries[i]); HM(RH({ref}.x160, 12)); HM(RH({ref}.x16C, 24));")
            elif n == "un_803FA258":
                L.append("hash_258();")
            elif n == "un_804D5990":
                L.append(f"HM(PI({ref}.image)); HM(PI({ref}.tlut));")
            elif n == "un_803FD274":
                L.append(f"HM(PI({ref}.image_ptr)); HM({ref}.width); HM({ref}.height); HM((u32) {ref}.format); HM({ref}.mipmap); HM(B({ref}.minLOD)); HM(B({ref}.maxLOD));")
            else:
                raise SystemExit(f"table hash: {n} {k} {ct}")
        else:
            ref = f"(*(&raw const m::{n}))"
            if k == "chars" or k == "ints":
                L.append(f"hm(rh(&{ref}));")
            elif k == "ptrs":
                L.append(f"for i in 0..{e['n']} {{ hm(pi({ref}[i])); }}")
            elif k == "rows":
                L.append(f"for i in 0..{e['n']} {{ g::hash_un_80304138_objalloc_t_x8(&{ref}[i]); }}")
            elif k == "scalar":
                if ct in ("int", "s32", "enum_t"):
                    L.append(f"hm({ref} as u32);")
                elif ct == "f32":
                    L.append(f"hm({ref}.to_bits());")
                else:
                    L.append(f"hm(pi({ref}));")
            elif n == "un_803F9FA4":
                L.append(f"for i in 0..11 {{ g::hash_un_80304138_objalloc_t_x8(&{ref}.entries[i]); }} hm(rh(&{ref}.x160)); hm(rh(&{ref}.x16C));")
            elif n == "un_803FA258":
                L.append("hash_258();")
            elif n == "un_804D5990":
                L.append(f"hm(pi({ref}.image)); hm(pi({ref}.tlut));")
            elif n == "un_803FD274":
                L.append(f"hm(pi({ref}.image_ptr)); hm({ref}.width as u32); hm({ref}.height as u32); hm({ref}.format as u32); hm({ref}.mipmap); hm({ref}.minLOD.to_bits()); hm({ref}.maxLOD.to_bits());")
            else:
                raise SystemExit(f"table hash: {n} {k} {ct}")
    return "\n    ".join(L)


def regs(lang):
    L = []
    for e in META:
        n = e["name"]
        if lang == "c":
            L.append(f"preg(&{n}, sizeof {n});")
        else:
            L.append(f"preg(&raw const m::{n}, core::mem::size_of_val(&*(&raw const m::{n})));")
    for n in CHARS_EXT:
        if lang == "c":
            L.append(f"preg(&{n}, sizeof {n});")
        else:
            L.append(f"preg(&raw const m::ext::{n}, core::mem::size_of_val(&*(&raw const m::ext::{n})));")
    return "\n    ".join(L)


OPS = [("tables", None)]  # (label, kind)
OTHER = ["un_802FF7DC", "un_802FF884", "un_802FFCD0", "un_802FFD94", "un_802FFEE0", "un_802FFF2C", "un_80301BA8", "un_80301C64"]
ALLOPS = ["tables"] + OTHER + CBS
KOF = {"tables": 1, "un_802FF7DC": 300, "un_802FF884": 50, "un_802FFCD0": 300, "un_802FFD94": 400, "un_802FFEE0": 300,
       "un_802FFF2C": 1500, "un_80301BA8": 300, "un_80301C64": 100}
for c in CBS:
    KOF[c] = KCB

CALLBACK_FNS = ["fn_802FFE6C", "fn_802FFE0C", "un_80301B48", "fn_80300CC8", "fn_80300DE0", "fn_80300ED0", "fn_803011EC"]


def c_op(name):
    if name == "tables":
        return "hash_tables();"
    if name == "un_802FF7DC":
        return "un_802FF7DC();"
    if name == "un_802FF884":
        return "ret = (u32) un_802FF884((rnd() & 1) ? STR : 0);"
    if name == "un_802FFCD0":
        return "{ int cnt = (int) (rnd() % 9); un_802FFCD0(cnt, ARR); }"
    if name == "un_802FFD94":
        return ("{ int a = (int) (rnd() % 4); void* t = (rnd() & 1) ? (void*) ARR : 0; u32 w = rnd() % 8; "
                "soundtest_callback cb = w == 0 ? 0 : w == 1 ? fn_802FFE6C : w == 2 ? fn_802FFE0C : w == 3 ? un_80301B48 : w == 4 ? fn_80300CC8 : w == 5 ? fn_80300DE0 : w == 6 ? fn_80300ED0 : fn_803011EC; "
                "un_802FFD94(a, t, cb); }")
    if name == "un_802FFEE0":
        return "un_802FFEE0(&O0);"
    if name == "un_802FFF2C":
        return "un_802FFF2C(&SMD);"
    if name == "un_80301BA8":
        return "un_80301BA8(&O1);"
    if name == "un_80301C64":
        return "un_80301C64(&O2);"
    r = "ret = (u32) " if name not in NORET else ""
    return f"{{ u32 a = pick(); {r}{name}(a); }}"


def r_op(name):
    if name == "tables":
        return "hash_tables();"
    if name == "un_802FF7DC":
        return "m::un_802FF7DC();"
    if name == "un_802FF884":
        return "{ let p = if rnd() & 1 != 0 { &raw mut STR as *mut i8 } else { null_mut() }; ret = m::un_802FF884(p) as u32; }"
    if name == "un_802FFCD0":
        return "{ let cnt = (rnd() % 9) as i32; m::un_802FFCD0(cnt, &raw mut ARR as *mut c_void); }"
    if name == "un_802FFD94":
        return ("{ let a = (rnd() % 4) as i32; let t = if rnd() & 1 != 0 { &raw mut ARR as *mut c_void } else { null_mut() }; let w = rnd() % 8; "
                "let cb: soundtest_callback = match w { 0 => None, 1 => Some(m::fn_802FFE6C), 2 => Some(m::fn_802FFE0C), 3 => Some(m::un_80301B48), "
                "4 => Some(m::fn_80300CC8), 5 => Some(m::fn_80300DE0), 6 => Some(m::fn_80300ED0), _ => Some(m::fn_803011EC) }; "
                "m::un_802FFD94(a, t, cb); }")
    if name == "un_802FFEE0":
        return "m::un_802FFEE0(&raw mut O0);"
    if name == "un_802FFF2C":
        return "m::un_802FFF2C(&raw mut SMD);"
    if name == "un_80301BA8":
        return "m::un_80301BA8(&raw mut O1);"
    if name == "un_80301C64":
        return "m::un_80301C64(&raw mut O2);"
    r = "ret = " if name not in NORET else ""
    tail = " as u32" if name not in NORET else ""
    return f"{{ let a = pick(); {r}m::{name}(a){tail}; }}" if name not in NORET else f"{{ let a = pick(); m::{name}(a); }}"


def main():
    # ---------------------------------------------------------------- C
    st_fill_c = []
    st_fill_r = []
    for n, e, k in STATICS:
        if k == "f":
            st_fill_c.append(f"{n} = {e};")
            st_fill_r.append(f"m::{n} = ((fv() % 5000) as f32) / 4.0;")
        else:
            st_fill_c.append(f"{n} = (int) ({e});")
            st_fill_r.append(f"m::{n} = ({e}) as _;")
    for n, ce, re_ in PSTAT:
        st_fill_c.append(f"{n} = {ce};")
        st_fill_r.append(f"m::{n} = {re_};")
    st_hash_c = []
    st_hash_r = []
    for n, e, k in STATICS:
        if k == "f":
            st_hash_c.append(f"HM(B({n}));")
            st_hash_r.append(f"hm(m::{n}.to_bits());")
        else:
            st_hash_c.append(f"HM((u32) {n});")
            st_hash_r.append(f"hm(m::{n} as u32);")
    for n, _, _ in PSTAT:
        st_hash_c.append(f"HM(PI({n}));")
        st_hash_r.append(f"hm(pi(m::{n}));")
    st_hash_c.append("HM(PI(un_804D6DA8));")
    st_hash_r.append("hm(pi(m::un_804D6DA8));")

    ops_c = []
    ops_r = []
    for i, name in enumerate(ALLOPS):
        ops_c.append(f"    case {i}: {c_op(name)} break;")
        ops_r.append(f"                    {i} => {{ {r_op(name)} }}")
    kcases = ", ".join(str(KOF[n]) for n in ALLOPS)
    n_ops = len(ALLOPS)
    fnames = ", ".join(f'"{n}"' for n in ALLOPS)

    c = f'''/* TU: src/melee/if/soundtest.c (written by gen/mtu/mk_soundtest.py: do not edit) */
/* C oracle for src/tu/soundtest.rs: the whole TU included verbatim; every callee out of it is a recording
 * stub. Op 0 hashes the data tables; the others run one function each on seeded states of the TU's statics.
 * Each line: op case ret state-hash trace-hash. Ops: {fnames} */
#include TU_SRC
#include "mtu_common.h"
static un_80304138_objalloc_t OA;
static DevText DT;
static HSD_GObj GO[4];
static HSD_SObj SOB;
static unsigned char MEM[64];
static char STR[8] = "abc";
static u32 U32S[5];
static int I32S[2];
static struct gmm_x0_528_t GMM[3];
static struct UnkAllstarData ALLS;
static unsigned long long SYMBUF[16];
static int A5[256], A6[64], ARR[16];
static struct UnkSoundTestData0 O0;
static struct UnkSoundTestData1 O1;
static un_80301C64_t O2;
static StartMeleeData SMD;
static struct SoundTestMenuData PRISTINE;
static HSD_ImageDesc PRISTINE_IMG;
#include "soundtest_gen.h"

static void hash_258(void)
{{
    {f258_hash("c")}
}}

static void hash_tables(void)
{{
    int i;
    {tables_hash("c")}
}}

static u32 pick(void)
{{
    u32 r = rnd();
    if ((r & 7) == 7) return fv() & 15;
    switch ((r >> 3) % 3) {{ case 0: return 0; case 1: return 1; default: return 6; }}
}}

static void setup(void)
{{
    int i;
    un_803F9FA4 = PRISTINE;
    un_803FD274 = PRISTINE_IMG;
    {f258_fill("c")}
    {chr(10).join("    " + x for x in st_fill_c).lstrip()}
    un_804D6DA8 = (int*) SYMBUF;
    for (i = 0; i < 4; i++) SYMBUF[i] = fv64();
    SYMBUF[4] = fv64(); SYMBUF[7] = fv64();
    SYMBUF[5] = (unsigned long long) A5; SYMBUF[6] = (unsigned long long) A6;
    for (i = 0; i < 256; i++) A5[i] = (int) fv();
    for (i = 0; i < 64; i++) A6[i] = (int) (rnd() % 4);
    for (i = 0; i < 16; i++) ARR[i] = (int) fv();
    zero(&OA, sizeof OA); zero(&DT, sizeof DT); zero(GO, sizeof GO); zero(&SOB, sizeof SOB);
    fill_un_80304138_objalloc_t(&OA);
    OA.x4 = &DT;
    DT.scale_x = rw(); DT.scale_y = rw();
    for (i = 0; i < 4; i++) fill_HSD_GObj(&GO[i]);
    SOB.x10 = rw(); SOB.x14 = rw(); SOB.x40 = fv();
    for (i = 0; i < 5; i++) U32S[i] = fv();
    for (i = 0; i < 2; i++) I32S[i] = (int) fv();
    for (i = 0; i < 3; i++) {{ GMM[i].c_kind = (s8) rnd(); GMM[i].stocks = (u8) rnd(); GMM[i].color = (u8) rnd(); GMM[i].cpu_level = (u8) rnd(); GMM[i].nametag = (u8) rnd(); GMM[i].x5 = (u8) rnd(); }}
    ALLS.x0.x0.ckind = (s8) rnd(); ALLS.x0.x0.color = (u8) rnd();
    for (i = 0; i < {WIN}; i++) gmMainLib_8045A6C0[i] = (unsigned char) rnd();
    O0.x0 = (u16) rnd(); O0.x2 = (u16) rnd(); O0.x4 = (int) fv(); O0.x8 = (int) fv();
    for (i = 0; i < 32; i++) ((unsigned char*) &O1)[i] = (unsigned char) rnd();
    O2.x0 = (rnd() & 1) ? (void*) MEM : 0; O2.x4 = (int) fv();
    fill_StartMeleeData(&SMD);
}}

static void hash_all(void)
{{
    int i;
    {chr(10).join("    " + x for x in st_hash_c).lstrip()}
    hash_258();
    for (i = 0; i < 11; i++) hash_un_80304138_objalloc_t_x8(&un_803F9FA4.entries[i]);
    HM(PI(un_803FD274.image_ptr));
    hash_un_80304138_objalloc_t(&OA);
    HM(B(DT.scale_x)); HM(B(DT.scale_y));
    for (i = 0; i < 4; i++) hash_HSD_GObj(&GO[i]);
    HM(B(SOB.x10)); HM(B(SOB.x14)); HM(SOB.x40);
    for (i = 0; i < 5; i++) HM(U32S[i]);
    for (i = 0; i < 2; i++) HM((u32) I32S[i]);
    for (i = 0; i < 3; i++) {{ HM((u32) (int) GMM[i].c_kind); HM(GMM[i].stocks); HM(GMM[i].color); HM(GMM[i].cpu_level); HM(GMM[i].nametag); HM(GMM[i].x5); }}
    HM((u32) (int) ALLS.x0.x0.ckind); HM(ALLS.x0.x0.color);
    HM(RH(gmMainLib_8045A6C0, {WIN}));
    HM(RH(ARR, sizeof ARR));
    HM(O0.x0); HM(O0.x2); HM((u32) O0.x4); HM((u32) O0.x8);
    HM(RH(&O1, 32));
    HM(PI(O2.x0)); HM((u32) O2.x4);
    hash_StartMeleeData(&SMD);
}}

int main(void)
{{
    static const int KC[{n_ops}] = {{ {kcases} }};
    int op, k;
    {regs("c")}
    preg(&OA, sizeof OA); preg(&DT, sizeof DT); preg(GO, sizeof GO); preg(&SOB, sizeof SOB); preg(MEM, sizeof MEM);
    preg(STR, sizeof STR); preg(U32S, sizeof U32S); preg(I32S, sizeof I32S); preg(GMM, sizeof GMM); preg(&ALLS, sizeof ALLS);
    preg(SYMBUF, sizeof SYMBUF); preg(A5, sizeof A5); preg(A6, sizeof A6); preg(ARR, sizeof ARR);
    preg(&O0, sizeof O0); preg(&O1, sizeof O1); preg(&O2, sizeof O2); preg(&SMD, sizeof SMD);
    freg_all();
    PRISTINE = un_803F9FA4;
    PRISTINE_IMG = un_803FD274;
    for (op = 0; op < {n_ops}; op++) {{
        for (k = 0; k < KC[op]; k++) {{
            u32 ret = 0;
            tu_case(op, k);
            setup();
            switch (op) {{
{chr(10).join(ops_c)}
            }}
            hash_all();
            printf("%d %d %08X %08X %08X%c", op, k, ret, H, TR, 10);
        }}
    }}
    return 0;
}}
'''
    (HERE / "tu/soundtest.c").write_text(c, encoding="utf-8", newline="\n")

    # ---------------------------------------------------------------- Rust
    r = f'''//! src/tu/soundtest.rs vs the C (gen/mtu/tu/soundtest.c: the whole TU, shipping contraction;
//! regenerate the expected file with `python rust/melee_world/gen/mtu/run_tu.py soundtest`).
//! Written by gen/mtu/mk_soundtest.py: do not edit. Ops: {fnames}
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, clippy::all)]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_world::tu::soundtest as m;
use mtu_harness::*;
#[path = "mtu/soundtest_gen.rs"]
mod g;

static mut OA: un_80304138_objalloc_t = unsafe {{ core::mem::zeroed() }};
static mut DT: DevText = unsafe {{ core::mem::zeroed() }};
static mut GO: [HSD_GObj; 4] = unsafe {{ core::mem::zeroed() }};
static mut SOB: HSD_SObj = unsafe {{ core::mem::zeroed() }};
static mut MEM: [u8; 64] = [0; 64];
static mut STR: [i8; 8] = [97, 98, 99, 0, 0, 0, 0, 0];
static mut U32S: [u32; 5] = [0; 5];
static mut I32S: [i32; 2] = [0; 2];
static mut GMM: [gmm_x0_528_t; 3] = unsafe {{ core::mem::zeroed() }};
static mut ALLS: UnkAllstarData = unsafe {{ core::mem::zeroed() }};
static mut SYMBUF: [u64; 16] = [0; 16];
static mut A5: [i32; 256] = [0; 256];
static mut A6: [i32; 64] = [0; 64];
static mut ARR: [i32; 16] = [0; 16];
static mut O0: m::UnkSoundTestData0 = unsafe {{ core::mem::zeroed() }};
static mut O1: m::UnkSoundTestData1 = unsafe {{ core::mem::zeroed() }};
static mut O2: un_80301C64_t = unsafe {{ core::mem::zeroed() }};
static mut SMD: StartMeleeData = unsafe {{ core::mem::zeroed() }};
static mut PRISTINE: Option<m::SoundTestMenuData> = None;
static mut PRISTINE_IMG: Option<HSD_ImageDesc> = None;

unsafe fn hash_258() {{
    {f258_hash("r")}
}}

unsafe fn hash_tables() {{
    {tables_hash("r")}
}}

fn pick() -> u32 {{
    let r = rnd();
    if r & 7 == 7 {{
        return fv() & 15;
    }}
    match (r >> 3) % 3 {{
        0 => 0,
        1 => 1,
        _ => 6,
    }}
}}

unsafe fn setup() {{
    m::un_803F9FA4 = PRISTINE.unwrap();
    m::un_803FD274 = PRISTINE_IMG.unwrap();
    {f258_fill("r")}
    {chr(10).join("    " + x for x in st_fill_r).lstrip()}
    m::un_804D6DA8 = &raw mut SYMBUF as *mut i32;
    for i in 0..4 {{
        SYMBUF[i] = fv64();
    }}
    SYMBUF[4] = fv64();
    SYMBUF[7] = fv64();
    SYMBUF[5] = &raw mut A5 as usize as u64;
    SYMBUF[6] = &raw mut A6 as usize as u64;
    for i in 0..256 {{
        A5[i] = fv() as i32;
    }}
    for i in 0..64 {{
        A6[i] = (rnd() % 4) as i32;
    }}
    for i in 0..16 {{
        ARR[i] = fv() as i32;
    }}
    OA = core::mem::zeroed();
    DT = core::mem::zeroed();
    GO = core::mem::zeroed();
    SOB = core::mem::zeroed();
    g::fill_un_80304138_objalloc_t(&mut OA);
    OA.x4 = &raw mut DT;
    DT.scale_x = rw();
    DT.scale_y = rw();
    for i in 0..4 {{
        g::fill_HSD_GObj(&mut GO[i]);
    }}
    SOB.x10 = rw();
    SOB.x14 = rw();
    SOB.x40 = fv();
    for i in 0..5 {{
        U32S[i] = fv();
    }}
    for i in 0..2 {{
        I32S[i] = fv() as i32;
    }}
    for i in 0..3 {{
        GMM[i].c_kind = rnd() as i8;
        GMM[i].stocks = rnd() as u8;
        GMM[i].color = rnd() as u8;
        GMM[i].cpu_level = rnd() as u8;
        GMM[i].nametag = rnd() as u8;
        GMM[i].x5 = rnd() as u8;
    }}
    ALLS.x0.x0.ckind = rnd() as i8;
    ALLS.x0.x0.color = rnd() as u8;
    for i in 0..{WIN} {{
        m::ext::gmMainLib_8045A6C0[i] = rnd() as u8;
    }}
    O0.x0 = rnd() as u16;
    O0.x2 = rnd() as u16;
    O0.x4 = fv() as i32;
    O0.x8 = fv() as i32;
    for i in 0..32 {{
        *(&raw mut O1 as *mut u8).add(i) = rnd() as u8;
    }}
    O2.x0 = if rnd() & 1 != 0 {{ &raw mut MEM as *mut c_void }} else {{ null_mut() }};
    O2.x4 = fv() as i32;
    g::fill_StartMeleeData(&mut SMD);
}}

unsafe fn hash_all() {{
    {chr(10).join("    " + x for x in st_hash_r).lstrip()}
    hash_258();
    for i in 0..11 {{
        g::hash_un_80304138_objalloc_t_x8(&m::un_803F9FA4.entries[i]);
    }}
    hm(pi(m::un_803FD274.image_ptr));
    g::hash_un_80304138_objalloc_t(&OA);
    hm(DT.scale_x.to_bits());
    hm(DT.scale_y.to_bits());
    for i in 0..4 {{
        g::hash_HSD_GObj(&GO[i]);
    }}
    hm(SOB.x10.to_bits());
    hm(SOB.x14.to_bits());
    hm(SOB.x40);
    for i in 0..5 {{
        hm(U32S[i]);
    }}
    for i in 0..2 {{
        hm(I32S[i] as u32);
    }}
    for i in 0..3 {{
        hm(GMM[i].c_kind as i32 as u32);
        hm(GMM[i].stocks as u32);
        hm(GMM[i].color as u32);
        hm(GMM[i].cpu_level as u32);
        hm(GMM[i].nametag as u32);
        hm(GMM[i].x5 as u32);
    }}
    hm(ALLS.x0.x0.ckind as i32 as u32);
    hm(ALLS.x0.x0.color as u32);
    hm(rh(&*(&raw const m::ext::gmMainLib_8045A6C0 as *const [u8; {WIN}])));
    hm(rh(&ARR));
    hm(O0.x0 as u32);
    hm(O0.x2 as u32);
    hm(O0.x4 as u32);
    hm(O0.x8 as u32);
    hm(rh(&*(&raw const O1 as *const [u8; 32])));
    hm(pi(O2.x0));
    hm(O2.x4 as u32);
    g::hash_StartMeleeData(&SMD);
}}

#[test]
fn soundtest_matches_c() {{
    unsafe {{
        g::install();
        const KC: [u32; {n_ops}] = [{kcases}];
        {regs("r")}
        preg(&raw const OA, core::mem::size_of_val(&OA));
        preg(&raw const DT, core::mem::size_of_val(&DT));
        preg(&raw const GO, core::mem::size_of_val(&GO));
        preg(&raw const SOB, core::mem::size_of_val(&SOB));
        preg(&raw const MEM, core::mem::size_of_val(&MEM));
        preg(&raw const STR, core::mem::size_of_val(&STR));
        preg(&raw const U32S, core::mem::size_of_val(&U32S));
        preg(&raw const I32S, core::mem::size_of_val(&I32S));
        preg(&raw const GMM, core::mem::size_of_val(&GMM));
        preg(&raw const ALLS, core::mem::size_of_val(&ALLS));
        preg(&raw const SYMBUF, core::mem::size_of_val(&SYMBUF));
        preg(&raw const A5, core::mem::size_of_val(&A5));
        preg(&raw const A6, core::mem::size_of_val(&A6));
        preg(&raw const ARR, core::mem::size_of_val(&ARR));
        preg(&raw const O0, core::mem::size_of_val(&O0));
        preg(&raw const O1, core::mem::size_of_val(&O1));
        preg(&raw const O2, core::mem::size_of_val(&O2));
        preg(&raw const SMD, core::mem::size_of_val(&SMD));
        g::freg_all();
        PRISTINE = Some(m::un_803F9FA4);
        PRISTINE_IMG = Some(m::un_803FD274);
        let mut c = Check::new(include_str!("data/mtu_soundtest.txt"));
        let mut total = 0;
        for op in 0..{n_ops}u32 {{
            for k in 0..KC[op as usize] {{
                case(op, k);
                setup();
                let mut ret = 0u32;
                match op {{
{chr(10).join(ops_r)}
                    _ => unreachable!(),
                }}
                hash_all();
                c.line(op, k, ret);
                total += 1;
            }}
        }}
        assert_eq!(c.done(), total);
    }}
}}
'''
    (ROOT / "rust/melee_world/tests/tu_soundtest.rs").write_text(r, encoding="utf-8", newline="\n")
    print(f"mk_soundtest: {n_ops} ops, {sum(KOF[n] for n in ALLOPS)} cases")


if __name__ == "__main__":
    main()
