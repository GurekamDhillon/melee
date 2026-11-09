/* TU: src/melee/it/kinds/itheart.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/if_drv.py. Each line: op case ret state-hash trace-hash. */
#include TU_SRC
#include "mtu_common.h"

#include "itheart_gen.h"



static void set_uv(int op)
{
    UV_SpawnItem_x44_flag = 0; UV_SpawnItem_x45_flag = 0; UV_SpawnItem_x46_flag = 0; UV_SpawnItem_x47_flag = 0; UV_HSD_JObj_u = 0; UV_ECBSource_x4 = 0; UV_CommandInfo_anon_8 = 0; UV_CommandInfo_anon_1020_x8 = 0; UV_ColorOverlay_x28_colanim = 0; UV_HitCapsule_x134 = 0; UV_Item_xD90 = 0; UV_Item_xDAA = 0; UV_Item_xDAA_xDAA_flag = 0; UV_Item_xDBC = 0; UV_Item_xDCE_flag = 0; UV_Item_xDCF_flag = 0; UV_Item_xDD0_flag = 0; UV_Item_xDD1_flag = 0; UV_Item_xDD2_flag = 0; UV_Item_xDD3_flag = 0; UV_Item_xDD4_itemVar = 0; UV_itHeart_ItemVars_xDD8 = 0; UV_HeartContainerAttr_x4 = 0;
    switch (op) {
    case 0: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_SpawnItem_x44_flag = 1; UV_SpawnItem_x45_flag = 1; UV_SpawnItem_x46_flag = 1; UV_SpawnItem_x47_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 34; UV_HeartContainerAttr_x4 = 1; break;
    case 1: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_SpawnItem_x44_flag = 1; UV_SpawnItem_x45_flag = 1; UV_SpawnItem_x46_flag = 1; UV_SpawnItem_x47_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 34; UV_HeartContainerAttr_x4 = 1; break;
    case 2: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_SpawnItem_x44_flag = 1; UV_SpawnItem_x45_flag = 1; UV_SpawnItem_x46_flag = 1; UV_SpawnItem_x47_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 34; UV_HeartContainerAttr_x4 = 1; break;
    case 3: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_SpawnItem_x44_flag = 1; UV_SpawnItem_x45_flag = 1; UV_SpawnItem_x46_flag = 1; UV_SpawnItem_x47_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 34; UV_HeartContainerAttr_x4 = 1; break;
    case 4: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_SpawnItem_x44_flag = 1; UV_SpawnItem_x45_flag = 1; UV_SpawnItem_x46_flag = 1; UV_SpawnItem_x47_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 34; UV_HeartContainerAttr_x4 = 1; break;
    case 5: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_SpawnItem_x44_flag = 1; UV_SpawnItem_x45_flag = 1; UV_SpawnItem_x46_flag = 1; UV_SpawnItem_x47_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 34; UV_HeartContainerAttr_x4 = 1; break;
    case 6: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_SpawnItem_x44_flag = 1; UV_SpawnItem_x45_flag = 1; UV_SpawnItem_x46_flag = 1; UV_SpawnItem_x47_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 34; UV_HeartContainerAttr_x4 = 1; break;
    case 7: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_SpawnItem_x44_flag = 1; UV_SpawnItem_x45_flag = 1; UV_SpawnItem_x46_flag = 1; UV_SpawnItem_x47_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 34; UV_HeartContainerAttr_x4 = 1; break;
    case 8: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_SpawnItem_x44_flag = 1; UV_SpawnItem_x45_flag = 1; UV_SpawnItem_x46_flag = 1; UV_SpawnItem_x47_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 34; UV_HeartContainerAttr_x4 = 1; break;
    case 9: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_SpawnItem_x44_flag = 1; UV_SpawnItem_x45_flag = 1; UV_SpawnItem_x46_flag = 1; UV_SpawnItem_x47_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 34; UV_HeartContainerAttr_x4 = 1; break;
    case 10: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_SpawnItem_x44_flag = 1; UV_SpawnItem_x45_flag = 1; UV_SpawnItem_x46_flag = 1; UV_SpawnItem_x47_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 34; UV_HeartContainerAttr_x4 = 1; break;
    case 11: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_SpawnItem_x44_flag = 1; UV_SpawnItem_x45_flag = 1; UV_SpawnItem_x46_flag = 1; UV_SpawnItem_x47_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 34; UV_HeartContainerAttr_x4 = 1; break;
    case 12: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_SpawnItem_x44_flag = 1; UV_SpawnItem_x45_flag = 1; UV_SpawnItem_x46_flag = 1; UV_SpawnItem_x47_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 34; UV_HeartContainerAttr_x4 = 1; break;
    case 13: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_SpawnItem_x44_flag = 1; UV_SpawnItem_x45_flag = 1; UV_SpawnItem_x46_flag = 1; UV_SpawnItem_x47_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 34; UV_HeartContainerAttr_x4 = 1; break;
    case 14: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_SpawnItem_x44_flag = 1; UV_SpawnItem_x45_flag = 1; UV_SpawnItem_x46_flag = 1; UV_SpawnItem_x47_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 34; UV_HeartContainerAttr_x4 = 1; break;
    case 15: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_SpawnItem_x44_flag = 1; UV_SpawnItem_x45_flag = 1; UV_SpawnItem_x46_flag = 1; UV_SpawnItem_x47_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 34; UV_HeartContainerAttr_x4 = 1; break;
    case 16: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_SpawnItem_x44_flag = 1; UV_SpawnItem_x45_flag = 1; UV_SpawnItem_x46_flag = 1; UV_SpawnItem_x47_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 34; UV_HeartContainerAttr_x4 = 1; break;
    case 17: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_SpawnItem_x44_flag = 1; UV_SpawnItem_x45_flag = 1; UV_SpawnItem_x46_flag = 1; UV_SpawnItem_x47_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 34; UV_HeartContainerAttr_x4 = 1; break;
    case 18: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_SpawnItem_x44_flag = 1; UV_SpawnItem_x45_flag = 1; UV_SpawnItem_x46_flag = 1; UV_SpawnItem_x47_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 34; UV_HeartContainerAttr_x4 = 1; break;
    case 19: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_SpawnItem_x44_flag = 1; UV_SpawnItem_x45_flag = 1; UV_SpawnItem_x46_flag = 1; UV_SpawnItem_x47_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 34; UV_HeartContainerAttr_x4 = 1; break;
    case 20: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_SpawnItem_x44_flag = 1; UV_SpawnItem_x45_flag = 1; UV_SpawnItem_x46_flag = 1; UV_SpawnItem_x47_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 34; UV_HeartContainerAttr_x4 = 1; break;
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
    for (op = 0; op < 21; op++) {
        static const int NC[] = {400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400};
        for (k = 0; k < NC[op]; k++) {
            u32 ret = 0;
            tu_case(op, k);
            set_uv(op);
            reset_state();
            setup_extra();
            switch (op) {
            case 0: { __typeof__(Item_GObj *) a0; __typeof__(Vec3 *) a1; __typeof__(s32) a2;  a0 = (Item_GObj *) (pick_HSD_GObj(1)); a1 = (Vec3 *) (pick_Vec3(0)); a2 = (s32) (fv()); ret = PI((const void*) it_80283AE4(a0, a1, a2)); } break;
            case 1: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); it_80283BD4(a0); } break;
            case 2: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); itHeart_Logic8_Spawned(a0); } break;
            case 3: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); itHeart_Logic8_Destroyed(a0); } break;
            case 4: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); it_80283C7C(a0); } break;
            case 5: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); ret = (u32) itHeart_UnkMotion0_Anim(a0); } break;
            case 6: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); itHeart_UnkMotion0_Phys(a0); } break;
            case 7: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); ret = (u32) itHeart_UnkMotion0_Coll(a0); } break;
            case 8: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); it_80283DD4(a0); } break;
            case 9: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); ret = (u32) itHeart_UnkMotion3_Anim(a0); } break;
            case 10: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); itHeart_UnkMotion3_Phys(a0); } break;
            case 11: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); ret = (u32) itHeart_UnkMotion3_Coll(a0); } break;
            case 12: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); itHeart_Logic8_PickedUp(a0); } break;
            case 13: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); ret = (u32) itHeart_UnkMotion2_Anim(a0); } break;
            case 14: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); itHeart_UnkMotion2_Phys(a0); } break;
            case 15: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); itHeart_Logic8_Dropped(a0); } break;
            case 16: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); itHeart_Logic8_EnteredAir(a0); } break;
            case 17: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); ret = (u32) itHeart_UnkMotion4_Anim(a0); } break;
            case 18: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); itHeart_UnkMotion4_Phys(a0); } break;
            case 19: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); ret = (u32) itHeart_UnkMotion4_Coll(a0); } break;
            case 20: { __typeof__(Item_GObj *) a0; __typeof__(Item_GObj *) a1;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); a1 = (Item_GObj *) (pick_HSD_GObj(0)); itHeart_Logic8_EvtUnk(a0, a1); } break;
            }
            hash_state();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
