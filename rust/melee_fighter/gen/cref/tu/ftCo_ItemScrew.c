/* TU: src/melee/ft/kinds/ftCommon/ftCo_ItemScrew.c */
/* C oracle for src/tu/ftCo_ItemScrew.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

static Vec3 V;

void ftCommon_8007D5D4(Fighter* f) { T(1); T(f == &F); }
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(2); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
void ftCo_800CB110(Fighter_GObj* g, bool a, f32 m) { T(3); T(g == &GO); T(a); T(B(m)); }
bool ftAnim_IsFramesRemaining(Fighter_GObj* g) { T(4); T(g == &GO); return SR(1); }
void ftCo_Fall_Enter(Fighter_GObj* g) { T(5); T(g == &GO); }
void ftCo_FallAerial_Enter(Fighter_GObj* g) { T(6); T(g == &GO); }
void ftCo_Jump_IASA(Fighter_GObj* g) { T(7); T(g == &GO); }
void ftCo_Jump_Phys_Inner(Fighter_GObj* g) { T(8); T(g == &GO); }
void ftCo_JumpAerial_IASA(Fighter_GObj* g) { T(9); T(g == &GO); }
void ft_80084DB0(Fighter_GObj* g) { T(10); T(g == &GO); }
void ftCo_800CBAC4(Fighter_GObj* g, FtMotionId m, Vec3* v, bool a)
{ T(11); T(g == &GO); T(m); T(v == &V); T(B(v->x)); T(B(v->y)); T(B(v->z)); T(a); }
bool ftCo_80096CC8(Fighter_GObj* g, int l) { T(12); return 0; }
void ft_80082B1C(Fighter_GObj* g) { T(13); }
void ft_800831CC(Fighter_GObj* g, bool (*a)(Fighter_GObj*, int), HSD_GObjEvent b)
{ T(14); T(g == &GO); T(a == ftCo_80096CC8); T(b == ft_80082B1C); }

static u32 state_hash(void)
{
    H = 0x811C9DC5u;
    HM(F.x2227_b0); HM(B(V.x)); HM(B(V.y)); HM(B(V.z));
    return H;
}

int main(void)
{
    int op, k;
    tu_init();
    for (op = 0; op < 10; op++) {
        for (k = 0; k < 1000; k++) {
            u32 ret = 0;
            tu_case(op, k);
            CD.x800 = rw();
            V.x = rw(); V.y = rw(); V.z = rw();
            switch (op) {
            case 0: ftCo_ItemScrew_Enter(&GO); break;
            case 1: ftCo_ItemScrew_Anim(&GO); break;
            case 2: ftCo_ItemScrew_IASA(&GO); break;
            case 3: ftCo_ItemScrew_Phys(&GO); break;
            case 4: ftCo_ItemScrew_Coll(&GO); break;
            case 5: ft_800D2E7C(&GO, &V); break;
            case 6: ftCo_ItemScrewAir_Anim(&GO); break;
            case 7: ftCo_ItemScrewAir_IASA(&GO); break;
            case 8: ftCo_ItemScrewAir_Phys(&GO); break;
            case 9: ftCo_ItemScrewAir_Coll(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
