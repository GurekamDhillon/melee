/* TU: src/melee/ft/kinds/ftCommon/ftCo_CaptureYoshi.c */
#include TU_SRC
#include "tu_common.h"
#include "ftco_cargo_fill.h"

#define VSTUB(name, id) void name(Fighter_GObj* g) { T(id); T(PI(g)); }
#define CMS void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o) \
{ T(9); T(PI(g)); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(PI(o)); }
static float K(int i) { return (float) ((int) (SRV[i & 15] % 41) - 20) / 4.0f; }
CMS
VSTUB(ftCommon_8007DB58, 1)
VSTUB(ftCo_8009750C, 2)
VSTUB(ftCo_800DD168, 3)
void ftCo_800DB368(Fighter* a, Fighter* b) { T(4); T(PI(a)); T(PI(b)); }
VSTUB(ftCo_800DB464, 5)
void ftCommon_8007D5D4(Fighter* f) { T(6); T(PI(f)); }
void ftCommon_8007E2F4(Fighter* f, s16 v) { T(7); T(PI(f)); T((u32) (s32) v); }
VSTUB(ftAnim_8006EBA4, 8)
VSTUB(ftCommon_8007E2FC, 10)

#define NOPS 5
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
            case 0: F.victim_gobj = NULL; F.x221B_b5 = 1; F.x221B_b7 = rnd() & 1; ftCo_800BBB8C(&GO, &GO2); break;
            case 1: ftCo_CaptureYoshi_Anim(&GO); break;
            case 2: ftCo_CaptureYoshi_IASA(&GO); break;
            case 3: ftCo_CaptureYoshi_Phys(&GO); break;
            case 4: ftCo_CaptureYoshi_Coll(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
