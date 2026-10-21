/* TU: src/melee/ft/kinds/ftCommon/ftCo_RunDirect.c */
#include TU_SRC
#include "tu_common.h"
#include "ftco_s209_fill.h"

#define VS(name, id) void name(Fighter_GObj* g) { T(id); T(PI(g)); }
#define BS(name, id) bool name(Fighter_GObj* g) { T(id); T(PI(g)); return SR(id); }
VS(ftCo_Run_Anim, 1)
VS(ftCo_Run_Phys, 2)
VS(ftCo_Run_Coll, 3)
BS(ftCo_SpecialS_CheckInput, 4)
BS(ftCo_Attack100_CheckInput, 5)
BS(ftCo_800D6824, 6)
BS(ftCo_800D68C0, 7)
BS(ftCo_800D8A38, 8)
BS(ftCo_AttackDash_CheckInput, 9)
VS(ftCo_AttackDash_SetMv0, 10)
BS(ftCo_80091A4C, 11)
void ftCo_80091B90(Fighter_GObj* g, int n) { T(12); T(PI(g)); T((u32) n); }
VS(ftCo_80091B9C, 13)
BS(ftCo_800DE9D8, 14)
BS(fn_800CAF78, 15)
BS(fn_800CA698, 16)
BS(ft_8008A244, 17)

#define NOPS 4
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
            case 0: ftCo_RunDirect_Anim(&GO); break;
            case 1: ftCo_RunDirect_IASA(&GO); break;
            case 2: ftCo_RunDirect_Phys(&GO); break;
            case 3: ftCo_RunDirect_Coll(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
