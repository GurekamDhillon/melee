/* C reference for src/tu/ftco_800c78b0.rs: src/melee/ft/ftCo_800C78B0.c, #included whole
 * (shipping contraction via run.py). Every callee is a recording stub; tests/tu_ftco_800c78b0.rs
 * installs the same recorders as hooks and replays the same generated inputs.
 * Each line: op case trace fighter-hash.   Regenerate: python rust/melee_fighter/gen/cref/run.py tu_ftco_800c78b0 */
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
static float MASH, HD, KB;
static u32 HE, HU;
static Vec3 IPOS;
static lbColl_80008D30_arg1 ARG3;

void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId ms, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(1); T(g == &GO); T((u32) ms); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
void ftCommon_8007DB58(Fighter_GObj* g) { T(2); T(g == &GO); }
void ftCo_8009750C(Fighter_GObj* g) { T(3); T(g == &GO); }
void ftCo_800DD168(Fighter_GObj* g) { T(4); T(g == &GO); }
void ftCommon_8007E2FC(Fighter_GObj* g) { T(7); T(g == &GO); }
s32 Player_80033BB8(int slot) { T(8); T((u32) slot); return RET_BB8; }
int Player_GetHandicap(int slot) { T(9); T((u32) slot); return RET_HC; }
void ftCommon_InitGrab(Fighter* fp, int a, float t) { T(10); T(fp == &F); T((u32) a); T(B(t)); }
void ftCommon_8007E2F4(Fighter* fp, s16 v) { T(11); T(fp == &F); T((u32) (int) v); }
bool ftCommon_GrabMash(Fighter* fp, float t) { T(15); T(fp == &F); T(B(t)); fp->grab_timer -= MASH; return 0; }
void fn_800C7890(Fighter_GObj* g) { (void) g; }
void ftAnim_8006EBA4(Fighter_GObj* g) { T(30); T(g == &GO); }
void it_802DBA68(Item_GObj* i) { T(31); T(i == &ITEM); }
void it_802DB9F4(Item_GObj* i) { T(32); T(i == &ITEM); }
void it_8026B294(Item_GObj* i, Vec3* p) { T(33); T(i == &ITEM); T(p == &F.cur_pos); *p = IPOS; }
void mpColl_80043670(CollData* c) { T(34); T(c == &F.coll_data); c->cur_pos.y = 7.0f; }
bool mpColl_800471F8(CollData* c) { T(35); T(c == &F.coll_data); c->cur_pos.x += 1.0f; return 0; }
void lbColl_80008D30(HitCapsule* h, lbColl_80008D30_arg1* a) { T(36); T(a == &ARG3); h->damage = HD; h->element = HE; h->unk_count = HU; }
void ftColl_800788D4(Fighter_GObj* g) { T(37); T(g == &GO); }
float ftColl_80079EA8(Fighter* fp, HitCapsule* h, u32 n) { T(38); T(fp == &F); T(B(h->damage)); T(n); return KB; }
void Fighter_UnkTakeDamage_8006CC30(Fighter* fp, float d) { T(39); T(fp == &F); T(B(d)); }
void ftCo_Damage_CalcKnockback(Fighter* fp) { T(40); T(fp == &F); }
void ftCo_8008E908(Fighter_GObj* g, float f) { T(41); T(g == &GO); T(B(f)); }
void Fighter_TakeDamage_8006CC7C(Fighter* fp, float d) { T(42); T(fp == &F); T(B(d)); }

#include "melee/ft/ftCo_800C78B0.c"

int printf(const char*, ...);
static void zero(void* p, unsigned n) { unsigned char* b = p; while (n--) *b++ = 0; }

static u32 H;
static void HM(u32 v) { H = H * 0x01000193u ^ v; }
static void HV(Vec3 v) { HM(B(v.x)); HM(B(v.y)); HM(B(v.z)); }
static u32 fhash(void)
{
    H = 0x811C9DC5u;
    HM(B(F.grab_timer)); HM(F.x221D_b5); HM(F.invisible); HM((u32) F.x1988);
    HM(F.mv.co.capturelikelike.x0 == &ITEM); HM((u32) F.mv.co.capturelikelike.x4);
    HM(F.take_dmg_cb != NULL); HM(F.death2_cb != NULL); HM(F.accessory1_cb != NULL);
    HV(F.cur_pos); HV(F.coll_data.cur_pos); HV(F.coll_data.last_pos);
    HM(B(F.dmg.kb_applied)); HM((u32) F.dmg.x1848_kb_angle); HM(B(F.dmg.facing_dir_1));
    HM((u32) F.dmg.x184c_damaged_hurtbox); HV(F.dmg.x1854_collpos); HM(F.dmg.x1860_element);
    return H;
}

#define NOPS 4
#define NCASE 3000

int main(void)
{
    int op, k;
    GO.user_data = &F;
    p_ftCommonData = &CD;
    for (op = 0; op < NOPS; op++) {
        for (k = 0; k < NCASE; k++) {
            Vec3 a1, a2;
            float ang;
            int dmg;
            zero(&F, sizeof F); zero(&CD, sizeof CD);
            TR = 0;
            rs = 0x78B0u + (u32) op * 7919u + (u32) k * 104729u;
            CD.x740 = rw(); CD.x744 = rw(); CD.x748 = rw(); CD.x74C = rw(); CD.x750 = rw(); CD.x754 = rw();
            CD.x758 = rw(); CD.x75C = rw();
            F.player_id = rnd() % 6;
            F.dmg.x1830_percent = rw();
            F.mv.co.capturelikelike.x0 = &ITEM;
            F.mv.co.capturelikelike.x4 = rnd() % 2;
            F.grab_timer = rw();
            F.facing_dir = rg();
            F.dmg.x184c_damaged_hurtbox = 5;
            RET_BB8 = (int) (rnd() % 9) - 4;
            RET_HC = (int) (rnd() % 12);
            MASH = rw(); HD = rw(); KB = rw(); HE = rnd() % 20; HU = rnd() % 4;
            IPOS.x = rw(); IPOS.y = rw(); IPOS.z = rw();
            a1.x = rw(); a1.y = rw(); a1.z = rw(); a2.x = rw(); a2.y = rw(); a2.z = rw();
            ang = (rnd() % 3 == 0) ? rw() * 10.0f : rw();
            dmg = (int) rnd() % 2000;
            switch (op) {
            case 0: ftCo_800C78B0(&GO, &ITEM); break;
            case 1: ftCo_CaptureLikelike_Anim(&GO); break;
            case 2: ftCo_800C7B0C(&GO, &a1, &a2, &ARG3, ang); break;
            case 3: ftCo_800C7A30(&GO); fn_800C7AE0(&GO); ftCo_800C7C60(&GO, dmg); break;
            }
            printf("%d %d %08x %08x\n", op, k, TR, fhash());
        }
    }
    return 0;
}
