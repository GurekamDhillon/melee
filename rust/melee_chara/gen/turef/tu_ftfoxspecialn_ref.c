/* C reference for src/tu/ftfoxspecialn.rs: the whole game TU is #included verbatim; its callees
 * outside the TU are recorders both sides share (the Rust test installs the same ones through the
 * crate's hooks). Targets: ftFx_SpecialN_CreateBlasterShot, ftFx_Special{N,AirN}Start_Anim,
 * ftFx_Special{N,AirN}Loop_Anim, ftFx_Special{N,AirN}{Start,Loop}_IASA.
 * Prints "iter fn | calls (each ends with ;) | state" (the recorders print as they are called). */
#include "ft/kinds/ftFox/ftfoxspecialn.c"
#include <stdio.h>

void __assert(const char* f, u32 l, const char* c) { (void) f; (void) l; (void) c; for (;;) {} }
static u32 B(float f) { union { float f; u32 u; } c; c.f = f; return c.u; }

static int R_FRAMES, R_KIND;
void ft_800892A0(Fighter_GObj* g) { (void) g; printf("892A0;"); }
void ft_80089824(Fighter_GObj* g) { (void) g; printf("89824;"); }
bool ftFx_Init_800E5534(HSD_GObj* g) { (void) g; printf("E5534;"); return 0; }
void ftFx_Init_800E5588(HSD_GObj* g) { (void) g; }
void ft_8008A2BC(Fighter_GObj* g) { (void) g; printf("wait;"); }
void ftCo_Fall_Enter(Fighter_GObj* g) { (void) g; printf("Fall;"); }
void ftCo_80096900(Fighter_GObj* g, int a, int b, bool c, float d, float e)
{
    (void) g;
    printf("fallspecial %d %d %d %08x %08x;", a, b, (int) c, B(d), B(e));
}
void ftpickupitem_80094818(Fighter_GObj* g, bool b) { (void) g; printf("pickup %d;", (int) b); }
void ftCommon_8007D7FC(Fighter* fp) { (void) fp; printf("7D7FC;"); }
void ft_80084F3C(Fighter_GObj* g) { (void) g; printf("84F3C;"); }
void ft_80084EEC(Fighter_GObj* g) { (void) g; printf("84EEC;"); }
void ft_80085134(Fighter_GObj* g) { (void) g; printf("85134;"); }
void ft_80085088(Fighter_GObj* g) { (void) g; printf("85088;"); }
void ftAnim_8006EBA4(Fighter_GObj* g) { (void) g; printf("6EBA4;"); }
bool ftAnim_IsFramesRemaining(Fighter_GObj* g) { (void) g; printf("Frames;"); return R_FRAMES; }
void Fighter_ChangeMotionState(Fighter_GObj* g, int ms, u32 fl, float a, float b, float c, Fighter_GObj* d)
{
    (void) g; (void) d;
    printf("ms %d %x %08x %08x %08x;", ms, (unsigned) fl, B(a), B(b), B(c));
}
int ftParts_GetBoneIndex_stub(int p) { return p ^ 5; }
Fighter_Part ftParts_GetBoneIndex(Fighter* fp, Fighter_Part part) { (void) fp; return (Fighter_Part) (part ^ 5); }
FighterKind ftLib_GetKind(HSD_GObj* g) { (void) g; return R_KIND; }
void ft_PlaySFX(Fighter* fp, enum_t id, u8 vol, u8 pan) { (void) fp; printf("sfx %d %d %d;", (int) id, (int) vol, (int) pan); }
void lb_8000B1CC(HSD_JObj* j, Vec3* off, Vec3* out)
{
    printf("hold %u %08x %08x %08x;", (unsigned) (size_t) j, B(off->x), B(off->y), B(off->z));
    out->x = off->z;
    out->y = off->y;
    out->z = 9.0f;
}
void it_8029C6A4(f32 angle, f32 vel, HSD_GObj* parent, Vec3* vec, int kind)
{
    (void) parent;
    printf("shot %08x %08x %08x %08x %08x %d;", B(angle), B(vel), B(vec->x), B(vec->y), B(vec->z), kind);
}
void it_802ADDD0(Item_GObj* it, s32 v) { printf("vis %u %d;", (unsigned) (size_t) it, (int) v); }
void it_802AE538(Item_GObj* it) { (void) it; printf("538;"); }
void it_802AE608(Item_GObj* it) { (void) it; printf("608;"); }
void it_802AE1D0(Item_GObj* it) { (void) it; printf("1D0;"); }
void it_802AEAB4(Item_GObj* it) { (void) it; printf("remove;"); }
HSD_GObj* it_802AE8A8(f32 d, HSD_GObj* g, Vec3* p, int bone, int k) { (void) g; (void) p; printf("spawn %08x %d %d;", B(d), bone, k); return NULL; }
void it_8026BAE8(Item_GObj* i, float s) { (void) i; printf("scale;"); }
void ft_80084DB0(Fighter_GObj* g) { (void) g; }
void ft_80083F88(Fighter_GObj* g) { (void) g; }
void ftCo_AirCatchHit_Coll(Fighter_GObj* g) { (void) g; }
void it_8029C6CC(f32 angle, f32 vel, HSD_GObj* parent, Vec3* vec, int kind) { (void) angle; (void) vel; (void) parent; (void) vec; (void) kind; }
int Geno_DefineBaseKind(int kind) { return kind; }
int Geno_DefineTemplateSkeleton(int kind) { (void) kind; return 0; }
void OSReport(char* fmt, ...) { (void) fmt; printf("osreport;"); }

static u32 s = 0xF0C5A1u;
static u32 rnd(void) { s ^= s << 13; s ^= s >> 17; s ^= s << 5; return s; }
static float rf(void)
{
    switch (rnd() % 8) {
    case 0: return 0.0f;
    case 1: return -0.0f;
    case 2: return 1.0f;
    case 3: return -1.0f;
    default: return (float) ((int) (rnd() % 4001) - 2000) / 997.0f;
    }
}

static Fighter F;
static HSD_GObj G;
static ftFox_DatAttrs DA;
static FighterBone PARTS[80];

static void state(void)
{
    printf(" | cv %u %u %u %u loop %d blaster %u acc4 %d x21EC %d dmg %d %d btn %d\n", (unsigned) F.cmd_vars[0],
           (unsigned) F.cmd_vars[1], (unsigned) F.cmd_vars[2], (unsigned) F.cmd_vars[3],
           (int) F.mv.fx.SpecialN.isBlasterLoop, (unsigned) (size_t) F.u.fx.x222C_blasterGObj,
           F.accessory4_cb != NULL, F.x21EC != NULL, F.take_dmg_cb != NULL, F.death2_cb != NULL,
           (int) F.input.pressed_buttons);
}

int main(void)
{
    int i, k;
    G.user_data = &F;
    F.dat_attrs = &DA;
    F.parts = PARTS;
    for (i = 0; i < 4000; i++) {
        for (k = 0; k < 80; k++) {
            PARTS[k].joint = (HSD_JObj*) (size_t) (0x1000 + k);
        }
        for (k = 0; k < 4; k++) {
            F.cmd_vars[k] = (rnd() & 1) ? 0 : (rnd() % 3);
        }
        F.mv.fx.SpecialN.isBlasterLoop = (rnd() % 3 == 0) ? (rnd() % 3) : (rnd() & 1);
        F.input.pressed_buttons = (rnd() & 1) ? HSD_PAD_B : (rnd() & 0xFFFF);
        if (rnd() & 1) {
            F.input.pressed_buttons |= HSD_PAD_B;
        }
        F.facing_dir = (rnd() & 1) ? 1.0f : -1.0f;
        F.u.fx.x222C_blasterGObj = (rnd() & 1) ? (HSD_GObj*) (size_t) (0x5150 + (rnd() % 4)) : NULL;
        F.accessory4_cb = NULL;
        F.x21EC = NULL;
        F.take_dmg_cb = NULL;
        F.death2_cb = NULL;
        DA.x10_FOX_BLASTER_ANGLE = rf();
        DA.x14_FOX_BLASTER_VEL = rf();
        DA.x1C_FOX_BLASTER_SHOT_ITKIND = (int) (rnd() % 50);
        F.cur_pos.x = rf();
        F.cur_pos.y = rf();
        F.cur_pos.z = rf();
        R_FRAMES = rnd() & 1;
        R_KIND = (rnd() % 3 == 0) ? Ft_Kind_Falco : ((rnd() % 3 == 0) ? (int) (rnd() % 30) : Ft_Kind_Fox);
        switch (i % 9) {
        case 0: printf("%d Shot |", i); ftFx_SpecialN_CreateBlasterShot(&G); break;
        case 1: printf("%d NStartAnim |", i); ftFx_SpecialNStart_Anim(&G); break;
        case 2: printf("%d AirNStartAnim |", i); ftFx_SpecialAirNStart_Anim(&G); break;
        case 3: printf("%d NLoopAnim |", i); ftFx_SpecialNLoop_Anim(&G); break;
        case 4: printf("%d AirNLoopAnim |", i); ftFx_SpecialAirNLoop_Anim(&G); break;
        case 5: printf("%d NStartIASA |", i); ftFx_SpecialNStart_IASA(&G); break;
        case 6: printf("%d AirNStartIASA |", i); ftFx_SpecialAirNStart_IASA(&G); break;
        case 7: printf("%d NLoopIASA |", i); ftFx_SpecialNLoop_IASA(&G); break;
        default: printf("%d AirNLoopIASA |", i); ftFx_SpecialAirNLoop_IASA(&G); break;
        }
        state();
    }
    return 0;
}
