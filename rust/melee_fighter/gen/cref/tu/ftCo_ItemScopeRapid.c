/* TU: src/melee/ft/kinds/ftCommon/ftCo_ItemScopeRapid.c */
/* C oracle for src/tu/ftCo_ItemScopeRapid.rs: the whole TU included verbatim; every callee out of
 * it is a recording stub. Each line: op case ret state-hash trace.
 *
 * The IASA functions use GameCube offsets: `fp + 0x2340` (= mv.co.common.x0),
 * `p_ftCommonData + 0x5BC` (= x5BC) and, in the air one, `gobj + 0x2C` (= user_data). Compiled
 * natively those are raw byte offsets, so this driver plants &F at GO+0x2C and seeds / hashes the
 * native bytes at F+0x2340 for those two ops; the Rust side uses the named fields. */
#include TU_SRC
#include "tu_common.h"

#define OV_I(o) (*(s32*) ((u8*) &F + (o)))

FtMotionId fn_800D769C(Fighter* f, FtMotionId m) { T(1); T(f == &F); T(m); return SR(0) ? m + 8 : m; }
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(2); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
void ftCommon_ClampAirDrift(Fighter* f) { T(3); }
void ftCommon_8007D7FC(Fighter* f) { T(4); }
void ftCommon_8007D5D4(Fighter* f) { T(5); }
void fn_800D8378(Fighter_GObj* g) { T(6); }
void ft_800892A0(Fighter_GObj* g) { T(7); }
void ft_80089824(Fighter_GObj* g) { T(8); }
void fn_800D86E0(Fighter_GObj* g) { T(9); }
void fn_800D874C(Fighter_GObj* g) { T(10); }
bool it_8026B594(Item_GObj* g) { T(11); T(g == &IGO); return SR(1); }
void ftCo_800CDE54(Fighter_GObj* g) { T(12); }
void ftCo_800CDE74(Fighter_GObj* g) { T(13); }
void ft_800841B8(Fighter_GObj* g, HSD_GObjEvent cb) { T(14); T(cb == fn_800D7D70); cb(g); }
void ft_80082C74(Fighter_GObj* g, HSD_GObjEvent cb) { T(15); T(cb == fn_800D7CEC); cb(g); }
int it_80291DAC(Item_GObj* g, int n) { T(16); T(n); return SR(2) ? 7 : -1; }
void fn_800D84D4(Fighter_GObj* g, int n) { T(17); T(n); }

static u32 state_hash(int op)
{
    H = 0x811C9DC5u;
    HM(F.accessory4_cb == NULL ? 0 : F.accessory4_cb == fn_800D80F4 ? 1 : 9);
    HM(F.take_dmg_cb == NULL ? 0 : F.take_dmg_cb == fn_800D8378 ? 2 : 9);
    HM(op == 6 || op == 7 ? OV_I(0x2340) : F.mv.co.common.x0);
    return H;
}

int main(void)
{
    int op, k;
    Fighter* pf = &F;
    tu_init();
    memcpy((u8*) &GO + 0x2C, &pf, sizeof pf);
    for (op = 0; op < 13; op++) {
        for (k = 0; k < 2000; k++) {
            u32 ret = 0;
            tu_case(op, k);
            F.item_gobj = (rnd() % 8) ? &IGO : NULL;
            F.motion_id = 158 + rnd() % 16;
            F.ground_or_air = rnd() & 1;
            F.input.pressed_buttons = rnd() & 0x100;
            F.cur_anim_frame = rg() * 2.0f; F.frame_speed_mul = rg() * 2.0f;
            F.mv.co.common.x0 = rnd() % 3;
            OV_I(0x2340) = F.mv.co.common.x0;
            CD.x5BC = rnd() % 100;
            switch (op) {
            case 0: fn_800D7BDC(&GO); break;
            case 1: fn_800D7C60(&GO); break;
            case 2: fn_800D7CEC(&GO); break;
            case 3: fn_800D7D70(&GO); break;
            case 4: ftCo_ItemScopeRapid_Anim(&GO); break;
            case 5: ftCo_ItemScopeAirRapid_Anim(&GO); break;
            case 6: ftCo_ItemScopeRapid_IASA(&GO); break;
            case 7: ftCo_ItemScopeAirRapid_IASA(&GO); break;
            case 8: ftCo_ItemScopeRapid_Phys(&GO); break;
            case 9: ftCo_ItemScopeAirRapid_Phys(&GO); break;
            case 10: ftCo_ItemScopeRapid_Coll(&GO); break;
            case 11: ftCo_ItemScopeAirRapid_Coll(&GO); break;
            case 12: fn_800D80F4(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(op), TR);
        }
    }
    return 0;
}
