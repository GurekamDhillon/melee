/* TU: src/melee/ft/kinds/ftCommon/ftCo_JumpAerial.c */
/* C oracle for src/tu/ftCo_JumpAerial.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

static ftData FD;
static FtSFX SFX;
static HSD_GObj X197C;
static FighterBone BONE;
static HSD_JObj JO;
static ftYoshiAttributes YS;
static Vec3 V;

int FighterControls_TapJumpOff(int port) { T(1); T(port); return SR(0); }
float ScriptGame_ModJump(int s, int f, float b) { T(2); T(s); T(f); T(B(b)); return SR(1) ? b * 2.0f : b; }
float ScriptGame_ModValue(int s, int f, float b) { T(3); T(s); T(f); T(B(b)); return SR(2) ? b * 0.5f : b; }
int ScriptGame_CapsMaxJumps(Fighter* f) { T(4); T(f == &F); return (int) (SRV[3] % 6); }
void Mex_OnDoubleJumpDispatch(int kind, void* g, void* v) { T(5); T(kind); T(g == &GO); T(v == NULL); }
void HSD_JObjSetMtxDirtySub(HSD_JObj* j) { T(6); T(j == &JO); }
bool ftCo_800C5240(Fighter_GObj* g) { T(7); return SR(4); }
bool ftCo_800D730C(Fighter_GObj* g, bool a) { T(8); T(a); return SR(5); }
bool ft_800D2D0C(Fighter_GObj* g) { T(9); return SR(6); }
void ft_800D2E7C(Fighter_GObj* g, Vec3* v) { T(10); T(B(v->x)); T(B(v->y)); T(B(v->z)); }
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(11); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
void ft_80088770(Fighter* f) { T(12); T(f == &F); }
void ft_80088328(Fighter* f, int id, u8 v, u8 p) { T(13); T(id); T(v); T(p); }
void ft_PlaySFX(Fighter* f, enum_t id, u8 v, u8 p) { T(14); T(id); T(v); T(p); }
void ftCommon_8007D5D4(Fighter* f) { T(15); f->x1968_jumpsUsed = 1; }
bool ftAnim_IsFramesRemaining(Fighter_GObj* g) { T(16); return SR(7); }
void ftCo_FallAerial_Enter(Fighter_GObj* g) { T(17); }
bool ftCo_SpecialAir_CheckInput(Fighter_GObj* g) { T(18); return SR(8); }
bool ftCo_80095328(Fighter_GObj* g, bool* p) { T(19); T(p == NULL); return SR(9); }
bool ftCo_800D7100(Fighter_GObj* g) { T(20); return SR(10); }
bool ftCo_800C3B10(Fighter_GObj* g) { T(21); return SR(11); }
bool ftCo_80099A58(Fighter_GObj* g) { T(22); return SR(12); }
bool ftCo_AttackAir_CheckItemThrowInput(Fighter_GObj* g) { T(23); return SR(13); }
bool ftCo_800D705C(Fighter_GObj* g) { T(24); return SR(14); }
bool ftPe_8011BA54(HSD_GObj* g) { T(25); return SR(15); }
bool ftPe_8011BAD8(HSD_GObj* g) { T(26); return SR(16); }
bool ftCo_800CEE70(Fighter_GObj* g) { T(27); return SR(17); }
void ft_80084DB0(Fighter_GObj* g) { T(28); }
void ftCommon_CalcSelfAccel_DriftFrom(Fighter* f, float v) { T(29); T(B(v)); f->x74_self_accel.x = v * 0.5f - 0.25f; }
void ftCommon_CalcSelfAccel_Drift(Fighter* f) { T(30); f->x74_self_accel.x = 0.75f; }
void ft_800851D0(Fighter_GObj* g) { T(31); }
bool ftCo_80096CC8(Fighter_GObj* g, int l) { T(32); return 0; }
void ft_80082B1C(Fighter_GObj* g) { T(33); }
void ft_800835B0(Fighter_GObj* g, bool (*a)(Fighter_GObj*, int), HSD_GObjEvent b)
{ T(34); T(g == &GO); T(a == ftCo_80096CC8); T(b == ft_80082B1C); }

static u32 cbid(HSD_GObjEvent cb)
{
    if (cb == NULL) return 0;
    if (cb == (HSD_GObjEvent) ftNs_JumpAerial_Phys_Cb) return 1;
    if (cb == (HSD_GObjEvent) ftCo_JumpAerial_Phys_Cb) return 2;
    if (cb == (HSD_GObjEvent) ftYs_JumpAerial_Anim_Cb) return 3;
    return 9;
}

static u32 state_hash(void)
{
    H = 0x811C9DC5u;
    HM(cbid(F.phys_cb)); HM(cbid(F.anim_cb));
    HM(B(F.self_vel.x)); HM(B(F.self_vel.y)); HM(B(F.self_vel.z));
    HM(F.mv.co.jumpaerial.x0); HM(B(F.mv.co.jumpaerial.init_h_vel));
    HM(B(F.facing_dir)); HM(F.x1968_jumpsUsed); HM(F.active_timer.lstick.y); HM(F.x2221_b7);
    HM(F.cmd_vars[0]); HM(B(F.dmg.armor1)); HM(B(JO.rotate.y)); HM(B(F.x74_self_accel.x));
    return H;
}

int main(void)
{
    int op, k;
    tu_init();
    for (op = 0; op < 18; op++) {
        for (k = 0; k < 2000; k++) {
            u32 ret = 0, kk;
            tu_case(op, k);
            zero(&FD, sizeof FD); zero(&SFX, sizeof SFX); zero(&BONE, sizeof BONE); zero(&JO, sizeof JO);
            zero(&YS, sizeof YS);
            SFX.x14 = rnd() % 1000; FD.x4C_sfx = &SFX; F.ft_data = &FD;
            BONE.joint = &JO; F.parts = &BONE;
            JO.flags = rnd() & (JOBJ_MTX_INDEP_SRT | JOBJ_USER_DEF_MTX | JOBJ_MTX_DIRTY);
            JO.rotate.y = rg();
            YS.x0 = 1 + rnd() % 20; YS.x4 = rg(); YS.x8 = rw(); F.dat_attrs = &YS;
            F.x197C = (rnd() & 1) ? &X197C : NULL;
            F.player_id = rnd() % 6;
            kk = rnd() % 6;
            F.kind = kk == 0 ? Ft_Kind_Ness : kk == 1 ? Ft_Kind_Yoshi : kk == 2 ? Ft_Kind_Peach : kk == 3 ? Ft_Kind_Mewtwo : rnd() % 33;
            F.input.lstick[0].x = rg(); F.input.lstick[0].y = rg();
            F.active_timer.lstick.y = rnd() % 12;
            F.input.pressed_buttons = rnd() & 0xF00;
            F.x68A = rnd() % 12;
            F.x1968_jumpsUsed = rnd() % 6;
            F.can_multijump = rnd() & 1;
            CD.tap_jump_threshold = rg(); CD.tap_jump_window = rnd() % 12; CD.x1C = rnd() % 12; CD.x78 = rg();
            F.co_attrs.air_jump_h_multiplier = rw();
            F.co_attrs.jump_v_initial_velocity = rw();
            F.co_attrs.air_jump_v_multiplier = rw();
            F.mv.co.jumpaerial.x0 = rnd() % 25; F.mv.co.jumpaerial.init_h_vel = rw();
            F.x6A4_transNOffset.z = rw(); F.x74_self_accel.x = rw();
            F.self_vel.x = rw(); F.self_vel.y = rw(); F.self_vel.z = rw();
            F.facing_dir = rg();
            F.cmd_vars[0] = rnd() & 1;
            V.x = rw(); V.y = rw(); V.z = rw();
            switch (op) {
            case 0: { int a = 1 + rnd() % 20; ft_800CB6EC(&F, a); break; }
            case 1: { int a = rnd() & 1; ret = ft_did_jump(&F, a); break; }
            case 2: ret = ftCo_800CB870(&GO); break;
            case 3: ret = ftCo_800CB8E0(&GO); break;
            case 4: { int a = rnd() & 1; ret = ftCo_JumpAerial_CheckInput(&GO, a); break; }
            case 5: { int m = rnd() % 400; int a = rnd() & 1; ftCo_800CBAC4(&GO, m, &V, a); break; }
            case 6: ftCo_JumpAerial_Enter_Basic(&GO); break;
            case 7: ftNs_JumpAerial_Enter(&GO); break;
            case 8: ftYs_JumpAerial_Enter(&GO); break;
            case 9: ftPe_JumpAerial_Enter(&GO); break;
            case 10: ftMt_JumpAerial_Enter(&GO); break;
            case 11: ftCo_JumpAerial_Anim(&GO); break;
            case 12: ftYs_JumpAerial_Anim_Cb(&GO); break;
            case 13: ftCo_JumpAerial_IASA(&GO); break;
            case 14: ftCo_JumpAerial_Phys(&GO); break;
            case 15: ftNs_JumpAerial_Phys_Cb(&GO); break;
            case 16: ftCo_JumpAerial_Phys_Cb(&GO); break;
            case 17: ftCo_JumpAerial_Coll(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
