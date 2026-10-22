/* C reference for rust/melee_modes/src/tu/gm_1A9B.rs: the game's gm_1A9B.c verbatim (so the two timers and the
 * three movie tables are the game's own); the gobj / audio / movie / scene-query calls are stubbed here
 * (logged; the Rust hooks log the same text).
 * Run: python rust/melee_modes/gen/turef/tu_shard537_gm.py tu_gm_1A9B */
#include "../../../../src/melee/gm/gm_1A9B.c"
#include "tu_gm_common.h"

u8 HSD_SObjLib_804D7960 = 9;

static HSD_GObj GP[4];
static int g_ng;
static HSD_SObj SOBJ;
static int g_mode, g_bf050, g_level, g_ckind, g_remap, g_port;
static u64 g_trig;

static int gid(HSD_GObj* g) { return (int) (g - GP); }

HSD_GObj* GObj_Create(u16 c, u8 pl, u8 pr)
{
    int n = g_ng++;
    memset(&GP[n], 0, sizeof GP[n]);
    lg("gcreate(%d,%d,%d)=g%d;", (int) c, (int) pl, (int) pr, n);
    return &GP[n];
}
void HSD_SObjLib_803A55DC(HSD_GObj* g, u16 w, u16 h, int n) { lg("55DC(g%d,%d,%d,%d);", gid(g), (int) w, (int) h, n); }
void HSD_GObjObject_80390A70(HSD_GObj* g, u8 kind, void* obj)
{
    lg("gobj(g%d,%d,%s);", gid(g), (int) kind, obj == NULL ? "null" : "?");
}
void GObj_SetupGXLink(HSD_GObj* g, GObj_RenderFunc cb, u8 gx, u32 prio)
{
    lg("gxlink(g%d,%s,%d,%u);", gid(g), cb == lbMthp8001F928 ? "mthp" : "?", (int) gx, (unsigned) prio);
}
void lbAudioAx_80026F2C(u32 v) { lg("6F2C(%u);", (unsigned) v); }
void lbAudioAx_8002702C(u32 flags, u64 mask) { lg("702C(%u,%u);", (unsigned) flags, (unsigned) mask); }
void lbAudioAx_80027168(void) { lg("7168;"); }
void lbAudioAx_80027648(void) { lg("7648;"); }
int lbAudioAx_80023694(void) { lg("3694;"); return 0; }
int lbAudioAx_800237A8(enum_t id, int vol, int pan)
{
    lg("37A8(%x,%d,%d);", (unsigned) id, vol, pan);
    return 0;
}
void lbAudioAx_80024030(int id) { lg("sfx(%d);", id); }
void lbBgFlash_800209F4(void) { lg("09F4;"); }
void lbBgFlash_8002063C(int n) { lg("063C(%d);", n); }
HSD_SObj* lbMthp8001F890(HSD_GObj* g)
{
    lg("F890(g%d);", gid(g));
    return &SOBJ;
}
void lbMthp8001F928(HSD_GObj* g, int x) { (void) g; (void) x; }
void lbMthp8001FAA0(const char* f, int w, int h) { lg("FAA0(%s,%d,%d);", f, w, h); }
CharacterKind gm_801BEFB0(void) { lg("EFB0;"); return (CharacterKind) g_ckind; }
int gm_801BF010(void) { lg("F010;"); return g_port; }
int gm_801BF050(void) { lg("F050;"); return g_bf050; }
u8 gm_GetCurrentGameMode(void) { lg("mode;"); return (u8) g_mode; }
int gm_8017DFF4(int v) { lg("DFF4(%d);", v); return g_level; }
void gm_801A6630(int v) { lg("6630(%d);", v); }
int gm_MexVanillaKind(int kind, int ck)
{
    int r = (kind + g_remap) % 26;
    lg("mex(%d,%d)=%d;", kind, ck, r);
    return r;
}
u64 gm_GetButtonsTriggered(u8 idx) { lg("trig(%d);", (int) idx); return g_trig; }

static const char* FUNC_NAMES[] = {"OnEnter", "OnFrame"};
#define NFUNCS 2
static const int NCASES[] = {520, 2000};
static const int MODES[] = {GM_CLASSIC_GOVER, GM_ADVENTURE_GOVER, 23, GM_DEBUG_GOVER};

int main(void)
{
    int f;
    int i;
    for (f = 0; f < NFUNCS; f++) {
        for (i = 0; i < NCASES[f]; i++) {
            u32 r;
            g_ng = 0;
            memset(GP, 0, sizeof GP);
            memset(&SOBJ, 0, sizeof SOBJ);
            r = rnd();
            SOBJ.x40 = r;
            r = rnd();
            g_remap = (int) (r % 3);
            r = rnd();
            g_bf050 = (int) (GM_CLASSIC_GOVER + r % 3);
            r = rnd();
            g_level = (int) (r % 6);
            r = rnd();
            g_port = (int) (r % 6);
            r = rnd();
            g_trig = 0;
            if (r % 2) g_trig |= PAD_BUTTON_A;
            r = rnd();
            if (r % 3 == 0) g_trig |= PAD_BUTTON_START;
            r = rnd();
            if (r % 2) g_trig |= (u64) 1 << 33;
            if (f == 0) {
                g_ckind = i % 26;
                g_mode = MODES[(i / 26) % 4];
            } else {
                r = rnd();
                g_mode = (r % 2) ? GM_DEBUG_GOVER : GM_CLASSIC_GOVER;
                r = rnd();
                gmRegend_InputDelayTimer = (u8) ((r % 3 == 0) ? 0 : r % 4);
                r = rnd();
                gmRegend_ExitTimer = (u8) ((r % 3 == 0) ? 0 : (r >> 4) % 3);
                g_ckind = 0;
            }
            g_loglen = 0;
            g_log[0] = 0;
            if (f == 0) {
                gm_Scene_Congrats_OnEnter(NULL);
            } else {
                gm_Scene_Congrats_OnFrame();
            }
            printf("%d.%d %s|log=%s|t=%u,%u|sobj=%08x,%08x,%x|g0=%llx\n", f, i, FUNC_NAMES[f], g_log,
                   (unsigned) gmRegend_InputDelayTimer, (unsigned) gmRegend_ExitTimer, (unsigned) bits(SOBJ.x10),
                   (unsigned) bits(SOBJ.x14), (unsigned) SOBJ.x40, (unsigned long long) GP[0].gxlink_prios);
        }
    }
    return 0;
}
