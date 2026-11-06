/* C reference for src/ft0877.rs (port round 5): all of src/melee/ft/ft_0877.c (from the TEST
 * macro to EOF) cut verbatim into ft0877_extract.inc by run.py, plus ftCommon_80080144 from
 * ftcommon.c. Six fighters, a random fighter p-link list, random fields, one op per case.
 * Callees outside the ported set (lbAudioAx, itIsHeavy/itGetKind, ftCo, Mex/Geno, HSD_Randi) are
 * stubs folding their arguments into the trace word g_tr; tests/ft0877_ref.rs installs the same
 * stubs as hooks. Built with the shipping contraction (run.py): ft_80087B34 has 5 fused sites.
 * Velocity vectors are sometimes swapped copies of each other (equal magnitude in exact math)
 * so fused vs unfused rounding decides the comparisons. */
#include "ft/types.h"
#include "ft/forward.h"
#include "ft/fighter.h"
#include "ft/ftcommon.h"
#include "ft/ft_0877.h"
#include "ft/ftdevice.h"
#include "ft/kinds/ftCommon/ftCo_0A01.h"
#include <melee/it/it_26B1.h>
#include <melee/lb/lbaudio_ax.h>
#include <sysdolphin/baselib/gobj.h>
#include <sysdolphin/baselib/random.h>

#define NF 6
static u32 g_tr;
static void T(u32 v) { g_tr = g_tr * 31u + v; }
static u32 B(float f) { union { float f; u32 u; } c; c.f = f; return c.u; }

static Fighter F[NF];
static HSD_GObj GO[NF], IT[3];
static HSD_GObj* plinks[64];
HSD_GObj** HSD_GObjPLinkHead;
void __assert(const char* file, u32 line, const char* cond) { (void) file; (void) line; (void) cond; for (;;) {} }

static u32 E(void* g)
{
    int i;
    if (g == NULL) return 0;
    for (i = 0; i < NF; i++) if (g == &GO[i]) return 1u + (u32) i;
    for (i = 0; i < 3; i++) if (g == &IT[i]) return 10u + (u32) i;
    return 99;
}
static u32 FI(Fighter* fp) { int i; for (i = 0; i < NF; i++) if (fp == &F[i]) return (u32) i; return 9; }

static const int SFXM[25] = { 407, 467, 503, 515, 290051, 509, 485, 491, 479, 473, 437, 455, 461,
                              449, 443, 425, 431, 419, 413, 377, 395, 401, 389, 383, 371 };
static const int SFXS[13] = { 359, 368, 365, 362, 344, 353, 356, 350, 347, 338, 341, 335, 332 };

bool itIsHeavy(Item_GObj* g) { T(0x300u + E(g)); return (g_tr >> 3) & 1; }
ItemKind itGetKind(Item_GObj* g)
{
    u32 k;
    T(0x310u + E(g));
    k = (g_tr >> 2) & 3;
    return k == 0 ? 30 : k == 1 ? 7 : (ItemKind) k;
}
s32 ftGetParasolStatus(Fighter_GObj* g) { T(0x320u + E(g)); return 3 + (s32) ((g_tr >> 1) % 4u); }
int ftCo_800C06B4(Fighter* fp) { u32 k; T(0x330u + FI(fp)); k = (g_tr >> 4) % 3u; return k == 0 ? 0x7B : k == 1 ? 0x80 : 5; }
void ftCo_800A101C(Fighter* fp, int a, int b, int c) { T(0x340u + FI(fp)); T((u32) a); T((u32) b); T((u32) c); }
int lbAudioAx_800233EC(int id)
{
    u32 k;
    T(0x350u); T((u32) id);
    k = g_tr % 4u;
    if (k == 0) return id;
    if (k == 1) return SFXM[(g_tr >> 3) % 25u];
    if (k == 2) return SFXS[(g_tr >> 3) % 13u];
    return 0x1FBFD + (int) ((g_tr >> 3) % 0xD0u);
}
int lbAudioAx_80023130(int sfx)
{
    u32 k;
    T(0x360u); T((u32) sfx);
    k = (g_tr >> 5) % 5u;
    if (k == 0 || k == 4) return 0;
    if (k == 1) return 13;
    if (k == 2) return 6 + (int) ((g_tr >> 8) % 28u);
    return ((g_tr >> 8) & 1) ? 32 : 34;
}
int lbAudioAx_800230C8(int ssm, int* off, int* out)
{
    T(0x370u + (u32) ssm);
    *off = (int) (g_tr % 0x20000u);
    *out = 1;
    return ((g_tr >> 9) & 3) == 0 ? 1 : 0;
}
int lbAudioAx_80023220(int ssm) { T(0x380u + (u32) ssm); return (int) ((g_tr >> 3) % 0x10000u); }
void lbAudioAx_80024DC4(int s) { T(0x390u); T((u32) s); }
void lbAudioAx_80024FDC(void) { T(0x3A0u); }
void lbAudioAx_80024FF4(void) { T(0x3B0u); }
void lbAudioAx_8002500C(int n) { T(0x3C0u); T((u32) n); }
void lbAudioAx_80025038(int n) { T(0x3D0u); T((u32) n); }
int lbAudioAx_800237A8(enum_t s, int v, int p) { T(0x3E0u); T((u32) s); T((u32) v); T((u32) p); return (int) (g_tr >> 4); }
int lbAudioAx_80024B94(int h, int v) { T(0x3F0u); T((u32) h); T((u32) v); return 0; }
int Mex_SsmForPortKind(int k) { T(0x270u + (u32) k); return (int) (g_tr % 5u) - 1; }
int Geno_DefineVoicePlayOwner(int k, int s, u32 owner) { (void) owner; T(0x400u + (u32) k); T((u32) s); return (g_tr >> 3) & 1; }
s32 HSD_Randi(s32 n) { T(0x160u + (u32) n); return (s32) ((g_tr >> 7) % (u32) n); }

#include "ft0877_extract.inc"

int printf(const char*, ...);
static void zero(void* p, unsigned n) { unsigned char* b = p; while (n--) *b++ = 0; }

static u32 rs;
static u32 rnd(void) { rs ^= rs << 13; rs ^= rs >> 17; rs ^= rs << 5; return rs; }
static float rf(void)
{
    u32 k = rnd() % 16;
    if (k == 0) return 0.0f;
    if (k == 1) return -0.0f;
    if (k == 2) return 1.0f;
    if (k == 3) return -1.0f;
    return (float) ((int) (rnd() % 2001) - 1000) / 7.0f;
}

static const int KINDS[12] = { 0, 2, 8, 10, 11, 14, 20, 27, 28, 33, 34, 40 };
#define NOPS 34

int main(void)
{
    int op, k, i;
    HSD_GObjPLinkHead = plinks;
    for (op = 0; op < NOPS; op++) {
        for (k = 0; k < 300; k++) {
            u32 r = 0, t = 0;
            int n, used = 0, ia, ib, sa;
            HSD_GObj *g, *prev = NULL;
            Fighter* f0;
            zero(F, sizeof F); zero(GO, sizeof GO); zero(IT, sizeof IT); zero(plinks, sizeof plinks);
            g_tr = 0;
            rs = 0x2468Au + (u32) op * 7919u + (u32) k * 104729u;
            g_tr = rnd(); /* random start so stub results vary */
            for (i = 0; i < NF; i++) {
                Fighter* f = &F[i];
                u32 m;
                GO[i].user_data = f;
                f->gobj = &GO[i];
                f->kind = KINDS[rnd() % 12];
                f->costume_id = (u8) (rnd() % 4);
                f->input.held_buttons[0] = rnd();
                f->input.pressed_buttons = rnd();
                f->x1968_jumpsUsed = (u8) (rnd() % 4);
                f->motion_id = (rnd() % 3) ? 0 : (int) (rnd() % 340);
                f->item_gobj = (rnd() & 1) ? &IT[rnd() % 3] : NULL;
                f->x197C = (rnd() % 4) == 0 ? &IT[rnd() % 3] : NULL;
                f->x1980 = (rnd() & 1) ? &IT[rnd() % 3] : NULL;
                f->smash_attrs.x2134_vibrateFrame = (u8) rnd();
                f->smash_attrs.x2138_smashSinceHitbox = rf();
                f->x213C = (s32) rnd();
                f->x34_scale.x = rf();
                f->x34_scale.y = (rnd() & 1) ? f->x34_scale.x : rf();
                f->self_vel.x = rf(); f->self_vel.y = rf();
                f->x8c_kb_vel.x = rf(); f->x8c_kb_vel.y = rf();
                f->x98_atk_shield_kb.x = rf(); f->x98_atk_shield_kb.y = rf();
                m = rnd() % 4;
                if (m == 1) { f->x8c_kb_vel.x = f->self_vel.y; f->x8c_kb_vel.y = f->self_vel.x; }
                if (m == 2) { f->x98_atk_shield_kb.x = f->self_vel.y; f->x98_atk_shield_kb.y = f->self_vel.x; }
                if (m == 3) { f->x98_atk_shield_kb.x = f->x8c_kb_vel.y; f->x98_atk_shield_kb.y = f->x8c_kb_vel.x; }
                f->cpu.kind = rnd() % 30;
                f->cpu.level = (int) (rnd() % 10);
                f->cpu.x14 = (int) rnd();
                f->x1988 = (int) rnd();
                f->x2164 = (int) (rnd() % 100);
                f->x2168 = (int) (rnd() % 100);
                f->x2160 = (int) rnd();
                f->x221D_b6 = rnd() & 1; f->x221D_b7 = rnd() & 1;
                f->invisible = rnd() & 1;
                f->x2220_b5 = rnd() & 1; f->x2220_b6 = rnd() & 1;
                f->is_metal = rnd() & 1;
                f->x2226_b4 = rnd() & 1; f->x2226_b5 = rnd() & 1;
                f->x2227_b1 = rnd() & 1; f->x2227_b5 = rnd() & 1;
                f->x2228_b3 = rnd() % 4;
                f->x2228_b5 = rnd() & 1;
            }
            n = (int) (rnd() % (NF + 1));
            for (i = 0; i < n; i++) {
                int j = (int) (rnd() % NF);
                while (used & (1 << j)) j = (j + 1) % NF;
                used |= 1 << j;
                if (prev == NULL) plinks[HSD_GOBJ_PLINK_FIGHTER] = &GO[j]; else prev->next = &GO[j];
                prev = &GO[j];
            }
            g = &GO[rnd() % NF];
            f0 = g->user_data;
            ia = (int) rnd();
            ib = (int) rnd();
            switch (rnd() % 3) {
            case 0: sa = (int) (rnd() % 12000); break;
            case 1: sa = 5000 + (int) (rnd() % 5000); break;
            default: sa = 0x1FBFD + (int) (rnd() % 0xD0); break;
            }
            switch (op) {
            case 0: r = ft_800877F8(g, ia); break;
            case 1: r = ft_80087818(g, ia); break;
            case 2: r = ft_80087838(g); break;
            case 3: r = ft_80087858(g); break;
            case 4: r = ft_80087878(g, 0x22 + (int) ((u32) ib % 5u)); break;
            case 5: r = ft_800878BC(g); break;
            case 6: r = ft_80087900(g); break;
            case 7: r = ft_80087944(g); break;
            case 8: r = ft_80087988(g); break;
            case 9: r = ft_800879D8(g); break;
            case 10: r = ft_800879F8(g); break;
            case 11: r = ft_80087A18(g); break;
            case 12: r = (u32) ft_80087A80(g); break;
            case 13: r = B(ft_80087A8C(g)); break;
            case 14: r = (u32) ft_80087A98(g); break;
            case 15: r = ft_80087AA4(g); break;
            case 16: r = ft_80087AB4(g); break;
            case 17: ft_80087AC0(g, (int) ((u32) ib % 3u)); break;
            case 18: r = ft_80087AEC(g); break;
            case 19: r = (u32) ft_80087B34(g); break;
            case 20: ft_80087BAC(g, (int) ((u32) ib % 3u)); break;
            case 21: ft_80087BC0(g, ib); break;
            case 22: ft_80087BEC(g, ib); break;
            case 23: r = (u32) ft_GetFtKindMask(); break;
            case 24: r = (u32) ft_80087C58(g); break;
            case 25: ft_80087C64(g, ib); break;
            case 26: r = (u32) ft_80087C70(f0, (int) ((u32) ib % 1000u)); break;
            case 27: r = (u32) ft_80087D0C(f0, sa); break;
            case 28: ft_8008805C(f0, sa); break;
            case 29: ft_80088080(f0); break;
            case 30: ft_800880AC(f0); break;
            case 31: ft_800880D8(f0); break;
            case 32: ft_80088110(f0); break;
            case 33: ft_PlaySFX(f0, sa, (u8) ia, (u8) ib); break;
            }
            for (i = 0; i < NF; i++) {
                Fighter* f = &F[i];
                t = t * 31u + (u32) (f->invisible | f->x2228_b5 << 1);
                t = t * 31u + (u32) f->x1988;
                t = t * 31u + (u32) f->x2164 * 3u + (u32) f->x2168 * 7u;
                t = t * 31u + (u32) f->x2160;
            }
            printf("%d %d %08X %08X %08X\n", op, k, r, t, g_tr);
        }
    }
    return 0;
}
