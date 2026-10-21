/* TU: src/melee/ft/kinds/ftCommon/ftCo_Landing.c */
/* C oracle for src/tu/ftCo_Landing.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

#define VSTUB(name, id) void name(Fighter_GObj* g) { T(id); T(g == &GO); }
#define CHK(name, id) bool name(Fighter_GObj* g) { T(id); T(g == &GO); return SR(id) & SR(id + 16) & SR(id + 32); }
void ftCommon_8007D7FC(Fighter* f) { T(1); T(f == &F); }
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(2); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
int Geno_DefineTemplateSkeleton(int k) { T(3); T(k); return rnd() & 1; }
int Geno_DefineBaseKind(int k) { T(4); T(k); return rnd() % 40; }
int ftKb_MexKirbyLike(int k) { T(5); T(k); return rnd() & 1; }
VSTUB(ftPe_8011D598, 6)
CHK(ftCo_800C5240, 7)
VSTUB(ftCo_HammerLanding_Enter, 8)
bool ftAnim_IsFramesRemaining(Fighter_GObj* g) { T(9); T(g == &GO); return SR(9); }
VSTUB(ft_8008A2BC, 10)
CHK(ftCo_SpecialS_CheckInput, 11)
CHK(ftCo_Attack100_CheckInput, 12)
CHK(ftCo_800D6824, 13)
CHK(ftCo_800D68C0, 14)
CHK(ftCo_Catch_CheckInput, 15)
CHK(ftCo_AttackS4_CheckInput, 16)
CHK(ftCo_AttackHi4_CheckInput, 17)
CHK(ftCo_AttackLw4_CheckInput, 18)
CHK(ftCo_AttackS3_CheckInput, 19)
CHK(ftCo_AttackHi3_CheckInput, 20)
CHK(ftCo_AttackLw3_CheckInput, 21)
CHK(ftCo_Attack1_CheckInput, 22)
CHK(ftCo_80091A4C, 23)
CHK(ftCo_800DE9D8, 24)
CHK(ftCo_Jump_CheckInput, 25)
CHK(ftCo_Dash_CheckInput, 26)
CHK(ftCo_SquatWait_CheckInput, 27)
CHK(ftCo_Turn_CheckInput, 28)
CHK(ftCo_Walk_CheckInput, 29)
VSTUB(ft_80084F3C, 30)
VSTUB(ft_80084280, 31)

static u32 state_hash(void)
{
    H = 0x811C9DC5u;
    HM(F.mv.co.landing.allow_interrupt);
    HM(F.u.mr.x2234_tornadoCharge); HM(F.u.mr.x2238_isCapeBoost); HM(F.u.pe.specialairn_used);
    HM(F.u.ms.x222C); HM(F.u.gw.x2234); HM(F.u.pp.x224C);
    HM(F.u.kb.xCC); HM(F.u.kb.xC4); HM(F.u.kb.x64); HM(F.u.mt.x223C_isConfusionBoost);
    return H;
}

int main(void)
{
    int op, k;
    tu_init();
    for (op = 0; op < 8; op++) {
        for (k = 0; k < 2000; k++) {
            u32 ret = 0;
            int ms, ai;
            u32 fl;
            float a, b;
            tu_case(op, k);
            F.kind = rnd() % 40;
            F.u.mr.x2234_tornadoCharge = rnd(); F.u.mr.x2238_isCapeBoost = rnd(); F.u.pe.specialairn_used = rnd();
            F.u.ms.x222C = rnd(); F.u.gw.x2234 = rnd(); F.u.pp.x224C = rnd();
            F.u.kb.xCC = rnd(); F.u.kb.xC4 = rnd(); F.u.kb.x64 = rnd(); F.u.mt.x223C_isConfusionBoost = rnd();
            F.mv.co.landing.allow_interrupt = rnd() % 3;
            F.cur_anim_frame = rw(); F.frame_speed_mul = rw(); F.co_attrs.normal_landing_lag = rw();
            F.x2EC = rw();
            ms = rnd() % 400; ai = rnd() & 1; fl = rnd(); a = rw(); b = rw();
            switch (op) {
            case 0: ftCo_Landing_Enter(&GO, ms, ai, fl, a, b); break;
            case 1: ftCo_Landing_Enter_Basic(&GO); break;
            case 2: ftCo_LandingFallSpecial_Enter_Basic(&GO); break;
            case 3: ftCo_LandingFallSpecial_Enter(&GO, ai, a); break;
            case 4: ftCo_Landing_Anim(&GO); break;
            case 5: ftCo_Landing_IASA(&GO); break;
            case 6: ftCo_Landing_Phys(&GO); break;
            case 7: ftCo_Landing_Coll(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
