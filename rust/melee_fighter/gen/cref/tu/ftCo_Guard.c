/* TU: src/melee/ft/kinds/ftCommon/ftCo_Guard.c */
/* C oracle for src/tu/ftCo_Guard.rs: the whole TU (TARGET_PC) included verbatim; every callee out
 * of it is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

static float MV, ENDF;
static FighterBone PARTS[8];
static HSD_JObj JO[8], JO2[8], SKEL;
static HSD_Joint CJ, CJC, JPOSE, OWN[3];
static HSD_Joint* JTAB[4];
static ftData FD;
static struct ftData_x8 FD8;
static ftData_x20 FD20;
static MotionState MS[ftCo_MS_Guard + 1];
static FighterPartsTable PT;
static FighterPartsTable* PTP[Ft_Kind_Max];
FighterPartsTable** ftPartsTable;
static u8 COL[64];
u8* Fighter_804D650C;
static int FIGA;
static Vec3 POS;

static void TJ(HSD_Joint* j)
{
    for (; j != NULL; j = j->next) {
        T(100); T(B(j->rotation.x)); T(B(j->rotation.y)); T(B(j->rotation.z));
        T(B(j->scale.x)); T(B(j->scale.y)); T(B(j->scale.z));
        T(B(j->position.x)); T(B(j->position.y)); T(B(j->position.z)); T(j->flags);
        TJ(j->child);
        T(101);
    }
}

int ScriptGame_CapsActionAllowed(Fighter* f, int flag) { T(1); T(f == &F); T(flag); return SR(6) | SR(7); }
float ScriptGame_ModValue(int slot, int field, float base) { T(2); T(slot); T(field); T(B(base)); return MV; }
float lb_8000D008(float y, float x) { T(3); T(B(y)); T(B(x)); return rg(); }
void HSD_JObjSetMtxDirtySub(HSD_JObj* j) { T(4); T(j - JO); }
void efLib_SetParamAlpha(HSD_GObj* g, u8 a) { T(5); T(g == &GO); T(a); }
HSD_Joint* GenoDefine_RestPoseTree(Fighter* f) { T(6); T(f == &F); return SR(8) ? &OWN[0] : NULL; }
int Geno_MotionAnimRow(Fighter* f, int m) { T(7); T(f == &F); T(m); return (int) (SRV[9] % 40) - 10; }
FigaTree* ftData_80085E50(Fighter* f, enum_t m) { T(8); T(f == &F); T(m); return SR(10) ? (FigaTree*) &FIGA : NULL; }
void ftAnim_8006F4C8(Fighter* f, bool b, FigaTree* t) { T(9); T(f == &F); T(b); T(t == (FigaTree*) &FIGA); }
void ftAnim_80070710(HSD_JObj* j, float fr) { T(10); T(j == &SKEL); T(B(fr)); }
void ftAnim_8006FB88(Fighter* f, Fighter_Part p, HSD_Joint* j) { T(11); T(f == &F); T(p); T(j == &CJC); T(j == &OWN[0]); }
void HSD_JObjAnimAll(HSD_JObj* j) { T(12); T(j == &SKEL); }
void ftAnim_80070108(Fighter* f, Fighter_Part p, float a, float b, HSD_Joint* j) { T(13); T(f == &F); T(p); T(B(a)); T(B(b)); T(j == &JPOSE); TJ(j); }
void ftAnim_8006FE9C(Fighter* f, Fighter_Part p, float a, float b) { T(14); T(f == &F); T(p); T(B(a)); T(B(b)); }
void ftAnim_8006FF74(Fighter* f, Fighter_Part p) { T(15); T(f == &F); T(p); }
void ftAnim_80070010(Fighter* f, Fighter_Part p, float a, float b, HSD_Joint* j) { T(16); T(f == &F); T(p); T(B(a)); T(B(b)); T(j == &JPOSE); TJ(j); }
void ftAnim_8006FA58(Fighter* f, Fighter_Part p, HSD_Joint* j) { T(17); T(f == &F); T(p); T(j == &JPOSE); TJ(j); }
u32 Player_GetUnk45(s32 slot) { T(18); T(slot); return SRV[11] % 4; }
void* efSync_Spawn(s32 id, HSD_GObj* g, ...)
{
    __builtin_va_list ap;
    HSD_JObj* j;
    int c;
    __builtin_va_start(ap, g);
    j = __builtin_va_arg(ap, HSD_JObj*);
    c = __builtin_va_arg(ap, int);
    __builtin_va_end(ap);
    T(19); T(id); T(g == &GO); T(j - JO); T(c);
    return NULL;
}
void ftColl_8007B1B8(Fighter_GObj* g, ShieldDesc* s, HSD_GObjEvent cb)
{
    AbsorbDesc* a = (AbsorbDesc*) s;
    T(20); T(g == &GO); T(a->x0_bone_id); T(B(a->x4_offset.x)); T(B(a->x4_offset.y)); T(B(a->x4_offset.z)); T(B(a->x10_size)); T(cb == ftCo_80092E50);
}
void ft_PlaySFX(Fighter* f, enum_t id, u8 a, u8 b) { T(21); T(f == &F); T(id); T(a); T(b); }
int Geno_DefineTemplateSkeleton(int kind) { T(22); T(kind); return SR(12); }
int Geno_DefineBaseKind(int kind) { T(23); T(kind); return SR(13) ? Ft_Kind_Yoshi : (SR(14) ? Ft_Kind_Mars : 5); }
void ftYs_Init_8012BECC(Fighter_GObj* g) { T(24); T(g == &GO); }
void ftYs_Shield_8012C1D4(Fighter_GObj* g) { T(25); T(g == &GO); }
void ftYs_Shield_8012C49C(Fighter_GObj* g) { T(26); T(g == &GO); }
void ftYs_Shield_8012C600(Fighter_GObj* g, bool b) { T(27); T(g == &GO); T(b); }
void ftYs_Shield_8012C850(Fighter_GObj* g) { T(28); T(g == &GO); }
void ftYs_Shield_8012C914(Fighter_GObj* g) { T(29); T(g == &GO); }
void ftParts_80074B0C(Fighter_GObj* g, int m, int v) { T(30); T(g == &GO); T(m); T(v); }
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(31); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
void ftAnim_8006EBA4(Fighter_GObj* g) { T(32); T(g == &GO); }
void pl_8003E0E8(int a, int b) { T(33); T(a); T(b); }
void pl_8003E114(int a, int b, float v) { T(34); T(a); T(b); T(B(v)); }
void ftCo_80098B20(Fighter_GObj* g) { T(35); T(g == &GO); }
bool ftCo_8009515C(Fighter_GObj* g) { T(36); return SR(16); }
bool ftCo_8009980C(Fighter_GObj* g) { T(37); return SR(17); }
bool ftCo_8009917C(Fighter_GObj* g) { T(38); return SR(18); }
bool ftCo_800D8B9C(Fighter_GObj* g) { T(39); return SR(19); }
bool ftCo_Catch_CheckInput(Fighter_GObj* g) { T(40); return SR(20); }
bool ftCo_800CB024(Fighter_GObj* g) { T(41); return SR(21); }
bool ftCo_8009A080(Fighter_GObj* g) { T(42); return SR(22); }
void ft_80084F3C(Fighter_GObj* g) { T(43); T(g == &GO); }
void ftColl_8007AEE0(Fighter_GObj* g) { T(44); T(g == &GO); }
void ftColl_8007AEF8(Fighter_GObj* g) { T(45); T(g == &GO); }
void ft_800845B4(Fighter_GObj* g) { T(46); T(g == &GO); }
void ft_80084104(Fighter_GObj* g) { T(47); T(g == &GO); }
bool ftAnim_IsFramesRemaining(Fighter_GObj* g) { T(48); return SR(23); }
void ft_8008A2BC(Fighter_GObj* g) { T(49); T(g == &GO); }
bool ftCo_SpecialS_CheckInput(Fighter_GObj* g) { T(50); return SR(24); }
bool ftCo_Attack100_CheckInput(Fighter_GObj* g) { T(51); return SR(25); }
bool ftCo_800D6824(Fighter_GObj* g) { T(52); return SR(26); }
bool ftCo_800D68C0(Fighter_GObj* g) { T(53); return SR(27); }
bool ftCo_AttackS4_CheckInput(Fighter_GObj* g) { T(54); return SR(28); }
bool ftCo_AttackHi4_CheckInput(Fighter_GObj* g) { T(55); return SR(29); }
bool ftCo_AttackLw4_CheckInput(Fighter_GObj* g) { T(56); return SR(30); }
bool ftCo_AttackS3_CheckInput(Fighter_GObj* g) { T(57); return SR(31); }
bool ftCo_AttackHi3_CheckInput(Fighter_GObj* g) { T(58); return SR(32); }
bool ftCo_AttackLw3_CheckInput(Fighter_GObj* g) { T(59); return SR(33); }
bool ftCo_Attack1_CheckInput(Fighter_GObj* g) { T(60); return SR(34); }
void ftCo_800C36DC(Fighter_GObj* g) { T(61); T(g == &GO); }
void ftAnim_SetAnimRate(Fighter_GObj* g, float v) { T(62); T(g == &GO); T(B(v)); }
float lbGetJObjEndFrame(HSD_JObj* j) { T(63); T(j - JO); return ENDF; }
int Replay_UcfVersion(int port) { T(64); T(port); return SR(35) ? 84 : 83; }
int ftUcf_PadRaw(int port, int which_y, int back) { T(65); T(port); T(which_y); T(back); return (int) (SRV[12 + back] % 200) - 100; }
void ftColl_CreateReflectHit(Fighter_GObj* g, ReflectDesc* r, HSD_GObjEvent cb)
{
    T(66); T(g == &GO); T(r->x0_bone_id); T(r->x4_max_damage); T(B(r->x8_offset.x)); T(B(r->x8_offset.y)); T(B(r->x8_offset.z));
    T(B(r->x14_size)); T(B(r->x18_damage_mul)); T(B(r->x1C_speed_mul)); T(r->x20_behavior); T(cb == ftCo_80093790);
}
void ftCommon_8007DB24(Fighter_GObj* g) { T(67); T(g == &GO); }
void lb_8000B1CC(HSD_JObj* j, Vec3* p0, Vec3* p1) { T(68); T(j - JO); T(p0 == NULL); T(p1 == &POS); p1->x = 1; p1->y = 2; p1->z = 3; }

static u32 state_hash(void)
{
    int i;
    H = 0x811C9DC5u;
    HM(B(F.mv.co.guard.x0)); HM(B(F.mv.co.guard.x4)); HM(B(F.mv.co.guard.x8)); HM(F.mv.co.guard.xC);
    HM(B(F.mv.co.guard.x10)); HM(B(F.mv.co.guard.x14)); HM(B(F.mv.co.guard.x18)); HM(F.mv.co.guard.x1C);
    HM(F.mv.co.guard.x20); HM(F.mv.co.guard.x24); HM(B(F.mv.co.guard.x2C));
    HM(B(F.lightshield_amount)); HM(B(F.shield_health)); HM(B(F.gr_vel)); HM(B(F.cur_pos.x)); HM(B(F.cur_pos.y));
    HM(F.active_timer.trigger); HM(F.active_timer.lstick.x);
    HM(F.reflecting); HM(F.x221B_b0); HM(F.x221A_b7); HM(F.x2219_b0); HM(F.x221C_b1); HM(F.x221C_b2); HM(F.x221C_b3);
    HM(F.hitlag_cb == ftCo_80093240); HM(F.post_hitlag_cb == ftCo_800932DC);
    for (i = 0; i < 8; i++) {
        HM(B(JO[i].scale.x)); HM(B(JO[i].scale.y)); HM(B(JO[i].scale.z));
        HM(B(JO[i].translate.x)); HM(B(JO[i].translate.y)); HM(B(JO[i].translate.z));
    }
    HM(B(POS.x)); HM(B(POS.y)); HM(B(POS.z));
    return H;
}

static const int KINDS[4] = { 2, Ft_Kind_Yoshi, Ft_Kind_Mars, 40 };

int main(void)
{
    int op, k, i;
    tu_init();
    PT.parts_num = 8;
    for (i = 0; i < Ft_Kind_Max; i++) PTP[i] = &PT;
    ftPartsTable = PTP;
    Fighter_804D650C = COL;
    for (op = 0; op < 54; op++) {
        for (k = 0; k < 1000; k++) {
            u32 ret = 0;
            tu_case(op, k);
            zero(JO, sizeof JO); zero(JO2, sizeof JO2); zero(OWN, sizeof OWN); zero(&JPOSE, sizeof JPOSE);
            zero(&POS, sizeof POS);
            for (i = 0; i < 64; i++) COL[i] = rnd() & 0xFF;
            for (i = 0; i < 8; i++) {
                PARTS[i].joint = &JO[i];
                PARTS[i].x4_jobj2 = (rnd() & 3) ? &JO2[i] : NULL;
                JO[i].flags = rnd() & (JOBJ_MTX_INDEP_SRT | JOBJ_USER_DEF_MTX | JOBJ_MTX_DIRTY);
                JO[i].scale.x = rg();
                JO2[i].rotate.x = rg(); JO2[i].rotate.y = rg(); JO2[i].rotate.z = rg();
                JO2[i].scale.y = rg(); JO2[i].translate.z = rg();
            }
            for (i = 0; i < 3; i++) {
                OWN[i].flags = rnd(); OWN[i].rotation.x = rg(); OWN[i].scale.z = rg(); OWN[i].position.y = rg();
            }
            OWN[0].child = &OWN[1]; OWN[1].next = &OWN[2];
            if (rnd() & 1) OWN[2].child = NULL;
            JPOSE.flags = rnd(); JPOSE.rotation.y = rg();
            JTAB[2] = &JPOSE; FD20.x0 = JTAB; FD.x20 = &FD20;
            FD8.x11 = rnd() % 4; FD.x8 = &FD8;
            CJ.child = &CJC;
            MS[ftCo_MS_Guard].anim_id = rnd() % 50;
            GO.hsd_obj = &JO[7];
            MV = rw(); ENDF = rw();

            F.ft_data = &FD; F.parts = PARTS; F.x8AC_animSkeleton = &SKEL; F.x108_costume_joint = &CJ;
            F.x1C_actionStateList = MS;
            F.kind = KINDS[rnd() % 4];
            F.player_id = rnd() % 4; F.x618_player_id = rnd() % 4; F.is_sub_fighter = rnd() & 1;
            F.input.held_buttons[0] = rnd(); F.input.pressed_buttons = rnd();
            F.input.triggers[0] = rg();
            F.input.lstick[0].x = rg(); F.input.lstick[0].y = rg(); F.input.lstick[1].x = rg();
            F.facing_dir = rg();
            F.active_timer.trigger = rnd() % 8; F.active_timer.lstick.x = rnd() % 8;
            F.active_sticky.lstick.x = rnd() % 3;
            F.shield_health = (rnd() & 3) ? rw() : 0; F.lightshield_amount = rg();
            F.x2E8 = (float) (rnd() % 5);
            F.mv.co.guard.x0 = (rnd() & 1) ? (float) (rnd() % 6) : rg();
            F.mv.co.guard.x4 = (rnd() & 1) ? 0 : rg();
            F.mv.co.guard.x8 = rw(); F.mv.co.guard.xC = rnd() & 1; F.mv.co.guard.x10 = rg();
            F.mv.co.guard.x14 = rg(); F.mv.co.guard.x18 = rg(); F.mv.co.guard.x1C = rnd() % 3;
            F.mv.co.guard.x20 = rnd(); F.mv.co.guard.x24 = rnd(); F.mv.co.guard.x2C = rg();
            F.x19A4 = rnd() % 30; F.x19B0 = (rnd() & 1) ? 10 : 3;
            F.specialn_facing_dir = rg();
            F.allow_sdi = rnd() & 1; F.ground_or_air = rnd() & 1;
            F.reflecting = rnd() & 1; F.x221B_b0 = rnd() & 1;
            F.x221C_b1 = rnd() & 1; F.x221C_b2 = rnd() & 1; F.x221C_b3 = rnd() & 1;
            F.co_attrs.initial_shield_size = rg();
            F.cur_anim_frame = rg(); F.cur_pos.x = rw(); F.cur_pos.y = rw();
            F.coll_data.floor.normal.x = rg(); F.coll_data.floor.normal.y = rg();

            CD.powershield_input_window = rnd() % 6; CD.x68 = rw(); CD.x44C = rg();
            CD.x260_startShieldHealth = rw(); CD.x264 = rg(); CD.x2D4 = rg(); CD.x2D8 = rg();
            CD.x2F4 = (float) (rnd() % 200) + rg(); CD.x268 = rg() * 3;
            CD.analog_shoulder_deadzone = rg(); CD.x278 = rg(); CD.x2EC = rg(); CD.x2F0 = rg();
            CD.x28C = rw(); CD.x290 = rg(); CD.x294 = rg(); CD.x298 = rg(); CD.x2BC = rg();
            CD.x2E4 = rg(); CD.x2E8 = rg();
            CD.sdi_min_stick_mag = rg(); CD.sdi_stick_window = rnd() % 8; CD.sdi_pos_scale = rg();
            CD.x4BC = rg(); CD.x4C0 = rw();
            CD.x2A4 = rg(); CD.x2A8 = rg(); CD.x2AC = rg(); CD.x2B0 = rg(); CD.x2B4 = rg(); CD.x2B8 = rnd() % 10;

            switch (op) {
            case 0: ret = ftCo_80091A2C(&GO); break;
            case 1: ret = ftCo_80091A4C(&GO); break;
            case 2: ret = ftCo_80091AD8(&GO, (int) rnd()); break;
            case 3: ftCo_80091B90(&GO, (int) rnd()); break;
            case 4: ftCo_80091B9C(&GO); break;
            case 5: ftCo_80091BC4(&F); break;
            case 6: ftCo_80091D58(&F); break;
            case 7: ftCo_80091E78(&GO, rg()); break;
            case 8: { int id = rnd() % 2000; ftCo_80092158(&GO, id, &JO[rnd() % 8]); break; }
            case 9: ftCo_800921DC(&GO); break;
            case 10: ftCo_800923B4(&GO); break;
            case 11: ftCo_80092450(&GO); break;
            case 12: ftCo_800924C0(&GO); break;
            case 13: ret = ftCo_800925A4(&GO); break;
            case 14: ftCo_GuardOn_Anim(&GO); break;
            case 15: ftCo_GuardOn_IASA(&GO); break;
            case 16: ftCo_GuardOn_Phys(&GO); break;
            case 17: ftCo_GuardOn_Coll(&GO); break;
            case 18: ftCo_800928CC(&GO); break;
            case 19: ftCo_80092908(&GO); break;
            case 20: ftCo_Guard_Anim(&GO); break;
            case 21: ftCo_Guard_IASA(&GO); break;
            case 22: ftCo_Guard_Phys(&GO); break;
            case 23: ftCo_Guard_Coll(&GO); break;
            case 24: ftCo_80092BCC(&GO); break;
            case 25: ftCo_80092BE8(&GO); break;
            case 26: ftCo_80092C54(&GO); break;
            case 27: ftCo_GuardOff_Anim(&GO); break;
            case 28: ftCo_GuardOff_IASA(&GO); break;
            case 29: ftCo_GuardOff_Phys(&GO); break;
            case 30: ftCo_GuardOff_Coll(&GO); break;
            case 31: ftCo_80092E50(&GO); break;
            case 32: { int a = rnd() % 40; ret = B(ftCo_80092ED8(a, rg())); break; }
            case 33: ftCo_80092F2C(&GO, rnd() & 1); break;
            case 34: ftCo_80093240(&GO); break;
            case 35: ftCo_800932DC(&GO); break;
            case 36: ftCo_GuardSetOff_Anim(&GO); break;
            case 37: ftCo_GuardSetOff_IASA(&GO); break;
            case 38: ftCo_GuardSetOff_Phys(&GO); break;
            case 39: ftCo_GuardSetOff_Coll(&GO); break;
            case 40: ret = ftCo_80093694(&GO); break;
            case 41: ftCo_8009370C(&GO, ftCo_80093790); break;
            case 42: ftCo_80093790(&GO); break;
            case 43: ftCo_80093850(&GO); break;
            case 44: ftCo_8009388C(&GO); break;
            case 45: ftCo_800939B4(&GO); break;
            case 46: ftCo_80093A50(&GO); break;
            case 47: ftCo_80093BC0(&GO); break;
            case 48: ftCo_GuardReflect_Anim(&GO); break;
            case 49: ftCo_GuardReflect_IASA(&GO); break;
            case 50: ftCo_GuardReflect_Phys(&GO); break;
            case 51: ftCo_GuardReflect_Coll(&GO); break;
            case 52: ret = B(ftCo_80094098(&GO, &POS.x)); break;
            case 53: ftCo_80094138(&F); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
