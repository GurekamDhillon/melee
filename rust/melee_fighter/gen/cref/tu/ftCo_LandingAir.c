/* TU: src/melee/ft/kinds/ftCommon/ftCo_LandingAir.c */
/* C oracle for src/tu/ftCo_LandingAir.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

#define VSTUB(name, id) void name(Fighter_GObj* g) { T(id); T(g == &GO); }
/* lag is only traced when the C passes a defined value (mode != -1). */
void ScriptGame_SkillLanding(Fighter* f, int mode, float lag, float lag_final)
{ T(1); T(f == &F); T(mode); if (mode != -1) { T(B(lag)); } T(B(lag_final)); }
void ftCommon_8007D7FC(Fighter* f) { T(2); T(f == &F); }
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(3); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
void ftAnim_SetAnimRate(Fighter_GObj* g, float r) { T(4); T(g == &GO); T(B(r)); }
float ftAnim_8006F484(Fighter_GObj* g) { T(5); T(g == &GO); return rw(); }
VSTUB(ftCo_Landing_Enter_Basic, 6)
VSTUB(ftCo_Landing_Anim, 7)
VSTUB(ftCo_Landing_Phys, 8)
VSTUB(ftCo_Landing_Coll, 9)

int main(void)
{
    int op, k;
    tu_init();
    for (op = 0; op < 6; op++) {
        for (k = 0; k < 4000; k++) {
            u32 ret = 0;
            int ms;
            float lag;
            tu_case(op, k);
            F.cmd_vars[0] = rnd() % 3 ? 1 : 0;
            F.motion_id = 63 + rnd() % 9;
            F.co_attrs.landingairn_lag = rw(); F.co_attrs.landingairf_lag = rw(); F.co_attrs.landingairb_lag = rw();
            F.co_attrs.landingairhi_lag = rw(); F.co_attrs.landingairlw_lag = rw();
            F.x67F = rnd() % 12;
            CD.xE4 = rnd() % 12;
            CD.xE8 = rw();
            ms = rnd() % 400; lag = rw();
            switch (op) {
            case 0: ftCo_LandingAir_EnterWithLag(&GO); break;
            case 1: ftCo_LandingAir_EnterWithMsidLag(&GO, ms, lag); break;
            case 2: ftCo_LandingAir_Anim(&GO); break;
            case 3: ftCo_LandingAir_IASA(&GO); break;
            case 4: ftCo_LandingAir_Phys(&GO); break;
            case 5: ftCo_LandingAir_Coll(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, 0u, TR);
        }
    }
    return 0;
}
