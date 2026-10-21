/* TU: src/melee/ft/kinds/ftCommon/ftCo_CaptureCaptain.c */
/* C oracle for src/tu/ftCo_CaptureCaptain.rs: the whole TU included verbatim; every callee out
 * of it is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

#define VSTUB(name, id) void name(Fighter_GObj* g) { T(id); T(g == &GO); }
VSTUB(ftCommon_8007DB58, 1)
VSTUB(ftCo_8009750C, 2)
VSTUB(ftCo_800DD168, 3)
void ftCo_800DB368(Fighter* v, Fighter* f) { T(4); T(v == &F2); T(f == &F); }
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(5); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == &GO2); }
void ftCo_800DB464(Fighter_GObj* g) {}
void ftCommon_8007E2F4(Fighter* f, s16 v) { T(6); T(f == &F); T(v); }
VSTUB(ftAnim_8006EBA4, 7)
VSTUB(ftCommon_8007E2FC, 8)
VSTUB(ft_80083B68, 9)

static u32 state_hash(void)
{
    H = 0x811C9DC5u;
    HM(F.victim_gobj == &GO2); HM(F.x1A5C == &GO2); HM(F.x221B_b5); HM(F.x221B_b7);
    HM(B(F.facing_dir)); HM(F.accessory1_cb == ftCo_800DB464); HM(F.accessory1_cb == NULL);
    return H;
}

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
            F.x221B_b5 = rnd() & 1; F.x221B_b7 = rnd() & 1; F.x2226_b2 = rnd() & 1;
            F.ground_or_air = rnd() & 1;
            F.facing_dir = rg(); F2.facing_dir = rg();
            switch (op) {
            case 0: ftCo_8009CA0C(&GO, &GO2); break;
            case 1: ftCo_CaptureCaptain_Anim(&GO); break;
            case 2: ftCo_CaptureCaptain_IASA(&GO); break;
            case 3: ftCo_CaptureCaptain_Phys(&GO); break;
            case 4: ftCo_CaptureCaptain_Coll(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
