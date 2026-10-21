/* TU: src/melee/ft/kinds/ftCommon/ftCo_AttackS3.c */
/* C oracle for src/tu/ftCo_AttackS3.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

static float ANG[2];
static int nang, nd;
static struct ftData_80085FD4_ret DR[4];
float ftCo_GetLStickAngle(Fighter* f) { T(1); return ANG[nang++ & 1]; }
enum_t it_8026B30C(Item_GObj* i) { T(2); T(i == (Item_GObj*) &IGO); return SRV[0] % 5; }
void ftCo_800957F4(Fighter_GObj* g, FtMotionId m) { T(4); T(g == &GO); T(m); }
void ftCo_Attack_800CCF58(Fighter_GObj* g, enum_t a) { T(5); T(g == &GO); T(a); }
void ftCo_Attack_800CDD14(Fighter_GObj* g) { T(6); T(g == &GO); }
bool ftpickupitem_80094790(HSD_GObj* g) { T(14); T(g == &GO); return SR(1); }
struct ftData_80085FD4_ret* ftData_80085FD4(Fighter* f, FtMotionId m) { T(15); T(f == &F); T(m); return &DR[nd++ & 3]; }
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(7); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
void ftAnim_8006EBA4(Fighter_GObj* g) { T(8); T(g == &GO); }
bool ftAnim_IsFramesRemaining(Fighter_GObj* g) { T(9); return SR(3); }
void ft_8008A2BC(Fighter_GObj* g) { T(10); T(g == &GO); }
void ftCo_Wait_IASA(Fighter_GObj* g) { T(11); T(g == &GO); }
void ft_80084F3C(Fighter_GObj* g) { T(12); T(g == &GO); }
void ft_80084104(Fighter_GObj* g) { T(13); T(g == &GO); }

static u32 state_hash(void)
{
    H = 0x811C9DC5u;
    HM(F.allow_interrupt);
    return H;
}

int main(void)
{
    int op, k, i;
    tu_init();
    for (op = 0; op < 6; op++) {
        for (k = 0; k < 2000; k++) {
            u32 ret = 0;
            tu_case(op, k);
            nang = nd = 0;
            for (i = 0; i < 4; i++) DR[i].x8 = SRV[4 + i] & 1;
            ANG[0] = rg() * 2; ANG[1] = rg() * 2;
            CD.x98 = rg(); CD.x20_radians = rg(); CD.x9C_radians = rg(); CD.xA0_radians = rg();
            CD.xA4_radians = rg(); CD.xA8_radians = rg();
            F.input.pressed_buttons = rnd() & 0x1FF;
            F.input.held_buttons[0] = rnd() & 0x80000000u;
            F.input.lstick[0].x = rg();
            F.facing_dir = rg();
            F.item_gobj = (rnd() & 1) ? (Item_GObj*) &IGO : NULL;
            F.allow_interrupt = rnd() & 1;
            switch (op) {
            case 0: ret = ftCo_AttackS3_CheckInput(&GO); break;
            case 1: decideAngle(&GO); break;
            case 2: ftCo_AttackS3_Anim(&GO); break;
            case 3: ftCo_AttackS3_IASA(&GO); break;
            case 4: ftCo_AttackS3_Phys(&GO); break;
            case 5: ftCo_AttackS3_Coll(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
