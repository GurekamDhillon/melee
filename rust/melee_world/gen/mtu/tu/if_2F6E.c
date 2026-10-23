/* TU: src/melee/if/if_2F6E.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/if_drv.py. Each line: op case ret state-hash trace-hash. */
#include TU_SRC
#include "mtu_common.h"
static DynamicModelDesc* MODELS[8];
static HSD_Archive* ARCP;
static void cbA(s32 a) { T(7001); T((u32) a); }
static void cbB(s32 a) { T(7002); T((u32) a); }
static void* cbsel(void) { u32 q = rnd() % 3; return q == 0 ? (void*) 0 : q == 1 ? (void*) cbA : (void*) cbB; }

#include "if_2F6E_gen.h"



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
    freg((const void*) cbA);
    freg((const void*) cbB);
    tu_case(99, 0); hash_init_state(); printf("%d %d %08X %08X %08X\n", 99, 0, 0u, H, TR);
    for (op = 0; op < 4; op++) {
        static const int NC[] = {400, 400, 400, 400};
        for (k = 0; k < NC[op]; k++) {
            u32 ret = 0;
            tu_case(op, k);
            reset_state();
            setup_extra();
            switch (op) {
            case 0: { int a0; int a1; int a2; int a3; Event a4; Event a5;  a0 = (int) ((0 + (int) (rnd() % 9u))); a1 = (int) (fv()); a2 = (int) (fv()); a3 = (int) (fv()); a4 = (Event) ((Event) cbsel()); a5 = (Event) ((Event) cbsel()); ifStatus_802F6EA4(a0, a1, a2, a3, a4, a5); } break;
            case 1: { void * a0;  a0 = (void *) (cbsel()); ifStatus_802F7034(a0); } break;
            case 2: {    ifStatus_802F7134(); } break;
            case 3: {    ifStatus_802F7220(); } break;
            }
            hash_state();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
