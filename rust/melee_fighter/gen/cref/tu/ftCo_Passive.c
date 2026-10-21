/* TU: src/melee/ft/kinds/ftCommon/ftCo_Passive.c */
/* C oracle for src/tu/ftCo_Passive.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"
#include "ftco_misc8_fill.h"

void ftCommon_8007D7FC(Fighter* f) { T(1); T(PI(f)); }
CMS(2)
void ft_800881D8(Fighter* f, int s, u8 v, u8 p) { T(3); T(PI(f)); T(s); T(v); T(p); }
void ft_PlaySFX(Fighter* f, enum_t s, u8 v, u8 p) { T(4); T(PI(f)); T(s); T(v); T(p); }
void ftCommon_SetGroundedKnockbackIfLanded(Fighter* f) { T(5); T(PI(f)); }
void efAsync_Spawn(HSD_GObj* g, void* q, u32 kind, u32 gfx, HSD_JObj* j, ...) { T(6); T(PI(g)); T(PI(q)); T(kind); T(gfx); T(PI(j)); }
bool ftCo_800BFFD0(Fighter* f, enum_t a, bool b) { T(7); T(PI(f)); T(a); T(b); return SR(7); }
BSTUB(ftAnim_IsFramesRemaining, 8)
VSTUB(ft_8008A2BC, 9)
VSTUB(ft_80084F3C, 10)
VSTUB(ft_80083F88, 11)

#define NOPS 5
#define NCASE 2000

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
            case 0: ftCo_800987D0(&GO); break;
            case 1: ftCo_Passive_Anim(&GO); break;
            case 2: ftCo_Passive_IASA(&GO); break;
            case 3: ftCo_Passive_Phys(&GO); break;
            case 4: ftCo_Passive_Coll(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
