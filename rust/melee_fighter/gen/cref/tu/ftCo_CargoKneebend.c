/* TU: src/melee/ft/kinds/ftCommon/ftCo_CargoKneebend.c */
#include TU_SRC
#include "tu_common.h"
#include "ftco_cargo_fill.h"

#define VSTUB(name, id) void name(Fighter_GObj* g) { T(id); T(PI(g)); }
#define CMS void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o) \
{ T(9); T(PI(g)); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(PI(o)); }
CMS
void ftAnim_SetAnimRate(Fighter_GObj* g, float r) { T(1); T(PI(g)); T(B(r)); }
void ftCo_8009C5A4(Fighter_GObj* g, int m) { T(2); T(PI(g)); T(m); }
VSTUB(ftCo_8009BB64, 3)
bool ftCo_8009BF3C(Fighter_GObj* g) { T(4); T(PI(g)); return SR(4); }
VSTUB(ftCo_KneeBend_Check_ShortHop, 5)
VSTUB(ft_80084F3C, 6)
VSTUB(ftCo_CargoWait_Coll, 7)

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
            case 0: ftCo_8009B9C8(&GO, (int) rnd()); break;
            case 1: ftCo_CargoKneebend_Anim(&GO); break;
            case 2: ftCo_CargoKneebend_IASA(&GO); break;
            case 3: ftCo_CargoKneebend_Phys(&GO); break;
            case 4: ftCo_CargoKneebend_Coll(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
