/* Bounded, host-only profiler. No game memory, RNG, pads, snapshot or simulation state.
 * Disabled calls perform one atomic load/branch, without clock, lock or allocation.
 * Enabled begin: clock + TLS stack; end: clock + short aggregation lock. Storage is fixed.
 * Percentiles use a deterministic Algorithm R reservoir across the run (4096 per ID,
 * 256 per detail); exact while observations fit, estimated thereafter. Mean/count/max exact.
 * Traces retain 131072 admitted events (120-frame filter); tiny repeats are sampled.
 * Report/trace allocation and disk IO occur only on explicit commands, shutdown or a hitch.
 * This is inclusive timing: overlapping CPU threads and nested zones must not be summed.
 */
#include "gw_profiler.h"
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>
#include <stdbool.h>
#if defined(GW_PROF_TRACY) && defined(GW_RELEASE_BUILD)
#error Tracy is development-only and must never be compiled into release builds
#endif
#ifdef GW_PROF_TRACY
#include "../third_party/tracy/public/tracy/TracyC.h"
static struct ___tracy_source_location_data prof_tracy_locations[GW_PROF_ZONE_COUNT];
#endif
#ifdef _WIN32
#define PSAPI_VERSION 2
#include <windows.h>
#include <tlhelp32.h>
#include <psapi.h>
#include <intrin.h>
/* Aurora owns timestamp availability; standalone tests supply this output hook. */
extern void aurora_profiler_enable(bool enabled);
#ifndef GW_TEST_STANDALONE
extern uint64_t aurora_profiler_gpu_dropped_zones(void);
extern uint64_t aurora_profiler_gpu_dropped_frames(void);
#endif
static SRWLOCK prof_mutex = SRWLOCK_INIT;
static volatile LONG prof_on, prof_generation = 1, prof_frame, prof_dropped;
static volatile LONG prof_sum;
#define PROF_TLS __declspec(thread)
static void prof_lock(void) { AcquireSRWLockExclusive(&prof_mutex); }
static void prof_unlock(void) { ReleaseSRWLockExclusive(&prof_mutex); }
static int prof_try_lock(void) { return TryAcquireSRWLockExclusive(&prof_mutex) != 0; }
/* Aligned x86 load is atomic; compiler acquire barrier needs no locked RMW/fence. */
static unsigned prof_load(volatile LONG *p) { unsigned v = (unsigned)*p; _ReadWriteBarrier(); return v; }
static unsigned prof_thread(void) { return GetCurrentThreadId(); }
static double prof_frequency;
static double prof_now(void) {
    LARGE_INTEGER t;
    QueryPerformanceCounter(&t);
    return (double)t.QuadPart * 1000.0 / prof_frequency;
}
#else
/* Standalone accounting tests can use a POSIX host; the port still builds on Windows. */
#include <pthread.h>
#include <stdatomic.h>
#include <time.h>
static pthread_mutex_t prof_mutex = PTHREAD_MUTEX_INITIALIZER;
static _Atomic unsigned prof_on, prof_generation = 1, prof_frame, prof_dropped;
static _Atomic unsigned prof_sum;
#define PROF_TLS _Thread_local
static void prof_lock(void) { pthread_mutex_lock(&prof_mutex); }
static void prof_unlock(void) { pthread_mutex_unlock(&prof_mutex); }
static int prof_try_lock(void) { return pthread_mutex_trylock(&prof_mutex) == 0; }
static unsigned prof_load(_Atomic unsigned *p) { return atomic_load(p); }
static unsigned prof_thread(void) { return (unsigned)(uintptr_t)pthread_self(); }
static double prof_now(void) {
    struct timespec t;
    clock_gettime(CLOCK_MONOTONIC, &t);
    return t.tv_sec * 1000.0 + t.tv_nsec / 1000000.0;
}
#endif
#define PROF_DEPTH 64
#define PROF_SAMPLES 4096
#define PROF_EVENTS 131072
#define PROF_DETAILS 512
#define PROF_DETAIL_SAMPLES 256
#define PROF_HISTORY 120
typedef struct ProfSeries {
    uint64_t count;
    double sum, max, sample[PROF_SAMPLES];
} ProfSeries;
typedef struct ProfEvent {
    double start, duration, value;
    unsigned id, detail, thread, frame;
    char phase;
    unsigned long long render_frame;
    char context[3][64];
} ProfEvent;
typedef struct ProfOpen {
    double start; unsigned id, detail, frame;
#ifdef GW_PROF_TRACY
    TracyCZoneCtx tracy;
#endif
} ProfOpen;
typedef struct ProfDetail {
    unsigned id, detail;
    uint64_t count;
    double sum, max;
    double sample[PROF_DETAIL_SAMPLES];
} ProfDetail;
typedef struct ProfLabel { unsigned detail; char name[128]; } ProfLabel;
static PROF_TLS struct {
    ProfOpen stack[PROF_DEPTH];
    unsigned depth, overflow, generation;
    double frame_start, pacing_ms;
    int frame_active;
} prof_tls;
static struct {
    ProfSeries series[GW_PROF_COUNTER_END];
    ProfDetail details[PROF_DETAILS];
    unsigned detail_count;
    ProfLabel labels[PROF_DETAILS];
    unsigned label_count;
    ProfEvent events[PROF_EVENTS];
    uint64_t event_count, completed, stack_overflow, unmatched_end, invalid_id;
    uint64_t detail_overflow, late_events, detail_aggregated, event_sampled, label_omitted;
    unsigned admission_frame[GW_PROF_COUNTER_END], admission_count[GW_PROF_COUNTER_END];
    char context[3][64];
    unsigned hitch_pending, hitch_last;
    double hitch_ms, epoch;
    int initialized, gpu_available;
    char report_path[1024], trace_path[1024];
} prof;

static void sum_set(unsigned value);
static void sum_cfg(void);
static void prof_set_on(unsigned value) {
#ifdef _WIN32
    InterlockedExchange(&prof_on, (LONG)value);
#else
    atomic_store(&prof_on, value);
#endif
}
static unsigned prof_next(volatile void *ptr) {
#ifdef _WIN32
    return (unsigned)InterlockedIncrement((volatile LONG *)ptr);
#else
    return atomic_fetch_add((_Atomic unsigned *)ptr, 1) + 1;
#endif
}
static const char *prof_names[GW_PROF_COUNTER_END] = {
    "frame", "input", "logic", "draw_recording", "render_worker", "present", "gpu",
    "fighter", "fighter_input", "fighter_action", "fighter_physics", "fighter_collision",
    "hitbox", "animation", "skinning", "items", "enemies", "stage", "particles",
    "geno_effects", "camera", "hud", "mex", "geno_hook", "lua_script", "lua_callback",
    "lua_api", "rollback", "snapshot_save", "snapshot_restore", "rewind", "file_read",
    "decode", "texture_upload", "mesh_upload", "shader_compile", "pipeline_compile",
    "chunk_load", "chunk_unload", "model_reload", "bench_call", "gpu_pass", "submit", "pacing",
    "audio_stream", "audio_mix", "object_callback", "frame_work", "zones",
    "gx_record", "envelope_setup", "effect_pipeline", "queue_wait",
    [GW_PROF_MEX_INSTRUCTIONS] = "mex_instructions", [GW_PROF_ROLLBACK_FRAMES] = "rollback_frames",
    [GW_PROF_SNAPSHOT_BYTES] = "snapshot_bytes", [GW_PROF_HEAP_FREE] = "heap_free",
    [GW_PROF_HEAP_USED] = "heap_used", [GW_PROF_DRAW_CALLS] = "draw_calls",
    [GW_PROF_VERTICES] = "vertices", [GW_PROF_PARTICLE_COUNT] = "particle_count",
    [GW_PROF_HEAP0_FREE] = "heap0_free", [GW_PROF_HEAP3_FREE] = "heap3_free",
    [GW_PROF_HEAP4_FREE] = "heap4_free", [GW_PROF_HEAP5_FREE] = "heap5_free",
    [GW_PROF_PIPELINE_SKIPS] = "pipeline_skips", [GW_PROF_PIPELINE_WAITS] = "pipeline_waits",
    [GW_PROF_ENV_SETUPS] = "env_setups", [GW_PROF_ENV_REUSED] = "env_reused", [GW_PROF_POBJ_DRAWS] = "pobj_draws",
    [GW_PROF_GX_BEGINS] = "gx_begins", [GW_PROF_GX_DLISTS] = "gx_dlists"
};
const char *gw_prof_name(unsigned id) {
    return id < GW_PROF_COUNTER_END ? prof_names[id] : NULL;
}
int gw_prof_enabled(void) { return prof_load(&prof_on) != 0; }
double gw_prof_clock_ms(void) { return gw_prof_enabled() ? prof_now() : 0; }
static void prof_tls_sync(void) {
    unsigned g = prof_load(&prof_generation);
    if (prof_tls.generation != g) {
#ifdef GW_PROF_TRACY
        while (prof_tls.depth) {
            ProfOpen *o = &prof_tls.stack[--prof_tls.depth];
            if (o->id < GW_PROF_ZONE_COUNT) ___tracy_emit_zone_end(o->tracy);
        }
#endif
        memset(&prof_tls, 0, sizeof prof_tls);
        prof_tls.generation = g;
    }
}
void gw_prof_init(void) {
    const char *v;
    if (prof.initialized) return; /* init is owned by the game thread before workers start */
#ifdef _WIN32
    { LARGE_INTEGER f; QueryPerformanceFrequency(&f); prof_frequency = (double)f.QuadPart; }
#endif
    prof.initialized = 1;
#ifdef GW_PROF_TRACY
    { unsigned id; for (id = 0; id < GW_PROF_ZONE_COUNT; ++id)
        prof_tracy_locations[id] = (struct ___tracy_source_location_data){prof_names[id], "gw_prof_begin", __FILE__, 0, 0}; }
#endif
    prof.epoch = prof_now();
    prof.hitch_ms = 1000.0 / 120.0 * 2.0;
    v = getenv("MELEE_PROF_HITCH_MS");
    if (v && atof(v) > 0) prof.hitch_ms = atof(v);
    v = getenv("MELEE_PROF_REPORT");
    snprintf(prof.report_path, sizeof prof.report_path, "%s", v && *v ? v : "profiler-report.json");
    v = getenv("MELEE_PROF_TRACE");
    snprintf(prof.trace_path, sizeof prof.trace_path, "%s", v && *v ? v : "profiler-trace.json");
    v = getenv("MELEE_PROFILER");
    prof_set_on(v && *v && strcmp(v, "0") && strcmp(v, "off"));
    /* The always-on summary: default ON; MELEE_PERF_SUMMARY=0 (or off) turns it off entirely. */
    v = getenv("MELEE_PERF_SUMMARY");
    sum_set(!(v && *v && (!strcmp(v, "0") || !strcmp(v, "off"))));
    sum_cfg();
}
void gw_prof_set_enabled(int enabled) {
    gw_prof_init();
    /* Generation change invalidates every thread's open scopes across off/on transitions. */
    prof_set_on(0);
    prof_lock();
    prof_next(&prof_generation);
    prof_unlock();
    prof_set_on(enabled != 0);
#ifdef _WIN32
    aurora_profiler_enable(enabled != 0 || prof_load(&prof_sum) != 0);
#endif
}
void gw_prof_reset(void) {
    int was_on = gw_prof_enabled();
    gw_prof_init();
    prof_set_on(0);
    prof_lock();
    memset(prof.series, 0, sizeof prof.series);
    memset(prof.details, 0, sizeof prof.details);
    prof.event_count = prof.completed = prof.stack_overflow = prof.unmatched_end = 0;
    prof.detail_count = 0;
    prof.invalid_id = prof.detail_overflow = prof.late_events = 0;
    prof.detail_aggregated = prof.event_sampled = prof.label_omitted = 0;
    memset(prof.admission_count, 0, sizeof prof.admission_count);
    memset(prof.admission_frame, 0, sizeof prof.admission_frame);
    memset(prof.context, 0, sizeof prof.context);
    prof.hitch_pending = prof.hitch_last = 0;
    prof.gpu_available = 0;
    prof.epoch = prof_now();
#ifdef _WIN32
    InterlockedExchange(&prof_frame, 0);
    InterlockedExchange(&prof_dropped, 0);
#else
    atomic_store(&prof_frame, 0);
    atomic_store(&prof_dropped, 0);
#endif
    prof_next(&prof_generation);
    prof_unlock();
    prof_set_on(was_on);
}
static void prof_add(unsigned id, double value, unsigned detail) {
    ProfSeries *s = &prof.series[id];
    unsigned i;
    /* Algorithm R, with a private deterministic mixer (MurmurHash3 finalization).
     * Never reads/advances simulation RNG. Repeated streams remain repeatable. */
    uint64_t select = s->count;
    if (select >= PROF_SAMPLES) {
        uint64_t h = (select + 1) ^ ((uint64_t)id << 32);
        h ^= h >> 33; h *= UINT64_C(0xff51afd7ed558ccd);
        h ^= h >> 33; h *= UINT64_C(0xc4ceb9fe1a85ec53); h ^= h >> 33;
        select = h % (s->count + 1);
    }
    if (select < PROF_SAMPLES) s->sample[select] = value;
    s->count++;
    s->sum += value;
    if (s->count == 1 || value > s->max) s->max = value;
    if (!detail) return;
    for (i = 0; i < prof.detail_count; ++i)
        if (prof.details[i].id == id && prof.details[i].detail == detail) break;
    /* Reserve one overflow distribution per ID. High-cardinality identities cannot
     * starve other zones or lose observations; UINT_MAX denotes the mixed bucket. */
    if (i == prof.detail_count && prof.detail_count >= PROF_DETAILS - GW_PROF_COUNTER_END) {
        detail = UINT32_MAX;
        prof.detail_aggregated++;
        for (i = 0; i < prof.detail_count; ++i)
            if (prof.details[i].id == id && prof.details[i].detail == detail) break;
    }
    if (i == prof.detail_count) {
        if (i == PROF_DETAILS) { prof.detail_overflow++; return; }
        prof.details[i].id = id;
        prof.details[i].detail = detail;
        prof.detail_count++;
        if (id >= GW_PROF_FIGHTER && id <= GW_PROF_SKINNING &&
            (detail >> 16) >= 1 && (detail >> 16) <= 6) {
            unsigned label;
            for (label = 0; label < prof.label_count; ++label)
                if (prof.labels[label].detail == detail) break;
            if (label == prof.label_count && label < PROF_DETAILS) {
                prof.labels[label].detail = detail;
                snprintf(prof.labels[label].name, sizeof prof.labels[label].name,
                         "P%u kind%u", detail >> 16, detail & 65535u);
                prof.label_count++;
            }
        }
    }
    select = prof.details[i].count;
    if (select >= PROF_DETAIL_SAMPLES) {
        uint64_t h = (select + 1) ^ ((uint64_t)detail << 32) ^ id;
        h ^= h >> 33; h *= UINT64_C(0xff51afd7ed558ccd);
        h ^= h >> 33; h *= UINT64_C(0xc4ceb9fe1a85ec53); h ^= h >> 33;
        select = h % (prof.details[i].count + 1);
    }
    if (select < PROF_DETAIL_SAMPLES) prof.details[i].sample[select] = value;
    prof.details[i].count++;
    prof.details[i].sum += value;
    if (prof.details[i].count == 1 || value > prof.details[i].max) prof.details[i].max = value;
}
/* ======================================================================================
 * perf-2: the always-on SUMMARY (perf record)
 * ======================================================================================
 * Cheap enough to leave on for every launch (target < 0.05 ms a frame). It shares nothing with the
 * full profiler's event ring or lock: zones feed per-frame bucket accumulators (one clock pair for
 * the few bucketed zones, only a stack push for the rest), the frame tick folds them into a
 * per-scene accumulator with small percentile reservoirs, and a scene's numbers become one JSON
 * object in perf.json. Host-only: no game memory, RNG, pad or snapshot state is touched.
 * Bucket times are INCLUSIVE (a nested zone of the same bucket is counted once; different
 * buckets overlap: animation and object_callbacks sit inside logic). render_worker and
 * pipeline_compile run on other threads and are never part of frame work. */
#define SUM_NB 16
#define SUM_DEPTH 48
#define SUM_FW_CAP 4096
#define SUM_B_CAP 512
#define SUM_MODS 32
#define SUM_SCENES 24
#define SUM_JSON 7168
#define SUM_FIRST_FRAMES 600
#define SUM_PACING 8
enum { SUM_NC = 5 };
static const struct { unsigned zone; const char *name; } sum_bucket[SUM_NB] = {
    {GW_PROF_LOGIC, "logic"}, {GW_PROF_DRAW, "draw_recording"}, {GW_PROF_RENDER, "render_worker"},
    {GW_PROF_SUBMIT, "submit"}, {GW_PROF_ANIMATION, "animation"}, {GW_PROF_OBJECT_CALLBACK, "object_callbacks"},
    {GW_PROF_LUA_SCRIPT, "script"}, {GW_PROF_REWIND, "rewind"}, {GW_PROF_PACING, "pacing"},
    {GW_PROF_SKINNING, "skinning"}, {GW_PROF_QUEUE_WAIT, "queue_wait"}, {GW_PROF_PIPELINE_COMPILE, "pipeline_compile"},
    {GW_PROF_FIGHTER, "fighter"}, {GW_PROF_FILE_READ, "file_read"}, {GW_PROF_SNAPSHOT_SAVE, "snapshot_save"},
    {GW_PROF_EFFECT_PIPELINE, "effect_pipeline"}};
static const char *const sum_cnt_name[SUM_NC] = {"env_setups", "env_reused", "pobj_draws", "gx_begins", "gx_dlists"};
#ifdef _WIN32
typedef volatile LONG SumInt;
#define SUM_ADD(p, v) InterlockedExchangeAdd((p), (LONG)(v))
#define SUM_XCHG(p) InterlockedExchange((p), 0)
#else
typedef _Atomic int SumInt;
#define SUM_ADD(p, v) atomic_fetch_add((p), (int)(v))
#define SUM_XCHG(p) atomic_exchange((p), 0)
#endif
typedef struct SumOpen { double start; unsigned short zone; unsigned char slot, mod, clocked; } SumOpen;
static PROF_TLS struct { SumOpen st[SUM_DEPTH]; unsigned depth, overflow; unsigned char open[SUM_NB]; } sum_tls;
static unsigned char sum_slot[GW_PROF_ZONE_COUNT];
static SumInt sum_ns[SUM_NB];
static unsigned sum_cnt[SUM_NC];            /* game thread, folded into the frame at the tick */
static unsigned sum_frame_draws;
static double sum_gx_ms = -1;
static int sum_presented;
static unsigned sum_end_frames;     /* every aurora_end_frame so far: game frames AND interpolated replays */
static double sum_replay_ms;        /* the pacing loop's EMA of one replay's game-thread cost */
static unsigned sum_win_end0;
static double sum_win_t0, sum_win_present_fps, sum_win_replays_per_frame;
typedef struct SumAcc {
    uint64_t n;
    double fw_sum, fw_max, b_sum[SUM_NB], b_max[SUM_NB], hitch_ms, hitch_worst, dc_sum, cnt_sum[SUM_NC];
    uint64_t hitch_n, hitch33_n, dc_n;
    unsigned dc_max;
    double gx_sum; uint64_t gx_n;
    float fw[SUM_FW_CAP], bs[SUM_NB][SUM_B_CAP];
} SumAcc;
static SumAcc sum_first, sum_steady, sum_win;
typedef struct SumScene {
    int active, recorded, in_win, win_done;
    char name[96];
    double t0;
    uint64_t frames, presents;
    unsigned end0; int end0_set;
    unsigned others_start, others_end;
    double lo_first, lo_last, lo_sum, lo_max; unsigned lo_n;
    double mod_ms[SUM_MODS];
} SumScene;
static SumScene sum_scene;
static char sum_mod_id[SUM_MODS][40];
static const char *sum_mod_ptr[SUM_MODS];
static unsigned sum_mod_detail[SUM_MODS];
static unsigned sum_mod_count;
static char sum_json[SUM_SCENES][SUM_JSON];
static unsigned sum_json_count, sum_scenes_total, sum_scenes_short;
static double sum_frame_start, sum_hitch_ms = 1000.0 / 60.0, sum_last_cpu_ms;
static int sum_frame_active, sum_cfg_done;
static unsigned sum_win_warm, sum_win_frames;
static char sum_path[1024] = "perf.json";
static int (*sum_cond_cb)(char *out, unsigned cap);
static void (*sum_log_cb)(const char *line);
#ifdef _WIN32
static ULONGLONG sum_cpu_idle0, sum_cpu_kernel0, sum_cpu_user0, sum_own0;
static int sum_cpu_primed;
#endif

static void sum_cfg(void) {
    const char *v;
    unsigned i;
    if (sum_cfg_done) return;
    sum_cfg_done = 1;
    for (i = 0; i < GW_PROF_ZONE_COUNT; ++i) sum_slot[i] = 0xFF;
    for (i = 0; i < SUM_NB; ++i) sum_slot[sum_bucket[i].zone] = (unsigned char)i;
    v = getenv("MELEE_PERF_HITCH_MS");
    sum_hitch_ms = v && atof(v) > 0 ? atof(v) : 1000.0 / 60.0;
    v = getenv("MELEE_PERF_WINDOW");
    if (v && sscanf(v, "%u,%u", &sum_win_warm, &sum_win_frames) == 2 && sum_win_frames > SUM_FW_CAP) sum_win_frames = SUM_FW_CAP;
    v = getenv("MELEE_PERF_PATH");
    snprintf(sum_path, sizeof sum_path, "%s", v && *v ? v : "perf.json");
}
int gw_prof_summary_enabled(void) { return prof_load(&prof_sum) != 0; }
int gw_prof_active(void) { return prof_load(&prof_on) != 0 || prof_load(&prof_sum) != 0; }
void gw_prof_set_conditions_cb(int (*fn)(char *out, unsigned cap)) { sum_cond_cb = fn; }
void gw_prof_set_log_cb(void (*fn)(const char *line)) { sum_log_cb = fn; }
#ifdef _WIN32
static void sum_set(unsigned value) { InterlockedExchange(&prof_sum, (LONG)value); }
#else
static void sum_set(unsigned value) { atomic_store(&prof_sum, value); }
#endif

/* ---- zone hooks ---- */
static void sum_begin(unsigned zone, unsigned detail) {
    SumOpen *e;
    unsigned s;
    if (sum_tls.depth >= SUM_DEPTH) { sum_tls.overflow++; return; }
    e = &sum_tls.st[sum_tls.depth++];
    e->zone = (unsigned short)zone; e->mod = 0xFF; e->clocked = 0; e->slot = 0xFF;
    s = zone < GW_PROF_ZONE_COUNT ? sum_slot[zone] : 0xFF;
    if (s != 0xFF) {
        e->slot = (unsigned char)s;
        if (!sum_tls.open[s]++) { e->start = prof_now(); e->clocked = 1; }
    }
    if (zone == GW_PROF_LUA_SCRIPT && detail) { /* full profiler path: map the identity hash to a mod slot */
        unsigned k;
        for (k = 0; k < sum_mod_count; ++k) if (sum_mod_detail[k] == detail) { e->mod = (unsigned char)k; break; }
        if (e->mod == 0xFF) {
            unsigned j; char name[128] = "";
            prof_lock();
            for (j = 0; j < prof.label_count; ++j) if (prof.labels[j].detail == detail) {
                snprintf(name, sizeof name, "%s", prof.labels[j].name); break; }
            prof_unlock();
            if (!strncmp(name, "script:", 7) && sum_mod_count < SUM_MODS) {
                snprintf(sum_mod_id[sum_mod_count], sizeof sum_mod_id[0], "%s", name + 7);
                sum_mod_detail[sum_mod_count] = detail;
                e->mod = (unsigned char)sum_mod_count++;
            }
        }
        if (e->mod != 0xFF && !e->clocked) { e->start = prof_now(); e->clocked = 2; }
    }
}
void gw_prof_script_begin(const char *id) {
    unsigned k;
    if (!prof_load(&prof_sum) || !id) return;
    for (k = 0; k < sum_mod_count; ++k) if (sum_mod_ptr[k] == id) break;
    if (k == sum_mod_count) {
        unsigned j;
        for (j = 0; j < sum_mod_count; ++j) if (!strncmp(sum_mod_id[j], id, sizeof sum_mod_id[0] - 1)) break;
        if (j == sum_mod_count) {
            if (sum_mod_count >= SUM_MODS) { sum_begin(GW_PROF_LUA_SCRIPT, 0); return; }
            snprintf(sum_mod_id[j], sizeof sum_mod_id[0], "%s", id);
            sum_mod_count++;
        }
        sum_mod_ptr[j] = id; k = j;
    }
    sum_begin(GW_PROF_LUA_SCRIPT, 0);
    if (sum_tls.depth && sum_tls.depth <= SUM_DEPTH) {
        SumOpen *e = &sum_tls.st[sum_tls.depth - 1];
        e->mod = (unsigned char)k;
        if (!e->clocked) { e->start = prof_now(); e->clocked = 2; }
    }
}
static void sum_end(void) {
    SumOpen *e;
    double now;
    if (sum_tls.overflow) { sum_tls.overflow--; return; }
    if (!sum_tls.depth) return;
    e = &sum_tls.st[--sum_tls.depth];
    if (e->slot != 0xFF) {
        unsigned char outer = --sum_tls.open[e->slot] == 0;
        if (e->clocked == 1 && outer) {
            now = prof_now();
            SUM_ADD(&sum_ns[e->slot], (now - e->start) * 1e6);
            if (e->mod != 0xFF && e->mod < SUM_MODS && sum_scene.active) sum_scene.mod_ms[e->mod] += now - e->start;
            return;
        }
    }
    if (e->clocked && e->mod != 0xFF && e->mod < SUM_MODS && sum_scene.active) /* nested script: its own mod only */
        sum_scene.mod_ms[e->mod] += prof_now() - e->start;
}
static unsigned sum_mark(void) { return sum_tls.depth + sum_tls.overflow; }
static void sum_unwind(unsigned mark) { while (sum_tls.depth + sum_tls.overflow > mark) sum_end(); }
static void sum_add(unsigned id, double ms) {
    unsigned s;
    if (!prof_load(&prof_sum) || id >= GW_PROF_ZONE_COUNT || !(ms >= 0)) return;
    s = sum_slot[id];
    if (s != 0xFF && s != SUM_PACING) SUM_ADD(&sum_ns[s], ms * 1e6);
}
void gw_prof_tally(unsigned id) {
    if (id >= GW_PROF_ENV_SETUPS && id < GW_PROF_ENV_SETUPS + SUM_NC) sum_cnt[id - GW_PROF_ENV_SETUPS]++;
}
void gw_prof_frame_stats(unsigned draw_calls, unsigned gx_begins, unsigned gx_dlists, int presented,
                         unsigned end_frames, double replay_ms) {
    sum_frame_draws = draw_calls;
    sum_end_frames = end_frames;
    sum_replay_ms = replay_ms;
    sum_presented = presented != 0;
    sum_cnt[GW_PROF_GX_BEGINS - GW_PROF_ENV_SETUPS] = gx_begins;
    sum_cnt[GW_PROF_GX_DLISTS - GW_PROF_ENV_SETUPS] = gx_dlists;
    if (prof_load(&prof_on)) gw_prof_counter(GW_PROF_DRAW_CALLS, draw_calls);
}
void gw_prof_gx_ms(double ms) {
    sum_gx_ms = ms;
    if (prof_load(&prof_on) && ms >= 0) gw_prof_sample(GW_PROF_GX_RECORD, ms, 0);
}

/* ---- accumulators ---- */
static void sum_res(float *arr, unsigned cap, uint64_t count, double v, unsigned salt) {
    uint64_t sel = count;
    if (sel >= cap) {
        uint64_t h = (sel + 1) ^ ((uint64_t)salt << 32);
        h ^= h >> 33; h *= UINT64_C(0xff51afd7ed558ccd); h ^= h >> 33; h *= UINT64_C(0xc4ceb9fe1a85ec53); h ^= h >> 33;
        sel = h % (sel + 1);
    }
    if (sel < cap) arr[sel] = (float)v;
}
static int sum_fcmp(const void *a, const void *b) {
    float x = *(const float *)a, y = *(const float *)b;
    return (x > y) - (x < y);
}
static double sum_pct(const float *v, unsigned n, double p) {
    unsigned rank = (unsigned)ceil(p * n);
    return n ? v[(rank ? rank : 1) - 1] : 0;
}
static void sum_acc_add(SumAcc *a, double fw, const double *b, unsigned draws, int has_draws, const unsigned *cnt, double gx, unsigned seed) {
    unsigned i;
    sum_res(a->fw, SUM_FW_CAP, a->n, fw, seed);
    for (i = 0; i < SUM_NB; ++i) sum_res(a->bs[i], SUM_B_CAP, a->n, b[i], seed + 1 + i);
    a->n++;
    a->fw_sum += fw; if (fw > a->fw_max) a->fw_max = fw;
    for (i = 0; i < SUM_NB; ++i) { a->b_sum[i] += b[i]; if (b[i] > a->b_max[i]) a->b_max[i] = b[i]; }
    if (fw > sum_hitch_ms) { a->hitch_n++; a->hitch_ms += fw; if (fw > a->hitch_worst) a->hitch_worst = fw; }
    if (fw > 2 * sum_hitch_ms) a->hitch33_n++;
    if (has_draws) { a->dc_n++; a->dc_sum += draws; if (draws > a->dc_max) a->dc_max = draws; }
    for (i = 0; i < SUM_NC; ++i) a->cnt_sum[i] += cnt[i];
    if (gx >= 0) { a->gx_sum += gx; a->gx_n++; }
}

#define SW(...) do { if (n < cap) n += (size_t)snprintf(out + n, cap - n, __VA_ARGS__); } while (0)
static size_t sum_acc_json(char *out, size_t cap, SumAcc *a) {
    static float tmp[SUM_FW_CAP];
    size_t n = 0;
    unsigned i, m;
    double fn;
    if (!a->n) { SW("{\"frames\":0}"); return n; }
    fn = (double)a->n;
    m = a->n < SUM_FW_CAP ? (unsigned)a->n : SUM_FW_CAP;
    memcpy(tmp, a->fw, m * sizeof(float)); qsort(tmp, m, sizeof(float), sum_fcmp);
    SW("{\"frames\":%llu,\"frame_work\":{\"mean\":%.4f,\"p95\":%.4f,\"p99\":%.4f,\"max\":%.4f},"
       "\"hitches\":{\"over_ms\":%.2f,\"count\":%llu,\"total_ms\":%.2f,\"worst_ms\":%.2f,\"over_2x\":%llu},\"buckets\":{",
       (unsigned long long)a->n, a->fw_sum / fn, sum_pct(tmp, m, .95), sum_pct(tmp, m, .99), a->fw_max,
       sum_hitch_ms, (unsigned long long)a->hitch_n, a->hitch_ms, a->hitch_worst, (unsigned long long)a->hitch33_n);
    for (i = 0; i < SUM_NB; ++i) {
        unsigned bm = a->n < SUM_B_CAP ? (unsigned)a->n : SUM_B_CAP;
        memcpy(tmp, a->bs[i], bm * sizeof(float)); qsort(tmp, bm, sizeof(float), sum_fcmp);
        SW("%s\"%s\":{\"mean\":%.4f,\"p95\":%.4f,\"max\":%.4f}", i ? "," : "", sum_bucket[i].name,
           a->b_sum[i] / fn, sum_pct(tmp, bm, .95), a->b_max[i]);
    }
    SW("},\"draw_calls\":{\"mean\":%.1f,\"max\":%u},\"per_frame\":{", a->dc_n ? a->dc_sum / a->dc_n : 0, a->dc_max);
    for (i = 0; i < SUM_NC; ++i) SW("%s\"%s\":%.2f", i ? "," : "", sum_cnt_name[i], a->cnt_sum[i] / fn);
    SW("}");
    if (a->gx_n) SW(",\"gx_record_ms\":%.4f", a->gx_sum / a->gx_n);
    SW("}");
    return n;
}

/* ---- machine state ---- */
static unsigned sum_other_instances(void) {
#ifdef _WIN32
    HANDLE snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
    PROCESSENTRY32 pe;
    unsigned n = 0;
    DWORD self = GetCurrentProcessId();
    if (snap == INVALID_HANDLE_VALUE) return 0xFFFFu;
    pe.dwSize = sizeof pe; /* names only: never command lines */
    if (Process32First(snap, &pe)) do {
        if (pe.th32ProcessID != self && !_stricmp(pe.szExeFile, "melee-pc.exe")) n++;
    } while (Process32Next(snap, &pe));
    CloseHandle(snap);
    return n;
#else
    return 0;
#endif
}
static void sum_memory(double *peak_ws, double *peak_commit, double *ws) {
#ifdef _WIN32
    PROCESS_MEMORY_COUNTERS pm;
    memset(&pm, 0, sizeof pm); pm.cb = sizeof pm;
    if (K32GetProcessMemoryInfo(GetCurrentProcess(), &pm, sizeof pm)) {
        *peak_ws = pm.PeakWorkingSetSize / 1048576.0; *peak_commit = pm.PeakPagefileUsage / 1048576.0;
        *ws = pm.WorkingSetSize / 1048576.0; return;
    }
#endif
    *peak_ws = *peak_commit = *ws = 0;
}
/* Machine CPU load from OTHER processes, as % of all logical CPUs, sampled about once a second. */
static void sum_cpu_sample(double now_ms) {
#ifdef _WIN32
    FILETIME i, k, u, c, e, pk, pu;
    ULONGLONG idle, kern, user, own;
    if (!GetSystemTimes(&i, &k, &u)) return;
    GetProcessTimes(GetCurrentProcess(), &c, &e, &pk, &pu);
#define FT(f) (((ULONGLONG)(f).dwHighDateTime << 32) | (f).dwLowDateTime)
    idle = FT(i); kern = FT(k); user = FT(u); own = FT(pk) + FT(pu);
#undef FT
    if (sum_cpu_primed) {
        double total = (double)((kern - sum_cpu_kernel0) + (user - sum_cpu_user0)); /* kernel includes idle */
        double busy = total - (double)(idle - sum_cpu_idle0);
        double mine = (double)(own - sum_own0);
        double load = total > 0 ? 100.0 * (busy - mine) / total : 0;
        if (load < 0) load = 0;
        if (sum_scene.lo_n == 0) sum_scene.lo_first = load;
        sum_scene.lo_last = load; sum_scene.lo_sum += load; sum_scene.lo_n++;
        if (load > sum_scene.lo_max) sum_scene.lo_max = load;
    }
    sum_cpu_idle0 = idle; sum_cpu_kernel0 = kern; sum_cpu_user0 = user; sum_own0 = own; sum_cpu_primed = 1;
#endif
    sum_last_cpu_ms = now_ms;
}

/* ---- scene objects ---- */
static void sum_json_escape(char *dst, size_t cap, const char *s) {
    size_t n = 0;
    for (; *s && n + 7 < cap; ++s) {
        unsigned char c = (unsigned char)*s;
        if (c == '"' || c == '\\') { dst[n++] = '\\'; dst[n++] = (char)c; }
        else if (c < 32) n += (size_t)snprintf(dst + n, cap - n, "\\u%04x", c);
        else dst[n++] = (char)c;
    }
    dst[n] = 0;
}
static size_t sum_scene_json(char *out, size_t cap, int in_progress) {
    size_t n = 0;
    unsigned i, k = 0;
    char esc[200], cond[2048];
    double now = prof_now(), wall = (now - sum_scene.t0) / 1000.0;
    double pw, pc, ws;
    sum_memory(&pw, &pc, &ws);
    sum_json_escape(esc, sizeof esc, sum_scene.name);
    SW("{\"name\":\"%s\",\"in_progress\":%s,\"frames\":%llu,\"wall_s\":%.2f,\"presents\":%llu,\"present_fps\":%.1f,\"replay_cost_ms\":%.2f,\"first_use_frames\":%d,\n \"first_use\":",
       esc, in_progress ? "true" : "false", (unsigned long long)sum_scene.frames, wall,
       (unsigned long long)sum_scene.presents,
       wall > 0 ? (double)(sum_end_frames - sum_scene.end0) / wall : 0.0, sum_replay_ms, SUM_FIRST_FRAMES);
    n += sum_acc_json(out + n, n < cap ? cap - n : 0, &sum_first);
    SW(",\n \"steady\":");
    n += sum_acc_json(out + n, n < cap ? cap - n : 0, &sum_steady);
    if (sum_win_frames) {
        SW(",\n \"window\":{\"warm\":%u,\"frames\":%u,\"complete\":%s,\"present_fps\":%.1f,\"replays_per_frame\":%.2f,\"stats\":", sum_win_warm, sum_win_frames,
           sum_scene.win_done ? "true" : "false", sum_win_present_fps, sum_win_replays_per_frame);
        n += sum_acc_json(out + n, n < cap ? cap - n : 0, &sum_win);
        SW("}");
    }
    SW(",\n \"script_ms_per_frame\":{");
    for (i = 0; i < sum_mod_count; ++i) if (sum_scene.mod_ms[i] > 0 && sum_scene.frames) {
        char id[96];
        sum_json_escape(id, sizeof id, sum_mod_id[i]);
        SW("%s\"%s\":%.4f", k++ ? "," : "", id, sum_scene.mod_ms[i] / (double)sum_scene.frames);
    }
    SW("},\n \"peak_working_set_mb\":%.1f,\"peak_commit_mb\":%.1f,\"working_set_mb\":%.1f,", pw, pc, ws);
    SW("\"machine\":{\"other_melee_instances_start\":%d,\"other_melee_instances_end\":%d,"
       "\"other_cpu_load_pct\":{\"first\":%.1f,\"last\":%.1f,\"mean\":%.1f,\"max\":%.1f}},",
       sum_scene.others_start == 0xFFFFu ? -1 : (int)sum_scene.others_start,
       sum_scene.others_end == 0xFFFFu ? -1 : (int)sum_scene.others_end,
       sum_scene.lo_first, sum_scene.lo_last, sum_scene.lo_n ? sum_scene.lo_sum / sum_scene.lo_n : 0, sum_scene.lo_max);
    cond[0] = 0;
    if (sum_cond_cb) sum_cond_cb(cond, sizeof cond);
    SW("\n \"conditions\":{%s}}", cond);
    return n;
}
#undef SW
static int sum_write_file(int in_progress_scene) {
    FILE *f;
    char tmp[1100];
    unsigned i, first;
    char cond[2048];
    snprintf(tmp, sizeof tmp, "%s.tmp", sum_path);
    f = fopen(tmp, "wb");
    if (!f) return 0;
    cond[0] = 0;
    if (sum_cond_cb) sum_cond_cb(cond, sizeof cond);
    fprintf(f, "{\"schema_version\":1,\"kind\":\"perf\",\"budget_ms\":8.333333333,\"pid\":%lu,\"summary\":%s,\"full_profiler\":%s,"
            "\"scenes_total\":%u,\"scenes_under_120_frames\":%u,\"run_conditions\":{%s},\"scenes\":[\n",
#ifdef _WIN32
            (unsigned long)GetCurrentProcessId(),
#else
            0ul,
#endif
            prof_load(&prof_sum) ? "true" : "false", prof_load(&prof_on) ? "true" : "false",
            sum_scenes_total, sum_scenes_short, cond);
    first = sum_json_count > SUM_SCENES ? sum_json_count - SUM_SCENES : 0;
    for (i = first; i < sum_json_count; ++i) fprintf(f, "%s%s", i > first ? ",\n" : "", sum_json[i % SUM_SCENES]);
    if (in_progress_scene && sum_scene.active && sum_scene.recorded) {
        static char buf[SUM_JSON];
        sum_scene_json(buf, sizeof buf, 1);
        fprintf(f, "%s%s", sum_json_count > first ? ",\n" : "", buf);
    }
    fprintf(f, "\n]}\n");
    { int ok = !ferror(f); if (fclose(f)) ok = 0;
      if (!ok) return 0;
#ifdef _WIN32
      return MoveFileExA(tmp, sum_path, MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH) != 0;
#else
      return rename(tmp, sum_path) == 0;
#endif
    }
}
int gw_prof_perf_write(const char *path) {
    if (path && *path) snprintf(sum_path, sizeof sum_path, "%s", path);
    sum_cfg();
    return sum_write_file(1);
}
static void sum_log_scene(void) {
    char line[520];
    SumAcc *a = sum_steady.n ? &sum_steady : &sum_first;
    static float tmp[SUM_FW_CAP];
    unsigned m = a->n < SUM_FW_CAP ? (unsigned)a->n : SUM_FW_CAP;
    double fn;
    if (!sum_log_cb || !a->n) return;
    fn = (double)a->n;
    memcpy(tmp, a->fw, m * sizeof(float)); qsort(tmp, m, sizeof(float), sum_fcmp);
    snprintf(line, sizeof line, "perf: scene=\"%s\" frames=%llu wall=%.1fs %s frame_work mean=%.2f p95=%.2f p99=%.2f max=%.2f ms | logic=%.2f draw=%.2f script=%.2f submit=%.2f worker=%.2f | draws mean=%.0f max=%u | hitches>%.1fms=%llu | others=%d/%d load=%.0f%%",
             sum_scene.name, (unsigned long long)sum_scene.frames, (prof_now() - sum_scene.t0) / 1000.0,
             sum_steady.n ? "steady" : "first-use", a->fw_sum / fn, sum_pct(tmp, m, .95), sum_pct(tmp, m, .99), a->fw_max,
             a->b_sum[0] / fn, a->b_sum[1] / fn, a->b_sum[6] / fn, a->b_sum[3] / fn, a->b_sum[2] / fn,
             a->dc_n ? a->dc_sum / a->dc_n : 0, a->dc_max, sum_hitch_ms, (unsigned long long)a->hitch_n,
             sum_scene.others_start == 0xFFFFu ? -1 : (int)sum_scene.others_start,
             sum_scene.others_end == 0xFFFFu ? -1 : (int)sum_scene.others_end,
             sum_scene.lo_n ? sum_scene.lo_sum / sum_scene.lo_n : 0);
    sum_log_cb(line);
}
void gw_prof_scene_end(void) {
    if (!sum_scene.active) return;
    if (sum_scene.recorded) {
        sum_scene.others_end = sum_other_instances();
        sum_scene_json(sum_json[sum_json_count % SUM_SCENES], SUM_JSON, 0);
        sum_json_count++; sum_scenes_total++;
        sum_log_scene();
        sum_scene.active = 0;
        sum_write_file(0);
    } else {
        sum_scenes_short++; sum_scenes_total++;
        sum_scene.active = 0;
    }
}
void gw_prof_scene_begin(const char *name) {
    if (!prof_load(&prof_sum)) return;
    sum_cfg();
    if (sum_scene.active) gw_prof_scene_end();
    memset(&sum_first, 0, sizeof sum_first);
    memset(&sum_steady, 0, sizeof sum_steady);
    memset(&sum_win, 0, sizeof sum_win);
    memset(&sum_scene, 0, sizeof sum_scene);
    sum_scene.active = 1;
    snprintf(sum_scene.name, sizeof sum_scene.name, "%s", name ? name : "?");
    sum_scene.t0 = prof_now();
    sum_scene.others_start = sum_scene.others_end = 0xFFFFu;
#ifdef _WIN32
    sum_cpu_primed = 0;
#endif
}

/* ---- the frame tick ---- */
static void sum_frame_begin(void) {
    sum_frame_start = prof_now();
    sum_frame_active = 1;
}
static void sum_frame_end(void) {
    double b[SUM_NB], now, fw;
    unsigned cnt[SUM_NC], i, draws = sum_frame_draws;
    SumAcc *acc;
    double gx;
    if (!sum_frame_active) return;
    sum_frame_active = 0;
    now = prof_now();
    for (i = 0; i < SUM_NB; ++i) b[i] = (double)SUM_XCHG(&sum_ns[i]) * 1e-6;
    for (i = 0; i < SUM_NC; ++i) { cnt[i] = sum_cnt[i]; sum_cnt[i] = 0; }
    sum_frame_draws = 0;
    fw = now - sum_frame_start - b[SUM_PACING];
    if (fw < 0) fw = 0;
    if (!sum_scene.active) gw_prof_scene_begin("boot"); /* frames before the first scene report */
    sum_scene.frames++;
    sum_scene.presents += (uint64_t)sum_presented; sum_presented = 0;
    if (!sum_scene.end0_set) { sum_scene.end0 = sum_end_frames; sum_scene.end0_set = 1; }
    if (sum_win_frames && sum_scene.frames == sum_win_warm) { sum_win_end0 = sum_end_frames; sum_win_t0 = now; }
    if (prof_load(&prof_on)) for (i = 0; i < SUM_NC; ++i) gw_prof_counter(GW_PROF_ENV_SETUPS + i, cnt[i]);
    gx = sum_gx_ms; sum_gx_ms = -1;
    acc = sum_scene.frames <= SUM_FIRST_FRAMES ? &sum_first : &sum_steady;
    sum_acc_add(acc, fw, b, draws, draws != 0, cnt, gx, (unsigned)sum_scene.frames * 3u);
    if (sum_win_frames && sum_scene.frames > sum_win_warm && sum_scene.frames <= (uint64_t)sum_win_warm + sum_win_frames) {
        sum_acc_add(&sum_win, fw, b, draws, draws != 0, cnt, gx, 7);
        if (sum_scene.frames == (uint64_t)sum_win_warm + sum_win_frames) {
            sum_scene.win_done = 1;
            sum_win_present_fps = (now > sum_win_t0) ? (double)(sum_end_frames - sum_win_end0) * 1000.0 / (now - sum_win_t0) : 0;
            sum_win_replays_per_frame = sum_win_frames ? (double)(sum_end_frames - sum_win_end0) / sum_win_frames - 1.0 : 0;
            if (sum_log_cb) sum_log_cb("perf: bench window complete");
        }
    }
    if (now - sum_last_cpu_ms >= 1000.0) sum_cpu_sample(now);
    if (!sum_scene.recorded && sum_scene.frames >= 120) { /* long enough to matter: take the start-of-scene machine state */
        sum_scene.recorded = 1;
        sum_scene.others_start = sum_other_instances();
    }
    if (sum_scene.recorded && sum_scene.frames % 3600 == 0) sum_write_file(1);
}
/* The current scene's draw-call numbers (first-use and steady together): for the suite and bench harness. */
int gw_prof_scene_draws(double *mean, unsigned *max, unsigned long long *frames, unsigned long long *with_draws) {
    uint64_t n = sum_first.dc_n + sum_steady.dc_n;
    if (mean) *mean = n ? (sum_first.dc_sum + sum_steady.dc_sum) / (double)n : 0;
    if (max) *max = sum_first.dc_max > sum_steady.dc_max ? sum_first.dc_max : sum_steady.dc_max;
    if (frames) *frames = sum_scene.frames;
    if (with_draws) *with_draws = n;
    return sum_scene.active;
}
int gw_prof_perf_status(char *out, unsigned cap) {
    snprintf(out, cap, "perf scene=\"%s\" frames=%llu window=%s summary=%d", sum_scene.name,
             (unsigned long long)sum_scene.frames,
             !sum_win_frames ? "off" : sum_scene.win_done ? "done" : "pending", prof_load(&prof_sum) ? 1 : 0);
    return 1;
}
void gw_prof_summary_set(int on) {
    gw_prof_init();
    sum_cfg();
    sum_set(on != 0);
#ifdef _WIN32
    aurora_profiler_enable(prof_load(&prof_on) != 0 || on != 0);
#endif
}
void gw_ProfCount(int id) { gw_prof_tally((unsigned)id); }
int gw_ProfActive(void) { return gw_prof_active(); }

void gw_prof_context(unsigned kind, const char *name) {
    if (!gw_prof_enabled() || kind >= 3 || !name) return;
    prof_lock();
    snprintf(prof.context[kind], sizeof prof.context[kind], "%s", name);
    prof_unlock();
}
unsigned gw_prof_log_frame(void) { return prof_load(&prof_frame); }
static void prof_event(ProfEvent e) {
    /* Exact aggregates above remain complete. Tiny repetitive zones/counters get
     * eight representative events per ID/frame, so a hot callback cannot erase
     * the 120-frame hitch neighbourhood. Significant spans are always retained. */
    if (prof.admission_frame[e.id] != e.frame) {
        prof.admission_frame[e.id] = e.frame;
        prof.admission_count[e.id] = 0;
    }
    if (e.duration < .25 && e.id != GW_PROF_FRAME &&
        prof.admission_count[e.id]++ >= 8) { prof.event_sampled++; return; }
    memcpy(e.context, prof.context, sizeof e.context);
    prof.events[prof.event_count++ % PROF_EVENTS] = e;
    if (prof_load(&prof_frame) > e.frame + PROF_HISTORY) prof.late_events++;
}
void gw_prof_begin(unsigned id, unsigned detail) {
    if (prof_load(&prof_sum)) sum_begin(id, detail);
    if (!gw_prof_enabled()) return;
    prof_tls_sync();
    if (prof_tls.depth == PROF_DEPTH || prof_tls.overflow) {
        prof_tls.overflow++;
        prof_lock(); prof.stack_overflow++; prof_unlock();
        return;
    }
    /* Invalid begins still occupy one stack slot, preserving the caller's matching end. */
    prof_tls.stack[prof_tls.depth++] = (ProfOpen){prof_now(), id, detail, prof_load(&prof_frame)};
#ifdef GW_PROF_TRACY
    if (id < GW_PROF_ZONE_COUNT) {
        ProfOpen *o = &prof_tls.stack[prof_tls.depth - 1];
        o->tracy = ___tracy_emit_zone_begin(&prof_tracy_locations[id], 1);
        ___tracy_emit_zone_value(o->tracy, detail);
    }
#endif
}
void gw_prof_end(void) {
    ProfOpen open;
    ProfEvent e;
    unsigned generation;
    if (prof_load(&prof_sum)) sum_end();
    if (!gw_prof_enabled()) return;
    prof_tls_sync();
    if (prof_tls.overflow) { prof_tls.overflow--; return; }
    if (!prof_tls.depth) { prof_lock(); prof.unmatched_end++; prof_unlock(); return; }
    generation = prof_tls.generation;
    open = prof_tls.stack[--prof_tls.depth];
#ifdef GW_PROF_TRACY
    if (open.id < GW_PROF_ZONE_COUNT) ___tracy_emit_zone_end(open.tracy);
#endif
    e = (ProfEvent){open.start, prof_now() - open.start, 0, open.id, open.detail,
                   prof_thread(), open.frame, 'X'};
    if (open.id == GW_PROF_PACING && prof_tls.frame_active) prof_tls.pacing_ms += e.duration;
    prof_lock();
    if (generation != prof_load(&prof_generation)) { prof_unlock(); return; }
    if (open.id >= GW_PROF_ZONE_COUNT) { prof.invalid_id++; prof_unlock(); return; }
    prof_add(open.id, e.duration, open.detail);
    prof_event(e);
    prof_unlock();
}
void gw_prof_counter(unsigned id, double value) {
    if (!gw_prof_enabled()) return;
    gw_prof_counter_detail(id, 0, value);
}
void gw_prof_counter_detail(unsigned id, unsigned detail, double value) {
    ProfEvent e;
    unsigned generation;
    if (!gw_prof_enabled()) return;
    if (!isfinite(value)) return;
    generation = prof_load(&prof_generation);
    e = (ProfEvent){prof_now(), 0, value, id, detail, prof_thread(), prof_load(&prof_frame), 'C'};
    prof_lock();
    if (generation != prof_load(&prof_generation)) { prof_unlock(); return; }
    if (id < GW_PROF_MEX_INSTRUCTIONS || id >= GW_PROF_COUNTER_END) {
        prof.invalid_id++; prof_unlock(); return;
    }
    prof_add(id, value, detail);
#ifdef GW_PROF_TRACY
    TracyCPlot(prof_names[id], value);
#endif
    prof_event(e);
    prof_unlock();
}
void gw_prof_sample(unsigned id, double milliseconds, unsigned detail) {
    unsigned generation;
    sum_add(id, milliseconds);
    if (!gw_prof_enabled()) return;
    if (!isfinite(milliseconds) || milliseconds < 0) return;
    generation = prof_load(&prof_generation);
    prof_lock();
    if (generation != prof_load(&prof_generation)) { prof_unlock(); return; }
    if (id >= GW_PROF_ZONE_COUNT) { prof.invalid_id++; prof_unlock(); return; }
    prof_add(id, milliseconds, detail);
    if (id == GW_PROF_GPU) prof.gpu_available = 1;
    prof_unlock();
}
void gw_prof_cpu_completed(unsigned id, unsigned detail, double milliseconds) {
    ProfEvent e;
    unsigned generation;
    sum_add(id, milliseconds);
    if (!gw_prof_enabled()) return;
    if (!isfinite(milliseconds) || milliseconds < 0) return;
    generation = prof_load(&prof_generation);
    e = (ProfEvent){prof_now() - milliseconds, milliseconds, 0, id, detail,
                   prof_thread(), prof_load(&prof_frame), 'A'};
    prof_lock();
    if (generation != prof_load(&prof_generation)) { prof_unlock(); return; }
    if (id >= GW_PROF_ZONE_COUNT || id == GW_PROF_GPU || id == GW_PROF_GPU_PASS) {
        prof.invalid_id++; prof_unlock(); return;
    }
    prof_add(id, milliseconds, detail);
    prof_event(e);
    prof_unlock();
}
void gw_prof_gpu_completed(unsigned id, unsigned detail, double milliseconds, unsigned long long render_frame) {
    ProfEvent e;
    unsigned generation;
    if (!gw_prof_enabled()) return;
    if (!isfinite(milliseconds) || milliseconds < 0) return;
    generation = prof_load(&prof_generation);
    e = (ProfEvent){prof_now(), 0, milliseconds, id, detail, prof_thread(),
                   prof_load(&prof_frame), 'G', render_frame};
    prof_lock();
    if (generation != prof_load(&prof_generation)) { prof_unlock(); return; }
    if (id != GW_PROF_GPU && id != GW_PROF_GPU_PASS) { prof.invalid_id++; prof_unlock(); return; }
    prof_add(id, milliseconds, detail);
    prof.gpu_available = 1;
    prof_event(e);
    prof_unlock();
}
void gw_prof_try_cpu_completed(unsigned id, unsigned detail, double milliseconds) {
    ProfEvent e;
    unsigned generation;
    sum_add(id, milliseconds);
    if (!gw_prof_enabled()) return;
    if (!isfinite(milliseconds) || milliseconds < 0) return;
    generation = prof_load(&prof_generation);
    e = (ProfEvent){prof_now() - milliseconds, milliseconds, 0, id, detail,
                   prof_thread(), prof_load(&prof_frame), 'A'};
    if (!prof_try_lock()) { prof_next(&prof_dropped); return; }
    if (generation != prof_load(&prof_generation)) { prof_unlock(); return; }
    if (id >= GW_PROF_ZONE_COUNT || id == GW_PROF_GPU || id == GW_PROF_GPU_PASS) {
        prof.invalid_id++; prof_unlock(); return;
    }
    prof_add(id, milliseconds, detail);
    prof_event(e);
    prof_unlock();
}
void gw_prof_frame_begin(void) {
    prof_next(&prof_frame); /* Log frame numbers exist even with collection off. */
    if (prof_load(&prof_sum)) sum_frame_begin();
    if (!gw_prof_enabled()) return;
    prof_tls_sync();
    prof_tls.frame_start = prof_now();
    prof_tls.pacing_ms = 0;
    prof_tls.frame_active = 1;
    gw_prof_begin(GW_PROF_FRAME, 0);
}
void gw_prof_frame_end(void) {
    double elapsed;
    unsigned frame, hitch = 0;
    char path[1200];
    if (prof_load(&prof_sum)) sum_frame_end();
    if (!gw_prof_enabled()) return;
    prof_tls_sync();
    if (!prof_tls.frame_active) return;
    prof_tls.frame_active = 0;
    elapsed = prof_now() - prof_tls.frame_start - prof_tls.pacing_ms;
    if (elapsed < 0) elapsed = 0;
    gw_prof_sample(GW_PROF_FRAME_WORK, elapsed, 0);
    gw_prof_end();
#ifdef GW_PROF_TRACY
    TracyCFrameMark;
#endif
    frame = prof_load(&prof_frame);
    prof_lock();
    prof.completed++;
    if (prof.hitch_pending && frame >= prof.hitch_pending + 2) {
        hitch = prof.hitch_pending;
        prof.hitch_pending = 0;
        prof.hitch_last = frame;
    }
    if (!prof.hitch_pending && elapsed > prof.hitch_ms &&
        (!prof.hitch_last || frame > prof.hitch_last + PROF_HISTORY)) prof.hitch_pending = frame;
    prof_unlock();
    if (hitch) {
        snprintf(path, sizeof path, "%s.hitch-%u.json", prof.trace_path, hitch);
        gw_prof_trace(path, 5);
    }
}
unsigned long long gw_prof_frames(void) {
    unsigned long long n;
    prof_lock(); n = prof.completed; prof_unlock();
    return n;
}
static int prof_compare(const void *a, const void *b) {
    double x = *(const double *)a, y = *(const double *)b;
    return (x > y) - (x < y);
}
static double prof_percentile(const double *values, unsigned n, double p) {
    /* Nearest rank, including singleton and exact integer ranks. */
    unsigned rank = (unsigned)ceil(p * n);
    return n ? values[(rank ? rank : 1) - 1] : 0;
}
int gw_prof_stats(unsigned id, GwProfStats *out) {
    double values[PROF_SAMPLES];
    unsigned n;
    ProfSeries *s;
    if (!out || !gw_prof_name(id)) return 0;
    memset(out, 0, sizeof *out);
    prof_lock();
    s = &prof.series[id];
    out->count = s->count;
    out->mean = s->count ? s->sum / s->count : 0;
    out->max = s->max;
    n = s->count < PROF_SAMPLES ? (unsigned)s->count : PROF_SAMPLES;
    memcpy(values, s->sample, n * sizeof *values);
    prof_unlock();
    qsort(values, n, sizeof *values, prof_compare);
    out->p50 = prof_percentile(values, n, .50);
    out->p95 = prof_percentile(values, n, .95);
    out->p99 = prof_percentile(values, n, .99);
    return 1;
}
static void prof_write_stats(FILE *f, GwProfStats s) {
    fprintf(f, "{\"count\":%llu,\"mean\":%.9g,\"p50\":%.9g,\"p95\":%.9g,\"p99\":%.9g,\"max\":%.9g}",
            s.count, s.mean, s.p50, s.p95, s.p99, s.max);
}
int gw_prof_details(GwProfDetailStats *out, int capacity) {
    unsigned i, count;
    if (!out || capacity <= 0) return 0;
    prof_lock();
    count = prof.detail_count < (unsigned)capacity ? prof.detail_count : (unsigned)capacity;
    for (i = 0; i < count; ++i) {
        ProfDetail *d = &prof.details[i];
        double values[PROF_DETAIL_SAMPLES];
        unsigned j, n = d->count < PROF_DETAIL_SAMPLES ? (unsigned)d->count : PROF_DETAIL_SAMPLES;
        memset(&out[i], 0, sizeof out[i]);
        out[i].id = d->id; out[i].detail = d->detail;
        out[i].stats.count = d->count;
        out[i].stats.mean = d->sum / d->count;
        out[i].stats.max = d->max;
        memcpy(values, d->sample, n * sizeof *values);
        qsort(values, n, sizeof *values, prof_compare);
        out[i].stats.p50 = prof_percentile(values,n,.50);
        out[i].stats.p95 = prof_percentile(values,n,.95);
        out[i].stats.p99 = prof_percentile(values,n,.99);
        for (j = 0; j < prof.label_count; ++j) if (prof.labels[j].detail == d->detail) {
            memcpy(out[i].name, prof.labels[j].name, sizeof out[i].name); break;
        }
    }
    prof_unlock();
    return (int)count;
}
void gw_prof_detail_name(unsigned detail, const char *name) {
    unsigned i;
    if (!detail || !name || !*name || !gw_prof_enabled()) return;
    prof_lock();
    for (i = 0; i < prof.label_count; ++i) if (prof.labels[i].detail == detail) break;
    if (i == PROF_DETAILS) { prof.label_omitted++; prof_unlock(); return; }
    if (i == prof.label_count) {
        prof.labels[i].detail = detail;
        snprintf(prof.labels[i].name, sizeof prof.labels[i].name, "%s", name);
        prof.label_count++;
    }
    prof_unlock();
}
unsigned gw_prof_mark(void) {
    if (!gw_prof_enabled()) return prof_load(&prof_sum) ? sum_mark() : 0;
    prof_tls_sync();
    return prof_tls.depth + prof_tls.overflow;
}
void gw_prof_unwind(unsigned mark) {
    if (gw_prof_enabled()) {
        prof_tls_sync();
        while (prof_tls.depth + prof_tls.overflow > mark) gw_prof_end();
    }
    if (prof_load(&prof_sum)) sum_unwind(mark);
}
static void prof_json_string(FILE *f, const char *s) {
    fputc('"', f);
    for (; *s; ++s) {
        unsigned char c = (unsigned char)*s;
        if (c == '"' || c == '\\') { fputc('\\', f); fputc(c, f); }
        else if (c < 32) fprintf(f, "\\u%04x", c);
        else fputc(c, f);
    }
    fputc('"', f);
}
int gw_prof_report(const char *path) {
    FILE *f;
    unsigned id, comma;
    GwProfStats s;
    unsigned long long gpu_dropped_zones = 0, gpu_dropped_frames = 0;
#if defined(_WIN32) && !defined(GW_TEST_STANDALONE)
    gpu_dropped_zones = aurora_profiler_gpu_dropped_zones();
    gpu_dropped_frames = aurora_profiler_gpu_dropped_frames();
#endif
    gw_prof_init();
    f = fopen(path && *path ? path : prof.report_path, "wb");
    if (!f) return 0;
    fprintf(f, "{\"schema_version\":1,\"frames\":%llu,\"enabled\":%s,\"budget_ms\":8.333333333,"
            "\"sample_window\":4096,\"detail_sample_window\":256,\"percentile_scope\":\"deterministic_run_reservoir\","
            "\"percentile_method\":\"nearest_rank\",\"zone_units\":\"ms\",\"zones\":{",
            gw_prof_frames(), gw_prof_enabled() ? "true" : "false");
    for (id = comma = 0; id < GW_PROF_ZONE_COUNT; ++id) {
        gw_prof_stats(id, &s);
        if (!s.count) continue;
        fprintf(f, "%s\"%s\":", comma++ ? "," : "", gw_prof_name(id)); prof_write_stats(f, s);
    }
    fprintf(f, "},\"counters\":{");
    for (id = GW_PROF_MEX_INSTRUCTIONS, comma = 0; id < GW_PROF_COUNTER_END; ++id) {
        gw_prof_stats(id, &s);
        if (!s.count) continue;
        fprintf(f, "%s\"%s\":", comma++ ? "," : "", gw_prof_name(id)); prof_write_stats(f, s);
    }
    prof_lock();
    fprintf(f, "},\"gpu\":{\"available\":%s,\"source\":\"external_timestamp_query\"},\"details\":[",
            prof.gpu_available ? "true" : "false");
    for (id = 0; id < prof.detail_count; ++id) {
        ProfDetail *d = &prof.details[id];
        double values[PROF_DETAIL_SAMPLES];
        unsigned n = d->count < PROF_DETAIL_SAMPLES ? (unsigned)d->count : PROF_DETAIL_SAMPLES;
        memcpy(values, d->sample, n * sizeof *values);
        qsort(values, n, sizeof *values, prof_compare);
        fprintf(f, "%s{\"zone\":\"%s\",\"detail\":%u,\"count\":%llu,\"mean\":%.9g,\"max\":%.9g,"
                "\"p50\":%.9g,\"p95\":%.9g,\"p99\":%.9g}",
                id ? "," : "", gw_prof_name(d->id), d->detail, (unsigned long long)d->count,
                d->sum / d->count, d->max, prof_percentile(values,n,.50),
                prof_percentile(values,n,.95),prof_percentile(values,n,.99));
    }
    fprintf(f, "],\"detail_names\":{");
    for (id = 0; id < prof.label_count; ++id) {
        fprintf(f, "%s\"%u\":", id ? "," : "", prof.labels[id].detail);
        prof_json_string(f, prof.labels[id].name);
    }
    fprintf(f, "},\"diagnostics\":{\"stack_overflow\":%llu,\"unmatched_end\":%llu,\"invalid_id\":%llu,"
            "\"event_overwrites\":%llu,\"event_sampled\":%llu,\"detail_aggregated\":%llu,\"label_omitted\":%llu,\"detail_overflow\":%llu,\"late_events\":%llu,\"dropped_realtime_samples\":%u,"
            "\"gpu_dropped_zones\":%llu,\"gpu_dropped_frames\":%llu,\"tracy\":%s}}\n",
            (unsigned long long)prof.stack_overflow, (unsigned long long)prof.unmatched_end,
            (unsigned long long)prof.invalid_id,
            (unsigned long long)(prof.event_count > PROF_EVENTS ? prof.event_count - PROF_EVENTS : 0),
            (unsigned long long)prof.event_sampled, (unsigned long long)prof.detail_aggregated, (unsigned long long)prof.label_omitted,
            (unsigned long long)prof.detail_overflow, (unsigned long long)prof.late_events,
            prof_load(&prof_dropped),
            gpu_dropped_zones, gpu_dropped_frames,
#ifdef GW_PROF_TRACY
            "true,\"development_marker\":\"GD_MELEE_TRACY_DEVELOPMENT_ONLY\""
#else
            "false"
#endif
            );
    prof_unlock();
    { int ok = !ferror(f); if (fclose(f)) ok = 0; return ok; }
}
int gw_prof_trace(const char *path, unsigned frames) {
    ProfEvent *copy;
    ProfLabel labels[PROF_DETAILS];
    FILE *f;
    uint64_t first, last, i;
    unsigned current, min_frame, count = 0, label_count;
    uint64_t sampled;
    double epoch;
    gw_prof_init();
    if (!frames) frames = PROF_HISTORY;
    if (frames > PROF_HISTORY) frames = PROF_HISTORY;
    copy = malloc(sizeof prof.events); /* Explicit output, never a disabled/enabled zone hot path. */
    if (!copy) return 0;
    prof_lock();
    current = prof_load(&prof_frame);
    min_frame = current >= frames ? current - frames + 1 : 0;
    last = prof.event_count;
    sampled = prof.event_sampled;
    first = last > PROF_EVENTS ? last - PROF_EVENTS : 0;
    epoch = prof.epoch;
    label_count = prof.label_count;
    memcpy(labels, prof.labels, label_count * sizeof *labels);
    for (i = first; i < last; ++i) {
        ProfEvent e = prof.events[i % PROF_EVENTS];
        if (e.frame >= min_frame && e.frame <= current) copy[count++] = e;
    }
    prof_unlock();
    f = fopen(path && *path ? path : prof.trace_path, "wb");
    if (!f) { free(copy); return 0; }
    fprintf(f, "{\"displayTimeUnit\":\"ms\",\"first_frame\":%u,\"last_frame\":%u,"
            "\"event_capacity\":%u,\"event_overwrites\":%llu,\"event_sampled\":%llu,\"small_event_limit_per_id_frame\":8,\"always_retain_ms\":0.25,\"frame_source\":\"content_frame\",\"traceEvents\":[",
            min_frame, current, PROF_EVENTS, (unsigned long long)(last > PROF_EVENTS ? last - PROF_EVENTS : 0), (unsigned long long)sampled);
    for (i = 0; i < count; ++i) {
        ProfEvent e = copy[i];
        unsigned j;
        const char *label = NULL;
        char event_name[256];
        for (j = 0; j < label_count; ++j) if (labels[j].detail == e.detail) { label = labels[j].name; break; }
        snprintf(event_name, sizeof event_name, "%s%s%s", gw_prof_name(e.id),
                 e.phase == 'G' && label ? "/" : "", e.phase == 'G' && label ? label : "");
        fprintf(f, "%s{\"name\":", i ? "," : "");
        prof_json_string(f, event_name);
        fprintf(f, ",\"cat\":\"melee\",\"ph\":\"%c\",\"pid\":1,\"tid\":%u,\"ts\":%.6f,",
                e.phase == 'A' ? 'X' : e.phase == 'G' ? 'C' : e.phase,
                e.thread, (e.start - epoch) * 1000.0);
        if (e.phase == 'X' || e.phase == 'A') fprintf(f, "\"dur\":%.6f,\"args\":{\"frame\":%u,\"detail\":%u,"
                "\"timestamp_source\":\"%s\"", e.duration * 1000.0, e.frame, e.detail,
                e.phase == 'A' ? "callback_arrival" : "zone_clock");
        else if (e.phase == 'G') fprintf(f, "\"id\":%u,\"args\":{\"duration_ms\":%.9g,\"detail\":%u,"
                "\"render_frame\":%llu,\"frame\":%u,\"timestamp_source\":\"callback_arrival\"",
                e.detail, e.value, e.detail, e.render_frame, e.frame);
        else fprintf(f, "\"args\":{\"value\":%.9g,\"detail\":%u", e.value, e.detail);
        { static const char *keys[] = {"last_script_call", "last_spawn", "last_area"};
          for (j = 0; j < 3; ++j) if (e.context[j][0]) {
              fprintf(f, ",\"%s\":", keys[j]); prof_json_string(f, e.context[j]);
          } }
        if (label) { fprintf(f, ",\"detail_name\":"); prof_json_string(f, label); }
        fprintf(f, "}}");
    }
    fprintf(f, "]}\n");
    free(copy);
    { int ok = !ferror(f); if (fclose(f)) ok = 0; return ok; }
}
void gw_prof_set_hitch(double milliseconds) {
    gw_prof_init();
    if (!isfinite(milliseconds) || milliseconds <= 0) return;
    prof_lock(); prof.hitch_ms = milliseconds; prof_unlock();
}
int gw_prof_command(const char *args, char *out, unsigned cap) {
    unsigned frames;
    double ms;
    int ok = 1;
    if (!strcmp(args, "on")) gw_prof_set_enabled(1);
    else if (!strcmp(args, "off")) gw_prof_set_enabled(0);
    else if (!strcmp(args, "reset")) gw_prof_reset();
    else if (!strcmp(args, "report")) ok = gw_prof_report(NULL);
    else if (!strcmp(args, "perf")) { ok = gw_prof_perf_write(NULL); gw_prof_perf_status(out, cap); return ok; }
    else if (!strcmp(args, "summary on")) gw_prof_summary_set(1);
    else if (!strcmp(args, "summary off")) gw_prof_summary_set(0);
    else if (sscanf(args, "trace %u", &frames) == 1) ok = gw_prof_trace(NULL, frames);
    else if (sscanf(args, "hitch %lf", &ms) == 1 && isfinite(ms) && ms > 0) gw_prof_set_hitch(ms);
    else { snprintf(out, cap, "prof on|off|reset|report|perf|summary on|off|trace <frames>|hitch <ms>"); return 0; }
    snprintf(out, cap, "prof %s: %s", args, ok ? "ok" : "output failed (check path)");
    return ok;
}
void gw_prof_shutdown(void) {
    const char *trace;
    unsigned pending;
    if (!prof.initialized) return;
    prof_lock(); pending = prof.hitch_pending; prof.hitch_pending = 0; prof_unlock();
    if (pending) {
        char path[1200];
        snprintf(path, sizeof path, "%s.hitch-%u.json", prof.trace_path, pending);
        gw_prof_trace(path, 5); /* End-of-run hitch retains available neighbours only. */
    }
    if (prof_load(&prof_sum)) { gw_prof_scene_end(); sum_write_file(0); }
    if (prof.completed || gw_prof_enabled()) gw_prof_report(NULL);
    trace = getenv("MELEE_PROF_TRACE");
    if (trace && *trace) gw_prof_trace(NULL, PROF_HISTORY);
    gw_prof_set_enabled(0);
    sum_set(0);
}
void gw_ProfBegin(int id, int detail) { gw_prof_begin((unsigned)id, (unsigned)detail); }
void gw_ProfEnd(void) { gw_prof_end(); }
void gw_ProfCounter(int id, int value) { gw_prof_counter((unsigned)id, value); }
void gw_ProfCounterDetail(int id, int detail, int value) { gw_prof_counter_detail((unsigned)id,(unsigned)detail,value); }
int gw_ProfEnabled(void) { return gw_prof_enabled(); }
