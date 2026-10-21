/* TU: src/melee/ft/kinds/ftCommon/ftCo_StopCeil.c */
/* C oracle for src/tu/ftCo_StopCeil.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"
#include "s217_fill.h"

CMS_STUB
void ftKb_SpecialN_800F1F1C(Fighter_GObj* g, Vec3* v) { T(10); T(PI(g)); T(B(v->x)); T(B(v->y)); T(B(v->z)); if (chk(10)) v->y = v->y + 1.0f; }
void ft_80082D40(Fighter_GObj* g, float y) { T(11); T(PI(g)); T(B(y)); }
STUB_CHK(ftAnim_IsFramesRemaining, 12)
STUB_V(ftCo_Fall_Enter, 13)
void ft_80083464(Fighter_GObj* g, bool (*a)(Fighter_GObj*, int), HSD_GObjEvent b) { T(14); T(PI(g)); T(a != NULL); T(b != NULL); }
bool ftCo_80096CC8(Fighter_GObj* g, int l) { T(15); return 0; }
STUB_V(ft_80082B1C, 16)

int main(void)
{
    int op, k;
    tu_init();
    s_init();
    for (op = 0; op < 6; op++) {
        for (k = 0; k < 3000; k++) {
            u32 ret = 0;
            tu_case(op, k);
            s_fill();
            switch (op) {
            case 0: ret = ftCo_8009EF68(&GO); break;
            case 1: ftCo_8009EFA4(&GO); break;
            case 2: ftCo_StopCeil_Anim(&GO); break;
            case 3: ftCo_StopCeil_IASA(&GO); break;
            case 4: ftCo_StopCeil_Phys(&GO); break;
            case 5: ftCo_StopCeil_Coll(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
