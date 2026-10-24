/* TU: src/melee/lb/lb_013B.c */
/* C oracle for src/tu/lb_013B.rs: the whole TU included verbatim; every callee out of it is a
 * recording stub. Each line: op case ret state-hash trace-hash. */
#include TU_SRC
static ColorOverlay CO;
static union ColorOverlay_x8_t W[128];
static Fighter_804D653C_t RUM[8];
static HSD_GObj G;
#include "mtu_common.h"
#include "lb_013B_gen.h"

static int wn;
static void wcolor(void)
{
    W[wn].light_color.r = (u8) fv(); W[wn].light_color.g = (u8) fv();
    W[wn].light_color.b = (u8) fv(); W[wn].light_color.a = (u8) fv();
    wn++;
}
/* one command at W[wn]: the words the handler of `op` reads, in the view it reads them */
static void put_cmd(u32 op)
{
    switch (op) {
    case 0xB: case 0xF: case 0x13:
        W[wn].unk.unk = op; W[wn].unk.timer = fv() & 0x3FFFFFF; wn++;
        if (op != 0xB) wcolor();
        break;
    case 0xD:
        W[wn].unk.unk = op; W[wn].light_rot2.light_enable = fv() & 1;
        W[wn].light_rot2.x = (int) (fv() & 0xFFF) - 0x800; W[wn].light_rot2.yz = (int) (fv() & 0xFFF) - 0x800; wn++;
        wcolor();
        break;
    case 0x10:
        W[wn].unk.unk = op; W[wn].light_rot1.x = (int) (fv() & 0x1FFF) - 0x1000;
        W[wn].light_rot1.yz = (int) (fv() & 0x1FFF) - 0x1000; wn++;
        break;
    case 0xE: case 0x12:
        W[wn].unk.unk = op; wn++; wcolor();
        break;
    default:
        W[wn].unk.unk = op; wn++;
        break;
    }
}
static void build_stream(int forced)
{
    static const u32 tbl[16] = { 0xB, 0xC, 0xD, 0xE, 0xF, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x20, 0x3F, 0x05, 0x00, 0xA };
    int n, i;
    zero(W, sizeof W);
    wn = 0;
    if (forced >= 0) {
        put_cmd((u32) forced);
        return;
    }
    n = 1 + (int) (rnd() % 10);
    for (i = 0; i < n; i++) put_cmd(tbl[rnd() % 16]);
    put_cmd(0xA);
}
static void cmdcb(Fighter_GObj* g, CommandInfo* i, int op)
{
    ColorOverlay* c = (ColorOverlay*) i;
    T(7001); T(PI(g)); T(PI(i)); T((u32) op);
    c->x8_ptr1++;
    if (rnd() % 4 == 0) c->x0_timer = (int) (rnd() % 3);
}

int main(void)
{
    int op, k, i;
    preg(&CO, sizeof CO);
    preg(W, sizeof W);
    preg(RUM, sizeof RUM);
    preg(&G, sizeof G);
    freg_all();
    for (op = 0; op < 18; op++) {
        for (k = 0; k < 300; k++) {
            u32 ret = 0;
            tu_case(op, k);
            zero(&CO, sizeof CO);
            zero(RUM, sizeof RUM);
            fill_ColorOverlay(&CO);
            for (i = 0; i < 8; i++) {
                fill_Fighter_804D653C_t(&RUM[i]);
                RUM[i].unk = &W[i * 3];
            }
                        switch (op) {
            case 0: build_stream(0xA); CO.x8_ptr1 = W; ret = lb_80013BB0(&CO); break;
            case 1: build_stream(0xB); CO.x8_ptr1 = W; ret = lb_80013BB8(&CO); break;
            case 2: build_stream(0xC); CO.x8_ptr1 = W; ret = lb_80013BE4(&CO); break;
            case 3: build_stream(0xD); CO.x8_ptr1 = W; ret = lb_80013C18(&CO); break;
            case 4: build_stream(0xE); CO.x8_ptr1 = W; ret = lb_80013D68(&CO); break;
            case 5: build_stream(0xF); CO.x8_ptr1 = W; ret = lb_80013E3C(&CO); break;
            case 6: build_stream(0x10); CO.x8_ptr1 = W; ret = lb_80013F78(&CO); break;
            case 7: build_stream(0x11); CO.x8_ptr1 = W; ret = lb_80013FF0(&CO); break;
            case 8: build_stream(0x12); CO.x8_ptr1 = W; ret = lb_80014014(&CO); break;
            case 9: build_stream(0x13); CO.x8_ptr1 = W; ret = lb_800140F8(&CO); break;
            case 10: build_stream(0x14); CO.x8_ptr1 = W; ret = lb_80014234(&CO); break;
            case 11:
                build_stream(-1);
                CO.x8_ptr1 = (rnd() % 8 == 0) ? 0 : W;
                ret = lb_80014258(&G, &CO, cmdcb);
                break;
            case 12: build_stream(-1); CO.x8_ptr1 = W; lb_80014498(&CO); break;
            case 13:
                build_stream(-1);
                CO.x28_colanim.i = (int) (rnd() % 8);
                { int a2 = (int) (rnd() % 8); int a3 = (int) fv(); ret = lb_800144C8(&CO, RUM, a2, a3); }
                break;
            case 14: lb_804D63C0 = 0; lb_80014534(); break;
            case 15: lb_804D63C0 = RUM; { u8 a0 = (u8) fv(); int a1 = (int) fv(); int a2 = (int) (rnd() % 8); int a3 = (int) fv(); lb_80014574(a0, a1, a2, a3); } break;
            case 16: lb_800145C0((u8) fv()); break;
            case 17: lb_800145F4(); break;
            }
                        if (TRACEON) { for (i = 0; i < 31; i++) printf("D %08X\n", ((u32*)&CO)[i]); fflush(0); }
            hash_ColorOverlay(&CO);
            for (i = 0; i < 8; i++) hash_Fighter_804D653C_t(&RUM[i]);
            HM(PI(lb_804D63C0));
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
