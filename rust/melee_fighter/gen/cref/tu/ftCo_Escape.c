/* TU: src/melee/ft/kinds/ftCommon/ftCo_Escape.c */
/* C oracle for src/tu/ftCo_Escape.rs: the whole TU included verbatim (TARGET_PC: the UCF checks);
 * every callee out of it is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"
#include "ftco_dfe_fill.h"

bool ftCo_800DF8B0(Fighter* f) { T(1); T(f == &F); return chk(1); }
bool ftCo_800DF8E8(Fighter* f) { T(2); T(f == &F); return chk(2); }
void ftCommon_8007EBAC(Fighter* f, u32 a, u32 b) { T(3); T(f == &F); T(a); T(b); }
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(4); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
DS_V(ftAnim_8006EBA4, 5)
void ftParts_80074B0C(Fighter_GObj* g, int a, int b) { T(6); T(g == &GO); T(a); T(b); }
DS_V(ftYs_Init_8012BDA0, 7)
DS_CHK(ftAnim_IsFramesRemaining, 8)
DS_V(ft_8008A2BC, 9)
DS_V(ftSs_SpecialLw_8012AEBC, 10)
DS_V(ftSs_SpecialLw_8012AF38, 11)
DS_CHK(ftYs_Shield_8012CC1C, 12)
DS_V(ftYs_Init_8012BE3C, 13)
DS_V(ftYs_Init_8012B8A4, 14)
DS_CHK(ftCo_8009563C, 15)
DS_V(ft_80085004, 16)
DS_V(ft_80084104, 17)
void ft_800847D0(Fighter_GObj* g, ftCollisionBox* b)
{ T(18); T(g == &GO); T(b == &((ftSs_DatAttrs*) DF_DAT)->height_attributes); }
int Replay_UcfVersion(int port) { static const int v[3] = { 73, 84, 0 }; T(19); T(port); return v[SRV[7] % 3]; }
DS_V(ft_80084F3C, 20)

static u32 cbid(HSD_GObjEvent p)
{
    if (p == NULL) return 0;
    if (p == ftCo_80099564) return 1;
    if (p == ftCo_80099754) return 2;
    if (p == ftCo_80099644) return 3;
    return 99;
}
static float fine(void) { return (float) ((int) (rnd() % 20001) - 10000) / 10000.0f; }

int main(void)
{
    int op, k;
    tu_init();
    for (op = 0; op < 26; op++) {
        for (k = 0; k < 2000; k++) {
            u32 ret = 0;
            int ms, a2;
            float v;
            tu_case(op, k);
            df_fill();
            ms = rnd() % 400; a2 = rnd() % 3;
            v = (rnd() & 1) ? rg() : fine();
            if (rnd() & 1) { F.input.lstick[0].x = fine(); F.input.lstick[0].y = fine(); }
            switch (op) {
            case 0: ret = ftCo_8009917C(&GO); break;
            case 1: ret = ftCo_80099264(&GO); break;
            case 2: ftCo_800992A8(&GO, ms, a2); break;
            case 3: ftCo_80099314(&GO, ms, a2); break;
            case 4: ftCo_80099390(&GO, ms, a2); break;
            case 5: ftCo_80099438(&GO, ms, a2); break;
            case 6: ftCo_Escape_Anim(&GO); break;
            case 7: ftCo_80099564(&GO); break;
            case 8: ftCo_80099644(&GO); break;
            case 9: ftCo_Escape_IASA(&GO); break;
            case 10: ftCo_Escape_Phys(&GO); break;
            case 11: ftCo_Escape_Coll(&GO); break;
            case 12: ftCo_80099754(&GO); break;
            case 13: ret = B(ftCo_Ucf073RimAxis(v)); break;
            case 14: ret = ftCo_Ucf073BlocksSpotdodge(&F); break;
            case 15: ret = ftCo_Ucf084RimUnits(v); break;
            case 16: ret = ftCo_Ucf084BlocksSpotdodge(&F); break;
            case 17: ret = ftCo_80099794(&GO); break;
            case 18: ret = ftCo_8009980C(&GO); break;
            case 19: ftCo_80099894(&GO); break;
            case 20: ftCo_800998EC(&GO); break;
            case 21: ftCo_80099954(&GO); break;
            case 22: ftCo_EscapeN_Anim(&GO); break;
            case 23: ftCo_EscapeN_IASA(&GO); break;
            case 24: ftCo_EscapeN_Phys(&GO); break;
            case 25: ftCo_EscapeN_Coll(&GO); break;
            }
            H = 0x811C9DC5u; df_hash(); HM(cbid(F.anim_cb)); HM(cbid(F.coll_cb));
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
