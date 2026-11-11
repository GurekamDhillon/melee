/* gd-ui-v1 / gd-lab-v1 shared constants (docs/rust-port/notes/wasm-ui-lab-api.md).
 * Rust mirror: rust/melee_modhost/src/pres.rs and src/ui/ (gd_ui host, implemented); rust/mods/gd_ui_enc (encoder).
 * gd-lab-v1 (Lab half, implemented): rust/melee_modhost/src/lab (imports, commands), rust/melee/src/lab.rs (sim side).
 * gd_match_setup is added by the match-setup step. */
#ifndef GD_UI_LAB_V1_H
#define GD_UI_LAB_V1_H

/* ABI versions (major<<16 | minor). A mod that declares any UI (LAB) feature bit, required or optional, exports
 * the i32 global gdmod_ui_abi (gdmod_lab_abi) whose VALUE is the version: major must equal the host's, minor may
 * be lower. */
#define GD_UI_ABI_VERSION  ((1u << 16) | 3u) /* 1.1: text style bits (GD_UI_TEXT_*); 1.2: UI_HOST catalogs + requests; 1.3: Nucleus lists, requests, gd_ui_host_pic */
#define GD_LAB_ABI_VERSION ((1u << 16) | 1u) /* 1.1: gd_geno_* (Geno queries, offline-only Geno commands) */

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
    GD_FEATURE_UI_VECTOR    = 1u << 15,
    GD_FEATURE_UI_HOST      = 1u << 16  /* v1.2: gd_ui_list_* / gd_ui_request (the in-game browser) */
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
/* v1.1: the role word of TEXT / gd_ui_measure may carry style bits on top of the role (bits 0-7). Fonts: Inter
 * (sans) and JetBrains Mono, regular and bold, drawn as SDF glyphs (any size stays sharp). A v1.0 role (0..5) draws
 * as before: sans, the role's size and weight. */
#define GD_UI_TEXT_REGULAR (1 << 8)          /* weight override: regular */
#define GD_UI_TEXT_BOLD    (2 << 8)          /* weight override: bold */
#define GD_UI_TEXT_MONO    (1 << 10)         /* JetBrains Mono instead of Inter */
#define GD_UI_TEXT_SIZE(px) (((px) & 0xFF) << 16) /* line height in canvas units, 1..255 (0: the role's size) */
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
int      gd_ui_log(const char* s, unsigned len);                  /* a log line (headless checks); <= 16 per present */
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
int      gd_ui_pad(unsigned port, void* out16);                   /* GD_ESTATE while a host overlay (settings) holds the pads */
int      gd_ui_pad_present(unsigned port);                       /* 1 a controller is plugged into the port, 0 none; no capture needed */
int      gd_ui_inject(unsigned port, const void* pad12);
int      gd_ui_keys(unsigned* out, unsigned max);
unsigned gd_ui_scene_register(const char* name, unsigned len);
int      gd_ui_scene_replace(int major, int minor, unsigned scene);
int      gd_ui_scene_goto(unsigned scene_or_retail);
int      gd_ui_scene_back(void);
int      gd_ui_match_start(const void* setup, unsigned len);     /* GD_ESTATE until gd_match_setup lands */
/* ---- gd-ui-v1.2 (UI_HOST): host catalogs and requests (wasm-ui-lab-api.md A6; rust/mods/browser) ----
 * The play flow fills each catalog (records of NUL-separated UTF-8 fields) at the first present and after a request
 * that changes it; gd_ui_list_info returns generation << 16 | count, so a mod re-reads only on a change.
 *   MODES   id, group, title, description, available (0/1), note
 *   MODS    id, name, description, kind, enabled next launch (0/1), enabled at this launch (0/1), status
 *   REPLAYS n (1-based), kind (mrpl/slp), date, stage, fighters, m:ss, file name, note
 *   SKINS   slot (plfxnr), slot file stem (PlFxNr), skin name, selected (0/1), verdict when selected
 *   STATUS  one record: code (0 info, 1 error, 2 done: the browser closes), text (the last request's result)
 *   ONLINE  one record: server configured (0/1), help text
 * Requests are queued (<= 8 per present, arg <= 256 bytes, NUL-separated fields) and applied after the present:
 *   LAUNCH(mode id)  MOD_SET(id, "0"/"1")  REPLAY_PLAY(n)  SKIN_SET(slot, name | "none")  NET_HOST()
 *   NET_JOIN(code or HOST:PORT)  NET_SPECTATE(code or HOST:PORT)  REFRESH(list kind digit; other = all) */
enum { GD_UI_LIST_MODES = 0, GD_UI_LIST_MODS, GD_UI_LIST_REPLAYS, GD_UI_LIST_SKINS, GD_UI_LIST_STATUS, GD_UI_LIST_ONLINE };
enum { GD_UI_REQ_LAUNCH = 1, GD_UI_REQ_MOD_SET, GD_UI_REQ_REPLAY_PLAY, GD_UI_REQ_SKIN_SET, GD_UI_REQ_NET_HOST,
       GD_UI_REQ_NET_JOIN, GD_UI_REQ_NET_SPECTATE, GD_UI_REQ_REFRESH };
int      gd_ui_list_info(unsigned kind);                          /* gen << 16 | count; GD_ECAP without UI_HOST */
int      gd_ui_list_get(unsigned kind, unsigned i, void* out, unsigned cap); /* bytes copied (truncated to cap) */
int      gd_ui_request(int op, const char* arg, unsigned len);    /* GD_OK queued, GD_EFULL, GD_EINVAL */
/* ---- gd-ui-v1.3 (UI_HOST): the SSBM Nucleus mod browser (wasm-ui-lab-api.md A6.1; host rust/melee/src/nucleus) ----
 *   NC        one record per search result (<= 60, then one "... N more" record with id 0): id, title, author, kind
 *             (fighter name for a costume), queue state ("" / queued / running / done / failed), percent, picture key
 *             (decimal u32, 0 none), installable (0/1), why not, downloads, description (cut to fit 256 bytes)
 *   NC_QUEUE  one record per queue entry: id, state, percent, title, reason (failed)
 *   NC_STATE  one record: sync line, catalog size, processing (0/1), pending, last install message, restart needed
 *             (0/1), online (0/1: nothing starts), last message is an error (0/1), site
 * Requests: NC_SEARCH(text, tab digit or "" = all, sort digit; opens the engine: threads + a polite sync)
 *   NC_QUEUE(post id: queue / unqueue)  NC_PROCESS("1" start / "0" pause; saved: a restart resumes)  NC_SYNC() */
enum { GD_UI_LIST_NC = 6, GD_UI_LIST_NC_QUEUE, GD_UI_LIST_NC_STATE };
enum { GD_UI_REQ_NC_SEARCH = 9, GD_UI_REQ_NC_QUEUE, GD_UI_REQ_NC_PROCESS, GD_UI_REQ_NC_SYNC };
/* texture handle (for GD_UI_OP_TEX) of host picture `key`, 96 x 72 RGBA fitted, 0 while not on hand (ask every frame
 * the picture is on screen: the ask renews it, an unknown key is fetched off the game thread; <= 2 uploads per present,
 * 16 slots recycled least recently used, so a handle shows whatever key was last asked through it) */
unsigned gd_ui_host_pic(unsigned key);
/* exports: int gd_ui_init(void); void gd_ui_frame(unsigned present_no) (required with UI_DRAW);
 * void gd_ui_event(unsigned node, int ev, int value); void gd_ui_scene_enter(unsigned); void gd_ui_scene_leave(unsigned) */

/* ---------------------------------------------------------------- gd-lab-v1 (module "gd_lab")
 * Sessions: allowed in OFFLINE and LAB_NETPLAY, GD_EOFFLINE elsewhere. Every call marked (cmd) changes the sim: it is
 * queued as a 16-byte lab command (GAP-13) and returns the frame it applies on (>= 0), or an error. Offline it applies
 * on the next lab frame; in LAB_NETPLAY it rides the lockstep stream with this peer's pad of that frame and both peers
 * apply it on that frame (host's commands first). At most 8 commands per peer per frame; more wait for later frames.
 * A lab match (offline or LAB_NETPLAY) is untimed unless the command line sets a time (--time-secs), and START belongs
 * to the Lab: the sim never sees it unless L+R+A are held (the retail pause, so L+R+A+START still quits).
 *
 * Command record (little-endian): u8 op, u8 port, u16 a, u32 b, u32 c, u32 d. Version GD_LAB_CMD_VERSION (part of
 * the netplay handshake's mod digest). Ops: 1 SAVE(a slot) 2 LOAD(a slot) 3 RING(a frames) 4 REWIND(b n) 5 PAUSE(a on)
 * 6 STEP(b n) 7 GO_LIVE 8 REC_START(a mask, b rec) 9 REC_STOP(b rec) 10 PLAY(a mask, b rec, c flags) 11 PLAY_STOP(a mask)
 * 12 REC_FREE(b rec) 13 CPU(port, a behaviour, b param) 14 REACTION(port, a kind, b option, c param) 15 PERCENT(port,
 * b f32) 16 POS(port, b x, c y) 17 PLAY_SET(port, a order | flags << 8, b/c 8 rec ids, d 8 x 4-bit weights)
 * 18 RNG_SEED(b seed) 19 REACTION_WEIGHTS(port, a kind, b weights). v1.1 (Geno, offline only, so never on the lockstep
 * stream and the version stays 1): 24 GENO_FLY(port, a mode) 25 GENO_TELEPORT(port, b x, c y) 26 GENO_FLY_ATTACK(port,
 * a flags | damage << 2 | angle << 7, b radius f32, c active | gap << 16, d kb | kbg << 16) 27 GENO_FLY_CLEAR(port)
 * 28 GENO_KI(port, b ki) 29 GENO_RELOAD (20..23 are the drills' ops). */
#define GD_LAB_CMD_VERSION 1
#define GD_LAB_SLOTS       16
#define GD_LAB_RING_MAX    600

enum { GD_LAB_PLAY_LOOP = 1, GD_LAB_PLAY_MIRROR = 2, GD_LAB_PLAY_FROM_SLOT = 4 };
enum { GD_LAB_ORDER_ORDERED = 0, GD_LAB_ORDER_RANDOM = 1, GD_LAB_ORDER_WEIGHTED = 2 };
enum { GD_LAB_CPU_NONE = 0, GD_LAB_CPU_STAND, GD_LAB_CPU_CROUCH, GD_LAB_CPU_JUMP, GD_LAB_CPU_SHIELD,
       GD_LAB_CPU_LEVEL /* param 1..9 */, GD_LAB_CPU_RECORDING /* param rec */ };
/* reaction kinds and options (RANDOM draws from the lab RNG, by the kind's weight table when one is set) */
enum { GD_LAB_RK_DI = 0, GD_LAB_RK_SDI, GD_LAB_RK_TECH, GD_LAB_RK_GETUP, GD_LAB_RK_LEDGE, GD_LAB_RK_SHIELD, GD_LAB_RK_OOS };
enum { GD_LAB_RO_NONE = 0, GD_LAB_RO_IN = 1, GD_LAB_RO_OUT = 2, GD_LAB_RO_AWAY = 3, GD_LAB_RO_TOWARD = 4,
       GD_LAB_RO_RANDOM = 5, GD_LAB_RO_UP = 6, GD_LAB_RO_DOWN = 7, GD_LAB_RO_RECORDING = 15 };
enum { GD_LAB_RO_TECH_IN_PLACE = 1, GD_LAB_RO_TECH_ROLL_IN = 2, GD_LAB_RO_TECH_ROLL_AWAY = 3, GD_LAB_RO_TECH_MISS = 4 };
enum { GD_LAB_RO_GETUP_STAND = 1, GD_LAB_RO_GETUP_ROLL_IN = 2, GD_LAB_RO_GETUP_ROLL_AWAY = 3, GD_LAB_RO_GETUP_ATTACK = 4,
       GD_LAB_RO_LEDGE_JUMP = 6, GD_LAB_RO_LEDGE_DROP = 7 };
enum { GD_LAB_RO_BLOCK_ALL = 1, GD_LAB_RO_BLOCK_AFTER_FIRST = 2, GD_LAB_RO_BLOCK_RANDOM = 3 };
enum { GD_LAB_RO_OOS_JUMP = 1, GD_LAB_RO_OOS_NAIR, GD_LAB_RO_OOS_GRAB, GD_LAB_RO_OOS_UPB, GD_LAB_RO_OOS_SPOTDODGE, GD_LAB_RO_OOS_ROLL,
       GD_LAB_RO_OOS_RANDOM = 8 /* RANDOM (5) is spot-dodge for this kind */ };
/* CPU_NONE with reactions: the reactions act on top of the port's own pad; any other behaviour replaces it */
/* GAP-5 reaction parameter: delay frames | DI/SDI strength 0..100 << 8 | random chance % << 16 | recording << 24 */
#define GD_LAB_REACT_PARAM(delay, strength, chance, rec) ((unsigned)(delay) | (unsigned)(strength) << 8 | (unsigned)(chance) << 16 | (unsigned)(rec) << 24)

/* B4 overlay bits (presentation only, local, never hashed) and style kinds */
enum { GD_LAB_OV_HITBOX = 1u << 0, GD_LAB_OV_HURTBOX = 1u << 1, GD_LAB_OV_ECB = 1u << 2, GD_LAB_OV_LEDGE_GRAB = 1u << 3,
       GD_LAB_OV_COLLISION = 1u << 4, GD_LAB_OV_BLASTZONE = 1u << 5, GD_LAB_OV_CAMERA = 1u << 6, GD_LAB_OV_LABELS = 1u << 7,
       GD_LAB_OV_SWEPT = 1u << 8, GD_LAB_OV_SHIELD = 1u << 9, GD_LAB_OV_GRAB = 1u << 10, GD_LAB_OV_IASA = 1u << 11,
       GD_LAB_OV_INVINC = 1u << 12, GD_LAB_OV_DI_LINE = 1u << 13, GD_LAB_OV_KB_ARC = 1u << 14 };
enum { GD_LAB_STYLE_HITBOX = 0, GD_LAB_STYLE_HURTBOX, GD_LAB_STYLE_ECB, GD_LAB_STYLE_LEDGE, GD_LAB_STYLE_COLLISION,
       GD_LAB_STYLE_DI, GD_LAB_STYLE_LABEL, GD_LAB_STYLE_INVINC };

/* B5 events (16 bytes): u8 kind, u8 port, u16 0, i32 frame, i32 a, i32 b. GAP-12 kinds 7..12. */
enum { GD_LAB_EV_LCANCEL = 1 /* a ok, b frames from press */, GD_LAB_EV_TECH = 2 /* a 0 miss 1 in place 2 fwd 3 back */,
       GD_LAB_EV_OOS = 3 /* a action, b frames in shield */, GD_LAB_EV_IASA = 4 /* a action, b frame */,
       GD_LAB_EV_HIT = 5 /* a attacker, b damage x100 */, GD_LAB_EV_SHIELD = 6 /* a attacker, b shield HP x100 */,
       GD_LAB_EV_POWERSHIELD = 7 /* a frames into shield */, GD_LAB_EV_WAVEDASH = 8 /* a angle, b frames after jumpsquat */,
       GD_LAB_EV_LEDGEDASH = 9 /* a GALINT */, GD_LAB_EV_SDI = 10 /* a dx x100, b dy x100 */, GD_LAB_EV_ASDI = 11,
       GD_LAB_EV_AIRDODGE = 12 /* a angle, b frames after jumpsquat or -1 */ };
/* frame meter cells (GAP-8) */
enum { GD_LAB_CELL_IDLE = 0, GD_LAB_CELL_STARTUP, GD_LAB_CELL_ACTIVE, GD_LAB_CELL_RECOVERY, GD_LAB_CELL_STUN, GD_LAB_CELL_CAN_ACT };

typedef struct gd_lab_fighter_v1 {  /* 128 bytes, little-endian */
    unsigned version, size;         /* 1, 128 */
    int   character, action;
    float anim_frame; int anim_id;
    float hitlag, hitstun;
    int   iasa, intangible, invincible;
    float shield_hp; int jumps_used, airborne;
    float pos_x, pos_y, vel_x, vel_y, percent, facing;
    float ecb[8];                   /* top xy, bottom xy, right xy, left xy (relative to pos) */
    int   grabbing, cell, action_frame, match_frame;
} gd_lab_fighter_v1;
typedef struct { float x, y, prev_x, prev_y, radius, damage; int angle, id; } gd_lab_hitbox;   /* 32 bytes */
typedef struct { float ax, ay, bx, by, radius; int state, bone; } gd_lab_hurtbox;             /* 28 bytes */

/* B1 */
int      gd_lab_save(unsigned slot);                  /* (cmd) */
int      gd_lab_load(unsigned slot);                  /* (cmd) forks the timeline */
int      gd_lab_slot_frame(unsigned slot);            /* -1 empty */
int      gd_lab_ring(unsigned frames);                /* (cmd) 0 off, max 600; a snapshot every 8 frames */
int      gd_lab_rewind(unsigned n);                   /* (cmd) the logged future then plays back until go_live */
int      gd_lab_pause(int on);                        /* (cmd) */
int      gd_lab_step(int n);                          /* (cmd) n < 0 = rewind(-n) */
int      gd_lab_go_live(void);                        /* (cmd) take over */
/* B2 */
unsigned gd_lab_rec_start(unsigned port_mask);        /* (cmd) the handle; 0 = refused */
int      gd_lab_rec_stop(unsigned rec);               /* (cmd) */
int      gd_lab_play(unsigned rec, unsigned port_mask, unsigned flags);  /* (cmd) one recorded port plays on any */
int      gd_lab_play_stop(void);                      /* (cmd) */
int      gd_lab_play_set(unsigned port, const void* entries /* n x {u32 rec, u32 weight 0..9} */, unsigned n,
                         unsigned order_flags /* order | flags << 8 */);  /* (cmd) loops; RANDOM / WEIGHTED: lab RNG */
int      gd_lab_rec_len(unsigned rec);
int      gd_lab_rec_free(unsigned rec);               /* (cmd) */
/* B3 */
int      gd_lab_cpu(unsigned port, int behaviour, int param);                  /* (cmd) */
int      gd_lab_reaction(unsigned port, int kind, int option, unsigned param); /* (cmd) GAP-5 param */
int      gd_lab_reaction_weights(unsigned port, int kind, unsigned weights);   /* (cmd) 8 x 4 bits, options 1..8 */
int      gd_lab_set_percent(unsigned port, float pct);                         /* (cmd) forks the timeline */
int      gd_lab_set_pos(unsigned port, float x, float y);                      /* (cmd) forks the timeline */
unsigned gd_lab_rng(void);              /* GAP-7: f(synced lab seed, frame, call index); reads, never advances */
int      gd_lab_rng_seed(unsigned seed);                                       /* (cmd) */
/* B4 */
int      gd_lab_overlay(unsigned port_mask, unsigned bits);
int      gd_lab_overlay_style(unsigned kind, unsigned rgba);
/* B5 (copies, never writes the sim) */
int      gd_lab_fighter(unsigned port, void* out, unsigned len);               /* bytes written */
int      gd_lab_hitboxes(unsigned port, void* out, unsigned max);              /* count */
int      gd_lab_hurtboxes(unsigned port, void* out, unsigned max);
int      gd_lab_framedata(int character, int action, void* out, unsigned len); /* observed: u32 n, i32 first active,
                                                                                  i32 last active, i32 IASA, n cells */
float    gd_lab_attr(int character, unsigned idx);    /* ftCo_DatAttrs word idx as f32; NaN if unknown */
int      gd_lab_frame_adv(unsigned port_a, unsigned port_b);  /* after the last exchange: (0, 1) or (1, 0) */
int      gd_lab_events(void* out, unsigned max);      /* the last frame's */
int      gd_lab_events_range(int from, int to, void* out, unsigned max);  /* GAP-1: the last 600 frames */
int      gd_lab_meter(unsigned port, unsigned char* out, unsigned n);      /* GAP-8: last n cells, oldest first */
int      gd_lab_applied(void* out, unsigned max); /* the commands applied on the last lab frame, in apply order: 20 bytes
                                                     each = u8 issuer (0 offline / host, 1 guest), 3 zero, the 16-byte
                                                     command record (the other player's actions as toasts online) */
int      gd_lab_status(int* out10);  /* active, match frame, paused, replaying, ring frames, ring length, command delay,
                                        recording mask, issuer (0 host / offline, 1 guest), apply frame */

/* ---- gd-lab-v1.1: Geno (docs/geno.md; docs/rust-port/notes/geno-wasm-plan.md step 4). Queries (LAB) copy from what
 * the play flow publishes after each frame (per port) and when the lab match starts or a reload installs new data
 * (the registry's names and counts); they answer in every lab session. Strings are copied truncated to cap, with no
 * NUL: the bytes written, or -1 (GD_EINVAL) when there is no such string or the buffer is outside the memory.
 * Commands (cmd) need LAB_CPU (the reload: LAB_STATE) and are OFFLINE only, as the C LAB's (GD_EOFFLINE in LAB_NETPLAY too); a refusal by the game is read
 * back with gd_geno_result. */
#define GD_GENO_COMBO_WHATS 144 /* gd_geno_combo_info what codes: 0..15, then 16 + route (128 routes) */
enum { GD_GENO_FLY_OFF = 0, GD_GENO_FLY_ON = 1, GD_GENO_FLY_PLACE = 2 };
enum { GD_GENO_ATTACK_ON = 1, GD_GENO_ATTACK_EVERY_FRAME = 2 };
int gd_geno_profile(unsigned port);                       /* the port's Geno profile (gw_Geno_ProfileForKind), -1 none */
int gd_geno_state_count(int profile);                     /* 0 when none */
int gd_geno_state_name(int profile, int state, char* out, unsigned cap);  /* geno.json name; motion 0x400 + state */
int gd_geno_define_name(int css_kind, char* out, unsigned cap);           /* a define's display name */
int gd_geno_combo_info(unsigned port, int what);  /* GenoGame_ComboInfo: 0 links, 1 ki (x100), 2 last route, 3 routes,
                                                     4 hits, 5 blocked, 6 taken, 7 max links, 8 ki max, 9 action time,
                                                     11 profile, 16 + r route r (0 closed, 1 open, 2 input ok); -1 off */
int gd_geno_combo_route_name(int profile, int route, char* out, unsigned cap);
int gd_geno_fly_get(unsigned port);                       /* GenoFly_Get: the fly mode, -1 no fighter */
int gd_geno_fly_state(unsigned port, unsigned field);     /* GenoFly_State 0..13: attack, target, damage, radius f32
                                                             bits, x, y bits, pulses, every frame, active, gap, kb, kbg,
                                                             angle, phase */
int gd_geno_result(void* out, unsigned cap);  /* the last applied Geno command: u32 seq, i32 op, i32 port, i32 rc, then
                                                 the text (reload: the hot-reload line; fly: why it was refused); bytes
                                                 written, GD_ESTATE before the first */
int gd_geno_fly(unsigned port, int mode);                 /* (cmd) GenoFly_Set */
int gd_geno_teleport(unsigned port, float x, float y);    /* (cmd) GenoFly_Teleport; finite, |x|, |y| <= 49000 */
int gd_geno_fly_attack(unsigned port, unsigned flags, unsigned damage /* 1..30 */, float radius /* 1..30 */,
                       unsigned active /* 0..600 */, unsigned gap /* 0..3000 */, unsigned kb, unsigned kbg /* 0..2000 */,
                       unsigned angle /* 0..361 */);      /* (cmd) GenoFly_AttackConfig (0 = the game's default);
                                                             flags 0 = attack off; on puts the fighter in flight first */
int gd_geno_fly_clear(unsigned port);                     /* (cmd) GenoFly_Clear */
int gd_geno_combo_ki(unsigned port, int ki);              /* (cmd) GenoGame_ComboSetKi, ki x100, >= 0 */
int gd_geno_reload(void);                                 /* (cmd, LAB_STATE) gw_Geno_Reload: refused (data kept),
                                                             layout changed (the LAB match restarts) or re-applied */
/* v1.1 fighter controls (docs/geno.md 26): the port's fighter's layouts / extras and its saved choice (published by the
 * host), and a save of the choice for the next match (a preference: never a lab command; the running match keeps the
 * mapping frozen at its start). The choice word: bit 0 Tap jump off, bits 2-5 the layout, bits 8-15 the extras. */
int gd_controls_count(unsigned port, unsigned field /* 0 layouts, 1 extras */);
int gd_controls_name(unsigned port, unsigned field, unsigned index, char* out, unsigned cap); /* bytes, no NUL */
int gd_controls_get(unsigned port);                       /* the saved choice, -1 with no fighter on the port */
int gd_controls_set(unsigned port, unsigned choice);      /* queued for the host; GD_EINVAL for bit 1 / bits 6, 7, 16+ */

#endif
