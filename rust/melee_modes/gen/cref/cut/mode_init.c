/* C reference for gm_Mode_Adventure_OnInit (gmadventure.c) and gm_Mode_AllStar_OnInit (gmallstar.c),
 * cut verbatim. The two settings accessors return two distinct records (as on the console, where they
 * are adjacent in the save data); a case fills both from the stream, runs one init, and prints the
 * bytes of both. Line: op case rec0 rec1 (each six bytes as hex). */
#include <melee/gm/gmmain_lib.h>
#include "mtu_common.h"
static struct gmm_x0_528_t REC[2];
struct gmm_x0_528_t* gmMainLib_8015CDD4(void) { return &REC[0]; }
struct gmm_x0_528_t* gmMainLib_8015CDE0(void) { return &REC[1]; }
//CUT src/melee/gm/gmadventure.c::^void gm_Mode_Adventure_OnInit::^}$
//CUT src/melee/gm/gmallstar.c::^void gm_Mode_AllStar_OnInit::^}$
static void dump(const struct gmm_x0_528_t* r)
{
    const unsigned char* b = (const unsigned char*) r;
    int i;
    for (i = 0; i < 6; i++) printf("%02X", b[i]);
}
int main(void)
{
    int op, k, i;
    for (op = 0; op < 2; op++) {
        for (k = 0; k < 20; k++) {
            unsigned char* b = (unsigned char*) REC;
            tu_case(op, k);
            for (i = 0; i < 12; i++) b[i] = (unsigned char) fv();
            if (op == 0) gm_Mode_Adventure_OnInit(); else gm_Mode_AllStar_OnInit();
            printf("%d %d ", op, k);
            dump(&REC[0]);
            printf(" ");
            dump(&REC[1]);
            printf("\n");
        }
    }
    return 0;
}
