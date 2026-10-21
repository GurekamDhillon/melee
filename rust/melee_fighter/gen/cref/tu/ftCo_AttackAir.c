/* TU: src/melee/ft/kinds/ftCommon/ftCo_AttackAir.c */
/* C oracle for src/tu/ftCo_AttackAir.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"
#include "ftco_attack_fill.h"

bool ftCo_800DF478(Fighter* f) { T(1); T(f == &F); return chk(1); }
enum_t it_8026B30C(Item_GObj* i) { T(2); T(af_item(i)); return SRV[0] % 5; }
STUB_V(ftCo_800CDDA0, 3)
STUB_V(ftLk_AttackAir_Enter, 4)
STUB_V(ftGw_AttackAirN_DecideAction, 5)
float ftCo_GetCStickAngle(Fighter* f) { T(6); T(f == &F); return angle_of(SRV[3]); }
float ftCo_GetLStickAngle(Fighter* f) { T(7); T(f == &F); return angle_of(SRV[11]); }
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(8); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
STUB_V(ftAnim_8006EBA4, 9)
STUB_CHK(ftAnim_IsFramesRemaining, 10)
STUB_V(ftCo_Fall_Enter, 11)
bool ftCo_80095328(Fighter_GObj* g, bool* o) { T(12); T(g == &GO); T(o == NULL); return chk(12); }
STUB_CHK(ftCo_800D7100, 13)
STUB_CHK(ftCo_800C3B10, 14)
STUB_CHK(ftCo_800D705C, 15)
STUB_CHK(ftCo_800CB870, 16)
STUB_V(ft_80084DB0, 17)
void ftCo_LandingAir_EnterWithLag(Fighter_GObj* g) { T(19); }
void ft_80082C74(Fighter_GObj* g, HSD_GObjEvent cb) { T(18); T(g == &GO); T(cb == ftCo_LandingAir_EnterWithLag); }

int main(void)
{
    int op, k;
    tu_init();
    af_init();
    for (op = 0; op < 16; op++) {
        for (k = 0; k < 2000; k++) {
            u32 ret = 0;
            int ms;
            tu_case(op, k);
            af_fill();
            ms = rnd() % 400;
            switch (op) {
            case 0: ret = checkItemThrowInput(&GO); break;
            case 1: ret = ftCo_AttackAir_CheckItemThrowInput(&GO); break;
            case 2: decideFighter(&GO); break;
            case 3: ret = ftCo_AttackAir_GetMsidFromCStick(&F); break;
            case 4: ftCo_AttackAir_EnterFromCStick(&GO); break;
            case 5: ftCo_AttackAir_EnterFromMsid(&GO, ms); break;
            case 6: ftCo_AttackAir_Anim(&GO); break;
            case 7: ftCo_AttackAirN_IASA(&GO); break;
            case 8: ftCo_AttackAirF_IASA(&GO); break;
            case 9: ftCo_AttackAirB_IASA(&GO); break;
            case 10: ftCo_AttackAirHi_IASA(&GO); break;
            case 11: ftCo_AttackAirLw_IASA(&GO); break;
            case 12: ftCo_AttackAir_Phys(&GO); break;
            case 13: ftCo_AttackAir_Coll(&GO); break;
            case 14: ret = ftCo_AttackAir_GetMsidFromCStick(&F); break;
            case 15: ret = checkItemThrowInput(&GO); break;
            }
            H = 0x811C9DC5u; af_hash();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
