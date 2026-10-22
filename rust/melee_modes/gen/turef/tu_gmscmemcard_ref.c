/* C reference for rust/melee_modes/src/tu/gmscmemcard.rs: the game's gmscmemcard.c verbatim (so enter_data and
 * the static helpers are the game's own); the notice window (gm_1ADD), the card library, the pad and the
 * TARGET_PC port hooks are stubbed here (logged; the Rust hooks log the same text). Every case seeds
 * enter_data and the stubs' answers from the stream, runs one function and prints the log, the return value
 * and enter_data. Run: python rust/melee_modes/gen/turef/tu_shard537_gm.py tu_gmscmemcard */
#include "../../../../src/melee/gm/gmscmemcard.c"
#include "tu_gm_common.h"

HSD_PadStatus HSD_PadCopyStatus[4];
DbLKind DbLevel;

static u32 g_pad, g_status;
static int g_probe, g_cbbc, g_cc4c, g_c8bc, g_b8c8, g_us, g_skip, g_mexen, g_settings;

void memzero(void* mem, ssize_t size) { memset(mem, 0, (size_t) size); }
u32 gm_801AEDC8(void) { lg("pad;"); return g_pad; }
void gm_801AE848(int a) { lg("848(%d);", a); }
bool gm_801ADE1C(int a, int b, float x, float y)
{
    lg("DE1C(%d,%d,%08x,%08x);", a, b, (unsigned) bits(x), (unsigned) bits(y));
    return 1;
}
void gm_801AE44C(int a, float v) { lg("E44C(%d,%08x);", a, (unsigned) bits(v)); }
void gm_801AE544(int a, float v) { lg("E544(%d,%08x);", a, (unsigned) bits(v)); }
bool gm_801AE050(int a, int b, int c, float x, float y)
{
    lg("E050(%d,%d,%d,%08x,%08x);", a, b, c, (unsigned) bits(x), (unsigned) bits(y));
    return 1;
}
void gm_801AE640(int a, bool b) { lg("E640(%d,%d);", a, (int) b); }
void gm_801AE74C(int a, bool b) { lg("E74C(%d,%d);", a, (int) b); }
void gm_801ADDD8(void) { lg("DDD8;"); }
int lbCardNew_ProbeEx(int chan) { lg("probe(%d);", chan); return g_probe; }
void lbCardNew_AllocWorkArea(void) { lg("alloc;"); }
void lbCardGame_LoadArchive(int a) { lg("loadarch(%d);", a); }
void lbCardGame_SetCardStatus(LbCardStatus s) { lg("setstatus(%d);", (int) s); }
void lbCardGame_SaveChanges(void) { lg("save;"); }
u32 lb_8001C87C(void) { lg("c87C;"); return g_status; }
lbCardResult lb_8001CBBC(void) { lg("cbbc;"); return (lbCardResult) g_cbbc; }
int lb_8001CC4C(void) { lg("cc4c;"); return g_cc4c; }
int lb_8001C8BC(void) { lg("c8bc;"); return g_c8bc; }
bool lb_8001B8C8(int chan) { lg("b8c8(%d);", chan); return g_b8c8; }
bool lbLang_IsSavedLanguageUS(void) { return g_us; }
void gmMainLib_8015FA34(s32 v) { lg("fa34(%d);", (int) v); }
void gm_801A4B60(void) { lg("4B60;"); }
void lbAudioAx_80024030(int id) { lg("sfx(%d);", id); }
void SceneReport_Memcard(int d, int o) { lg("rep(%d,%d);", d, o); }
void SceneReport_Cursor(const char* what, int a, int b) { lg("cursor(%s,%d,%d);", what, a, b); }
int SceneLaunch_SkipMemcard(void) { lg("skip;"); return g_skip; }
int Mex_Enabled(const char* n) { lg("mex(%s);", n); return g_mexen; }
int Settings_Int(const char* k, int d) { lg("set(%s,%d);", k, d); return g_settings; }

static const char* FUNC_NAMES[] = {"gm_801AEE6C",   "gm_801AF0D4",           "gm_801AF250",
                                   "OnFrame",       "gm_Scene_MemCard_OnEnter", "gm_Scene_MemCard_OnExit"};
#define NFUNCS ((int) (sizeof FUNC_NAMES / sizeof FUNC_NAMES[0]))
static const int NCASES[] = {300, 500, 500, 3000, 300, 100};

static const int PADBITS[] = {0, 1, 2, 3, 16, 17, 18, 19, 8, 12};

static void setup_case(void)
{
    int k;
    u32 r;
    g_pad = 0;
    for (k = 0; k < 10; k++) {
        r = rnd();
        if (r % 3 == 0) {
            g_pad |= 1u << PADBITS[k];
        }
    }
    r = rnd();
    enter_data.decision = (int) (r % 22);
    r = rnd();
    enter_data.unk0.unk0 = (int) (r % 3);
    r = rnd();
    enter_data.unk0.chan = (u8) (r % 2);
    r = rnd();
    enter_data.unk0.mode_id = (u8) r;
    r = rnd();
    enter_data.unk8.unk0 = (int) (r % 2);
    r = rnd();
    enter_data.unk8.mode_id = (u8) r;
    r = rnd();
    enter_data.unk10 = (int) (r % 2);
    r = rnd();
    enter_data.unk18 = (int) (r % 2);
    r = rnd();
    enter_data.unk1C = (u8) (r % 2);
    r = rnd();
    gm_804D6870 = (u8) (r % 2);
    r = rnd();
    gm_804D6872 = (u16) (6 + r % 19);
    r = rnd();
    g_probe = (int) (r % 2);
    r = rnd();
    g_status = r % 17;
    r = rnd();
    g_cbbc = (int) ((const int[]){0, 2, 0xF, 0xD, 1, 3, 5}[r % 7]);
    r = rnd();
    g_cc4c = (int) (r % 2);
    r = rnd();
    g_c8bc = (int) (r % 2);
    r = rnd();
    g_b8c8 = (int) (r % 2);
    r = rnd();
    g_us = (int) (r % 2);
    r = rnd();
    DbLevel = (DbLKind) (r % 5);
    r = rnd();
    g_skip = (int) (r % 4 == 0);
    r = rnd();
    g_mexen = (int) (r % 2);
    r = rnd();
    g_settings = (int) (r % 2);
    HSD_PadCopyStatus[0].button = 0;
    HSD_PadCopyStatus[0].trigger = 0;
    r = rnd();
    if (r % 2) HSD_PadCopyStatus[0].button |= HSD_PAD_L;
    r = rnd();
    if (r % 2) HSD_PadCopyStatus[0].button |= HSD_PAD_R;
    r = rnd();
    if (r % 2) HSD_PadCopyStatus[0].button |= HSD_PAD_A;
    r = rnd();
    if (r % 2) HSD_PadCopyStatus[0].button |= HSD_PAD_B;
    r = rnd();
    if (r % 2) HSD_PadCopyStatus[0].trigger |= HSD_PAD_L;
    r = rnd();
    if (r % 2) HSD_PadCopyStatus[0].trigger |= HSD_PAD_R;
}

static void dump(int f, int i, int ret, int ex0, int exm)
{
    printf("%d.%d %s|log=%s|ret=%d|ed=%d,%u,%u,%d,%u,%d,%d,%d,%u|g=%u,%u|ex=%d,%d\n", f, i, FUNC_NAMES[f], g_log, ret,
           (int) enter_data.unk0.unk0, (unsigned) enter_data.unk0.chan, (unsigned) enter_data.unk0.mode_id,
           (int) enter_data.unk8.unk0, (unsigned) enter_data.unk8.mode_id, (int) enter_data.unk10,
           (int) enter_data.decision, (int) enter_data.unk18, (unsigned) enter_data.unk1C, (unsigned) gm_804D6870,
           (unsigned) gm_804D6872, ex0, exm);
}

int main(void)
{
    int f;
    int i;
    for (f = 0; f < NFUNCS; f++) {
        for (i = 0; i < NCASES[f]; i++) {
            int ret = 0;
            enterData in;
            exitData out;
            int use_in;
            u32 a;
            u32 b;
            u32 c;
            u32 r;
            setup_case();
            a = rnd();
            b = rnd();
            c = rnd();
            r = rnd();
            use_in = (int) (r % 4 != 0);
            memset(&in, 0, sizeof in);
            in.unk0.unk0 = (int) (a % 3);
            in.unk0.chan = (u8) (b % 2);
            in.unk0.mode_id = (u8) c;
            in.decision = 99;
            out.unk0 = 77;
            out.mode_id = 66;
            g_loglen = 0;
            g_log[0] = 0;
            switch (f) {
            case 0: gm_801AEE6C((int) (a % 4), (int) (b % 25), (int) (c % 6) + 2); break;
            case 1: ret = gm_801AF0D4(); break;
            case 2: gm_801AF250(); break;
            case 3: gm_Scene_MemCard_OnFrame(); break;
            case 4: gm_Scene_MemCard_OnEnter(use_in ? &in : NULL); break;
            default: gm_Scene_MemCard_OnExit(use_in ? &out : NULL); break;
            }
            dump(f, i, ret, (int) out.unk0, (int) out.mode_id);
        }
    }
    return 0;
}
