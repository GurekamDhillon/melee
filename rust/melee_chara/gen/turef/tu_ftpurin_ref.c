/* C reference for src/tu/ftpurin.rs ftPr_Init_8013C2F8 (clears the first five hat-joint cache slots;
 * slot 5 is the extra one): the function is cut verbatim from ftpurin.c by tu_pre.py, the static
 * cache array is redeclared. Prints "iter slot0..slot5" after the call. */
#include "ft/types.h"
#include "ft/forward.h"
#include <stdio.h>

static HSD_Joint* ft_8045A1E0[6];
#include "tu_ftpurin_extract.inc"

static u32 s = 0x8013C2u;
static u32 rnd(void) { s ^= s << 13; s ^= s >> 17; s ^= s << 5; return s; }

int main(void)
{
    int i, k;
    for (i = 0; i < 200; i++) {
        for (k = 0; k < 6; k++) {
            ft_8045A1E0[k] = (rnd() & 1) ? (HSD_Joint*) (size_t) (0x1000 + (rnd() % 100)) : NULL;
        }
        ftPr_Init_8013C2F8();
        printf("%d", i);
        for (k = 0; k < 6; k++) {
            printf(" %u", (unsigned) (size_t) ft_8045A1E0[k]);
        }
        printf("\n");
    }
    return 0;
}
