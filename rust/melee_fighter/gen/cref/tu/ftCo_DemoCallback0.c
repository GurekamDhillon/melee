/* TU: src/melee/ft/kinds/ftCommon/ftCo_DemoCallback0.c */
/* C oracle for src/tu/ftCo_DemoCallback0.rs. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(7); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }

static u32 state_hash(void)
{
    H = 0x811C9DC5u;
    HM(F.x2219_b1); HM(F.x2219_b2);
    return H;
}

int main(void)
{
    int k;
    tu_init();
    for (k = 0; k < 200; k++) {
        tu_case(0, k);
        F.x2219_b1 = rnd() & 1; F.x2219_b2 = rnd() & 1;
        ftCo_800C6150(&GO);
        printf("%d %d %08X %08X %08X\n", 0, k, 0, state_hash(), TR);
    }
    return 0;
}
