/* TU: src/melee/ft/ftlib.c */
/* C oracle for ftLib_UpdateLogicOffCamera (src/tu/ftlib.rs): ftlib.c included whole; Camera_80030A50 and
 * Camera_LogicToScreen are recording stubs. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

static CmSubject CB;
static u32 EP(const void* p) { return p == 0 ? 0 : p == &CB ? 1 : p == &GO ? 2 : 3; }
HSD_GObj* Camera_80030A50(void) { T(1); return SR(0) ? &GO2 : NULL; }
bool Camera_LogicToScreen(CmSubject* s, S32Vec2* pt)
{
    T(2); T(EP(s));
    pt->x = (s32) rnd(); pt->y = (s32) rnd();
    return SR(1);
}
int main(void)
{
    int k;
    tu_init();
    for (k = 0; k < 2000; k++) {
        tu_case(0, k);
        F.x890_cameraBox = &CB;
        F.x221F_b3 = rnd() & 1; F.x2229_b3 = rnd() & 1; F.x2220_b7 = rnd() & 1; F.x221F_b0 = rnd() & 1;
        F.x2188.x = (s32) rnd(); F.x2188.y = (s32) rnd();
        ftLib_UpdateLogicOffCamera(&GO);
        H = 0x811C9DC5u;
        HM(F.x221F_b0); HM((u32) F.x2188.x); HM((u32) F.x2188.y);
        printf("%d %d %08X %08X %08X\n", 0, k, 0u, H, TR);
    }
    return 0;
}
