/* TU: src/melee/lb/lb_0146.c */
/* C oracle for src/tu/lb_0146.rs: the whole TU included verbatim; every callee out of it is a
 * recording stub. Each line: op case ret state-hash trace-hash. */
#include TU_SRC
static HSD_CObj COB;
static struct lb_80014638_arg0_t A0;
static struct lb_80014638_arg1_t A1;
static Vec3 V[2];
static float M[3][4];
#include "mtu_common.h"
#include "lb_0146_gen.h"

static void hash_colors(void)
{
    HM(*(u32*) &lb_ColorWhite); HM(*(u32*) &lb_ColorBlack); HM(*(u32*) &red);
    HM(*(u32*) &translucent_red); HM(*(u32*) &yellow);
}

int main(void)
{
    int op, k, i, j;
    preg(&COB, sizeof COB);
    preg(&A0, sizeof A0);
    preg(&A1, sizeof A1);
    preg(V, sizeof V);
    preg(M, sizeof M);
    freg_all();
    for (op = 0; op < 3; op++) {
        for (k = 0; k < 600; k++) {
            u32 ret = 0;
            tu_case(op, k);
            zero(&COB, sizeof COB);
            fill_lb_80014638_arg0_t(&A0);
            fill_lb_80014638_arg1_t(&A1);
            fill_Vec3(&V[0]);
            fill_Vec3(&V[1]);
            for (i = 0; i < 3; i++) for (j = 0; j < 4; j++) M[i][j] = rg();
            switch (op) {
            case 0:
                /* make the z tests of the segment check pass often */
                if (rnd() & 1) { A1.x8.z = A0.x0.z + 0.5f; A0.xC.z = A0.x0.z; }
                ret = lb_80014638(&A0, &A1);
                break;
            case 1: ret = lb_80014770(V, (rnd() & 1) ? 2 : (int) (fv() % 4)); break;
            case 2: ret = lb_800149E0(M, (rnd() & 1) ? 0 : fv() % 3); break;
            }
            hash_lb_80014638_arg0_t(&A0);
            hash_lb_80014638_arg1_t(&A1);
            hash_Vec3(&V[0]);
            hash_Vec3(&V[1]);
            for (i = 0; i < 3; i++) for (j = 0; j < 4; j++) HM(B(M[i][j]));
            hash_colors();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
