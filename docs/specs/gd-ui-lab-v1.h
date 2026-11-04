/* gd-ui-v1 / gd-lab-v1 shared constants (docs/rust-port/notes/wasm-ui-lab-api.md).
 * Rust mirror: rust/melee_modhost/src/pres.rs and src/ui/ (gd_ui host, implemented); rust/mods/gd_ui_enc (encoder).
 * gd_match_setup and gd_lab_fighter_v1 are added by the Lab half / the match-setup step. */
#ifndef GD_UI_LAB_V1_H
#define GD_UI_LAB_V1_H

/* ABI versions (major<<16 | minor). A mod that declares any UI (LAB) feature bit, required or optional, exports
 * the i32 global gdmod_ui_abi (gdmod_lab_abi) whose VALUE is the version: major must equal the host's, minor may
 * be lower. */
#define GD_UI_ABI_VERSION  (1u << 16)
#define GD_LAB_ABI_VERSION (1u << 16)

/* features bits, extending GD_FEATURE_* in gdmod.h */
enum {
    GD_FEATURE_UI_DRAW      = 1u << 4,
    GD_FEATURE_UI_MENU      = 1u << 5,
    GD_FEATURE_UI_INPUT     = 1u << 6,
    GD_FEATURE_UI_SCENE     = 1u << 7,
    GD_FEATURE_UI_SIM_INPUT = 1u << 8,
    GD_FEATURE_UI_HTML      = 1u << 9,  /* reserved, refused by v1 */
    GD_FEATURE_LAB          = 1u << 10,
    GD_FEATURE_LAB_STATE    = 1u << 11,
    GD_FEATURE_LAB_INPUT    = 1u << 12,
    GD_FEATURE_LAB_CPU      = 1u << 13,
    GD_FEATURE_LAB_OVERLAY  = 1u << 14,
    GD_FEATURE_UI_VECTOR    = 1u << 15
};

/* results: 0 ok, negative = error */
enum {
    GD_OK       = 0,
    GD_EINVAL   = -1,
    GD_ECAP     = -2,
    GD_EOFFLINE = -3, /* not allowed in this session (normal/ranked netplay, replay, synctest) */
    GD_ESTATE   = -4,
    GD_EFULL    = -5
};

/* ---- gd-ui-v1 (implemented: rust/melee_modhost/src/ui) ----
 * Colours are u32 0xRRGGBBAA, straight alpha. Canvas units: the 640x480 4:3 band, or window pixels after SPACE.
 * A presentation mod imports from module "gd_ui" only, declares UI bits, has hook_count 0 and exports gd_ui_frame. */

/* command buffer records (A1): u32 head = op | words << 16, then `words` payload words; LE, 4-byte aligned.
 * One linear scan validates the whole buffer; the first bad record refuses all of it (GD_EINVAL, VECTOR: GD_ECAP). */
enum {
    GD_UI_OP_LAYER  = 1,  /* i32 layer, i32 z */
    GD_UI_OP_RECT   = 2,  /* f32 x,y,w,h, u32 rgba, f32 shear (top-edge x offset per unit of height) */
    GD_UI_OP_TEX    = 3,  /* u32 tex, f32 x,y,w,h, f32 u0,v0,u1,v1 (0..1 of the texture), u32 rgba */
    GD_UI_OP_TEXT   = 4,  /* f32 x,y, i32 role, u32 rgba, i32 align, f32 max_w (0: none), u32 len, bytes padded to 4 */
    GD_UI_OP_KIT    = 5,  /* u32 kit_id, f32 x,y, f32 scale, u32 rgba */
    GD_UI_OP_CLIP   = 6,  /* f32 x,y,w,h (intersected with the enclosing clip; depth 8) */
    GD_UI_OP_UNCLIP = 7,  /* no payload */
    GD_UI_OP_VECTOR = 8,  /* u32 path_id, f32 affine[6], u32 rgba: UI_VECTOR only (no Vello in this build) */
    GD_UI_OP_SLICE  = 9,  /* u32 slice_id, f32 x,y,w,h, u32 rgba: a host 9-slice piece (theme radius/border/glow) */
    GD_UI_OP_SPACE  = 10  /* i32 space, f32 scale: GAP-0; following coordinates and sizes times scale; not inside a CLIP */
};
enum { GD_UI_SPACE_BAND = 0, GD_UI_SPACE_WINDOW = 1 };
/* layers, drawn in this order; within a layer by z, then mod load order, then submission order (a total order) */
enum { GD_UI_LAYER_SCENE = 0, GD_UI_LAYER_HUD_UNDER, GD_UI_LAYER_HUD, GD_UI_LAYER_OVERLAY, GD_UI_LAYER_MENU,
       GD_UI_LAYER_MODAL, GD_UI_LAYER_DEBUG };
/* text roles (base 13 px): label 11 (uppercased), value 12, body 13, big 18 bold, title 14 bold, toast 13 bold */
enum { GD_UI_ROLE_LABEL = 0, GD_UI_ROLE_VALUE, GD_UI_ROLE_BODY, GD_UI_ROLE_BIG, GD_UI_ROLE_TITLE, GD_UI_ROLE_TOAST };
enum { GD_UI_ALIGN_LEFT = 0, GD_UI_ALIGN_CENTER = 1, GD_UI_ALIGN_RIGHT = 2 };
enum { GD_UI_SLICE_FILL_R4 = 1, GD_UI_SLICE_FILL_R6, GD_UI_SLICE_FILL_R14, GD_UI_SLICE_LINE_R4, GD_UI_SLICE_LINE_R6,
       GD_UI_SLICE_LINE_R14_3, GD_UI_SLICE_GLOW_R6, GD_UI_SLICE_FILL_R1 };
/* kit pieces by name (gd_ui_kit_id): "ui.dot" 10x10, "ui.ring" 56x56, "ui.diamond" 9x9, "ui.knob" 32x32, "ui.hatch" 9x14 */

/* theme tokens (gd_ui_theme / gd_ui_token), lab-ui-spec.md section 7 */
enum { GD_UI_THEME_A = 0, GD_UI_THEME_C, GD_UI_THEME_E, GD_UI_THEME_G };
enum { GD_UI_TOK_PANEL_BG = 0, GD_UI_TOK_PANEL_BG_HUD, GD_UI_TOK_BORDER, GD_UI_TOK_TEXT, GD_UI_TOK_MUTED, GD_UI_TOK_ACCENT,
       GD_UI_TOK_ACCENT_INK, GD_UI_TOK_DIVIDER, GD_UI_TOK_KEY_BORDER, GD_UI_TOK_KEY_TEXT, GD_UI_TOK_P1, GD_UI_TOK_P2,
       GD_UI_TOK_CELL_IDLE, GD_UI_TOK_CELL_STARTUP, GD_UI_TOK_CELL_ACTIVE, GD_UI_TOK_CELL_RECOVERY, GD_UI_TOK_CELL_STUN,
       GD_UI_TOK_CELL_CAN_ACT, GD_UI_TOK_SLICE_FILL, GD_UI_TOK_SLICE_BORDER, GD_UI_TOK_BORDER_W, GD_UI_TOK_SHADOW,
       GD_UI_TOK_SHADOW_X, GD_UI_TOK_SHADOW_Y, GD_UI_TOK_GLOW, GD_UI_TOK_BASE_SIZE };

/* nodes (A2) */
enum { GD_UI_NODE_PANEL = 0, GD_UI_NODE_LIST, GD_UI_NODE_TABS, GD_UI_NODE_BUTTON, GD_UI_NODE_TOGGLE, GD_UI_NODE_SLIDER,
       GD_UI_NODE_CHOICE, GD_UI_NODE_LABEL };
enum { GD_UI_NODE_HIDDEN = 1, GD_UI_NODE_DISABLED = 2, GD_UI_NODE_NO_WRAP = 4 };
enum { GD_UI_EV_ACTIVATE = 0, GD_UI_EV_CHANGE, GD_UI_EV_BACK, GD_UI_EV_FOCUS };

/* input (A4). KEYBOARD: Offline and LabNetplay (GAP-14: local-only input); INJECT: Offline only;
 * EXCLUSIVE: not in a Netplay/Synctest match. gd_ui_pad writes 16 bytes: PADStatus (12) + u16 new presses + pad. */
enum { GD_UI_CAP_EXCLUSIVE = 1, GD_UI_CAP_SHARED = 2, GD_UI_CAP_KEYBOARD = 4, GD_UI_CAP_INJECT = 8 };
/* gd_ui_keys events: kind << 24 | value; kind 1 down / 2 up (value GD_UI_KEY_*), 3 char (value = code point) */
enum { GD_UI_KEY_ENTER = 1, GD_UI_KEY_ESCAPE, GD_UI_KEY_BACKSPACE, GD_UI_KEY_UP, GD_UI_KEY_DOWN, GD_UI_KEY_LEFT,
       GD_UI_KEY_RIGHT, GD_UI_KEY_TAB };
enum { GD_UI_SESSION_OFFLINE = 0, GD_UI_SESSION_NETPLAY, GD_UI_SESSION_SYNCTEST, GD_UI_SESSION_REPLAY,
       GD_UI_SESSION_LAB_NETPLAY };
#define GD_UI_SCENE_RETAIL 0x80000000u /* gd_ui_scene_goto(GD_UI_SCENE_RETAIL | major << 8 | minor) */

/* imports (module "gd_ui"); u32 handles, 0 invalid */
int      gd_ui_submit(const void* buf, unsigned len);
float    gd_ui_measure(const char* s, unsigned len, int role);
unsigned gd_ui_kit_id(const char* name, unsigned len);
unsigned gd_ui_tex_load(const char* path, unsigned len);        /* .png in the mod's package dir, <= 512x512 */
int      gd_ui_band_rect(float out[6]);                          /* GAP-0: band x,y,w,h and window w,h in pixels */
int      gd_ui_theme(int theme);
unsigned gd_ui_token(int tok);
int      gd_ui_session(void);
int      gd_ui_scene(void);                                      /* the custom scene on top, 0 = retail */
int      gd_ui_retail_scene(void);                               /* major << 8 | minor */
unsigned gd_ui_node(unsigned parent, int kind, unsigned id);
int      gd_ui_node_label(unsigned n, const char* s, unsigned len);
int      gd_ui_node_value(unsigned n, int v);
int      gd_ui_node_get(unsigned n);
int      gd_ui_node_range(unsigned n, int lo, int hi, int step);
int      gd_ui_node_choices(unsigned n, const char* nul_separated, unsigned len);
int      gd_ui_node_flags(unsigned n, unsigned flags);
int      gd_ui_node_rect(unsigned root, float x, float y, float w, float h); /* placement of a root (band units) */
void     gd_ui_node_free(unsigned n);
int      gd_ui_focus(unsigned n);
unsigned gd_ui_focused(void);
int      gd_ui_show(unsigned root, int layer, int z);
void     gd_ui_hide(unsigned root);
unsigned gd_ui_capture(unsigned port_mask, unsigned flags);
void     gd_ui_release(unsigned token);
int      gd_ui_pad(unsigned port, void* out16);
int      gd_ui_inject(unsigned port, const void* pad12);
int      gd_ui_keys(unsigned* out, unsigned max);
unsigned gd_ui_scene_register(const char* name, unsigned len);
int      gd_ui_scene_replace(int major, int minor, unsigned scene);
int      gd_ui_scene_goto(unsigned scene_or_retail);
int      gd_ui_scene_back(void);
int      gd_ui_match_start(const void* setup, unsigned len);     /* GD_ESTATE until gd_match_setup lands */
/* exports: int gd_ui_init(void); void gd_ui_frame(unsigned present_no) (required with UI_DRAW);
 * void gd_ui_event(unsigned node, int ev, int value); void gd_ui_scene_enter(unsigned); void gd_ui_scene_leave(unsigned) */

#endif
