"""Oracle patch for src/melee/gm/gmfrontend.c (applied to a copy; see gen/tu/run.py).

gmfrontend.c `#include`s fifteen `gmfrontend_*.inc` files into its own translation unit: the layout
player, the kit, the panel art, the mouse, the menu tree, the Atlas adapters, the online lobby, the
select screens and the settings pages (about 700 KB of C that are separate units in the port). The
file's own functions call into them. For the oracle those units must be recorded callees, exactly as
every other unported callee is, so this patch cuts the includes and puts their interface in their
place: every function the .c calls becomes an extern prototype (recorded stub), and each state
object the .c touches (`fm`, `fk`, `fl`, `fp`, `fms`, `fte`, `fss`, `ff`, `kit`) becomes an extern
VIEW holding only the members gmfrontend.c reads or writes, in their real order. The Rust port
(tu/gmfrontend.rs, section "inc-owned state views") declares the same views field for field.

Nothing else of the file changes: its own statics, tables and functions are compiled as they are.
"""
import re

DECLS = r'''
/* ---- oracle stand-ins for the gmfrontend_*.inc interface (see gen/tu/patches/gmfrontend.py) ---- */
enum { FM_HUB, FM_LIST };
#define FM_FADE 8
enum { FSP_VIDEO, FSP_AUDIO, FSP_CONTROLS, FSP_ONLINE, FSP_MODS, FSP_GAMEPLAY, FSP_COUNT };
#define SEL_MAIN_MODS 0x45
typedef struct FePanelRect { float x, y, w, h; } FePanelRect;
typedef struct FeMenu {
    u8 kind;
    u8 style;
    u8 hero;
    u8 back_kind, back_sel;
    u8 port_all;
    const char* title;
    const void* items;
    int n;
    const char* atlas_id;
} FeMenu;
typedef struct FpTile { int q_plate, q_face; } FpTile;
extern struct FmView {
    bool active;
    u8 start_kind, start_sel;
    bool native_req;
    u8 native_kind, native_sel;
    const FeMenu* menu;
    int nvis;
    int fade;
} fm;
extern struct FkView { bool on; int nrows; int tile[12]; int scroll; } fk;
extern struct FlView { bool on; } fl;
extern struct FpView { bool ok; int frame; FpTile tile[16]; } fp;
extern struct FmsView { bool moved; bool click; int wheel; } fms;
extern struct FteView { bool on; } fte;
extern struct FssView { const char* want_label; } fss;
extern struct FfView { int state; } ff;
extern struct KitView { int state; } kit;
extern bool fa_mods_door;
extern bool fa_page_from_main;
extern int fm_scroll;
extern int fcr_port;
extern const FrontendScreen fe_screen_remap;
extern const FrontendScreen fe_screen_settings[FSP_COUNT];
extern void Controls_Menu(int editor);
extern int Ui_Ready(void);
void fa_room_end(void);
void fa_room_leaving(void);
bool fa_room_on(void);
void fcr_back(void);
bool fcr_frame(void);
void fe_panel_art_frame(const FePanelRect* bounds, float corner, float edge, unsigned cuts, bool filled);
void fe_panel_art_image(int tex, const FePanelRect* bounds, float authored_w, float authored_h, float cx, float cy, GXColor fallback);
void fe_panel_rect(FePanelRect* r, float x, float y, float w, float h);
void fe_panel_solid(float x, float y, float w, float h, GXColor colour);
void fe_settings_apply_online(void);
void ff_load(void);
void fk_build(bool animate_in);
void fk_bump(int dir);
void fk_exit(void);
void fk_frame(void);
void fk_note_dir(int dir);
void fl_exit(void);
void fl_frame(void);
void fl_frame_anim(void);
void fl_open(int art);
void fm_leave_scene(u8 mode);
void fm_position_for(u8 previous_mode, u8* kind, u8* sel);
bool fm_replaced(int kind);
void fm_scene_enter(void);
void fm_scene_exit(void);
void fm_scene_frame(void);
bool fms_in_quad(int q);
u32 fms_menu_bits(void);
void fms_poll(void);
void fp_draw(void);
void fp_evaluate(int f);
void fp_free_textures(void);
void fp_scroll_cue(int scroll, int count);
void fsm_fill(void);
void fss_exit(void);
bool fss_frame(void);
bool fss_intercepts(const FrontendScreen* s);
void fss_switch(const FrontendScreen* s);
bool fte_frame(void);
void kit_load(void);
'''

# forward declarations in gmfrontend.c of functions that live in the .inc files
FWD = [
    "static void fl_code_reset(void);",
    "static void fl_rejoin_reset(void);",
    "static void fl_lobby_notice(const char* s);",
    "static void fss_close(void);",
    "static bool fas_active(void);",
    "static bool fe_is_settings(const FrontendScreen* s);",
    "static bool fe_is_controls_help(const FrontendScreen* s);",
    "static void fe_rules_from_menus(void);",
    "static void fe_rules_resume(void);",
]


# Two function-local statics (a lazily cached answer from the host) become file-scope variables
# of the same initial value, so the oracle can reset them per case and the port registers them as
# globals. The code is otherwise the same text.
CACHES = [
    ("""static int fe_enabled(void)
{
    static int enabled = -1;
    if (enabled < 0) {
        extern int PcFrontendEnabled(void);
        enabled = PcFrontendEnabled();
    }
    return enabled;
}""", """static int fe_enabled_cache = -1;
static int fe_enabled(void)
{
    if (fe_enabled_cache < 0) {
        extern int PcFrontendEnabled(void);
        fe_enabled_cache = PcFrontendEnabled();
    }
    return fe_enabled_cache;
}"""),
    ("""static int fe_menus_on(void)
{
    static int on = -1;
    if (on < 0) {
        extern int PcFrontendMenusEnabled(void);
        on = fe_enabled() && PcFrontendMenusEnabled();
    }
    return on;
}""", """static int fe_menus_on_cache = -1;
static int fe_menus_on(void)
{
    if (fe_menus_on_cache < 0) {
        extern int PcFrontendMenusEnabled(void);
        fe_menus_on_cache = fe_enabled() && PcFrontendMenusEnabled();
    }
    return fe_menus_on_cache;
}"""),
]


def patch(t):
    first = [True]

    def rep(m):
        if first[0]:
            first[0] = False
            return DECLS
        return ""

    t, n = re.subn(r'#include "gmfrontend_(?!items)\w+\.inc"\n', rep, t)
    assert n == 15, n
    for a, b in CACHES:
        assert t.count(a) == 1, a[:40]
        t = t.replace(a, b)
    for f in FWD:
        assert t.count(f) == 1, f
        t = t.replace(f, f[len("static "):])
    return t
