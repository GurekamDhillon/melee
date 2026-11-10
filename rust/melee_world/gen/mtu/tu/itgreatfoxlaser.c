/* TU: src/melee/it/kinds/itgreatfoxlaser.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/if_drv.py. Each line: op case ret state-hash trace-hash. */
#include TU_SRC
#include "mtu_common.h"

#include "itgreatfoxlaser_gen.h"



static void set_uv(int op)
{
    UV_SpawnItem_x44_flag = 0; UV_SpawnItem_x45_flag = 0; UV_SpawnItem_x46_flag = 0; UV_SpawnItem_x47_flag = 0; UV_HSD_JObj_u = 0; UV_ECBSource_x4 = 0; UV_CommandInfo_anon_8 = 0; UV_CommandInfo_anon_1020_x8 = 0; UV_ColorOverlay_x28_colanim = 0; UV_HitCapsule_x134 = 0; UV_Item_xD90 = 0; UV_Item_xDAA = 0; UV_Item_xDAA_xDAA_flag = 0; UV_Item_xDBC = 0; UV_Item_xDCE_flag = 0; UV_Item_xDCF_flag = 0; UV_Item_xDD0_flag = 0; UV_Item_xDD1_flag = 0; UV_Item_xDD2_flag = 0; UV_Item_xDD3_flag = 0; UV_Item_xDD4_itemVar = 0;
    switch (op) {
    case 0: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_SpawnItem_x44_flag = 1; UV_SpawnItem_x45_flag = 1; UV_SpawnItem_x46_flag = 1; UV_SpawnItem_x47_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 29; break;
    case 1: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_SpawnItem_x44_flag = 1; UV_SpawnItem_x45_flag = 1; UV_SpawnItem_x46_flag = 1; UV_SpawnItem_x47_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 29; break;
    case 2: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_SpawnItem_x44_flag = 1; UV_SpawnItem_x45_flag = 1; UV_SpawnItem_x46_flag = 1; UV_SpawnItem_x47_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 29; break;
    case 3: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_SpawnItem_x44_flag = 1; UV_SpawnItem_x45_flag = 1; UV_SpawnItem_x46_flag = 1; UV_SpawnItem_x47_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 29; break;
    case 4: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_SpawnItem_x44_flag = 1; UV_SpawnItem_x45_flag = 1; UV_SpawnItem_x46_flag = 1; UV_SpawnItem_x47_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 29; break;
    case 5: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_SpawnItem_x44_flag = 1; UV_SpawnItem_x45_flag = 1; UV_SpawnItem_x46_flag = 1; UV_SpawnItem_x47_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 29; break;
    case 6: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_SpawnItem_x44_flag = 1; UV_SpawnItem_x45_flag = 1; UV_SpawnItem_x46_flag = 1; UV_SpawnItem_x47_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 29; break;
    case 7: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_SpawnItem_x44_flag = 1; UV_SpawnItem_x45_flag = 1; UV_SpawnItem_x46_flag = 1; UV_SpawnItem_x47_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 29; break;
    case 8: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_SpawnItem_x44_flag = 1; UV_SpawnItem_x45_flag = 1; UV_SpawnItem_x46_flag = 1; UV_SpawnItem_x47_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 29; break;
    case 9: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_SpawnItem_x44_flag = 1; UV_SpawnItem_x45_flag = 1; UV_SpawnItem_x46_flag = 1; UV_SpawnItem_x47_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 29; break;
    default: break;
    }
}

static void setup_extra(void)
{
    int i, n;
    (void) i; (void) n;
    for (n = 0; n < 6; n++) P_HSD_JObj[n].flags &= ~(1u << 17);
    for (n = 0; n < 8; n++) P_DynamicBoneTable[0].bones[n] = &P_HSD_JObj[rnd() % 6u];
}

int main(void)
{
    int op, k;
    preg_state();
    freg_all();
    for (op = 0; op < 10; op++) {
        static const int NC[] = {400, 400, 400, 400, 400, 400, 400, 400, 400, 400};
        for (k = 0; k < NC[op]; k++) {
            u32 ret = 0;
            tu_case(op, k);
            set_uv(op);
            reset_state();
            setup_extra();
            switch (op) {
            case 0: { __typeof__(HSD_GObj *) a0; __typeof__(Vec3 *) a1; __typeof__(int) a2;  a0 = (HSD_GObj *) (pick_HSD_GObj(0)); a1 = (Vec3 *) (pick_Vec3(0)); a2 = (int) ((0 + (int) (rnd() % 2u))); ret = PI((const void*) it_802EAF34(a0, a1, a2)); } break;
            case 1: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); it_802EB1EC(a0); } break;
            case 2: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); it_802EB268(a0); } break;
            case 3: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); ret = (u32) itGreatfoxlaser_UnkMotion1_Anim(a0); } break;
            case 4: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); itGreatfoxlaser_UnkMotion1_Phys(a0); } break;
            case 5: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); ret = (u32) itGreatFoxLaser_Logic27_DmgDealt(a0); } break;
            case 6: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); ret = (u32) itGreatFoxLaser_Logic27_Clanked(a0); } break;
            case 7: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); ret = (u32) itGreatFoxLaser_Logic27_Absorbed(a0); } break;
            case 8: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); ret = (u32) it_2725_Logic27_Reflected(a0); } break;
            case 9: { __typeof__(Item_GObj *) a0; __typeof__(Item_GObj *) a1;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); a1 = (Item_GObj *) (pick_HSD_GObj(0)); it_802EB5A8(a0, a1); } break;
            }
            hash_state();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
