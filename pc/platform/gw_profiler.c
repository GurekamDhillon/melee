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
#include <windows.h>
#include <intrin.h>
/* Aurora owns timestamp availability; standalone tests supply this output hook. */
extern void aurora_profiler_enable(bool enabled);
#ifndef GW_TEST_STANDALONE
extern uint64_t aurora_profiler_gpu_dropped_zones(void);
extern uint64_t aurora_profiler_gpu_dropped_frames(void);
#endif
static SRWLOCK prof_mutex = SRWLOCK_INIT;
static volatile LONG prof_on, prof_generation = 1, prof_frame, prof_dropped;
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
    [GW_PROF_MEX_INSTRUCTIONS] = "mex_instructions", [GW_PROF_ROLLBACK_FRAMES] = "rollback_frames",
    [GW_PROF_SNAPSHOT_BYTES] = "snapshot_bytes", [GW_PROF_HEAP_FREE] = "heap_free",
    [GW_PROF_HEAP_USED] = "heap_used", [GW_PROF_DRAW_CALLS] = "draw_calls",
    [GW_PROF_VERTICES] = "vertices", [GW_PROF_PARTICLE_COUNT] = "particle_count",
    [GW_PROF_HEAP0_FREE] = "heap0_free", [GW_PROF_HEAP3_FREE] = "heap3_free",
    [GW_PROF_HEAP4_FREE] = "heap4_free", [GW_PROF_HEAP5_FREE] = "heap5_free",
    [GW_PROF_PIPELINE_SKIPS] = "pipeline_skips", [GW_PROF_PIPELINE_WAITS] = "pipeline_waits"
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
    aurora_profiler_enable(enabled != 0);
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
    if (!gw_prof_enabled()) return 0;
    prof_tls_sync();
    return prof_tls.depth + prof_tls.overflow;
}
void gw_prof_unwind(unsigned mark) {
    if (!gw_prof_enabled()) return;
    prof_tls_sync();
    while (prof_tls.depth + prof_tls.overflow > mark) gw_prof_end();
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
    else if (sscanf(args, "trace %u", &frames) == 1) ok = gw_prof_trace(NULL, frames);
    else if (sscanf(args, "hitch %lf", &ms) == 1 && isfinite(ms) && ms > 0) gw_prof_set_hitch(ms);
    else { snprintf(out, cap, "prof on|off|reset|report|trace <frames>|hitch <ms>"); return 0; }
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
    if (prof.completed || gw_prof_enabled()) gw_prof_report(NULL);
    trace = getenv("MELEE_PROF_TRACE");
    if (trace && *trace) gw_prof_trace(NULL, PROF_HISTORY);
    gw_prof_set_enabled(0);
}
void gw_ProfBegin(int id, int detail) { gw_prof_begin((unsigned)id, (unsigned)detail); }
void gw_ProfEnd(void) { gw_prof_end(); }
void gw_ProfCounter(int id, int value) { gw_prof_counter((unsigned)id, value); }
void gw_ProfCounterDetail(int id, int detail, int value) { gw_prof_counter_detail((unsigned)id,(unsigned)detail,value); }
int gw_ProfEnabled(void) { return gw_prof_enabled(); }
