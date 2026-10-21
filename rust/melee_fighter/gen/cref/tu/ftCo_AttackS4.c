/* TU: src/melee/ft/kinds/ftCommon/ftCo_AttackS4.c */
/* C oracle for src/tu/ftCo_AttackS4.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

static float ANG[2];
static int nd;
static struct ftData_80085FD4_ret DR[4];
#define BSTUB(name, id) bool name(Fighter_GObj* g) { T(id); T(g == &GO); return SR(id); }
#define VSTUB(name, id) void name(Fighter_GObj* g) { T(id); T(g == &GO); }
float ftCo_GetLStickAngle(Fighter* f) { T(1); return ANG[0]; }
float ftCo_GetCStickAngle(Fighter* f) { T(2); return ANG[1]; }
bool ftCo_800DF1C8(Fighter* f) { T(3); return SR(3); }
bool ftCo_800DF21C(Fighter* f) { T(4); return SR(4); }
enum_t it_8026B30C(Item_GObj* i) { T(5); T(i == (Item_GObj*) &IGO); return SRV[5] % 5; }
bool it_8026B594(Item_GObj* i) { T(6); T(i == (Item_GObj*) &IGO); return SR(6); }
void ftCo_800957F4(Fighter_GObj* g, FtMotionId m) { T(7); T(g == &GO); T(m); }
void ftCo_Attack_800CCF58(Fighter_GObj* g, enum_t a) { T(8); T(g == &GO); T(a); }
VSTUB(ftCo_Attack_800CDD14, 9)
int Mex_HasFSmashHook(int kind) { T(10); T(kind); return SR(10); }
void Mex_OnFSmashDispatch(int kind, void* g, void* v) { T(11); T(kind); T(g == &GO); T(v == NULL); }
VSTUB(ftNs_AttackS4_Enter, 12)
VSTUB(ftPe_AttackS4_Enter, 13)
VSTUB(ftGw_AttackS4_Enter, 14)
void efLib_PauseAll(HSD_GObj* g) {}
void efLib_ResumeAll(HSD_GObj* g) {}
struct ftData_80085FD4_ret* ftData_80085FD4(Fighter* f, FtMotionId m) { T(15); T(f == &F); T(m); return &DR[nd++ & 3]; }
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(16); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
VSTUB(ftAnim_8006EBA4, 17)
BSTUB(ftAnim_IsFramesRemaining, 18)
VSTUB(ft_8008A2BC, 19)
VSTUB(ft_80084FA8, 20)
VSTUB(ft_80084104, 21)
BSTUB(ftCo_SpecialS_CheckInput, 22)
BSTUB(ftCo_Attack100_CheckInput, 23)
BSTUB(ftCo_800D6824, 24)
BSTUB(ftCo_800D68C0, 25)
BSTUB(ftCo_Catch_CheckInput, 26)
BSTUB(ftCo_800CECE8, 27)
BSTUB(ftCo_AttackHi4_CheckInput, 28)
BSTUB(ftCo_AttackLw4_CheckInput, 29)
BSTUB(ftCo_AttackS3_CheckInput, 30)
BSTUB(ftCo_AttackHi3_CheckInput, 31)
BSTUB(ftCo_AttackLw3_CheckInput, 32)
BSTUB(ftCo_Attack1_CheckInput, 33)
BSTUB(ftCo_80091A4C, 34)
BSTUB(ftCo_800DE9D8, 35)
BSTUB(ftCo_Jump_CheckInput, 36)
BSTUB(ftCo_Dash_CheckInput, 37)
BSTUB(ftCo_800D5FB0, 38)
BSTUB(ftCo_Turn_CheckInput, 39)
BSTUB(ftCo_Walk_CheckInput, 40)

static u32 state_hash(void)
{
    H = 0x811C9DC5u;
    HM(F.allow_interrupt); HM(B(F.facing_dir)); HM(F.cmd_vars[0]); HM(F.throw_flags);
    HM(F.pre_hitlag_cb == efLib_PauseAll); HM(F.post_hitlag_cb == efLib_ResumeAll);
    HM(F.pre_hitlag_cb == NULL); HM(F.post_hitlag_cb == NULL);
    return H;
}

static const int KINDS[5] = { Ft_Kind_Ness, Ft_Kind_Peach, Ft_Kind_GameWatch, Ft_Kind_Pikachu, Ft_Kind_Pichu };

int main(void)
{
    int op, k, i;
    tu_init();
    for (op = 0; op < 12; op++) {
        for (k = 0; k < 2000; k++) {
            u32 ret = 0;
            float a0, a1;
            tu_case(op, k);
            nd = 0;
            for (i = 0; i < 4; i++) DR[i].x8 = SR(48 + i);
            ANG[0] = rg() * 2; ANG[1] = rg() * 2;
            CD.dash_smash_stick_threshold = rg(); CD.dash_smash_window = (int) (rnd() % 6) - 1;
            CD.xB8_radians = rg(); CD.xBC_radians = rg(); CD.xC0_radians = rg(); CD.xC4_radians = rg();
            F.input.pressed_buttons = rnd() & 0x1FF;
            F.input.held_buttons[0] = rnd() & 0x80000000u;
            F.input.lstick[0].x = rg();
            F.input.cstick[0].x = rg();
            F.active_timer.lstick.x = rnd() % 6;
            F.facing_dir = rg();
            F.item_gobj = (rnd() & 1) ? (Item_GObj*) &IGO : NULL;
            F.kind = (rnd() & 1) ? KINDS[rnd() % 5] : (int) (rnd() % 33);
            F.allow_interrupt = rnd() & 1;
            F.cmd_vars[0] = rnd();
            F.throw_flags = rnd();
            a0 = rg(); a1 = rg() * 2;
            switch (op) {
            case 0: ret = checkLStick(&F); break;
            case 1: ret = ftCo_AttackS4_CheckInput(&GO); break;
            case 2: ret = checkFacingDir(&F); break;
            case 3: ret = ftCo_AttackS4_8008C114(&GO); break;
            case 4: ret = checkItemThrow(&GO, a0); break;
            case 5: decideFighter(&GO, a0, a1); break;
            case 6: doEnter(&GO, a1); break;
            case 7: ftCo_AttackS4_Anim(&GO); break;
            case 8: ftCo_AttackS4_IASA(&GO); break;
            case 9: ftCo_AttackS4_Phys(&GO); break;
            case 10: ftCo_AttackS4_Coll(&GO); break;
            case 11: Fighter_SetEffectHitlagCallbacks(&F); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
