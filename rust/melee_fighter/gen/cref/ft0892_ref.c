/* C reference for src/ft0892.rs + src/pltrick.rs (port round 7): all of ft_0892.c, the
 * attack-stats functions of pltrick.c (pl_803BCE70 .. pl_80037ECC, pl_800384DC, pl_80038628),
 * plAttack_80037B08 and Player_GetActionStats, cut verbatim into ft0892_extract.inc by run.py.
 * No fused sites in either file. One fighter, random fields and action stats, one op per case.
 * Callees outside the set (it_8026B6C8, ftFx_AppealS_CheckIfUsed, pl_8003FE40, pl_8003DFF4,
 * ftData_80085FD4, Geno) are stubs folding their arguments into the trace word g_tr;
 * tests/ft0892_ref.rs installs the same stubs as hooks.
 * Byte order: an int literal stored into union Struct2070 (ft_800895E0's Luigi / item paths)
 * is laid out little-endian here but big-endian in the game; the driver prints a tag for those
 * two values instead of the fields, and the Rust test maps its (big-endian) result to the tag. */
#include "ft/types.h"
#include "ft/forward.h"
#include "ft/fighter.h"
#include "ft/ft_0892.h"
#include "ft/ftdata.h"
#include "ft/inlines.h"
#include "ft/kinds/ftFox/ftfoxappeals.h"
#include <melee/it/it_26B1.h>
#include <melee/pl/plattack.h>
#include <melee/pl/player.h>
#include <melee/pl/pltrick.h>
#include <melee/pl/plbonuslib.h>
#include <sysdolphin/baselib/gobj.h>

static u32 g_tr;
static void T(u32 v) { g_tr = g_tr * 31u + v; }
static u32 B(float f) { union { float f; u32 u; } c; c.f = f; return c.u; }

StaticPlayer player_slots[6];
u16 unk_804D6480;
void __assert(const char* file, u32 line, const char* cond) { (void) file; (void) line; (void) cond; for (;;) {} }
void OSReport(char* fmt, ...) { (void) fmt; }

static Fighter F;
static HSD_GObj GO, IT;
static struct ftData_80085FD4_ret FD;

s32 it_8026B6C8(Item_GObj* g) { T(0x500u + (g == &IT)); return (s32) ((g_tr >> 3) % 3u); }
bool ftFx_AppealS_CheckIfUsed(Fighter* fp) { T(0x510u + (fp == &F)); return (g_tr >> 4) & 1; }
void pl_8003FE40(int a, int b) { T(0x520u); T((u32) a); T((u32) b); }
void pl_8003DFF4(int a, int b, int c) { T(0x530u); T((u32) a); T((u32) b); T((u32) c); }
struct ftData_80085FD4_ret* ftData_80085FD4(Fighter* fp, FtMotionId m)
{
    T(0x540u + (fp == &F)); T((u32) m);
    FD.x8 = (g_tr >> 5) & 1;
    return &FD;
}
int Geno_DefineTemplateSkeleton(int k) { T(0x550u + (u32) k); return (g_tr >> 2) & 1; }
int Geno_DefineBaseKind(int k) { T(0x560u + (u32) k); return (g_tr >> 6) & 1 ? 1 : 22; }

#include "ft0892_extract.inc"

int printf(const char*, ...);
static void zero(void* p, unsigned n) { unsigned char* b = p; while (n--) *b++ = 0; }

static u32 rs;
static u32 rnd(void) { rs ^= rs << 13; rs ^= rs >> 17; rs ^= rs << 5; return rs; }

static const int KINDS[8] = { 1, 22, 16, 17, 33, 34, 2, 9 };
#define NOPS 19

/* x2070 field 1..0x30: never 0 or 0x63/0x3D, so random fields never read as the two literals */
static void r2070(union Struct2070* u)
{
    u->x0.x2070 = (s8) (1 + rnd() % 0x30);
    u->x0.x2071_b0_3 = rnd() & 15; u->x0.x2071_b4 = rnd() & 1; u->x0.x2071_b5 = rnd() & 1;
    u->x0.x2071_b6 = rnd() & 1; u->x0.x2071_b7 = rnd() & 1; u->x0.x2072_b0 = rnd() & 1;
    u->x0.x2072_b1 = rnd() & 1; u->x0.x2072_b2 = rnd() & 1; u->x0.count_x1A4 = rnd() & 1;
    u->x0.count_thrown_items = rnd() & 1; u->x0.count_aerials = rnd() & 1; u->x0.count_x1A0 = rnd() & 1;
    u->x0.count_specials = rnd() & 1;
    switch (rnd() % 6) {
    case 0: u->x0.x2073 = 0; break;
    case 1: u->x0.x2073 = (u8) (0x60 + rnd() % 0x14); break; /* 0x60..0x73: 0x62/0x63/0x71/0x72 */
    case 2: u->x0.x2073 = (u8) (100 + rnd() % 16); break;    /* by_attack_hi, u32 fields */
    default: u->x0.x2073 = (u8) (rnd() % 100); break;
    }
}
static void r18d8(ft_800898B4_t* e)
{
    e->x0 = (int) rnd(); e->x4 = (int) rnd(); e->kb_applied1 = (float) (int) (rnd() % 2001) / 7.0f;
    e->xC = (int) (rnd() % 20);
    e->x10_b0 = rnd() & 1; e->x10_b1 = rnd() & 1; e->x10_b2 = rnd() & 1; e->x10_b3 = rnd() & 1;
    e->x10_b4 = rnd() & 1; e->x10_b5 = rnd() & 1; e->x10_b6 = rnd() & 1; e->x10_b7 = rnd() & 1;
    e->x11_b0 = rnd() & 1; e->x11_b1 = rnd() & 1; e->x11_b2 = rnd() & 1; e->x11_b3 = rnd() & 1;
    e->x11_b4 = rnd() & 1;
}
static u32 h2070(u32 t, union Struct2070* u)
{
    t = t * 31u + (u32) (u8) u->x0.x2070; t = t * 31u + u->x0.x2071_b0_3; t = t * 31u + u->x0.x2071_b4;
    t = t * 31u + u->x0.x2071_b5; t = t * 31u + u->x0.x2071_b6; t = t * 31u + u->x0.x2071_b7;
    t = t * 31u + u->x0.x2072_b0; t = t * 31u + u->x0.x2072_b1; t = t * 31u + u->x0.x2072_b2;
    t = t * 31u + u->x0.count_x1A4; t = t * 31u + u->x0.count_thrown_items; t = t * 31u + u->x0.count_aerials;
    t = t * 31u + u->x0.count_x1A0; t = t * 31u + u->x0.count_specials; t = t * 31u + u->x0.x2073;
    return t;
}
static u32 h18d8(u32 t, ft_800898B4_t* e)
{
    t = t * 31u + (u32) e->x0; t = t * 31u + (u32) e->x4; t = t * 31u + B(e->kb_applied1);
    t = t * 31u + (u32) e->xC;
    t = t * 31u + e->x10_b0; t = t * 31u + e->x10_b1; t = t * 31u + e->x10_b2; t = t * 31u + e->x10_b3;
    t = t * 31u + e->x10_b4; t = t * 31u + e->x10_b5; t = t * 31u + e->x10_b6; t = t * 31u + e->x10_b7;
    t = t * 31u + e->x11_b0; t = t * 31u + e->x11_b1; t = t * 31u + e->x11_b2; t = t * 31u + e->x11_b3;
    t = t * 31u + e->x11_b4;
    return t;
}
static u32 hatk(u32 t, struct plAttackStats* a)
{
    int i;
    t = t * 31u + a->total;
    for (i = 0; i < 100; i++) t = t * 31u + a->by_attack_counts[i];
    t = t * 31u + a->thrown_item_count; t = t * 31u + a->aerials_count; t = t * 31u + a->specials_count;
    t = t * 31u + a->x1A0_count; t = t * 31u + a->x1A4_count; t = t * 31u + a->x1A8;
    return t;
}
static void ratk(struct plAttackStats* a)
{
    int i;
    a->total = rnd() % 50;
    for (i = 0; i < 100; i++) a->by_attack_counts[i] = rnd() % 4;
    a->thrown_item_count = rnd() % 9; a->aerials_count = rnd() % 9; a->specials_count = rnd() % 9;
    a->x1A0_count = rnd() % 9; a->x1A4_count = rnd() % 9; a->x1A8 = rnd() % 9;
}
static u32 hstats(u32 t, plActionStats* s)
{
    int i;
    t = hatk(t, &s->attacks); t = hatk(t, &s->hits); t = hatk(t, &s->x358.x358_hits);
    for (i = 0; i < 100; i++) t = t * 31u + s->x504[i];
    t = t * 31u + s->x568; t = t * 31u + s->x56C; t = t * 31u + s->x570; t = t * 31u + s->x574;
    t = t * 31u + s->x578; t = t * 31u + s->x57C; t = t * 31u + s->x580; t = t * 31u + s->x584;
    t = t * 31u + s->x588; t = t * 31u + s->x58C; t = t * 31u + s->x590; t = t * 31u + s->x594;
    for (i = 0; i < 8; i++) t = t * 31u + s->x598[i];
    t = t * 31u + s->x5BC_b0; t = t * 31u + s->x5BC_b1; t = t * 31u + s->x5BC_b2; t = t * 31u + s->x5BC_b3;
    return t;
}

int main(void)
{
    int op, k, i;
    for (op = 0; op < NOPS; op++) {
        for (k = 0; k < 300; k++) {
            u32 t = 0, ret = 0;
            Fighter* f = &F;
            plActionStats* st;
            union Struct2070 arg;
            ft_800898B4_t evd;
            zero(&F, sizeof F); zero(&GO, sizeof GO); zero(&IT, sizeof IT);
            zero(player_slots, sizeof player_slots);
            rs = 0x2468Bu + (u32) op * 7919u + (u32) k * 104729u;
            g_tr = rnd();
            unk_804D6480 = (u16) ((rnd() % 4) == 0 ? 0xFFFF : rnd());
            GO.user_data = f;
            f->gobj = &GO;
            f->kind = KINDS[rnd() % 8];
            f->player_idx = (u8) (rnd() % 6);
            f->is_sub_fighter = rnd() & 1;
            f->is_metal = rnd() & 1;
            f->x221D_b6 = rnd() & 1; f->x2226_b4 = rnd() & 1;
            f->x2220_b5 = rnd() & 1; f->x2220_b6 = rnd() & 1;
            f->x197C = (rnd() & 1) ? &IT : NULL;
            f->item_gobj = (rnd() & 1) ? &IT : NULL;
            f->motion_id = (int) (rnd() % 400);
            f->dmg.x1830_percent = (float) (int) (rnd() % 4001) / 13.0f;
            r2070(&f->x2070);
            *(u32*) &f->x2074.x2074_vec.x = rnd(); *(u32*) &f->x2074.x2074_vec.y = rnd();
            *(u32*) &f->x2074.x207C.x = rnd(); f->x2074.x207C.y = (int) rnd();
            f->x2074.x10.x0.x2084_b0 = rnd() & 1; f->x2074.x10.x0.x2084_b1 = rnd() & 1; f->x2074.x10.x0.x2084_b2 = rnd() & 1;
            f->x2074.x10.x0.x2084_b3 = rnd() & 1; f->x2074.x10.x0.x2084_b4 = rnd() & 1; f->x2074.x10.x0.x2084_b5 = rnd() & 1;
            f->x2074.x10.x0.x2084_b6 = rnd() & 1; f->x2074.x10.x0.x2084_b7 = rnd() & 1; f->x2074.x10.x0.x2085_b0 = rnd() & 1;
            f->x2074.x10.x0.x2085_b1 = rnd() & 1; f->x2074.x10.x0.x2085_b2 = rnd() & 1; f->x2074.x10.x0.x2085_b3 = rnd() & 1;
            f->x2074.x10.x0.x2085_b4 = rnd() & 1; f->x2074.x10.x0.x2085_b5 = rnd() & 1; f->x2074.x10.x0.x2085_b6 = rnd() & 1;
            f->x2074.x10.x0.x2085_b7 = rnd() & 1;
            f->x2074.x2088 = (u16) rnd();
            f->dmg.x18d4.x0.x2070 = (s8) rnd(); ((u8*) &f->dmg.x18d4)[1] = (u8) rnd();
            f->dmg.x18d4.x0.x2072_b0 = rnd() & 1; f->dmg.x18d4.x0.count_specials = rnd() & 1;
            f->dmg.x18d4.x0.x2073 = (u8) rnd();
            r18d8(&f->dmg.x18d8);
            f->dmg.x18ec_instancehitby = (u16) rnd();
            st = Player_GetActionStats(f->player_idx);
            ratk(&st->attacks); ratk(&st->hits); ratk(&st->x358.x358_hits);
            for (i = 0; i < 100; i++) st->x504[i] = (u8) rnd();
            st->x5BC_b0 = rnd() & 1; st->x5BC_b1 = rnd() & 1; st->x5BC_b2 = rnd() & 1; st->x5BC_b3 = rnd() & 1;
            r2070(&arg);
            if ((rnd() % 3) == 0) arg.x0.x2073 = f->x2070.x0.x2073;
            if ((rnd() % 4) == 0) { u32 q = rnd() % 3; arg.x0.x2073 = q == 0 ? 0x71 : q == 1 ? 0x62 : 0x63; }
            r18d8(&evd);
            switch (op) {
            case 0: ft_800892D4(f); break;
            case 1: ft_80089460(f); break;
            case 2: ft_800895E0(f, arg.x2070_int); break;
            case 3: ft_80089768(&f->x2074.x2074_vec); break;
            case 4: ft_80089824(&GO); break;
            case 5: ret = (void*) ft_80089884(&GO) == (void*) &f->x2070; break;
            case 6: ret = (void*) ft_80089890(&GO) == (void*) &f->dmg.x18d4; break;
            case 7: ret = ft_8008989C(&GO); break;
            case 8: ret = (void*) ft_800898A8(&GO) == (void*) &f->x2074; break;
            case 9: ret = (void*) ft_800898B4(&GO) == (void*) &f->dmg.x18d8; break;
            case 10: ret = ft_800898C0(&GO); break;
            case 11: { int ms = 45 + (int) (rnd() % 22); ret = (u32) ft_80089914(&GO, ms); break; }
            case 12: { int hp = (int) (rnd() % 8); int idx = (int) (rnd() % 100); ret = (u32) pl_80037B2C(st, hp, idx); break; }
            case 13: pl_80037BC0((rnd() & 1) ? &st->attacks : &st->hits, &arg); break;
            case 14: pl_80037C60(&GO, arg.x2070_int); break;
            case 15: pl_80037DF4(&GO, &arg); break;
            case 16: pl_80037ECC(&GO); break;
            case 17: pl_800384DC(&GO, arg.x2070_int, &evd); break;
            case 18: { int kd = 1 + (int) (rnd() % 16); ret = pl_80038628(&GO, kd); break; }
            }
            if (f->x2070.x2070_int == 0x240063) t = t * 31u + 0xA1u;
            else if (f->x2070.x2070_int == 0x44003D) t = t * 31u + 0xA2u;
            else t = h2070(t, &f->x2070);
            t = t * 31u + (u32) f->x2074.x2074_vec.x; t = t * 31u + (u32) f->x2074.x2074_vec.y;
            t = t * 31u + B(f->x2074.x207C.x); t = t * 31u + (u32) f->x2074.x207C.y;
            t = t * 31u + f->x2074.x10.x0.x2084_b0; t = t * 31u + f->x2074.x10.x0.x2084_b1; t = t * 31u + f->x2074.x10.x0.x2084_b2;
            t = t * 31u + f->x2074.x10.x0.x2084_b3; t = t * 31u + f->x2074.x10.x0.x2084_b4; t = t * 31u + f->x2074.x10.x0.x2084_b5;
            t = t * 31u + f->x2074.x10.x0.x2084_b6; t = t * 31u + f->x2074.x10.x0.x2084_b7; t = t * 31u + f->x2074.x10.x0.x2085_b0;
            t = t * 31u + f->x2074.x10.x0.x2085_b1; t = t * 31u + f->x2074.x10.x0.x2085_b2; t = t * 31u + f->x2074.x10.x0.x2085_b3;
            t = t * 31u + f->x2074.x10.x0.x2085_b4; t = t * 31u + f->x2074.x10.x0.x2085_b5; t = t * 31u + f->x2074.x10.x0.x2085_b6;
            t = t * 31u + f->x2074.x10.x0.x2085_b7; t = t * 31u + f->x2074.x2088;
            t = t * 31u + (u8) f->dmg.x18d4.x0.x2070; t = t * 31u + ((u8*) &f->dmg.x18d4)[1]; t = t * 31u + f->dmg.x18d4.x0.x2072_b0;
            t = t * 31u + f->dmg.x18d4.x0.count_specials; t = t * 31u + f->dmg.x18d4.x0.x2073;
            t = h18d8(t, &f->dmg.x18d8);
            t = t * 31u + f->dmg.x18ec_instancehitby;
            t = hstats(t, st);
            t = t * 31u + unk_804D6480;
            printf("%d %d %08X %08X %08X\n", op, k, ret, t, g_tr);
        }
    }
    return 0;
}
