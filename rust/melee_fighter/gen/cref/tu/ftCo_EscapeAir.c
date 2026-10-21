/* TU: src/melee/ft/kinds/ftCommon/ftCo_EscapeAir.c */
/* C oracle for src/tu/ftCo_EscapeAir.rs: the whole TU included verbatim (TARGET_PC: the script
 * caps check); every callee out of it is a recording stub (cosf/sinf: the real gekko_fp.c).
 * Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"
#include "ftco_dfe_fill.h"

int ScriptGame_CapsActionAllowed(Fighter* f, int flag) { T(1); T(f == &F); T(flag); return chk(1); }
DS_CHK(ftCo_800D705C, 2)
float ftCommon_8007D9D4(Fighter* f) { T(3); T(f == &F); return rw(); }
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(6); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
DS_V(ftAnim_8006EBA4, 7)
void ftCommon_8007EBAC(Fighter* f, u32 a, u32 b) { T(8); T(f == &F); T(a); T(b); }
DS_CHK(ftAnim_IsFramesRemaining, 9)
void ftCo_80096900(Fighter_GObj* g, int a1, int a2, bool ai, float mob, float ll)
{ T(10); T(g == &GO); T(a1); T(a2); T(ai); T(B(mob)); T(B(ll)); }
bool ftCo_80095328(Fighter_GObj* g, bool* b) { T(11); T(g == &GO); T(b != NULL); if (b != NULL) *b = chk(12); return chk(11); }
DS_CHK(ftCo_800C3B10, 13)
DS_V(ft_80084DB0, 14)
void ft_80082C74(Fighter_GObj* g, HSD_GObjEvent cb) { T(15); T(g == &GO); T(cb == ftCo_80099D70); }
void ftCo_LandingFallSpecial_Enter(Fighter_GObj* g, bool a, float b) { T(16); T(g == &GO); T(a); T(B(b)); }

int main(void)
{
    int op, k;
    tu_init();
    for (op = 0; op < 8; op++) {
        for (k = 0; k < 3000; k++) {
            u32 ret = 0;
            int timer;
            tu_case(op, k);
            df_fill();
            timer = rnd() % 3;
            switch (op) {
            case 0: ret = ftCo_80099A58(&GO); break;
            case 1: ftCo_80099A9C(&GO, timer); break;
            case 2: inlineA0(&F); break;
            case 3: ftCo_EscapeAir_Anim(&GO); break;
            case 4: ftCo_EscapeAir_IASA(&GO); break;
            case 5: ftCo_EscapeAir_Phys(&GO); break;
            case 6: ftCo_EscapeAir_Coll(&GO); break;
            case 7: ftCo_80099D70(&GO); break;
            }
            H = 0x811C9DC5u; df_hash();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
