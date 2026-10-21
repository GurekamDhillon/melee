/* TU: src/melee/ft/kinds/ftCommon/ftCo_DamageSong.c */
/* C oracle for src/tu/ftCo_DamageSong.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

static int HC, BB;
void ftCommon_8007DB58(Fighter_GObj* g) { T(1); T(g == &GO); }
void ftCo_8009750C(Fighter_GObj* g) { T(2); T(g == &GO); }
void ftCo_800DD168(Fighter_GObj* g) { T(3); T(g == &GO); }
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(4); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
int Player_GetHandicap(int slot) { T(5); T(slot); return HC; }
s32 Player_80033BB8(int slot) { T(6); T(slot); return BB; }
void ftCommon_InitGrab(Fighter* f, bool a, float t) { T(7); T(f == &F); T(a); T(B(t)); }
bool ftCommon_GrabMash(Fighter* f, float v) { T(8); T(f == &F); T(B(v)); return SR(9); }
bool ftAnim_IsFramesRemaining(Fighter_GObj* g) { T(9); return SR(3); }
void ft_8008A2BC(Fighter_GObj* g) { T(10); T(g == &GO); }
void ft_80084F3C(Fighter_GObj* g) { T(11); T(g == &GO); }
void ft_80084280(Fighter_GObj* g) { T(12); T(g == &GO); }
void ftCo_800C3598(Fighter_GObj* g) { T(13); T(g == &GO); }

static u32 state_hash(void)
{
    H = 0x811C9DC5u;
    HM(B(F.grab_timer));
    return H;
}

int main(void)
{
    int op, k;
    tu_init();
    for (op = 0; op < 16; op++) {
        for (k = 0; k < 3000; k++) {
            u32 ret = 0;
            tu_case(op, k);
            HC = (int) (rnd() % 9) - 1; BB = (int) (rnd() % 6) - 1;
            CD.x624 = rw(); CD.x628 = rw(); CD.x62C = rw(); CD.x630 = rw(); CD.x634 = rw();
            CD.x638 = rw(); CD.x63C = rw(); CD.x640 = rg(); CD.x644 = rw();
            F.dmg.x1830_percent = rw();
            F.player_id = rnd() % 6;
            F.grab_timer = rw();
            F.x2222_b2 = rnd() & 1;
            F.x2070.x2071_b0_3 = rnd() & 15;
            switch (op) {
            case 0: ftCo_800C318C(&GO, rnd() & 1); break;
            case 1: ftCo_DamageSong_Anim(&GO); break;
            case 2: ftCo_DamageSong_IASA(&GO); break;
            case 3: ftCo_DamageSong_Phys(&GO); break;
            case 4: ftCo_DamageSong_Coll(&GO); break;
            case 5: ftCo_800C3390(&GO); break;
            case 6: ftCo_DamageSongWait_Anim(&GO); break;
            case 7: ftCo_DamageSongWait_IASA(&GO); break;
            case 8: ftCo_DamageSongWait_Phys(&GO); break;
            case 9: ftCo_DamageSongWait_Coll(&GO); break;
            case 10: ftCo_800C3480(&GO); break;
            case 11: ftCo_DamageSongRv_Anim(&GO); break;
            case 12: ftCo_DamageSongRv_IASA(&GO); break;
            case 13: ftCo_DamageSongRv_Phys(&GO); break;
            case 14: ftCo_DamageSongRv_Coll(&GO); break;
            case 15: ret = ftCo_800C3538(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
