/* TU: src/melee/it/kinds/ithassam.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/if_drv.py. Each line: op case ret state-hash trace-hash. */
#include TU_SRC
#include "mtu_common.h"
static HSD_GObj FG[3];
static Fighter FT[3];
static HSD_GObj* PLH[32];

#include "ithassam_gen.h"



static void set_uv(int op)
{
    UV_HSD_JObj_u = 0; UV_ECBSource_anon_4 = 0; UV_CommandInfo_anon_8 = 0; UV_ColorOverlay_x28_colanim = 0; UV_HitCapsule_anon_134 = 0; UV_Item_xD90 = 0; UV_Item_anon_daa = 0; UV_Item_anon_477_xDAA_flag = 0; UV_Item_anon_dbc = 0; UV_Item_xDCE_flag = 0; UV_Item_xDCF_flag = 0; UV_Item_xDD0_flag = 0; UV_Item_xDD1_flag = 0; UV_Item_xDD2_flag = 0; UV_Item_xDD3_flag = 0; UV_Item_xDD4_itemVar = 0;
    switch (op) {
    case 0: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 33; break;
    case 1: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 33; break;
    case 2: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 33; break;
    case 3: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 33; break;
    case 4: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 33; break;
    case 5: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 33; break;
    case 6: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 33; break;
    case 7: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 33; break;
    case 8: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 33; break;
    case 9: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 33; break;
    case 10: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 33; break;
    case 11: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 33; break;
    case 12: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 33; break;
    case 13: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 33; break;
    case 14: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 33; break;
    case 15: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 33; break;
    case 16: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 33; break;
    case 17: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 33; break;
    case 18: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 33; break;
    case 19: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 33; break;
    case 20: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 33; break;
    case 21: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 33; break;
    case 22: UV_Item_xDCE_flag = 1; UV_Item_xDCF_flag = 1; UV_Item_xDD0_flag = 1; UV_Item_xDD1_flag = 1; UV_Item_xDD2_flag = 1; UV_Item_xDD3_flag = 1; UV_Item_anon_477_xDAA_flag = 1; UV_Item_xD90 = 0; UV_Item_xDD4_itemVar = 33; break;
    default: break;
    }
}

static void setup_extra(void)
{
    int i, n;
    (void) i; (void) n;
    for (n = 0; n < 6; n++) P_HSD_JObj[n].flags &= ~(1u << 17);
    for (n = 0; n < 8; n++) P_DynamicBoneTable[0].bones[n] = &P_HSD_JObj[rnd() % 6u];
    for (n = 0; n < 3; n++) { FG[n].next = (n < 2) ? &FG[n + 1] : 0; FG[n].user_data = &FT[n]; FT[n].x221F_b3 = rnd() & 1; FT[n].team = rnd() % 3u; }
    PLH[HSD_GOBJ_PLINK_FIGHTER] = &FG[0]; HSD_GObjPLinkHead = PLH;
}

int main(void)
{
    int op, k;
    preg_state();
    freg_all();
    for (op = 0; op < 23; op++) {
        static const int NC[] = {400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400};
        for (k = 0; k < NC[op]; k++) {
            u32 ret = 0;
            tu_case(op, k);
            set_uv(op);
            reset_state();
            setup_extra();
            switch (op) {
            case 0: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); itHassam_802CDBE0(a0); } break;
            case 1: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); itHassam_802CDC5C(a0); } break;
            case 2: { __typeof__(Item_GObj *) a0; __typeof__(Item_GObj *) a1;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); a1 = (Item_GObj *) (pick_HSD_GObj(0)); itHassam_802CDC60(a0, a1); } break;
            case 3: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); itHassam_802CDC80(a0); } break;
            case 4: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); itHassam_802CDCB4(a0); } break;
            case 5: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); ret = (u32) itHassam_UnkMotion0_Anim(a0); } break;
            case 6: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); itHassam_UnkMotion0_Phys(a0); } break;
            case 7: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); ret = (u32) itHassam_UnkMotion0_Coll(a0); } break;
            case 8: { __typeof__(Vec3 *) a0; __typeof__(Item_GObj *) a1;  a0 = (Vec3 *) (pick_Vec3(0)); a1 = (Item_GObj *) (((rnd() & 3) == 0 ? (HSD_GObj*) 0 : &FG[rnd() % 3u])); ret = PI((const void*) itHassam_802CDE1C(a0, a1)); } break;
            case 9: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); itHassam_802CDF28(a0); } break;
            case 10: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); itHassam_802CE008(a0); } break;
            case 11: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); ret = (u32) itHassam_UnkMotion1_Anim(a0); } break;
            case 12: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); itHassam_UnkMotion1_Phys(a0); } break;
            case 13: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); ret = (u32) itHassam_UnkMotion1_Coll(a0); } break;
            case 14: { __typeof__(HSD_GObj *) a0; __typeof__(int) a1; __typeof__(Item_StateChangeFlags) a2;  a0 = (HSD_GObj *) (pick_HSD_GObj(0)); a1 = (int) (fv()); a2 = (Item_StateChangeFlags) (fv()); itHassam_802CE400_sub(a0, a1, a2); } break;
            case 15: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); itHassam_802CE400(a0); } break;
            case 16: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); ret = (u32) itHassam_UnkMotion2_Anim(a0); } break;
            case 17: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); itHassam_UnkMotion2_Phys(a0); } break;
            case 18: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); ret = (u32) itHassam_UnkMotion2_Coll(a0); } break;
            case 19: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); it_802CE640(a0); } break;
            case 20: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); ret = (u32) itHassam_UnkMotion3_Anim(a0); } break;
            case 21: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); itHassam_UnkMotion3_Phys(a0); } break;
            case 22: { __typeof__(Item_GObj *) a0;  a0 = (Item_GObj *) (pick_HSD_GObj(0)); ret = (u32) itHassam_UnkMotion3_Coll(a0); } break;
            }
            hash_state();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
