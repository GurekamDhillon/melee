/* TU: src/melee/ft/kinds/ftCommon/ftCo_FallSpecial.c */
/* C oracle for src/tu/ftCo_FallSpecial.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"
#include "ftco_dfe_fill.h"

DS_V(ftCo_80090780, 1)
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(2); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
DS_FV(ftCommon_8007D60C, 3)
DS_FV(ftCommon_UseAllJumps, 4)
int un_80322598(int a, float y) { T(5); T(a); T(B(y)); return 0; }
void ftCo_Fall_Anim_Inner(Fighter_GObj* g, f32* p, FtMotionId a, FtMotionId b, FtMotionId c)
{ T(6); T(g == &GO); T(p == &F.mv.co.fallspecial.x4); T(a); T(b); T(c); *p = rg(); }
void ftCo_800CC988(Fighter_GObj* g, f32 a) { T(7); T(g == &GO); T(B(a)); }
DS_CHK(ftCo_800CEE70, 8)
DS_CHK(ftCo_800D705C, 9)
DS_CHK(ftCo_800CB870, 10)
bool ftCommon_CheckFallFast(Fighter* f) { T(11); T(f == &F); return chk(11); }
DS_FV(ftCommon_FallFast, 12)
void ftCommon_Fall(Fighter* f, float a, float b) { T(13); T(f == &F); T(B(a)); T(B(b)); }
void ftCommon_CalcSelfAccel_AccelToVelClamped(Fighter* f, float a, float b, float c) { T(14); T(f == &F); T(BN(a)); T(BN(b)); T(BN(c)); }
int Geno_FallSpecialColl(Fighter_GObj* g) { T(15); T(g == &GO); return chk(15); }
void ft_80083090(Fighter_GObj* g, bool (*a)(Fighter_GObj*, int), HSD_GObjEvent b)
{ T(16); T(g == &GO); T(a == ftCo_80096CC8); T(b == ftCo_80096D28); }
u32 mpLineGetFlags(int l) { T(17); T(l); return SRV[6] & 0x1FF; }
float ftCo_800D0EC8(Fighter* f) { T(18); T(f == &F); return rw(); }
void ftCo_LandingFallSpecial_Enter(Fighter_GObj* g, bool a, float b) { T(19); T(g == &GO); T(a); T(B(b)); }
DS_V(ft_8008A2BC, 20)

int main(void)
{
    int op, k;
    tu_init();
    for (op = 0; op < 9; op++) {
        for (k = 0; k < 3000; k++) {
            u32 ret = 0;
            int a1, a2, ai, line;
            float mob, ll, a6;
            tu_case(op, k);
            df_fill();
            a1 = rnd() % 3; a2 = rnd() % 3; ai = rnd() % 3;
            mob = rg(); ll = rw(); a6 = rg();
            line = (rnd() & 1) ? -1 : (int) (rnd() % 8);
            switch (op) {
            case 0: ftCo_800968C8(&GO); break;
            case 1: ftCo_80096900(&GO, a1, a2, ai, mob, ll); break;
            case 2: ftCo_800969D8(&GO, a1, a2, ai, mob, ll, a6); break;
            case 3: ftCo_FallSpecial_Anim(&GO); break;
            case 4: ftCo_FallSpecial_IASA(&GO); break;
            case 5: ftCo_FallSpecial_Phys(&GO); break;
            case 6: ftCo_FallSpecial_Coll(&GO); break;
            case 7: ret = ftCo_80096CC8(&GO, line); break;
            case 8: ftCo_80096D28(&GO); break;
            }
            H = 0x811C9DC5u; df_hash();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
