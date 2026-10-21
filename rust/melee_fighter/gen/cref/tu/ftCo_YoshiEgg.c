/* TU: src/melee/ft/kinds/ftCommon/ftCo_YoshiEgg.c */
/* C oracle for src/tu/ftCo_YoshiEgg.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"
#include <stdarg.h>

static HSD_JObj JG, JA, JC, JP[4];
static HSD_Joint JD;
static FighterBone PARTS[4];
static u32 jid(HSD_JObj* j)
{
    int i;
    if (j == &JG) return 1;
    if (j == &JA) return 2;
    if (j == &JC) return 3;
    for (i = 0; i < 4; i++) if (j == &JP[i]) return 10 + i;
    return j == NULL ? 0 : 99;
}

void ftColl_8007B62C(Fighter_GObj* g, enum_t v) { T(1); T(g == &GO); T(v); }
void Fighter_UpdateModelScale(Fighter_GObj* g) { T(2); T(g == &GO); }
void HSD_JObjSetMtxDirtySub(HSD_JObj* j) { T(3); T(jid(j)); }
void ftColl_8007B0C0(Fighter_GObj* g, HurtCapsuleState s) { T(4); T(g == &GO); T(s); }
Fighter_Part ftParts_GetBoneIndex(Fighter* f, Fighter_Part p) { T(5); T(f == &F); T(p); return (SRV[5] >> (p * 2)) & 3; }
void ftColl_HurtboxInit(Fighter* f, FighterHurtCapsule* c, ftHurtboxInit* h)
{
    T(6); T(f == &F); T(c == &F.hurt_capsules[0]); T(h->bone_idx); T(h->height); T(h->is_grabbable);
    T(B(h->a_offset.x)); T(B(h->a_offset.y)); T(B(h->a_offset.z));
    T(B(h->b_offset.x)); T(B(h->b_offset.y)); T(B(h->b_offset.z)); T(B(h->scale));
}
void ftCommon_8007D5D4(Fighter* f) { T(7); T(f == &F); }
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(8); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == &GO2); }
void ftCo_800BC438(Fighter_GObj* g) { T(9); }
void ftCommon_8007E2F4(Fighter* f, s16 v) { T(10); T(f == &F); T((u32) (s32) v); }
void ftCommon_SetAccessory(Fighter* f, HSD_Joint* j) { T(11); T(f == &F); T(j == &JD); }
HSD_Joint* ftYs_SpecialN_8012CDD4(Fighter_GObj* g) { T(12); T(g == &GO2); return &JD; }
void lb_8000C2F8(HSD_JObj* a, HSD_JObj* b) { T(13); T(jid(a)); T(jid(b)); }
void ftYs_SpecialN_SetupItemVel(Fighter_GObj* g, Vec3* out) { T(14); T(g == &GO2); T(out == &F.self_vel); out->x = rw(); out->y = rw(); out->z = rw(); }
float ftYs_SpecialN_GetFacingDir(Fighter_GObj* g) { T(15); T(g == &GO2); return rw(); }
float ftYs_SpecialN_GetDatAttr18(Fighter_GObj* g) { T(16); T(g == &GO2); return rw(); }
float ftYs_SpecialN_GetDatAttr1C(Fighter_GObj* g) { T(17); T(g == &GO2); return rw(); }
float ftYs_SpecialN_GetDatAttr20(Fighter_GObj* g) { T(18); T(g == &GO2); return rw(); }
float ftYs_SpecialN_GetDatAttr24(Fighter_GObj* g) { T(19); T(g == &GO2); return rw(); }
void ftCommon_8007EFC0(Fighter* f, u32 v) { T(20); T(f == &F); T(v); }
void ftCommon_InitGrab(Fighter* f, bool a, float t) { T(21); T(f == &F); T(a); T(B(t)); }
float ftYs_SpecialN_GetExtAttr28(void) { T(22); return rw(); }
float ftYs_SpecialN_GetExtAttr2C(void) { T(23); return rw(); }
float ftYs_SpecialN_8012CD40(void) { T(24); return rw(); }
float ftYs_SpecialN_GetExtAttr34(void) { T(25); return rw(); }
float ftYs_SpecialN_8012CDB4(void) { T(26); return rw(); }
bool ftYs_SpecialN_GetExtAttr38(void) { T(27); return SR(27); }
bool ftCommon_GrabMash(Fighter* f, float v) { T(28); T(f == &F); T(B(v)); return SR(28); }
void ft_PlaySFX(Fighter* f, enum_t id, u8 v, u8 p) { T(29); T(f == &F); T(id); T(v); T(p); }
void efAsync_Spawn(HSD_GObj* g, void* q, u32 k, u32 gfx, HSD_JObj* j, ...)
{
    va_list ap;
    void* p;
    va_start(ap, j);
    p = va_arg(ap, void*);
    va_end(ap);
    T(30); T(g == &GO); T(q == &F.x60C); T(k); T(gfx); T(jid(j)); T(p == &F.co_attrs.xBC);
}
void ftYs_SpecialN_8012CD88(Vec3* v) { T(31); T(v == &F.self_vel); }
void ftColl_8007B760(Fighter_GObj* g, int v) { T(32); T(g == &GO); T(v); }
void ftCo_Fall_Enter_YoshiEgg(Fighter_GObj* g) { T(33); T(g == &GO); }
void ftAnim_SetAnimRate(Fighter_GObj* g, float r) { T(34); T(g == &GO); T(B(r)); }
void ft_80084F3C(Fighter_GObj* g) { T(35); T(g == &GO); }
void ft_80084DB0(Fighter_GObj* g) { T(36); T(g == &GO); }
void ft_8008403C(Fighter_GObj* g, HSD_GObjEvent cb) { T(37); T(g == &GO); T(cb == ftCo_800BC3AC); }
void ft_80082C74(Fighter_GObj* g, HSD_GObjEvent cb) { T(38); T(g == &GO); T(cb == ftCo_800BC388); }
void ftCommon_8007D7FC(Fighter* f) { T(39); T(f == &F); }
bool ftCo_800C0C88(ftCommon_BuryType t) { T(40); T(t); return SR(40); }

/* NaN payload/sign is not pinned (x86 picks the first NaN operand and LLVM may commute fmul), so
 * NaNs hash as one value; every other bit pattern is compared exactly. */
static u32 BN(float f) { return f != f ? 0x7FC00000u : B(f); }
static void HV(Vec3* v) { HM(BN(v->x)); HM(BN(v->y)); HM(BN(v->z)); }
static u32 state_hash(void)
{
    H = 0x811C9DC5u;
    HM(B(F.grab_timer)); HM(F.invisible); HM(F.x1828);
    HM(F.mv.co.yoshiegg.x0 == &GO2); HM(F.mv.co.yoshiegg.x4); HM(B(F.mv.co.yoshiegg.x8));
    HM(B(F.mv.co.yoshiegg.xC)); HM(B(F.mv.co.yoshiegg.x10)); HM(B(F.mv.co.yoshiegg.x14));
    HV(&F.mv.co.yoshiegg.x18); HV(&F.mv.co.yoshiegg.scale); HM(B(F.mv.ca.specialhi.vel.y));
    HV(&F.self_vel); HM(B(F.facing_dir)); HM(B(F.dmg.x182c_behavior));
    HM(F.take_dmg_cb == ftCo_800BC438); HM(F.take_dmg_2_cb == ftCo_800BC3D0);
    HM(F.accessory4_cb == NULL ? 0 : F.accessory4_cb == ftCo_800BBCC0 ? 1 : 2);
    HV(&JG.scale); HV(&JA.scale);
    return H;
}

int main(void)
{
    int op, k, i;
    tu_init();
    for (op = 0; op < 10; op++) {
        for (k = 0; k < 1000; k++) {
            u32 ret = 0;
            tu_case(op, k);
            zero(&JG, sizeof JG); zero(&JA, sizeof JA); zero(&JC, sizeof JC); zero(JP, sizeof JP); zero(PARTS, sizeof PARTS);
            GO.hsd_obj = &JG;
            JA.child = &JC;
            for (i = 0; i < 4; i++) PARTS[i].joint = &JP[i];
            F.parts = PARTS;
            F.x20A0_accessory = &JA;
            JG.flags = rnd() & (JOBJ_MTX_DIRTY | JOBJ_MTX_INDEP_SRT);
            JA.flags = rnd() & (JOBJ_MTX_DIRTY | JOBJ_MTX_INDEP_SRT);
            JG.scale.x = rw(); JG.scale.y = rw(); JG.scale.z = rw();
            F.ground_or_air = rnd() & 1;
            F.grab_timer = rw();
            F.mv.co.yoshiegg.x4 = rnd() & 1;
            F.mv.co.yoshiegg.x8 = rg() * 3.0f;
            F.mv.co.yoshiegg.xC = rg();
            F.mv.co.yoshiegg.x10 = rw();
            F.mv.co.yoshiegg.x14 = rg() * 4.0f;
            F.mv.co.yoshiegg.x18.x = rw(); F.mv.co.yoshiegg.x18.y = rw(); F.mv.co.yoshiegg.x18.z = rw();
            F.mv.co.yoshiegg.scale.x = rw(); F.mv.co.yoshiegg.scale.y = rw(); F.mv.co.yoshiegg.scale.z = rw();
            F.co_attrs.xBC.size = rw();
            F.co_attrs.xBC.x4.x = rw(); F.co_attrs.xBC.x4.y = rw(); F.co_attrs.xBC.x4.z = rw();
            F.co_attrs.xBC.x10.x = rw(); F.co_attrs.xBC.x10.y = rw(); F.co_attrs.xBC.x10.z = rw();
            F.co_attrs.xBC.x1C = rw();
            F.dmg.x1838_percentTemp = rw();
            F.dmg.x18CC = rnd() % 4;
            F.dmg.x18D0 = rnd() % 5;
            switch (op) {
            case 0: ftCo_800BBC88(&GO); break;
            case 1: ftCo_800BBCC0(&GO); break;
            case 2: ftCo_800BBED4(&GO, &GO2); break;
            case 3: ftCo_YoshiEgg_Anim(&GO); break;
            case 4: ftCo_YoshiEgg_IASA(&GO); break;
            case 5: ftCo_YoshiEgg_Phys(&GO); break;
            case 6: ftCo_YoshiEgg_Coll(&GO); break;
            case 7: ftCo_800BC388(&GO); break;
            case 8: ftCo_800BC3AC(&GO); break;
            case 9: ftCo_800BC3D0(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
