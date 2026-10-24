/* TU: src/melee/lb/lb_0195.c */
/* C oracle for src/tu/lb_0195.rs: the whole TU included verbatim; every callee out of it is a
 * recording stub. Each line: op case ret state-hash trace-hash. */
/* __OSBusClock is a read of guest memory 0x800000F8 (a wild pointer natively): bind it to the
 * 162 MHz bus clock OSInit stores there */
#include <dolphin/os.h>
#undef __OSBusClock
extern unsigned int g_osbus;
#define __OSBusClock g_osbus
unsigned int g_osbus = 162000000u;
#include TU_SRC
#include "mtu_common.h"
#include "lb_0195_gen.h"

static void evfn(void) { T(7003); }

int main(void)
{
    int op, k;
    preg(&lb_804329F0, sizeof lb_804329F0);
    freg_all();
    freg((const void*) evfn);
    for (op = 0; op < 11; op++) {
        for (k = 0; k < 500; k++) {
            u32 ret = 0;
            tu_case(op, k);
            zero(&lb_804329F0, sizeof lb_804329F0);
            fill_lb_804329F0_t(&lb_804329F0);
            /* bring the period branches into reach */
            if (rnd() % 4 == 0) lb_804329F0.x38 = (u64) lb_804329F0.x0[0].x0;
            if (rnd() % 3 == 0) lb_804329F0.x0[1].x0 = (s64) fv() * 1000;
            if (rnd() % 3 == 0) lb_804329F0.x0[0].x0 = (s64) fv() * 1000;
            if (rnd() % 3 == 0) lb_804329F0.x38 = (u64) fv() * 1000;
            if (rnd() % 3 == 0) lb_804329F0.x40 = (s64) fv() * 5000;
            lb_804329F0.x0[0].x10 = (int) (fv() & 1);
            lb_804329F0.x0[1].x10 = (int) (fv() & 1);
            switch (op) {
            case 0: lb_8001955C(); break;
            case 1: lb_800195D0(); break;
            case 2: fn_800195FC(); break;
            case 3: lb_80019628(); break;
            case 4: { u64 hi = fv(); u64 v = (hi << 32) | fv(); lb_80019880(v); } break;
            case 5: ret = lb_80019894(); break;
            case 6: lb_800198E0(); break;
            case 7: lb_80019900(); break;
            case 8: ret = (u32) lb_80019A30((int) (rnd() & 1)); break;
            case 9: lb_80019A48(); break;
            case 10: lb_80019AAC(evfn); break;
            }
            hash_lb_804329F0_t(&lb_804329F0);
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
