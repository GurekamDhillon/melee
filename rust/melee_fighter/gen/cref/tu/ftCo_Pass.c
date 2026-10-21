/* TU: src/melee/ft/kinds/ftCommon/ftCo_Pass.c */
/* C oracle for src/tu/ftCo_Pass.rs (TARGET_PC): the whole TU included verbatim; every callee out
 * of it is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"
#include "ftco_misc8_fill.h"

bool mpColl_IsOnPlatform(CollData* c) { T(1); T(c == &F.coll_data); return SR(1); }
void mpUpdateFloorSkip(CollData* c) { T(2); T(c == &F.coll_data); }
BSTUB(ftCo_800C5240, 3)
BSTUB(ftCo_800C60C8, 4)
int Replay_UcfVersion(int port) { T(5); T(port); return (SRV[5] & 1) ? 84 : (int) (rnd() % 90); }
int ftUcf_FlickCounter(int port) { T(6); T(port); return (int) (rnd() % 4); }
void ftCommon_8007D5D4(Fighter* f) { T(7); T(PI(f)); }
void ftCommon_ClampAirDrift(Fighter* f) { T(8); T(PI(f)); }
CMS(9)
BSTUB(ftAnim_IsFramesRemaining, 10)
VSTUB(ftCo_Fall_Enter, 11)
BSTUB(ftCo_SpecialAir_CheckInput, 12)
bool ftCo_80095328(Fighter_GObj* g, bool* b) { T(13); T(PI(g)); T(b == NULL); return SR(13); }
BSTUB(ftCo_800D7100, 14)
BSTUB(ftCo_800C3B10, 15)
BSTUB(ftCo_80099A58, 16)
BSTUB(ftCo_AttackAir_CheckItemThrowInput, 17)
BSTUB(ftCo_800D705C, 18)
BSTUB(ftCo_800CB870, 19)
VSTUB(ft_80084DB0, 20)
VSTUB(ft_80082F28, 21)

#define NOPS 11
#define NCASE 3000

int main(void)
{
    int op, k;
    tu_init();
    for (op = 0; op < NOPS; op++) {
        for (k = 0; k < NCASE; k++) {
            u32 ret = 0;
            tu_case(op, k);
            m8_fill();
            switch (op) {
            case 0: ret = ftCo_80099F1C(&GO); break;
            case 1: ret = ftCo_80099F9C(&GO); break;
            case 2: ret = ftCo_8009A080(&GO); break;
            case 3: ret = ftCo_8009A134(&GO); break;
            case 4: { FtMotionId ms = (FtMotionId) (rnd() % 400); MotionFlags mf = rnd(); float st = rg(); ftCo_8009A184(&GO, ms, mf, st); break; }
            case 5: ftCo_8009A228(&GO); break;
            case 6: ftCo_Pass_Anim(&GO); break;
            case 7: ftCo_Pass_IASA(&GO); break;
            case 8: ftCo_Pass_Phys(&GO); break;
            case 9: ftCo_Pass_Coll(&GO); break;
            case 10: ret = ftCo_8009A080(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
