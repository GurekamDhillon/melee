/* TU: src/melee/ft/kinds/ftCommon/ftCo_Bury.c */
/* C oracle for src/tu/ftCo_Bury.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

StageInfo stage_info;
static HSD_JObj JO[5];
static FighterBone PARTS[4];
static DynamicsDesc DD;
static u32 jid(HSD_JObj* j) { int i; for (i = 0; i < 5; i++) if (j == &JO[i]) return i + 1; return j == NULL ? 0 : 99; }
#define VSTUB(name, id) void name(Fighter_GObj* g) { T(id); T(g == &GO); }
#define BSTUB(name, id) bool name(Fighter_GObj* g) { T(id); T(g == &GO); return SR(id); }
VSTUB(ftCommon_8007DB58, 1)
VSTUB(ftCo_8009750C, 2)
VSTUB(ftCo_800DD168, 3)
VSTUB(ftCo_8009EB18, 4)
float ftColl_800765F0(Fighter* f, Fighter_GObj* v, float x) { T(5); T(f == &F); T(v == NULL); T(B(x)); return rw(); }
bool ftColl_80076640(Fighter* f, float* x) { T(6); T(f == &F); T(B(*x)); *x = rg(); return SR(6); }
void ftColl_80076764(int a, enum_t b, Fighter_GObj* g, DynamicsDesc* d, Fighter* f, FighterHurtCapsule* h)
{ T(7); T(a); T(b); T(g == &GO2); T(g == NULL); T(d == &DD); T(f == &F); T(h == &F.hurt_capsules[0]); }
void lbColl_80008D30(HitCapsule* h, lbColl_80008D30_arg1* a) { T(8); T(a == (void*) &DD); }
void ftColl_80078384(Fighter* f, FighterHurtCapsule* h, HitCapsule* hit) { T(9); T(f == &F); T(h == &F.hurt_capsules[0]); }
void pl_8003EC30(int s, int sub, int k, float x) { T(10); T(s); T(sub); T(k); T(B(x)); }
DynamicsDesc* Ground_801C5700(int i) { T(11); T(i); return SR(11 + (i & 7) * 16) ? &DD : NULL; }
void ftCo_8008DCE0(Fighter_GObj* g, int a, float d) { T(12); T(g == &GO); T(a); T(B(d)); }
void ftCommon_8007D7FC(Fighter* f) { T(13); T(f == &F); }
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(14); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
VSTUB(ftCommon_8007E2FC, 15)
void ftCommon_InitGrab(Fighter* f, bool a, float v) { T(16); T(f == &F); T(a); T(B(v)); }
int Player_GetHandicap(int s) { T(17); T(s); return (int) (rnd() % 10); }
s32 Player_80033BB8(int s) { T(18); T(s); return (s32) (rnd() % 5) - 1; }
void ftCommon_8007E2F4(Fighter* f, s16 v) { T(19); T(f == &F); T(v); }
void ft_80084CB0(Fighter* f, ftCollisionBox* b) { T(20); T(f == &F); T(b == &F.mv.co.bury.coll_box); b->bottom = rg(); b->left.y = rw(); }
void lb_8000B1CC(HSD_JObj* j, Vec3* p0, Vec3* p1) { T(21); T(jid(j)); T(p0 == NULL); p1->x = rg(); p1->y = rw(); p1->z = rg(); }
Fighter_Part ftParts_GetBoneIndex(Fighter* f, Fighter_Part p) { T(22); T(f == &F); T(p); return (Fighter_Part) (rnd() % 4); }
void* efSync_Spawn(s32 id, HSD_GObj* g, ...)
{
    __builtin_va_list ap;
    Vec3* p;
    float* s;
    __builtin_va_start(ap, g);
    p = __builtin_va_arg(ap, Vec3*);
    s = __builtin_va_arg(ap, float*);
    __builtin_va_end(ap);
    T(23); T(id); T(g == &GO); T(p == &F.cur_pos); T(s == &F.x34_scale.y);
    return NULL;
}
bool ftCommon_GrabMash(Fighter* f, float v) { T(24); T(f == &F); T(B(v)); f->grab_timer = rg(); return SR(24); }
bool mpLib_80054ED8(int l) { T(25); T(l); return SR(25); }
Vec3* mpLineGetNormal(int l, Vec3* n) { T(26); T(l); n->x = rg(); n->y = rg(); n->z = rg(); return n; }
float atan2f(float y, float x) { T(27); T(B(y)); T(B(x)); return rw(); }
void HSD_JObjSetMtxDirtySub(HSD_JObj* j) { T(28); T(jid(j)); }
bool mpGetSpeed(int l, Vec3* p, Vec3* s) { T(29); T(l); T(p == &F.mv.co.bury.translate); s->x = rg(); s->y = rw(); s->z = rg(); return SR(29); }
void PSVECAdd(Vec* a, Vec* b, Vec* c)
{ T(30); T(a == &F.mv.co.bury.translate); T(c == a); c->x = a->x + b->x; c->y = a->y + b->y; c->z = a->z + b->z; }
bool ft_80082888(Fighter_GObj* g, ftCollisionBox* b) { T(31); T(g == &GO); T(b == &F.mv.co.bury.coll_box); return SR(31); }
BSTUB(ftCo_800C5240, 32)
VSTUB(ftCo_800C5A98, 33)
void ftCommon_8007D5D4(Fighter* f) { T(34); T(f == &F); }
void ftColl_8007B7A4(Fighter_GObj* g, int v) { T(35); T(g == &GO); T(v); }
BSTUB(ftAnim_IsFramesRemaining, 36)
VSTUB(ftCo_Fall_Enter, 37)
BSTUB(ftCo_SpecialAir_CheckInput, 38)
bool ftCo_80095328(Fighter_GObj* g, bool* b) { T(39); T(g == &GO); T(b == NULL); return SR(39); }
BSTUB(ftCo_800D7100, 40)
BSTUB(ftCo_800C3B10, 41)
BSTUB(ftCo_80099A58, 42)
BSTUB(ftCo_AttackAir_CheckItemThrowInput, 43)
BSTUB(ftCo_800D705C, 44)
BSTUB(ftCo_800CB870, 45)
void ftCommon_Fall(Fighter* f, float a, float b) { T(46); T(f == &F); T(B(a)); T(B(b)); }
void ftCommon_CalcSelfAccel_Drift(Fighter* f) { T(47); T(f == &F); }
void ft_80082D40(Fighter_GObj* g, float v) { T(48); T(g == &GO); T(B(v)); }
VSTUB(ftCo_AirCatchHit_Coll, 49)

static const int KINDS[8] = { 4, 8, 30, 32, 39, 0, 1, 2 };

static u32 state_hash(void)
{
    int i;
    H = 0x811C9DC5u;
    HM(F.bury_timer_1); HM(F.bury_timer_2); HM(F.bury_grkind);
    HM(F.x221D_b5); HM(F.x2220_b3); HM(F.x2224_b4); HM(F.x2219_b0);
    HM(F.mv.co.bury.x0); HM(B(F.mv.co.bury.x1C)); HM(F.mv.co.bury.x20);
    HM(B(F.mv.co.bury.coll_box.top)); HM(B(F.mv.co.bury.coll_box.bottom));
    HM(B(F.mv.co.bury.coll_box.left.x)); HM(B(F.mv.co.bury.coll_box.left.y));
    HM(B(F.mv.co.bury.coll_box.right.x)); HM(B(F.mv.co.bury.coll_box.right.y));
    HM(B(F.mv.co.bury.translate.x)); HM(B(F.mv.co.bury.translate.y)); HM(B(F.mv.co.bury.translate.z));
    HM(B(F.self_vel.x)); HM(B(F.self_vel.y)); HM(B(F.mv.co.buryjump.x0)); HM(B(F.grab_timer));
    for (i = 0; i < 5; i++) {
        HM(B(JO[i].rotate.z)); HM(B(JO[i].translate.x)); HM(B(JO[i].translate.y)); HM(B(JO[i].translate.z));
    }
    return H;
}

#define NOPS 24
#define NCASE 1000

int main(void)
{
    int op, k, i;
    tu_init();
    for (i = 0; i < 4; i++) PARTS[i].joint = &JO[i];
    for (op = 0; op < NOPS; op++) {
        for (k = 0; k < NCASE; k++) {
            u32 ret = 0;
            int arg;
            tu_case(op, k);
            zero(JO, sizeof JO);
            for (i = 0; i < 5; i++) JO[i].flags = rnd() & (JOBJ_MTX_DIRTY | JOBJ_USER_DEF_MTX | JOBJ_MTX_INDEP_SRT);
            GO2.hsd_obj = &JO[4];
            F.parts = PARTS;
            DD.count = rnd() % 100;
            stage_info.grkind = KINDS[rnd() % 8];
            CD.bury_timer_unk1 = rnd(); CD.bury_timer_unk2 = rnd(); CD.bury_timer_unk3 = rnd();
            CD.x5F4 = (int) (rnd() % 5);
            CD.x5F8 = rw(); CD.x5FC = rw(); CD.x600 = rw(); CD.x604 = rw(); CD.x608 = rw();
            CD.x60C = rw(); CD.x610 = rg(); CD.x614 = rg(); CD.x618 = rw(); CD.x61C = rg();
            CD.x620 = (int) rnd();
            F.bury_timer_1 = rnd() % 3; F.bury_timer_2 = rnd() % 3; F.bury_grkind = KINDS[rnd() % 8];
            F.motion_id = (rnd() & 1) ? ftCo_MS_BarrelWait : (FtMotionId) (rnd() % 400);
            F.x1A6A = rnd() & 0xFFFF;
            F.x2224_b2 = rnd() & 1; F.x2227_b6 = rnd() & 1; F.is_sub_fighter = rnd() & 1;
            F.dmg.x1860_element = rnd() % 12; F.ground_or_air = rnd() & 1;
            F.dmg.x1830_percent = rw();
            F.player_id = rnd() % 4;
            F.coll_data.env_flags = (rnd() & 1) ? 0 : (int) (rnd() & 0x1FFFF);
            F.coll_data.floor.index = rnd() % 8; F.coll_data.ceiling.index = rnd() % 8;
            F.coll_data.right_facing_wall.index = rnd() % 8; F.coll_data.left_facing_wall.index = rnd() % 8;
            F.grab_timer = rw();
            F.mv.co.bury.x0 = (int) (rnd() % 3);
            F.mv.co.bury.x20 = rnd() % 8;
            F.mv.co.bury.coll_box.top = rg(); F.mv.co.bury.coll_box.bottom = rw();
            F.mv.co.bury.coll_box.left.x = rg(); F.mv.co.bury.coll_box.left.y = rw();
            F.mv.co.bury.coll_box.right.x = rg(); F.mv.co.bury.coll_box.right.y = rw();
            F.mv.co.bury.x1C = rg();
            F.mv.co.bury.translate.x = rw(); F.mv.co.bury.translate.y = rg(); F.mv.co.bury.translate.z = rg();
            if (rnd() & 1) F.mv.ca.specials.grav = rg();
            if (rnd() & 1) F.mv.co.buryjump.x0 = rw();
            F.self_vel.x = rg(); F.self_vel.y = rg();
            F.co_attrs.gravity = rg(); F.co_attrs.terminal_velocity = rw();
            F.cur_pos.x = rw(); F.cur_pos.y = rg(); F.cur_pos.z = rg();
            arg = (int) (rnd() % 5);
            switch (op) {
            case 0: ftCo_800C0874(&GO, NULL, arg); break;
            case 1: ftCo_800C08A0(&GO, (rnd() & 1) ? &GO2 : NULL, &DD, arg); break;
            case 2: ftCo_800C09B4(&GO); break;
            case 3: ret = ftCo_800C0A28(&GO, NULL, arg); break;
            case 4: ftCo_800C0A98(&GO); break;
            case 5: ftCo_800C0B20(&GO); break;
            case 6: ret = ftCo_800C0C88(arg); break;
            case 7: ret = ftCo_800C0CB8(&GO); break;
            case 8: ftCo_800C0D0C(&GO); break;
            case 9: ftCo_Bury_Anim(&GO); break;
            case 10: ftCo_Bury_IASA(&GO); break;
            case 11: ftCo_800C0FCC(&GO2, &GO); break;
            case 12: ftCo_Bury_Phys(&GO); break;
            case 13: ftCo_Bury_Coll(&GO); break;
            case 14: ftCo_800C124C(&GO); break;
            case 15: ftCo_BuryWait_Anim(&GO); break;
            case 16: ftCo_BuryWait_IASA(&GO); break;
            case 17: ftCo_BuryWait_Phys(&GO); break;
            case 18: ftCo_BuryWait_Coll(&GO); break;
            case 19: ftCo_800C13BC(&GO); break;
            case 20: ftCo_BuryJump_Anim(&GO); break;
            case 21: ftCo_BuryJump_IASA(&GO); break;
            case 22: ftCo_BuryJump_Phys(&GO); break;
            case 23: ftCo_BuryJump_Coll(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
