/* TU: src/melee/ft/kinds/ftCommon/ftCo_AttackLw4.c */
/* C oracle for src/tu/ftCo_AttackLw4.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

bool ftCo_800DF3A8(Fighter* f) { T(1); return SR(0); }
bool ftCo_80094E54(Fighter* f) { T(2); return SR(1); }
bool ftCo_800DF3DC(Fighter* f) { T(3); return SR(2); }
void ftCo_800957F4(Fighter_GObj* g, FtMotionId m) { T(4); T(g == &GO); T(m); }
void ftNs_AttackLw4_Enter(Fighter_GObj* g) { T(5); T(g == &GO); }
void Mex_OnDSmashDispatch(int kind, void* g, void* v) { T(6); T(kind); T(g == &GO); T(v == NULL); }
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

#define NOPS 7
#define NCASE 1000

int main(void)
{
    int op, k;
    tu_init();
    for (op = 0; op < NOPS; op++) {
        for (k = 0; k < NCASE; k++) {
            u32 ret = 0;
            tu_case(op, k);
            CD.xD4 = rg(); CD.xD8 = rw();
            F.input.pressed_buttons = rnd() & 0x1FF;
            F.input.lstick[0].y = rg();
            F.active_timer.lstick.y = rnd() % 8;
            F.item_gobj = (rnd() & 1) ? (Item_GObj*) &IGO : NULL;
            F.kind = (rnd() & 1) ? Ft_Kind_Ness : (int) (rnd() % 33);
            F.allow_interrupt = rnd() & 1;
            switch (op) {
            case 0: ret = checkLStick(&F); break;
            case 1: ret = ftCo_AttackLw4_CheckInput(&GO); break;
            case 2: doEnter(&GO); break;
            case 3: ftCo_AttackLw4_Anim(&GO); break;
            case 4: ftCo_AttackLw4_IASA(&GO); break;
            case 5: ftCo_AttackLw4_Phys(&GO); break;
            case 6: ftCo_AttackLw4_Coll(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
