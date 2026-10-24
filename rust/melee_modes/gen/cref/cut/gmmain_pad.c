/* C reference for gmmain.c's static gmMain_8015FD24 (pad clamp setup), gmMain_8015FDA0 and
 * gmMain_8015FDA4 (debug level from develop.ini + launch buttons), cut verbatim. Callees are recording
 * stubs; the globals are the driver's. Line: op case state-hash trace-hash. */
#include <Runtime/platform.h>
#include <sysdolphin/baselib/forward.h>
#include <dolphin/dvd.h>
#include <dolphin/pad.h>
#include <melee/db/db.h>
#include <sysdolphin/baselib/controller.h>
#include <sysdolphin/baselib/debug.h>
#include "mtu_common.h"
//CUT src/melee/gm/gmmain.c::^static HSD_PadData gmMain_8046B108::^static HSD_PadRumbleListData gmMain_8046B1F8
DbLKind DbLevel; bool db_804D6B20; u16 db_gameLaunchButtonState; PadLibData HSD_PadLibData;
void PADSetSpec(u32 spec) { T(1); T(spec); }
void HSD_PadInit(u8 n, HSD_PadData* pad, u16 rn, HSD_PadRumbleListData* rumble)
{
    T(2); T((u32) n); T(PI(pad)); T((u32) rn); T(PI(rumble));
}
s32 DVDConvertPathToEntrynum(const char* path) { T(3); T(SH(path)); return (DbLevel == DbLKind_NoDebugRom && (fv() & 1)) ? -1 : (s32) (fv() % 5); }
void HSD_Panic(const char* f, u32 l, const char* c) { T(4); for (;;) {} }
//CUT src/melee/gm/gmmain.c::^static void gmMain_8015FD24::^}$
//CUT src/melee/gm/gmmain.c::^static void gmMain_8015FDA0::gmMain_8015FDA0
//CUT src/melee/gm/gmmain.c::^static void gmMain_8015FDA4::^}$
static void hash_all(void)
{
    HM((u32) DbLevel);
    HM((u32) db_804D6B20);
    HM(RH(&HSD_PadLibData, sizeof HSD_PadLibData));
}
int main(void)
{
    int op, k;
    preg(gmMain_8046B108, sizeof gmMain_8046B108);
    preg(gmMain_8046B1F8, sizeof gmMain_8046B1F8);
    for (op = 0; op < 3; op++) {
        for (k = 0; k < 600; k++) {
            unsigned char* b = (unsigned char*) &HSD_PadLibData;
            unsigned i;
            tu_case(op, k);
            for (i = 0; i < sizeof HSD_PadLibData; i++) b[i] = (unsigned char) fv();
            DbLevel = (DbLKind) (fv() % 6);
            db_804D6B20 = fv() & 1;
            db_gameLaunchButtonState = fv();
            if (op == 2 && (k & 1)) DbLevel = DbLKind_NoDebugRom;  /* (a missing develop.ini asserts DbLevel == NoDebugRom: the stub only reports it missing then) */
            if (op == 0) gmMain_8015FD24();
            else if (op == 1) gmMain_8015FDA0(fv());
            else gmMain_8015FDA4();
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, 0u, H, TR);
        }
    }
    return 0;
}
