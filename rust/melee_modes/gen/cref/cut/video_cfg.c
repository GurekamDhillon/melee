/* C reference for HSD_VISetConfigure and HSD_VISetBlack (baselib/video.c), cut verbatim, over the
 * driver's HSD_VIData. Line: op case ret state-hash(bytes of HSD_VIData) trace. */
#include <Runtime/platform.h>
#include <sysdolphin/baselib/video.h>
#include "mtu_common.h"
HSD_VIInfo HSD_VIData;
#define _p ((HSD_VIInfo*) &HSD_VIData)
//CUT src/sysdolphin/baselib/video.c::^void HSD_VISetConfigure::^}$
//CUT src/sysdolphin/baselib/video.c::^void HSD_VISetBlack::^}$
int main(void)
{
    int op, k;
    for (op = 0; op < 2; op++) {
        for (k = 0; k < 300; k++) {
            GXRenderModeObj rm;
            unsigned char* b = (unsigned char*) &HSD_VIData;
            unsigned char* r = (unsigned char*) &rm;
            unsigned i;
            tu_case(op, k);
            for (i = 0; i < sizeof HSD_VIData; i++) b[i] = (unsigned char) fv();
            for (i = 0; i < sizeof rm; i++) r[i] = (unsigned char) fv();
            if (op == 0) HSD_VISetConfigure(&rm); else HSD_VISetBlack((int) fv());
            HM(RH(&HSD_VIData, sizeof HSD_VIData));
            printf("%d %d %08X %08X %08X\n", op, k, 0u, H, TR);
        }
    }
    return 0;
}
