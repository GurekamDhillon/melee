/* TU: src/melee/ft/kinds/ftCommon/ftCo_ShieldBreakDown.c */
#include TU_SRC
#include "tu_common.h"
#include "ftco_s209_fill.h"

#define VS(name, id) void name(Fighter_GObj* g) { T(id); T(PI(g)); }
#define BS(name, id) bool name(Fighter_GObj* g) { T(id); T(PI(g)); return SR(id); }
void ftCommon_8007D7FC(Fighter* f) { T(1); T(PI(f)); }
BS(ftCo_80097570, 2)
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(3); T(PI(g)); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(PI(o)); }
VS(ftCo_800978D4, 4)
BS(ftAnim_IsFramesRemaining, 5)
VS(ftCo_80098F3C, 6)
VS(ft_80084F3C, 7)
VS(ft_80083F88, 8)

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
            case 0: ftCo_80098E3C(&GO); break;
            case 1: ftCo_ShieldBreakDown_Anim(&GO); break;
            case 2: ftCo_ShieldBreakDown_IASA(&GO); break;
            case 3: ftCo_ShieldBreakDown_Phys(&GO); break;
            case 4: ftCo_ShieldBreakDown_Coll(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
