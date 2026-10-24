/* TU: src/melee/lb/lbtrigf.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/lbtu/mk_driver.py. Each line: op case ret state-hash trace-hash. */
#include TU_SRC
#include "mtu_common.h"
#include "lbtu_common.h"


#include "lbtrigf_gen.h"

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
    { union { u32 u; float f; } c; c.u = 0x7FFFFFFFu; MSL_TrigF_80400770[0] = c.f; c.u = 0x7F800000u; MSL_TrigF_80400774[0] = c.f; }
    for (op = 0; op < 7; op++) {
        for (k = 0; k < 400; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup();
            switch (op) {
            case 0: {  float v0 = (float) (rf()); float v1 = (float) (rf()); ret = B(atan2f(v0, v1)); } break; /* atan2f */
            case 1: {  float v0 = (float) (rf()); ret = B(acosf(v0)); } break; /* acosf */
            case 2: {  float v0 = (float) (rf()); ret = B(asinf(v0)); } break; /* asinf */
            case 3: {  float v0 = (float) (rf()); ret = B(lb_sqrtf(v0)); } break; /* lb_sqrtf */
            case 4: {  float v0 = (float) (rf()); float v1 = (float) (rf()); float v2 = (float) (rf()); ret = B(lbTrigf_Fmadds(v0, v1, v2)); } break; /* lbTrigf_Fmadds */
            case 5: {  float v0 = (float) (rf()); float v1 = (float) (rf()); float v2 = (float) (rf()); ret = B(lbTrigf_Fnmsubs(v0, v1, v2)); } break; /* lbTrigf_Fnmsubs */
            case 6: {  float v0 = (float) (rf()); ret = B(atanf(v0)); } break; /* atanf */
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
