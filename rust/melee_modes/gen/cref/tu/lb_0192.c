/* TU: src/melee/lb/lb_0192.c */
/* C oracle for src/tu/lb_0192.rs: the whole TU included verbatim; every callee out of it is a
 * recording stub. Each line: op case ret state-hash trace-hash. */
#include TU_SRC
static HSD_Text TXT;
static const long DVDT[16] = { 5, 4, 6, 11, -1, 1, 0, 2, 3, 7, 8, 9, 10, 12, 13, 5 };
#include "mtu_common.h"
#include "lb_0192_gen.h"

static void cbfn(void) { T(7002); }

int main(void)
{
    int op, k;
    preg(&TXT, sizeof TXT);
    preg(&gmMainLib_8046B0F0, sizeof gmMainLib_8046B0F0);
    freg_all();
    freg((const void*) cbfn);
    for (op = 0; op < 2; op++) {
        for (k = 0; k < 600; k++) {
            u32 ret = 0;
            tu_case(op, k);
            zero(&TXT, sizeof TXT);
            fill_HSD_Text(&TXT);
            fill_gmMainLib_8046B0F0_t(&gmMainLib_8046B0F0);
            lb_804D63D0 = 0;
            if (op == 0) {
                ret = (u32) lb_80019230();
            } else {
                lb_800192A8((rnd() & 1) ? cbfn : 0);
            }
            hash_HSD_Text(&TXT);
            hash_gmMainLib_8046B0F0_t(&gmMainLib_8046B0F0);
            HM(PI(lb_804D63D0));
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
