/* TU: src/melee/lb/lbvector.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/lbtu/mk_driver.py. Each line: op case ret state-hash trace-hash. */
#include TU_SRC
#include "mtu_common.h"
#include "lbtu_common.h"
static Vec V[4];
static HSD_CObj CO[1];
static Quaternion Q[1];
static Mtx MT[2];

#include "lbvector_gen.h"

static void setup(void)
{
    int i, k;
    (void) i; (void) k;
    zero(V, sizeof V);
    zero(CO, sizeof CO);
    zero(Q, sizeof Q);
    zero(MT, sizeof MT);
    for (i = 0; i < 4; i++) fill_Vec(&V[i]);
    for (i = 0; i < 1; i++) fill_HSD_CObj(&CO[i]);
    for (i = 0; i < 1; i++) fill_Quaternion(&Q[i]);
    { int i, j, m; for (m = 0; m < 2; m++) for (i = 0; i < 3; i++) for (j = 0; j < 4; j++) MT[m][i][j] = rf(); }
}
static void hash_all(void)
{
    int i, k;
    (void) i; (void) k;

    for (i = 0; i < 4; i++) hash_Vec(&V[i]);
    for (i = 0; i < 1; i++) hash_HSD_CObj(&CO[i]);
    for (i = 0; i < 1; i++) hash_Quaternion(&Q[i]);
    { int m, i, j; for (m = 0; m < 2; m++) for (i = 0; i < 3; i++) for (j = 0; j < 4; j++) HM(B(MT[m][i][j])); }
}
int main(void)
{
    int op, k;
    preg(V, sizeof V);
    preg(CO, sizeof CO);
    preg(Q, sizeof Q);
    preg(MT, sizeof MT);
    freg_all();

    for (op = 0; op < 24; op++) {
        for (k = 0; k < 400; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup();
            switch (op) {
            case 0: {   ret = B(lbVector_Normalize(&V[0])); } break; /* lbVector_Normalize */
            case 1: {   ret = B(lbVector_NormalizeXY(&V[0])); } break; /* lbVector_NormalizeXY */
            case 2: {   ret = PI(lbVector_Add(&V[0], &V[1])); } break; /* lbVector_Add */
            case 3: {   ret = PI(lbVector_Add_xy(&V[0], &V[1])); } break; /* lbVector_Add_xy */
            case 4: {   ret = PI(lbVector_Sub(&V[0], &V[1])); } break; /* lbVector_Sub */
            case 5: {   ret = PI(lbVector_Diff(&V[0], &V[1], &V[2])); } break; /* lbVector_Diff */
            case 6: {   ret = PI(lbVector_CrossprodNormalized(&V[0], &V[1], &V[2])); } break; /* lbVector_CrossprodNormalized */
            case 7: {   ret = B(lbVector_Angle(&V[0], &V[1])); } break; /* lbVector_Angle */
            case 8: {   ret = B(lbVector_AngleXY(&V[0], &V[1])); } break; /* lbVector_AngleXY */
            case 9: {  float v0 = (float) (rf()); ret = B(lbvector_sin(v0)); } break; /* lbvector_sin */
            case 10: {  float v0 = (float) (rf()); ret = B(lbvector_cos(v0)); } break; /* lbvector_cos */
            case 11: {  float v2 = (float) (rf()); lbVector_RotateAboutUnitAxis(&V[0], &V[1], v2); } break; /* lbVector_RotateAboutUnitAxis */
            case 12: {  int v1 = (int) ((int) (fv() % 6)); float v2 = (float) (rf()); lbVector_Rotate(&V[0], v1, v2); } break; /* lbVector_Rotate */
            case 13: {   lbVector_Mirror(&V[0], &V[1]); } break; /* lbVector_Mirror */
            case 14: {   ret = B(lbVector_CosAngle(&V[0], &V[1])); } break; /* lbVector_CosAngle */
            case 15: {  float v3 = (float) (rf()); ret = PI(lbVector_Lerp(&V[0], &V[1], &V[2], v3)); } break; /* lbVector_Lerp */
            case 16: {  float v2 = (float) (rf()); ret = PI(lbVector_8000DE38(MT[0], &V[0], v2)); } break; /* lbVector_8000DE38 */
            case 17: {   ret = PI(lbVector_EulerAnglesFromONB(&V[0], &V[1], &V[2], &V[3])); } break; /* lbVector_EulerAnglesFromONB */
            case 18: {   ret = PI(lbVector_EulerAnglesFromPartialONB(&V[0], &V[1], &V[2])); } break; /* lbVector_EulerAnglesFromPartialONB */
            case 19: {   ret = PI(lbVector_ApplyEulerRotation(&V[0], &V[1])); } break; /* lbVector_ApplyEulerRotation */
            case 20: {  float v0 = (float) (rf()); ret = B(lbVector_sqrtf_accurate(v0)); } break; /* lbVector_sqrtf_accurate */
            case 21: { V[0].x = (float) ((int) (rnd() % 2001) - 1000) * 0.5f; V[0].y = (float) ((int) (rnd() % 2001) - 1000) * 0.5f; V[0].z = (float) ((int) (rnd() % 2001) - 1000) * 0.5f; int v3 = (int) ((int) (fv() & 1)); ret = PI(lbVector_WorldToScreen(&CO[0], &V[0], &V[1], v3)); } break; /* lbVector_WorldToScreen */
            case 22: {   lbVector_CreateEulerMatrix(MT[0], &Q[0]); } break; /* lbVector_CreateEulerMatrix */
            case 23: {   ret = B(lbVector_8000E838(&V[0], &V[1], &V[2], &V[3])); } break; /* lbVector_8000E838 */
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
