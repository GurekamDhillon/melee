/* TU: src/melee/ft/kinds/ftCommon/ftCo_DownBound.c */
/* C oracle for src/tu/ftCo_DownBound.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

static FighterBone PARTS[8];
static HSD_JObj JO[2];
static float ATAN;
static int SFXA[4];

Fighter_Part ftParts_GetBoneIndex(Fighter* f, Fighter_Part p) { T(1); T(f == &F); T(p); return (Fighter_Part) (SRV[5] % 2 + 3); }
void HSD_JObjSetupMatrixSub(HSD_JObj* j) { T(2); T(j == &JO[0]); T(j == &JO[1]); }
float atan2f(float y, float x) { T(3); T(B(y)); T(B(x)); return ATAN; }
void efAsync_Spawn(HSD_GObj* g, void* q, u32 kind, u32 id, HSD_JObj* j, ...)
{
    __builtin_va_list ap;
    float* p;
    __builtin_va_start(ap, j);
    p = __builtin_va_arg(ap, float*);
    __builtin_va_end(ap);
    T(4); T(g == &GO); T(q == &F.x60C); T(kind); T(id); T(j == &JO[0]); T(j == &JO[1]); T(B(*p));
}
void ft_PlaySFX(Fighter* f, enum_t id, u8 a, u8 b) { T(5); T(f == &F); T(id); T(a); T(b); }
bool ft_80084C74(Fighter_GObj* g, int* sfx, int* a2, int* a3)
{
    int i;
    T(6); T(g == &GO); T(*a2);
    for (i = 0; i < 4; i++) sfx[i] = SR(10 + i) ? -1 : (int) (SRV[6 + i] % 50);
    if (SR(16)) *a2 = SR(17);
    *a3 = SR(15) ? -1 : (int) (1000 + SRV[10] % 50);
    return SR(14);
}
void ftCo_8009F834(Fighter_GObj* g, int id, Fighter_Part p, int a, int b, Vec3* v1, Vec3* v0, float f)
{ T(7); T(g == &GO); T(id); T(p); T(a); T(b); T(B(v1->x)); T(B(v1->y)); T(B(v1->z)); T(B(v0->x)); T(B(v0->y)); T(B(v0->z)); T(B(f)); }
void Camera_RequestQuake(CmQuakeKind k, Vec* v) { T(8); T(k); T(v == &F.cur_pos); }
void ftCommon_8007EBAC(Fighter* f, u32 a, u32 b) { T(9); T(f == &F); T(a); T(b); }
void ftCommon_8007D7FC(Fighter* f) { T(10); T(f == &F); }
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(11); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
void ftCommon_8007E2F4(Fighter* f, s16 v) { T(12); T(f == &F); T(v); }
void ftCommon_SetGroundedKnockbackIfLanded(Fighter* f) { T(13); T(f == &F); }
void ft_8008A2BC(Fighter_GObj* g) { T(14); T(g == &GO); }
void ftSb_Init_8014FBA4(HSD_GObj* g) { T(15); T(g == &GO); }
bool ftAnim_IsFramesRemaining(Fighter_GObj* g) { T(16); return SR(3); }
bool ftCo_80098400(Fighter_GObj* g) { T(17); return SR(4); }
bool ftCo_Down_CheckInput(Fighter_GObj* g) { T(18); return SR(5); }
void ft_80084F3C(Fighter_GObj* g) { T(19); T(g == &GO); }
GroundOrAir ft_80082708(Fighter_GObj* g) { T(20); return SR(6); }
void ftCo_Fall_Enter(Fighter_GObj* g) { T(21); T(g == &GO); }
bool ftCo_800C7CA0(Fighter_GObj* g) { T(22); return SR(7); }
void ftAnim_8006EBA4(Fighter_GObj* g) { T(23); T(g == &GO); }
void ftCo_80098160(Fighter_GObj* g, FtMotionId m) { T(24); T(g == &GO); T(m); }
bool ftCo_800984D4(Fighter_GObj* g) { T(25); return SR(8); }
bool ftCo_800980BC(Fighter_GObj* g) { T(26); return SR(9); }
void ft_80083F88(Fighter_GObj* g) { T(27); T(g == &GO); }

static u32 state_hash(void)
{
    H = 0x811C9DC5u;
    HM(F.x67C); HM(F.x67D); HM(B(F.mv.co.downwait.x0)); HM(F.mv.co.downspot.x4);
    return H;
}

int main(void)
{
    static const FtMotionId ms[6] = { ftCo_MS_DownBoundU, ftCo_MS_DownSpotU, ftCo_MS_DownWaitU, ftCo_MS_DownDamageU, ftCo_MS_DownWaitD, 0 };
    static const u32 jf[4] = { 0, JOBJ_MTX_DIRTY, JOBJ_USER_DEF_MTX, JOBJ_MTX_DIRTY | JOBJ_USER_DEF_MTX };
    int op, k, i, r, c;
    tu_init();
    for (op = 0; op < 21; op++) {
        for (k = 0; k < 3000; k++) {
            u32 ret = 0;
            tu_case(op, k);
            zero(PARTS, sizeof PARTS); zero(JO, sizeof JO);
            for (i = 0; i < 8; i++) PARTS[i].joint = (rnd() & 7) ? &JO[i & 1] : NULL;
            PARTS[3].joint = &JO[0]; PARTS[4].joint = &JO[1];
            for (i = 0; i < 2; i++) {
                JO[i].flags = jf[rnd() % 4];
                for (r = 0; r < 3; r++) for (c = 0; c < 4; c++) JO[i].mtx[r][c] = rg();
            }
            ATAN = rg();
            for (i = 0; i < 4; i++) SFXA[i] = rnd() % 30;
            CD.x1F0 = rw(); CD.x1F4 = rw(); CD.x424 = rw();
            F.parts = PARTS;
            F.x2226_b0 = rnd() & 1; F.x2226_b1 = rnd() & 1; F.x2224_b2 = rnd() & 1;
            F.is_sandbag = rnd() & 1;
            F.ground_or_air = rnd() & 1;
            F.motion_id = ms[rnd() % 6];
            F.self_vel.x = rw(); F.self_vel.y = rw(); F.x8c_kb_vel.x = rw(); F.x8c_kb_vel.y = rw();
            F.co_attrs.weight = rw();
            F.coll_data.floor.normal.x = rg(); F.coll_data.floor.normal.y = rg();
            F.facing_dir = rg();
            F.x67C = rnd() % 256; F.x67D = rnd() % 256;
            F.mv.co.downwait.x0 = rw();
            F.mv.co.downspot.x4 = rnd() % 256;
            switch (op) {
            case 0: ret = ftCo_80097570(&GO); break;
            case 1: ftCo_80097630(&F, SFXA, rw()); break;
            case 2: ftCo_800976A4(&GO); break;
            case 3: ftCo_800978D4(&GO); break;
            case 4: ftCo_8009794C(&GO); break;
            case 5: ftCo_80097AF4(&GO); break;
            case 6: ftCo_80097D40(&GO); break;
            case 7: ftCo_80097D88(&GO); break;
            case 8: ftCo_DownBound_Anim(&GO); break;
            case 9: ftCo_DownBound_IASA(&GO); break;
            case 10: ftCo_DownBound_Phys(&GO); break;
            case 11: ftCo_DownBound_Coll(&GO); break;
            case 12: ftCo_80097E8C(&GO); break;
            case 13: ftCo_80097F38(&GO); break;
            case 14: ftCo_DownWait_Anim(&GO); break;
            case 15: ftCo_DownWait_IASA(&GO); break;
            case 16: ftCo_DownWait_Phys(&GO); break;
            case 17: ftCo_DownWait_Coll(&GO); break;
            case 18: { float p; ftCo_800978D4_inline(&GO, &p); ret = B(p); break; }
            case 19: { float d = rw(); ftCo_800976A4_inline(&F, &d); ret = B(d); break; }
            case 20: inlineA1(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
