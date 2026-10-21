/* TU: src/melee/ft/kinds/ftCommon/ftCo_CargoLanding.c */
#include TU_SRC
#include "tu_common.h"
#include "ftco_cargo_fill.h"

#define VSTUB(name, id) void name(Fighter_GObj* g) { T(id); T(PI(g)); }
#define CMS void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o) \
{ T(9); T(PI(g)); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(PI(o)); }
void ftCo_Landing_Enter(Fighter_GObj* g, FtMotionId m, bool ai, MotionFlags fl, float a, float b)
{ T(1); T(PI(g)); T(m); T(ai); T(fl); T(B(a)); T(B(b)); }
void ftAnim_SetAnimRate(Fighter_GObj* g, float r) { T(2); T(PI(g)); T(B(r)); }
void ftCo_8009C5A4(Fighter_GObj* g, int m) { T(3); T(PI(g)); T(m); }
VSTUB(ftCo_8009B518, 4)
VSTUB(ft_80084F3C, 5)
VSTUB(ftCo_CargoWait_Coll, 6)

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
            cargo_fill();
            switch (op) {
            case 0: ftCo_8009BD4C(&GO); break;
            case 1: ftCo_CargoLanding_Anim(&GO); break;
            case 2: ftCo_CargoLanding_Phys(&GO); break;
            case 3: ftCo_CargoLanding_Coll(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
