/* C reference for src/ftdmgfall.rs (group 3 round 14): ftCo_DamageFall.c's ftCo_80090574,
 * static calcShift, ftCo_80090594, ftCo_80090690, ftCo_80090718, ftCo_DamageFall_Anim,
 * ftCo_DamageFall_Phys, and ftparts.c's ftParts_GetBoneIndex, ftPartsRemap, ftParts_8007506C,
 * cut verbatim into ftdmgfall_extract.inc by run.py together with ftCommon_CalcHitlag (ported,
 * ftcommon.rs). Calls out of the cut are recording stubs; tests/ftdmgfall_ref.rs installs the
 * same recorders as hooks. Each line: op, case, return bits, hash of the dmg fields, call trace. */
#include "ft/types.h"
#include "ft/forward.h"
#include "ft/fighter.h"
#include "ft/inlines.h"
#include "ft/ftcommon.h"
#include "ft/ftparts.h"
#include "ft/kinds/ftCommon/ftCo_DamageFall.h"
#include <sysdolphin/baselib/gobj.h>

#include <Runtime/platform.h>

ftCommonData* p_ftCommonData;
Vec2** Fighter_804D6530;
FighterPartsTable** ftPartsTable;
Fighter_804D6540_t** Fighter_804D6540;
void __assert(const char* file, u32 line, const char* cond) { (void) file; (void) line; (void) cond; for (;;) {} }

static u32 TR;
static void T(u32 v) { TR = TR * 31u + v; }
static u32 B(float f) { union { float f; u32 u; } c; c.f = f; return c.u; }
static HSD_GObj GO;
static FtSFXArr SFXA;

void ft_80081DD4(Fighter_GObj* g) { T(1); T(g == &GO); }
void ft_PlaySFX(Fighter* fp, enum_t id, u8 vol, u8 pan) { (void) fp; T(2); T((u32) id); T(vol); T(pan); }
void ft_80084DB0(Fighter_GObj* g) { T(3); T(g == &GO); }
void ft_800889F4(Fighter* fp, FtSFXArr* a) { (void) fp; T(4); T(a == &SFXA); }
void OSReport(char* f, ...) { (void) f; }

#include "ftdmgfall_extract.inc"

int printf(const char*, ...);
static void zero(void* p, unsigned n) { unsigned char* b = p; while (n--) *b++ = 0; }

static u32 rs;
static u32 rnd(void) { rs ^= rs << 13; rs ^= rs >> 17; rs ^= rs << 5; return rs; }
static float rg(void)
{
    union { float f; u32 u; } c;
    switch (rnd() % 16) {
    case 0: return 0.0f;
    case 1: return -0.0f;
    case 2: c.u = 0x7FC00000u; return c.f;
    case 3: return 1.0f;
    case 4: return -1.0f;
    default: return (float) ((int) (rnd() % 41) - 20) / 20.0f;
    }
}
/* wide floats so calcShift's u16 conversion saturates / wraps both ways */
static float rw(void)
{
    static const float k[4] = { 1.0f, 13.0f, 1000.0f, 100000.0f };
    float a = rg();
    return a * k[rnd() % 4];
}

static Fighter F;
static ftCommonData CD;
static Vec2 SH[3][8];
static Vec2* T6530[6];
static u8 J2P[4][64], P2J[4][64];
static FighterPartsTable PT[4];
static FighterPartsTable* PTP[4];
static Fighter_804D6540_x0_t E40[4][6];
static Fighter_804D6540_t T40[4];
static Fighter_804D6540_t* T40P[4];

static u32 H;
static void HM(u32 v) { H = H * 0x01000193u ^ v; }

static u32 state_hash(void)
{
    H = 0x811C9DC5u;
    HM(F.dmg.x18F8); HM(F.dmg.x18fa_model_shift_frames); HM(F.dmg.x18FC); HM(F.dmg.x18FD);
    HM(B(F.dmg.x1900)); HM(B(F.dmg.x1904)); HM((u32) F.dmg.x1908); HM(F.dmg.x190C == NULL);
    return H;
}

#define NOPS 10
#define NCASE 1500

int main(void)
{
    int op, k, i, j;
    GO.user_data = &F;
    p_ftCommonData = &CD;
    Fighter_804D6530 = T6530;
    ftPartsTable = PTP;
    Fighter_804D6540 = T40P;
    for (op = 0; op < NOPS; op++) {
        for (k = 0; k < NCASE; k++) {
            u32 ret = 0;
            float mul;
            int elem, dmg, msid, ga, part, kind;
            u32 to, from, joint;
            Vec2 sh;
            zero(&F, sizeof F); zero(&CD, sizeof CD);
            TR = 0;
            rs = 0x9059u + (u32) op * 7919u + (u32) k * 104729u;
            for (i = 0; i < 3; i++) {
                for (j = 0; j < 8; j++) { SH[i][j].x = rg(); SH[i][j].y = rg(); }
                T6530[i * 2] = SH[i];
                T6530[i * 2 + 1] = (Vec2*) (uintptr_t) rnd();
            }
            for (i = 0; i < 4; i++) {
                for (j = 0; j < 64; j++) {
                    J2P[i][j] = (rnd() % 5 == 0) ? 0xFF : (u8) (rnd() % 64);
                    P2J[i][j] = (u8) rnd();
                }
                PT[i].joint_to_part = J2P[i]; PT[i].part_to_joint = P2J[i];
                PT[i].parts_num = rnd() % 65;
                PTP[i] = (rnd() % 4 == 0) ? NULL : &PT[i];
                for (j = 0; j < 6; j++) { E40[i][j].x0 = rnd() % 8; E40[i][j].x1 = rnd(); }
                T40[i].x0 = E40[i]; T40[i].x4 = rnd() % 7;
                T40P[i] = (rnd() % 4 == 0) ? NULL : &T40[i];
            }
            CD.x168 = rw(); CD.x16C = rw(); CD.x198 = rw(); CD.x19C = rw(); CD.x1A0 = rw();
            F.kind = rnd() % 4;
            F.motion_id = (rnd() % 4 == 0) ? ftCo_MS_DamageIce : (int) (rnd() % 400);
            F.facing_dir = rg();
            F.coll_data.floor.normal.x = rg(); F.coll_data.floor.normal.y = rg();
            F.dmg.x18F8 = rnd() % 3;
            F.dmg.x18fa_model_shift_frames = (rnd() & 1) ? 0 : (u16) rnd();
            F.dmg.x18FC = rnd() % 8;
            F.dmg.x18FD = rnd();
            F.dmg.x1900 = rg(); F.dmg.x1904 = rg();
            F.dmg.x1908 = (rnd() & 1) ? -1 : (int) (rnd() % 0x200);
            F.dmg.x190C = (rnd() & 1) ? NULL : &SFXA;
            mul = rw();
            elem = rnd() % 14;
            dmg = (int) (rnd() % 120) - 20;
            msid = (rnd() & 1) ? ftCo_MS_Squat - 1 + (int) (rnd() % 4) : (int) (rnd() % 400);
            ga = rnd() % 3;
            part = rnd() % 64;
            kind = rnd() % 4;
            to = rnd() % 4; from = rnd() % 4; joint = rnd() % 80;
            switch (op) {
            case 0: ftCo_80090574(&GO); break;
            case 1: ret = calcShift(mul, msid, dmg); break;
            case 2: ftCo_80090594(&F, elem, dmg, msid, ga, mul); break;
            case 3: { Vec2* r = ftCo_80090690(&F, &sh); ret = r == NULL ? 0xFFFFFFFFu : (B(sh.x) * 31u + B(sh.y)) ^ (r == &sh); } break;
            case 4: ftCo_80090718(&F); break;
            case 5: ftCo_DamageFall_Anim(&GO); break;
            case 6: ftCo_DamageFall_Phys(&GO); break;
            case 7: if (PTP[F.kind] != NULL) ret = (u32) ftParts_GetBoneIndex(&F, part); else ret = 0xEEEEEEEEu; break;
            case 8: ret = (u32) ftPartsRemap(to, from, joint); break;
            case 9: ret = ftParts_8007506C(kind, part % 9); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
