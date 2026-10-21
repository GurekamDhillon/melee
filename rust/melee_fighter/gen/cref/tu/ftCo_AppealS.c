/* TU: src/melee/ft/kinds/ftCommon/ftCo_AppealS.c */
/* C oracle for src/tu/ftCo_AppealS.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"
#include "ftco_attack_fill.h"

DbLKind DbLevel;
static int nd;
static struct ftData_80085FD4_ret DR[4];
STUB_V(ftCl_Init_80149318, 1)
STUB_V(ftDr_Init_80149910, 2)
void lb_8000B1CC(HSD_JObj* j, Vec3* p0, Vec3* p1) { T(3); T(af_jobj(j)); T(p0 == NULL); p1->x = rg(); p1->y = rg(); p1->z = rg(); }
void lb_800119DC(Vec3* p, int n, float a, float b, float c) { T(4); T(B(p->x)); T(B(p->y)); T(B(p->z)); T(n); T(B(a)); T(B(b)); T(B(c)); }
void pl_80040120(int a, int b) { T(5); T(a); T(b); }
struct ftData_80085FD4_ret* ftData_80085FD4(Fighter* f, FtMotionId m) { T(6); T(f == &F); T(m); return &DR[nd++ & 3]; }
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(7); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
STUB_V(ftPe_Init_8011B93C, 8)
STUB_V(ftZd_Init_801395C8, 9)
int ftKb_MexKirbyLike(int k) { T(10); T(k); return chk(10); }
void ftKb_SpecialN_800F5D04(Fighter_GObj* g, bool b) { T(11); T(g == &GO); T(b); }
void Mex_OnTauntDispatch(int kind, void* g, void* v) { T(12); T(kind); T(g == &GO); T(v == NULL); }
STUB_CHK(ftAnim_IsFramesRemaining, 13)
STUB_V(ft_8008A2BC, 14)
STUB_CHK(ftCo_SpecialS_CheckInput, 15)
STUB_CHK(ftCo_Attack100_CheckInput, 16)
STUB_CHK(ftCo_800D6824, 17)
STUB_CHK(ftCo_800D68C0, 18)
STUB_CHK(ftCo_Catch_CheckInput, 19)
STUB_CHK(ftCo_AttackS4_CheckInput, 20)
STUB_CHK(ftCo_AttackHi4_CheckInput, 21)
STUB_CHK(ftCo_AttackLw4_CheckInput, 22)
STUB_CHK(ftCo_AttackS3_CheckInput, 23)
STUB_CHK(ftCo_AttackHi3_CheckInput, 24)
STUB_CHK(ftCo_AttackLw3_CheckInput, 25)
STUB_CHK(ftCo_Attack1_CheckInput, 26)
STUB_CHK(ftCo_80099794, 27)
STUB_CHK(ftCo_80091A4C, 28)
STUB_V(ft_80084FA8, 29)
STUB_V(ft_80084104, 30)

int main(void)
{
    int op, k, i;
    tu_init();
    af_init();
    for (op = 0; op < 9; op++) {
        for (k = 0; k < 2000; k++) {
            u32 ret = 0;
            int m0, m1;
            tu_case(op, k);
            nd = 0;
            for (i = 0; i < 4; i++) DR[i].x8 = SRV[4 + i] & 1;
            af_fill();
            DbLevel = rnd() % 5;
            m0 = rnd() % 400; m1 = rnd() % 400;
            switch (op) {
            case 0: ret = ftCo_800DE9B8(&GO); break;
            case 1: ret = ftCo_800DE9D8(&GO); break;
            case 2: ftCo_800DEA28(&GO); break;
            case 3: ftCo_800DEAE8(&GO, m0, m1); break;
            case 4: ftCo_800DEBD0(&GO); break;
            case 5: ftCo_AppealS_Anim(&GO); break;
            case 6: ftCo_AppealS_IASA(&GO); break;
            case 7: ftCo_AppealS_Phys(&GO); break;
            case 8: ftCo_AppealS_Coll(&GO); break;
            }
            H = 0x811C9DC5u; af_hash();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
