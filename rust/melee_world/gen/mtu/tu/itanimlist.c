/* TU: src/melee/it/itanimlist.c (written by gen/mtu/mk_itanimlist.py: do not edit) */
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
    ITEM.x5D4_hitboxes[0].hit.state = fv();
    ITEM.x5D4_hitboxes[0].hit.x4 = fv();
    ITEM.x5D4_hitboxes[0].hit.unk_count = fv();
    ITEM.x5D4_hitboxes[0].hit.x24 = fv();
    ITEM.x5D4_hitboxes[0].hit.x28 = fv();
    ITEM.x5D4_hitboxes[0].hit.x2C = fv();
    ITEM.x5D4_hitboxes[0].hit.element = fv();
    ITEM.x5D4_hitboxes[0].hit.sfx_kind = fv();
    ITEM.x5D4_hitboxes[0].hit.state = rnd() & 1;
    ITEM.x5D4_hitboxes[0].hit.x4 = rnd() % 8;
    ITEM.x5D4_hitboxes[0].hit.kb_angle = (int) fv();
    ITEM.x5D4_hitboxes[0].hit.x34 = (int) fv();
    ITEM.x5D4_hitboxes[0].hit.sfx_severity = (int) fv();
    ITEM.x5D4_hitboxes[0].hit.damage = rw();
    ITEM.x5D4_hitboxes[0].hit.scale = rw();
    ITEM.x5D4_hitboxes[0].hit.b_offset.x = rw();
    ITEM.x5D4_hitboxes[0].hit.b_offset.y = rw();
    ITEM.x5D4_hitboxes[0].hit.b_offset.z = rw();
    ITEM.x5D4_hitboxes[0].hit.x40_b0 = rnd() & 1;
    ITEM.x5D4_hitboxes[0].hit.x40_b1 = rnd() & 1;
    ITEM.x5D4_hitboxes[0].hit.x40_b2 = rnd() & 1;
    ITEM.x5D4_hitboxes[0].hit.x40_b3 = rnd() & 1;
    ITEM.x5D4_hitboxes[0].hit.x41_b4 = rnd() & 1;
    ITEM.x5D4_hitboxes[0].hit.x41_b5 = rnd() & 1;
    ITEM.x5D4_hitboxes[0].hit.x41_b6 = rnd() & 1;
    ITEM.x5D4_hitboxes[0].hit.x41_b7 = rnd() & 1;
    ITEM.x5D4_hitboxes[0].hit.x42_b0 = rnd() & 1;
    ITEM.x5D4_hitboxes[0].hit.x42_b1 = rnd() & 1;
    ITEM.x5D4_hitboxes[0].hit.x42_b2 = rnd() & 1;
    ITEM.x5D4_hitboxes[0].hit.x42_b3 = rnd() & 1;
    ITEM.x5D4_hitboxes[0].hit.x42_b4 = rnd() & 1;
    ITEM.x5D4_hitboxes[0].hit.x42_b5 = rnd() & 1;
    ITEM.x5D4_hitboxes[0].hit.x42_b6 = rnd() & 1;
    ITEM.x5D4_hitboxes[0].hit.x42_b7 = rnd() & 1;
    ITEM.x5D4_hitboxes[0].hit.x43_b0 = rnd() & 1;
    ITEM.x5D4_hitboxes[0].hit.x43_b1 = rnd() & 1;
    ITEM.x5D4_hitboxes[0].hit.x43_b2 = rnd() & 1;
    ITEM.x5D4_hitboxes[0].hit.x43_b3 = rnd() & 1;
    ITEM.x5D4_hitboxes[0].hit.x43_b4 = rnd() & 1;
    ITEM.x5D4_hitboxes[0].hit.x43_b5 = rnd() & 1;
    ITEM.x5D4_hitboxes[0].hit.x43_b6 = rnd() & 1;
    ITEM.x5D4_hitboxes[0].hit.x43_b7 = rnd() & 1;
    ITEM.x5D4_hitboxes[0].hit.x40_b4 = rnd() & 0xFF;
    ITEM.x5D4_hitboxes[0].hit.x44 = (u8) rnd(); ITEM.x5D4_hitboxes[0].hit.x45 = (u8) rnd();
    ITEM.x5D4_hitboxes[0].hit.jobj = (rnd() & 1) ? &BONES[rnd() % 8] : 0;
    ITEM.x5D4_hitboxes[0].x138 = rnd() & 1;
    ITEM.x5D4_hitboxes[1].hit.state = fv();
    ITEM.x5D4_hitboxes[1].hit.x4 = fv();
    ITEM.x5D4_hitboxes[1].hit.unk_count = fv();
    ITEM.x5D4_hitboxes[1].hit.x24 = fv();
    ITEM.x5D4_hitboxes[1].hit.x28 = fv();
    ITEM.x5D4_hitboxes[1].hit.x2C = fv();
    ITEM.x5D4_hitboxes[1].hit.element = fv();
    ITEM.x5D4_hitboxes[1].hit.sfx_kind = fv();
    ITEM.x5D4_hitboxes[1].hit.state = rnd() & 1;
    ITEM.x5D4_hitboxes[1].hit.x4 = rnd() % 8;
    ITEM.x5D4_hitboxes[1].hit.kb_angle = (int) fv();
    ITEM.x5D4_hitboxes[1].hit.x34 = (int) fv();
    ITEM.x5D4_hitboxes[1].hit.sfx_severity = (int) fv();
    ITEM.x5D4_hitboxes[1].hit.damage = rw();
    ITEM.x5D4_hitboxes[1].hit.scale = rw();
    ITEM.x5D4_hitboxes[1].hit.b_offset.x = rw();
    ITEM.x5D4_hitboxes[1].hit.b_offset.y = rw();
    ITEM.x5D4_hitboxes[1].hit.b_offset.z = rw();
    ITEM.x5D4_hitboxes[1].hit.x40_b0 = rnd() & 1;
    ITEM.x5D4_hitboxes[1].hit.x40_b1 = rnd() & 1;
    ITEM.x5D4_hitboxes[1].hit.x40_b2 = rnd() & 1;
    ITEM.x5D4_hitboxes[1].hit.x40_b3 = rnd() & 1;
    ITEM.x5D4_hitboxes[1].hit.x41_b4 = rnd() & 1;
    ITEM.x5D4_hitboxes[1].hit.x41_b5 = rnd() & 1;
    ITEM.x5D4_hitboxes[1].hit.x41_b6 = rnd() & 1;
    ITEM.x5D4_hitboxes[1].hit.x41_b7 = rnd() & 1;
    ITEM.x5D4_hitboxes[1].hit.x42_b0 = rnd() & 1;
    ITEM.x5D4_hitboxes[1].hit.x42_b1 = rnd() & 1;
    ITEM.x5D4_hitboxes[1].hit.x42_b2 = rnd() & 1;
    ITEM.x5D4_hitboxes[1].hit.x42_b3 = rnd() & 1;
    ITEM.x5D4_hitboxes[1].hit.x42_b4 = rnd() & 1;
    ITEM.x5D4_hitboxes[1].hit.x42_b5 = rnd() & 1;
    ITEM.x5D4_hitboxes[1].hit.x42_b6 = rnd() & 1;
    ITEM.x5D4_hitboxes[1].hit.x42_b7 = rnd() & 1;
    ITEM.x5D4_hitboxes[1].hit.x43_b0 = rnd() & 1;
    ITEM.x5D4_hitboxes[1].hit.x43_b1 = rnd() & 1;
    ITEM.x5D4_hitboxes[1].hit.x43_b2 = rnd() & 1;
    ITEM.x5D4_hitboxes[1].hit.x43_b3 = rnd() & 1;
    ITEM.x5D4_hitboxes[1].hit.x43_b4 = rnd() & 1;
    ITEM.x5D4_hitboxes[1].hit.x43_b5 = rnd() & 1;
    ITEM.x5D4_hitboxes[1].hit.x43_b6 = rnd() & 1;
    ITEM.x5D4_hitboxes[1].hit.x43_b7 = rnd() & 1;
    ITEM.x5D4_hitboxes[1].hit.x40_b4 = rnd() & 0xFF;
    ITEM.x5D4_hitboxes[1].hit.x44 = (u8) rnd(); ITEM.x5D4_hitboxes[1].hit.x45 = (u8) rnd();
    ITEM.x5D4_hitboxes[1].hit.jobj = (rnd() & 1) ? &BONES[rnd() % 8] : 0;
    ITEM.x5D4_hitboxes[1].x138 = rnd() & 1;
    ITEM.x5D4_hitboxes[2].hit.state = fv();
    ITEM.x5D4_hitboxes[2].hit.x4 = fv();
    ITEM.x5D4_hitboxes[2].hit.unk_count = fv();
    ITEM.x5D4_hitboxes[2].hit.x24 = fv();
    ITEM.x5D4_hitboxes[2].hit.x28 = fv();
    ITEM.x5D4_hitboxes[2].hit.x2C = fv();
    ITEM.x5D4_hitboxes[2].hit.element = fv();
    ITEM.x5D4_hitboxes[2].hit.sfx_kind = fv();
    ITEM.x5D4_hitboxes[2].hit.state = rnd() & 1;
    ITEM.x5D4_hitboxes[2].hit.x4 = rnd() % 8;
    ITEM.x5D4_hitboxes[2].hit.kb_angle = (int) fv();
    ITEM.x5D4_hitboxes[2].hit.x34 = (int) fv();
    ITEM.x5D4_hitboxes[2].hit.sfx_severity = (int) fv();
    ITEM.x5D4_hitboxes[2].hit.damage = rw();
    ITEM.x5D4_hitboxes[2].hit.scale = rw();
    ITEM.x5D4_hitboxes[2].hit.b_offset.x = rw();
    ITEM.x5D4_hitboxes[2].hit.b_offset.y = rw();
    ITEM.x5D4_hitboxes[2].hit.b_offset.z = rw();
    ITEM.x5D4_hitboxes[2].hit.x40_b0 = rnd() & 1;
    ITEM.x5D4_hitboxes[2].hit.x40_b1 = rnd() & 1;
    ITEM.x5D4_hitboxes[2].hit.x40_b2 = rnd() & 1;
    ITEM.x5D4_hitboxes[2].hit.x40_b3 = rnd() & 1;
    ITEM.x5D4_hitboxes[2].hit.x41_b4 = rnd() & 1;
    ITEM.x5D4_hitboxes[2].hit.x41_b5 = rnd() & 1;
    ITEM.x5D4_hitboxes[2].hit.x41_b6 = rnd() & 1;
    ITEM.x5D4_hitboxes[2].hit.x41_b7 = rnd() & 1;
    ITEM.x5D4_hitboxes[2].hit.x42_b0 = rnd() & 1;
    ITEM.x5D4_hitboxes[2].hit.x42_b1 = rnd() & 1;
    ITEM.x5D4_hitboxes[2].hit.x42_b2 = rnd() & 1;
    ITEM.x5D4_hitboxes[2].hit.x42_b3 = rnd() & 1;
    ITEM.x5D4_hitboxes[2].hit.x42_b4 = rnd() & 1;
    ITEM.x5D4_hitboxes[2].hit.x42_b5 = rnd() & 1;
    ITEM.x5D4_hitboxes[2].hit.x42_b6 = rnd() & 1;
    ITEM.x5D4_hitboxes[2].hit.x42_b7 = rnd() & 1;
    ITEM.x5D4_hitboxes[2].hit.x43_b0 = rnd() & 1;
    ITEM.x5D4_hitboxes[2].hit.x43_b1 = rnd() & 1;
    ITEM.x5D4_hitboxes[2].hit.x43_b2 = rnd() & 1;
    ITEM.x5D4_hitboxes[2].hit.x43_b3 = rnd() & 1;
    ITEM.x5D4_hitboxes[2].hit.x43_b4 = rnd() & 1;
    ITEM.x5D4_hitboxes[2].hit.x43_b5 = rnd() & 1;
    ITEM.x5D4_hitboxes[2].hit.x43_b6 = rnd() & 1;
    ITEM.x5D4_hitboxes[2].hit.x43_b7 = rnd() & 1;
    ITEM.x5D4_hitboxes[2].hit.x40_b4 = rnd() & 0xFF;
    ITEM.x5D4_hitboxes[2].hit.x44 = (u8) rnd(); ITEM.x5D4_hitboxes[2].hit.x45 = (u8) rnd();
    ITEM.x5D4_hitboxes[2].hit.jobj = (rnd() & 1) ? &BONES[rnd() % 8] : 0;
    ITEM.x5D4_hitboxes[2].x138 = rnd() & 1;
    ITEM.x5D4_hitboxes[3].hit.state = fv();
    ITEM.x5D4_hitboxes[3].hit.x4 = fv();
    ITEM.x5D4_hitboxes[3].hit.unk_count = fv();
    ITEM.x5D4_hitboxes[3].hit.x24 = fv();
    ITEM.x5D4_hitboxes[3].hit.x28 = fv();
    ITEM.x5D4_hitboxes[3].hit.x2C = fv();
    ITEM.x5D4_hitboxes[3].hit.element = fv();
    ITEM.x5D4_hitboxes[3].hit.sfx_kind = fv();
    ITEM.x5D4_hitboxes[3].hit.state = rnd() & 1;
    ITEM.x5D4_hitboxes[3].hit.x4 = rnd() % 8;
    ITEM.x5D4_hitboxes[3].hit.kb_angle = (int) fv();
    ITEM.x5D4_hitboxes[3].hit.x34 = (int) fv();
    ITEM.x5D4_hitboxes[3].hit.sfx_severity = (int) fv();
    ITEM.x5D4_hitboxes[3].hit.damage = rw();
    ITEM.x5D4_hitboxes[3].hit.scale = rw();
    ITEM.x5D4_hitboxes[3].hit.b_offset.x = rw();
    ITEM.x5D4_hitboxes[3].hit.b_offset.y = rw();
    ITEM.x5D4_hitboxes[3].hit.b_offset.z = rw();
    ITEM.x5D4_hitboxes[3].hit.x40_b0 = rnd() & 1;
    ITEM.x5D4_hitboxes[3].hit.x40_b1 = rnd() & 1;
    ITEM.x5D4_hitboxes[3].hit.x40_b2 = rnd() & 1;
    ITEM.x5D4_hitboxes[3].hit.x40_b3 = rnd() & 1;
    ITEM.x5D4_hitboxes[3].hit.x41_b4 = rnd() & 1;
    ITEM.x5D4_hitboxes[3].hit.x41_b5 = rnd() & 1;
    ITEM.x5D4_hitboxes[3].hit.x41_b6 = rnd() & 1;
    ITEM.x5D4_hitboxes[3].hit.x41_b7 = rnd() & 1;
    ITEM.x5D4_hitboxes[3].hit.x42_b0 = rnd() & 1;
    ITEM.x5D4_hitboxes[3].hit.x42_b1 = rnd() & 1;
    ITEM.x5D4_hitboxes[3].hit.x42_b2 = rnd() & 1;
    ITEM.x5D4_hitboxes[3].hit.x42_b3 = rnd() & 1;
    ITEM.x5D4_hitboxes[3].hit.x42_b4 = rnd() & 1;
    ITEM.x5D4_hitboxes[3].hit.x42_b5 = rnd() & 1;
    ITEM.x5D4_hitboxes[3].hit.x42_b6 = rnd() & 1;
    ITEM.x5D4_hitboxes[3].hit.x42_b7 = rnd() & 1;
    ITEM.x5D4_hitboxes[3].hit.x43_b0 = rnd() & 1;
    ITEM.x5D4_hitboxes[3].hit.x43_b1 = rnd() & 1;
    ITEM.x5D4_hitboxes[3].hit.x43_b2 = rnd() & 1;
    ITEM.x5D4_hitboxes[3].hit.x43_b3 = rnd() & 1;
    ITEM.x5D4_hitboxes[3].hit.x43_b4 = rnd() & 1;
    ITEM.x5D4_hitboxes[3].hit.x43_b5 = rnd() & 1;
    ITEM.x5D4_hitboxes[3].hit.x43_b6 = rnd() & 1;
    ITEM.x5D4_hitboxes[3].hit.x43_b7 = rnd() & 1;
    ITEM.x5D4_hitboxes[3].hit.x40_b4 = rnd() & 0xFF;
    ITEM.x5D4_hitboxes[3].hit.x44 = (u8) rnd(); ITEM.x5D4_hitboxes[3].hit.x45 = (u8) rnd();
    ITEM.x5D4_hitboxes[3].hit.jobj = (rnd() & 1) ? &BONES[rnd() % 8] : 0;
    ITEM.x5D4_hitboxes[3].x138 = rnd() & 1;
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
    HM((u32) ITEM.x5D4_hitboxes[0].hit.state);
    HM((u32) ITEM.x5D4_hitboxes[0].hit.x4);
    HM((u32) ITEM.x5D4_hitboxes[0].hit.unk_count);
    HM((u32) ITEM.x5D4_hitboxes[0].hit.x24);
    HM((u32) ITEM.x5D4_hitboxes[0].hit.x28);
    HM((u32) ITEM.x5D4_hitboxes[0].hit.x2C);
    HM((u32) ITEM.x5D4_hitboxes[0].hit.element);
    HM((u32) ITEM.x5D4_hitboxes[0].hit.sfx_kind);
    HM((u32) ITEM.x5D4_hitboxes[0].hit.kb_angle);
    HM((u32) ITEM.x5D4_hitboxes[0].hit.x34);
    HM((u32) ITEM.x5D4_hitboxes[0].hit.sfx_severity);
    HM(B(ITEM.x5D4_hitboxes[0].hit.damage));
    HM(B(ITEM.x5D4_hitboxes[0].hit.scale));
    HM(B(ITEM.x5D4_hitboxes[0].hit.b_offset.x));
    HM(B(ITEM.x5D4_hitboxes[0].hit.b_offset.y));
    HM(B(ITEM.x5D4_hitboxes[0].hit.b_offset.z));
    HM((u32) ITEM.x5D4_hitboxes[0].hit.x40_b0);
    HM((u32) ITEM.x5D4_hitboxes[0].hit.x40_b1);
    HM((u32) ITEM.x5D4_hitboxes[0].hit.x40_b2);
    HM((u32) ITEM.x5D4_hitboxes[0].hit.x40_b3);
    HM((u32) ITEM.x5D4_hitboxes[0].hit.x41_b4);
    HM((u32) ITEM.x5D4_hitboxes[0].hit.x41_b5);
    HM((u32) ITEM.x5D4_hitboxes[0].hit.x41_b6);
    HM((u32) ITEM.x5D4_hitboxes[0].hit.x41_b7);
    HM((u32) ITEM.x5D4_hitboxes[0].hit.x42_b0);
    HM((u32) ITEM.x5D4_hitboxes[0].hit.x42_b1);
    HM((u32) ITEM.x5D4_hitboxes[0].hit.x42_b2);
    HM((u32) ITEM.x5D4_hitboxes[0].hit.x42_b3);
    HM((u32) ITEM.x5D4_hitboxes[0].hit.x42_b4);
    HM((u32) ITEM.x5D4_hitboxes[0].hit.x42_b5);
    HM((u32) ITEM.x5D4_hitboxes[0].hit.x42_b6);
    HM((u32) ITEM.x5D4_hitboxes[0].hit.x42_b7);
    HM((u32) ITEM.x5D4_hitboxes[0].hit.x43_b0);
    HM((u32) ITEM.x5D4_hitboxes[0].hit.x43_b1);
    HM((u32) ITEM.x5D4_hitboxes[0].hit.x43_b2);
    HM((u32) ITEM.x5D4_hitboxes[0].hit.x43_b3);
    HM((u32) ITEM.x5D4_hitboxes[0].hit.x43_b4);
    HM((u32) ITEM.x5D4_hitboxes[0].hit.x43_b5);
    HM((u32) ITEM.x5D4_hitboxes[0].hit.x43_b6);
    HM((u32) ITEM.x5D4_hitboxes[0].hit.x43_b7);
    HM((u32) ITEM.x5D4_hitboxes[0].hit.x40_b4);
    HM(ITEM.x5D4_hitboxes[0].hit.x44); HM(ITEM.x5D4_hitboxes[0].hit.x45); HM(PI(ITEM.x5D4_hitboxes[0].hit.jobj)); HM((u32) ITEM.x5D4_hitboxes[0].x138);
    HM((u32) ITEM.x5D4_hitboxes[1].hit.state);
    HM((u32) ITEM.x5D4_hitboxes[1].hit.x4);
    HM((u32) ITEM.x5D4_hitboxes[1].hit.unk_count);
    HM((u32) ITEM.x5D4_hitboxes[1].hit.x24);
    HM((u32) ITEM.x5D4_hitboxes[1].hit.x28);
    HM((u32) ITEM.x5D4_hitboxes[1].hit.x2C);
    HM((u32) ITEM.x5D4_hitboxes[1].hit.element);
    HM((u32) ITEM.x5D4_hitboxes[1].hit.sfx_kind);
    HM((u32) ITEM.x5D4_hitboxes[1].hit.kb_angle);
    HM((u32) ITEM.x5D4_hitboxes[1].hit.x34);
    HM((u32) ITEM.x5D4_hitboxes[1].hit.sfx_severity);
    HM(B(ITEM.x5D4_hitboxes[1].hit.damage));
    HM(B(ITEM.x5D4_hitboxes[1].hit.scale));
    HM(B(ITEM.x5D4_hitboxes[1].hit.b_offset.x));
    HM(B(ITEM.x5D4_hitboxes[1].hit.b_offset.y));
    HM(B(ITEM.x5D4_hitboxes[1].hit.b_offset.z));
    HM((u32) ITEM.x5D4_hitboxes[1].hit.x40_b0);
    HM((u32) ITEM.x5D4_hitboxes[1].hit.x40_b1);
    HM((u32) ITEM.x5D4_hitboxes[1].hit.x40_b2);
    HM((u32) ITEM.x5D4_hitboxes[1].hit.x40_b3);
    HM((u32) ITEM.x5D4_hitboxes[1].hit.x41_b4);
    HM((u32) ITEM.x5D4_hitboxes[1].hit.x41_b5);
    HM((u32) ITEM.x5D4_hitboxes[1].hit.x41_b6);
    HM((u32) ITEM.x5D4_hitboxes[1].hit.x41_b7);
    HM((u32) ITEM.x5D4_hitboxes[1].hit.x42_b0);
    HM((u32) ITEM.x5D4_hitboxes[1].hit.x42_b1);
    HM((u32) ITEM.x5D4_hitboxes[1].hit.x42_b2);
    HM((u32) ITEM.x5D4_hitboxes[1].hit.x42_b3);
    HM((u32) ITEM.x5D4_hitboxes[1].hit.x42_b4);
    HM((u32) ITEM.x5D4_hitboxes[1].hit.x42_b5);
    HM((u32) ITEM.x5D4_hitboxes[1].hit.x42_b6);
    HM((u32) ITEM.x5D4_hitboxes[1].hit.x42_b7);
    HM((u32) ITEM.x5D4_hitboxes[1].hit.x43_b0);
    HM((u32) ITEM.x5D4_hitboxes[1].hit.x43_b1);
    HM((u32) ITEM.x5D4_hitboxes[1].hit.x43_b2);
    HM((u32) ITEM.x5D4_hitboxes[1].hit.x43_b3);
    HM((u32) ITEM.x5D4_hitboxes[1].hit.x43_b4);
    HM((u32) ITEM.x5D4_hitboxes[1].hit.x43_b5);
    HM((u32) ITEM.x5D4_hitboxes[1].hit.x43_b6);
    HM((u32) ITEM.x5D4_hitboxes[1].hit.x43_b7);
    HM((u32) ITEM.x5D4_hitboxes[1].hit.x40_b4);
    HM(ITEM.x5D4_hitboxes[1].hit.x44); HM(ITEM.x5D4_hitboxes[1].hit.x45); HM(PI(ITEM.x5D4_hitboxes[1].hit.jobj)); HM((u32) ITEM.x5D4_hitboxes[1].x138);
    HM((u32) ITEM.x5D4_hitboxes[2].hit.state);
    HM((u32) ITEM.x5D4_hitboxes[2].hit.x4);
    HM((u32) ITEM.x5D4_hitboxes[2].hit.unk_count);
    HM((u32) ITEM.x5D4_hitboxes[2].hit.x24);
    HM((u32) ITEM.x5D4_hitboxes[2].hit.x28);
    HM((u32) ITEM.x5D4_hitboxes[2].hit.x2C);
    HM((u32) ITEM.x5D4_hitboxes[2].hit.element);
    HM((u32) ITEM.x5D4_hitboxes[2].hit.sfx_kind);
    HM((u32) ITEM.x5D4_hitboxes[2].hit.kb_angle);
    HM((u32) ITEM.x5D4_hitboxes[2].hit.x34);
    HM((u32) ITEM.x5D4_hitboxes[2].hit.sfx_severity);
    HM(B(ITEM.x5D4_hitboxes[2].hit.damage));
    HM(B(ITEM.x5D4_hitboxes[2].hit.scale));
    HM(B(ITEM.x5D4_hitboxes[2].hit.b_offset.x));
    HM(B(ITEM.x5D4_hitboxes[2].hit.b_offset.y));
    HM(B(ITEM.x5D4_hitboxes[2].hit.b_offset.z));
    HM((u32) ITEM.x5D4_hitboxes[2].hit.x40_b0);
    HM((u32) ITEM.x5D4_hitboxes[2].hit.x40_b1);
    HM((u32) ITEM.x5D4_hitboxes[2].hit.x40_b2);
    HM((u32) ITEM.x5D4_hitboxes[2].hit.x40_b3);
    HM((u32) ITEM.x5D4_hitboxes[2].hit.x41_b4);
    HM((u32) ITEM.x5D4_hitboxes[2].hit.x41_b5);
    HM((u32) ITEM.x5D4_hitboxes[2].hit.x41_b6);
    HM((u32) ITEM.x5D4_hitboxes[2].hit.x41_b7);
    HM((u32) ITEM.x5D4_hitboxes[2].hit.x42_b0);
    HM((u32) ITEM.x5D4_hitboxes[2].hit.x42_b1);
    HM((u32) ITEM.x5D4_hitboxes[2].hit.x42_b2);
    HM((u32) ITEM.x5D4_hitboxes[2].hit.x42_b3);
    HM((u32) ITEM.x5D4_hitboxes[2].hit.x42_b4);
    HM((u32) ITEM.x5D4_hitboxes[2].hit.x42_b5);
    HM((u32) ITEM.x5D4_hitboxes[2].hit.x42_b6);
    HM((u32) ITEM.x5D4_hitboxes[2].hit.x42_b7);
    HM((u32) ITEM.x5D4_hitboxes[2].hit.x43_b0);
    HM((u32) ITEM.x5D4_hitboxes[2].hit.x43_b1);
    HM((u32) ITEM.x5D4_hitboxes[2].hit.x43_b2);
    HM((u32) ITEM.x5D4_hitboxes[2].hit.x43_b3);
    HM((u32) ITEM.x5D4_hitboxes[2].hit.x43_b4);
    HM((u32) ITEM.x5D4_hitboxes[2].hit.x43_b5);
    HM((u32) ITEM.x5D4_hitboxes[2].hit.x43_b6);
    HM((u32) ITEM.x5D4_hitboxes[2].hit.x43_b7);
    HM((u32) ITEM.x5D4_hitboxes[2].hit.x40_b4);
    HM(ITEM.x5D4_hitboxes[2].hit.x44); HM(ITEM.x5D4_hitboxes[2].hit.x45); HM(PI(ITEM.x5D4_hitboxes[2].hit.jobj)); HM((u32) ITEM.x5D4_hitboxes[2].x138);
    HM((u32) ITEM.x5D4_hitboxes[3].hit.state);
    HM((u32) ITEM.x5D4_hitboxes[3].hit.x4);
    HM((u32) ITEM.x5D4_hitboxes[3].hit.unk_count);
    HM((u32) ITEM.x5D4_hitboxes[3].hit.x24);
    HM((u32) ITEM.x5D4_hitboxes[3].hit.x28);
    HM((u32) ITEM.x5D4_hitboxes[3].hit.x2C);
    HM((u32) ITEM.x5D4_hitboxes[3].hit.element);
    HM((u32) ITEM.x5D4_hitboxes[3].hit.sfx_kind);
    HM((u32) ITEM.x5D4_hitboxes[3].hit.kb_angle);
    HM((u32) ITEM.x5D4_hitboxes[3].hit.x34);
    HM((u32) ITEM.x5D4_hitboxes[3].hit.sfx_severity);
    HM(B(ITEM.x5D4_hitboxes[3].hit.damage));
    HM(B(ITEM.x5D4_hitboxes[3].hit.scale));
    HM(B(ITEM.x5D4_hitboxes[3].hit.b_offset.x));
    HM(B(ITEM.x5D4_hitboxes[3].hit.b_offset.y));
    HM(B(ITEM.x5D4_hitboxes[3].hit.b_offset.z));
    HM((u32) ITEM.x5D4_hitboxes[3].hit.x40_b0);
    HM((u32) ITEM.x5D4_hitboxes[3].hit.x40_b1);
    HM((u32) ITEM.x5D4_hitboxes[3].hit.x40_b2);
    HM((u32) ITEM.x5D4_hitboxes[3].hit.x40_b3);
    HM((u32) ITEM.x5D4_hitboxes[3].hit.x41_b4);
    HM((u32) ITEM.x5D4_hitboxes[3].hit.x41_b5);
    HM((u32) ITEM.x5D4_hitboxes[3].hit.x41_b6);
    HM((u32) ITEM.x5D4_hitboxes[3].hit.x41_b7);
    HM((u32) ITEM.x5D4_hitboxes[3].hit.x42_b0);
    HM((u32) ITEM.x5D4_hitboxes[3].hit.x42_b1);
    HM((u32) ITEM.x5D4_hitboxes[3].hit.x42_b2);
    HM((u32) ITEM.x5D4_hitboxes[3].hit.x42_b3);
    HM((u32) ITEM.x5D4_hitboxes[3].hit.x42_b4);
    HM((u32) ITEM.x5D4_hitboxes[3].hit.x42_b5);
    HM((u32) ITEM.x5D4_hitboxes[3].hit.x42_b6);
    HM((u32) ITEM.x5D4_hitboxes[3].hit.x42_b7);
    HM((u32) ITEM.x5D4_hitboxes[3].hit.x43_b0);
    HM((u32) ITEM.x5D4_hitboxes[3].hit.x43_b1);
    HM((u32) ITEM.x5D4_hitboxes[3].hit.x43_b2);
    HM((u32) ITEM.x5D4_hitboxes[3].hit.x43_b3);
    HM((u32) ITEM.x5D4_hitboxes[3].hit.x43_b4);
    HM((u32) ITEM.x5D4_hitboxes[3].hit.x43_b5);
    HM((u32) ITEM.x5D4_hitboxes[3].hit.x43_b6);
    HM((u32) ITEM.x5D4_hitboxes[3].hit.x43_b7);
    HM((u32) ITEM.x5D4_hitboxes[3].hit.x40_b4);
    HM(ITEM.x5D4_hitboxes[3].hit.x44); HM(ITEM.x5D4_hitboxes[3].hit.x45); HM(PI(ITEM.x5D4_hitboxes[3].hit.jobj)); HM((u32) ITEM.x5D4_hitboxes[3].x138);
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
