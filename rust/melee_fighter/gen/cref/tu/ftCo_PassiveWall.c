/* TU: src/melee/ft/kinds/ftCommon/ftCo_PassiveWall.c */
/* C oracle for src/tu/ftCo_PassiveWall.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"
#include "ftco_misc8_fill.h"

BSTUB(ftCo_800986B0, 1)
VSTUB(ftCommon_8007E2FC, 2)
void ftKb_SpecialN_800F1F1C(Fighter_GObj* g, Vec3* v) { T(3); T(PI(g)); T(B(v->x)); T(B(v->y)); T(B(v->z)); v->x = rw(); v->y = rg(); }
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
BSTUB(ft_80081F2C, 6)
void ft_800881D8(Fighter* f, int s, u8 v, u8 p) { T(7); T(PI(f)); T(s); T(v); T(p); }
void ftCommon_8007EBAC(Fighter* f, u32 a, u32 b) { T(8); T(PI(f)); T(a); T(b); }
void ft_PlaySFX(Fighter* f, enum_t s, u8 v, u8 p) { T(9); T(PI(f)); T(s); T(v); T(p); }
bool ftCo_800BFFD0(Fighter* f, enum_t a, bool b) { T(10); T(PI(f)); T(a); T(b); return SR(10); }
void ftColl_8007B760(Fighter_GObj* g, int v) { T(11); T(PI(g)); T(v); }
void ftAnim_SetAnimRate(Fighter_GObj* g, float r) { T(12); T(PI(g)); T(B(r)); }
float powf(float x, float y) { T(13); T(B(x)); T(B(y)); return rw(); }
BSTUB(ftAnim_IsFramesRemaining, 14)
VSTUB(ftCo_Fall_Enter, 15)
BSTUB(ftCo_SpecialAir_CheckInput, 16)
bool ftCo_80095328(Fighter_GObj* g, bool* b) { T(17); T(PI(g)); T(b == NULL); return SR(17); }
BSTUB(ftCo_800D7100, 18)
BSTUB(ftCo_800C3B10, 19)
BSTUB(ftCo_80099A58, 20)
BSTUB(ftCo_AttackAir_CheckItemThrowInput, 21)
BSTUB(ftCo_800D705C, 22)
BSTUB(ftCo_800CB870, 23)
BSTUB(ftCo_800CEE70, 24)
bool ftCommon_CheckFallFast(Fighter* f) { T(25); T(PI(f)); f->fall_fast = SR(25); return SR(41); }
void ftCommon_FallFast(Fighter* f) { T(26); T(PI(f)); }
void ftCommon_Fall(Fighter* f, float a, float b) { T(27); T(PI(f)); T(B(a)); T(B(b)); }
void ftCommon_CalcSelfAccel_AccelToVelClamped(Fighter* f, float a, float b, float c) { T(28); T(PI(f)); T(B(a)); T(B(b)); T(B(c)); }
void ft_80083318(Fighter_GObj* g, bool (*a)(Fighter_GObj*, int), void (*b)(Fighter_GObj*)) { T(29); T(PI(g)); T(a(g, 5)); b(g); }
void ft_800831CC(Fighter_GObj* g, bool (*a)(Fighter_GObj*, int), void (*b)(Fighter_GObj*)) { T(30); T(PI(g)); T(a(g, 3)); b(g); }
bool ftCo_80096CC8(Fighter_GObj* g, int l) { T(31); T(PI(g)); T(l); return SR(31); }
VSTUB(ft_80082B1C, 32)

#define NOPS 8
#define NCASE 4000

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
            case 0: ret = ftCo_800C1D38(&GO); break;
            case 1: ret = ftCo_800C1E0C(&F); break;
            case 2: {
                int ms = (int) (rnd() % 400);
                int tm = (int) (rnd() % 3);
                int ve = (int) (rnd() % 4);
                float fd = rg();
                ftCo_800C1E64(&GO, ms, tm, ve, fd);
                break;
            }
            case 3: ftCo_PassiveWall_Anim(&GO); break;
            case 4: ftCo_PassiveWall_IASA(&GO); break;
            case 5: ftCo_PassiveWall_Phys(&GO); break;
            case 6: ftCo_PassiveWall_Coll(&GO); break;
            case 7: ftCo_PassiveWall_Anim(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
