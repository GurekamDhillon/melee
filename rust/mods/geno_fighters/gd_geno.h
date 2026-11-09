/* gd_geno.h - the geno 12 fighter-code ABI (melee_engine::geno_wasm). A define's `wasm` block names a module; each
 * callback is an exported `void f(void)`; the ctx is reached through these imports (module "gd_geno"). Values are
 * doubles, as the geno 10 Lua ctx's numbers were. A bad argument traps the call (GENO_LUA_FAULT_*, as in Lua). */
#ifndef GD_GENO_H
#define GD_GENO_H
#define GD_IMPORT(n) __attribute__((import_module("gd_geno"), import_name(#n)))
#define GD_EXPORT(n) __attribute__((export_name(#n)))

/* ctx.self.* / ctx.input.* (gd_geno_ctx keys) */
enum {
    GD_AIR = 0, GD_ANIM_ENDED, GD_FACING, GD_PERCENT, GD_X, GD_Y, GD_VEL_X, GD_VEL_Y, GD_ACTION_FRAME, GD_MOTION,
    GD_HIT_DAMAGE, GD_HIT_FROM, GD_COUNTERED, GD_KI, GD_COMBO_LINKS, GD_CONNECTED, GD_STICK_X, GD_STICK_Y, GD_STICK_FWD,
    GD_BUTTONS = 20 /* + 2 * button held, + 2 * button + 1 pressed */
};
enum { GD_BTN_ATTACK, GD_BTN_SPECIAL, GD_BTN_JUMP, GD_BTN_SHIELD, GD_BTN_GRAB, GD_BTN_BLAST };
#define GD_HELD(b) (GD_BUTTONS + 2 * (b))
#define GD_PRESSED(b) (GD_BUTTONS + 2 * (b) + 1)

GD_IMPORT(ctx) double gd_geno_ctx(int key);
GD_IMPORT(slot) int gd_geno_slot(const char *name, int len);
GD_IMPORT(get) double gd_geno_get(int slot);
GD_IMPORT(set) void gd_geno_set(int slot, double value);
GD_IMPORT(go) void gd_geno_go(const char *state, int len);
GD_IMPORT(velocity) void gd_geno_velocity(double forward, double up);
GD_IMPORT(hitbox_damage) void gd_geno_hitbox_damage(int mask, double damage);
GD_IMPORT(loop) void gd_geno_loop(void);
GD_IMPORT(turn) void gd_geno_turn(void);
GD_IMPORT(ki) void gd_geno_ki(double delta);
GD_IMPORT(ledge) void gd_geno_ledge(int mode);

static inline int gd_strlen(const char *s) { int n = 0; while (s[n]) ++n; return n; }
/* ctx.state.<name> (read / write) and ctx.go("<name>") with a C string */
static inline double gd_state(const char *name) { return gd_geno_get(gd_geno_slot(name, gd_strlen(name))); }
static inline void gd_state_set(const char *name, double v) { gd_geno_set(gd_geno_slot(name, gd_strlen(name)), v); }
static inline void gd_go(const char *state) { gd_geno_go(state, gd_strlen(state)); }
static inline int gd_flag(int key) { return gd_geno_ctx(key) != 0.0; }
#endif
