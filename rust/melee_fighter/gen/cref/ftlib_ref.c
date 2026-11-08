/* C reference for src/ftlib.rs (port round 3): src/melee/ft/ftlib.c cut verbatim into
 * ftlib_extract.inc by run.py (everything but ftLib_ApplyMetalBox, 80087508/80087574/80087610 and
 * 8008777C), plus the ftcommon.c functions it calls that group 3 has ported
 * (8007ED2C, GetModelScale, 80080144, 80080174). Six fighters, a random fighter p-link list,
 * random fields, one random op per case. Callees outside the ported set are stubs that fold
 * their arguments into the trace word g_tr; tests/ftlib_ref.rs installs the same stubs as hooks.
 * Built with the shipping contraction (run.py): ftLib_FindNearestOpponent / 80086368 fuse dx*dx + dy*dy. */
#include "ft/types.h"
#include "ft/forward.h"
#include "ft/ftlib.h"
#include "ft/ftcommon.h"
#include "ft/fighter.h"
#include "ft/ftdata.h"
#include "ft/ftparts.h"
#include "ft/ftanim.h"
#include "ft/kinds/ftGameWatch/ftgamewatch.h"
#include "ft/kinds/ftKirby/ftkirby.h"
#include <melee/cm/camera.h>
#include <melee/cm/types.h>
#include <melee/gm/gm_unsplit.h>
#include <melee/pl/player.h>
#include <melee/pl/plbonuslib.h>
#include <melee/lb/lb_00B0.h>
#include <sysdolphin/baselib/gobj.h>
#include <sysdolphin/baselib/random.h>

#define NF 6
static u32 g_tr;
static void T(u32 v) { g_tr = g_tr * 31u + v; }
static u32 B(float f) { union { float f; u32 u; } c; c.f = f; return c.u; }

static Fighter F[NF];
static HSD_GObj GO[NF], IT[3], CAMG;
static u64 JO[8][32];
static ftData FD[NF];
static ftCo_DatAttrs DA[NF];
static FtSFX SFX[NF];
static CmSubject CS[NF];
static FighterBone PB[NF][8];
static ftCommonData cd;
static HSD_GObj* plinks[64];
static int g_slip;

static u32 E(void* g)
{
    int i;
    if (g == NULL) return 0;
    for (i = 0; i < NF; i++) if (g == &GO[i]) return 1u + (u32) i;
    for (i = 0; i < 3; i++) if (g == &IT[i]) return 10u + (u32) i;
    if (g == &CAMG) return 20;
    for (i = 0; i < 8; i++) if (g == (void*) JO[i]) return 30u + (u32) i;
    for (i = 0; i < NF; i++) if (g == &CS[i]) return 40u + (u32) i;
    for (i = 0; i < NF; i++) if (g == &F[i].coll_data) return 50u + (u32) i;
    for (i = 0; i < NF; i++) if (g == &F[i].x20A4) return 60u + (u32) i;
    return 99;
}

void __assert(const char* file, u32 line, const char* cond) { (void) file; (void) line; (void) cond; for (;;) {} }
ftCommonData* p_ftCommonData;
HSD_GObj** HSD_GObjPLinkHead;

bool gm_8016B168(void) { T(0x140u); return (g_tr >> 4) & 1; }
int Slippi_Codes(void) { T(0x150u); return g_slip; }
s32 HSD_Randi(s32 n) { T(0x160u + (u32) n); return (s32) ((g_tr >> 7) & 1); }
Fighter_Part ftParts_GetBoneIndex(Fighter* fp, Fighter_Part i) { T(0x170u + (u32) i); return (Fighter_Part) (fp->player_id % 8); }
void Fighter_UpdateModelScale(Fighter_GObj* g) { T(0x180u + E(g)); }
void lb_8000B1CC(HSD_JObj* j, Vec3* off, Vec3* out)
{
    u32 e = E(j);
    T(0x190u + e);
    out->x = off->x + (float) e;
    out->y = off->y - (float) e;
    out->z = off->z;
}
void ftCommon_8007E6DC(Fighter_GObj* g, Fighter_GObj* o, s32 i) { T(0x1A0u + E(g)); T(E(o)); T((u32) i); }
void ftCommon_8007E82C(Fighter_GObj* g) { T(0x1B0u + E(g)); }
void Fighter_ResetInputData_80068854(Fighter_GObj* g) { T(0x1C0u + E(g)); }
HSD_GObj* Camera_80030A50(void) { T(0x1D0u); return ((g_tr >> 3) & 1) ? &CAMG : NULL; }
bool Camera_LogicToScreen(CmSubject* s, S32Vec2* p) { T(0x1E0u + E(s)); p->x += 1; return (g_tr >> 2) & 1; }
bool Camera_80031154(Vec3* v) { T(0x1F0u); T(B(v->x)); return (g_tr >> 6) & 1; }
bool Player_8003544C(s32 slot, bool sub) { return ((u32) slot + (u32) sub) & 1; }
void lb_80014574(u8 port, int id, int a, int b) { T(0x600u + port); T((u32) id); T((u32) a); T((u32) b); }
void HSD_PadRumbleRemoveId(u8 port, int id) { T(0x700u + port); T((u32) id); }
void lb_800145C0(u8 slot) { T(0x800u + slot); }
bool Player_GetMoreFlagsBit4(s32 slot) { T(0x200u + (u32) slot); return (g_tr >> 5) & 1; }
void ftData_80085560(int a, int b) { T(0x210u); T((u32) a); T((u32) b); }
void ftCo_800D4F24(Fighter_GObj* g, int i) { T(0x220u + E(g)); T((u32) i); }
void Fighter_Spawn(Fighter_GObj* g) { T(0x230u + E(g)); }
void ftCommon_8007D92C(Fighter_GObj* g) { T(0x240u + E(g)); }
int Geno_DefineTemplateSkeleton(int k) { T(0x250u + (u32) k); return (g_tr >> 1) & 1; }
int Geno_DefineBaseKind(int k) { T(0x260u + (u32) k); return k % 27; }
int Mex_SsmForPortKind(int k) { T(0x270u + (u32) k); return (int) (g_tr % 5u) - 1; }
void pl_8003E4A4(int slot, bool sub, void* p, int a) { (void) p; T(0x280u + (u32) slot * 2u + (u32) sub); T((u32) a); }
bool ftAnim_IsFramesRemaining(Fighter_GObj* g) { T(0x290u + E(g)); return true; }
void ftGw_Init_8014A7F4(HSD_GObj* g, ItemModStruct* d) { (void) d; T(0x2A0u + E(g)); }
void ftGw_Init_8014A814(HSD_GObj* g, ItemModStruct* d) { (void) d; T(0x2B0u + E(g)); }
void ftKb_Init_800EEB00(Fighter_GObj* g, ArticleDynamicBones** d) { (void) d; T(0x2C0u + E(g)); }
void ftKb_Init_800EEB1C(Fighter_GObj* g, s32* d) { (void) d; T(0x2D0u + E(g)); }
void it_80294E78(Item_GObj* g, float f) { T(0xA00u + E(g)); T(B(f)); }
void it_8029A89C(Item_GObj* g, f32 f) { T(0xB00u + E(g)); T(B(f)); }

#include "ftlib_extract.inc"

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
static HSD_GObj* rg(void) /* a gobj argument: a fighter, an item, or NULL */
{
    u32 k = rnd() % 10;
    if (k < NF) return &GO[k];
    if (k < 9) return &IT[k - NF];
    return NULL;
}

static const int KINDS[] = { 0, 2, 8, 14, 15, 20, 24, 27, 28, 32, 33, 34, 40 };
#define NOPS 87

int main(void)
{
    int op, k, i;
    p_ftCommonData = &cd;
    HSD_GObjPLinkHead = plinks;
    for (op = 0; op < NOPS; op++) {
        for (k = 0; k < 120; k++) {
            u32 r = 0, o1 = 0, o2 = 0, o3 = 0, q1, q2, q3;
            HSD_GObj *gn, *gi;
            int t = 0, n, used = 0, ia, ib;
            float a;
            Vec3 v = { 0, 0, 0 }, w;
            S32Vec2 sv = { 0, 0 };
            HSD_GObj *g, *g2, *prev = NULL;
            zero(F, sizeof F); zero(GO, sizeof GO); zero(IT, sizeof IT); zero(&cd, sizeof cd);
            zero(FD, sizeof FD); zero(DA, sizeof DA); zero(SFX, sizeof SFX); zero(CS, sizeof CS);
            zero(PB, sizeof PB); zero(plinks, sizeof plinks);
            g_tr = 0;
            rs = 0x13579u + (u32) op * 7919u + (u32) k * 104729u;
            g_slip = (int) (rnd() % 3);
            for (i = 0; i < NF; i++) {
                Fighter* f = &F[i];
                int j;
                GO[i].user_data = f;
                GO[i].classifier = (rnd() % 4) ? 4 : 6;
                f->gobj = &GO[i];
                f->ft_data = &FD[i];
                FD[i].x0 = &DA[i];
                FD[i].x4C_sfx = &SFX[i];
                f->parts = PB[i];
                for (j = 0; j < 8; j++) PB[i][j].joint = (HSD_JObj*) JO[rnd() % 8];
                f->x890_cameraBox = &CS[i];
                f->kind = KINDS[rnd() % 13];
                f->x8_spawnNum = (int) (rnd() % 8);
                f->player_id = (u8) (rnd() % 4);
                f->x618_player_id = (u8) (rnd() % 4);
                f->x619_costume_id = (u8) (rnd() % 4);
                f->x61A_controller_index = (u8) (rnd() % 4);
                f->team = (u8) (rnd() % 3);
                f->motion_id = (int) (rnd() % 340);
                f->facing_dir = rf();
                f->x34_scale.x = rf(); f->x34_scale.y = rf();
                f->cur_pos.x = rf(); f->cur_pos.y = rf(); f->cur_pos.z = rf();
                f->prev_pos.x = rf(); f->prev_pos.y = rf(); f->prev_pos.z = rf();
                f->pos_delta.x = rf(); f->pos_delta.y = rf(); f->pos_delta.z = rf();
                f->ground_or_air = (GroundOrAir) (rnd() & 1);
                f->xF8_playerNudgeVel.x = rf();
                f->input.lstick[0].x = rf(); f->input.lstick[0].y = rf();
                f->coll_data.ecb.top.y = rf(); f->coll_data.ecb.bottom.y = rf();
                f->co_attrs.model_scaling = rf();
                f->co_attrs.name_tag_height = rf();
                f->co_attrs.camera_zoom_target_bone = (int) (rnd() % 8);
                f->co_attrs.x170.x = rf(); f->co_attrs.x170.y = rf(); f->co_attrs.x170.z = rf();
                f->co_attrs.x12C = rf();
                f->co_attrs.x13C = rf();
                DA[i].name_tag_height = rf();
                DA[i].camera_zoom_target_bone = (int) (rnd() % 8);
                SFX[i].x34 = (rnd() & 1) ? (int) (rnd() % 12000) : 5000 + (int) (rnd() % 5000);
                f->dmg.x1830_percent = rf();
                f->dmg.x18A4_knockbackMagnitude = rf();
                f->dmg.x18B8 = (rnd() & 1) ? rf() : 0.0f;
                f->dmg.x18BC = (rnd() & 1) ? rf() : 0.0f;
                f->dmg.x18c4_source_ply = (int) (rnd() % 7);
                f->dmg.x18CC = (int) rnd();
                f->dmg.x18D0 = (int) rnd();
                f->dmg.x1958 = rf();
                f->item_gobj = (rnd() & 1) ? &IT[rnd() % 3] : NULL;
                f->x1978 = (rnd() & 1) ? &IT[rnd() % 3] : NULL;
                f->x197C = (rnd() & 1) ? &IT[rnd() % 3] : NULL;
                f->x1980 = (rnd() & 1) ? &IT[rnd() % 3] : NULL;
                f->x1984_heldItemSpec = (rnd() & 1) ? &IT[rnd() % 3] : NULL;
                f->x2024 = (int) (rnd() % 201) - 100;
                f->x209A = (u16) (rnd() % 3);
                f->smash_attrs.state = (int) (rnd() % 4);
                f->x2188.x = (int) rnd(); f->x2188.y = (int) rnd();
                CS[i].target_ext.v.z = rf();
                CS[i].bone_pos.x = rf(); CS[i].bone_pos.y = rf(); CS[i].bone_pos.z = rf();
                CS[i].pos.x = rf();
                f->invisible = rnd() & 1;
                f->is_sandbag = rnd() & 1;
                f->is_sub_fighter = rnd() & 1;
                f->x221C_b6 = rnd() & 1;
                f->x221D_b4 = rnd() & 1;
                f->x221E_b1 = rnd() & 1; f->x221E_b2 = rnd() & 1; f->x221E_b3 = rnd() & 1;
                f->x221E_b4 = rnd() & 1; f->x221E_b5 = rnd() & 1;
                f->x221F_b0 = rnd() & 1; f->x221F_b1 = rnd() & 1; f->x221F_b2 = rnd() & 1;
                f->x221F_b3 = (rnd() % 4) == 0; f->x221F_b5 = rnd() & 1;
                f->x2220_b7 = (rnd() % 4) == 0;
                f->x2224_b2 = rnd() & 1;
                f->x2226_b4 = rnd() & 1;
                f->x2229_b3 = (rnd() % 4) == 0;
            }
            cd.x708 = rf(); cd.x70C = rf(); cd.x710 = rf();
            for (i = 0; i < 4; i++) {
                cd.x6DC_colorsByPlayer[i].r = (u8) rnd(); cd.x6DC_colorsByPlayer[i].g = (u8) rnd();
                cd.x6DC_colorsByPlayer[i].b = (u8) rnd(); cd.x6DC_colorsByPlayer[i].a = (u8) rnd();
            }
            cd.x6D8[0] = (int) rnd();
            /* the fighter list: n distinct fighters in random order */
            n = (int) (rnd() % (NF + 1));
            for (i = 0; i < n; i++) {
                int j = (int) (rnd() % NF);
                while (used & (1 << j)) j = (j + 1) % NF;
                used |= 1 << j;
                if (prev == NULL) plinks[HSD_GOBJ_PLINK_FIGHTER] = &GO[j]; else prev->next = &GO[j];
                prev = &GO[j];
            }
            g = &GO[rnd() % NF];
            g2 = rg();
            a = rf();
            ia = (int) (rnd() % 16) - 4;
            ib = (int) rnd();
            w.x = rf(); w.y = rf(); w.z = rf();
            q1 = rnd(); q2 = rnd(); q3 = rnd();
            gn = (q1 & 3) ? g : NULL;
            gi = (q2 & 1) ? &IT[q3 % 3] : g2;
            switch (op) {
            case 0: r = (u32) ftLib_CountFighters(); break;
            case 1: r = ftLib_IsMasterHandPresent(); break;
            case 2: r = ftLib_IsCrazyHandPresent(); break;
            case 3: r = E(ftLib_FindLowestPercentOpponent(gn)); break;
            case 4: r = E(ftLib_FindNearestOpponent(&w, gn)); break;
            case 5: r = E(ftLib_FindNearestOpponentInDir(&w, gn, (q2 & 1) ? -1.0f : (q2 & 2) ? 1.0f : a)); break;
            case 6: r = B(ftLib_GetOpponentsDir(&w, gn)); break;
            case 7: r = B(ftLib_GetFacingDir(g)); break;
            case 8: r = (u32) ftLib_GetGroundAir(g); break;
            case 9: { float x, y; ftLib_GetLStick(g, &x, &y); o1 = B(x); o2 = B(y); } break;
            case 10: r = E(ftLib_GetHipJObj(g)); break;
            case 11: r = E(ftLib_GetPartJObj(g, (Fighter_Part) (q1 % 8))); break;
            case 12: ftLib_GetPos(g, &v); break;
            case 13: ftLib_SetPos(g, &w); break;
            case 14: ftLib_GetPrevPos(g, &v); break;
            case 15: ftLib_SetScale(g, a); break;
            case 16: ftLib_GetCameraBonePos(g, &v); break;
            case 17: ftLib_ReleaseItem(g, g2); break;
            case 18: ftLib_ClearHeldSpecialItem(g); break;
            case 19: r = E(ftLib_GetItem(g)); break;
            case 20: r = ftLib_IsHoldingItem(g, gi); break;
            case 21: r = E(ftLib_GetHeldSpecialItem(g)); break;
            case 22: r = ftLib_IsInputDisabled(g); break;
            case 23: ftLib_DisableInput(g); break;
            case 24: ftLib_DisableAllInput(); break;
            case 25: ftLib_EnableInput(g); break;
            case 26: ftLib_EnableAllInput(); break;
            case 27: r = ftLib_IsItemVisible(g, gi); break;
            case 28: r = ftLib_IsFighter(g2); break;
            case 29: r = E(ftLib_GetCollData(g)); break;
            case 30: ftLib_GetECBCenter(g, &v); break;
            case 31: r = B(ftLib_GetModelScale(g)); break;
            case 32: r = B(ftLib_800869F8(g)); break;
            case 33: r = B(ftLib_GetScale(g)); break;
            case 34: r = ftLib_IsShielding(g); break;
            case 35: ftLib_80086A4C(g, a); break;
            case 36: r = ftLib_GetScreenPos(g, &sv); break;
            case 37: r = ftLib_UpdateScreenVisibility(g); break;
            case 38: ftLib_UpdateLogicOffCamera(g); break;
            case 39: r = ftLib_IsOffscreen(g); break;
            case 40: r = E(ftLib_GetCameraSubject(g)); break;
            case 41: r = B(ftLib_80086B80(g)); break;
            case 42: ftLib_GetCameraSubjectBonePos(g, &v); break;
            case 43: r = ftLib_IsCameraSubjectInBounds(g); break;
            case 44: r = ftLib_GetPlayerIndex(g); break;
            case 45: ftLib_GetPosDelta(g, &v); break;
            case 46: r = (u32) ftLib_GetMotionId(g); break;
            case 47: ftLib_StartRumble(g, ia, ib); break;
            case 48: ftLib_StartRumbleAll(ia, ib); break;
            case 49: ftLib_StartItemRumble(g, ia, ib); break;
            case 50: ftLib_StartItemRumbleAll(ia, ib); break;
            case 51: ftLib_StopItemRumble(g); break;
            case 52: r = (u32) ftLib_GetTeam(g); break;
            case 53: r = ftLib_IsInHitstun(g); break;
            case 54: r = ftLib_CanBeMagnified(g); break;
            case 55: r = ftLib_80086F4C(g); break;
            case 56: r = B(ftLib_GetNameTagHeight(g)); break;
            case 57: r = ftLib_IsSwordSwing(g); break;
            case 58: r = ftLib_IsSamePlayer(gn, (q2 & 3) ? &GO[q3 % NF] : NULL); break;
            case 59: r = ftLib_IsSleeping(g); break;
            case 60: ftLib_IncFighterRefCount(ia); break;
            case 61: ftLib_DecFighterRefCount(ia); break;
            case 62: r = ftLib_GetShakeOffset(g, &v); break;
            case 63: { int val = 0x55; r = ftLib_GetSubColor(g, &val); o1 = (u32) val; } break;
            case 64: ftLib_SetPercent(g, ia * 37); break;
            case 65: r = (u32) ftLib_GetPercent(g); break;
            case 66: ftLib_80087140(g); break;
            case 67: r = ftLib_80087284(g); break;
            case 68: r = (u32) ftLib_GetKind(g); break;
            case 69: r = E(ftLib_GetShadow(g)); break;
            case 70: r = ftLib_CastsShadow(g); break;
            case 71: r = (u32) ftLib_GetLastAttackerSlot(g); break;
            case 72: r = (u32) ftLib_IsLastAttackerSubFighter(g); break;
            case 73: r = (u32) ftLib_IsSleeping_8008731C(g); break;
            case 74: r = ftLib_IsDead(g) | ftLib_IsDeadUp(g) << 1 | ftLib_IsDeadUpStar(g) << 2 |
                         ftLib_IsDeadUpFall(g) << 3 | ftLib_IsRebirth(g) << 4 | ftLib_IsEntry(g) << 5; break;
            case 75: F[q1 % NF].motion_id = (int) (q2 % 14); r = ftLib_IsDead(g) | ftLib_IsDeadUp(g) << 1 |
                         ftLib_IsDeadUpStar(g) << 2 | ftLib_IsDeadUpFall(g) << 3 | ftLib_IsRebirth(g) << 4; break;
            case 76: r = E(ftLib_FindBySpawnNum(q1 % 9)); break;
            case 77: r = B(ftLib_GetCrowdKnockback(g)); break;
            case 78: r = ftLib_GetSpawnNum(g); break;
            case 79: r = (u32) ftLib_8008746C(g); break;
            case 80: r = (u32) ftLib_IsSubFighter(g); break;
            case 81: ftLib_800874CC(g, NULL, ib); break;
            case 82: ftLib_IsFramesRemaining(g); break;
            case 83: r = ftLib_IsChargingSmash(g); break;
            case 84: r = (u32) ftLib_GetLastHitSourceType(g) ^ (u32) ftLib_GetLastHitSourceKind(g); break;
            case 85: ftLib_GetGameWatchColor(g, NULL); ftLib_GetGameWatchOutlineColor(g, NULL); break;
            case 86: r = ftLib_IsBeingNudged(g); break;
            }
            for (i = 0; i < NF; i++) {
                Fighter* f = &F[i];
                t = t * 31 + (int) (B(f->cur_pos.x) ^ B(f->cur_pos.y) * 3u ^ B(f->cur_pos.z) * 7u);
                t = t * 31 + (int) (B(f->x34_scale.y) ^ B(f->dmg.x1958) * 5u ^ B(f->dmg.x1830_percent) * 11u);
                t = t * 31 + (int) (f->x221D_b4 | f->x221F_b0 << 1) + f->x2188.x;
            }
            o3 = (u32) sv.x;
            printf("%d %d %08X %08X %08X %08X %08X %08X %08X %08X %08X\n", op, k, r, o1, o2, o3,
                   B(v.x), B(v.y), B(v.z), (u32) t, g_tr);
        }
    }
    return 0;
}
