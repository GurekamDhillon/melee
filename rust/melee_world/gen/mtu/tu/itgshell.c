/* TU: src/melee/it/kinds/itgshell.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/if_drv.py. Each line: op case ret state-hash trace-hash. */
#include TU_SRC
#include "mtu_common.h"

#include "itgshell_gen.h"



static void set_uv(int op)
{
    UV_HSD_JObj_u = 0; UV_ECBSource_anon_4 = 0; UV_CommandInfo_anon_8 = 0; UV_ColorOverlay_x28_colanim = 0; UV_HitCapsule_anon_134 = 0; UV_Item_xD90 = 0; UV_Item_anon_daa = 0; UV_Item_anon_477_xDAA_flag = 0; UV_Item_anon_dbc = 0; UV_Item_xDCE_flag = 0; UV_Item_xDCF_flag = 0; UV_Item_xDD0_flag = 0; UV_Item_xDD1_flag = 0; UV_Item_xDD2_flag = 0; UV_Item_xDD3_flag = 0; UV_Item_xDD4_itemVar = 0;
    switch (op) {
    case 0: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 30; break;
    case 1: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 30; break;
    case 2: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 30; break;
    case 3: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 30; break;
    case 4: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 30; break;
    case 5: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 30; break;
    case 6: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 30; break;
    case 7: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 30; break;
    case 8: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 30; break;
    case 9: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 30; break;
    case 10: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 30; break;
    case 11: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 30; break;
    case 12: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 30; break;
    case 13: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 30; break;
    case 14: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 30; break;
    case 15: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 30; break;
    case 16: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 30; break;
    case 17: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 30; break;
    case 18: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 30; break;
    case 19: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 30; break;
    case 20: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 30; break;
    case 21: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 30; break;
    case 22: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 30; break;
    case 23: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 30; break;
    case 24: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 30; break;
    case 25: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 30; break;
    case 26: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 30; break;
    case 27: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 30; break;
    case 28: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 30; break;
    case 29: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 30; break;
    case 30: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 30; break;
    case 31: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 30; break;
    case 32: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 30; break;
    case 33: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 30; break;
    case 34: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 30; break;
    case 35: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 30; break;
    case 36: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 30; break;
    case 37: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 30; break;
    case 38: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 30; break;
    case 39: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 30; break;
    case 40: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 30; break;
    case 41: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 30; break;
    case 42: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 30; break;
    case 43: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 30; break;
    case 44: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 30; break;
    case 45: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 30; break;
    default: break;
    }
}

static void setup_extra(void)
{
    int i, n;
    (void) i; (void) n;
    for (n = 0; n < 6; n++) P_HSD_JObj[n].flags &= ~(1u << 17);
}

int main(void)
{
    int op, k;
    preg_state();
    freg_all();
    for (op = 0; op < 46; op++) {
        static const int NC[] = {400, 400, 400, 3000, 3000, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 3000, 400, 400, 400, 400, 400, 400, 400};
        for (k = 0; k < NC[op]; k++) {
            u32 ret = 0;
            tu_case(op, k);
            set_uv(op);
            reset_state();
            setup_extra();
            switch (op) {
            case 0: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); it_8028B8D8(a0); } break;
            case 1: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); it_8028B988(a0); } break;
            case 2: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); it_8028BA2C(a0); } break;
            case 3: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); it_8028BAD8(a0); } break;
            case 4: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); it_8028BC2C(a0); } break;
            case 5: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); itGShell_Logic14_Spawned(a0); } break;
            case 6: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); it_8028BE54(a0); } break;
            case 7: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); ret = (u32) itGshell_UnkMotion0_Anim(a0); } break;
            case 8: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); itGshell_UnkMotion0_Phys(a0); } break;
            case 9: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); ret = (u32) itGshell_UnkMotion0_Coll(a0); } break;
            case 10: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); it_8028C018(a0); } break;
            case 11: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); ret = (u32) itGshell_UnkMotion1_Anim(a0); } break;
            case 12: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); itGshell_UnkMotion1_Phys(a0); } break;
            case 13: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); ret = (u32) itGshell_UnkMotion1_Coll(a0); } break;
            case 14: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); itGShell_Logic14_PickedUp(a0); } break;
            case 15: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); ret = (u32) itGshell_UnkMotion2_Anim(a0); } break;
            case 16: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); itGshell_UnkMotion2_Phys(a0); } break;
            case 17: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); itGShell_Logic14_Thrown(a0); } break;
            case 18: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); ret = (u32) itGshell_UnkMotion3_Anim(a0); } break;
            case 19: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); itGshell_UnkMotion3_Phys(a0); } break;
            case 20: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); ret = (u32) itGshell_UnkMotion3_Coll(a0); } break;
            case 21: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); itGShell_Logic14_Dropped(a0); } break;
            case 22: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); ret = (u32) itGshell_UnkMotion4_Anim(a0); } break;
            case 23: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); itGshell_UnkMotion4_Phys(a0); } break;
            case 24: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); ret = (u32) itGshell_UnkMotion4_Coll(a0); } break;
            case 25: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); it_8028C3A8(a0); } break;
            case 26: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); ret = (u32) itGshell_UnkMotion6_Anim(a0); } break;
            case 27: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); itGshell_UnkMotion6_Phys(a0); } break;
            case 28: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); ret = (u32) itGshell_UnkMotion6_Coll(a0); } break;
            case 29: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); it_8028C898(a0); } break;
            case 30: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); ret = (u32) itGshell_UnkMotion8_Anim(a0); } break;
            case 31: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); itGshell_UnkMotion8_Phys(a0); } break;
            case 32: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); ret = (u32) itGshell_UnkMotion8_Coll(a0); } break;
            case 33: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); itGShell_Logic14_EnteredAir(a0); } break;
            case 34: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); ret = (u32) itGshell_UnkMotion9_Anim(a0); } break;
            case 35: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); itGshell_UnkMotion9_Phys(a0); } break;
            case 36: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); ret = (u32) itGshell_UnkMotion9_Coll(a0); } break;
            case 37: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); ret = (u32) itGShell_Logic14_DmgDealt(a0); } break;
            case 38: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); ret = (u32) itGShell_Logic14_DmgReceived(a0); } break;
            case 39: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); ret = (u32) itGShell_Logic14_Reflected(a0); } break;
            case 40: { __typeof__(HSD_GObj *) a0;  a0 = (HSD_GObj *) (pick_HSD_GObj(0)); shellHit(a0); } break;
            case 41: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); ret = (u32) itGShell_Logic14_Clanked(a0); } break;
            case 42: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); ret = (u32) itGShell_Logic14_HitShield(a0); } break;
            case 43: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); ret = (u32) itGShell_Logic14_ShieldBounced(a0); } break;
            case 44: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); ret = (u32) it_8028CF68(a0); } break;
            case 45: { __typeof__(Item_GObj *) a0; __typeof__(Item_GObj *) a1;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); a1 = (Item_GObj *) (pick_HSD_GObj(0)); itGShell_Logic14_EvtUnk(a0, a1); } break;
            }
            hash_state();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
