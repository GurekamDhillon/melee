/* C reference for src/ftcommon3.rs (port round 8): more of src/melee/ft/ftcommon.c (landing +
 * the TARGET_PC floor guard, nudge, bone-4 follow, accessory, item / knockback / parasol
 * callbacks, head and held items), plus the ftcommon.c functions they call that earlier rounds
 * ported (ClampGroundVel, UnlockECB.., 8007EBAC, 8007F8B4, 8007FDA0) and ft_0881.c's
 * ft_80088770 / ft_800887CC, all cut verbatim into ftcommon3_extract.inc by run.py.
 * Six fighters, a random fighter p-link list, random fields, one op per case. Callees outside
 * the set are stubs folding their arguments into the trace word g_tr; tests/ftcommon3_ref.rs
 * installs the same stubs as hooks. The HSD_JObj header inlines (GetTranslation,
 * SetTranslate, MtxIsDirty, SetScale) are redirected to stubs by macro after the headers, so
 * the extract calls the stubs. Shipping contraction (run.py): 8007DD7C / 8007DFD0 fuse two
 * sites each, 8007E3EC three. The driver itself has no a*b+c. */
#include "ft/types.h"
#include "ft/forward.h"
#include "ft/ftcommon.h"
#include "ft/fighter.h"
#include "ft/ft_081B.h"
#include "ft/ft_0877.h"
#include "ft/ft_0881.h"
#include "ft/ftlib.h"
#include "ft/ftparts.h"
#include "ft/kinds/ftCommon/ftCo_Damage.h"
#include "ft/kinds/ftCommon/ftCo_Fall.h"
#include <melee/ef/eflib.h>
#include <melee/gm/gm_unsplit.h>
#include <melee/it/it_26B1.h>
#include <melee/it/item.h>
#include <melee/it/kinds/it_2E5A.h>
#include <melee/it/kinds/itpeachparasol.h>
#include <melee/it/kinds/itsword.h>
#include <melee/lb/lbaudio_ax.h>
#include <melee/mp/mplib.h>
#include <melee/mp/types.h>
#include <melee/pl/player.h>
#include <melee/pl/plbonuslib.h>
#include <sysdolphin/baselib/gobj.h>
#include <sysdolphin/baselib/jobj.h>

#define FT_CAPS_MAX_JUMPS(fp) ((fp)->co_attrs.max_jumps)
#define NF 6

static u32 g_tr;
static void T(u32 v) { g_tr = g_tr * 31u + v; }
static u32 B(float f) { union { float f; u32 u; } c; c.f = f; return c.u; }

static Fighter F[NF];
static HSD_GObj GO[NF], IT[3];
static u64 JO[8][32];
static ftData FD[NF];
static ftCo_DatAttrs DA[NF];
static struct ftData_x8 X8[NF];
static FighterBone PB[NF][8];
static ftCommonData cd;
static HSD_GObj* plinks[64];
static MapCollData MAP;
static int g_map_null;
static int ITK[3], ITM[3];

static u32 E(void* g)
{
    int i;
    if (g == NULL) return 0;
    for (i = 0; i < NF; i++) if (g == &GO[i]) return 1u + (u32) i;
    for (i = 0; i < 3; i++) if (g == &IT[i]) return 10u + (u32) i;
    for (i = 0; i < 8; i++) if (g == (void*) JO[i]) return 30u + (u32) i;
    return 99;
}

void __assert(const char* file, u32 line, const char* cond) { (void) file; (void) line; (void) cond; for (;;) {} }
ftCommonData* p_ftCommonData;
HSD_GObj** HSD_GObjPLinkHead;
const Vec3 ftCo_803B74A0 = { 0 };

float ScriptGame_ModValue(int slot, int field, float base) { T(0x100u + (u32) slot); T((u32) field); return base + 1.0f; }
void Geno_GroundEdge(Fighter* fp, int landing) { T(0x110u + E(fp->gobj) * 2u + (u32) landing); }
void Script_GameEvent(int what, int a, int b, int c, int d) { T(0x120u + (u32) what); T((u32) a); T((u32) b); T((u32) c); T((u32) d); }
void OSReport(char* fmt, ...) { (void) fmt; T(0x130u); }
MapCollData* mpLib_8004D164(void) { T(0x140u); return g_map_null ? NULL : &MAP; }
bool ft_80084A18(Fighter_GObj* g) { T(0x150u + E(g)); return (g_tr >> 3) & 1; }
void ftCo_Fall_Enter(HSD_GObj* g) { T(0x160u + E(g)); }
void ft_8008A2BC(HSD_GObj* g) { T(0x170u + E(g)); }
bool un_803224DC(s32 spawn, f32 x, f32 kb) { T(0x180u + (u32) spawn); T(B(x)); T(B(kb)); return spawn & 1; }
void pl_8003FC44(int slot, int sub) { T(0x190u + (u32) slot * 2u + (u32) sub); }
void efLib_DestroyAll(HSD_GObj* g) { T(0x1A0u + E(g)); }
int lbAudioAx_80023870(int a, int b, int c, int d) { T(0x1B0u); T((u32) a); T((u32) b); T((u32) c); T((u32) d); return 0; }
Fighter_Part ftParts_GetBoneIndex(Fighter* fp, Fighter_Part i) { T(0x1C0u + (u32) i); return (Fighter_Part) (fp->player_id % 8); }
static void stub_GetTranslation(HSD_JObj* j, Vec3* v)
{
    T(0x1D0u + E(j));
    v->x = (float) E(j); v->y = -(float) (g_tr & 7); v->z = 0.5f;
}
static void stub_SetTranslate(HSD_JObj* j, Vec3* v) { T(0x1E0u + E(j)); T(B(v->x)); T(B(v->y)); T(B(v->z)); }
static bool stub_MtxIsDirty(HSD_JObj* j) { T(0x1F0u + E(j)); return (g_tr >> 2) & 1; }
static void stub_SetScale(HSD_JObj* j, Vec3* v) { T(0x200u + E(j)); T(B(v->x)); T(B(v->y)); T(B(v->z)); }
HSD_JObj* HSD_JObjLoadJoint(HSD_Joint* j) { T(0x210u + E(j)); return (HSD_JObj*) JO[g_tr % 8u]; }
void HSD_JObjAddAnimAll(HSD_JObj* j, HSD_AnimJoint* a, HSD_MatAnimJoint* m, HSD_ShapeAnimJoint* s)
{
    T(0x220u + E(j)); T(E(a)); T((m == NULL) + (s == NULL) * 2u);
}
void HSD_JObjReqAnimAll(HSD_JObj* j, f32 f) { T(0x230u + E(j)); T(B(f)); }
bool ftCo_800C5240(Fighter_GObj* g) { T(0x240u + E(g)); return (g_tr >> 1) & 1; }
void ftCo_800C5500(Fighter_GObj* g) { T(0x250u + E(g)); }
void pl_8003EA08(int slot, int sub) { T(0x260u + (u32) slot * 2u + (u32) sub); }

void (*ftData_OnItemDropExt[64])(HSD_GObj*, s32);
void (*ftData_OnItemDrop[64])(HSD_GObj*, s32);
void (*ftData_OnItemPickup[64])(HSD_GObj*, s32);
void (*ftData_OnItemInvisible[64])(HSD_GObj*);
void (*ftData_OnItemVisible[64])(HSD_GObj*);
void (*ftData_OnKnockbackEnter[64])(HSD_GObj*);
void (*ftData_OnKnockbackExit[64])(HSD_GObj*);
void (*ftData_UnkMotionStates2[64])(HSD_GObj*);
void (*ftData_UnkMotionStates1[64])(HSD_GObj*);
#define MEX3(name, n) \
    void name(int kind, void* g, void* a, void* v) { (void) v; T(0x270u + n * 16u + E(g)); T((u32) kind); T((u32) (intptr_t) a); }
#define MEX2(name, n) \
    void name(int kind, void* g, void* v) { (void) v; T(0x270u + n * 16u + E(g)); T((u32) kind); }
MEX3(Mex_OnItemDropExtDispatch, 0)
MEX3(Mex_OnItemDropDispatch, 1)
MEX3(Mex_OnItemPickup2Dispatch, 2)
MEX2(Mex_OnItemInvisibleDispatch, 3)
MEX2(Mex_OnItemVisibleDispatch, 4)
MEX2(Mex_OnKnockbackEnterDispatch, 5)
MEX2(Mex_OnKnockbackExitDispatch, 6)
MEX2(Mex_OnRemoveHeadItemDispatch, 7)
MEX2(Mex_OnApplyHeadItemDispatch, 8)

static int item_ix(void* g) { u32 e = E(g); return e >= 10 && e < 13 ? (int) (e - 10) : -1; }
ItemKind itGetKind(Item_GObj* g) { int i = item_ix(g); T(0x300u + E(g)); return i < 0 ? 0 : ITK[i]; }
s32 itGetMotionId(Item_GObj* g) { int i = item_ix(g); T(0x310u + E(g)); return i < 0 ? 0 : ITM[i]; }
s32 it_8028B08C(void* g, s32 s) { T(0x320u + E(g)); T((u32) s); return (int) (g_tr % 7u) - 2; }
int it_802BDA40(Item_GObj* g, int s) { T(0x330u + E(g)); T((u32) s); return (int) (g_tr % 9u) - 3; }
#define PT(name, ix) void name(HSD_GObj* g, float v) { T(0x340u + ix); T(E(g)); T(B(v)); }
PT(it_8028B718, 0) PT(it_8028B780, 1) PT(it_8028B7E8, 2) PT(it_8028B850, 3)
PT(it_8028B648, 4) PT(it_8028B6B0, 5) PT(it_8028B618, 6)
void it_802BDD40(Item_GObj* g, f32 v) { T(0x350u + 4u); T(E(g)); T(B(v)); }
void it_802BDDB4(Item_GObj* g, f32 v) { T(0x350u + 6u); T(E(g)); T(B(v)); }
bool gm_8016B0B4(void) { T(0x360u); return (g_tr >> 2) & 1; }
float ftCo_Damage_CalcAngle(Fighter* fp, float kb)
{
    T(0x370u + E(fp->gobj)); T(B(kb));
    return (g_tr % 8u) == 0 ? (float) M_PI : (float) (g_tr % 700u) / 100.0f;
}
void ftLib_GetCameraBonePos(HSD_GObj* g, Vec3* v) { T(0x380u + E(g)); v->x = 1.5f; v->y = -(float) (g_tr & 15); v->z = 2.0f; }
s32 it_802E5F8C(Item_GObj* g, Vec3* v, s32 a, s32 b, f32 ang, f32 kb)
{
    T(0x390u + E(g)); T(B(v->x)); T(B(v->y)); T(B(v->z)); T((u32) a); T((u32) b); T(B(ang)); T(B(kb));
    return (s32) (g_tr % 100u);
}
s32 Player_GetUnk98(s32 slot) { T(0x3A0u + (u32) slot); return (s32) (g_tr % 1000u); }
void Player_SetUnk98(s32 slot, s32 v) { T(0x3B0u + (u32) slot); T((u32) v); }
HSD_GObj* Player_GetEntity(s32 slot) { T(0x3C0u + (u32) slot); return &GO[(g_tr >> 1) % NF]; }
int mpLineGetNext(int l) { T(0x3D0u); T((u32) l); return l + (int) (g_tr & 1); }
int mpLineGetPrev(int l) { T(0x3D8u); T((u32) l); return l - (int) ((g_tr >> 1) & 1); }
bool ftLib_IsSamePlayer(HSD_GObj* a, HSD_GObj* b)
{
    T(0x3E0u + E(a)); T(E(b));
    return a == b || ((Fighter*) a->user_data)->player_id == ((Fighter*) b->user_data)->player_id;
}
bool Player_8003544C(s32 slot, bool sub) { return ((u32) slot + (u32) sub) & 1; }
void lb_80014574(u8 port, int id, int a, int b) { T(0x600u + port); T((u32) id); T((u32) a); T((u32) b); }
void it_80284FC4(Item_GObj* g, int a, float f) { T(0x3F0u + E(g)); T((u32) a); T(B(f)); }
void it_80285024(Item_GObj* g, int a) { T(0x400u + E(g)); T((u32) a); }
void ftCo_800D105C(Fighter_GObj* g) { T(0x410u + E(g)); }
void ft_80081C88(Fighter_GObj* g, float s) { T(0x420u + E(g)); T(B(s)); }
void it_80294EB0(Item_GObj* g, Vec3* a, Vec3* b)
{
    T(0x430u + E(g)); T(B(a->x)); T(B(a->y)); T(B(a->z)); T(B(b->x)); T(B(b->y)); T(B(b->z));
}
void it_80294E78(Item_GObj* g, float f) { T(0x440u + E(g)); T(B(f)); }
float it_8026B54C(Item_GObj* g) { T(0x450u + E(g)); return (float) (g_tr % 600u) - 0.5f; }
void Item_8026A8EC(Item_GObj* g) { T(0x460u + E(g)); }
void it_8026BD0C(Item_GObj* g) { T(0x470u + E(g)); }
void Item_8026ABD8(Item_GObj* g, Vec3* v, float f) { T(0x480u + E(g)); T(B(v->x)); T(B(v->y)); T(B(v->z)); T(B(f)); }
void Item_8026AB54(HSD_GObj* g, HSD_GObj* o, Fighter_Part p) { T(0x490u + E(g)); T(E(o)); T((u32) p); }
void it_8026BCF4(Item_GObj* g) { T(0x4A0u + E(g)); }
HSD_GObj* it_8029A748(Item_GObj* g, Vec3* p, Fighter_Part part, f32 d)
{
    T(0x4B0u + E(g)); T(B(p->x)); T(B(p->y)); T(B(p->z)); T((u32) part); T(B(d));
    return (g_tr & 1) ? &IT[g_tr % 3u] : NULL;
}
void it_8029A8F4(Item_GObj* g, Vec3* v) { T(0x4C0u + E(g)); T(B(v->x)); T(B(v->y)); T(B(v->z)); }
void it_8029A89C(Item_GObj* g, f32 f) { T(0x4D0u + E(g)); T(B(f)); }
void ft_PlaySFX(Fighter* fp, enum_t id, u8 vol, u8 pan) { T(0x4E0u + E(fp->gobj)); T((u32) id); T(vol); T(pan); }

static void cb_dmg(HSD_GObj* g) { T(0x4F0u + E(g)); }
static void cb_death(HSD_GObj* g) { T(0x500u + E(g)); }

#define HSD_JObjGetTranslation stub_GetTranslation
#define HSD_JObjSetTranslate stub_SetTranslate
#define HSD_JObjMtxIsDirty stub_MtxIsDirty
#define HSD_JObjSetScale stub_SetScale

#include "ftcommon3_extract.inc"

int printf(const char*, ...);
static void zero(void* p, unsigned n) { unsigned char* b = p; while (n--) *b++ = 0; }

static u32 rs;
static u32 rnd(void) { rs ^= rs << 13; rs ^= rs >> 17; rs ^= rs << 5; return rs; }
static float rf(void)
{
    u32 k = rnd() % 16;
    if (k == 0) return 0.0f;
    if (k == 1) return -0.0f;
    if (k == 2) return 1.0f;
    if (k == 3) return -1.0f;
    return (float) ((int) (rnd() % 2001) - 1000) / 7.0f;
}
static HSD_GObj* rit(void) { u32 k = rnd() % 4; return k < 3 ? &IT[k] : NULL; }

static const int KINDS[] = { 0, 2, 8, 14, 15, 20, 24, 27, 28, 32, 33, 34, 40 };
static const int IKINDS[] = { 12, 13, 103, 5 };
#define NOPS 34

int main(void)
{
    int op, k, i;
    p_ftCommonData = &cd;
    HSD_GObjPLinkHead = plinks;
    for (op = 0; op < NOPS; op++) {
        for (k = 0; k < 300; k++) {
            u32 r = 0, t = 0;
            int n, used = 0, ia, ib;
            float a;
            Vec3 w;
            HSD_GObj *g, *gi, *prev = NULL;
            Fighter* fp;
            zero(F, sizeof F); zero(GO, sizeof GO); zero(IT, sizeof IT); zero(&cd, sizeof cd);
            zero(FD, sizeof FD); zero(DA, sizeof DA); zero(X8, sizeof X8); zero(PB, sizeof PB);
            zero(plinks, sizeof plinks); zero(&MAP, sizeof MAP);
            g_tr = 0;
            rs = 0x5A5A1u + (u32) op * 7919u + (u32) k * 104729u;
            g_map_null = (rnd() % 8) == 0;
            MAP.line_count = (int) (rnd() % 30);
            for (i = 0; i < 3; i++) {
                IT[i].classifier = (rnd() % 4) ? 6 : 4;
                ITK[i] = IKINDS[rnd() % 4];
                ITM[i] = (int) (rnd() % 12);
            }
            for (i = 0; i < NF; i++) {
                Fighter* f = &F[i];
                int j;
                GO[i].user_data = f;
                GO[i].classifier = 4;
                f->gobj = &GO[i];
                f->ft_data = &FD[i];
                FD[i].x0 = &DA[i];
                FD[i].x8 = &X8[i];
                X8[i].x12 = (u8) (rnd() % 40);
                DA[i].model_scaling = rf();
                f->parts = PB[i];
                for (j = 0; j < 8; j++) PB[i][j].joint = (HSD_JObj*) JO[rnd() % 8];
                f->kind = KINDS[rnd() % 13];
                f->x8_spawnNum = (int) (rnd() % 8);
                f->player_id = (u8) (rnd() % 4);
                f->x618_player_id = (u8) (rnd() % 4);
                f->motion_id = (int) (rnd() % 340);
                f->facing_dir = rf();
                f->gr_vel = rf(); f->self_vel.x = rf();
                f->x34_scale.y = rf();
                f->cur_pos.x = rf(); f->cur_pos.y = rf(); f->cur_pos.z = rf();
                f->xD4_unk_vel.x = rf(); f->xD4_unk_vel.y = rf(); f->xD4_unk_vel.z = rf();
                f->x2C4.x = rf(); f->x2C4.y = rf();
                f->x6A4_transNOffset.z = rf();
                f->ground_or_air = (GroundOrAir) (rnd() & 1);
                f->ecb_lock = (int) (rnd() % 20);
                f->coll_data.x130_flags = rnd();
                f->coll_data.floor.index = (int) (rnd() % 6) - 2;
                f->x1968_jumpsUsed = (u8) (rnd() % 4);
                f->x1969_walljumpUsed = (u8) (rnd() % 2);
                f->co_attrs.max_jumps = (int) (rnd() % 7);
                f->co_attrs.ground_max_horizontal_velocity = rf();
                f->co_attrs.x114.x = rf(); f->co_attrs.x114.y = rf(); f->co_attrs.x114.z = rf();
                f->co_attrs.x120.x = rf(); f->co_attrs.x120.y = rf(); f->co_attrs.x120.z = rf();
                f->co_attrs.x12C = rf();
                f->co_attrs.x130.x = rf(); f->co_attrs.x130.y = rf(); f->co_attrs.x130.z = rf();
                f->co_attrs.x13C = rf();
                f->frame_speed_mul = rf();
                f->dmg.x18A4_knockbackMagnitude = rf();
                f->dmg.kb_applied = (rnd() & 1) ? rf() : 0.0f;
                f->dmg.facing_dir_1 = rf();
                f->dmg.x1854_collpos.x = rf(); f->dmg.x1854_collpos.y = rf();
                f->dmg.x18c4_source_ply = (int) (rnd() % 7);
                f->xF8_playerNudgeVel.x = rf(); f->xF8_playerNudgeVel.y = rf();
                f->x1A6C = rf();
                f->x1A7C.x = rf(); f->x1A7C.y = rf(); f->x1A7C.z = rf();
                f->x20A0_accessory = (rnd() & 1) ? (HSD_JObj*) JO[rnd() % 8] : NULL;
                f->victim_gobj = (rnd() % 4) == 0 ? &GO[rnd() % NF] : NULL;
                f->item_gobj = rit();
                f->x1978 = rit();
                f->x197C = rit();
                f->x1980 = rit();
                f->x1984_heldItemSpec = rit();
                f->x2014 = (int) rnd();
                f->x2018 = (int) (rnd() % 2001) - 1000;
                f->x201C = (int) rnd();
                f->x2020 = (s8) rnd(); f->x2021 = (s8) rnd();
                f->x2024 = (int) (rnd() % 201) - 100;
                f->x2144 = (int) rnd(); f->x2148 = (int) rnd();
                f->take_dmg_cb = (rnd() & 1) ? cb_dmg : NULL;
                f->death1_cb = (rnd() & 1) ? cb_death : NULL;
                f->x594_b0 = rnd() & 1;
                f->is_sub_fighter = rnd() & 1;
                f->x2219_b0 = rnd() & 1;
                f->x2219_b1 = (rnd() % 4) == 0; f->x2219_b5 = (rnd() % 4) == 0;
                f->x221D_b5 = (rnd() % 4) == 0;
                f->x221E_b3 = rnd() & 1;
                f->x221F_b3 = (rnd() % 4) == 0;
                f->x2224_b2 = rnd() & 1;
                f->x2227_b0 = rnd() & 1;
            }
            cd.x450 = rf(); cd.x454 = rf(); cd.x458 = rf(); cd.x45C = rf(); cd.x460 = rf();
            cd.x5C0 = rf();
            cd.x6F8 = (int) rnd(); cd.x700 = (int) (rnd() % 2001) - 1000; cd.x704 = rf();
            cd.x708 = rf(); cd.x70C = rf(); cd.x710 = rf();
            n = (int) (rnd() % (NF + 1));
            for (i = 0; i < n; i++) {
                int j = (int) (rnd() % NF);
                while (used & (1 << j)) j = (j + 1) % NF;
                used |= 1 << j;
                if (prev == NULL) plinks[HSD_GOBJ_PLINK_FIGHTER] = &GO[j]; else prev->next = &GO[j];
                prev = &GO[j];
            }
            g = &GO[rnd() % NF];
            fp = g->user_data;
            gi = rit();
            a = rf();
            ia = (int) (rnd() % 16) - 4;
            ib = (int) rnd();
            w.x = rf(); w.y = rf(); w.z = rf();
            /* nudge ops, odd cases: the other fighters' side of f0 is exactly 0 and the arg
             * side cancels exactly unfused, so only the fused residual decides f0's sign */
            if ((op == 5 || op == 6) && (k & 1)) {
                for (i = 0; i < NF; i++) { F[i].x2C4.x = 0; F[i].cur_pos.x = 0; F[i].xD4_unk_vel.x = 0; }
                fp->x2C4.x = rf();
                w.x = -(fp->x2C4.x * fp->facing_dir);
            }
            g_tr = rnd(); /* varied stub results from the first call on */
            switch (op) {
            case 0: ftCommon_8007D6A4(fp); break;
            case 1: ftCommon_8007D7FC(fp); break;
            case 2: ftCommon_8007D92C(g); break;
            case 3: ftCommon_8007DB24(g); break;
            case 4: ftCommon_8007DB58(g); break;
            case 5: ftCommon_8007DD7C(g, &w); break;
            case 6: ftCommon_8007DFD0(g, &w); break;
            case 7: ftCommon_8007E0E4(g); break;
            case 8: ftCommon_8007E358(g); break;
            case 9: ftCommon_8007E3EC(g); break;
            case 10: fp->x20A0_accessory = NULL; ftCommon_SetAccessory(fp, (HSD_Joint*) JO[(u32) ib % 8u]); break;
            case 11: ftCommon_8007E690(fp, (HSD_AnimJoint*) JO[(u32) ib % 8u]); break;
            case 12: ftCommon_8007E6DC(g, (ib & 1) ? fp->x1978 : gi, ib); break;
            case 13: ftCommon_8007E79C(g, ib); break;
            case 14: ftCommon_8007E7E4(g, ib); break;
            case 15: ftCommon_8007E82C(g); break;
            case 16:
                if (ftGetParasolStatus(g) == -1) { T(0xBADu); break; }
                ftCommon_8007E83C(g, (int) ((u32) ib % 7u), (ib & 8) ? 0.0f : a);
                break;
            case 17: r = (u32) ftGetParasolStatus(g); break;
            case 18: ftCommon_8007EA90(fp, ib); break;
            case 19: ftCommon_8007EC30((u32) ia, (u32) ib); break;
            case 20: ftCommon_8007EEC8(fp, ia, ib); break;
            case 21: ftCommon_8007EF5C(fp, ia); break;
            case 22: ftCommon_8007F578(g); break;
            case 23: ftCommon_8007F5CC(g, ia & 3); break;
            case 24: ftCommon_8007F6A4(fp, (HSD_JObj*) JO[(u32) ib % 8u]); break;
            case 25: ftCommon_8007F824(g); break;
            case 26: ftCommon_8007F86C(g); break;
            case 27: ftCommon_8007F8E8(g); break;
            case 28: ftCommon_8007F948(g, gi, ib); break;
            case 29: ftCommon_8007F9B4(g); break;
            case 30: ftCommon_8007FA00(g); break;
            case 31: ftCommon_8007FA58(g, gi); break;
            case 32: ftCommon_8007FC7C(g, a); break;
            case 33: ftCommon_8007FE84(g, gi, ib, a); break;
            }
            for (i = 0; i < NF; i++) {
                Fighter* f = &F[i];
                t = t * 31u + (B(f->self_vel.x) ^ B(f->gr_vel) * 3u ^ B(f->dmg.x18A4_knockbackMagnitude) * 7u);
                t = t * 31u + (B(f->xF8_playerNudgeVel.x) ^ B(f->xF8_playerNudgeVel.y) * 5u);
                t = t * 31u + (B(f->x1A7C.x) ^ B(f->x1A7C.y) * 3u ^ B(f->x1A7C.z) * 11u);
                t = t * 31u + (u32) f->ground_or_air + (u32) f->ecb_lock * 4u + f->coll_data.x130_flags * 13u;
                t = t * 31u + (u32) f->x1968_jumpsUsed + (u32) f->x1969_walljumpUsed * 8u;
                t = t * 31u + E(f->x20A0_accessory) + E(f->item_gobj) * 100u + E(f->x1978) * 10000u;
                t = t * 31u + E(f->x197C) + E(f->x1980) * 100u + E(f->x1984_heldItemSpec) * 10000u;
                t = t * 31u + (u32) f->x2014 + (u32) f->x2018 * 3u + (u32) f->x201C * 5u + (u32) f->x2024 * 7u;
                t = t * 31u + (u32) (u8) f->x2020 + (u32) (u8) f->x2021 * 256u;
                t = t * 31u + (u32) f->x2144 + (u32) f->x2148 * 3u;
                t = t * 31u + (u32) (f->x2227_b0 | f->x2219_b0 << 1 | f->x221E_b3 << 2);
            }
            printf("%d %d %08X %08X %08X\n", op, k, r, t, g_tr);
        }
    }
    return 0;
}
