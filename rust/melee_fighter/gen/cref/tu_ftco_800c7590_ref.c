/* C reference for src/tu/ftco_800c7590.rs: src/melee/ft/ftCo_800C7590.c, #included whole
 * (shipping contraction via run.py). Every callee is a recording stub; tests/tu_ftco_800c7590.rs
 * installs the same recorders as hooks and replays the same generated inputs.
 * Each line: op case trace fighter-hash.   Regenerate: python rust/melee_fighter/gen/cref/run.py tu_ftco_800c7590 */
#include "ft/types.h"
#include "ft/forward.h"
#include "ft/fighter.h"
#include "ft/inlines.h"
#include <sysdolphin/baselib/gobj.h>

ftCommonData* p_ftCommonData;
void __assert(const char* file, u32 line, const char* cond) { (void) file; (void) line; (void) cond; for (;;) {} }
void OSReport(char* f, ...) { (void) f; }

static u32 TR;
static void T(u32 v) { TR = TR * 31u + v; }
static u32 B(float f) { union { float f; u32 u; } c; c.f = f; return c.u; }
static u32 rs;
static u32 rnd(void) { rs ^= rs << 13; rs ^= rs >> 17; rs ^= rs << 5; return rs; }
static float rg(void)
{
    switch (rnd() % 16) {
    case 0: return 0.0f;
    case 1: return -0.0f;
    case 2: { union { u32 u; float f; } c; c.u = 0x7FC00000u; return c.f; }
    case 3: return 1.0f;
    case 4: return -1.0f;
    default: return (float) ((int) (rnd() % 41) - 20) / 20.0f;
    }
}
static float rw(void) { static const float k[4] = { 1.0f, 13.0f, 1000.0f, 100000.0f }; float a = rg(); return a * k[rnd() % 4]; }

static HSD_GObj GO;
static HSD_GObj ITEM;
static Fighter F;
static ftCommonData CD;
static int RET_BB8, RET_HC;
static float MASH;

void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId ms, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(1); T(g == &GO); T((u32) ms); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
void ftCommon_8007DB58(Fighter_GObj* g) { T(2); T(g == &GO); }
void ftCo_8009750C(Fighter_GObj* g) { T(3); T(g == &GO); }
void ftCo_800DD168(Fighter_GObj* g) { T(4); T(g == &GO); }
void ftCo_8008DCE0(Fighter_GObj* g, int a, float f) { T(5); T(g == &GO); T((u32) a); T(B(f)); }
void ftCommon_8007D7FC(Fighter* fp) { T(6); T(fp == &F); }
void ftCommon_8007E2FC(Fighter_GObj* g) { T(7); T(g == &GO); }
s32 Player_80033BB8(int slot) { T(8); T((u32) slot); return RET_BB8; }
int Player_GetHandicap(int slot) { T(9); T((u32) slot); return RET_HC; }
void ftCommon_InitGrab(Fighter* fp, int a, float t) { T(10); T(fp == &F); T((u32) a); T(B(t)); }
void ftCommon_8007E2F4(Fighter* fp, s16 v) { T(11); T(fp == &F); T((u32) (int) v); }
void it_802EAAEC(Item_GObj* i, Fighter_GObj* g, int a, f32 f) { T(12); T(i == &ITEM); T(g == &GO); T((u32) a); T(B(f)); }
void ftCommon_8007EBAC(Fighter* fp, u32 a, u32 b) { T(13); T(fp == &F); T(a); T(b); }
void Camera_RequestQuake(CmQuakeKind k, Vec* p) { T(14); T((u32) k); T(p == &F.cur_pos); }
bool ftCommon_GrabMash(Fighter* fp, float t) { T(15); T(fp == &F); T(B(t)); fp->grab_timer -= MASH; return 0; }
void it_802EAE80(Item_GObj* i) { T(16); T(i == &ITEM); }
void it_802EADD8(Item_GObj* i) { T(17); T(i == &ITEM); }
void ft_8008403C(Fighter_GObj* g, HSD_GObjEvent cb) { T(18); T(g == &GO); T(cb != NULL); }
void ftCo_80090780(Fighter_GObj* g) { T(19); T(g == &GO); }
void ftCo_800C7A30(Fighter_GObj* g) { T(20); T(g == &GO); }
void fn_800C74CC(Fighter_GObj* g) { (void) g; }
void fn_800C7568(Fighter_GObj* g) { (void) g; }

#include "melee/ft/ftCo_800C7590.c"

int printf(const char*, ...);
static void zero(void* p, unsigned n) { unsigned char* b = p; while (n--) *b++ = 0; }

static u32 H;
static void HM(u32 v) { H = H * 0x01000193u ^ v; }
static u32 fhash(void)
{
    H = 0x811C9DC5u;
    HM(B(F.grab_timer)); HM(B(F.gr_vel)); HM(B(F.self_vel.x)); HM((u32) F.capture_timer);
    HM(F.x2219_b1); HM(F.x2219_b2); HM(F.x221D_b5); HM(F.x2220_b3);
    HM(F.mv.co.captureleadead.x0 == &ITEM); HM(F.take_dmg_cb != NULL); HM(F.death2_cb != NULL);
    return H;
}

#define NOPS 3
#define NCASE 3000

int main(void)
{
    int op, k;
    GO.user_data = &F;
    p_ftCommonData = &CD;
    for (op = 0; op < NOPS; op++) {
        for (k = 0; k < NCASE; k++) {
            zero(&F, sizeof F); zero(&CD, sizeof CD);
            TR = 0;
            rs = 0x7590u + (u32) op * 7919u + (u32) k * 104729u;
            CD.x71C = rw(); CD.x720 = rw(); CD.x724 = rw(); CD.x728 = rw(); CD.x72C = rw(); CD.x730 = rw();
            CD.leadead_grab_timer_step = rw(); CD.leadead_grab_break_threshold = rw();
            CD.x370 = rw(); CD.x73C = (int) rnd();
            F.player_id = rnd() % 6;
            F.dmg.x1830_percent = rw();
            F.dmg.x1868_source = &ITEM;
            F.mv.co.captureleadead.x0 = &ITEM;
            F.grab_timer = rw();
            F.facing_dir = rg();
            F.ground_or_air = rnd() % 2;
            RET_BB8 = (int) (rnd() % 9) - 4;
            RET_HC = (int) (rnd() % 12);
            MASH = rw();
            switch (op) {
            case 0: ftCo_800C7590(&GO); break;
            case 1: ftCo_CaptureLeadead_Anim(&GO); break;
            case 2: ftCo_CaptureLeadead_Coll(&GO); ftCo_800C7800(&GO); fn_800C7890(&GO); break;
            }
            printf("%d %d %08x %08x\n", op, k, TR, fhash());
        }
    }
    return 0;
}
