/* TU: src/melee/ft/kinds/ftCommon/ftCo_HammerFall.c */
/* C oracle for src/tu/ftCo_HammerFall.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

static float FR;
static u32 FL;
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(1); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
u32 ftCo_800C54C4(Fighter* f) { T(2); T(f == &F); return FL; }
float ftCo_800C548C(Fighter* f) { T(3); T(f == &F); return FR; }
void ftCo_800C4E94(Fighter* f) { T(4); T(f == &F); }
void ftCo_800C4F64(Fighter_GObj* g) { T(5); T(g == &GO); }
bool ftCo_800C5240(Fighter_GObj* g) { T(6); T(g == &GO); return SR(1); }
void ftCo_800C544C(Fighter* f) { T(7); T(f == &F); }
void ftCommon_8007D5D4(Fighter* f) { T(8); T(f == &F); }
void Item_8026ABD8(Item_GObj* g, Vec3* p, float v) { T(9); T(g == &IGO); T(B(p->x)); T(B(p->y)); T(B(p->z)); T(B(v)); }
void ftCo_Fall_Enter(Fighter_GObj* g) { T(10); T(g == &GO); }
void pl_8003FDF4(int a, int b) { T(11); T(a); T(b); }
void ft_80084DB0(Fighter_GObj* g) { T(12); T(g == &GO); }
void ft_80082C74(Fighter_GObj* g, HSD_GObjEvent cb) { T(13); T(g == &GO); T(cb == ftCo_HammerLanding_Enter); }
void ftCo_HammerLanding_Enter(Fighter_GObj* g) { T(14); }

static u32 state_hash(void)
{
    H = 0x811C9DC5u;
    return H;
}

int main(void)
{
    int op, k;
    tu_init();
    for (op = 0; op < 7; op++) {
        for (k = 0; k < 2000; k++) {
            u32 ret = 0;
            tu_case(op, k);
            FR = rw(); FL = rnd();
            F.input.pressed_buttons = rnd(); F.input.held_buttons[0] = rnd();
            F.x2338.y = rnd() & 1; F.x683 = rnd() & 0xFF; CD.x1C = rnd() % 256;
            F.ground_or_air = rnd() & 1; F.item_gobj = &IGO; F.player_id = rnd() % 6;
            F.is_sub_fighter = rnd() & 1;
            switch (op) {
            case 0: ret = ftCo_800C5CD4(&GO); break;
            case 1: ftCo_800C5D34(&GO); break;
            case 2: ftCo_HammerFall_Anim(&GO); break;
            case 3: ret = ftCo_800C5DDC(&GO); break;
            case 4: ftCo_HammerFall_IASA(&GO); break;
            case 5: ftCo_HammerFall_Phys(&GO); break;
            case 6: ftCo_HammerFall_Coll(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
