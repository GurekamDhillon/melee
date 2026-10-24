/* C reference for src/tu/ftcaptainspeciallw.rs: the whole game TU is #included verbatim; its
 * callees outside the TU are recorders both sides share (the Rust test installs the same ones
 * through the crate's hooks). ftParts_GetBoneIndex / ftLib_GetKind are cut verbatim (tu_shard_pre.py).
 * Targets: ftCa_SpecialHi_800E3EAC, ftCa_SpecialLw_Coll, ftCa_SpecialLw_Anim,
 * ftCa_SpecialLwEndAir_Phys (+ ftCa_SpecialLw_Phys / ftCa_SpecialAirLw_Phys which call 3EAC).
 * Prints "iter fn | calls (each ends with ;) | state" (the recorders print as they are called). */
#include "ft/kinds/ftCaptain/ftcaptainspeciallw.c"
#include "tu_ftcaptainspeciallw_extract.inc"
#include <stdio.h>
#include <string.h>
#include <stdarg.h>

FighterPartsTable** ftPartsTable;
int Geno_DefineBaseKind(int kind) { return kind; }
int Geno_DefineTemplateSkeleton(int kind) { (void) kind; return 0; }
void __assert(const char* f, u32 l, const char* c) { (void) f; (void) l; (void) c; for (;;) {} }

#define lg(...) printf(__VA_ARGS__)
static u32 B(float f) { union { float f; u32 u; } c; c.f = f; return c.u; }

static int R_82708, R_81D0C, R_800827A0, R_FRAMES;
void ftCommon_8007D7FC(Fighter* fp) { (void) fp; lg("7D7FC;"); }
void ftCommon_8007D5D4(Fighter* fp) { (void) fp; lg("7D5D4;"); }
void ftCommon_8007DB24(Fighter_GObj* g) { (void) g; lg("7DB24;"); }
void ftCommon_8007E5AC(Fighter* fp) { (void) fp; lg("7E5AC;"); }
void ft_80085088(Fighter_GObj* g) { (void) g; lg("85088;"); }
void ft_80085134(Fighter_GObj* g) { (void) g; lg("85134;"); }
void ft_80084EEC(Fighter_GObj* g) { (void) g; lg("84EEC;"); }
void ft_80084F3C(Fighter_GObj* g) { (void) g; lg("84F3C;"); }
GroundOrAir ft_80082708(Fighter_GObj* g) { (void) g; lg("82708;"); return R_82708; }
bool ft_800827A0(Fighter_GObj* g) { (void) g; lg("827A0;"); return R_800827A0; }
GroundOrAir ft_80081D0C(Fighter_GObj* g) { (void) g; lg("81D0C;"); return R_81D0C; }
void ft_8008A2BC(Fighter_GObj* g) { (void) g; lg("8A2BC;"); }
void ftCo_Fall_Enter(Fighter_GObj* g) { (void) g; lg("Fall;"); }
void ftAnim_8006EBA4(Fighter_GObj* g) { (void) g; lg("6EBA4;"); }
bool ftAnim_IsFramesRemaining(Fighter_GObj* g) { (void) g; lg("Frames;"); return R_FRAMES; }
void ftPartSetRotZ(Fighter* fp, int part, f32 v) { (void) fp; lg("rotz %d %08x;", part, B(v)); }
void ftCommon_CalcGroundAccel_Deaccel(Fighter* fp, float v) { (void) fp; lg("accel %08x;", B(v)); }
void ftCommon_SetSelfMovementFromGroundedMovement(Fighter_GObj* g) { (void) g; lg("selfmove;"); }
void Fighter_ChangeMotionState(Fighter_GObj* g, int ms, u32 fl, float a, float b, float c, Fighter_GObj* d)
{
    (void) g; (void) d;
    lg("ms %d %x %08x %08x %08x;", ms, (unsigned) fl, B(a), B(b), B(c));
}
void efAsync_Spawn(HSD_GObj* gobj, void* q, u32 kind, u32 gfx, HSD_JObj* jobj, ...)
{
    va_list ap;
    float* ang;
    (void) gobj; (void) q;
    va_start(ap, jobj);
    ang = va_arg(ap, float*);
    va_end(ap);
    lg("ef %u %x %u %08x;", (unsigned) kind, (unsigned) gfx, (unsigned) (size_t) jobj, B(*ang));
}
void ftCommon_8007D92C(Fighter_GObj* g) { (void) g; lg("7D92C;"); }
void ft_80084104(Fighter_GObj* g) { (void) g; lg("84104;"); }
void efLib_PauseAll(HSD_GObj* g) { (void) g; }
void efLib_ResumeAll(HSD_GObj* g) { (void) g; }

static u32 s = 0xCA17F0u;
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
static ftCaptain_DatAttrs DA;
static FighterBone PARTS[80];
static FighterPartsTable TBL[64];
static FighterPartsTable* TBLP[64];
static u8 P2J[64][256];

static void state(void)
{
    printf(" | ga %d cv %u %u %u %u thr0 %d b1 %d x2219 %d vel %08x %08x pre %d post %d\n", (int) F.ground_or_air, (unsigned) F.cmd_vars[0], (unsigned) F.cmd_vars[1], (unsigned) F.cmd_vars[2],
           (unsigned) F.cmd_vars[3], F.throw_flags == 0, (int) F.throw_flags_b1, (int) F.x2219_b0,
           B(F.self_vel.x), B(F.self_vel.y), F.pre_hitlag_cb != NULL, F.post_hitlag_cb != NULL);
}

int main(void)
{
    int i, k;
    for (k = 0; k < 64; k++) {
        TBLP[k] = &TBL[k];
        TBL[k].part_to_joint = P2J[k];
    }
    ftPartsTable = TBLP;
    G.user_data = &F;
    F.dat_attrs = &DA;
    F.parts = PARTS;
    for (i = 0; i < 4000; i++) {
        int fn;
        for (k = 0; k < 80; k++) {
            PARTS[k].joint = (HSD_JObj*) (size_t) (0x1000 + k);
        }
        for (k = 0; k < 64; k++) {
            int j;
            for (j = 0; j < 256; j++) {
                P2J[k][j] = (u8) (rnd() % 80);
            }
        }
        switch (rnd() % 4) {
        case 0: F.motion_id = 0x165; break;
        case 1: F.motion_id = 0x167; break;
        case 2: F.motion_id = 0x166; break;
        default: F.motion_id = rnd() % 0x200; break;
        }
        F.kind = (rnd() & 1) ? ((rnd() & 1) ? Ft_Kind_Captain : Ft_Kind_Ganon) : (int) (rnd() % Ft_Kind_Mex0);
        F.ground_or_air = (rnd() & 1) ? GA_Ground : GA_Air;
        F.throw_flags = rnd();
        F.throw_flags_b1 = rnd() & 1;
        F.x2219_b0 = rnd() & 1;
        F.cmd_vars[0] = (rnd() & 1) ? 0 : (rnd() % 3);
        F.cmd_vars[1] = (rnd() & 1) ? 0 : (rnd() % 3);
        F.cmd_vars[2] = (rnd() & 1) ? 0 : (rnd() % 3);
        F.cmd_vars[3] = rnd() % 3;
        F.facing_dir = (rnd() & 1) ? 1.0f : -1.0f;
        F.coll_data.env_flags = rnd();
        if (rnd() & 1) {
            F.coll_data.env_flags |= (rnd() & 1) ? Collide_RightWallHug : Collide_LeftWallHug;
        }
        F.self_vel.x = rf();
        F.self_vel.y = rf();
        F.mv.ca.speciallw.friction = rf();
        F.pre_hitlag_cb = NULL;
        F.post_hitlag_cb = NULL;
        DA.speciallw_flame_particle_angle = rf() * 90.0f;
        DA.speciallw_ground_lag_mul = rf();
        R_82708 = rnd() % 3;
        R_81D0C = rnd() % 3;
        R_800827A0 = rnd() & 1;
        R_FRAMES = rnd() & 1;
        fn = i % 5;
        switch (fn) {
        case 0: printf("%d 3EAC |", i); ftCa_SpecialHi_800E3EAC(&G); state(); break;
        case 1: printf("%d LwColl |", i); ftCa_SpecialLw_Coll(&G); state(); break;
        case 2: printf("%d LwAnim |", i); ftCa_SpecialLw_Anim(&G); state(); break;
        case 3: printf("%d EndAirPhys |", i); ftCa_SpecialLwEndAir_Phys(&G); state(); break;
        default:
            if (i & 5) { printf("%d LwPhys |", i); ftCa_SpecialLw_Phys(&G); state(); }
            else { printf("%d AirLwPhys |", i); ftCa_SpecialAirLw_Phys(&G); state(); }
            break;
        }
    }
    return 0;
}
