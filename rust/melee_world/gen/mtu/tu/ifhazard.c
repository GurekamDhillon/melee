/* TU: src/melee/if/ifhazard.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/if_drv.py. Each line: op case ret state-hash trace-hash. */
#include TU_SRC
#include "mtu_common.h"

#include "ifhazard_gen.h"



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
    for (op = 0; op < 12; op++) {
        static const int NC[] = {400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400};
        for (k = 0; k < NC[op]; k++) {
            u32 ret = 0;
            tu_case(op, k);
            reset_state();
            setup_extra();
            switch (op) {
            case 0: { int a0;  a0 = (int) (fv()); un_802FD604(a0); } break;
            case 1: {    un_802FD65C(); } break;
            case 2: {    un_802FD668(); } break;
            case 3: {    un_802FD674(); } break;
            case 4: { HSD_GObj * a0;  a0 = (HSD_GObj *) (pick_HSD_GObj(0)); fn_802FD680(a0); } break;
            case 5: { HSD_GObj * a0; int a1;  a0 = (HSD_GObj *) (pick_HSD_GObj(0)); a1 = (int) (fv()); fn_802FD6CC(a0, a1); } break;
            case 6: {    un_802FD704(); } break;
            case 7: { DynamicModelDesc * a0; int a1;  a0 = (DynamicModelDesc *) (pick_DynamicModelDesc(0)); a1 = (int) (fv()); un_802FD708(a0, a1); } break;
            case 8: { DynamicModelDesc * a0;  a0 = (DynamicModelDesc *) (pick_DynamicModelDesc(0)); un_802FD8A0(a0); } break;
            case 9: { DynamicModelDesc * a0;  a0 = (DynamicModelDesc *) (pick_DynamicModelDesc(0)); un_802FD8C4(a0); } break;
            case 10: { DynamicModelDesc * a0;  a0 = (DynamicModelDesc *) (pick_DynamicModelDesc(0)); un_802FD8E8(a0); } break;
            case 11: {    un_802FD90C(); } break;
            }
            hash_state();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
