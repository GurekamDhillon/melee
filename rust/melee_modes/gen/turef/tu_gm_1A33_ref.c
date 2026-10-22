/* C reference for rust/melee_modes/src/tu/gm_1A33.rs: the game's gm_1A33.c verbatim (so the text layout and the
 * static data are the game's own); the camera UI (gmcamera.c), the SIS text engine, the snap/menu queries and
 * the pad are stubbed here (logged; the Rust hooks log the same text).
 * Run: python rust/melee_modes/gen/turef/tu_shard537_gm.py tu_gm_1A33 */
#include "../../../../src/melee/gm/gm_1A33.c"
#include "tu_gm_common.h"

static HSD_Text TX[16];
static int g_ntx;
static u32 X0V;
static int g_snap0, g_snap1, g_mn, g_sp10, g_spC, g_2640;
static u64 g_trig;

static int txid(HSD_Text* t) { return t == NULL ? -1 : (int) (t - TX); }

void gmCamera_801A25C8(void) { lg("25C8;"); }
HSD_Text* gmCamera_801A2334(s32 a, f32 x, f32 y, f32 w, f32 h)
{
    int n = g_ntx++;
    memset(&TX[n], 0, sizeof TX[n]);
    lg("2334(%d,%08x,%08x,%08x,%08x)=x%d;", (int) a, (unsigned) bits(x), (unsigned) bits(y), (unsigned) bits(w),
       (unsigned) bits(h), n);
    return &TX[n];
}
void gmCamera_801A253C(s32* a, s32* b)
{
    lg("253C;");
    *a = g_sp10;
    *b = g_spC;
}
s32 gmCamera_801A2640(void) { lg("2640;"); return g_2640; }
void gmCamera_801A2650(void) { lg("2650;"); }
u8* gmCamera_801A2224(u8* p, u32 v)
{
    lg("2224(%s,%u);", p == (u8*) &TX[15] ? "digits" : "?", (unsigned) v);
    return p;
}
HSD_Text* HSD_SisLib_803A5ACC(int a, s32 b, f32 x, f32 y, f32 z, f32 w, f32 h)
{
    int n = g_ntx++;
    memset(&TX[n], 0, sizeof TX[n]);
    lg("5ACC(%d,%d,%08x,%08x,%08x,%08x,%08x)=x%d;", a, (int) b, (unsigned) bits(x), (unsigned) bits(y),
       (unsigned) bits(z), (unsigned) bits(w), (unsigned) bits(h), n);
    return &TX[n];
}
void HSD_SisLib_803A6368(HSD_Text* t, s32 v) { lg("6368(x%d,%d);", txid(t), (int) v); }
u8* HSD_SisLib_803A6530(s32 a, s32 b, s32 c)
{
    lg("6530(%d,%d,%d);", (int) a, (int) b, (int) c);
    return (u8*) &TX[15];
}
void HSD_SisLib_803A660C(s32 a, s32 b, s32 c) { lg("660C(%d,%d,%d);", (int) a, (int) b, (int) c); }
void HSD_SisLib_803A5CC4(HSD_Text* t) { lg("5CC4(x%d);", txid(t)); }
int lbSnap_8001D338(int i) { lg("snap(%d);", i); return i == 0 ? g_snap0 : g_snap1; }
bool mn_8022F218(void) { lg("mnF218;"); return g_mn; }
void mn_8022F268(void) { lg("mnF268;"); }
void gm_801A4B60(void) { lg("4B60;"); }
void lbAudioAx_80024030(int id) { lg("sfx(%d);", id); }
u64 gm_GetButtonsTriggered(u8 idx) { lg("trig(%d);", (int) idx); return g_trig; }

static const char* FUNC_NAMES[] = {"gmCamera_801A33BC", "OnFrame", "OnEnter", "OnExit"};
#define NFUNCS ((int) (sizeof FUNC_NAMES / sizeof FUNC_NAMES[0]))
static const int NCASES[] = {300, 1500, 300, 50};

static void setup_case(void)
{
    int i;
    u32 r;
    memset(TX, 0, sizeof TX);
    g_ntx = 0;
    for (i = 0; i < 3; i++) {
        r = rnd();
        gmCamera_VsCameraTextLayout.text[i] = (r % 2) ? &TX[10 + i] : NULL;
    }
    r = rnd();
    X0V = r;
    gmCamera_VsCameraTextLayout.x0 = &X0V;
    r = rnd();
    g_snap0 = (int) (r % 4 == 0);
    r = rnd();
    g_snap1 = (int) (r % 4 == 0);
    r = rnd();
    g_mn = (int) (r % 3 == 0);
    r = rnd();
    g_sp10 = (int) (r % 5);
    r = rnd();
    g_spC = (int) (r % 20000);
    r = rnd();
    g_2640 = (int) (r % 50);
    g_trig = 0;
    r = rnd();
    if (r % 2) g_trig |= HSD_PAD_A;
    r = rnd();
    if (r % 3 == 0) g_trig |= HSD_PAD_START;
    r = rnd();
    if (r % 2) g_trig |= PAD_BUTTON_B;
    r = rnd();
    if (r % 2) g_trig |= (u64) 1 << 40;
}

static void dump(int f, int i)
{
    int k;
    CameraVsData* d = &gmCamera_VsCameraTextLayout;
    printf("%d.%d %s|log=%s|x0=%d,%u|slots=%d,%d,%d", f, i, FUNC_NAMES[f], g_log, d->x0 == &X0V ? 1 : (d->x0 == NULL ? 0 : 2),
           (unsigned) X0V, txid(d->slot_a), txid(d->slot_b), txid(d->bottom_text));
    for (k = 0; k < g_ntx; k++) {
        printf("|X%d:%u,%u,%u,%08x,%08x", k, (unsigned) TX[k].default_alignment, (unsigned) TX[k].default_kerning,
               (unsigned) TX[k].default_fitting, (unsigned) bits(TX[k].font_size.x), (unsigned) bits(TX[k].font_size.y));
    }
    printf("\n");
}

int main(void)
{
    int f;
    int i;
    for (f = 0; f < NFUNCS; f++) {
        for (i = 0; i < NCASES[f]; i++) {
            int use;
            u32 r;
            setup_case();
            r = rnd();
            use = (int) (r % 4 != 0);
            g_loglen = 0;
            g_log[0] = 0;
            switch (f) {
            case 0: gmCamera_801A33BC(); break;
            case 1: gm_Scene_CameraVs_OnFrame(); break;
            case 2: gm_Scene_CameraVs_OnEnter(use ? (void*) &X0V : (void*) NULL); break;
            default: gm_Scene_CameraVs_OnExit(NULL); break;
            }
            dump(f, i);
        }
    }
    return 0;
}
