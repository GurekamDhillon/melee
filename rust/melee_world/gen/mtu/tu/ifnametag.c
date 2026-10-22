/* TU: src/melee/if/ifnametag.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/if_drv.py. Each line: op case ret state-hash trace-hash. */
#include TU_SRC
#include "mtu_common.h"
static u8 SLOTB;
static u8 HEAPB[16];
static DynamicModelDesc* NMODELS[2];
static HSD_Archive* ARCP;

#include "ifnametag_gen.h"



static void setup_extra(void)
{
    int i, n;
    (void) i; (void) n;
int k;
    P_HSD_GObj[0].user_data = &SLOTB;
    for (k = 1; k < 4; k++) P_HSD_GObj[k].user_data = &P_Fighter[k & 1];
    for (k = 0; k < 2; k++) { P_Fighter[k].kind = (rnd() & 1) ? Ft_Kind_Mewtwo : (int) fv(); P_Fighter[k].motion_id = (rnd() & 1) ? 0xEC : (int) fv(); }
}

int main(void)
{
    int op, k;
    preg_state();
    freg_all();
    tu_case(99, 0); hash_init_state(); printf("%d %d %08X %08X %08X\n", 99, 0, 0u, H, TR);
    for (op = 0; op < 16; op++) {
        static const int NC[] = {400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400};
        for (k = 0; k < NC[op]; k++) {
            u32 ret = 0;
            tu_case(op, k);
            reset_state();
            setup_extra();
            switch (op) {
            case 0: { unsigned char a0; unsigned char a1; unsigned char a2; unsigned char a3;  a0 = (unsigned char) (rnd() % 6u); a1 = (unsigned char) ((0 + (int) (rnd() % 5u))); a2 = (unsigned char) (fv()); a3 = (unsigned char) (fv()); ret = B(un_802FC9B4(a0, a1, a2, a3)); } break;
            case 1: { HSD_GObj * a0; int a1;  a0 = (HSD_GObj *) (pick_HSD_GObj(1)); a1 = (int) (fv()); NameTag_RenderCallback(a0, a1); } break;
            case 2: { HSD_GObj * a0; int a1;  a0 = (HSD_GObj *) (pick_HSD_GObj(0)); a1 = (int) (fv()); fn_802FCAC4(a0, a1); } break;
            case 3: {    un_802FCBA0(); } break;
            case 4: { int a0;  a0 = (int) (rnd() % 6u); ret = (u32) has_nametag(a0); } break;
            case 5: { HSD_GObj * a0;  a0 = (HSD_GObj *) (&P_HSD_GObj[0]); fn_802FCC44(a0); } break;
            case 6: { float a0;  a0 = (float) (rw()); ret = B(inlineA1(a0)); } break;
            case 7: { int a0;  a0 = (int) (rnd() % 6u); NameTag_Create(a0); } break;
            case 8: { int a0;  a0 = (int) (rnd() % 6u); un_802FD28C(a0); } break;
            case 9: {    un_802FD404(); } break;
            case 10: {    un_802FD428(); } break;
            case 11: {    un_802FD450(); } break;
            case 12: {    un_802FD45C(); } break;
            case 13: {  { int q; for (q = 0; q < 6; q++) if (rnd() & 1) un_804A1EE0[q] = 0; }  un_802FD468(); } break;
            case 14: { int a0;  a0 = (int) (fv()); ret = PI((const void*) un_802FD4C8_inline(a0)); } break;
            case 15: {    un_802FD4C8(); } break;
            }
            hash_state();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
