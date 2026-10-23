#!/usr/bin/env python3
"""Writes the itanimlist oracle driver pair from one description (like mk_stage_driver.py):

    python rust/melee_world/gen/mtu/mk_itanimlist.py      (from the worktree root)

  rust/melee_world/gen/mtu/tu/itanimlist.c   C driver (the game's itanimlist.c included verbatim,
                                              except one patched expression, see specs.py)
  rust/melee_items/tests/tu_itanimlist.rs    Rust test, draw for draw the same

The two sides keep a script word in different byte layouts (the C build's bitfields are LSB-first
over little-endian words, melee_types' are GameCube MSB-first over big-endian bytes), so each word
is built from the same field VALUES with each side's own setters; the raw halfword / word reads
(`((u16*)u)[k]`, `*(u32*)u`) see the byte-swapped copy on the C side, which is what makes them
equal the port's big-endian reads. Item state is compared field by field (hitboxes, flags, item
vars, the command info), not as raw bytes.
"""
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
HIT1 = ["x40_b0", "x40_b1", "x40_b2", "x40_b3"]
HIT_BF1 = ["x41_b4", "x41_b5", "x41_b6", "x41_b7"] + [f"x42_b{i}" for i in range(8)] + [f"x43_b{i}" for i in range(8)]
HIT_U32 = ["state", "x4", "unk_count", "x24", "x28", "x2C", "element", "sfx_kind"]
HIT_I32 = ["kb_angle", "x34", "sfx_severity"]
HIT_F32 = ["damage", "scale"]
HIT_V3 = ["x", "y", "z"]
FLAG_B = [f"b{i}" for i in range(8)]

# ---------------------------------------------------------------------------------------- C side

def c_setup():
    L = []
    for h in range(4):
        p = f"ITEM.x5D4_hitboxes[{h}]"
        for f in HIT_U32:
            L.append(f"    {p}.hit.{f} = fv();")
        L.append(f"    {p}.hit.state = rnd() & 1;")
        L.append(f"    {p}.hit.x4 = rnd() % 8;")
        for f in HIT_I32:
            L.append(f"    {p}.hit.{f} = (int) fv();")
        for f in HIT_F32:
            L.append(f"    {p}.hit.{f} = rw();")
        for a in HIT_V3:
            L.append(f"    {p}.hit.b_offset.{a} = rw();")
        for f in HIT1 + HIT_BF1:
            L.append(f"    {p}.hit.{f} = rnd() & 1;")
        L.append(f"    {p}.hit.x40_b4 = rnd() & 0xFF;")
        L.append(f"    {p}.hit.x44 = (u8) rnd(); {p}.hit.x45 = (u8) rnd();")
        L.append(f"    {p}.hit.jobj = (rnd() & 1) ? &BONES[rnd() % 8] : 0;")
        L.append(f"    {p}.x138 = rnd() & 1;")
    return "\n".join(L)


def c_hash():
    L = []
    for h in range(4):
        p = f"ITEM.x5D4_hitboxes[{h}]"
        for f in HIT_U32 + HIT_I32:
            L.append(f"    HM((u32) {p}.hit.{f});")
        for f in HIT_F32:
            L.append(f"    HM(B({p}.hit.{f}));")
        for a in HIT_V3:
            L.append(f"    HM(B({p}.hit.b_offset.{a}));")
        for f in HIT1 + HIT_BF1 + ["x40_b4"]:
            L.append(f"    HM((u32) {p}.hit.{f});")
        L.append(f"    HM({p}.hit.x44); HM({p}.hit.x45); HM(PI({p}.hit.jobj)); HM((u32) {p}.x138);")
    return "\n".join(L)


C_TEMPLATE = r'''/* TU: src/melee/it/itanimlist.c (written by gen/mtu/mk_itanimlist.py: do not edit) */
/* C oracle for melee_items src/tu/itanimlist.rs: the whole TU included verbatim (one expression
 * patched, see specs.py); every callee out of it is a recording stub. Each line: op case ret
 * state-hash trace-hash. */
#include TU_SRC
#include "mtu_common.h"
static HSD_GObj G;
static Item ITEM;
static HSD_JObj JT, BONES[8];
static DynamicBoneTable DBT;
static HSD_GObjProc PROCX;
static union CmdUnion W[24];
static CommandInfo CMDX;
static Fighter_804D653C_t FD;
static int CE_RET;
static int GKIND;
#include "itanimlist_gen.h"

static void wbytes(int w, u32 b0, u32 b1, u32 b2, u32 b3)
{
    unsigned char* p = (unsigned char*) &W[w];
    p[0] = (unsigned char) b0; p[1] = (unsigned char) b1; p[2] = (unsigned char) b2; p[3] = (unsigned char) b3;
}

/* one typed command (opcode 10 + kind) at W[0], everything else zero */
static void genw(int kind)
{
    zero(W, sizeof W);
    GKIND = kind;
    switch (kind) {
    case 0: {
        int w;
        for (w = 0; w < 5; w++) {
            u32 v = rnd();
            u32 r0 = v >> 24, r1 = (v >> 16) & 0xFF, r2 = (v >> 8) & 0xFF, r3 = v & 0xFF;
            if (w == 0) { r0 = (10u << 2) | (r0 & 3); r1 = (r1 & 0xC0) | 10; }
            else {
                /* a header the dispatcher may read: the opcode is the same in both bitfield orders and
                   never a handler (the handlers' bitfield operands are covered by the typed ops) */
                u32 a = r0 >> 2;
                if (a >= 10 && a <= 25) a += 16;
                r0 = (a << 2) | (r0 & 3);
                r1 = (r1 & 0xC0) | a;
            }
            wbytes(w, r1, r0, r3, r2);
        }
        break;
    }
    case 1: {
        u32 bone;
        W[0].it_create_hitbox_0.opcode = 11;
        W[0].it_create_hitbox_0.id = rnd() % 4;
        W[0].it_create_hitbox_0.hit_group = rnd() % 8;
        bone = (rnd() % 3 == 0) ? 0 : 1 + rnd() % 99;
        W[0].it_create_hitbox_0.bone = bone;
        W[0].it_create_hitbox_0.damage = rnd() & 0x1FFF;
        W[1].create_hitbox_1.size = rnd() & 0xFFFF;
        W[1].create_hitbox_1.z_offset = (s16) rnd();
        W[2].create_hitbox_2.y_offset = (s16) rnd();
        W[2].create_hitbox_2.x_offset = (s16) rnd();
        W[3].create_hitbox_3.angle = rnd() & 0x1FF;
        W[3].create_hitbox_3.knockback_growth = rnd() & 0x1FF;
        W[3].create_hitbox_3.weight_set_knockback = rnd() & 0x1FF;
        W[3].create_hitbox_3.item_hit_interaction = rnd() & 1;
        W[3].create_hitbox_3.ignore_thrown_fighters = rnd() & 1;
        W[3].create_hitbox_3.ignore_fighter_scale = rnd() & 1;
        W[3].create_hitbox_3.clank = rnd() & 1;
        W[3].create_hitbox_3.rebound = rnd() & 1;
        W[4].it_create_hitbox_4.base_knockback = rnd() & 0x1FF;
        W[4].it_create_hitbox_4.element = rnd() & 0x1F;
        W[4].it_create_hitbox_4.x40_b0 = rnd() & 1;
        W[4].it_create_hitbox_4.shield_damage = (s8) rnd();
        W[4].it_create_hitbox_4.sfx_severity = rnd() & 7;
        W[4].it_create_hitbox_4.sfx_kind = rnd() & 0xF;
        W[4].it_create_hitbox_4.x40_b3 = rnd() & 1;
        W[4].it_create_hitbox_4.x40_b2 = rnd() & 1;
        W[5].create_hitbox_5.x0 = rnd() & 0xFF;
        W[5].create_hitbox_5.x1_b0 = rnd() & 1;
        W[5].create_hitbox_5.x1_b1 = rnd() & 1;
        W[5].create_hitbox_5.x1_b2 = rnd() & 1;
        W[5].create_hitbox_5.x1_b3 = rnd() & 1;
        W[5].create_hitbox_5.x1_b4 = rnd() & 1;
        W[5].create_hitbox_5.x1_b5 = rnd() & 1;
        W[5].create_hitbox_5.x1_b6 = rnd() & 1;
        W[5].create_hitbox_5.x1_b7 = rnd() & 1;
        ((unsigned char*) &W[5])[2] = (unsigned char) (rnd() & 0xFF);
        break;
    }
    case 2: {
        u32 hv;
        W[0].set_hitbox_damage.opcdoe = 12;
        W[0].set_hitbox_damage.idx = rnd() % 4;
        hv = rnd() & 0xFFFF;
        ((unsigned char*) &W[0])[2] = (unsigned char) (hv & 0xFF);
        ((unsigned char*) &W[0])[3] = (unsigned char) (hv >> 8);
        break;
    }
    case 3:
        W[0].set_hitbox_scale.opcode = 13;
        W[0].set_hitbox_scale.idx = rnd() % 4;
        W[0].set_hitbox_scale.value = rnd() & 0x7FFFFF;
        break;
    case 4: W[0].set_throw_flags.opcode = 14; W[0].set_throw_flags.hit_idx = rnd() & 0x3FFFFFF; break;
    case 5: W[0].set_throw_flags.opcode = 15; break;
    case 6: {
        static const u32 ops[7] = { 0, 1, 2, 10, 11, 3, 200 };
        u32 v;
        ((itAnimlistCmdUnk*) &W[0])->x0_b0 = 16;
        ((itAnimlistCmdUnk*) &W[0])->opcode = ops[rnd() % 7];
        ((itAnimlistCmdUnk*) &W[0])->x0_b14 = rnd() & 3;
        ((itAnimlistCmdUnk*) &W[0])->x2 = rnd() & 0xFFFF;
        v = rnd();
        { u32 a = v >> 26; if (a >= 10 && a <= 25) a += 16; v = (v & 0x03FFFFC0u) | (a << 26) | a; } /* header opcode: low 6 bits natively, top 6 on the console */
        wbytes(1, v & 0xFF, (v >> 8) & 0xFF, (v >> 16) & 0xFF, v >> 24);
        v = rnd();
        { u32 a = v >> 26; if (a >= 10 && a <= 25) a += 16; v = (v & 0x03FFFFFFu) | (a << 26); }
        wbytes(2, v >> 26, (v >> 16) & 0xFF, (v >> 8) & 0xFF, v & 0xFF);
        break;
    }
    case 7: W[0].set_throw_flags.opcode = 17; W[0].set_throw_flags.hit_idx = rnd() & 0x3FFFFFF; break;
    case 8: W[0].set_throw_flags.opcode = 18; W[0].set_throw_flags.hit_idx = rnd() & 0x3FFFFFF; break;
    case 9: W[0].set_throw_flags.opcode = 19; W[0].set_throw_flags.hit_idx = rnd() & 0x3FFFFFF; break;
    case 10: W[0].set_throw_flags.opcode = 20; break;
    case 11: W[0].unk33.opcode = 21; W[0].unk33.unk0 = rnd() & 0x1FFF; W[0].unk33.unk1 = rnd() & 0x1FFF; break;
    case 12: W[0].set_throw_flags.opcode = 22; break;
    case 13: W[0].unk33.opcode = 23; W[0].unk33.unk0 = rnd() & 0x1FFF; W[0].unk33.unk1 = rnd() & 0x1FFF; break;
    case 14: W[0].unk13.unk0 = 24; W[0].unk13.unk1 = rnd() & 0xFF; W[0].unk13.unk2 = rnd() & 0x3FFFF; break;
    default: W[0].set_throw_flags.opcode = 25; break;
    }
}

static void setup(void)
{
    int i;
    zero(&G, sizeof G); zero(&ITEM, sizeof ITEM); zero(&JT, sizeof JT); zero(BONES, sizeof BONES);
    zero(&DBT, sizeof DBT); zero(&PROCX, sizeof PROCX); zero(&CMDX, sizeof CMDX); zero(&FD, sizeof FD);
    zero(W, sizeof W);
    G.user_data = &ITEM;
    G.hsd_obj = &JT;
    for (i = 0; i < 100; i++) DBT.bones[i] = &BONES[i % 8];
    ITEM.xBBC_dynamicBoneTable = &DBT;
PLACEHOLDER_SETUP
    ITEM.xC3C = rw(); ITEM.xC40 = rw(); ITEM.scl = rw(); ITEM.x3C = rw();
    ITEM.x5CC_currentAnimFrame = rw(); ITEM.x5D0_animFrameSpeed = rw();
    ITEM.xDAC_itcmd_var0 = fv(); ITEM.xDB0_itcmd_var1 = fv(); ITEM.xDB4_itcmd_var2 = fv();
    ITEM.xDC8_word.flags.x16 = rnd() & 1;
    ITEM.xDAA_flag.b0 = rnd() & 1; ITEM.xDAA_flag.b1 = rnd() & 1; ITEM.xDAA_flag.b2 = rnd() & 1; ITEM.xDAA_flag.b3 = rnd() & 1;
    ITEM.xDAA_flag.b4 = rnd() & 1; ITEM.xDAA_flag.b5 = rnd() & 1; ITEM.xDAA_flag.b6 = rnd() & 1; ITEM.xDAA_flag.b7 = rnd() & 1;
    ITEM.xDBC_itcmd_var4_word = rnd() & 1;
    ITEM.x524_cmd.timer = (rnd() % 4 == 0) ? F32_MAX : rw();
    ITEM.x524_cmd.frame_count = rw();
    ITEM.x524_cmd.u = (rnd() % 5 == 0) ? 0 : &W[0];
    PROCX.s_link = (u8) (rnd() % 20);
    HSD_GObj_CurrentInvokedProc = (rnd() & 1) ? &PROCX : 0;
    it_804D6D04 = (rnd() & 1) ? &FD : 0;
    CMDX.timer = rw(); CMDX.frame_count = rw(); CMDX.u = &W[0];
    CE_RET = 0;
}

static void hash_all(void)
{
PLACEHOLDER_HASH
    HM(B(ITEM.x3C));
    HM((u32) ITEM.xDC8_word.flags.x16);
    HM(ITEM.xDAA_flag.b0); HM(ITEM.xDAA_flag.b1); HM(ITEM.xDAA_flag.b2); HM(ITEM.xDAA_flag.b3);
    HM(ITEM.xDAA_flag.b4); HM(ITEM.xDAA_flag.b5); HM(ITEM.xDAA_flag.b6); HM(ITEM.xDAA_flag.b7);
    HM(ITEM.xDAC_itcmd_var0); HM(ITEM.xDB0_itcmd_var1); HM(ITEM.xDB4_itcmd_var2);
    HM((u32) (ITEM.xDBC_itcmd_var4_word != 0));
    HM(B(ITEM.x524_cmd.timer)); HM(B(ITEM.x524_cmd.frame_count)); HM(PI(ITEM.x524_cmd.u));
    HM(B(CMDX.timer)); HM(B(CMDX.frame_count)); HM(PI(CMDX.u));
    HM((u32) CE_RET);
}

int main(void)
{
    int op, k;
    preg(&G, sizeof G); preg(&ITEM, sizeof ITEM); preg(&JT, sizeof JT); preg(BONES, sizeof BONES);
    preg(&DBT, sizeof DBT); preg(&PROCX, sizeof PROCX); preg(W, sizeof W); preg(&CMDX, sizeof CMDX); preg(&FD, sizeof FD);
    freg_all();
    for (op = 0; op < 24; op++) {
        for (k = 0; k < 2000; k++) {
            u32 ret = 0;
            static const int kind_of_op[16] = { 0, 1, 2, 3, 4, 5, 7, 8, 9, 10, 6, 11, 12, 13, 14, 15 };
            tu_case(op, k);
            setup();
            if (op < 16) {
                genw(kind_of_op[op]);
                CMDX.u = &W[0];
            }
            switch (op) {
            case 0: it_80278F2C(&G, &CMDX); break;
            case 1: it_802790C0(&G, &CMDX); break;
            case 2: it_80279544(&G, &CMDX); break;
            case 3: it_802795EC(&G, &CMDX); break;
            case 4: it_80279680(&G, &CMDX); break;
            case 5: it_802796C4(&G, &CMDX); break;
            case 6: it_802796FC(&G, &CMDX); break;
            case 7: it_80279720(&G, &CMDX); break;
            case 8: it_80279744(&G, &CMDX); break;
            case 9: it_80279768(&G, &CMDX); break;
            case 10: it_8027978C(&G, &CMDX); break;
            case 11: it_80279888(&G, &CMDX); break;
            case 12: it_802798D4(&G, &CMDX); break;
            case 13: it_8027990C(&G, &CMDX); break;
            case 14: it_80279958(&G, &CMDX); break;
            case 15: it_802799A8(&G, &CMDX); break;
            case 16: {
                int kd = (int) (rnd() % 16);
                genw(kd);
                it_802799E4(&G);
                break;
            }
            case 17: genw(0); it_80279AF0(&G, &CMDX); break;
            case 18: genw(6); it_80279B10(&G, &CMDX); break;
            case 19: {
                int a2 = 21 + (int) (rnd() & 1);
                genw(a2 == 21 ? 0 : 6);
                fn_80279B30(&G, &CMDX, a2);
                break;
            }
            case 20: it_80279B64(&ITEM); break;
            case 21: { int a = (int) fv(); int b = (int) fv(); it_80279B88(&ITEM, a, b); break; }
            case 22: it_80279BBC(&ITEM); break;
            case 23: genw((rnd() & 1) ? 0 : 6); it_80279BE0(&G); break;
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
'''

# ------------------------------------------------------------------------------------- Rust side

def r_setup():
    L = []
    for h in range(4):
        p = f"ITEM.x5D4_hitboxes[{h}]"
        for f in HIT_U32:
            L.append(f"    {p}.hit.{f} = fv() as _;")
        L.append(f"    {p}.hit.state = rnd() & 1;")
        L.append(f"    {p}.hit.x4 = rnd() % 8;")
        for f in HIT_I32:
            L.append(f"    {p}.hit.{f} = fv() as i32;")
        for f in HIT_F32:
            L.append(f"    {p}.hit.{f} = rw();")
        for a in HIT_V3:
            L.append(f"    {p}.hit.b_offset.{a} = rw();")
        for f in HIT1 + HIT_BF1:
            L.append(f"    {p}.hit.set_{f}((rnd() & 1) as _);")
        L.append(f"    {p}.hit.set_x40_b4((rnd() & 0xFF) as _);")
        L.append(f"    {p}.hit.x44 = rnd() as u8; {p}.hit.x45 = rnd() as u8;")
        L.append(f"    {p}.hit.jobj = if rnd() & 1 != 0 {{ &raw mut BONES[(rnd() % 8) as usize] }} else {{ null_mut() }};")
        L.append(f"    {p}.set_x138((rnd() & 1) as _);")
    return "\n".join(L)


def r_hash():
    L = []
    for h in range(4):
        p = f"ITEM.x5D4_hitboxes[{h}]"
        for f in HIT_U32 + HIT_I32:
            L.append(f"    hm({p}.hit.{f} as u32);")
        for f in HIT_F32:
            L.append(f"    hm({p}.hit.{f}.to_bits());")
        for a in HIT_V3:
            L.append(f"    hm({p}.hit.b_offset.{a}.to_bits());")
        for f in HIT1 + HIT_BF1 + ["x40_b4"]:
            L.append(f"    hm({p}.hit.{f}() as u32);")
        L.append(f"    hm({p}.hit.x44 as u32); hm({p}.hit.x45 as u32); hm(pi({p}.hit.jobj)); hm({p}.x138() as u32);")
    return "\n".join(L)


R_TEMPLATE = r'''//! melee_items src/tu/itanimlist.rs vs the C (gen/mtu/tu/itanimlist.c: the whole TU, shipping
//! contraction). Written by rust/melee_world/gen/mtu/mk_itanimlist.py from the same description as
//! the C driver; regenerate the expected file with
//! `python rust/melee_world/gen/mtu/run_tu.py itanimlist`.
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments)]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_items::tu::itanimlist as m;
use melee_types::*;
use mtu_harness::*;
#[path = "mtu/itanimlist_gen.rs"]
mod g;

static mut G: HSD_GObj = unsafe { core::mem::zeroed() };
static mut ITEM: Item = unsafe { core::mem::zeroed() };
static mut JT: HSD_JObj = unsafe { core::mem::zeroed() };
static mut BONES: [HSD_JObj; 8] = unsafe { core::mem::zeroed() };
static mut DBT: DynamicBoneTable = unsafe { core::mem::zeroed() };
static mut PROCX: HSD_GObjProc = unsafe { core::mem::zeroed() };
static mut W: [CmdUnion; 24] = unsafe { core::mem::zeroed() };
static mut CMDX: CommandInfo = unsafe { core::mem::zeroed() };
static mut FD: Fighter_804D653C_t = unsafe { core::mem::zeroed() };
static mut CE_RET: i32 = 0;
static mut GKIND: i32 = 0;

unsafe fn wbytes(w: usize, b0: u32, b1: u32, b2: u32, b3: u32) {
    let p = &raw mut W[w] as *mut u8;
    *p = b0 as u8;
    *p.add(1) = b1 as u8;
    *p.add(2) = b2 as u8;
    *p.add(3) = b3 as u8;
}

/// one typed command (opcode 10 + kind) at W[0], everything else zero
unsafe fn genw(kind: i32) {
    W = core::mem::zeroed();
    GKIND = kind;
    match kind {
        0 => {
            for w in 0..5usize {
                let v = rnd();
                let mut r0 = v >> 24;
                let mut r1 = (v >> 16) & 0xFF;
                let r2 = (v >> 8) & 0xFF;
                let r3 = v & 0xFF;
                if w == 0 {
                    r0 = (10u32 << 2) | (r0 & 3);
                    r1 = (r1 & 0xC0) | 10;
                } else {
                    let mut a = r0 >> 2;
                    if (10..=25).contains(&a) {
                        a += 16;
                    }
                    r0 = (a << 2) | (r0 & 3);
                    r1 = (r1 & 0xC0) | a;
                }
                wbytes(w, r0, r1, r2, r3);
            }
        }
        1 => {
            W[0].it_create_hitbox_0.set_opcode(11);
            W[0].it_create_hitbox_0.set_id(rnd() % 4);
            W[0].it_create_hitbox_0.set_hit_group(rnd() % 8);
            let bone = if rnd() % 3 == 0 { 0 } else { 1 + rnd() % 99 };
            W[0].it_create_hitbox_0.set_bone(bone);
            W[0].it_create_hitbox_0.set_damage(rnd() & 0x1FFF);
            W[1].create_hitbox_1.set_size(rnd() & 0xFFFF);
            W[1].create_hitbox_1.set_z_offset(rnd() as u16 as i16 as i32);
            W[2].create_hitbox_2.set_y_offset(rnd() as u16 as i16 as i32);
            W[2].create_hitbox_2.set_x_offset(rnd() as u16 as i16 as i32);
            W[3].create_hitbox_3.set_angle(rnd() & 0x1FF);
            W[3].create_hitbox_3.set_knockback_growth(rnd() & 0x1FF);
            W[3].create_hitbox_3.set_weight_set_knockback(rnd() & 0x1FF);
            W[3].create_hitbox_3.set_item_hit_interaction(rnd() & 1);
            W[3].create_hitbox_3.set_ignore_thrown_fighters(rnd() & 1);
            W[3].create_hitbox_3.set_ignore_fighter_scale(rnd() & 1);
            W[3].create_hitbox_3.set_clank(rnd() & 1);
            W[3].create_hitbox_3.set_rebound(rnd() & 1);
            W[4].it_create_hitbox_4.set_base_knockback(rnd() & 0x1FF);
            W[4].it_create_hitbox_4.set_element(rnd() & 0x1F);
            W[4].it_create_hitbox_4.set_x40_b0(rnd() & 1);
            W[4].it_create_hitbox_4.set_shield_damage(rnd() as u8 as i8 as i32);
            W[4].it_create_hitbox_4.set_sfx_severity(rnd() & 7);
            W[4].it_create_hitbox_4.set_sfx_kind(rnd() & 0xF);
            W[4].it_create_hitbox_4.set_x40_b3(rnd() & 1);
            W[4].it_create_hitbox_4.set_x40_b2(rnd() & 1);
            W[5].create_hitbox_5.set_x0(rnd() & 0xFF);
            W[5].create_hitbox_5.set_x1_b0(rnd() & 1);
            W[5].create_hitbox_5.set_x1_b1(rnd() & 1);
            W[5].create_hitbox_5.set_x1_b2(rnd() & 1);
            W[5].create_hitbox_5.set_x1_b3(rnd() & 1);
            W[5].create_hitbox_5.set_x1_b4(rnd() & 1);
            W[5].create_hitbox_5.set_x1_b5(rnd() & 1);
            W[5].create_hitbox_5.set_x1_b6(rnd() & 1);
            W[5].create_hitbox_5.set_x1_b7(rnd() & 1);
            *(&raw mut W[5] as *mut u8).add(2) = (rnd() & 0xFF) as u8;
        }
        2 => {
            W[0].set_hitbox_damage.set_opcdoe(12);
            W[0].set_hitbox_damage.set_idx(rnd() % 4);
            let hv = rnd() & 0xFFFF;
            let p = &raw mut W[0] as *mut u8;
            *p.add(2) = (hv >> 8) as u8;
            *p.add(3) = (hv & 0xFF) as u8;
        }
        3 => {
            W[0].set_hitbox_scale.set_opcode(13);
            W[0].set_hitbox_scale.set_idx(rnd() % 4);
            W[0].set_hitbox_scale.set_value(rnd() & 0x7FFFFF);
        }
        4 => {
            W[0].set_throw_flags.set_opcode(14);
            W[0].set_throw_flags.set_hit_idx(rnd() & 0x3FFFFFF);
        }
        5 => W[0].set_throw_flags.set_opcode(15),
        6 => {
            const OPS: [u32; 7] = [0, 1, 2, 10, 11, 3, 200];
            let op8 = OPS[(rnd() % 7) as usize];
            let b14 = rnd() & 3;
            let x2 = rnd() & 0xFFFF;
            let h = (16u32 << 10) | (op8 << 2) | b14;
            wbytes(0, h >> 8, h & 0xFF, x2 >> 8, x2 & 0xFF);
            let v = rnd();
            let v = {
                let mut a = v >> 26;
                if (10..=25).contains(&a) {
                    a += 16;
                }
                (v & 0x03FF_FFC0) | (a << 26) | a
            };
            wbytes(1, v >> 24, (v >> 16) & 0xFF, (v >> 8) & 0xFF, v & 0xFF);
            let v = rnd();
            let v = {
                let mut a = v >> 26;
                if (10..=25).contains(&a) {
                    a += 16;
                }
                (v & 0x03FF_FFFF) | (a << 26)
            };
            wbytes(2, v >> 24, (v >> 16) & 0xFF, (v >> 8) & 0xFF, v & 0xFF);
        }
        7 => {
            W[0].set_throw_flags.set_opcode(17);
            W[0].set_throw_flags.set_hit_idx(rnd() & 0x3FFFFFF);
        }
        8 => {
            W[0].set_throw_flags.set_opcode(18);
            W[0].set_throw_flags.set_hit_idx(rnd() & 0x3FFFFFF);
        }
        9 => {
            W[0].set_throw_flags.set_opcode(19);
            W[0].set_throw_flags.set_hit_idx(rnd() & 0x3FFFFFF);
        }
        10 => W[0].set_throw_flags.set_opcode(20),
        11 => {
            W[0].unk33.set_opcode(21);
            W[0].unk33.set_unk0(rnd() & 0x1FFF);
            W[0].unk33.set_unk1(rnd() & 0x1FFF);
        }
        12 => W[0].set_throw_flags.set_opcode(22),
        13 => {
            W[0].unk33.set_opcode(23);
            W[0].unk33.set_unk0(rnd() & 0x1FFF);
            W[0].unk33.set_unk1(rnd() & 0x1FFF);
        }
        14 => {
            W[0].unk13.set_unk0(24);
            W[0].unk13.set_unk1(rnd() & 0xFF);
            W[0].unk13.set_unk2(rnd() & 0x3FFFF);
        }
        _ => W[0].set_throw_flags.set_opcode(25),
    }
}

unsafe fn setup() {
    G = core::mem::zeroed();
    ITEM = core::mem::zeroed();
    JT = core::mem::zeroed();
    BONES = core::mem::zeroed();
    DBT = core::mem::zeroed();
    PROCX = core::mem::zeroed();
    CMDX = core::mem::zeroed();
    FD = core::mem::zeroed();
    W = core::mem::zeroed();
    G.user_data = &raw mut ITEM as *mut c_void;
    G.hsd_obj = &raw mut JT as *mut c_void;
    for i in 0..100 {
        DBT.bones[i] = &raw mut BONES[i % 8];
    }
    ITEM.xBBC_dynamicBoneTable = &raw mut DBT;
PLACEHOLDER_SETUP
    ITEM.xC3C = rw();
    ITEM.xC40 = rw();
    ITEM.scl = rw();
    ITEM.x3C = rw();
    ITEM.x5CC_currentAnimFrame = rw();
    ITEM.x5D0_animFrameSpeed = rw();
    ITEM.xDAC_itcmd_var0 = fv();
    ITEM.xDB0_itcmd_var1 = fv();
    ITEM.xDB4_itcmd_var2 = fv();
    ITEM.xDC8_word.flags.set_x16(rnd() & 1);
    ITEM.anon_daa.xDAA_flag.anon_0.set_b0((rnd() & 1) as u8);
    ITEM.anon_daa.xDAA_flag.anon_0.set_b1((rnd() & 1) as u8);
    ITEM.anon_daa.xDAA_flag.anon_0.set_b2((rnd() & 1) as u8);
    ITEM.anon_daa.xDAA_flag.anon_0.set_b3((rnd() & 1) as u8);
    ITEM.anon_daa.xDAA_flag.anon_0.set_b4((rnd() & 1) as u8);
    ITEM.anon_daa.xDAA_flag.anon_0.set_b5((rnd() & 1) as u8);
    ITEM.anon_daa.xDAA_flag.anon_0.set_b6((rnd() & 1) as u8);
    ITEM.anon_daa.xDAA_flag.anon_0.set_b7((rnd() & 1) as u8);
    ITEM.anon_dbc.xDBC_itcmd_var4_word = rnd() & 1;
    ITEM.x524_cmd.timer = if rnd() % 4 == 0 { m::F32_MAX } else { rw() };
    ITEM.x524_cmd.frame_count = rw();
    ITEM.x524_cmd.anon_8.u = if rnd() % 5 == 0 { null_mut() } else { &raw mut W[0] };
    PROCX.s_link = (rnd() % 20) as u8;
    m::ext::HSD_GObj_CurrentInvokedProc = if rnd() & 1 != 0 { &raw mut PROCX } else { null_mut() };
    m::ext::it_804D6D04 = if rnd() & 1 != 0 { &raw mut FD } else { null_mut() };
    CMDX.timer = rw();
    CMDX.frame_count = rw();
    CMDX.anon_8.u = &raw mut W[0];
    CE_RET = 0;
}

unsafe fn hash_all() {
PLACEHOLDER_HASH
    hm(ITEM.x3C.to_bits());
    hm(ITEM.xDC8_word.flags.x16());
    hm(ITEM.anon_daa.xDAA_flag.anon_0.b0() as u32);
    hm(ITEM.anon_daa.xDAA_flag.anon_0.b1() as u32);
    hm(ITEM.anon_daa.xDAA_flag.anon_0.b2() as u32);
    hm(ITEM.anon_daa.xDAA_flag.anon_0.b3() as u32);
    hm(ITEM.anon_daa.xDAA_flag.anon_0.b4() as u32);
    hm(ITEM.anon_daa.xDAA_flag.anon_0.b5() as u32);
    hm(ITEM.anon_daa.xDAA_flag.anon_0.b6() as u32);
    hm(ITEM.anon_daa.xDAA_flag.anon_0.b7() as u32);
    hm(ITEM.xDAC_itcmd_var0);
    hm(ITEM.xDB0_itcmd_var1);
    hm(ITEM.xDB4_itcmd_var2);
    hm((ITEM.anon_dbc.xDBC_itcmd_var4_word != 0) as u32);
    hm(ITEM.x524_cmd.timer.to_bits());
    hm(ITEM.x524_cmd.frame_count.to_bits());
    hm(pi(ITEM.x524_cmd.anon_8.u));
    hm(CMDX.timer.to_bits());
    hm(CMDX.frame_count.to_bits());
    hm(pi(CMDX.anon_8.u));
    hm(CE_RET as u32);
}

#[test]
fn itanimlist_matches_c() {
    unsafe {
        g::install();
        preg(&raw const G, core::mem::size_of::<HSD_GObj>());
        preg(&raw const ITEM, core::mem::size_of::<Item>());
        preg(&raw const JT, core::mem::size_of::<HSD_JObj>());
        preg(&raw const BONES, core::mem::size_of_val(&BONES));
        preg(&raw const DBT, core::mem::size_of::<DynamicBoneTable>());
        preg(&raw const PROCX, core::mem::size_of::<HSD_GObjProc>());
        preg(&raw const W, core::mem::size_of_val(&W));
        preg(&raw const CMDX, core::mem::size_of::<CommandInfo>());
        preg(&raw const FD, core::mem::size_of::<Fighter_804D653C_t>());
        g::freg_all();
        let mut c = Check::new(include_str!("data/mtu_itanimlist.txt"));
        const KIND_OF_OP: [i32; 16] = [0, 1, 2, 3, 4, 5, 7, 8, 9, 10, 6, 11, 12, 13, 14, 15];
        for op in 0..24u32 {
            for k in 0..2000u32 {
                case(op, k);
                setup();
                let ret = 0u32;
                if op < 16 {
                    genw(KIND_OF_OP[op as usize]);
                    CMDX.anon_8.u = &raw mut W[0];
                }
                let g_ = &raw mut G;
                let cmd = &raw mut CMDX;
                match op {
                    0 => m::it_80278F2C(g_, cmd),
                    1 => m::it_802790C0(g_, cmd),
                    2 => m::it_80279544(g_, cmd),
                    3 => m::it_802795EC(g_, cmd),
                    4 => m::it_80279680(g_, cmd),
                    5 => m::it_802796C4(g_, cmd),
                    6 => m::it_802796FC(g_, cmd),
                    7 => m::it_80279720(g_, cmd),
                    8 => m::it_80279744(g_, cmd),
                    9 => m::it_80279768(g_, cmd),
                    10 => m::it_8027978C(g_, cmd),
                    11 => m::it_80279888(g_, cmd),
                    12 => m::it_802798D4(g_, cmd),
                    13 => m::it_8027990C(g_, cmd),
                    14 => m::it_80279958(g_, cmd),
                    15 => m::it_802799A8(g_, cmd),
                    16 => {
                        let kd = (rnd() % 16) as i32;
                        genw(kd);
                       
                        m::it_802799E4(g_);
                    }
                    17 => {
                        genw(0);
                        m::it_80279AF0(g_, cmd);
                    }
                    18 => {
                        genw(6);
                        m::it_80279B10(g_, cmd);
                    }
                    19 => {
                        let a2 = 21 + (rnd() & 1) as i32;
                        genw(if a2 == 21 { 0 } else { 6 });
                        m::fn_80279B30(g_, cmd, a2);
                    }
                    20 => m::it_80279B64(&raw mut ITEM),
                    21 => {
                        let a = fv() as i32;
                        let b = fv() as i32;
                        m::it_80279B88(&raw mut ITEM, a, b);
                    }
                    22 => m::it_80279BBC(&raw mut ITEM),
                    _ => {
                        genw(if rnd() & 1 != 0 { 0 } else { 6 });
                       
                        m::it_80279BE0(g_);
                    }
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 24 * 2000);
    }
}
'''


def main():
    c = C_TEMPLATE.replace("PLACEHOLDER_SETUP", c_setup()).replace("PLACEHOLDER_HASH", c_hash())
    (ROOT / "rust/melee_world/gen/mtu/tu/itanimlist.c").write_text(c, encoding="utf-8", newline="\n")
    r = R_TEMPLATE.replace("PLACEHOLDER_SETUP", r_setup()).replace("PLACEHOLDER_HASH", r_hash())
    (ROOT / "rust/melee_items/tests/tu_itanimlist.rs").write_text(r, encoding="utf-8", newline="\n")
    print("mk_itanimlist: wrote the C driver and the Rust test")


if __name__ == "__main__":
    main()
