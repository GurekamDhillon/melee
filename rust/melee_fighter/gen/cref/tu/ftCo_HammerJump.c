/* TU: src/melee/ft/kinds/ftCommon/ftCo_HammerJump.c */
/* C oracle for src/tu/ftCo_HammerJump.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

static float FR;
static u32 FL, JI;
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(1); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
u32 ftCo_800C54C4(Fighter* f) { T(2); T(f == &F); return FL; }
float ftCo_800C548C(Fighter* f) { T(3); T(f == &F); return FR; }
void ftCo_800C4E94(Fighter* f) { T(4); T(f == &F); }
void ftCo_800C4F64(Fighter_GObj* g) { T(5); T(g == &GO); }
ftCo_JumpInput ftCo_Jump_GetInput(Fighter_GObj* g) { T(6); T(g == &GO); return JI; }
void ftCo_800C5B88(Fighter_GObj* g, enum_t e) { T(7); T(g == &GO); T(e); }
void ftCommon_8007D5D4(Fighter* f) { T(8); T(f == &F); }
void ftCo_800CB110(Fighter_GObj* g, bool b, float v) { T(9); T(g == &GO); T(b); T(B(v)); }
void ftCo_Jump_Phys_Inner(Fighter_GObj* g) { T(10); T(g == &GO); }
void ftCo_HammerFall_Coll(Fighter_GObj* g) { T(11); T(g == &GO); }

int main(void)
{
    int op, k;
    tu_init();
    for (op = 0; op < 6; op++) {
        for (k = 0; k < 2000; k++) {
            u32 ret = 0;
            tu_case(op, k);
            FR = rw(); FL = rnd(); JI = rnd() % 4;
            switch (op) {
            case 0: ret = ftCo_800C5A50(&GO); break;
            case 1: ftCo_800C5A98(&GO); break;
            case 2: ftCo_HammerJump_Anim(&GO); break;
            case 3: ftCo_HammerJump_IASA(&GO); break;
            case 4: ftCo_HammerJump_Phys(&GO); break;
            case 5: ftCo_HammerJump_Coll(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, 0x811C9DC5u, TR);
        }
    }
    return 0;
}
