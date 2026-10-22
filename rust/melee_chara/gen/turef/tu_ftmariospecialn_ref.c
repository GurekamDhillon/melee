/* C reference for src/tu/ftmariospecialn.rs: the whole game TU is #included verbatim; everything
 * it calls is a recording stand-in (HSD_Randi draws from a deterministic stream both sides share).
 * Prints, per case, the Megavitamin colour pick and the spawn dispatch of ItemFireSpawn. */
#include "ft/kinds/ftMario/ftmariospecialn.c"
#include <stdio.h>

void __assert(const char* f, u32 l, const char* c) { (void) f; (void) l; (void) c; for (;;) {} }

static u32 s = 0x5E1CA7u;
static u32 rnd(void) { s ^= s << 13; s ^= s >> 17; s ^= s << 5; return s; }
static u32 rs = 0x13572468u;
static u32 rrnd(void) { rs ^= rs << 13; rs ^= rs >> 17; rs ^= rs << 5; return rs; }
static int RANDI_N;
int HSD_Randi(int n) { RANDI_N = n; return n > 0 ? (int) (rrnd() % (u32) n) : 0; }
int Geno_DefineBaseKind(int kind) { return kind == 40 ? Ft_Kind_Mario : kind; }
static int LOGN;
static char LOGB[256];
static void note(const char* fmt, int a, int b, int c)
{
    LOGN += snprintf(LOGB + LOGN, sizeof LOGB - LOGN, fmt, a, b, c);
}
void lb_8000B1CC(HSD_JObj* j, Vec3* off, Vec3* out) { (void) j; (void) off; out->x = 1.0f; out->y = 2.0f; out->z = 3.0f; }
int ftParts_GetBoneIndex(Fighter* fp, int i) { (void) fp; return i; }
void it_8029B6F8(HSD_GObj* g, Vec3* pos, int kind, float facing) { (void) g; (void) pos; note("fire%d,%d,%d;", kind, (int) facing, 0); }
void* efSync_Spawn(s32 id, HSD_GObj* g, ...) { (void) g; note("ef%d,%d,%d;", id, 0, 0); return 0; }
void itDrMario_unused(void);
void itDrMarioPill_Spawn(Item_GObj* g, Vec3* pos, s32 color, ItemKind kind, f32 facing)
{ (void) g; (void) pos; note("pill%d,%d,%d;", color, kind, (int) facing); }
void Fighter_ChangeMotionState(HSD_GObj* g, int m, u32 f, float a, float b, float c, HSD_GObj* d)
{ (void) g; (void) m; (void) f; (void) a; (void) b; (void) c; (void) d; }
void ftAnim_8006EBA4(HSD_GObj* g) { (void) g; }
int ftAnim_IsFramesRemaining(HSD_GObj* g) { (void) g; return 0; }
void ft_8008A2BC(HSD_GObj* g) { (void) g; }
void ftCo_Fall_Enter(HSD_GObj* g) { (void) g; }
bool ftCo_Fall_IASA_Inner(Fighter_GObj* g) { (void) g; return 0; }
void ftCo_Wait_IASA(HSD_GObj* g) { (void) g; }
void ft_80084F3C(HSD_GObj* g) { (void) g; }
void ft_80084DB0(HSD_GObj* g) { (void) g; }
int ft_80082708(HSD_GObj* g) { (void) g; return 1; }
int ft_80081D0C(HSD_GObj* g) { (void) g; return 0; }
void ftCommon_8007D5D4(Fighter* fp) { (void) fp; }
void ftCommon_8007D7FC(Fighter* fp) { (void) fp; }

int main(void)
{
    static Fighter F;
    static HSD_GObj G;
    static FighterBone PARTS[80];
    int i;
    G.user_data = &F;
    F.parts = PARTS;
    for (i = 0; i < 3000; i++) {
        int r;
        F.u.mr.x222C_vitaminCurr = (int) (rnd() % 11);
        F.u.mr.x2230_vitaminPrev = (int) (rnd() % 11);
        r = ftMr_SpecialN_VitaminRandom(&G);
        printf("V %d %d %d %d %d\n", i, r, F.u.mr.x222C_vitaminCurr, F.u.mr.x2230_vitaminPrev, RANDI_N);
        F.throw_flags_b0 = rnd() & 1;
        F.kind = (rnd() % 3 == 0) ? Ft_Kind_Mario : ((rnd() & 1) ? 40 : Ft_Kind_DrMario);
        F.facing_dir = (rnd() & 1) ? 1.0f : -1.0f;
        LOGN = 0;
        LOGB[0] = 0;
        ftMr_SpecialN_ItemFireSpawn(&G);
        printf("F %d %d %s\n", i, (int) F.throw_flags_b0, LOGB);
    }
    return 0;
}
