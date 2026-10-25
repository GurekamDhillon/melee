/* C reference for the jobj.h inline setters HSD_JObjSetRotationX / ScaleY / TranslateX / TranslateY (and the
 * HSD_JObjSetMtxDirty macro and HSD_JObjMtxIsDirty they use), straight from the header. HSD_JObjSetMtxDirtySub is a
 * recording stub. Line: op case ret state-hash(bytes of the joint) trace-hash. */
#include <Runtime/platform.h>
#include <sysdolphin/baselib/jobj.h>
#include "mtu_common.h"
static HSD_JObj J;
void HSD_JObjSetMtxDirtySub(HSD_JObj* jobj) { T(500); T(PI(jobj)); }
int main(void)
{
    int op, k;
    preg(&J, sizeof J);
    for (op = 0; op < 4; op++) {
        for (k = 0; k < 800; k++) {
            unsigned char* b = (unsigned char*) &J;
            unsigned i;
            f32 v;
            tu_case(op, k);
            for (i = 0; i < sizeof J; i++) b[i] = (unsigned char) fv();
            J.flags = fv() & ~(u32) JOBJ_USE_QUATERNION;
            if (fv() & 1) J.flags &= ~(u32) JOBJ_MTX_INDEP_SRT;
            if (fv() & 1) J.flags &= ~(u32) JOBJ_USER_DEF_MTX;
            v = rg();
            switch (op) {
            case 0: HSD_JObjSetRotationX(&J, v); break;
            case 1: HSD_JObjSetScaleY(&J, v); break;
            case 2: HSD_JObjSetTranslateX(&J, v); break;
            default: HSD_JObjSetTranslateY(&J, v); break;
            }
            HM(RH(&J, sizeof J));
            printf("%d %d %08X %08X %08X\n", op, k, 0u, H, TR);
        }
    }
    return 0;
}
