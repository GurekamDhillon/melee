/* TU: src/melee/ft/kinds/ftCommon/ftCo_CargoJump.c */
#include TU_SRC
#include "tu_common.h"
#include "ftco_cargo_fill.h"

#define VSTUB(name, id) void name(Fighter_GObj* g) { T(id); T(PI(g)); }
#define CMS void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o) \
{ T(9); T(PI(g)); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(PI(o)); }
CMS
ftCo_JumpInput ftCo_Jump_GetInput(Fighter_GObj* g) { T(1); T(PI(g)); return SRV[1] % 3; }
void ftCo_8009B9C8(Fighter_GObj* g, int a) { T(2); T(PI(g)); T(a); }
void ftCommon_8007D5D4(Fighter* f) { T(3); T(PI(f)); }
void ftAnim_SetAnimRate(Fighter_GObj* g, float r) { T(4); T(PI(g)); T(B(r)); }
void ftCo_800CB110(Fighter_GObj* g, bool b, float f) { T(5); T(PI(g)); T(b); T(B(f)); }
void ftCo_8009C5A4(Fighter_GObj* g, int m) { T(6); T(PI(g)); T(m); }
bool ftCo_8009BF3C(Fighter_GObj* g) { T(7); T(PI(g)); return SR(7); }
VSTUB(ftCo_Jump_Phys_Inner, 8)
VSTUB(ftCo_CargoFall_Coll, 10)

#define NOPS 6
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
            case 0: ret = ftCo_8009BB1C(&GO); break;
            case 1: ftCo_8009BB64(&GO); break;
            case 2: ftCo_CargoJump_Anim(&GO); break;
            case 3: ftCo_CargoJump_IASA(&GO); break;
            case 4: ftCo_CargoJump_Phys(&GO); break;
            case 5: ftCo_CargoJump_Coll(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
