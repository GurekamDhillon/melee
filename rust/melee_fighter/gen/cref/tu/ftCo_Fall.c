/* TU: src/melee/ft/kinds/ftCommon/ftCo_Fall.c */
/* C oracle for src/tu/ftCo_Fall.rs: the whole TU included verbatim; every callee out of it is a
 * recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"
#include "ftco_dfe_fill.h"

DS_V(ftCo_80090780, 1)
DS_CHK(ftCo_800C5240, 2)
DS_V(ftCo_800C5D34, 3)
FighterKind ftLib_GetKind(HSD_GObj* g)
{
    T(4); T(g == &GO);
    switch (SRV[5] % 4) {
    case 0: return Ft_Kind_MasterH;
    case 1: return Ft_Kind_CrezyH;
    default: return F.kind;
    }
}
DS_V(ftMh_MS_389_80151018, 5)
DS_V(ftCh_GrabUnk1_8015BC88, 6)
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(7); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
DS_FV(ftCommon_ClampAirDrift, 8)
DS_FV(ftCommon_8007D5D4, 9)
void HSD_JObjAnimAll(HSD_JObj* j) { T(10); T(j == (HSD_JObj*) DF_SKEL); }
void ftAnim_8006FE9C(Fighter* f, Fighter_Part p, float a, float b) { T(11); T(f == &F); T(p); T(B(a)); T(B(b)); }
void ftAnim_8006FF74(Fighter* f, Fighter_Part p) { T(12); T(f == &F); T(p); }
void ftAnim_8006EDD0(Fighter* f, int s, float a, float b) { T(13); T(f == &F); T(s); T(B(a)); T(B(b)); }
DS_CHK(ftCo_SpecialAir_CheckInput, 14)
bool ftCo_80095328(Fighter_GObj* g, bool* b) { T(15); T(g == &GO); T(b == NULL); return chk(15); }
DS_CHK(ftCo_800D7100, 16)
DS_CHK(ftCo_800C3B10, 17)
DS_CHK(ftCo_80099A58, 18)
DS_CHK(ftCo_AttackAir_CheckItemThrowInput, 19)
DS_CHK(ftCo_800D705C, 20)
DS_CHK(ftCo_800CEE70, 21)
DS_CHK(ftPe_8011BA54, 22)
DS_CHK(ftCo_800CB870, 23)
DS_CHK(ftPe_8011BAD8, 24)
DS_V(ft_80084DB0, 25)
bool ftCo_80096CC8(Fighter_GObj* g, int l) { (void) g; (void) l; return 0; }
void ft_80082B1C(Fighter_GObj* g) { (void) g; }
void ft_800831CC(Fighter_GObj* g, bool (*a)(Fighter_GObj*, int), HSD_GObjEvent b)
{ T(26); T(g == &GO); T(a == ftCo_80096CC8); T(b == (HSD_GObjEvent) ft_80082B1C); }

int main(void)
{
    int op, k;
    tu_init();
    for (op = 0; op < 11; op++) {
        for (k = 0; k < 3000; k++) {
            u32 ret = 0;
            float a;
            int s0, s1, s2;
            tu_case(op, k);
            df_fill();
            switch (rnd() % 3) { case 0: a = 0.0f; break; case 1: a = 1.0f; break; default: a = rg(); break; }
            s0 = rnd() % 64; s1 = rnd() % 64; s2 = rnd() % 64;
            switch (op) {
            case 0: ftCo_Fall_Enter(&GO); break;
            case 1: ftCo_Fall_Enter_YoshiEgg(&GO); break;
            case 2: ftCo_Fall_Enter_YoshiEgg_Kirby(&GO); break;
            case 3: ftCo_800CC988(&GO, a); break;
            case 4: ftCo_Fall_Anim(&GO); break;
            case 5: ret = ftCo_Fall_IASA_Inner(&GO); break;
            case 6: ftCo_Fall_Anim_Inner(&GO, (rnd() & 1) ? &F.mv.co.fall.x4 : &F.mv.co.fallspecial.x4, s0, s1, s2); break;
            case 7: ftCo_Fall_IASA(&GO); break;
            case 8: ftCo_Fall_Phys(&GO); break;
            case 9: ftCo_Fall_Coll(&GO); break;
            case 10: ret = ftCo_Fall_inline(&GO); break;
            }
            H = 0x811C9DC5u; df_hash();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
