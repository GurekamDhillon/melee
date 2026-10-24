/* TU: src/melee/lb/lb_00CE.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/lbtu/mk_driver.py. Each line: op case ret state-hash trace-hash. */
#include TU_SRC
#include "mtu_common.h"
#include "lbtu_common.h"


#include "lb_00CE_gen.h"

static void setup(void)
{
    int i, k;
    (void) i; (void) k;



}
static void hash_all(void)
{
    int i, k;
    (void) i; (void) k;



}
int main(void)
{
    int op, k;

    freg_all();

    for (op = 0; op < 5; op++) {
        for (k = 0; k < 400; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup();
            switch (op) {
            case 0: {  f32 v0 = (f32) ((float) ((int) (fv() % 81) - 40) / 4.0f); ret = B(expf(v0)); } break; /* expf */
            case 1: {  f32 v0 = (f32) ((float) (fv() % 64) / 8.0f); f32 v1 = (f32) ((float) ((int) (fv() % 25) - 12) / 4.0f); ret = B(powf(v0, v1)); } break; /* powf */
            case 2: {  s32 v0 = (s32) ((int) (fv() % 9) - 4); s32 v1 = (s32) ((int) (fv() % 40) - 3); ret = (u32) powi(v0, v1); } break; /* powi */
            case 3: {  f32 v0 = (f32) (rf()); f32 v1 = (f32) (rf()); ret = B(lb_8000D008(v0, v1)); } break; /* lb_8000D008 */
            case 4: {  f32 v0 = (f32) (rf()); f32 v1 = (f32) (rf()); f32 v2 = (f32) (rf()); f32 v3 = (f32) (rf()); f32 v4 = (f32) (rf()); f32 v5 = (f32) (rf()); f32 v6 = (f32) (rf()); ret = (u32) lb_8000D148(v0, v1, v2, v3, v4, v5, v6); } break; /* lb_8000D148 */
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
