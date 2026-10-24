/* C reference for rust/melee_modes/src/tu/lbgx.rs: the game's lbgx.c verbatim, every GX call stubbed and logged.
 * Run: python rust/melee_modes/gen/turef/tu_shard833.py tu_lbgx */
#include "../../../../src/melee/lb/lbgx.c"
#include "tu_gm_common.h"

#define LG(...) lg(__VA_ARGS__)
static HSD_CObj CO;
static Vec4 V4;
static Vec3 V3;
static U8Vec4 CL;
static const char* pid(const void* p)
{
    if (p == NULL) return "null";
    if (p == CO.view_mtx) return "view";
    return "?";
}
HSD_CObj* HSD_CObjGetCurrent(void) { LG("cobj();"); return &CO; }
#include "tu_lbgx_stubs.inc"

int main(void)
{
    int i, k;
    for (i = 0; i < 120; i++) {
        u32 sel = (rnd() % 4 == 0) ? (rnd() % 5) : 2;
        float f1 = rf();
        V4.x = rf(); V4.y = rf(); V4.z = rf(); V4.w = rf();
        V3.x = rf(); V3.y = rf(); V3.z = rf();
        CL.x = (u8) rnd(); CL.y = (u8) rnd(); CL.z = (u8) rnd(); CL.w = (u8) rnd();
        g_loglen = 0; g_log[0] = 0;
        k = lbGx_8001E2F8(&V4, &V3, &CL, sel, f1);
        printf("%d sel=%u f1=%08x ret=%d log=%s\n", i, (unsigned) sel, bits(f1), k, g_log);
    }
    return 0;
}
