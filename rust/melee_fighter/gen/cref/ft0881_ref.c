/* C reference for src/ft0881.rs (port round 6): ft_0881.c ft_800881D8..ft_80088C5C (with its
 * TARGET_PC Geno voice macro) cut verbatim into ft0881_extract.inc by run.py, plus all of
 * ft_0877.c (ft_80087D0C) and ftCommon_80080144. Stubs as in ft0877_ref.c, plus the three
 * lbAudioAx channel calls. No fused sites in ft_0881.c. Six fighters, a random fighter p-link list, random fields, one op per case.
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
#include "ft/ft_0881.h"
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

int lbAudioAx_80023870(int s, int v, int p, int c) { T(0x410u); T((u32) s); T((u32) v); T((u32) p); T((u32) c); return (int) (g_tr >> 6) - 3; }
bool lbAudioAx_80026510(HSD_GObj* g) { T(0x420u + E(g)); return (g_tr >> 2) & 1; }
bool lbAudioAx_800265C4(HSD_GObj* g, int s) { T(0x430u + E(g)); T((u32) s); return (g_tr >> 5) % 3u == 1 ? 1 : (g_tr >> 5) % 3u == 2 ? 2 : 0; }

#include "ft0881_extract.inc"

int printf(const char*, ...);
static void zero(void* p, unsigned n) { unsigned char* b = p; while (n--) *b++ = 0; }

static u32 rs;
static u32 rnd(void) { rs ^= rs << 13; rs ^= rs >> 17; rs ^= rs << 5; return rs; }

static const int KINDS[8] = { 0, 2, 13, 24, 20, 33, 34, 8 };
static s32 IDS[5];
static FtSFXArr ARR;
#define NOPS 17

static int rch(void) { return (rnd() % 3) == 0 ? -1 : (int) (rnd() % 1000); }

int main(void)
{
    int op, k, i;
    HSD_GObjPLinkHead = plinks;
    for (op = 0; op < NOPS; op++) {
        for (k = 0; k < 300; k++) {
            u32 t = 0;
            int sa;
            u8 va, pa;
            HSD_GObj* g;
            Fighter* f0;
            zero(F, sizeof F); zero(GO, sizeof GO); zero(IT, sizeof IT); zero(plinks, sizeof plinks);
            rs = 0x13579u + (u32) op * 7919u + (u32) k * 104729u;
            g_tr = rnd();
            for (i = 0; i < 5; i++) IDS[i] = (rnd() % 4) == 0 ? 0x83D61 : (s32) (rnd() % 12000);
            ARR.num = 1 + (int) (rnd() % 5);
            ARR.sfx_ids = IDS;
            for (i = 0; i < NF; i++) {
                Fighter* f = &F[i];
                GO[i].user_data = f;
                f->gobj = &GO[i];
                f->kind = KINDS[rnd() % 8];
                f->x619_costume_id = (u8) (rnd() % 4);
                f->player_id = (u8) (rnd() % 6);
                f->is_sub_fighter = rnd() & 1;
                f->x2225_b6 = rnd() & 1;
                f->is_metal = rnd() & 1;
                f->x2220_b5 = rnd() & 1; f->x2220_b6 = rnd() & 1;
                f->x34_scale.x = 1.0f; f->x34_scale.y = (rnd() & 1) ? 1.0f : 1.5f;
                f->x2144 = rch(); f->x2148 = rch(); f->x214C = rch(); f->x2150 = rch();
                f->x2154 = rch(); f->x2158 = rch(); f->x215C = rch(); f->x2160 = rch();
                f->x2164 = (int) (rnd() % 100);
                f->x2168 = (int) (rnd() % 100);
            }
            g = &GO[rnd() % NF];
            f0 = g->user_data;
            switch (rnd() % 4) {
            case 0: sa = 0x83D60; break;
            case 1: sa = 0x83D61; break;
            case 2: sa = 5000 + (int) (rnd() % 5000); break;
            default: sa = (int) (rnd() % 12000); break;
            }
            va = (u8) rnd(); pa = (u8) rnd(); /* locals: MS-ABI clang evaluates call args right to left */
            switch (op) {
            case 0: ft_800881D8(f0, sa, va, pa); break;
            case 1: ft_80088328(f0, sa, va, pa); break;
            case 2: ft_80088478(f0, sa, va, pa); break;
            case 3: ft_80088510(f0, sa, va, pa); break;
            case 4: ft_800885A8(f0, sa, va, pa); break;
            case 5: ft_80088640(f0, sa, va, pa); break;
            case 6: ftCo_800886D8(f0, sa, va, pa); break;
            case 7: ft_80088770(f0); break;
            case 8: ft_800887CC(f0); break;
            case 9: ft_80088828(f0); break;
            case 10: ft_80088884(f0); break;
            case 11: ft_800888E0(f0); break;
            case 12: ft_8008893C(f0); break;
            case 13: ft_80088998(f0); break;
            case 14: ft_800889F4(f0, (rnd() % 4) == 0 ? NULL : &ARR); break;
            case 15: ft_80088A50(f0); break;
            case 16: ft_80088C5C(g); break;
            }
            for (i = 0; i < NF; i++) {
                Fighter* f = &F[i];
                t = t * 31u + (u32) f->x2144; t = t * 31u + (u32) f->x2148;
                t = t * 31u + (u32) f->x214C; t = t * 31u + (u32) f->x2150;
                t = t * 31u + (u32) f->x2154; t = t * 31u + (u32) f->x2158;
                t = t * 31u + (u32) f->x215C; t = t * 31u + (u32) f->x2160;
                t = t * 31u + (u32) f->x2164; t = t * 31u + (u32) f->x2168;
            }
            printf("%d %d %08X %08X\n", op, k, t, g_tr);
        }
    }
    return 0;
}
