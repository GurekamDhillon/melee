/* TU: src/melee/ft/kinds/ftCommon/ftCo_DamageBind.c */
/* C oracle for src/tu/ftCo_DamageBind.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

void ftCommon_8007DB58(Fighter_GObj* g) { T(1); T(g == &GO); }
void ftCo_8009750C(Fighter_GObj* g) { T(2); T(g == &GO); }
void ftCo_800DD168(Fighter_GObj* g) { T(3); T(g == &GO); }
void ftCo_80090780(Fighter_GObj* g) { T(4); T(g == &GO); }
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(5); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
int Player_GetHandicap(int s) { T(6); T(s); return SRV[1] % 8; }
s32 Player_80033BB8(int s) { T(7); T(s); return SRV[2] % 5; }
void ftCommon_InitGrab(Fighter* f, bool a, float t) { T(8); T(f == &F); T(a); T(B(t)); }
bool ftCommon_GrabMash(Fighter* f, float v) { T(9); T(f == &F); T(B(v)); return SR(3); }
void ft_8008A2BC(Fighter_GObj* g) { T(10); T(g == &GO); }
void ft_80084F3C(Fighter_GObj* g) { T(11); T(g == &GO); }
void ft_80084280(Fighter_GObj* g) { T(12); T(g == &GO); }

static u32 state_hash(void)
{
    H = 0x811C9DC5u;
    HM(B(F.grab_timer));
    return H;
}

static u32 run(int op)
{
    switch (op) {
    case 0: return ftCo_800C44CC(&GO);
    case 1: commonCall(&F); break;
    case 2: ftCo_800C4550(&GO); break;
    case 3: ftCo_DamageBind_Anim(&GO); break;
    case 4: ftCo_DamageBind_IASA(&GO); break;
    case 5: ftCo_DamageBind_Phys(&GO); break;
    case 6: ftCo_DamageBind_Coll(&GO); break;
    }
    return 0;
}

int main(void)
{
    int op, k;
    tu_init();
    for (op = 0; op < 7; op++) {
        for (k = 0; k < 2000; k++) {
            u32 ret;
            tu_case(op, k);
            F.dmg.x1860_element = rnd() % 12;
            F.is_sandbag = rnd() & 1;
            F.x2070.x2071_b0_3 = rnd() % 16;
            F.ground_or_air = rnd() & 1;
            F.dmg.x1830_percent = rw();
            F.player_id = rnd() % 4;
            F.grab_timer = rw();
            CD.x658 = rw();
            CD.x65C = rg();
            CD.x660 = rw();
            CD.x664 = rg();
            CD.pressed_inputs = rw();
            CD.released_inputs = rg();
            CD.x670 = rg();
            CD.x674 = rg();
            ret = run(op);
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
