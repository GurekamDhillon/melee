/* TU: src/melee/it/kinds/itharisen.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/if_drv.py. Each line: op case ret state-hash trace-hash. */
#include TU_SRC
#include "mtu_common.h"

#include "itharisen_gen.h"



static void set_uv(int op)
{
    UV_HSD_JObj_u = 0; UV_ECBSource_x4 = 0; UV_CommandInfo_anon_8 = 0; UV_CommandInfo_anon_1020_x8 = 0; UV_ColorOverlay_x28_colanim = 0; UV_HitCapsule_x134 = 0; UV_Item_xD90 = 0; UV_Item_xDAA = 0; UV_Item_xDAA_xDAA_flag = 0; UV_Item_xDBC = 0; UV_Item_xDCE_flag = 0; UV_Item_xDCF_flag = 0; UV_Item_xDD0_flag = 0; UV_Item_xDD1_flag = 0; UV_Item_xDD2_flag = 0; UV_Item_xDD3_flag = 0; UV_Item_xDD4_itemVar = 0;
    switch (op) {
    case 0: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_xD90 = 0; break;
    case 1: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_xD90 = 0; break;
    case 2: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_xD90 = 0; break;
    case 3: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_xD90 = 0; break;
    case 4: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_xD90 = 0; break;
    case 5: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_xD90 = 0; break;
    case 6: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_xD90 = 0; break;
    case 7: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_xD90 = 0; break;
    case 8: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_xD90 = 0; break;
    case 9: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_xD90 = 0; break;
    case 10: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_xD90 = 0; break;
    case 11: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_xD90 = 0; break;
    case 12: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_xD90 = 0; break;
    case 13: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_xD90 = 0; break;
    case 14: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_xD90 = 0; break;
    case 15: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_xD90 = 0; break;
    case 16: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_xD90 = 0; break;
    case 17: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_xD90 = 0; break;
    case 18: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_xD90 = 0; break;
    case 19: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_xD90 = 0; break;
    case 20: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_xD90 = 0; break;
    case 21: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_xD90 = 0; break;
    case 22: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_xD90 = 0; break;
    case 23: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_xD90 = 0; break;
    case 24: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_xD90 = 0; break;
    case 25: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_xD90 = 0; break;
    case 26: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_xD90 = 0; break;
    case 27: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_xD90 = 0; break;
    case 28: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_xD90 = 0; break;
    case 29: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_xD90 = 0; break;
    case 30: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_xD90 = 0; break;
    case 31: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_xD90 = 0; break;
    case 32: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_xD90 = 0; break;
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
    for (op = 0; op < 33; op++) {
        static const int NC[] = {400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400};
        for (k = 0; k < NC[op]; k++) {
            u32 ret = 0;
            tu_case(op, k);
            set_uv(op);
            reset_state();
            setup_extra();
            switch (op) {
            case 0: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); it_802927E8(a0); } break;
            case 1: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); it_8029282C(a0); } break;
            case 2: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); itHarisen_Logic24_Spawned(a0); } break;
            case 3: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); it_8029287C(a0); } break;
            case 4: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); ret = (u32) itHarisen_UnkMotion0_Anim(a0); } break;
            case 5: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); itHarisen_UnkMotion0_Phys(a0); } break;
            case 6: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); ret = (u32) itHarisen_UnkMotion0_Coll(a0); } break;
            case 7: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); it_8029290C(a0); } break;
            case 8: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); ret = (u32) itHarisen_UnkMotion8_Anim(a0); } break;
            case 9: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); itHarisen_UnkMotion1_Phys(a0); } break;
            case 10: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); ret = (u32) itHarisen_UnkMotion1_Coll(a0); } break;
            case 11: { __typeof__(Item_GObj *) a0; __typeof__(f32) a1;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); a1 = (f32) (rg()); it_80292998(a0, a1); } break;
            case 12: { __typeof__(Item_GObj *) a0; __typeof__(f32) a1;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); a1 = (f32) (rg()); it_802929C8(a0, a1); } break;
            case 13: { __typeof__(Item_GObj *) a0; __typeof__(f32) a1;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); a1 = (f32) (rg()); it_802929F8(a0, a1); } break;
            case 14: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); it_80292A28(a0); } break;
            case 15: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); itHarisen_Logic24_PickedUp(a0); } break;
            case 16: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); ret = (u32) itHarisen_UnkMotion6_Anim(a0); } break;
            case 17: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); itHarisen_UnkMotion6_Phys(a0); } break;
            case 18: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); itHarisen_Logic24_Dropped(a0); } break;
            case 19: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); ret = (u32) itHarisen_UnkMotion8_Coll(a0); } break;
            case 20: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); itHarisen_Logic24_Thrown(a0); } break;
            case 21: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); itHarisen_UnkMotion8_Phys(a0); } break;
            case 22: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); ret = (u32) itHarisen_UnkMotion7_Coll(a0); } break;
            case 23: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); ret = (u32) itHarisen_Logic24_DmgDealt(a0); } break;
            case 24: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); itHarisen_Logic24_EnteredAir(a0); } break;
            case 25: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); ret = (u32) itHarisen_UnkMotion9_Anim(a0); } break;
            case 26: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); itHarisen_UnkMotion9_Phys(a0); } break;
            case 27: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); ret = (u32) itHarisen_UnkMotion9_Coll(a0); } break;
            case 28: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); ret = (u32) itHarisen_Logic24_Clanked(a0); } break;
            case 29: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); ret = (u32) itHarisen_Logic24_Reflected(a0); } break;
            case 30: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); ret = (u32) itHarisen_Logic24_HitShield(a0); } break;
            case 31: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); ret = (u32) itHarisen_Logic24_ShieldBounced(a0); } break;
            case 32: { __typeof__(Item_GObj *) a0; __typeof__(Item_GObj *) a1;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); a1 = (Item_GObj *) (pick_HSD_GObj(0)); itHarisen_Logic24_EvtUnk(a0, a1); } break;
            }
            hash_state();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
