/* TU: src/melee/if/ifmagnify.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/if_drv.py. Each line: op case ret state-hash trace-hash. */
#include TU_SRC
#include "mtu_common.h"
static GXColor GCOL[9];
static float STG[4];
static Vec2 VEC2A[2];
static Vec3 V3A;
static int NULLRET;
static DynamicModelDesc* PMDL[2];
static u8 HEAPB[16];
static HSD_Archive* ARCP;
static GXColor rcol(void) { GXColor c; c.r = (u8) fv(); c.g = (u8) fv(); c.b = (u8) fv(); c.a = (u8) fv(); return c; }
static int view_bits(void) { static const u32 k[3] = { 0x3FAAAAABu, 0x3FE38E39u, 0x40000000u }; u32 q = fv() & 3; return (int) (q < 3 ? k[q] : fv()); }

#include "ifmagnify_gen.h"



static void setup_extra(void)
{
    int i, n;
    (void) i; (void) n;

}

int main(void)
{
    int op, k;
    preg_state();
    freg_all();
    tu_case(99, 0); hash_init_state(); printf("%d %d %08X %08X %08X\n", 99, 0, 0u, H, TR);
    for (op = 0; op < 20; op++) {
        static const int NC[] = {400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400};
        for (k = 0; k < NC[op]; k++) {
            u32 ret = 0;
            tu_case(op, k);
            reset_state();
            setup_extra();
            switch (op) {
            case 0: {    ret = B(ifMagnify_ViewAspect()); } break;
            case 1: { HSD_GObj * a0;  a0 = (HSD_GObj *) (pick_HSD_GObj(0)); ret = (u32) ifMagnify_VisualOffscreen(a0); } break;
            case 2: {    ret = (u32) ifMagnify_IsHUDVisible(); } break;
            case 3: { GXColor * a0; s32 a1;  a0 = (GXColor *) (&GCOL[0]); a1 = (s32) (rnd() % 6u); ifMagnify_GetPlayerColor(a0, a1); } break;
            case 4: { s32 a0;  a0 = (s32) (rnd() % 6u); ret = (u32) ifMagnify_802FB6E8(a0); } break;
            case 5: { ifMagnifyPlayer * a0; Vec2 * a1; Vec2 * a2;  a0 = (ifMagnifyPlayer *) (&ifMagnify_804A1DE0.player[0]); a1 = (Vec2 *) (&VEC2A[0]); a2 = (Vec2 *) (&VEC2A[1]); ret = PI((const void*) ifMagnify_802FB73C(a0, a1, a2)); } break;
            case 6: { HSD_GObj * a0; int a1; P_HSD_GObj[0].user_data = &ifMagnify_804A1DE0.player[rnd() % 6u]; a0 = (HSD_GObj *) (&P_HSD_GObj[0]); a1 = (int) ((0 + (int) (rnd() % 3u))); ifMagnify_802FB8C0(a0, a1); } break;
            case 7: { GXColor * a0; Vec3 * a1;  a0 = (GXColor *) (&GCOL[0]); a1 = (Vec3 *) (&V3A); ifMagnify_GetCornerColors(a0, a1); } break;
            case 8: { HSD_GObj * a0;  a0 = (HSD_GObj *) (pick_HSD_GObj(0)); ifMagnify_802FBBDC(a0); } break;
            case 9: {    ifMagnify_802FC3BC(); } break;
            case 10: { s32 a0;  a0 = (s32) (rnd() % 6u); ifMagnify_802FC3C0(a0); } break;
            case 11: {    ifMagnify_802FC618(); } break;
            case 12: {    ifMagnify_802FC750(); } break;
            case 13: { ifMagnify * a0; NULLRET = 1; a0 = (ifMagnify *) (&ifMagnify_804A1DE0); ifMagnify_802FC7C0(a0); } break;
            case 14: {    ifMagnify_802FC870(); } break;
            case 15: {    ifMagnify_802FC8E8(); } break;
            case 16: {    ifMagnify_802FC940(); } break;
            case 17: {    ifMagnify_LogicDisarm(); } break;
            case 18: {    ifMagnify_UpdateLogicOffscreen(); } break;
            case 19: { s32 a0;  a0 = (s32) (rnd() % 6u); ret = (u32) ifMagnify_802FC998(a0); } break;
            }
            hash_state();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
