/* TU: src/melee/ft/kinds/ftCommon/ftCo_ShieldBreakFall.c */
#include TU_SRC
#include "tu_common.h"
#include "ftco_s209_fill.h"

#define VS(name, id) void name(Fighter_GObj* g) { T(id); T(PI(g)); }
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(1); T(PI(g)); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(PI(o)); }
void ftCommon_ClampAirDrift(Fighter* f) { T(2); T(PI(f)); }
void ftCommon_8007EBAC(Fighter* f, u32 a, u32 b) { T(3); T(PI(f)); T(a); T(b); }
VS(ft_80084EEC, 4)
VS(ftCo_80098E3C, 6) /* ftCo_ShieldBreakDown.c; only its address is used */
void ft_80082C74(Fighter_GObj* g, HSD_GObjEvent cb) { T(5); T(PI(g)); T(cb == ftCo_80098E3C); }

#define NOPS 5
#define NCASE 1000

int main(void)
{
    int op, k;
    tu_init();
    for (op = 0; op < NOPS; op++) {
        for (k = 0; k < NCASE; k++) {
            u32 ret = 0;
            tu_case(op, k);
            s209_fill();
            switch (op) {
            case 0: ftCo_80098D90(&GO); break;
            case 1: ftCo_ShieldBreakFall_Anim(&GO); break;
            case 2: ftCo_ShieldBreakFall_IASA(&GO); break;
            case 3: ftCo_ShieldBreakFall_Phys(&GO); break;
            case 4: ftCo_ShieldBreakFall_Coll(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
