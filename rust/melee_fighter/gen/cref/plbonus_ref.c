/* C reference for src/plbonus.rs (port round 4): all of src/melee/pl/pl_040D.c and the tail of
 * src/melee/pl/plbonuslib.c (pl_8003FBFC..EOF, + its static inline unk_cond), cut verbatim into
 * plbonus_extract.inc by run.py, together with the player.c accessors they call (the same cuts as
 * player_ref.c, whose stubs are repeated below). Random player tables / stale-table extensions /
 * pl_804D6470 constants, 81 random ops; every call out (plbonus.c, ft_0877/0892, gm) is a stub
 * folding its arguments into the trace word g_tr. After each op: return value, trace word, hash of
 * every field these functions write. No fused (a*b+c) site in these functions
 * (tools/rust-port/fma_sites.py); built with the shipping contraction anyway (run.py). */
#include <melee/pl/player.h>
#include <melee/pl/types.h>
#include <melee/pl/forward.h>
#include <melee/ft/forward.h>
#include <melee/ft/types.h>
#include <melee/ft/ftlib.h>
#include <melee/ft/ftdemo.h>
#include <melee/ft/ft_0877.h>
#include <melee/ft/ft_0D4D.h>
#include <melee/ft/kinds/ftKirby/ftkirby.h>
#include <melee/gm/gm_unsplit.h>
#include <melee/if/ifstatus.h>
#include <sysdolphin/baselib/debug.h>
#include <sysdolphin/baselib/gobj.h>
#include <sysdolphin/baselib/objalloc.h>

static u32 g_tr;
static void T(u32 v) { g_tr = g_tr * 31u + v; }
static u32 B(float f) { union { float f; u32 u; } c; c.f = f; return c.u; }
static HSD_GObj ga, gb;
static u32 E(HSD_GObj* g) { return g == NULL ? 0u : g == &ga ? 1u : g == &gb ? 2u : 3u; }

int printf(const char*, ...);
void exit(int);
void __assert(const char* file, u32 line, const char* cond) { printf("ASSERT %s:%u %s\n", file, (unsigned) line, cond); exit(3); }
void OSReport(char* fmt, ...) { (void) fmt; }
float ScriptGame_ModValue(int slot, int field, float base) { T(0x220u + (u32) slot * 2u + (u32) field); T(B(base)); return base; }

void ftLib_800867E8(HSD_GObj* g) { T(0x10u + E(g)); }
void ftLib_8008688C(HSD_GObj* g) { T(0x20u + E(g)); }
void ftKb_SpecialN_800EED50(s32 a, s32 b) { T(0x30u); T((u32) a); T((u32) b); }
bool ftLib_8008701C(HSD_GObj* g) { T(0x40u + E(g)); return (g_tr >> 3) & 1; }
void ftCo_800D4F24(Fighter_GObj* g, int i) { T(0x50u + E(g)); T((u32) i); }
void HSD_GObjFree(HSD_GObj* g) { T(0x60u + E(g)); }
void ftCo_800D4FF4(Fighter_GObj* g) { T(0x70u + E(g)); }
void ifStatus_802F6E1C(int s) { T(0x80u + (u32) s); }
bool ftLib_800873CC(HSD_GObj* g) { T(0x90u + E(g)); return (g_tr >> 5) & 1; }
void ftLib_80087140(HSD_GObj* g) { T(0xA0u + E(g)); }
void ftLib_80086664(HSD_GObj* g, Vec3* v) { T(0xB0u + E(g)); T(B(v->x)); T(B(v->y)); T(B(v->z)); }
void ftDemo_SetFacingDirection(Fighter_GObj* g, float f) { T(0xC0u + E(g)); T(B(f)); }
float ftLib_800869F8(HSD_GObj* g) { T(0xD0u + E(g)); return (float) (int) (g_tr % 1000u) / 7.0f; }
void ftLib_SetScale(HSD_GObj* g, float f) { T(0xE0u + E(g)); T(B(f)); }
void ft_80087BEC(Fighter_GObj* g, int v) { T(0xF0u + E(g)); T((u32) v); }
void ft_80087BC0(Fighter_GObj* g, int v) { T(0x100u + E(g)); T((u32) v); }
int gm_8016C5C0(int s) { T(0x110u + (u32) s); return (int) (g_tr % 97u); }
void ftLib_800870F0(HSD_GObj* g, s32 v) { T(0x120u + E(g)); T((u32) v); }
void ifStatus_802F6D10(s32 s) { T(0x130u + (u32) s); }
bool gm_8016B168(void) { T(0x140u); return (g_tr >> 4) & 1; }
void ifStatus_802F69C0(s32 a, s32 b) { T(0x150u); T((u32) a); T((u32) b); }
void ifStatus_802F6AF8(s32 s) { T(0x160u + (u32) s); }
u32 gm_GetFrameCount(void) { T(0x170u); return g_tr; }
void ifStatus_802F6C04(s32 s) { T(0x180u + (u32) s); }
bool ftLib_80086F4C(HSD_GObj* g) { T(0x190u + E(g)); return (g_tr >> 2) & 1; }
float ftLib_80086F80(HSD_GObj* g) { T(0x1A0u + E(g)); return (float) (int) (g_tr % 777u) / 3.0f; }
FighterKind ftLib_GetKind(HSD_GObj* g) { T(0x1B0u + E(g)); return (FighterKind) (g_tr % 40u); }
s32 ftLib_80087300(HSD_GObj* g) { T(0x1C0u + E(g)); return (s32) (g_tr % 7u); /* round 10: a slot or 6 (pl_8003DF44 indexes with it) */ }
void ft_80087AC0(Fighter_GObj* g, s32 v) { T(0x1D0u + E(g)); T((u32) v); }
void ftLib_80086A4C(HSD_GObj* g, float f) { T(0x1E0u + E(g)); T(B(f)); }
void ft_80087BAC(Fighter_GObj* g, s32 v) { T(0x1F0u + E(g)); T((u32) v); }
bool ftLib_80086BB4(HSD_GObj* g) { T(0x200u + E(g)); return (g_tr >> 6) & 1; }
void ftLib_80086B90(HSD_GObj* g, Vec3* v) { T(0x210u + E(g)); v->x = (float) (int) (g_tr % 100u); v->y = -1.0f; }
#include <melee/pl/pl_040D.h>
#include <melee/pl/plbonus.h>
#include <melee/pl/plbonuslib.h>
#include <melee/pl/plbonusinline.h>
#include <melee/pl/inlines.h>
#include <melee/ft/ft_0877.h>
#include <melee/ft/ft_0892.h>
#include <melee/gm/gm_16F1.h>
#include <melee/gm/gmvs.h>
#include <melee/it/forward.h>

static ft_800898B4_t g_kb;
void pl_80038788(int s, int k, int n) { T(0x300u + (u32) s); T((u32) k); T((u32) n); }
void pl_80038824(int s, int k) { T(0x310u + (u32) s); T((u32) k); }
void pl_80038898(int s, int k) { T(0x320u + (u32) s); T((u32) k); }
unsigned int pl_80039418(int s, int k) { T(0x330u + (u32) s); T((u32) k); return (g_tr % 5u) == 0; }
unsigned int pl_800386D8(plActionStats* st, ssize_t n) { T(0x340u); T(st->attacks.total); T((u32) n); return g_tr % 1000u; }
float ft_80087A8C(Fighter_GObj* g) { T(0x350u + E(g)); return (float) ((int) (g_tr % 2001u) - 1000) / 7.0f; }
s32 ft_80087A98(Fighter_GObj* g) { T(0x360u + E(g)); return (s32) (g_tr % 4u); }
bool ft_80087AA4(Fighter_GObj* g) { T(0x370u + E(g)); return (g_tr >> 2) & 1; }
u8 ft_80087AB4(Fighter_GObj* g) { T(0x380u + E(g)); return (u8) (g_tr % 3u); }
ft_800898B4_t* ft_800898B4(Fighter_GObj* g) { T(0x390u + E(g)); g_kb.kb_applied1 = (float) (int) (g_tr % 400u) / 2.0f; return &g_kb; }
int gm_8016F2F8(int k, unsigned char s) { T(0x3A0u); T((u32) k); T((u32) s); return (int) (g_tr % 50u); }
void gm_8016B6E8(int s, int v) { T(0x3B0u + (u32) s); T((u32) v); }
/* round 10 (pl_8003DF44..pl_8003ED0C): item / ftLib / ft_0892 calls */
#include <melee/it/it_26B1.h>
static struct Struct2074 g_s2074;
static HSD_GObj gi;
static const int ikinds[12] = { 0, 7, 6, 16, 34, 35, 3, 38, 205, 225, 226, 40 };
/* pl_8003E4A4 inputs: only kinds match_item_kind maps (an unmapped one stores to seen[-1] in the C) */
static const int mkinds[10] = { 0, 7, 6, 16, 34, 3, 205, 225, 226, 1 };
ItemKind itGetKind(Item_GObj* g) { T(0x4C0u + E(g)); return (ItemKind) ikinds[g_tr % 12u]; }
s32 it_8026B7E8(Item_GObj* g) { T(0x420u + E(g)); return g_tr % 3u == 0; }
bool it_8026B774(Item_GObj* g, u8 a) { T(0x410u + E(g)); T((u32) a); return (g_tr >> 3) & 1; }
HSD_GObj* itGetOwner(Item_GObj* g) { u32 k; T(0x430u + E(g)); k = g_tr % 3u; return k == 0 ? NULL : k == 1 ? &ga : &gb; }
bool ftLib_80086960(HSD_GObj* g) { T(0x440u + E(g)); return (g_tr >> 1) & 1; }
u8 ftLib_80086BE0(HSD_GObj* g) { T(0x450u + E(g)); return (u8) (g_tr % 6u); }
s32 ftLib_800874BC(HSD_GObj* g) { T(0x460u + E(g)); return (s32) ((g_tr >> 4) & 1); }
s32 it_8026B7BC(Item_GObj* g) { T(0x470u + E(g)); return (s32) ((g_tr >> 2) & 1); }
float it_8026B960(Item_GObj* g) { T(0x480u + E(g)); return (float) ((int) (g_tr % 2001u) - 1000) / 7.0f; }
s32 it_8026B7CC(Item_GObj* g) { T(0x490u + E(g)); return (s32) (g_tr % 8u); }
u32 it_8026C1D4(void) { T(0x4A0u); return g_tr % 4u; }
struct Struct2074* ft_800898A8(Fighter_GObj* g) { T(0x4B0u + E(g)); return &g_s2074; }

#include "plbonus_extract.inc"

int printf(const char*, ...);
static u32 rs;
static u32 rnd(void) { rs ^= rs << 13; rs ^= rs >> 17; rs ^= rs << 5; return rs; }
static float rf(void)
{
    u32 k = rnd() % 16;
    if (k == 0) return 0.0f;
    if (k == 1) return -0.0f;
    if (k == 2) return 1.0f;
    if (k == 3) return -1.0f;
    if (k == 4) return 5.0e9f;
    if (k == 5) return -3.0f;
    return (float) ((int) (rnd() % 2001) - 1000) / 7.0f;
}
static HSD_GObj* rent(void) { u32 k = rnd() % 3; return k == 0 ? NULL : k == 1 ? &ga : &gb; }
static u32 z(void) { return rnd() % 2 ? 0u : rnd(); }

static void init_slot(int slot)
{
    StaticPlayer* p = &player_slots[slot];
    pl_StaleMoveTableExt_t* x = Player_GetStaleMoveTableIndexPtr2(slot);
    StaleMoveTable* s = &x->x0_staleMoveTable;
    plActionStats* a = &s->total_attack_count_struct;
    char* c = (char*) x;
    int i;
    for (i = 0; i < (int) sizeof(*x); i++) c[i] = 0;
    p->player_state = (s32) (rnd() % 4);
    p->team = (u8) (rnd() % 3);
    p->transformed[0] = (u8) (rnd() % 2);
    p->transformed[1] = (u8) (rnd() % 2);
    p->player_entity[0] = rent();
    p->player_entity[1] = rent();
    s->x5EC.x0 = (u8) (rnd() % 6); s->x5EC.x4 = rf(); s->x5EC.x8 = rnd() % 4; s->x5EC.xC = rnd() % 4;
    for (i = 0; i < 6; i++) {
        s->x5EC.x10[i].x0 = rf(); s->x5EC.x10[i].x4 = (s16) (rnd() % 3); s->x5EC.x10[i].x8 = (s16) (rnd() % 5);
        s->x5EC.x10[i].xC = rnd() % 3 == 0 ? 0x5A : (int) (rnd() % 0x62);
        s->x5EC.x10[i].x10 = (s16) (rnd() % 6); s->x5EC.x10[i].x12_b0 = rnd() & 1;
    }
    s->x8C0.x0 = (int) (rnd() % 5);
    for (i = 0; i < 5; i++) { s->x8C0.x4[i].x0 = (u16) (rnd() % 8); s->x8C0.x4[i].x2 = (u8) (rnd() % 7); s->x8C0.x4[i].x3_b0 = rnd() & 1; }
    s->x8D8.x0 = (int) (rnd() % 5);
    for (i = 0; i < 5; i++) { s->x8D8.x4[i].x0 = rnd() % 8; s->x8D8.x4[i].x4_b0 = rnd() & 1; s->x8D8.x4[i].x4_b1 = rnd() & 1; }
    s->xC60 = rf(); s->xC64 = rf(); s->xC68 = (int) rnd(); s->xC6C = rf(); s->xC90 = rnd() % 5;
    s->xCAC = rnd(); s->xCB0 = (int) rnd(); s->xCB4 = (int) rnd(); s->xCB8 = (int) (rnd() % 7);
    s->xCBC.x3 = (u8) (0x30 + rnd() % 16); s->xCC0.x10_b0 = rnd() & 1;
    s->xCDC = rf(); s->xCE0 = rf(); s->xCE4 = rnd(); s->xCE8 = rnd(); s->xCEC = (int) z(); s->xCF0 = (int) rnd();
    x->xCF8 = rf(); x->xCFC = (int) rnd(); x->xD00 = (int) rnd(); x->xD04 = rnd(); x->xD20 = rnd(); x->xD24 = rnd();
    x->xD40 = rnd(); x->xD48 = rnd(); x->xD4C = z(); x->xD50 = z(); x->xD54 = z(); x->xD60 = rnd();
    x->xD64 = rnd() % 2 ? 6 : (int) rnd(); x->xD68 = (int) rnd(); x->xD70 = (int) rnd();
    x->xD74 = rf(); x->xD78 = rf(); x->xD7C = rf(); x->xD80 = rf(); x->xD84 = rf(); x->xD88 = rf();
    x->xDD0.bit0 = rnd() & 1; x->xDD0.bit5 = rnd() & 1;
    x->xDD1.bit4 = rnd() & 1; x->xDD1.bit6 = rnd() & 1;
    a->attacks.total = z(); a->attacks.aerials_count = rnd(); a->attacks.thrown_item_count = rnd();
    a->attacks.specials_count = rnd();
    for (i = 30; i < 70; i++) a->attacks.by_attack_counts[i] = rnd();
    a->x358_hits.total = rnd();
    /* round 10 fields */
    for (i = 0; i < Pl_ItemLog_Terminate; i++) { s->x674[i] = rnd() % 4; s->x710[i] = (int) (rnd() % 4); s->x7AC[i] = (int) (rnd() % 4); }
    s->xCD8 = (int) rnd(); s->xCA8 = rnd() % 4; s->xC70 = rf(); s->xC74 = rf();
    for (i = 0; i < 6; i++) s->xC78[i] = rf();
    s->xC98 = rf(); s->xCA0 = rf(); s->xCA4 = (int) (rnd() % 100);
    x->xD08 = rnd(); x->xD10 = (int) rnd(); x->xD44 = rnd(); x->xD6C = rnd() % 2 ? 6 : (int) (rnd() % 6);
}

static u32 H;
static void h(u32 v) { H = H * 0x01000193u ^ v; }
static void hash_slot(int slot)
{
    pl_StaleMoveTableExt_t* x = Player_GetStaleMoveTableIndexPtr2(slot);
    StaleMoveTable* s = &x->x0_staleMoveTable;
    int i;
    h(s->x5EC.x0); h(B(s->x5EC.x4)); h(s->x5EC.x8); h(s->x5EC.xC);
    for (i = 0; i < 6; i++) {
        h(B(s->x5EC.x10[i].x0)); h((u32) s->x5EC.x10[i].x4); h((u32) s->x5EC.x10[i].x8); h((u32) s->x5EC.x10[i].xC);
        h((u32) s->x5EC.x10[i].x10); h(s->x5EC.x10[i].x12_b0);
    }
    h((u32) s->x8C0.x0);
    for (i = 0; i < 5; i++) { h(s->x8C0.x4[i].x0); h(s->x8C0.x4[i].x2); h(s->x8C0.x4[i].x3_b0); }
    h((u32) s->x8D8.x0);
    for (i = 0; i < 5; i++) { h(s->x8D8.x4[i].x0); h(s->x8D8.x4[i].x4_b0); h(s->x8D8.x4[i].x4_b1); }
    h((u32) s->xC68); h(s->xC90); h(s->xCAC); h((u32) s->xCB0); h(B(s->xCDC)); h(B(s->xCE0)); h(s->xCE4); h(s->xCE8);
    h((u32) s->xCEC); h((u32) s->xCF0); h(B(x->xCF8)); h(x->xD4C); h(x->xD50); h(x->xD60); h((u32) x->xD70);
    h(x->xDD0.bit0); h(x->xDD0.bit5); h(x->xDD1.bit4); h(x->xDD1.bit6);
    for (i = 0; i < Pl_ItemLog_Terminate; i++) { h(s->x674[i]); h((u32) s->x710[i]); h((u32) s->x7AC[i]); }
    h((u32) s->xCD8); h(s->xCA8); h(B(s->xC6C)); h(B(s->xC70)); h(B(s->xC74));
    for (i = 0; i < 6; i++) h(B(s->xC78[i]));
    h(B(s->xC98)); h(B(s->xC60)); h(B(s->xC64)); h(B(s->xCA0)); h((u32) s->xCA4);
    h(x->xD08); h((u32) x->xD10); h(x->xD44); h(x->xD54); h((u32) x->xD6C);
}

static const int items[14] = { It_Kind_Old_Kuri, It_Kind_Kuriboh, It_Kind_Heiho, It_Kind_Nokonoko,
    It_Kind_Patapata, It_Kind_Likelike, It_Kind_Old_Lead, It_Kind_Leadead, It_Kind_Old_Octa,
    It_Kind_Octarock, It_Kind_Old_Otto, It_Kind_Ottosea, It_Kind_Whitebea, 0 };

#define NOPS 104

int main(void)
{
    static pl_804D6470_t tab;
    int k, step, i;
    pl_804D6470 = &tab;
    for (k = 0; k < 300; k++) {
        rs = 0xB0B05u + (u32) k * 2654435761u;
        g_tr = 0;
        for (i = 0; i < 6; i++) init_slot(i);
        tab.x4 = (int) (rnd() % 6); tab.x17C = (int) (rnd() % 5); tab.xD4 = rf(); tab.x38 = rf();
        tab.x98 = rf(); tab.xA0 = rf(); tab.xA4 = rnd(); tab.xB0 = rnd(); tab.x124 = rnd();
        tab.x4C = rf(); tab.xEC = (int) rnd();
        tab.x84 = rf(); tab.x138 = rnd() % 4; tab.xB4 = rnd(); tab.x180 = rf();
        g_s2074.x207C.y = 0;
        for (step = 0; step < 50; step++) {
            int slot = (int) (rnd() % 6);
            u32 op = rnd() % NOPS;
            int a = (int) (rnd() % 6);
            int b = (int) (rnd() % 3);
            int c = (int) rnd();
            int w = (int) (rnd() % 0x62);
            int z7 = (int) (rnd() % 7);
            int it = items[rnd() % 14];
            float d = rf();
            float e = rf();
            int q = rnd() % 3 == 0 ? 0x59 : w % 20;
            int a5 = w % 4 == 0 ? 0x34 : w % 4 == 1 ? 0x5C : w % 4 == 2 ? 0x5D : w;
            int moves[6];
            u32 ret = 0;
            switch (op) {
            case 0: pl_80040DDC(slot); break;
            case 1: pl_80040ED4(slot, c, a, b, w, d); break;
            case 2: pl_80040FBC(slot, c, a, b, w); break;
            case 3: pl_800410F4(slot, c, a, b, w); break;
            case 4: pl_800411C4(slot, b); break;
            case 5: pl_80041280(slot, b); break;
            case 6: ret = (u32) pl_800412D0(slot); break;
            case 7: ret = (u32) pl_80041300(slot); break;
            case 8: ret = (u32) pl_8004134C(slot); break;
            case 9: fn_8004138C(slot, b); break;
            case 10: ret = B(pl_800414C0(slot)); break;
            case 11: pl_80041524(slot); break;
            case 12: fn_800415B0(slot, c, (u16) (w % 8), (u32) (b & 1)); break;
            case 13: pl_80041720(slot, c, (u16) (w % 8)); break;
            case 14: pl_80041744(slot, b, (u16) (w % 8)); break;
            case 15: fn_80041770(slot, (u8) (w % 8), (u32) b); break;
            case 16: pl_8004182C(slot); break;
            case 17: pl_800418F4(slot); break;
            case 18: pl_800419AC(slot, c, (u16) (w % 8)); break;
            case 19: pl_80041B08(slot, NULL, (u16) (w % 8)); break;
            case 20: fn_80041BC8(slot, (u32) (b & 1), (u32) (a & 1)); break;
            case 21: pl_80041BFC(slot); break;
            case 22: ret = (u32) pl_8003FBFC(slot); break;
            case 23: ret = (u32) pl_8003FC20(slot); break;
            case 24: pl_8003FC44(slot, b); break;
            case 25: pl_8003FC88(slot, b, w % 50); break;
            case 26: pl_8003FDA0(slot, c); break;
            case 27: pl_8003FDC8(slot, c); break;
            case 28: pl_8003FDF4(slot, c); break;
            case 29: pl_8003FE1C(slot, c); break;
            case 30: pl_8003FE40(slot, c); break;
            case 31: pl_8003FE64(slot, c); break;
            case 32: pl_8003FED0(slot, b); break;
            case 33: pl_8003FF44(slot, b & 1, w % 20); break;
            case 34: pl_8003FFDC(slot, c, a, b & 1, w % 4); break;
            case 35: pl_80040048(slot, b & 1); break;
            case 36: pl_80040120(slot, b); break;
            case 37: pl_800401F0(slot, c, d, e); break;
            case 38: pl_80040270(slot, b, d); break;
            case 39: pl_800402D0(slot, b, a & 1); break;
            case 40: pl_80040330(slot, b, d); break;
            case 41: pl_80040374(slot, b); break;
            case 42: pl_800403C0(slot, c); break;
            case 43: pl_800403FC(c, b, slot, b, 0x50 + w % 10); break;
            case 44: pl_80040460(slot, b); break;
            case 45: pl_8004049C(z7, (ItemKind) it); break;
            case 46: pl_80040614(slot, b, d); break;
            case 47: pl_8004065C(slot, b); break;
            case 48: pl_80040688(slot, z7, b); break;
            case 49: ret = B(pl_800407C8(slot)); break;
            case 50: ret = B(pl_80040870(slot)); break;
            case 51: ret = pl_80040894(slot); break;
            case 52: ret = pl_800408B8(slot); break;
            case 53: ret = (u32) pl_800408DC(slot); break;
            case 54: ret = (u32) pl_80040900(slot); break;
            case 55: ret = (u32) pl_80040924(slot); break;
            case 56: ret = B(pl_80040948(slot)); break;
            case 57: ret = (u32) pl_80040A04(slot); break;
            case 58: ret = (u32) pl_80040A30(slot); break;
            case 59: ret = (u32) pl_80040A54(slot); break;
            case 60: ret = (u32) pl_80040A78(slot); break;
            case 61: ret = (u32) pl_80040A9C(slot); break;
            case 62: ret = pl_80040AF0(slot); break;
            case 63: ret = (u32) pl_80040B18(slot); break;
            case 64: ret = pl_80040B3C(slot); break;
            case 65: ret = pl_80040B64(slot); break;
            case 66: pl_80040B8C(slot, b, c); break;
            case 67: ret = (u32) pl_80040BD8(slot); break;
            case 68: ret = pl_80040BFC(slot); break;
            case 69: ret = B(pl_80040C24(slot)); break;
            case 70: ret = B(pl_80040C48(slot)); break;
            case 71: ret = B(pl_80040C6C(slot)); break;
            case 72: ret = B(pl_80040C90(slot)); break;
            case 73: ret = (u32) pl_80040CB4(slot); break;
            case 74: ret = (u32) pl_80040CD8(slot); break;
            case 75: ret = pl_80040CFC(slot); break;
            case 76: ret = (u32) pl_80040D20(slot); break;
            case 77: ret = B(pl_80040D44(slot)); break;
            case 78: ret = B(pl_80040D68(slot)); break;
            case 79: pl_80040D8C(slot, c); break;
            case 80: ret = (u32) pl_80040DB8(slot); break;
            /* round 10 */
            case 81: pl_8003DF44(slot, b & 1); break;
            case 82: pl_8003DFF4(slot, b & 1, q); break;
            case 83: pl_8003E058(z7, c, a, b); break;
            case 84: pl_8003E0E8(slot, b); break;
            case 85: pl_8003E114(slot, c, d); break;
            case 86: pl_8003E150(slot, c); break;
            case 87: pl_8003E17C(slot, c, &gi); break;
            case 88: ret = pl_8003E2CC(slot, (Pl_ItemLog) (w % 39)); break;
            case 89: ret = pl_8003E334(slot, (Pl_ItemLog) (w % 39)); break;
            case 90: ret = (u32) pl_8003E39C(slot); break;
            case 91: ret = pl_8003E420(slot); break;
            case 92:
                for (i = 0; i < 6; i++) moves[i] = mkinds[rnd() % 10];
                pl_8003E4A4(slot, b & 1, moves, a);
                break;
            case 93: pl_8003E854(slot, b & 1, &gi); break;
            case 94: pl_8003E978(slot, b & 1, &gi); break;
            case 95: fn_8003E998(slot, b & 1); break;
            case 96: pl_8003EA08(slot, b); break;
            case 97: pl_8003EA40(slot, c, a, b & 1, z7); break;
            case 98: pl_8003EA74(slot, b); break;
            case 99: pl_8003EAAC(slot, b & 1, q); break;
            case 100: pl_8003EB30(d, z7, c, b == 2 ? 6 : a, b & 1, a5); break;
            case 101: pl_8003EC30(slot, b, w % 4, d); break;
            case 102: pl_8003EC9C(slot, b, d, e); break;
            case 103: pl_8003ED0C(slot, c, a, b, d); break;
            }
            H = 0x811C9DC5u;
            for (i = 0; i < 6; i++) hash_slot(i);
            h((u32) g_s2074.x207C.y);
            printf("%d %d %u %x %x %x\n", k, step, op, ret, g_tr, H);
        }
    }
    return 0;
}
