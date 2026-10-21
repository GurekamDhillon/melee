/* TU: src/melee/ft/kinds/ftCommon/ftCo_CargoWait.c */
/* C oracle for src/tu/ftCo_CargoWait.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

static ftDonkeyAttributes DK;
bool ft_8008A1FC(Fighter_GObj* g) { T(1); T(g == &GO); return SR(1); }
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(2); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
void ftCo_8009C5A4(Fighter_GObj* g, int m) { T(3); T(g == &GO2); T(m); }
void ftCo_8009C640(Fighter_GObj* g, int m) { T(4); T(g == &GO2); T(m); }
bool ftCo_8009BF3C(Fighter_GObj* g) { T(5); return SR(2); }
bool ftCo_8009BB1C(Fighter_GObj* g) { T(6); return SR(3); }
bool ftCo_8009C4F8(Fighter_GObj* g) { T(7); return SR(4); }
bool ftCo_8009B818(Fighter_GObj* g) { T(8); return SR(5); }
bool ftCo_8009B67C(Fighter_GObj* g) { T(9); return SR(6); }
void ft_80084F3C(Fighter_GObj* g) { T(10); T(g == &GO); }
void ftCo_8009BC58(Fighter_GObj* g) { T(11); T(g == &GO); }
void ft_8008403C(Fighter_GObj* g, HSD_GObjEvent cb) { T(12); T(g == &GO); T(cb == ftCo_8009BC58); cb(g); }

static u32 state_hash(void) { H = 0x811C9DC5u; HM(F.motion_id); return H; }

int main(void)
{
    int op, k;
    tu_init();
    for (op = 0; op < 8; op++) {
        for (k = 0; k < 2000; k++) {
            u32 ret = 0;
            tu_case(op, k);
            zero(&DK, sizeof DK);
            DK.x4_motion_state = rnd() % 400;
            F.x2CC = &DK; F.victim_gobj = &GO2;
            switch (op) {
            case 0: ret = ftCo_8009B4D0(&GO); break;
            case 1: ftCo_8009B518(&GO); break;
            case 2: ftCo_8009B56C(&GO); break;
            case 3: ftCo_CargoWait_Anim(&GO); break;
            case 4: ftCo_CargoWait_IASA(&GO); break;
            case 5: ftCo_CargoWait_Phys(&GO); break;
            case 6: ftCo_CargoWait_Coll(&GO); break;
            case 7: ftCo_8009B518(&GO); ftCo_8009B56C(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
