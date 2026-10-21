/* TU: src/melee/ft/kinds/ftCommon/ftCo_CaptureWaitKoopa.c */
#include TU_SRC
#include "tu_common.h"
#include "ftco_cargo_fill.h"

#define VSTUB(name, id) void name(Fighter_GObj* g) { T(id); T(PI(g)); }
#define CMS void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o) \
{ T(9); T(PI(g)); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(PI(o)); }
static float K(int i) { return (float) ((int) (SRV[i & 15] % 41) - 20) / 4.0f; }
CMS
VSTUB(ftCo_800DB464, 1)
void ftCommon_8007E2F4(Fighter* f, s16 v) { T(2); T(PI(f)); T((u32) (s32) v); }
VSTUB(ftCo_800BC458, 3)
VSTUB(ftCo_800BC4A8, 4)
void ftCo_800DA698(Fighter_GObj* g, bool b) { T(5); T(PI(g)); T(b); }
VSTUB(ftCo_CaptureCut_Enter, 6)

#define NOPS 12
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
            case 0: ftCo_800BCC20(&GO); break;
            case 1: ftCo_CaptureWaitKoopa_Anim(&GO); break;
            case 2: ftCo_CaptureWaitKoopa_IASA(&GO); break;
            case 3: ftCo_CaptureWaitKoopa_Phys(&GO); break;
            case 4: ftCo_CaptureWaitKoopa_Coll(&GO); break;
            case 5: ftCo_800BCD00(&GO); break;
            case 6: ftCo_CaptureWaitKoopaAir_Anim(&GO); break;
            case 7: ftCo_CaptureWaitKoopaAir_IASA(&GO); break;
            case 8: ftCo_CaptureWaitKoopaAir_Phys(&GO); break;
            case 9: ftCo_CaptureWaitKoopaAir_Coll(&GO); break;
            case 10: ftCo_CaptureWaitKoopa_Anim(&GO); F.grab_timer = 0; ftCo_CaptureWaitKoopaAir_Anim(&GO); break;
            case 11: F.grab_timer = 1; ftCo_CaptureWaitKoopa_Anim(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
