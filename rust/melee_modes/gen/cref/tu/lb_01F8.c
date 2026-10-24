/* TU: src/melee/lb/lb_01F8.c */
/* C oracle for src/tu/lb_01F8.rs: the whole TU included verbatim; every callee out of it is a
 * recording stub. Each line: op case ret state-hash trace-hash. */
#include TU_SRC
static u8 MB[1024];
static int MBN;
static HSD_SObj SOB;
static u8 DEC[64];
static HSD_GObj G;
#include "mtu_common.h"
#include "lb_01F8_gen.h"

int main(void)
{
    int op, k;
    preg(MB, sizeof MB);
    preg(&SOB, sizeof SOB);
    preg(DEC, sizeof DEC);
    preg(&G, sizeof G);
    preg(&lbl_804335B8, sizeof lbl_804335B8);
    freg_all();
    for (op = 0; op < 3; op++) {
        for (k = 0; k < 400; k++) {
            u32 ret = 0;
            tu_case(op, k);
            MBN = 0;
            zero(&lbl_804335B8, sizeof lbl_804335B8);
            fill_lbl_804335B8_t(&lbl_804335B8);
            zero(&SOB, sizeof SOB);
            fill_HSD_SObj(&SOB);
            /* x20 / x44 / x68 / unk94 are pointers the fill leaves NULL: point them into MB */
            lbl_804335B8.x20 = MB + 600; lbl_804335B8.x44 = MB + 700; lbl_804335B8.x68 = MB + 800;
            lbl_804335B8.unk94 = MB + 900;
            lbl_804335B8.x6C = (u16) ((rnd() & 1) ? 0x280 : fv());
            switch (op) {
            case 0: ret = (u32) (lbMthp8001F890(&G) == &SOB); break;
            case 1: lbMthp8001F928(&G, (int) fv()); break;
            case 2: { int w = (rnd() & 1) ? 0x280 : (int) fv(); int h = (int) fv(); lbMthp8001FAA0("movie.mth", w, h); } break;
            }
            if (TRACEON) { int i; printf("D %d %d\n", (int) sizeof lbl_804335B8, (int) sizeof SOB); printf("S P %08X %08X %08X %08X %08X %08X %08X\n", PI(lbl_804335B8.x20), PI(lbl_804335B8.x44), PI(lbl_804335B8.x68), PI(lbl_804335B8.x88.image), PI(lbl_804335B8.x90), PI(lbl_804335B8.unk94), PI(lbl_804335B8.x88.tlut)); for (i = 0; i < (int) sizeof lbl_804335B8 / 4; i++) printf("D %08X\n", ((u32*)&lbl_804335B8)[i]); for (i = 0; i < (int) sizeof SOB / 4; i++) printf("S %08X\n", ((u32*)&SOB)[i]); fflush(0); }
            hash_lbl_804335B8_t(&lbl_804335B8);
            hash_HSD_SObj(&SOB);
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
