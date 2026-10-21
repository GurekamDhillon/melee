/* C reference for src/ft0df.rs (group 3 round 13): all of src/melee/ft/ft_0DF0.c and
 * ft_0DF1.c, ftColl_8007AEE0/AEF8/AF10/AF28/B8A8/B8CC (ftcoll.c), ftCo_800C0658/0674/0694
 * (ftdevice.c) and ftParts_80074A4C/A74/A8C/ACC (ftparts.c), cut verbatim into
 * ft0df_extract.inc by run.py together with what they call that group 3 has ported
 * (ftCommon_8007EBAC/ECD4, ftCo_GetCStickAngle, lbtrigf.c atan2f). Calls out of the cut are
 * recording stubs; tests/ft0df_ref.rs installs the same recorders as hooks. Each line:
 * op, case, return bits, hash of every field the ops write, call-trace hash. */
#include "ft/types.h"
#include "ft/forward.h"
#include "ft/fighter.h"
#include "ft/inlines.h"
#include "ft/ftcommon.h"
#include "ft/ft_0DF0.h"
#include "ft/ft_0DF1.h"
#include "ft/ftcoll.h"
#include "ft/ftdevice.h"
#include "ft/ftparts.h"
#include <melee/lb/types.h>
#include <sysdolphin/baselib/gobj.h>
#include <sysdolphin/baselib/jobj.h>

#include <Runtime/platform.h>

ftCommonData* p_ftCommonData;
struct Fighter_ShakeTable_t* Fighter_SmashChargeShakeTable;
float MSL_TrigF_80400770[1];
float MSL_TrigF_80400774[1];
void __assert(const char* file, u32 line, const char* cond) { (void) file; (void) line; (void) cond; for (;;) {} }

static u32 TR;
static void T(u32 v) { TR = TR * 31u + v; }
static u32 B(float f) { union { float f; u32 u; } c; c.f = f; return c.u; }
static int g_gm, g_it, g_p354;
static HSD_GObj GO;

bool gm_8016B0FC(void) { T(1); return g_gm; }
enum_t it_8026B30C(Item_GObj* g) { T(2); T(g != NULL); return g_it; }
void ftAnim_SetAnimRate(Fighter_GObj* g, float r) { T(3); T(g == &GO); T(B(r)); }
void ftCo_800C0200(Fighter* fp, int i) { (void) fp; T(4); T((u32) i); }
bool ftCo_800BFFD0(Fighter* fp, enum_t k, bool b) { (void) fp; T(5); T((u32) k); T((u32) b); return 0; }
bool Player_8003544C(s32 slot, bool sub) { T(6); T((u32) slot); T((u32) sub); return g_p354; }
void lb_80014574(u8 port, int id, int a, int b) { T(7); T(port); T((u32) id); T((u32) a); T((u32) b); }
void HSD_PadRumbleRemoveId(u8 port, int id) { T(8); T(port); T((u32) id); }
void ft_PlaySFX(Fighter* fp, enum_t id, u8 vol, u8 pan) { (void) fp; T(9); T((u32) id); T(vol); T(pan); }

#include "ft0df_extract.inc"

int printf(const char*, ...);
static void zero(void* p, unsigned n) { unsigned char* b = p; while (n--) *b++ = 0; }

static u32 rs;
static u32 rnd(void) { rs ^= rs << 13; rs ^= rs >> 17; rs ^= rs << 5; return rs; }
static float rg(void)
{
    union { float f; u32 u; } c;
    switch (rnd() % 16) {
    case 0: return 0.0f;
    case 1: return -0.0f;
    case 2: c.u = 0x7FC00000u; return c.f;
    case 3: return 1.0f;
    case 4: return -1.0f;
    default: return (float) ((int) (rnd() % 41) - 20) / 20.0f;
    }
}

static Fighter F, GF;
static HSD_GObj GG;
static ftCommonData CD;
static struct Fighter_ShakeTable_t ST;
static Vec2 SHK[8];
static HSD_GObj IG;
static HSD_JObj JO;
static HitCapsule HB;

static u32 H;
static void HM(u32 v) { H = H * 0x01000193u ^ v; }

static u32 state_hash(void)
{
    int i;
    SmashAttr* a = &F.smash_attrs;
    H = 0x811C9DC5u;
    HM((u32) a->state); HM(B(a->x2118_frames)); HM(B(a->x211C_holdFrame)); HM(B(a->x2120_damageMul));
    HM(B(a->x2124_frameSpeedMul)); HM((u32) a->x2128); HM(a->x212C); HM(a->x212D); HM((u32) a->x2130_sfxBool);
    HM(F.shield_hit.skip_update_pos); HM(F.reflect_hit.skip_update_pos); HM(F.absorb_hit.skip_update_pos);
    for (i = 0; i < 15; i++) HM(F.hurt_capsules[i].capsule.skip_update_pos);
    HM(F.x1064_thrownHitbox.owner == &GG); HM(F.x1064_thrownHitbox.owner == NULL);
    HM(F.x119C_teamUnk); HM(F.grabber_unk1);
    HM(HB.jobj == NULL); HM(B(HB.b_offset.x)); HM(B(HB.b_offset.y)); HM(B(HB.b_offset.z));
    for (i = 0; i < 12; i++) { HM((u32) (s32) F.x5F4_arr[i].prev); HM((u32) (s32) F.x5F4_arr[i].idx); }
    HM(F.x221D_b2);
    return H;
}

#define NOPS 40
#define NCASE 600

int main(void)
{
    int op, k, i;
    GO.user_data = &F;
    GG.user_data = &GF;
    p_ftCommonData = &CD;
    ST.x0 = SHK;
    Fighter_SmashChargeShakeTable = &ST;
    for (op = 0; op < NOPS; op++) {
        for (k = 0; k < NCASE; k++) {
            u32 ret = 0, u;
            float a, b;
            int i1, i2;
            Vec3 V;
            Vec2 sh;
            zero(&F, sizeof F); zero(&GF, sizeof GF); zero(&CD, sizeof CD); zero(&HB, sizeof HB);
            TR = 0;
            rs = 0xDF0u + (u32) op * 7919u + (u32) k * 104729u;
            F.smash_attrs.state = rnd() % 5;
            F.smash_attrs.x2118_frames = rg(); F.smash_attrs.x211C_holdFrame = rg();
            F.smash_attrs.x2120_damageMul = rg(); F.smash_attrs.x2124_frameSpeedMul = rg();
            F.smash_attrs.x2128 = (rnd() & 1) ? 0x7B : (int) (rnd() % 200);
            F.smash_attrs.x212C = rnd() % 8; F.smash_attrs.x212D = rnd() % 9;
            F.smash_attrs.x2130_sfxBool = rnd() & 1;
            F.frame_speed_mul = rg();
            F.input.held_buttons[0] = rnd() & 0x300;
            F.input.cstick[0].x = rg(); F.input.cstick[0].y = rg();
            F.input.cstick[1].x = rg(); F.input.cstick[1].y = rg();
            F.facing_dir = rg();
            F.item_gobj = (rnd() & 1) ? &IG : NULL;
            g_gm = rnd() & 1; g_it = rnd() % 3; g_p354 = rnd() & 1;
            F.x221F_b3 = rnd() & 1; F.x2224_b2 = rnd() & 1;
            F.x618_player_id = rnd() % 4; F.player_id = rnd() % 4; F.is_sub_fighter = rnd() & 1;
            CD.dash_smash_stick_threshold = rg(); CD.xCC = rg(); CD.xD4 = rg(); CD.xDC = rg();
            CD.xE0 = rg(); CD.sdi_min_stick_mag = rg(); CD.x7F4 = rg(); CD.x248 = rg();
            CD.x20_radians = rg(); CD.x7F8 = rg(); CD.x7FC = rg(); CD.x494 = rg(); CD.x98 = rg();
            CD.attackhi3_stick_threshold_y = rg(); CD.xB0 = rg(); CD.x31C = rg(); CD.x314 = rg();
            CD.tap_jump_threshold = rg(); CD.x7C8 = rg();
            for (i = 0; i < 8; i++) { SHK[i].x = rg(); SHK[i].y = rg(); }
            ST.x4 = rnd() % 9;
            F.x408.x28_colanim.i = (rnd() & 1) ? 0 : (int) rnd();
            F.x488.x28_colanim.i = (int) rnd();
            F.shield_hit.skip_update_pos = rnd() & 1;
            F.reflect_hit.skip_update_pos = rnd() & 1;
            F.absorb_hit.skip_update_pos = rnd() & 1;
            F.hurt_capsules_len = rnd() % 16;
            for (i = 0; i < 15; i++) F.hurt_capsules[i].capsule.skip_update_pos = rnd() & 1;
            GF.team = rnd(); GF.player_id = rnd();
            HB.jobj = &JO;
            V.x = rg(); V.y = rg(); V.z = rg();
            F.x5AC.model_num = rnd() % 13;
            for (i = 0; i < 12; i++) { F.x5F4_arr[i].prev = (s8) rnd(); F.x5F4_arr[i].idx = (s8) rnd(); }
            F.x221D_b2 = rnd() & 1;
            a = rg(); b = rg(); u = rnd(); i1 = rnd() % 12; i2 = (int) rnd();
            switch (op) {
            case 0: ftCo_800DEE84(&GO, u, a, b); break;
            case 1: ftCo_800DEEA8(&GO); break;
            case 2: ret = B(ftCo_800DEEB8(&F, a)); break;
            case 3: { Vec2* r = ftCo_800DEEE8(&F, &sh); ret = r == NULL ? 0xFFFFFFFFu : (B(sh.x) * 31u + B(sh.y)) ^ (r == &sh); } break;
            case 4: ftCo_800DEF38(&GO); break;
            case 5: ftCo_800DF0D0(&GO); break;
            case 6: ret = canUseCstick(&F); break;
            case 7: ret = ftCo_800DF1C8(&F); break;
            case 8: ret = ftCo_800DF21C(&F); break;
            case 9: ret = ftCo_800DF2D8(&F); break;
            case 10: ret = ftCo_800DF30C(&F); break;
            case 11: ret = ftCo_800DF3A8(&F); break;
            case 12: ret = ftCo_800DF3DC(&F); break;
            case 13: ret = ftCo_800DF478(&F); break;
            case 14: ret = ftCo_800DF50C(&F); break;
            case 15: ret = ftCo_800DF608(&F); break;
            case 16: ret = ftCo_800DF644(&F); break;
            case 17: ret = ftCo_800DF678(&F); break;
            case 18: ret = ftCo_800DF6F8(&F); break;
            case 19: ret = ftCo_800DF72C(&F); break;
            case 20: ret = ftCo_800DF79C(&F); break;
            case 21: ret = ftCo_800DF7F4(&F); break;
            case 22: ret = ftCo_800DF844(&F); break;
            case 23: ret = ftCo_800DF878(&F); break;
            case 24: ret = ftCo_800DF8B0(&F); break;
            case 25: ret = ftCo_800DF8E8(&F); break;
            case 26: ret = ftCo_800DF910(&F); break;
            case 27: ftColl_8007AEE0(&GO); break;
            case 28: ftColl_8007AEF8(&GO); break;
            case 29: ftColl_8007AF10(&GO); break;
            case 30: ftColl_8007AF28(&GO); break;
            case 31: ftColl_8007B8A8(&HB, &V); break;
            case 32: ftColl_8007B8CC(&F, &GG); break;
            case 33: ret = ftCo_800C0658(&F) == &F.x408 ? 0 : ftCo_800C0658(&F) == &F.x488 ? 1 : 2; break;
            case 34: ret = ftCo_800C0674(&GO) == &F.x408 ? 0 : ftCo_800C0674(&GO) == &F.x488 ? 1 : 2; break;
            case 35: ret = (u32) ftCo_800C0694(&F); break;
            case 36: ftParts_80074A4C(&GO, i1, i2); break;
            case 37: ret = (u32) ftParts_80074A74(&GO, i1); break;
            case 38: ftParts_80074A8C(&GO); break;
            case 39: ftParts_80074ACC(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
