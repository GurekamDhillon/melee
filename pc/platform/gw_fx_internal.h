/* gw_fx_internal.h - the Geno effects runtime's data, shared by the simulation (gw_fx.c) and the renderer
 * (gw_fx_render.cpp). Nothing outside those two files includes it. */
#ifndef GW_FX_INTERNAL_H
#define GW_FX_INTERNAL_H
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define FX_MAX_PKGS 32
#define FX_MAX_EMITTERS 32      /* per package */
#define FX_MAX_TEX 24           /* per package */
#define FX_MAX_INST 64
#define FX_MAX_PARTICLES 2000
#define FX_RING 16              /* frames of state kept for a rollback */
#define FX_KEYS 8
#define FX_SAMPLERS 3
#define FX_PATTERN_TABLE 32

enum { FX_SHAPE_POINT, FX_SHAPE_SPHERE, FX_SHAPE_SPHERE_FILL, FX_SHAPE_CIRCLE, FX_SHAPE_CIRCLE_FILL, FX_SHAPE_OTHER };
enum { FX_SH_SPRITE, FX_SH_WARP, FX_SH_DISTORTION };            /* the shader library (geno.md 20.2) */
enum { FX_COL_FLAT, FX_COL_MODULATE, FX_COL_LERP };
enum { FX_BLEND_ALPHA, FX_BLEND_ADD, FX_BLEND_SUB, FX_BLEND_MUL, FX_BLEND_SCREEN };
enum { FX_PAT_NONE, FX_PAT_FIT_LIFE, FX_PAT_CLAMP, FX_PAT_LOOP, FX_PAT_RANDOM };
enum { FX_WRAP_MIRROR, FX_WRAP_REPEAT, FX_WRAP_CLAMP };

typedef struct { int n; float k[FX_KEYS][4]; float value[3]; int keyed; } fx_curve;

typedef struct {
    int tex;                    /* index into the package's textures, -1 = none */
    int wrap[2];
    int pattern, pattern_count, table_n;
    float pattern_freq;
    int table[FX_PATTERN_TABLE];
    int div[2];
    float scroll[2], scroll_add[2], scale[2], scale_add[2], rotate, rotate_add;
    int en_scroll, en_scale, en_rotate;
} fx_sampler;

typedef struct {
    char name[48];
    int mesh, follow;           /* follow: 0 srt, 1 none, 2 translate */
    float trans[3], rot[3];
    int start, duration, one_time, interval;
    float rate, rate_random;
    int by_dist, dist_max_particles;
    float dist_unit, dist_min, dist_max;
    int shape;
    float radius[3], caliber;
    int life, infinite;
    float life_random;          /* 0..1 */
    float vel_all, vel_dir[3], vel_dir_scale, vel_random, inherit;
    float grav_dir[3], grav, air;
    int grav_world;
    float scale[2], scale_random;
    fx_curve scale_keys, color0, alpha0, color1, alpha1, param;
    float color_scale;
    float rot_init[3], rot_init_random[3], rot_add[3], rot_add_random[3];
    /* material (the shader library type + parameters) */
    int shader, color_mode, offset, offset_mask, color_mask, alpha_mask, blend, depth_test, alpha_test;
    float strength[2], alpha_threshold, bloom_threshold, bloom_intensity;
    fx_sampler smp[FX_SAMPLERS];
} fx_emitter;

typedef struct {
    char name[64];
    char path[260];             /* the .gfx.json */
    char dir[260];              /* its directory (textures are relative to it) */
    int nem;
    fx_emitter em[FX_MAX_EMITTERS];
    int ntex, nmesh;
    char tex_file[FX_MAX_TEX][96];
    char tex_swizzle[FX_MAX_TEX][5];
    float forward[3], up[3];    /* the effect's own axes ("space"): which local axis is the owner's travel direction
                                   and which is up (Ultimate articles: +Z forward, +Y up) */
} fx_pkg;

typedef struct {
    int used, pkg, em, detached, attach_frame, age, emitted, facing;
    uint32_t owner, mtx_off, rng;
    float accum, dist_accum;
    float pos[3], prev[3], m[3][4];
    float basis[3][3];          /* effect-local -> owner-local (the package's space and the facing) */
    int have_prev;
} fx_inst;

typedef struct {
    int inst;                   /* -1 = free */
    int age, life;
    float pos[3], vel[3], scale, rot, rot_add;
    float lpos[3], lvel[3];     /* follow srt: in the emitter frame; translate: world axes, from the owner */
    uint32_t seed;
} fx_part;

typedef struct {
    int frame;                  /* the game frame this state is at the end of */
    fx_inst inst[FX_MAX_INST];
    fx_part part[FX_MAX_PARTICLES];
    int nlive;
    uint32_t spawned, killed, refused;
} fx_state;

/* gw_fx.c */
const fx_state *gw_fx_state(void);
const fx_pkg *gw_fx_pkg(int i);
int gw_fx_npkg(void);
float gw_fx_curve_at(const fx_curve *c, float t, int ch);

#ifdef __cplusplus
}
#endif
#endif
