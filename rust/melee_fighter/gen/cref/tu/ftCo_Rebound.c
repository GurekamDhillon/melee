/* TU: src/melee/ft/kinds/ftCommon/ftCo_Rebound.c */
/* C oracle for src/tu/ftCo_Rebound.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"
#include "ftco_misc8_fill.h"

CMS(1)
void ftCommon_800804A0(Fighter* f, float v) { T(2); T(PI(f)); T(B(v)); }
BSTUB(ftAnim_IsFramesRemaining, 3)
VSTUB(ft_8008A2BC, 4)
VSTUB(ft_80084F3C, 5)
VSTUB(ft_80083F88, 6)

#define NOPS 7
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
            switch (op) {
            case 0: ftCo_80099D9C(&GO); break;
            case 1: ftCo_ReboundStop_Anim(&GO); break;
            case 2: ftCo_80099E44(&GO); break;
            case 3: ftCo_Rebound_Anim(&GO); break;
            case 4: ftCo_Rebound_IASA(&GO); break;
            case 5: ftCo_Rebound_Phys(&GO); break;
            case 6: ftCo_Rebound_Coll(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
