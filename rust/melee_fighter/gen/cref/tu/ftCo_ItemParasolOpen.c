/* TU: src/melee/ft/kinds/ftCommon/ftCo_ItemParasolOpen.c */
/* C oracle for src/tu/ftCo_ItemParasolOpen.rs: the whole TU included verbatim; every callee out
 * of it is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

static u32 isf(Fighter* f) { return f == &F; }
s32 ftGetParasolStatus(Fighter_GObj* g) { T(1); T(g == &GO); return SR(1) ? 6 : (s32) (rnd() % 8); }
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(2); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
void ftAnim_8006EBA4(Fighter_GObj* g) { T(3); T(g == &GO); }
bool ftAnim_IsFramesRemaining(Fighter_GObj* g) { T(4); T(g == &GO); return SR(2); }
void ftCo_800CF3C8(Fighter_GObj* g) { T(5); T(g == &GO); }
void ftCo_800CF280(Fighter_GObj* g) { T(6); T(g == &GO); }
bool ftCo_SpecialAir_CheckInput(Fighter_GObj* g) { T(7); return SR(3); }
bool ftCo_80095328(Fighter_GObj* g, bool* p) { T(8); T(p == NULL); return SR(4); }
bool ftCo_800C3B10(Fighter_GObj* g) { T(9); return SR(5); }
bool ftCo_AttackAir_CheckItemThrowInput(Fighter_GObj* g) { T(10); return SR(6); }
bool ftCo_800CB870(Fighter_GObj* g) { T(11); return SR(7); }
void ftCommon_Fall(Fighter* f, float a, float b) { T(12); T(isf(f)); T(B(a)); T(B(b)); }
void ftCommon_CalcSelfAccel_AccelToVelClamped(Fighter* f, float a, float b, float c)
{ T(13); T(isf(f)); T(B(a)); T(B(b)); T(B(c)); }
void ftCo_Landing_Enter_Basic(Fighter_GObj* g) { T(14); T(g == &GO); }
void ft_8008370C(Fighter_GObj* g, HSD_GObjEvent cb) { T(15); T(g == &GO); T(cb == ftCo_Landing_Enter_Basic); cb(g); }

static u32 state_hash(void)
{
    H = 0x811C9DC5u;
    HM(F.motion_id); HM(F.mv.co.parasol_open.prev_msid); HM(F.x2104);
    HM(F.x2221_b4); HM(F.x2221_b5); HM(F.x2221_b6); HM(F.x2221_b7);
    return H;
}

int main(void)
{
    int op, k;
    static const int ms[4] = { 29, ftCo_MS_FallSpecial, 36, 360 };
    tu_init();
    for (op = 0; op < 10; op++) {
        for (k = 0; k < 2000; k++) {
            u32 ret = 0;
            tu_case(op, k);
            F.motion_id = ms[rnd() % 4];
            F.mv.co.parasol_open.prev_msid = ms[rnd() % 4];
            F.kind = (rnd() & 1) ? Ft_Kind_Peach : (FighterKind) (rnd() % 30);
            F.u.pe.parasol_gobj_0 = (rnd() & 1) ? (Item_GObj*) &IGO : NULL;
            F.x2221_b4 = rnd() & 1; F.x2221_b5 = rnd() & 1; F.x2221_b6 = rnd() & 1; F.x2221_b7 = rnd() & 1;
            F.x2104 = (int) (rnd() % 100);
            F.input.lstick[0].x = rg(); F.input.lstick[0].y = rg();
            F.self_vel.y = rg();
            F.co_attrs.gravity = rw(); F.co_attrs.terminal_velocity = rw();
            F.co_attrs.air_drift_stick_mul = rw(); F.co_attrs.air_drift_max = rw();
            F.co_attrs.aerial_friction = rw();
            CD.x590 = rg(); CD.x58C = rw(); CD.x258 = rg(); CD.open_parasol_threshold = rg();
            CD.x59C = rw();
            switch (op) {
            case 0: ret = ftCo_800CEE70(&GO); break;
            case 1: ft_800CEF08(&GO); break;
            case 2: ftCo_800CEFE0(&GO, (int) rnd()); break;
            case 3: ftCo_ItemParasolOpen_Anim(&GO); break;
            case 4: ftCo_ItemParasolOpen_IASA(&GO); break;
            case 5: ftCo_ItemParasolOpen_Phys(&GO); break;
            case 6: ftCo_ItemParasolOpen_Coll(&GO); break;
            case 7: ret = ftCo_ItemParasolGetFallMotionId(&GO, 1000, 2000); break;
            case 8: ftCo_ItemParasol_Phys(&GO); break;
            case 9: ret = ftCo_800CEE70(&GO); ftCo_ItemParasolOpen_Anim(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
