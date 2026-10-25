/* C reference for mnName_8023749C (mnname.c) with the name tables mnnamenew.c defines (one empty
 * string each), cut verbatim. Line: op case ret(0 = NULL, else string hash | 1) state-hash trace-hash. */
#include <Runtime/platform.h>
#include <melee/lb/lblanguage.h>
#include "mtu_common.h"
//CUT src/melee/mn/mnnamenew.c::^static u8 mnNameNew_804D4F98::^char\* mnNameNew_803EE724
//CUT src/melee/mn/mnname.c::^char mnName_StringTerminator::^char mnName_StringTerminator
bool lbLang_IsSavedLanguageUS(void) { T(1); return fv() & 1; }
//CUT src/melee/mn/mnname.c::^char\* mnName_8023749C::^}$
int main(void)
{
    int k;
    for (k = 0; k < 512; k++) {
        char* r;
        u32 ret;
        tu_case(0, k);
        r = mnName_8023749C(k < 256 ? k : (int) fv());
        ret = r ? (SH(r) | 1) : 0;
        HM(0);
        printf("%d %d %08X %08X %08X\n", 0, k, ret, H, TR);
    }
    return 0;
}
