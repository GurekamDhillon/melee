/* TU: src/melee/ft/kinds/ftCommon/ftCo_CargoFall.c */
#include TU_SRC
#include "tu_common.h"
#include "ftco_cargo_fill.h"

#define VSTUB(name, id) void name(Fighter_GObj* g) { T(id); T(PI(g)); }
#define CMS void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o) \
{ T(9); T(PI(g)); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(PI(o)); }
CMS
void ftAnim_SetAnimRate(Fighter_GObj* g, float r) { T(2); T(PI(g)); T(B(r)); }
void ftCommon_8007D5D4(Fighter* f) { T(3); T(PI(f)); }
void ftCo_8009C5A4(Fighter_GObj* g, int m) { T(4); T(PI(g)); T(m); }
bool ftCo_8009BF3C(Fighter_GObj* g) { T(5); T(PI(g)); return SR(5); }
VSTUB(ft_80084DB0, 6)
void ft_80082C74(Fighter_GObj* g, HSD_GObjEvent cb) { T(7); T(PI(g)); T(cb == ftCo_8009BD4C); }
void ftCo_8009BD4C(Fighter_GObj* g) { T(8); }

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
            cargo_fill();
            switch (op) {
            case 0: ftCo_8009BC58(&GO); break;
            case 1: ftCo_CargoFall_Anim(&GO); break;
            case 2: ftCo_CargoFall_IASA(&GO); break;
            case 3: ftCo_CargoFall_Phys(&GO); break;
            case 4: ftCo_CargoFall_Coll(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
