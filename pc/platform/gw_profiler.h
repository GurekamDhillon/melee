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
    GW_PROF_OBJECT_CALLBACK, GW_PROF_FRAME_WORK, GW_PROF_ZONES, GW_PROF_ZONE_COUNT,
    GW_PROF_MEX_INSTRUCTIONS = 64, GW_PROF_ROLLBACK_FRAMES,
    GW_PROF_SNAPSHOT_BYTES, GW_PROF_HEAP_FREE, GW_PROF_HEAP_USED,
    GW_PROF_DRAW_CALLS, GW_PROF_VERTICES, GW_PROF_PARTICLE_COUNT,
    GW_PROF_HEAP0_FREE, GW_PROF_HEAP3_FREE, GW_PROF_HEAP4_FREE, GW_PROF_HEAP5_FREE,
    GW_PROF_PIPELINE_SKIPS, GW_PROF_PIPELINE_WAITS, GW_PROF_COUNTER_END
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
int gw_prof_enabled(void);
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
#define GW_PROF_ZONE_BEGIN(id, detail) gw_prof_begin((id), (detail))
#define GW_PROF_ZONE_END() gw_prof_end()
#define GW_PROF_JOIN_(a,b) a##b
#define GW_PROF_JOIN(a,b) GW_PROF_JOIN_(a,b)
#ifdef __clang__
/* Clang cleanup handles every C return path; the game uses scalar pairs instead. */
static inline void gw_prof_scope_cleanup(unsigned *mark) { if (*mark != ~0u) gw_prof_unwind(*mark); }
#define GW_PROF_ZONE(id) \
    unsigned GW_PROF_JOIN(gw_prof_scope_,__LINE__) __attribute__((cleanup(gw_prof_scope_cleanup))) = \
        gw_prof_enabled() ? gw_prof_mark() : ~0u; gw_prof_begin((id), 0)
#endif
#ifdef __cplusplus
}
#endif
#endif
