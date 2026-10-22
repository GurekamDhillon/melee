/* TU: src/melee/if/iftime.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/if_drv.py. Each line: op case ret state-hash trace-hash. */
#include TU_SRC
#include "mtu_common.h"
static DynamicModelDesc* TIMM[4];
static DynamicModelDesc* CDM[4];
static HSD_Archive* ARCP;
static Vec3 TPOS;
void OSPanic(const char* f, int l, const char* m, ...) { for (;;) {} }

#include "iftime_gen.h"



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
    for (op = 0; op < 15; op++) {
        static const int NC[] = {400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400};
        for (k = 0; k < NC[op]; k++) {
            u32 ret = 0;
            tu_case(op, k);
            reset_state();
            setup_extra();
            switch (op) {
            case 0: {    ret = (u32) ifTime_LoadModels(); } break;
            case 1: { HSD_JObj * a0; unsigned int a1;  a0 = (HSD_JObj *) (pick_HSD_JObj(0)); a1 = (unsigned int) (fv()); ifTime_SetDigit(a0, a1); } break;
            case 2: { HSD_JObj * a0; int a1; int a2;  a0 = (HSD_JObj *) (pick_HSD_JObj(0)); a1 = (int) (fv()); a2 = (int) ((0 + (int) (rnd() % 100u))); ifTime_SetTime(a0, a1, a2); } break;
            case 3: {    ifTime_HideTimers(); } break;
            case 4: {    ifTime_ShowTimers(); } break;
            case 5: {    ret = (u32) ifTime_GetCountdownSeconds(); } break;
            case 6: {    ret = (u32) ifTime_GetCountdownSeconds_dontinline(); } break;
            case 7: { HSD_GObj * a0; int a1;  a0 = (HSD_GObj *) (pick_HSD_GObj(0)); a1 = (int) (fv()); ifTime_RenderGuarded(a0, a1); } break;
            case 8: { HSD_GObj * a0;  a0 = (HSD_GObj *) (pick_HSD_GObj(0)); ifTime_UpdateCountdown(a0); } break;
            case 9: {    ifTime_FreeCountdown(); } break;
            case 10: { HSD_GObj * a0;  a0 = (HSD_GObj *) (pick_HSD_GObj(0)); ifTime_UpdateTimers(a0); } break;
            case 11: {    ifTime_CreateTimers(); } break;
            case 12: {    ifTime_Reset(); } break;
            case 13: {    ifTime_FreeTimers(); } break;
            case 14: {    ret = (u32) ifTime_IsTimerHidden(); } break;
            }
            hash_state();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
