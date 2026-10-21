/* C reference for src/tu/ftafterimage.rs: src/melee/ft/ftafterimage.c and src/melee/lb/lbvector.c
 * #included verbatim (shipping contraction via tu_run.py). The GX calls are macro'd (after the
 * headers, before the TU body) to no-ops, except GXBegin / GXPosition3f32 / GXColor4u8, which
 * print what the strip hands to GX. Link with random ring entries and random trail params. */
#include "ft/ftafterimage.h"
#include "ft/inlines.h"
#include "ft/kinds/ftLink/types.h"
#include <dolphin/gx.h>
#include <dolphin/mtx.h>
#include <melee/lb/lbvector.h>
#include <sysdolphin/baselib/cobj.h>
#include <sysdolphin/baselib/state.h>

static u32 B(float f) { union { float f; u32 u; } c; c.f = f; return c.u; }
static void rec_begin(int n) { printf(" N %d", n); }
static void rec_pos(float x, float y, float z) { printf(" %08x %08x %08x", B(x), B(y), B(z)); }
static void rec_clr(u8 r, u8 g, u8 b, u8 a) { printf(" %02x%02x%02x%02x", r, g, b, a); }
static HSD_CObj g_cobj;

#define GXSetColorUpdate(...) ((void) 0)
#define GXSetAlphaUpdate(...) ((void) 0)
#define GXSetBlendMode(...) ((void) 0)
#define GXSetAlphaCompare(...) ((void) 0)
#define GXSetZMode(...) ((void) 0)
#define GXSetZCompLoc(...) ((void) 0)
#define GXSetNumTexGens(...) ((void) 0)
#define GXSetTevClampMode(...) ((void) 0)
#define GXSetNumTevStages(...) ((void) 0)
#define GXSetTevOrder(...) ((void) 0)
#define GXSetTevOp(...) ((void) 0)
#define GXSetNumChans(...) ((void) 0)
#define GXSetChanCtrl(...) ((void) 0)
#define GXSetCullMode(...) ((void) 0)
#define GXClearVtxDesc(...) ((void) 0)
#define GXSetVtxAttrFmt(...) ((void) 0)
#define GXSetVtxDesc(...) ((void) 0)
#define GXLoadPosMtxImm(...) ((void) 0)
#define GXSetCurrentMtx(...) ((void) 0)
#define HSD_CObjGetCurrent() (&g_cobj)
#define HSD_StateInvalidate(...) ((void) 0)
#define GXBegin(a, b, n) rec_begin(n)
#define GXPosition3f32 rec_pos
#define GXColor4u8 rec_clr

#include "ft/ftafterimage.c"
#include "lb/lbvector.c"
#include "mtx_pc.c"
#include "lb/lbtrigf.c" /* the game's acosf (else the CRT one links) */
/* src/MSL/float.c's tables (int-typed there), as the same bits: 0x7FFFFFFF, 0x7F800000 */
float MSL_TrigF_80400770[] = { __builtin_nanf("0x3fffff") };
float MSL_TrigF_80400774[] = { __builtin_inff() };

static u32 s = 0x2468aceu;
static u32 rnd(void) { s ^= s << 13; s ^= s >> 17; s ^= s << 5; return s; }
static float rf(float k) { return ((float) ((int) (rnd() % 20001) - 10000)) / k; }
static float rcomp(void)
{
    u32 m = rnd() % 8;
    if (m == 0) return 0.0f;
    if (m == 1) return -0.0f;
    return rf(10000.0f);
}

int main(void)
{
    static Fighter F;
    static HSD_GObj G;
    static ftLk_DatAttrs DA;
    int i, j;
    G.user_data = &F;
    F.dat_attrs = &DA;
    F.kind = Ft_Kind_Link;
    for (i = 0; i < 3000; i++) {
        F.x2100 = (s8) (rnd() % 5) - 1;
        F.x2101_bits_0to6 = rnd() % 3;
        F.x20F8 = rf(1000.0f);
        F.x20FC = rf(1000.0f);
        DA.x64.x0 = rf(5000.0f);
        DA.x64.x4 = rf(5000.0f);
        DA.x64.x8 = rnd(); DA.x64.x9 = rnd(); DA.x64.xA = rnd(); DA.x64.xB = rnd();
        DA.x64.xC = rnd(); DA.x64.xE = rnd(); DA.x64.xF = rnd(); DA.x64.x10 = rnd();
        for (j = 0; j < 3; j++) {
            F.x20B0[j].x0.x = rf(100.0f); F.x20B0[j].x0.y = rf(100.0f); F.x20B0[j].x0.z = rf(100.0f);
            F.x20B0[j].xC.x = rcomp(); F.x20B0[j].xC.y = rcomp(); F.x20B0[j].xC.z = rcomp();
            if (rnd() % 4 == 0 && j > 0) F.x20B0[j].xC = F.x20B0[j - 1].xC;
        }
        printf("C %d %d %08x %08x %08x %08x %02x%02x%02x%02x%02x%02x%02x%02x", F.x2100,
               F.x2101_bits_0to6, B(F.x20F8), B(F.x20FC), B(DA.x64.x0), B(DA.x64.x4), DA.x64.x8,
               DA.x64.x9, DA.x64.xA, DA.x64.xB, DA.x64.xC, DA.x64.xE, DA.x64.xF, DA.x64.x10);
        for (j = 0; j < 3; j++)
            printf(" %08x %08x %08x %08x %08x %08x", B(F.x20B0[j].x0.x), B(F.x20B0[j].x0.y),
                   B(F.x20B0[j].x0.z), B(F.x20B0[j].xC.x), B(F.x20B0[j].xC.y), B(F.x20B0[j].xC.z));
        printf(" |");
        ftCo_800C2600(&G, 2);
        printf("\n");
    }
    return 0;
}
