/* TU: src/melee/ft/kinds/ftCommon/ftCo_Furafura.c */
/* C oracle for src/tu/ftCo_Furafura.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(1); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
void ftCommon_InitGrab(Fighter* f, bool a, float t) { T(2); T(f == &F); T(a); T(B(t)); }
void ftCommon_8007EBAC(Fighter* f, u32 a, u32 b) { T(3); T(f == &F); T(a); T(b); }
void ft_800885A8(Fighter* f, int s, u8 v, u8 p) { T(4); T(f == &F); T(s); T(v); T(p); }
bool ftCommon_GrabMash(Fighter* f, float v) { T(5); T(f == &F); T(B(v)); T(B(f->grab_timer)); return SR(1); }
void ft_8008A2BC(Fighter_GObj* g) { T(6); T(g == &GO); }
void ft_80084F3C(Fighter_GObj* g) { T(7); T(g == &GO); }
void ft_80083F88(Fighter_GObj* g) { T(8); T(g == &GO); }

static u32 state_hash(void)
{
    H = 0x811C9DC5u;
    HM(B(F.shield_health)); HM(B(F.grab_timer));
    return H;
}

int main(void)
{
    int op, k;
    tu_init();
    for (op = 0; op < 5; op++) {
        for (k = 0; k < 2000; k++) {
            u32 ret = 0;
            tu_case(op, k);
            CD.x280_unkShieldHealth = rw(); CD.x2F8 = rw(); CD.x2FC = rw(); CD.x300 = rg(); CD.x304 = rg();
            F.dmg.x1830_percent = rw(); F.grab_timer = rg(); F.shield_health = rg();
            switch (op) {
            case 0: ftCo_80099010(&GO); break;
            case 1: ftCo_Furafura_Anim(&GO); break;
            case 2: ftCo_Furafura_IASA(&GO); break;
            case 3: ftCo_Furafura_Phys(&GO); break;
            case 4: ftCo_Furafura_Coll(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
