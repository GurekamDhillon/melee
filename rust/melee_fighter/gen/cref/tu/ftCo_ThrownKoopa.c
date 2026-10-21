/* TU: src/melee/ft/kinds/ftCommon/ftCo_ThrownKoopa.c */
/* C oracle for src/tu/ftCo_ThrownKoopa.rs: the whole TU included verbatim; every callee out of it is a
 * recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(1); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == &GO2); }
void ftCo_800DE508(Fighter_GObj* g) { T(2); }
void ftCommon_8007E2F4(Fighter* f, s16 v) { T(3); T(f == &F); T((u32) (s32) v); }
void ftAnim_8006EBA4(Fighter_GObj* g) { T(4); T(g == &GO); }

static u32 state_hash(void)
{
    H = 0x811C9DC5u;
    HM(B(F.facing_dir)); HM((u32) F.mv.co.capturekoopa.x0); HM(F.accessory1_cb != NULL);
    return H;
}

int main(void)
{
    int op, k;
    tu_init();
    for (op = 0; op < 2; op++) {
        for (k = 0; k < 2000; k++) {
            u32 ret = 0;
            tu_case(op, k);
            F.victim_gobj = &GO2;
            F2.facing_dir = rg();
            F.facing_dir = rg();
            F.x2222_b6 = rnd() & 1;
            F.cur_anim_frame = rw();
            F.mv.co.capturekoopa.x0 = rnd() & 1;
            switch (op) {
            case 0: ftCo_800BCDE0(&GO, (FtMotionId) (rnd() % 400)); break;
            case 1: ftCo_800BCE64(&GO, (FtMotionId) (rnd() % 400)); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
