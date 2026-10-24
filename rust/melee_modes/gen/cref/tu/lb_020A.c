/* TU: src/melee/lb/lb_020A.c */
/* C oracle for src/tu/lb_020A.rs: the whole TU included verbatim; every callee out of it is a
 * recording stub (matrix / quaternion producers fill their output from the random stream).
 * Each line: op case ret state-hash trace-hash. */
#include TU_SRC
static HSD_JObj J[3];
static IKState IK;
static Vec3 V;
static float M[12];
#include "mtu_common.h"
#include "lb_020A_gen.h"

int main(void)
{
    int op, k, i;
    preg(J, sizeof J);
    preg(&IK, sizeof IK);
    preg(&V, sizeof V);
    preg(M, sizeof M);
    freg_all();
    for (op = 0; op < 4; op++) {
        for (k = 0; k < 500; k++) {
            u32 ret = 0;
            tu_case(op, k);
            zero(J, sizeof J);
            for (i = 0; i < 3; i++) {
                fill_HSD_JObj(&J[i]);
                J[i].flags = (rnd() % 3 == 0 ? JOBJ_MTX_DIRTY : 0) | (rnd() % 4 == 0 ? JOBJ_USER_DEF_MTX : 0) |
                             (rnd() % 2 ? JOBJ_USE_QUATERNION : 0) | (rnd() % 3 == 0 ? JOBJ_MTX_INDEP_SRT : 0) |
                             (fv() & 0xF);
            }
            if (TRACEON) printf("S F %08X %08X %08X%c", (u32) J[0].id, (u32) ((u64) J[0].id >> 32), J[0].flags, 10);
            J[0].parent = &J[1];
            J[1].parent = &J[2];
            J[2].parent = 0;
            zero(&IK, sizeof IK);
            fill_IKState(&IK);
            IK.jobj0 = &J[0];
            IK.jobj1 = &J[1];
            IK.jobj2 = &J[2];
            fill_Vec3(&V);
            if (rnd() % 5 == 0) V.x = V.y = V.z = 0.0f;
            for (i = 0; i < 12; i++) M[i] = rg();
            switch (op) {
            case 0: fn_80020AEC(&J[0], (float (*)[4]) M); break;
            case 1: { float a = rg(); float b = rg(); float c = rg(); lbBgFlash_80020E38(&J[0], &V, a, b, c); } break;
            case 2: { float a = rg(); fn_8002113C(&J[0], &V, a); } break;
            case 3: lbBgFlash_80021410(&IK); break;
            }
            if (TRACEON) { int q; for (q = 0; q < (int) sizeof J / 4; q++) printf("D %08X%c", ((u32*) J)[q], 10); for (q = 0; q < (int) sizeof IK / 4; q++) printf("S %08X%c", ((u32*) &IK)[q], 10); for (q = 0; q < 12; q++) printf("S %08X%c", ((u32*) M)[q], 10); fflush(0); }
            hash_HSD_JObj(&J[0]);
            hash_HSD_JObj(&J[1]);
            hash_HSD_JObj(&J[2]);
            hash_IKState(&IK);
            hash_Vec3(&V);
            for (i = 0; i < 12; i++) HM(B(M[i]));
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
