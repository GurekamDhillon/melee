/* TU: src/melee/ft/kinds/ftCommon/ftCo_AttackDash.c */
/* C oracle for src/tu/ftCo_AttackDash.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret(^first mv word) state-hash trace. */
#include TU_SRC
#include "tu_common.h"
#include "ftco_attack_fill.h"

enum_t it_8026B30C(Item_GObj* i) { T(1); T(af_item(i)); return SRV[0] % 5; }
void ftCo_800957F4(Fighter_GObj* g, FtMotionId m) { T(2); T(g == &GO); T(m); }
void ftCo_Attack_800CCF58(Fighter_GObj* g, enum_t a) { T(3); T(g == &GO); T(a); }
int ftKb_MexKirbyLike(int k) { T(4); T(k); return chk(4); }
STUB_V(ftKb_SpecialN_800F1F68, 5)
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(6); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
STUB_V(ftAnim_8006EBA4, 7)
STUB_CHK(ftAnim_IsFramesRemaining, 8)
STUB_V(ft_8008A2BC, 9)
STUB_CHK(ftCo_800D8AE0, 10)
STUB_V(ftCo_Wait_IASA, 11)
void ft_80085030(Fighter_GObj* g, float a, float b) { T(12); T(g == &GO); T(B(a)); T(B(b)); }
STUB_V(ft_80084104, 13)

int main(void)
{
    int op, k;
    tu_init();
    af_init();
    for (op = 0; op < 8; op++) {
        for (k = 0; k < 2000; k++) {
            u32 ret = 0;
            tu_case(op, k);
            af_fill();
            *(u32*) &F.mv = rnd();
            switch (op) {
            case 0: ret = ftCo_AttackDash_CheckInput(&GO); break;
            case 1: decideFighter(&GO); break;
            case 2: doEnter(&GO); break;
            case 3: ftCo_AttackDash_Anim(&GO); break;
            case 4: ftCo_AttackDash_SetMv0(&GO); break;
            case 5: ftCo_AttackDash_IASA(&GO); break;
            case 6: ftCo_AttackDash_Phys(&GO); break;
            case 7: ftCo_AttackDash_Coll(&GO); break;
            }
            ret ^= *(u32*) &F.mv;
            H = 0x811C9DC5u; af_hash();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
