/* TU: src/melee/lb/lbtime.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/lbtu/mk_driver.py. Each line: op case ret state-hash trace-hash. */
#include TU_SRC
#include "mtu_common.h"
#include "lbtu_common.h"
static OSCalendarTime CT[2];

#include "lbtime_gen.h"

static void setup(void)
{
    int i, k;
    (void) i; (void) k;
    zero(CT, sizeof CT);
    for (i = 0; i < 2; i++) fill_OSCalendarTime(&CT[i]);
    __OSBusClock = 162000000u;
}
static void hash_all(void)
{
    int i, k;
    (void) i; (void) k;

    for (i = 0; i < 2; i++) hash_OSCalendarTime(&CT[i]);

}
int main(void)
{
    int op, k;
    preg(CT, sizeof CT);
    freg_all();

    for (op = 0; op < 6; op++) {
        for (k = 0; k < 400; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup();
            switch (op) {
            case 0: {  u32 v0 = (u32) (fv()); u32 v1 = (u32) (fv()); ret = (u32) lbTime_8000AEC8(v0, v1); } break; /* lbTime_8000AEC8 */
            case 1: {  u32 v0 = (u32) (fv()); int v1 = (int) ((int) fv() - (int) (fv() & 0x300)); ret = (u32) lbTime_8000AEE4(v0, v1); } break; /* lbTime_8000AEE4 */
            case 2: {  u32 v0 = (u32) (fv()); int v1 = (int) ((int) fv()); ret = (u32) lbTime_8000AF24(v0, v1); } break; /* lbTime_8000AF24 */
            case 3: {  u32 v0 = (u32) (fv()); int v1 = (int) ((int) fv()); ret = (u32) lbTime_8000AF74(v0, v1); } break; /* lbTime_8000AF74 */
            case 4: {   ret = (u32) lbTime_GetTimeInSeconds(); } break; /* lbTime_GetTimeInSeconds */
            case 5: {  unsigned int v1 = (unsigned int) (fv()); lbTime_8000B028(&CT[0], v1); } break; /* lbTime_8000B028 */
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
