/* TU: src/melee/ft/kinds/ftCommon/ftCo_PassiveCeil.c */
/* C oracle for src/tu/ftCo_PassiveCeil.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"
#include "ftco_misc8_fill.h"

BSTUB(ftCo_800986B0, 1)
VSTUB(ftCommon_8007E2FC, 2)
void ftKb_SpecialN_800F1F1C(Fighter_GObj* g, Vec3* v) { T(3); T(PI(g)); T(B(v->x)); T(B(v->y)); T(B(v->z)); v->y = rw(); v->x = rg(); }
CMS(4)
void efAsync_Spawn(HSD_GObj* g, void* q, u32 kind, u32 gfx, HSD_JObj* j, ...)
{
    __builtin_va_list ap;
    Vec3* p;
    __builtin_va_start(ap, j);
    p = __builtin_va_arg(ap, Vec3*);
    __builtin_va_end(ap);
    T(5); T(PI(g)); T(PI(q)); T(kind); T(gfx); T(PI(j)); T(B(p->x)); T(B(p->y)); T(B(p->z));
}
VSTUB(ftCo_80090574, 6)
void ft_800881D8(Fighter* f, int s, u8 v, u8 p) { T(7); T(PI(f)); T(s); T(v); T(p); }
void ft_PlaySFX(Fighter* f, enum_t s, u8 v, u8 p) { T(8); T(PI(f)); T(s); T(v); T(p); }
void ftCommon_8007EBAC(Fighter* f, u32 a, u32 b) { T(9); T(PI(f)); T(a); T(b); }
bool ftCo_800BFFD0(Fighter* f, enum_t a, bool b) { T(10); T(PI(f)); T(a); T(b); return SR(10); }
BSTUB(ftAnim_IsFramesRemaining, 11)
VSTUB(ftCo_Fall_Enter, 12)
VSTUB(ft_80084DB0, 13)
void ft_800831CC(Fighter_GObj* g, bool (*a)(Fighter_GObj*, int), void (*b)(Fighter_GObj*)) { T(14); T(PI(g)); T(a(g, 3)); b(g); }
bool ftCo_80096CC8(Fighter_GObj* g, int l) { T(15); T(PI(g)); T(l); return SR(15); }
VSTUB(ft_80082B1C, 16)

#define NOPS 6
#define NCASE 3000

int main(void)
{
    int op, k;
    tu_init();
    for (op = 0; op < NOPS; op++) {
        for (k = 0; k < NCASE; k++) {
            u32 ret = 0;
            tu_case(op, k);
            m8_fill();
            switch (op) {
            case 0: ret = ftCo_800C23A0(&GO); break;
            case 1: ftCo_800C23FC(&GO); break;
            case 2: ftCo_PassiveCeil_Anim(&GO); break;
            case 3: ftCo_PassiveCeil_IASA(&GO); break;
            case 4: ftCo_PassiveCeil_Phys(&GO); break;
            case 5: ftCo_PassiveCeil_Coll(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
