/* TU: src/melee/ft/kinds/ftCommon/ftCo_SpecialS.c */
/* C oracle for src/tu/ftCo_SpecialS.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"
#include "s217_fill.h"

HSD_GObjEvent ftData_SpecialS[Ft_Kind_Max];
static void vanilla_fn(HSD_GObj* g) { T(99); }
int ScriptGame_CapsActionAllowed(Fighter* f, int fl) { T(1); T(PI(f)); T(fl); return chk(1) | chk(7); }
void ftCommon_UpdateFacing(Fighter* f) { T(2); T(PI(f)); }
float ft_GetGroundFrictionMultiplier(Fighter* f) { T(3); T(PI(f)); return rg(); }
int Geno_SpecialEnter(Fighter_GObj* g, int w) { T(4); T(PI(g)); T(w); return chk(4); }
void Mex_SpecialSDispatch(int k, void* g, void* v) { T(5); T(k); T(PI(g)); T(v == NULL ? 0 : v == (void*) vanilla_fn ? 1 : 9); }

int main(void)
{
    int op, k;
    tu_init();
    s_init();
    for (op = 0; op < 3; op++) {
        for (k = 0; k < 3000; k++) {
            u32 ret = 0;
            tu_case(op, k);
            s_fill();
            ftData_SpecialS[F.kind] = (SRV[1] & 1) ? vanilla_fn : NULL;
            switch (op) {
            case 0: ret = ftCo_SpecialS_HasInput(&F); break;
            case 1: ret = ftCo_SpecialS_CheckInput(&GO); break;
            case 2: doEnter(&GO); break;
            }
            ftData_SpecialS[F.kind] = NULL;
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
