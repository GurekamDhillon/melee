/* C reference for src/tu/ftco_0d8e.rs: ftCo_0D8E.c from its local attrs struct to EOF, cut
 * verbatim (tu_shard97.py). Callees fold their arguments into the trace TR; the Rust test
 * installs the same stubs. fn_800D8EC8's wall-probe end is the fused f64 site. */
#include "ft/types.h"
#include "ft/forward.h"
#include "ft/fighter.h"
#include "ft/ft_0877.h"
#include "ft/ft_0892.h"
#include "ft/ftparts.h"
#include "ft/inlines.h"
#include "ft/kinds/ftLink/ftlink.h"
#include "it/inlines.h"
#include "it/types.h"
#include "it/kinds/itlinkhookshot.h"
#include "lb/lb_00B0.h"
#include "mp/mplib.h"
#include <sysdolphin/baselib/jobj.h>
#include <stdio.h>

void __assert(const char* f, u32 l, const char* c) { (void) f; (void) l; (void) c; for (;;) {} }
static u32 TR;
static void T(u32 v) { TR = TR * 31u + v; }
static u32 B(float f) { union { float f; u32 u; } c; c.f = f; return c.u; }

static Fighter F;
static HSD_GObj GO, IG[2];
static Item IT[2];
static Article ART;
static float SA[64];
static s32 DA[64];
static HSD_JObj J;
static FighterBone PB[8];
static int BONE, SPAWN, REMAP;
static Vec3 LBV;

static u32 E(void* p)
{
    if (p == NULL) return 0;
    if (p == &IG[0]) return 1;
    if (p == &IG[1]) return 2;
    if (p == &GO) return 3;
    if (p == &J) return 4;
    return 99;
}

Fighter_Part ftParts_GetBoneIndex(Fighter* fp, Fighter_Part part) { T(20); T(fp == &F); T((u32) part); return BONE; }
void lb_8000B1CC(HSD_JObj* j, Vec3* off, Vec3* out) { T(1); T(E(j)); T(off == NULL); *out = LBV; }
Item_GObj* it_802A2BA4(Fighter_GObj* g, Vec3* pos, f32 f, int x)
{
    T(2); T(E(g)); T(B(pos->x)); T(B(pos->y)); T(B(pos->z)); T(B(f)); T((u32) x);
    return SPAWN < 2 ? &IG[SPAWN] : NULL;
}
void ft_8008A2BC(Fighter_GObj* g) { T(3); T(E(g)); }
void HSD_JObjSetupMatrixSub(HSD_JObj* j) { T(4); T(E(j)); }
bool mpCheckAllRemap(Vec3* a, int* b, u32* c, Vec3* d, int skip, int only, float x0, float y0, float x1, float y1)
{
    T(5); T(a == NULL && b == NULL && c == NULL && d == NULL); T((u32) skip); T((u32) only);
    T(B(x0)); T(B(y0)); T(B(x1)); T(B(y1));
    return REMAP;
}
void it_802A2B10(Item_GObj* g) { T(6); T(E(g)); }
void it_802A78B8(HSD_GObj* g, Vec3* v) { T(7); T(E(g)); T(B(v->x)); T(B(v->y)); T(B(v->z)); }
void ft_PlaySFX(Fighter* fp, enum_t id, u8 vol, u8 pan) { T(8); T(fp == &F); T((u32) id); T(vol); T(pan); }
void it_802A77DC(Item_GObj* g) { T(9); T(E(g)); }
bool ftLk_Init_BoomerangExists(HSD_GObj* g) { T(10); T(E(g)); return false; }
void it_802A7AF0(HSD_GObj* g) { (void) g; }
void it_802A7AAC(HSD_GObj* g) { (void) g; }
void it_802A7B34(HSD_GObj* g) { (void) g; }

#include "tu_ftco_0d8e_extract.inc"

static u32 rs = 0x0D8E0D8Eu;
static u32 rnd(void) { rs ^= rs << 13; rs ^= rs >> 17; rs ^= rs << 5; return rs; }
static float rf(void)
{
    switch (rnd() % 10) {
    case 0: return 0.0f;
    case 1: return -0.0f;
    case 2: return 1.0f;
    case 3: return -1.0f;
    default: return (float) ((int) (rnd() % 20001) - 10000) / 1013.0f;
    }
}
static const FighterKind KINDS[4] = { Ft_Kind_Link, Ft_Kind_CLink, Ft_Kind_Samus, Ft_Kind_Mario };

int main(void)
{
    int i, k;
    GO.user_data = &F;
    F.parts = PB;
    F.dat_attrs = DA;
    ART.x4_specialAttributes = SA;
    for (k = 0; k < 2; k++) { IG[k].user_data = &IT[k]; IT[k].xC4_article_data = &ART; }
    for (i = 0; i < 8000; i++) {
        u32 op = rnd() % 5, ret = 0;
        TR = 0;
        for (k = 0x84 / 4; k <= 0xBC / 4; k++) DA[k] = (s32) (rnd() % 8);
        for (k = 0; k < 64; k++) SA[k] = rf();
        BONE = (int) (rnd() % 8);
        SPAWN = (int) (rnd() % 3);
        REMAP = (int) (rnd() % 2);
        LBV.x = rf(); LBV.y = rf(); LBV.z = rf();
        F.kind = KINDS[rnd() % 4];
        F.mv.co.catch_.x0 = (float) (int) (rnd() % 9) - 1.0f;
        F.facing_dir = rf();
        F.x34_scale.y = rf();
        F.coll_data.cur_pos.x = rf();
        F.u.lk.xC = (rnd() % 3 == 0) ? NULL : &IG[rnd() % 2];
        F.accessory2_cb = NULL; F.death1_cb = NULL; F.accessory3_cb = NULL;
        J.flags = rnd() & (JOBJ_MTX_DIRTY | JOBJ_USER_DEF_MTX);
        for (k = 0; k < 4; k++) { J.mtx[0][k] = rf(); J.mtx[1][k] = rf(); J.mtx[2][k] = rf(); }
        PB[BONE].joint = &J;
        if (op <= 1 && F.u.lk.xC == NULL) F.u.lk.xC = &IG[0];
        switch (op) {
        case 0: ret = fn_800D8EC8(&GO); break;
        case 1: ret = fn_800D9228(&GO); break;
        case 2: fn_800D949C(&GO); break;
        case 3: ftCo_800D94D8(&GO); break;
        default: ret = fn_800D952C(&GO); break;
        }
        printf("%d %u %u %08X %08X %u %d%d%d\n", i, op, ret, TR, B(F.mv.co.catch_.x0), E(F.u.lk.xC),
               F.accessory2_cb == it_802A7AF0, F.death1_cb == it_802A7AAC, F.accessory3_cb == it_802A7B34);
    }
    return 0;
}
