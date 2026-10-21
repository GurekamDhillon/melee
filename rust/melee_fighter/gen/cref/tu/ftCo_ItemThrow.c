/* TU: src/melee/ft/kinds/ftCommon/ftCo_ItemThrow.c */
/* C oracle for src/tu/ftCo_ItemThrow.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace.
 *
 * ftCo_80095D5C reads `((ftCo_ItemThrowCmd*) cmd_vars)->angle` (s32:12 after u32:20). The port
 * reads it in GameCube bit order (low 12 bits); this native build reads the top 12 bits. The
 * generated cmd_vars[0] repeats the same 12 bits in both places so both orders agree. */
#include TU_SRC
#include "tu_common.h"

static float TBL[30 * 3];
int** Fighter_804D6550;
static Vec3 V, LBV;
static bool BOOLV;
static HSD_JObj JO;

enum_t it_8026B30C(Item_GObj* g) { T(1); T(g == &IGO); return SR(0); }
float ftCo_GetLStickAngle(Fighter* f) { T(2); return f->input.lstick[0].y * 1.25f; }
float ftCo_GetCStickAngle(Fighter* f) { T(3); return f->input.cstick[0].y * -1.25f; }
bool ftCo_800DF50C(Fighter* f) { T(4); return SR(1); }
s32 ftGetParasolStatus(Fighter_GObj* g) { T(5); return SR(2) ? -1 : 1; }
void ftCo_800968C8(Fighter_GObj* g) { T(6); }
void ftCo_Fall_Enter(Fighter_GObj* g) { T(7); }
void Item_8026ABD8(Item_GObj* g, Vec3* p, float f) { T(8); T(B(p->x)); T(B(p->y)); T(B(p->z)); T(B(f)); }
float itGetDamageMultiplier(Item_GObj* g) { T(9); return SR(3) ? 0.5f : 3.0f; }
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(10); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
void ftAnim_8006EBA4(Fighter_GObj* g) { T(11); }
void ftCo_800974C4(Fighter_GObj* g) { T(12); }
void ftCo_80096E68(Fighter_GObj* g) { T(13); }
bool ftAnim_IsFramesRemaining(Fighter_GObj* g) { T(14); return SR(4); }
void ftCommon_8007D92C(Fighter_GObj* g) { T(15); }
HSD_JObj* it_80272C90(Item_GObj* g) { T(16); return &JO; }
void lb_8000B1CC(HSD_JObj* j, Vec3* a, Vec3* b) { T(17); T(j == &JO); T(a == NULL); *b = LBV; }
void pl_8003E978(int slot, bool sub, Item_GObj* item) { T(18); T(slot); T(sub); T(item == &IGO); }
void Item_8026AC74(HSD_GObj* g, Vec3* p, Vec3* v, float f)
{ T(19); T(B(p->x)); T(B(p->y)); T(B(p->z)); T(B(v->x)); T(B(v->y)); T(B(v->z)); T(B(f)); }
void Item_8026AD20(HSD_GObj* g, Vec3* p, Vec3* v, float f, bool s)
{ T(20); T(B(p->x)); T(B(p->y)); T(B(p->z)); T(B(v->x)); T(B(v->y)); T(B(v->z)); T(B(f)); T(s); }
bool itIsHeavy(Item_GObj* g) { T(21); return SR(5); }
void ftCommon_8007EBAC(Fighter* f, u32 a, u32 b) { T(22); T(a); T(b); }
void ft_80084F3C(Fighter_GObj* g) { T(23); }
void ft_80085030(Fighter_GObj* g, float a, float b) { T(24); T(B(a)); T(B(b)); }
void ft_80084DB0(Fighter_GObj* g) { T(25); }
void ft_80084104(Fighter_GObj* g) { T(26); }
void ft_800841B8(Fighter_GObj* g, HSD_GObjEvent cb) { T(27); T(cb == ftCo_80096250); T(cb == ftCo_80096E68); cb(g); }
void ft_80082C74(Fighter_GObj* g, HSD_GObjEvent cb) { T(28); T(cb == ftCo_80096374); cb(g); }
enum_t ft_80082708(Fighter_GObj* g) { T(29); return SR(6); }
enum_t ft_80081D0C(Fighter_GObj* g) { T(30); return SR(7); }
void ftCommon_8007D5D4(Fighter* f) { T(31); }
void ftCommon_8007D7FC(Fighter* f) { T(32); }

static u32 cbid(HSD_GObjEvent cb)
{
    if (cb == NULL) return 0;
    if (cb == ftCo_80095EFC) return 1;
    if (cb == ftCo_800974C4) return 2;
    if (cb == ftCo_800961D0) return 3;
    if (cb == ftCo_80096498) return 4;
    return 9;
}

static u32 state_hash(void)
{
    H = 0x811C9DC5u;
    HM(cbid(F.accessory4_cb)); HM(cbid(F.take_dmg_cb)); HM(cbid(F.phys_cb)); HM(cbid(F.coll_cb));
    HM(F.cmd_vars[0]); HM(F.cmd_vars[1]); HM(F.throw_flags_b3); HM(F.throw_flags_b4);
    HM(B(F.facing_dir)); HM(F.x2224_b1); HM(BOOLV);
    HM(B(F.mv.co.itemthrow4.x8.x)); HM(B(F.mv.co.itemthrow4.x8.y)); HM(B(F.mv.co.itemthrow4.x8.z));
    HM(B(F.mv.co.itemthrow4.anim_spd)); HM(F.mv.co.itemthrow4.unk_timer);
    HM(B(F.mv.co.itemthrow.facing_dir)); HM(F.mv.co.itemthrow.x20);
    HM(B(V.x)); HM(B(V.y)); HM(B(V.z));
    return H;
}

int main(void)
{
    int op, k, i;
    tu_init();
    Fighter_804D6550 = (int**) TBL;
    for (op = 0; op < 28; op++) {
        for (k = 0; k < 2000; k++) {
            u32 ret = 0, a12;
            tu_case(op, k);
            for (i = 0; i < 90; i++) TBL[i] = rg();
            V.x = rw(); V.y = rw(); V.z = rw();
            LBV.x = rw(); LBV.y = rw(); LBV.z = rw();
            BOOLV = rnd() & 1;
            F.item_gobj = (rnd() % 8) ? &IGO : NULL;
            F.input.pressed_buttons = rnd() & 0x80000F60;
            F.input.held_buttons[0] = rnd() & 0x80000F60;
            F.input.lstick[0].x = rg(); F.input.lstick[0].y = rg();
            F.input.cstick[0].x = rg(); F.input.cstick[0].y = rg();
            F.active_sticky.lstick.x = rnd() % 16; F.active_sticky.lstick.y = rnd() % 16;
            F.facing_dir = rg(); F.facing_dir1 = rg();
            F.motion_id = (rnd() % 8) == 0 ? ftCo_MS_EscapeF : 94 + rnd() % 30;
            F.player_id = rnd() % 6;
            F.is_sub_fighter = rnd() & 1;
            F.x2222_b0 = rnd() & 1;
            F.throw_flags_b3 = rnd() & 1; F.throw_flags_b4 = rnd() & 1;
            F.ground_or_air = rnd() & 1;
            F.cur_anim_frame = rw(); F.frame_speed_mul = rw(); F.cmd_timer = rw();
            a12 = rnd() & 0xFFF;
            F.cmd_vars[0] = (rnd() & 1) ? ((a12 << 20) | ((rnd() & 0xFF) << 12) | a12) : 0;
            F.cmd_vars[1] = (rnd() & 1) ? rnd() : 0;
            F.co_attrs.jump_startup_time = rw(); F.co_attrs.item_throw_velocity_multiplier = rw();
            F.co_attrs.heavy_throw_velocity_multiplier = rw(); F.co_attrs.ground_friction = rw();
            F.mv.co.itemthrow.facing_dir = rg();
            F.mv.co.itemthrow4.x8.x = rw(); F.mv.co.itemthrow4.x8.y = rw(); F.mv.co.itemthrow4.x8.z = rw();
            F.mv.co.itemthrow.x20 = rnd() % 3;
            CD.dash_smash_stick_threshold = rg(); CD.dash_smash_window = rnd() % 8; CD.x44 = rg();
            CD.xCC = rg(); CD.xD0 = rg(); CD.xD4 = rg(); CD.xD8 = rg() * 8; CD.x98 = rg();
            CD.x20_radians = rg(); CD.attackhi3_stick_threshold_y = rg(); CD.xB0 = rg();
            CD.xDC = rg(); CD.xE0 = rg(); CD.x3FC = rnd() % 16; CD.x400 = rw();
            CD.x404 = rw(); CD.x408 = rw(); CD.x40C = rw();
            switch (op) {
            case 0: ret = ftCo_80094E54(&F); break;
            case 1: ret = ftCo_80094EA4(&GO); break;
            case 2: ret = ftCo_8009515C(&GO); break;
            case 3: ret = ftCo_800951D0(&GO); break;
            case 4: ret = ftCo_80095254(&GO); break;
            case 5: ret = ftCo_800952DC(&GO); break;
            case 6: { int p = rnd() & 1; ret = ftCo_80095328(&GO, p ? &BOOLV : NULL); break; }
            case 7: ret = ftCo_8009563C(&GO); break;
            case 8: { int a = 94 + rnd() % 26; ftCo_80095700(&GO, a); break; }
            case 9: { int p = rnd() & 1; ftCo_80095744(&GO, p ? &BOOLV : NULL); break; }
            case 10: { int m = 94 + rnd() % 30; ftCo_800957F4(&GO, m); break; }
            case 11: { int m = 94 + rnd() % 30; ftCo_800958FC(&GO, m); break; }
            case 12: ftCo_80095A30(&GO); break;
            case 13: if (F.motion_id < 124) ftCo_80095D5C(&F, &V); break;
            case 14: ftCo_ItemThrow_Anim(&GO); break;
            case 15: if (F.motion_id < 124) ftCo_80095EFC(&GO); break;
            case 16: ftCo_ItemThrow_IASA(&GO); break;
            case 17: ftCo_ItemThrow_Phys(&GO); break;
            case 18: ftCo_LightThrowDash_Phys(&GO); break;
            case 19: ftCo_LightThrowAir_Phys(&GO); break;
            case 20: ftCo_800961D0(&GO); break;
            case 21: ftCo_LightThrowDashDrop_Coll(&GO); break;
            case 22: ftCo_LightThrow_Coll(&GO); break;
            case 23: ftCo_80096250(&GO); break;
            case 24: ftCo_LightThrowAir_Coll(&GO); break;
            case 25: ftCo_80096374(&GO); break;
            case 26: ftCo_HeavyThrow_Coll(&GO); break;
            case 27: ftCo_80096498(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
