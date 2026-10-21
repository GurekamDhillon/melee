/* TU: src/melee/ft/kinds/ftCommon/ftCo_CaptureWaitKirby.c */
#include TU_SRC
#include "tu_common.h"
#include "ftco_cargo_fill.h"

#define VSTUB(name, id) void name(Fighter_GObj* g) { T(id); T(PI(g)); }
#define CMS void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o) \
{ T(9); T(PI(g)); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(PI(o)); }
static float K(int i) { return (float) ((int) (SRV[i & 15] % 41) - 20) / 4.0f; }
CMS
void ftCo_800DB368(Fighter* a, Fighter* b) { T(1); T(PI(a)); T(PI(b)); }
VSTUB(Fighter_UpdateModelScale, 2)
VSTUB(ftCo_800DB464, 3)
void ftColl_8007B62C(Fighter_GObj* g, enum_t v) { T(4); T(PI(g)); T(v); }
void ftCommon_8007D5D4(Fighter* f) { T(5); T(PI(f)); }
void ftCommon_8007E2F4(Fighter* f, s16 v) { T(6); T(PI(f)); T((u32) (s32) v); }
VSTUB(ftAnim_8006EBA4, 7)
VSTUB(ftCommon_8007E2FC, 8)
void ftKb_SpecialN_800F598C(Fighter_GObj* g, int a) { T(10); T(PI(g)); T(a); }
bool ftKb_SpecialN_800F597C(Fighter_GObj* g) { T(11); T(PI(g)); return SR(11); }
VSTUB(ftKb_SpecialN_800F5954, 12)
void ftKb_SpecialN_800F5A04(Fighter_GObj* g, f32 a) { T(13); T(PI(g)); T(B(a)); }
bool ftKb_SpecialN_800F5A38(Fighter_GObj* g, Fighter* v) { T(14); T(PI(g)); T(PI(v)); return SR(14); }
float ftKb_SpecialN_800F5B00(Fighter_GObj* g) { T(15); T(PI(g)); return K(15); }
float ftKb_SpecialN_800F5B10(Fighter_GObj* g) { T(16); T(PI(g)); return K(0); }
float ftKb_SpecialN_800F5A60(Fighter_GObj* g) { T(17); T(PI(g)); return K(1); }
bool ftCommon_GrabMash(Fighter* f, float a) { T(18); T(PI(f)); T(B(a)); return SR(18); }
void HSD_JObjSetMtxDirtySub(HSD_JObj* j) { T(19); T(j == &JO); }
VSTUB(ftCo_CaptureCut_Enter, 20)
void ftCo_800DA698(Fighter_GObj* g, bool b) { T(21); T(PI(g)); T(b); }

#define NOPS 7
#define NCASE 1000

int main(void)
{
    int op, k;
    tu_init();
    for (op = 0; op < NOPS; op++) {
        for (k = 0; k < NCASE; k++) {
            u32 ret = 0;
            tu_case(op, k);
            cargo_fill();
            switch (op) {
            case 0: ftCo_800BD620(&GO); break;
            case 1: ftCo_CaptureWaitKirby_Anim(&GO); break;
            case 2: ftCo_800BD6EC_noinline(&GO); break;
            case 3: ftCo_800BD6EC(&GO); break;
            case 4: ftCo_CaptureWaitKirby_IASA(&GO); break;
            case 5: ftCo_CaptureWaitKirby_Phys(&GO); break;
            case 6: ftCo_CaptureWaitKirby_Coll(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
