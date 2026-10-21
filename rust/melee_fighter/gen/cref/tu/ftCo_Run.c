/* TU: src/melee/ft/kinds/ftCommon/ftCo_Run.c */
/* C oracle for src/tu/ftCo_Run.rs (TARGET_PC): the whole TU included verbatim; every callee out
 * of it is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"
#include "ftco_misc8_fill.h"

static float FR, MV[2];
static int nmv;
int ScriptGame_CapsActionAllowed(Fighter* f, int flag) { T(1); T(PI(f)); T(flag); return SR(1); }
float ScriptGame_ModValue(int slot, int field, float base) { T(2); T(slot); T(field); T(B(base)); return MV[nmv++ & 1]; }
CMS(3)
float ft_GetGroundFrictionMultiplier(Fighter* f) { T(4); T(PI(f)); return FR; }
void ftAnim_SetAnimRate(Fighter_GObj* g, float r) { T(5); T(PI(g)); T(B(r)); }
BSTUB(ftCo_SpecialS_CheckInput, 6)
BSTUB(ftCo_Attack100_CheckInput, 7)
BSTUB(ftCo_800D6824, 8)
BSTUB(ftCo_800D68C0, 9)
BSTUB(ftCo_800D8A38, 10)
BSTUB(ftCo_AttackDash_CheckInput, 11)
VSTUB(ftCo_AttackDash_SetMv0, 12)
BSTUB(ftCo_80091A4C, 13)
void ftCo_80091B90(Fighter_GObj* g, int v) { T(14); T(PI(g)); T(v); }
VSTUB(ftCo_80091B9C, 15)
BSTUB(ftCo_800DE9D8, 16)
BSTUB(fn_800CAF78, 17)
BSTUB(fn_800C9D40, 18)
BSTUB(ftCo_RunBrake_CheckInput, 19)
void ftCommon_CalcGroundAccel_DashRun(Fighter* f, float a, float t, float fr) { T(20); T(PI(f)); T(B(a)); T(B(t)); T(B(fr)); }
VSTUB(ftCommon_SetSelfMovementFromGroundedMovement, 21)
VSTUB(ft_800844EC, 22)

#define NOPS 10
#define NCASE 3000

int main(void)
{
    int op, k;
    tu_init();
    for (op = 0; op < NOPS; op++) {
        for (k = 0; k < NCASE; k++) {
            u32 ret = 0;
            tu_case(op, k);
            m8_fill();
            nmv = 0; FR = rg(); MV[0] = rw(); MV[1] = rw();
            if (rnd() & 1) MV[1] = 0.0f;
            switch (op) {
            case 0: ret = fn_800CA5F0(&GO); break;
            case 1: ret = fn_800CA644(&GO); break;
            case 2: ret = fn_800CA698(&GO); break;
            case 3: ftCo_Run_Enter(&GO, rw()); break;
            case 4: { float a = rw(); float b = rg(); float c = rg(); ftCo_Run_Enter_Full(&GO, a, b, c); break; }
            case 5: ftCo_Run_Anim(&GO); break;
            case 6: ftCo_Run_IASA(&GO); break;
            case 7: ftCo_Run_Phys(&GO); break;
            case 8: ftCo_Run_Coll(&GO); break;
            case 9: ftCo_Run_Phys(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
