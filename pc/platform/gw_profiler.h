/* Native diagnostics only. IDs cross the PPC boundary; names and pointers never do.
 * detail is caller-owned identity: fighter = (slot << 16) | character, shader/callsite = hash/id.
 * Inclusive zone time overlaps nested zones and other threads; never sum it as frame time. */
#ifndef GW_PROFILER_H
#define GW_PROFILER_H
#ifdef __cplusplus
extern "C" {
#endif
enum GwProfId {
    GW_PROF_FRAME = 0, GW_PROF_INPUT, GW_PROF_LOGIC, GW_PROF_DRAW,
    GW_PROF_RENDER, GW_PROF_PRESENT, GW_PROF_GPU, GW_PROF_FIGHTER,
    GW_PROF_FIGHTER_INPUT, GW_PROF_FIGHTER_ACTION, GW_PROF_FIGHTER_PHYSICS,
    GW_PROF_FIGHTER_COLLISION, GW_PROF_HITBOX, GW_PROF_ANIMATION, GW_PROF_SKINNING,
    GW_PROF_ITEMS, GW_PROF_ENEMIES, GW_PROF_STAGE, GW_PROF_PARTICLES,
    GW_PROF_GENO_EFFECTS, GW_PROF_CAMERA, GW_PROF_HUD, GW_PROF_MEX,
    GW_PROF_GENO_HOOK, GW_PROF_LUA_SCRIPT, GW_PROF_LUA_CALLBACK, GW_PROF_LUA_API,
    GW_PROF_ROLLBACK, GW_PROF_SNAPSHOT_SAVE, GW_PROF_SNAPSHOT_RESTORE,
    GW_PROF_REWIND, GW_PROF_FILE_READ, GW_PROF_DECODE, GW_PROF_TEXTURE_UPLOAD,
    GW_PROF_MESH_UPLOAD, GW_PROF_SHADER_COMPILE, GW_PROF_PIPELINE_COMPILE,
    GW_PROF_CHUNK_LOAD, GW_PROF_CHUNK_UNLOAD, GW_PROF_MODEL_RELOAD, GW_PROF_BENCH_CALL,
    GW_PROF_GPU_PASS, GW_PROF_SUBMIT, GW_PROF_PACING, GW_PROF_AUDIO, GW_PROF_AUDIO_MIX,
    GW_PROF_OBJECT_CALLBACK, GW_PROF_FRAME_WORK, GW_PROF_ZONES,
    /* perf-2: GX display-list recording (per-frame sample, full profiler only), CPU skinning matrix
     * setup, effect pipeline creation, and the game thread blocked on the render worker. */
    GW_PROF_GX_RECORD, GW_PROF_ENVELOPE, GW_PROF_EFFECT_PIPELINE, GW_PROF_QUEUE_WAIT,
    GW_PROF_ZONE_COUNT,
    GW_PROF_MEX_INSTRUCTIONS = 64, GW_PROF_ROLLBACK_FRAMES,
    GW_PROF_SNAPSHOT_BYTES, GW_PROF_HEAP_FREE, GW_PROF_HEAP_USED,
    GW_PROF_DRAW_CALLS, GW_PROF_VERTICES, GW_PROF_PARTICLE_COUNT,
    GW_PROF_HEAP0_FREE, GW_PROF_HEAP3_FREE, GW_PROF_HEAP4_FREE, GW_PROF_HEAP5_FREE,
    GW_PROF_PIPELINE_SKIPS, GW_PROF_PIPELINE_WAITS,
    /* per-frame event counts (game thread): see gw_prof_tally */
    GW_PROF_ENV_SETUPS, GW_PROF_ENV_REUSED, GW_PROF_POBJ_DRAWS, GW_PROF_GX_BEGINS, GW_PROF_GX_DLISTS,
    GW_PROF_COUNTER_END
};
typedef struct GwProfStats {
    unsigned long long count;
    double mean, p50, p95, p99, max;
} GwProfStats;
typedef struct GwProfDetailStats {
    unsigned id, detail;
    GwProfStats stats;
    char name[128];
} GwProfDetailStats;
void gw_prof_init(void);
void gw_prof_shutdown(void);
int gw_prof_enabled(void);   /* the FULL profiler (MELEE_PROFILER=1): events, traces, details */
int gw_prof_active(void);    /* full profiler or the always-on summary */
double gw_prof_clock_ms(void);
void gw_prof_set_enabled(int enabled);
void gw_prof_reset(void);
void gw_prof_begin(unsigned id, unsigned detail);
void gw_prof_end(void);
void gw_prof_counter(unsigned id, double value);
void gw_prof_counter_detail(unsigned id, unsigned detail, double value);
/* For externally measured GPU durations; never label CPU submission as GPU time. */
void gw_prof_sample(unsigned id, double milliseconds, unsigned detail);
void gw_prof_cpu_completed(unsigned id, unsigned detail, double milliseconds);
void gw_prof_try_cpu_completed(unsigned id, unsigned detail, double milliseconds);
void gw_prof_gpu_completed(unsigned id, unsigned detail, double milliseconds, unsigned long long render_frame);
/* Diagnostics context: 0 last Lua API, 1 last spawn, 2 last area operation.
 * Copied into retained events; last observed event is correlation, not causation. */
void gw_prof_context(unsigned kind, const char *name);
unsigned gw_prof_log_frame(void);
void gw_prof_detail_name(unsigned detail, const char *name);
unsigned gw_prof_mark(void);
void gw_prof_unwind(unsigned mark);
void gw_prof_frame_begin(void);
void gw_prof_frame_end(void);
const char *gw_prof_name(unsigned id);
int gw_prof_stats(unsigned id, GwProfStats *out);
int gw_prof_details(GwProfDetailStats *out, int capacity);
unsigned long long gw_prof_frames(void);
int gw_prof_report(const char *path);
int gw_prof_trace(const char *path, unsigned frames);
void gw_prof_set_hitch(double milliseconds);
/* Handles on/off/reset/report/trace N/hitch MS. Returns 1 for valid command. */
int gw_prof_command(const char *args, char *out, unsigned cap);
void gw_ProfBegin(int id, int detail);
void gw_ProfEnd(void);
void gw_ProfCounter(int id, int value);
void gw_ProfCounterDetail(int id, int detail, int value);
int gw_ProfEnabled(void);
/* ---- always-on summary (perf record) --------------------------------------------------------
 * MELEE_PERF_SUMMARY=0 turns it off. It keeps per-scene frame-work and bucket accumulators and a
 * small percentile reservoir; no event ring, no hitch dumps. perf.json is written at scene end and
 * exit (path: MELEE_PERF_PATH, default perf.json in the working directory). */
void gw_prof_summary_set(int on);
int gw_prof_summary_enabled(void);
/* Increment a per-frame event counter (GW_PROF_ENV_SETUPS ...). Game thread only. */
void gw_prof_tally(unsigned id);
void gw_prof_scene_begin(const char *name);
void gw_prof_scene_end(void);
/* Called once per presented frame by the frame tick: aurora's draw call count for that frame. */
void gw_prof_frame_stats(unsigned draw_calls, unsigned gx_begins, unsigned gx_dlists, int presented,
                         unsigned end_frames, double replay_ms);
/* GX call time of the frame just ended, in ms (full profiler / perf overlay only). */
void gw_prof_gx_ms(double ms);
/* Summary-mode script attribution: push a script zone for mod `id` (the pointer must stay valid). */
void gw_prof_script_begin(const char *id);
/* Callbacks the platform registers: a JSON fragment of run conditions, and a log line sink. */
void gw_prof_set_conditions_cb(int (*fn)(char *out, unsigned cap));
void gw_prof_set_log_cb(void (*fn)(const char *line));
int gw_prof_perf_write(const char *path);
/* "perf" console/Lua command helper: status line. Bench window: MELEE_PERF_WINDOW=<warm>,<frames>. */
int gw_prof_perf_status(char *out, unsigned cap);
int gw_prof_scene_draws(double *mean, unsigned *max, unsigned long long *frames, unsigned long long *with_draws);
void gw_ProfCount(int id);
int gw_ProfActive(void);
#define GW_PROF_ZONE_BEGIN(id, detail) gw_prof_begin((id), (detail))
#define GW_PROF_ZONE_END() gw_prof_end()
#define GW_PROF_JOIN_(a,b) a##b
#define GW_PROF_JOIN(a,b) GW_PROF_JOIN_(a,b)
#ifdef __clang__
/* Clang cleanup handles every C return path; the game uses scalar pairs instead. */
static inline void gw_prof_scope_cleanup(unsigned *mark) { if (*mark != ~0u) gw_prof_unwind(*mark); }
#define GW_PROF_ZONE(id) \
    unsigned GW_PROF_JOIN(gw_prof_scope_,__LINE__) __attribute__((cleanup(gw_prof_scope_cleanup))) = \
        gw_prof_active() ? gw_prof_mark() : ~0u; gw_prof_begin((id), 0)
#endif
#ifdef __cplusplus
}
#endif
#endif
