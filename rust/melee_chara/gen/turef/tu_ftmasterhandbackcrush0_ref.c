/* C reference for src/tu/ftmasterhandbackcrush0.rs: the whole game TU is #included verbatim
 * (shipping contraction via run.py). HSD_JObjGetScale/SetScale are the real jobj.h inlines over a
 * stand-in joint. Prints
 * "init x70 x74 x78 x7C sclx scly sclz -> cury x0 i70 x64 x58 (jobj scale after)" and
 * "anim i70 cmd x64 x58 sclx.. -> i70 cmd scl" lines. */
#include "ft/kinds/ftMasterHand/ftmasterhandbackcrush0.c"
#include <stdio.h>

static Vec3 TGT;
void ft_80085134(HSD_GObj* g) { (void) g; }
void ftBossLib_8015C208(HSD_GObj* g, Vec3* out) { (void) g; *out = TGT; }
void ftBossLib_8015BD20(HSD_GObj* g) { (void) g; }
void ftBossLib_8015BF74(HSD_GObj* g, float f) { (void) g; (void) f; }
void ftBossLib_8015C190(HSD_GObj* g) { (void) g; }
void Fighter_ChangeMotionState(HSD_GObj* g, int m, u32 f, float a, float b, float c, HSD_GObj* d) { (void) g; (void) m; (void) f; (void) a; (void) b; (void) c; (void) d; }
void ftAnim_8006EBA4(HSD_GObj* g) { (void) g; }
int ftAnim_IsFramesRemaining(HSD_GObj* g) { (void) g; return 1; }
int Player_GetPlayerSlotType(int slot) { return slot; }
void ftMh_MS_389_80151018(HSD_GObj* g) { (void) g; }
void HSD_JObjSetMtxDirtySub(HSD_JObj* j) { (void) j; }
void __assert(const char* f, u32 l, const char* c) { (void) f; (void) l; (void) c; for (;;) {} }

static u32 s = 0xC2A5Du;
static u32 rnd(void) { s ^= s << 13; s ^= s >> 17; s ^= s << 5; return s; }
static float rf(void)
{
    u32 k = rnd() % 16;
    if (k == 0) return 0.0f;
    if (k == 1) return -0.0f;
    if (k == 2) return (float) (rnd() % 7) * 1e-4f;
    return ((float) ((int) (rnd() % 20001) - 10000)) / 37.0f;
}
static u32 B(float f) { union { float f; u32 u; } c; c.f = f; return c.u; }

int main(void)
{
    static Fighter F;
    static HSD_GObj G;
    static ftData FD;
    static ftMasterHand_SpecialAttrs DA;
    static HSD_JObj J;
    int i, k;
    G.user_data = &F;
    G.hsd_obj = &J;
    F.ft_data = &FD;
    FD.ext_attr = &DA;
    J.flags = JOBJ_MTX_INDEP_SRT;
    for (i = 0; i < 2000; i++) {
        J.scale.x = rf(); J.scale.y = rf(); J.scale.z = rf();
        DA.x70 = (int) (rnd() % 500) - 100;
        DA.x74 = (int) (rnd() % 500) - 100;
        DA.x78 = rf();
        DA.x7C = (int) (rnd() % 40) - 5;
        if (DA.x7C == 0 && (rnd() & 1)) DA.x7C = 1;
        TGT.x = rf(); TGT.y = rf(); TGT.z = rf();
        printf("init %d %d %08x %d %08x %08x %08x", DA.x70, DA.x74, B(DA.x78), DA.x7C, B(J.scale.x), B(J.scale.y), B(J.scale.z));
        ftMh_MS_370_80153D2C(&G);
        printf(" -> %08x %08x %d %08x %08x %08x %08x %08x %08x %08x %08x %08x %d\n", B(F.cur_pos.x), B(F.cur_pos.y),
               F.mv.mh.unk0.x70, B(F.mv.mh.unk0.x0), B(F.mv.mh.unk0.x64.x), B(F.mv.mh.unk0.x64.y), B(F.mv.mh.unk0.x64.z),
               B(F.mv.mh.unk0.x58.x), B(F.mv.mh.unk0.x58.y), B(F.mv.mh.unk0.x58.z), B(J.scale.x), B(J.scale.y), F.cmd_vars[0]);
        for (k = 0; k < 3; k++) {
            printf("anim %d %d", F.mv.mh.unk0.x70, F.cmd_vars[0]);
            ftMh_BackCrush_Anim(&G);
            printf(" -> %d %d %08x %08x %08x\n", F.mv.mh.unk0.x70, F.cmd_vars[0], B(J.scale.x), B(J.scale.y), B(J.scale.z));
        }
    }
    return 0;
}
