/* TU: src/melee/ft/kinds/ftCommon/ftCo_MissFoot.c */
/* C oracle for src/tu/ftCo_MissFoot.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(1); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
void ftCommon_ClampAirDrift(Fighter* f) { T(2); T(f == &F); }
void ftCommon_8007D5D4(Fighter* f) { T(3); T(f == &F); }
bool ftAnim_IsFramesRemaining(Fighter_GObj* g) { T(4); T(g == &GO); return SR(4); }
void ftCo_80090780(Fighter_GObj* g) { T(5); T(g == &GO); }
void ft_80084DB0(Fighter_GObj* g) { T(6); T(g == &GO); }
void ft_80082F28(Fighter_GObj* g) { T(7); T(g == &GO); }

static u32 state_hash(void) { H = 0x811C9DC5u; HM(B(F.x8c_kb_vel.x)); HM(B(F.x8c_kb_vel.y)); HM(F.ground_or_air); return H; }

int main(void)
{
    int op, k;
    tu_init();
    for (op = 0; op < 5; op++) {
        for (k = 0; k < 1000; k++) {
            u32 ret = 0;
            tu_case(op, k);
            F.x8c_kb_vel.x = rw(); F.x8c_kb_vel.y = rw();
            F.ground_or_air = rnd() & 1;
            switch (op) {
            case 0: ftCo_8009F39C(&GO); break;
            case 1: ftCo_MissFoot_Anim(&GO); break;
            case 2: ftCo_MissFoot_IASA(&GO); break;
            case 3: ftCo_MissFoot_Phys(&GO); break;
            case 4: ftCo_MissFoot_Coll(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
