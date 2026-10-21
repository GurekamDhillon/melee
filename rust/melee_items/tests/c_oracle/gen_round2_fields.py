#!/usr/bin/env python3
"""Emit the round-2 Item fill/snapshot code for both sides of the oracle from one field list.

  python gen_round2_fields.py   ->  round2_fields.h (C, #included by round2.c)
                                    ../round2/fields.rs (Rust, #[path] module of tests/oracle.rs)

Both sides fill the same fields, in the same order, from the same LCG, and hash the same fields
(in the same order) after the function under test runs. Bitfield units go through a "canonical"
value: the GameCube big-endian word of the unit (MSB = first declared field), built from the C
fields one by one, and read straight from the Rust `bf_*` bytes. So a test also checks the
generated bit numbering of every unit it touches.
"""
import os

here = os.path.dirname(os.path.abspath(__file__))

F, I, P, C = "f", "i", "p", "c"
fields = []


def f(c, r=None):
    fields.append((F, c, r or c))


def i(c, r=None, cexpr="R()", rexpr="g.r()"):
    fields.append((I, c, r or c, cexpr, rexpr))


def p(c, r=None):
    fields.append((P, c, r or c))


def canon(cget, cset, rget, rset):
    fields.append((C, cget, cset, rget, rset))


def vec3(c, r=None):
    r = r or c
    for k in "xyz":
        f(f"{c}.{k}", f"{r}.{k}")


def vec2(c, r=None):
    r = r or c
    for k in "xy":
        f(f"{c}.{k}", f"{r}.{k}")


KINDS = "(KINDS[R() % NKINDS])"
i("ip->kind", cexpr=KINDS, rexpr="KINDS[(g.r() % KINDS.len() as u32) as usize]")
i("ip->msid", cexpr="(s32) (R() % 14) - 1", rexpr="(g.r() % 14) as i32 - 1")
i("ip->x1C")
i("ip->x20_team_id")
f("ip->facing_dir")
f("ip->scl")
for v in ["pos", "x40_vel", "x58_vec_unk", "x64_vec_unk2", "x70_nudge", "x7C", "x88", "x94", "xA0",
          "xAC_unk"]:
    vec3(f"ip->{v}")
for k in range(4):
    h = f"ip->x5D4_hitboxes[{k}].hit"
    i(f"{h}.state", cexpr="(R() & 1) ? 0 : R() % 5", rexpr="if g.r() & 1 != 0 { 0 } else { g.r() % 5 }")
    f(f"{h}.scale")
    f(f"{h}.damage")
    vec3(f"{h}.x4C")
    vec3(f"{h}.x58")
    canon(f"hit_x40_get(&{h})", f"hit_x40_set(&{h}, (u16) R())",
          f"(u32::from_be_bytes({h}.bf_40) >> 16)",
          f"{{ let v = g.r() as u16; {h}.bf_40[..2].copy_from_slice(&v.to_be_bytes()); }}")
i("ip->xAC8_hurtboxNum", cexpr="R() % 3", rexpr="(g.r() % 3) as u8")
for k in range(2):
    h = f"ip->xACC_itemHurtbox[{k}]"
    i(f"{h}.state", cexpr="R() % 3", rexpr="g.r() % 3")
    vec3(f"{h}.a_offset")
    vec3(f"{h}.b_offset")
    f(f"{h}.scale")
    canon(f"hurt_x24_get(&{h})", f"hurt_x24_set(&{h}, (u8) R())",
          f"({h}.bf_24[0] as u32)", f"{{ {h}.bf_24[0] = g.r() as u8; }}")
f("ip->xB54.unk_x")
f("ip->xB54.unk_y")
vec3("ip->xB54.x8")
vec2("ip->xBCC_unk")
vec2("ip->xBD4_grabRange")
for e in ["xBDC", "xBEC"]:
    for k in ["top", "bottom", "right", "left"]:
        f(f"ip->{e}.{k}")
for n in ["xC30", "xC34_damageDealt"]:
    i(f"ip->{n}")
for n in ["xC3C", "xC40", "xC44"]:
    f(f"ip->{n}")
for n in ["xC48", "xC4C", "xC50"]:
    i(f"ip->{n}")
p("ip->xC64_reflectGObj")
f("ip->xC70")
p("ip->xC90_absorbGObj")
for n in ["xC94", "xC9C", "xCA0", "xCA4", "xCB0_source_ply"]:
    i(f"ip->{n}")
f("ip->xCBC_hitlagFrames")
for n in ["xCEC_fighterGObj", "xCF4_fighterGObjUnk", "toucher", "owner", "grab_victim"]:
    p(f"ip->{n}")
i("ip->xD09")
i("ip->xD0C")
f("ip->xD44_lifeTimer")
f("ip->xD48_halfLifeTimer")
i("ip->xD4C", cexpr="(s32) (R() % 5) - 2", rexpr="(g.r() % 5) as i32 - 2")
i("ip->xD88_attackID")
i("ip->xDC0")
canon("flags_daa_get(ip)", "flags_daa_set(ip, (u8) R())",
      "(ip.anon_daa.xDAA_flag.byte as u32)", "{ ip.anon_daa.xDAA_flag.byte = g.r() as u8; }")
canon("flag32_get(&ip->xDC8_word)", "flag32_set(&ip->xDC8_word, R())",
      "u32::from_be_bytes(ip.xDC8_word.flags.bf_0)", "{ ip.xDC8_word.flags.bf_0 = g.r().to_be_bytes(); }")
canon("dcc_get(ip)", "dcc_set(ip, (u8) R())",
      "(ip.xDCC_flag.bf_0[0] as u32)", "{ ip.xDCC_flag.bf_0[0] = g.r() as u8; }")
canon("dcd_get(ip)", "dcd_set(ip, (u8) R())",
      "(ip.xDCD_flag.anon_0.bf_0[0] as u32)", "{ ip.xDCD_flag.anon_0.bf_0[0] = g.r() as u8; }")
for n in ["xDCE_flag", "xDD0_flag"]:
    canon(f"ufs_get(&ip->{n})", f"ufs_set(&ip->{n}, (u8) R())",
          f"(ip.{n}.byte as u32)", f"{{ ip.{n}.byte = g.r() as u8; }}")
# item vars read by the heal/ammo getters (a union: later writes win, the same on both sides)
i("ip->xDD4_itemVar.heart.xDD4_heal")
i("ip->xDD4_itemVar.tomato.heal_amount")
i("ip->xDD4_itemVar.foods.heal_amount")
i("ip->xDD4_itemVar.egg.heal_amount")
i("ip->xDD4_itemVar.whispyapple.xDD4_heal")
f("ip->xDD4_itemVar.bombhei.xDEC")
cd = "ip->x378_itemColl"
i(f"{cd}.env_flags")
for s in ["floor", "left_facing_wall", "right_facing_wall", "ceiling"]:
    i(f"{cd}.{s}.index")
    vec3(f"{cd}.{s}.normal")
for e in ["ecb", "xE4_ecb"]:
    for k in ["top", "bottom", "right", "left"]:
        vec2(f"{cd}.{e}.{k}")
vec3(f"{cd}.last_pos")


def rust_path(s):
    return s.replace("ip->", "ip.").replace("->", ".")


c_fill, c_snap, r_fill, r_snap = [], [], [], []
for e in fields:
    kind = e[0]
    if kind == F:
        c_fill.append(f"    {e[1]} = RF();")
        c_snap.append(f"    h = H(h, bits({e[1]}));")
        r_fill.append(f"        {rust_path(e[2])} = g.rf();")
        r_snap.append(f"        h = hmix(h, {rust_path(e[2])}.to_bits());")
    elif kind == I:
        c_fill.append(f"    {e[1]} = {e[3]};")
        c_snap.append(f"    h = H(h, (u32) {e[1]});")
        r_fill.append(f"        {rust_path(e[2])} = ({e[4]}) as _;")
        r_snap.append(f"        h = hmix(h, {rust_path(e[2])} as u32);")
    elif kind == P:
        c_fill.append(f"    {e[1]} = PTR(R());")
        c_snap.append(f"    h = H(h, PIDX({e[1]}));")
        r_fill.append(f"        {rust_path(e[2])} = g.ptr();")
        r_snap.append(f"        h = hmix(h, g.pidx({rust_path(e[2])}));")
    else:
        c_fill.append(f"    {e[2]};")
        c_snap.append(f"    h = H(h, (u32) {e[1]});")
        r_fill.append(f"        {rust_path(e[4])}")
        r_snap.append(f"        h = hmix(h, {rust_path(e[3])});")

hdr = "generated by tests/c_oracle/gen_round2_fields.py; do not edit"
with open(os.path.join(here, "round2_fields.h"), "w", newline="\n") as o:
    o.write(f"/* @{hdr} */\n")
    o.write("static void fill(Item* ip)\n{\n    memset(ip, 0, sizeof *ip);\n")
    o.write("\n".join(c_fill) + "\n}\n")
    o.write("static u32 snap(Item* ip)\n{\n    u32 h = 0x811C9DC5u;\n")
    o.write("\n".join(c_snap) + "\n    return h;\n}\n")
with open(os.path.join(here, "..", "round2", "fields.rs"), "w", newline="\n") as o:
    o.write(f"// @{hdr}\n")
    o.write("#![allow(unused_unsafe, unused_parens, clippy::all)]\nuse super::{hmix, Gen, KINDS};\n"
            "use melee_items::layout::Item;\n\n")
    o.write("pub fn fill(ip: &mut Item, g: &mut Gen) {\n    *ip = Item::zeroed();\n    unsafe {\n")
    o.write("\n".join(r_fill) + "\n    }\n}\n\n")
    o.write("pub fn snap(ip: &Item, g: &Gen) -> u32 {\n    let mut h = 0x811C9DC5u32;\n    unsafe {\n")
    o.write("\n".join(r_snap) + "\n    }\n    h\n}\n")
print(len(fields), "fields")
