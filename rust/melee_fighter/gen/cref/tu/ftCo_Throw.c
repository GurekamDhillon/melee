/* TU: src/melee/ft/kinds/ftCommon/ftCo_Throw.c */
/* C oracle for src/tu/ftCo_Throw.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"
#include "s217_fill.h"

static int NEVER;
static void cb_fn(HSD_GObj* g) { T(98); T(PI(g)); }
CMS_STUB
void ftCo_800DC920(Fighter_GObj* a, Fighter_GObj* b) { T(10); T(PI(a)); T(PI(b)); }
STUB_V(ftCo_800DCFD4, 11)
STUB_V(ftCommon_8007D92C, 12)
bool ftCo_800DF7F4(Fighter* f) { T(13); T(PI(f)); return chk(13); }
bool ftCo_800DF844(Fighter* f) { T(14); T(PI(f)); return chk(14); }
bool ftCo_800DF878(Fighter* f) { T(15); T(PI(f)); return chk(15); }
void ftAnim_8006EBA4(Fighter_GObj* g) { T(16); T(PI(g)); if (!NEVER && (rnd() & 7) == 0) GET_FIGHTER(g)->cmd_vars[0] = 1; }
void ftCommon_8007E2F4(Fighter* f, s16 v) { T(17); T(PI(f)); T((u32) (s32) v); }
void ftCo_800DE3FC(Fighter_GObj* g, FtMotionId m, float s) { T(18); T(PI(g)); T((u32) m); T(B(s)); }
int ftKb_MexKirbyLike(int k) { T(19); T((u32) k); return chk(19); }
void ftSs_Init_CreateThrowGrappleBeam(HSD_GObj* g, s32 m, float s) { T(20); T(PI(g)); T((u32) m); T(B(s)); }
void ftColl_8007B7A4(Fighter_GObj* g, int n) { T(21); T(PI(g)); T((u32) n); }
int Geno_DefineTemplateSkeleton(int k) { T(22); T((u32) k); return chk(22); }
int Geno_DefineBaseKind(int k) { T(23); T((u32) k); return (SRV[6] & 1) ? 19 : 9; }
void ftAnim_SetAnimRate(Fighter_GObj* g, float r) { T(24); T(PI(g)); T(B(r)); }
STUB_V(ftCo_800DE974, 25)
void ftCommon_8007D7FC(Fighter* f) { T(26); T(PI(f)); }
STUB_V(ftCo_Fall_Enter, 27)
u32 mpLineGetFlags(int l) { T(28); T((u32) l); return SRV[7]; }
void pl_80040614(int a, bool b, float f) { T(29); T((u32) a); T(b); T(B(f)); }
void ftCo_800DE7C0(Fighter_GObj* g, Fighter_GObj* v, bool up) { T(30); T(PI(g)); T(PI(v)); T(up); }
void ftCo_800DE920(Fighter_GObj* g, f32 f) { T(31); T(PI(g)); T(B(f)); }
STUB_V(ftMt_SpecialN_Shoot, 32)
STUB_CHK(ftAnim_IsFramesRemaining, 33)
STUB_V(ftCo_8009B56C, 34)
STUB_V(ft_80085134, 35)
STUB_V(ft_80085004, 36)
void ft_80083C00(Fighter_GObj* g, HSD_GObjEvent cb) { T(37); T(PI(g)); T(cb != NULL); }
void ft_80083CE4(Fighter_GObj* g, bool (*a)(Fighter_GObj*, int), HSD_GObjEvent b) { T(38); T(PI(g)); T(a != NULL); T(b != NULL); }
STUB_V(ft_80083B68, 39)
void ft_800841B8(Fighter_GObj* g, HSD_GObjEvent cb) { T(40); T(PI(g)); T(cb != NULL); }
STUB_V(ftFx_Throw_Anim, 41)
STUB_V(ftCamera_800762F4, 42)
STUB_V(ftCamera_UpdateCameraBox, 43)
void ftColl_8007891C(Fighter_GObj* a, Fighter_GObj* b, float d) { T(44); T(PI(a)); T(PI(b)); T(B(d)); }
void ftColl_8007B62C(Fighter_GObj* g, enum_t n) { T(45); T(PI(g)); T((u32) n); }
Fighter_Part ftParts_GetBoneIndex(Fighter* f, Fighter_Part p) { T(46); T(PI(f)); T(p); return (SRV[5] + p) % 8; }
void lb_8000B1CC(HSD_JObj* j, Vec3* p0, Vec3* out) { T(47); T(PI(j)); T(PI(p0)); out->x = rw(); out->y = rw(); out->z = rw(); }
s32 ftColl_8007B868(Fighter_GObj* g) { T(48); T(PI(g)); return chk(48); }
float ScriptGame_FighterDealtDamage(Fighter* f, float d) { T(49); T(PI(f)); T(B(d)); return d + 1.0f; }
float ScriptGame_HitRuleContact(HitCapsule* h, Fighter* f, float v, int kb) { T(50); T(PI(h)); T(PI(f)); T(B(v)); T((u32) kb); return v + (float) kb; }
bool ftColl_80076640(Fighter* f, float* d) { T(51); T(PI(f)); T(B(*d)); if (chk(51)) { *d = *d + 2.0f; return 1; } return 0; }
void ScriptGame_HitRulePercentQueue(HitCapsule* h, Fighter* f, float d) { T(52); T(PI(h)); T(PI(f)); T(B(d)); }
float gm_8016B248(void) { T(53); return rg(); }
f32 Player_GetAttackRatio(int s) { T(54); T((u32) s); return rg(); }
f32 Player_GetDefenseRatio(int s) { T(55); T((u32) s); return rg(); }
float ftColl_80079AB0(Fighter* f, HitCapsule* h, u32 n, float a, float b, float c, float d)
{ T(56); T(PI(f)); T(PI(h)); T(n); T(B(a)); T(B(b)); T(B(c)); T(B(d)); return rw(); }
void ScriptGame_HitRuleWon(HitCapsule* h, Fighter* f) { T(57); T(PI(h)); T(PI(f)); }
void ScriptGame_HitRuleReport(Fighter* a, Fighter* b, HitCapsule* h, float d) { T(58); T(PI(a)); T(PI(b)); T(PI(h)); T(B(d)); }
void ftColl_80078710(Fighter_GObj* a, Fighter_GObj* b, UNK_T p) { T(59); T(PI(a)); T(PI(b)); T(PI(p)); }
void ftCommon_8007EE0C(Fighter* f, s32 v) { T(60); T(PI(f)); T((u32) v); }
void ftCo_Damage_CalcKnockback(Fighter* f) { T(61); T(PI(f)); }
void ftCommon_8007D5D4(Fighter* f) { T(62); T(PI(f)); }
void lb_8000C390(HSD_JObj* j) { T(63); T(PI(j)); }
void HSD_JObjSetMtxDirtySub(HSD_JObj* j) { T(64); T(PI(j)); }
void mpColl_80043670(CollData* c) { T(65); T(PI(c)); }
void ftCommon_UnlockECB(Fighter* f) { T(66); T(PI(f)); }
bool mpColl_800471F8(CollData* c) { T(67); T(PI(c)); c->cur_pos.x = c->cur_pos.x + 1.0f; return chk(67); }
void lbColl_80008D30(HitCapsule* h, lbColl_80008D30_arg1* a)
{ T(68); T(PI(h) == 99); T(PI(a)); h->damage = rw(); h->unk_count = rnd() % 5; h->kb_angle = (int) (rnd() % 361); h->element = rnd() % 5; }
STUB_V(ftColl_800788D4, 69)
float ScriptGame_ModValue(int s, int fl, float base) { T(70); T((u32) s); T((u32) fl); T(B(base)); return base + (float) fl; }
void Fighter_UnkTakeDamage_8006CC30(Fighter* f, float d) { T(71); T(PI(f)); T(B(d)); }
void ftCo_8008E908(Fighter_GObj* g, float v) { T(72); T(PI(g)); T(B(v)); }

int main(void)
{
    int op, k;
    tu_init();
    s_init();
    for (op = 0; op < 31; op++) {
        for (k = 0; k < 1000; k++) {
            u32 ret = 0;
            int ms, vms, li, ar;
            float sp;
            tu_case(op, k);
            s_fill();
            NEVER = SRV[3] & 1;
            F.anim_cb = (rnd() & 1) ? cb_fn : NULL;
            ms = 219 + (int) (rnd() % 4);
            vms = 239 + (int) (rnd() % 5);
            sp = rg();
            li = (rnd() & 3) == 0 ? -1 : (int) (rnd() % 50);
            ar = rnd() & 1;
            switch (op) {
            case 0: ftCo_800DD100(&GO); break;
            case 1: ftCo_800DD168(&GO); break;
            case 2: F.victim_gobj = &GO2; ret = ftCo_800DD1E4(&GO); break;
            case 3: F.victim_gobj = &GO2; ftCo_800DD398(&GO, ms, vms, sp); break;
            case 4: F.victim_gobj = &GO2; ftCo_800DD4B0(&GO, ms); break;
            case 5: fn_800DD568(&GO); break;
            case 6: fn_800DD5EC(&GO); break;
            case 7: fn_800DD684(&GO); break;
            case 8: ret = fn_800DD6E4(&GO, li); break;
            case 9: ftCo_800DD724(&GO); break;
            case 10: ftCo_ThrowF_Anim(&GO); break;
            case 11: ftCo_ThrowF_IASA(&GO); break;
            case 12: ftCo_ThrowF_Phys(&GO); break;
            case 13: ftCo_ThrowF_Coll(&GO); break;
            case 14: ftCo_ThrowB_Anim(&GO); break;
            case 15: ftCo_ThrowB_IASA(&GO); break;
            case 16: ftCo_ThrowB_Phys(&GO); break;
            case 17: ftCo_ThrowB_Coll(&GO); break;
            case 18: ftCo_ThrowHi_Anim(&GO); break;
            case 19: ftCo_ThrowHi_IASA(&GO); break;
            case 20: ftCo_ThrowHi_Phys(&GO); break;
            case 21: ftCo_ThrowHi_Coll(&GO); break;
            case 22: ftCo_ThrowHi_Cam(&GO); break;
            case 23: ftCo_ThrowLw_Anim(&GO); break;
            case 24: ftCo_ThrowLw_IASA(&GO); break;
            case 25: ftCo_ThrowLw_Phys(&GO); break;
            case 26: ftCo_ThrowLw_Coll(&GO); break;
            case 27: ftCo_800DDDE4(&GO, &GO2, ar); break;
            case 28: ftCo_800DE2A8(&GO, &GO2); break;
            case 29: ftCo_800DE2CC(&GO, &GO2); break;
            case 30: ftCo_800DE2F0(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
