/* TU: src/melee/lb/lbsnap.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/lbtu/mk_driver.py. Each line: op case ret state-hash trace-hash. */
#include TU_SRC
#include "mtu_common.h"
#include "lbtu_common.h"
static Unk80433380_0 SNAP[1];
static Unk80433380_48 SLOT[2];
static LbMcSnapMemSnapIconData ICON[2];
static unsigned char BANNER[6208];
static unsigned char ICNB[1024];
static unsigned char SRC[458752];
static char TXT[64];
static u32 SRCT[16];
#include "lbsnap_gen.h"

static void setup(void)
{
    int i, k;
    (void) i; (void) k;
    zero(SNAP, sizeof SNAP);
    zero(SLOT, sizeof SLOT);
    zero(ICON, sizeof ICON);
    zero(BANNER, sizeof BANNER);
    zero(ICNB, sizeof ICNB);
    zero(SRC, sizeof SRC);
    zero(TXT, sizeof TXT);
    for (i = 0; i < 1; i++) fill_Unk80433380_0(&SNAP[i]);
    for (i = 0; i < 2; i++) fill_Unk80433380_48(&SLOT[i]);
    __OSBusClock = 162000000u;
    for (k = 0; k < 2; k++) {
        int q;
        fill_Unk80433380_48(&SLOT[k]);
        SLOT[k].num = 8 + (int) (rnd() % 100);
        SLOT[k].card_result = (rnd() % 3 == 0) ? 0 : (int) fv();
        for (q = 0; q < 0x7F; q++) SLOT[k].entries[q].time = rnd() % 6;
    }
    fill_Unk80433380_0(&SNAP[0]);
    SNAP[0].x0 = (rnd() & 1) ? 4 : (int) fv();
    fill_Unk80433380(&lbSnap_80433380);
    lbSnap_80433380.snap = &SNAP[0];
    lbSnap_80433380.slot = &SLOT[0];
    lbSnap_80433380.icon_data = ICON;
    ICON[0].ptr = BANNER; ICON[1].ptr = ICNB;
    for (k = 0; k < 0x40; k++) TXT[k] = (char) (rnd() | 1);
    for (k = 0; k < 0x1800 + 64; k++) BANNER[k] = (unsigned char) rnd();
    for (k = 0; k < 0x400; k++) ICNB[k] = (unsigned char) rnd();
}
static void hash_all(void)
{
    int i, k;
    (void) i; (void) k;
    hash_Unk80433380(&lbSnap_80433380);
    hash_Unk803BACC8(&lbSnap_803BACC8);
    HM(PIB(ICON[0].ptr)); HM(PIB(ICON[1].ptr));
    hash_Unk80433380_0(&SNAP[0]);
    for (k = 0; k < 2; k++) hash_Unk80433380_48(&SLOT[k]);
    for (i = 0; i < 1; i++) hash_Unk80433380_0(&SNAP[i]);
    for (i = 0; i < 2; i++) hash_Unk80433380_48(&SLOT[i]);
    for (k = 0; k < 0x40; k++) HM((u32) (unsigned char) TXT[k]);
    for (k = 0; k < 0x1800 + 64; k++) HM(BANNER[k]);
    for (k = 0; k < 0x400; k++) HM(ICNB[k]);
}
int main(void)
{
    int op, k;
    preg(SNAP, sizeof SNAP);
    preg(SLOT, sizeof SLOT);
    preg(ICON, sizeof ICON);
    preg(BANNER, sizeof BANNER);
    preg(ICNB, sizeof ICNB);
    preg(SRC, sizeof SRC);
    preg(TXT, sizeof TXT);
    preg(&lbSnap_80433380, sizeof lbSnap_80433380);
    preg(&lbSnap_803BACC8, sizeof lbSnap_803BACC8);
    freg_all();

    for (op = 0; op < 22; op++) {
        for (k = 0; k < 400; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup();
            switch (op) {
            case 0: {   lbSnap_8001D2BC(); } break; /* lbSnap_8001D2BC */
            case 1: {  int v0 = (int) ((int) (fv() % 3)); ret = (u32) lbSnap_8001D338(v0); } break; /* lbSnap_8001D338 */
            case 2: {  int v0 = (int) ((int) (fv() & 1)); ret = (u32) lbSnap_8001D350(v0); } break; /* lbSnap_8001D350 */
            case 3: {  int v0 = (int) ((int) (fv() & 1)); ret = (u32) lbSnap_8001D394(v0); } break; /* lbSnap_8001D394 */
            case 4: {  int v0 = (int) ((int) (fv() & 1)); ret = (u32) lbSnap_8001D3B0(v0); } break; /* lbSnap_8001D3B0 */
            case 5: {  int v0 = (int) ((int) (fv() & 1)); ret = (u32) lbSnap_8001D3CC(v0); } break; /* lbSnap_8001D3CC */
            case 6: {  int v0 = (int) ((int) (fv() & 1)); int v1 = (int) ((int) (fv() % 8)); ret = (u32) lbSnap_8001D3E8(v0, v1); } break; /* lbSnap_8001D3E8 */
            case 7: {  int v0 = (int) ((int) (fv() & 1)); ret = (u32) lbSnap_8001D40C(v0); } break; /* lbSnap_8001D40C */
            case 8: {  int v0 = (int) ((int) (fv() & 1)); lbSnap_8001D4A4(v0, TXT); } break; /* lbSnap_8001D4A4 */
            case 9: {  int v0 = (int) ((int) (fv() & 1)); int v1 = (int) ((int) (fv() % 8)); ret = (u32) lbSnap_8001D5FC(v0, v1); } break; /* lbSnap_8001D5FC */
            case 10: {  int v0 = (int) ((int) (fv() & 1)); int v1 = (int) ((int) (fv() % 8)); int v2 = (int) ((int) (fv() % 8)); ret = (u32) lbSnap_8001D7B0(v0, v1, v2); } break; /* lbSnap_8001D7B0 */
            case 11: { { int q; for (q = 0; q < 16; q++) SRCT[q] = rnd(); for (q = 0; q < 0x70000 / 4; q++) ((u32*) SRC)[q] = SRCT[q & 15] ^ ((u32) q * 2654435761u); }  lbSnap_8001DA5C(SRC); } break; /* lbSnap_8001DA5C */
            case 12: { { int q; for (q = 0; q < 16; q++) SRCT[q] = rnd(); for (q = 0; q < 0x70000 / 4; q++) ((u32*) SRC)[q] = SRCT[q & 15] ^ ((u32) q * 2654435761u); }  ret = (u32) lbSnap_8001DC0C(SRC); } break; /* lbSnap_8001DC0C */
            case 13: { { int q; for (q = 0; q < 16; q++) SRCT[q] = rnd(); for (q = 0; q < 0x70000 / 4; q++) ((u32*) SRC)[q] = SRCT[q & 15] ^ ((u32) q * 2654435761u); }  ret = (u32) lbSnap_8001DE8C(SRC); } break; /* lbSnap_8001DE8C */
            case 14: {   ret = (u32) lbSnap_8001DF20(); } break; /* lbSnap_8001DF20 */
            case 15: {  int v0 = (int) ((int) (fv() & 1)); ret = (u32) lbSnap_8001DF6C(v0); } break; /* lbSnap_8001DF6C */
            case 16: {  int v0 = (int) ((int) (fv() & 1)); int v1 = (int) ((int) (fv() % 8)); ret = (u32) lbSnap_8001E058(v0, v1); } break; /* lbSnap_8001E058 */
            case 17: {   ret = (u32) lbSnap_8001E204(); } break; /* lbSnap_8001E204 */
            case 18: {   ret = (u32) lbSnap_8001E210(); } break; /* lbSnap_8001E210 */
            case 19: {   lbSnap_8001E218(SRC, &SLOT[0]); } break; /* lbSnap_8001E218 */
            case 20: {   lbSnap_8001E27C(); } break; /* lbSnap_8001E27C */
            case 21: {   lbSnap_8001E290(); } break; /* lbSnap_8001E290 */
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
