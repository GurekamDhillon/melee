/* TU: src/melee/ft/kinds/ftCommon/ftCo_DownAttack.c */
/* C oracle for src/tu/ftCo_DownAttack.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

HSD_GObj** HSD_GObjPLinkHead;
static HSD_GObj* PL[16];
static Item_FtTrack TRK;
bool ftCo_800DF644(Fighter* f) { T(3); T(f == &F); return SR(2); }
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(4); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
void ftAnim_8006EBA4(Fighter_GObj* g) { T(5); T(g == &GO); }
bool ftAnim_IsFramesRemaining(Fighter_GObj* g) { T(7); return SR(3); }
void ft_8008A2BC(Fighter_GObj* g) { T(8); T(g == &GO); }
void ft_80084F3C(Fighter_GObj* g) { T(9); T(g == &GO); }
void ft_80084104(Fighter_GObj* g) { T(10); T(g == &GO); }
bool ftCo_800C5240(Fighter_GObj* g) { T(12); T(g == &GO); return SR(4); }
void ftCo_800987D0(Fighter_GObj* g) { T(13); T(g == &GO); }

static u32 state_hash(void)
{
    int i;
    H = 0x811C9DC5u;
    HM(F.motion_id);
    HM(TRK.count);
    for (i = 0; i < 11; i++) {
        HM(B(TRK.ecb_offset_arr[i].top)); HM(B(TRK.ecb_offset_arr[i].bottom));
        HM(B(TRK.ecb_offset_arr[i].right)); HM(B(TRK.ecb_offset_arr[i].left));
        HM(B(TRK.ft_pos_arr[i].x)); HM(B(TRK.ft_pos_arr[i].y)); HM(B(TRK.ft_pos_arr[i].z));
    }
    return H;
}

int main(void)
{
    static const FtMotionId ms[3] = { ftCo_MS_DownWaitU, ftCo_MS_DownWaitD, 0 };
    int op, k;
    tu_init();
    HSD_GObjPLinkHead = PL;
    for (op = 0; op < 9; op++) {
        for (k = 0; k < 2000; k++) {
            u32 ret = 0;
            tu_case(op, k);
            zero(&TRK, sizeof TRK);
            CD.x250 = rw(); CD.x1C = rnd() % 20;
            F.input.pressed_buttons = rnd() & 0x3FF;
            F.motion_id = ms[rnd() % 3];
            F.x680 = rnd() % 20; F.x684 = rnd() % 20;
            F.cur_pos.x = rw(); F.cur_pos.y = rw(); F.cur_pos.z = rw();
            F2.cur_pos.x = rw(); F2.cur_pos.y = rw(); F2.cur_pos.z = rw();
            switch (rnd() % 3) {
            case 0: PL[HSD_GOBJ_PLINK_FIGHTER] = NULL; break;
            case 1: PL[HSD_GOBJ_PLINK_FIGHTER] = &GO; GO.next = NULL; break;
            default: PL[HSD_GOBJ_PLINK_FIGHTER] = &GO2; GO2.next = &GO; GO.next = NULL; break;
            }
            switch (op) {
            case 0: ret = ftCo_800984D4(&GO); break;
            case 1: ftCo_8009856C(&GO, rnd() % 400); break;
            case 2: ftCo_DownAttack_Anim(&GO); break;
            case 3: ftCo_DownAttack_IASA(&GO); break;
            case 4: ftCo_DownAttack_Phys(&GO); break;
            case 5: ftCo_DownAttack_Coll(&GO); break;
            case 6: ftCo_80098634(&TRK); break;
            case 7: ret = ftCo_800986B0(&GO); break;
            case 8: ret = ftCo_8009872C(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
