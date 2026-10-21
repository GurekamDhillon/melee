/* TU: src/melee/ft/kinds/ftCommon/ftCo_PassiveStand.c */
/* C oracle for src/tu/ftCo_PassiveStand.rs: the whole TU included verbatim; every callee out of
 * it is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"
#include "ftco_misc8_fill.h"

BSTUB(ftCo_800986B0, 1)
void ftCommon_8007D7FC(Fighter* f) { T(2); T(PI(f)); }
CMS(3)
void ft_800881D8(Fighter* f, int s, u8 v, u8 p) { T(4); T(PI(f)); T(s); T(v); T(p); }
void ft_PlaySFX(Fighter* f, enum_t s, u8 v, u8 p) { T(5); T(PI(f)); T(s); T(v); T(p); }
void efAsync_Spawn(HSD_GObj* g, void* q, u32 kind, u32 gfx, HSD_JObj* j, ...) { T(6); T(PI(g)); T(PI(q)); T(kind); T(gfx); T(PI(j)); }
BSTUB(ftAnim_IsFramesRemaining, 7)
VSTUB(ft_8008A2BC, 8)
VSTUB(ft_80084FA8, 9)
VSTUB(ft_80084104, 10)

#define NOPS 6
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
            case 0: ret = ftCo_80098928(&GO); break;
            case 1: ftCo_800989D4(&GO, (FtMotionId) (rnd() % 400)); break;
            case 2: ftCo_PassiveStand_Anim(&GO); break;
            case 3: ftCo_PassiveStand_IASA(&GO); break;
            case 4: ftCo_PassiveStand_Phys(&GO); break;
            case 5: ftCo_PassiveStand_Coll(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
