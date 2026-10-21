/* TU: src/melee/ft/kinds/ftCommon/ftCo_CargoThrow.c */
/* C oracle for src/tu/ftCo_CargoThrow.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"
#include "ftco_cargo_fill.h"

#define VSTUB(name, id) void name(Fighter_GObj* g) { T(id); T(PI(g)); }
VSTUB(ftCo_8008F744, 1)
VSTUB(ftCo_8009BC58, 2)
VSTUB(ftCo_8009B518, 3)
VSTUB(ftCo_Damage_Phys, 4)
GroundOrAir ft_80082708(Fighter_GObj* g) { T(5); T(PI(g)); return SRV[5] % 2; }
GroundOrAir ft_80081D0C(Fighter_GObj* g) { T(6); T(PI(g)); return SRV[6] % 2; }
void ftCommon_8007D5D4(Fighter* f) { T(7); T(PI(f)); }
void ftCommon_8007D7FC(Fighter* f) { T(8); T(PI(f)); }
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(9); T(PI(g)); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(PI(o)); }
VSTUB(ftAnim_8006EBA4, 10)
void ftCommon_8007E2F4(Fighter* f, s16 v) { T(11); T(PI(f)); T((u32) (s32) v); }
void ftCo_800DE3FC(Fighter_GObj* g, FtMotionId m, float s) { T(12); T(PI(g)); T(m); T(B(s)); }
VSTUB(ftCo_800DD724, 13)
bool ftAnim_IsFramesRemaining(Fighter_GObj* g) { T(14); T(PI(g)); return SR(14); }
VSTUB(ftCommon_8007D92C, 15)
VSTUB(ftCo_ThrowF_IASA, 16)
VSTUB(ftCo_ThrowF_Phys, 17)
VSTUB(ftCo_ThrowB_Anim, 18)
VSTUB(ftCo_ThrowB_IASA, 19)
VSTUB(ftCo_ThrowB_Phys, 20)
VSTUB(ftCo_ThrowHi_Anim, 21)
VSTUB(ftCo_ThrowHi_IASA, 22)
VSTUB(ftCo_ThrowHi_Phys, 23)
VSTUB(ftCo_ThrowLw_Anim, 24)
VSTUB(ftCo_ThrowLw_IASA, 25)
VSTUB(ftCo_ThrowLw_Phys, 26)
void ft_8008403C(Fighter_GObj* g, HSD_GObjEvent cb) { T(27); T(PI(g)); T(cb == ftCo_8009C170); }
VSTUB(ft_80084DB0, 28)
void ft_80082C74(Fighter_GObj* g, HSD_GObjEvent cb) { T(29); T(PI(g)); T(cb == ftCo_8009C45C); }

#define NOPS 27
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
            case 0: ftCo_CargoWait2_Anim(&GO); break;
            case 1: ftCo_CargoWait2_Phys(&GO); break;
            case 2: ftCo_CargoWait2_Coll(&GO); break;
            case 3: ret = ftCo_8009BF3C(&GO); break;
            case 4: ftCo_8009C02C(&GO, (FtMotionId) (rnd() % 400)); break;
            case 5: ftCo_CargoThrowF_Anim(&GO); break;
            case 6: ftCo_CargoThrowF_IASA(&GO); break;
            case 7: ftCo_CargoThrowF_Phys(&GO); break;
            case 8: ftCo_8009C170(&GO); break;
            case 9: ftCo_CargoThrowF_Coll(&GO); break;
            case 10: ftCo_CargoThrowB_Anim(&GO); break;
            case 11: ftCo_CargoThrowB_IASA(&GO); break;
            case 12: ftCo_CargoThrowB_Phys(&GO); break;
            case 13: ftCo_CargoThrowB_Coll(&GO); break;
            case 14: ftCo_CargoThrowHi_Anim(&GO); break;
            case 15: ftCo_CargoThrowHi_IASA(&GO); break;
            case 16: ftCo_CargoThrowHi_Phys(&GO); break;
            case 17: ftCo_CargoThrowHi_Coll(&GO); break;
            case 18: ftCo_CargoThrowLw_Anim(&GO); break;
            case 19: ftCo_CargoThrowLw_IASA(&GO); break;
            case 20: ftCo_CargoThrowLw_Phys(&GO); break;
            case 21: ftCo_CargoThrowLw_Coll(&GO); break;
            case 22: ftCo_CargoThrowAir_Anim(&GO); break;
            case 23: ftCo_CargoThrowAir_IASA(&GO); break;
            case 24: ftCo_CargoThrowAir_Phys(&GO); break;
            case 25: ftCo_CargoThrowAir_Coll(&GO); break;
            case 26: ftCo_8009C45C(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
