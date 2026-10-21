/* C reference for src/plbonus3.rs (group 3 round 11): the primitives at the top of
 * src/melee/pl/plbonus.c (setFlag, setPointValue, pl_800386D8..pl_80038914, pl_80038F10..
 * resetBonuses), cut verbatim into plbonusc_extract.inc by run.py, with the player.c accessors
 * they call (the same cuts and stubs as plbonus_ref.c). gmDecisionGetType and gm_GetFrameCount are
 * tracing stubs (kind % 3 == 0 -> Point; the frame count is chosen per step). After each op: the
 * return value, out-params, trace word and a hash of x904[] and the fields pl_80038F10 writes.
 * No fused (a*b+c) site; built with the shipping contraction anyway (run.py). */
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

static u32 g_fc;
u32 gm_GetFrameCount(void) { T(0x170u); return g_fc; }
Gm_DecType gmDecisionGetType(int kind) { T(0x500u + (u32) kind); return kind % 3 == 0 ? Gm_DecType_Point : Gm_DecType_Flag; }

#include "plbonusc_extract.inc"

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

static void init_slot(int slot)
{
    pl_StaleMoveTableExt_t* x = Player_GetStaleMoveTableIndexPtr2(slot);
    StaleMoveTable* s = &x->x0_staleMoveTable;
    int i;
    for (i = 0; i < 215; i++) s->x904[i] = rnd() % 2 ? rnd() % 3 : rnd();
    for (i = 0; i < 65; i++) s->total_attack_count_struct.by_attack_hi[i] = rnd();
    s->xCAC = rnd() % 8; x->xD50 = rnd(); x->xD54 = rnd(); x->xD60 = rnd();
    x->xD64 = (int) (rnd() % 8); x->xD6C = (int) (rnd() % 8);
}

static u32 H;
static void h(u32 v) { H = H * 0x01000193u ^ v; }
static void hash_slot(int slot)
{
    pl_StaleMoveTableExt_t* x = Player_GetStaleMoveTableIndexPtr2(slot);
    StaleMoveTable* s = &x->x0_staleMoveTable;
    int i;
    for (i = 0; i < 215; i++) h(s->x904[i]);
    h(s->xCAC); h(x->xD50); h(x->xD54); h(x->xD60); h((u32) x->xD64); h((u32) x->xD6C);
}

#define NOPS 15

int main(void)
{
    static pl_804D6470_t tab;
    static pl_800386E8_arg0_t a6e8;
    static int arr[40];
    int k, step, i;
    pl_804D6470 = &tab;
    for (k = 0; k < 300; k++) {
        rs = 0xC0B05u + (u32) k * 2654435761u;
        g_tr = 0;
        for (i = 0; i < 6; i++) init_slot(i);
        for (step = 0; step < 50; step++) {
            int slot = (int) (rnd() % 6);
            u32 op = rnd() % NOPS;
            int pk = (int) (rnd() % 72) * 3;           /* a Point kind, 0..213 */
            int fk = pk + 1 + (int) (rnd() % 2);       /* a Flag kind, 1..215 */
            int anyk = (int) (rnd() % 215);
            int kk = rnd() % 3 == 0 ? -1 : pk;
            u32 c = rnd();
            u32 fsel = rnd() % 6;
            u32 a3 = 1u + rnd() % 20, a4 = rnd() % 8, a5 = rnd() % 16;
            u32 uo = rnd() % 3;
            int io = (int) (rnd() % 16);
            float fo = rf(), f6 = rf(), f7 = rf();
            int lo = (int) (rnd() % 22) - 1, hi = (int) (rnd() % 40) - 2;
            u32 ret = 0, o1 = 0, o2 = 0;
            if (fk > 214) fk = 214 - (214 % 3 == 0);
            g_fc = fsel == 0 ? 0u : fsel == 1 ? (u32) -2 : fsel == 5 ? rnd() : 1u + rnd() % 40;
            for (i = 0; i < 40; i++) arr[i] = (int) rnd();
            a6e8.unk_190 = (int) rnd(); a6e8.unk_5AC = (int) rnd(); a6e8.unk_5B0 = (int) rnd();
            switch (op) {
            case 0: ret = pl_800386D8(&Player_GetStaleMoveTableIndexPtr2(slot)->x0_staleMoveTable.total_attack_count_struct, (ssize_t) (c % 65)); break;
            case 1: ret = (u32) pl_800386E8(&a6e8); break;
            case 2: ret = (u32) fn_80038700(arr, lo, hi < 38 ? hi : 38); break;
            case 3: pl_80038788(slot, pk, (int) c); break;
            case 4: pl_80038824(slot, pk); break;
            case 5: pl_80038898(slot, pk); break;
            case 6: ret = pl_80038914() == &tab; break;
            case 7: pl_80038F10(slot); break;
            case 8: fn_80038FB8(slot); break;
            case 9: ret = (u32) pl_8003906C(slot, kk, &uo, a3, a4, a5, &io); o1 = uo; o2 = (u32) io; break;
            case 10: { int iv = (int) uo; ret = (u32) pl_80039238(slot, kk, &iv, a3, &fo, f6, f7); o1 = (u32) iv; o2 = B(fo); } break;
            case 11: ret = pl_80039418(slot, anyk); break;
            case 12: setFlag(slot, fk); break;
            case 13: setPointValue(slot, pk, c); break;
            case 14: resetBonuses(slot); break;
            }
            H = 0x811C9DC5u;
            for (i = 0; i < 6; i++) hash_slot(i);
            printf("%d %d %u %x %x %x %x %x\n", k, step, op, ret, o1, o2, g_tr, H);
        }
    }
    return 0;
}
