/* TU: src/melee/ft/kinds/ftCommon/ftCo_ShieldBreakFly.c */
#include TU_SRC
#include "tu_common.h"
#include "ftco_s209_fill.h"

#define VS(name, id) void name(Fighter_GObj* g) { T(id); T(PI(g)); }
#define BS(name, id) bool name(Fighter_GObj* g) { T(id); T(PI(g)); return SR(id); }
void ftCommon_8007D5D4(Fighter* f) { T(1); T(PI(f)); }
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(2); T(PI(g)); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(PI(o)); }
VS(ftAnim_8006EBA4, 3)
void efAsync_Spawn(HSD_GObj* g, void* q, u32 kind, u32 gfx, HSD_JObj* j, ...) { T(4); T(PI(g)); T(PI(q)); T(kind); T(gfx); T(PI(j)); }
void ft_PlaySFX(Fighter* f, enum_t id, u8 vol, u8 pan) { T(5); T(PI(f)); T(id); T(vol); T(pan); }
void ftCommon_8007EBAC(Fighter* f, u32 a, u32 b) { T(6); T(PI(f)); T(a); T(b); }
void ftColl_8007B62C(Fighter_GObj* g, enum_t n) { T(7); T(PI(g)); T((u32) n); }
BS(ftAnim_IsFramesRemaining, 8)
VS(ftCo_80098D90, 9)
VS(ft_80084EEC, 10)
VS(ftCo_80098E3C, 12) /* ftCo_ShieldBreakDown.c; only its address is used */
void ft_80082C74(Fighter_GObj* g, HSD_GObjEvent cb) { T(11); T(PI(g)); T(cb == ftCo_80098E3C); }
/* ftGetTemplateKind (TARGET_PC): only reached for m-ex kinds, never here (kind < 33). */
int Geno_DefineTemplateSkeleton(int kind) { T(90); return 0; }
int Geno_DefineBaseKind(int kind) { T(91); return kind; }

#define NOPS 6
#define NCASE 1000

int main(void)
{
    int op, k;
    tu_init();
    for (op = 0; op < NOPS; op++) {
        for (k = 0; k < NCASE; k++) {
            u32 ret = 0;
            tu_case(op, k);
            s209_fill();
            switch (op) {
            case 0: if (k & 1) ftCo_80098C9C(&GO); else ftCo_80098B20(&GO); break;
            case 1: ftCo_ShieldBreakFly_Anim(&GO); break;
            case 2: ftCo_ShieldBreakFly_IASA(&GO); break;
            case 3: ftCo_ShieldBreakFly_Phys(&GO); break;
            case 4: ftCo_ShieldBreakFly_Coll(&GO); break;
            case 5: F.kind = (k & 1) ? Ft_Kind_Purin : F.kind; ftCo_80098B20(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
