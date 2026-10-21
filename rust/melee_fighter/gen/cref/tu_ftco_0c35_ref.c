/* C reference for src/tu/ftco_0c35.rs: ftCo_0C35.c (all of it) + ftCommon_800804FC cut verbatim
 * (tu_shard97.py). Callees outside the ported set fold their arguments into the trace TR;
 * tests/tu_ftco_0c35.rs installs the same stubs. cosf / sinf are gekko_fp.c's (linked by run.py). */
#include "ft/types.h"
#include "ft/forward.h"
#include "ft/fighter.h"
#include "ft/ftcommon.h"
#include "ft/ftparts.h"
#include "ft/kinds/ftCommon/ftCo_Damage.h"
#include <sysdolphin/baselib/jobj.h>
#include <stdio.h>

void __assert(const char* f, u32 l, const char* c) { (void) f; (void) l; (void) c; for (;;) {} }
static u32 TR;
static void T(u32 v) { TR = TR * 31u + v; }
static u32 B(float f) { union { float f; u32 u; } c; c.f = f; return c.u; }

static Fighter F;
static HSD_GObj GO;
static HSD_JObj J;
static FighterBone PB[8];
static ftCommonData CD;
ftCommonData* p_ftCommonData;
static float ANG;
static int BONE;

void Fighter_UnkRecursiveFunc_8006D044(Fighter_GObj* g) { T(1); T(g == &GO); }
float ftCo_Damage_CalcAngle(Fighter* fp, float kb) { T(2); T(fp == &F); T(B(kb)); return ANG; }
void ftCo_Damage_CalcVel(Fighter* fp, float x, float y) { T(3); T(fp == &F); T(B(x)); T(B(y)); }
Fighter_Part ftParts_GetBoneIndex(Fighter* fp, Fighter_Part part) { T(4); T(fp == &F); T((u32) part); return BONE; }
void ftPartSetRotY(Fighter* fp, int part, float r) { T(5); T(fp == &F); T((u32) part); T(B(r)); }
void HSD_JObjSetMtxDirtySub(HSD_JObj* j) { T(6); T(j == &J); }
static void cb21F8(HSD_GObj* g) { T(7); T(g == &GO); }

#include "tu_ftco_0c35_extract.inc"

static u32 rs = 0x51ED2701u;
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

int main(void)
{
    int i;
    p_ftCommonData = &CD;
    GO.user_data = &F;
    F.parts = PB;
    for (i = 0; i < 6000; i++) {
        u32 op = rnd() % 3;
        TR = 0;
        CD.x648 = (s32) (rnd() % 40) - 2;
        CD.x64C = rf();
        CD.x650 = rf();
        CD.x654 = rf();
        ANG = rf();
        BONE = (int) (rnd() % 8);
        F.ground_or_air = rnd() % 2;
        F.dmg.x18F4 = (s32) (rnd() % 42) - 2;
        F.dmg.x1954 = rf();
        F.dmg.kb_applied = rf();
        F.dmg.facing_dir_1 = rf();
        F.dmg.x18c4_source_ply = 0;
        F.dmg.x18C8 = 0;
        F.specialn_facing_dir = rf();
        F.coll_data.floor.normal.x = rf();
        F.coll_data.floor.normal.y = rf();
        F.facing_dir = rf();
        F.xF0_ground_kb_vel = rf();
        F.x2220_b4 = rnd() % 2;
        F.x21F8 = rnd() % 2 ? cb21F8 : NULL;
        J.flags = rnd() & (JOBJ_MTX_INDEP_SRT | JOBJ_MTX_DIRTY | JOBJ_USER_DEF_MTX);
        J.rotate.y = 7.0f;
        PB[BONE].x4_jobj2 = &J;
        if (op == 0) ftCo_800C3598(&GO);
        else if (op == 1) ftCo_800C36DC(&GO);
        else ftCo_800C37A0(&GO);
        printf("%d %u %08X %d %08X %d %08X %08X %08X %d %d\n", i, op, TR, F.dmg.x18F4, B(F.dmg.x1954),
               (int) F.x2220_b4, B(F.xF0_ground_kb_vel), B(F.facing_dir), B(J.rotate.y),
               F.dmg.x18c4_source_ply, F.dmg.x18C8);
    }
    return 0;
}
