/* TU: src/melee/ft/kinds/ftCommon/ftCo_ShieldBreakStand.c */
#include TU_SRC
#include "tu_common.h"
#include "ftco_s209_fill.h"

#define VS(name, id) void name(Fighter_GObj* g) { T(id); T(PI(g)); }
#define BS(name, id) bool name(Fighter_GObj* g) { T(id); T(PI(g)); return SR(id); }
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(1); T(PI(g)); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(PI(o)); }
BS(ftAnim_IsFramesRemaining, 2)
VS(ftCo_80099010, 3)
VS(ft_80084F3C, 4)
VS(ft_80083F88, 5)

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
            case 0: ftCo_80098F3C(&GO); break;
            case 1: ftCo_ShieldBreakStand_Anim(&GO); break;
            case 2: ftCo_ShieldBreakStand_IASA(&GO); break;
            case 3: ftCo_ShieldBreakStand_Phys(&GO); break;
            case 4: ftCo_ShieldBreakStand_Coll(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
