/* Standalone native accounting fixture. Do not link into game (includes implementation).
 * Integrator compiles this test separately; no disc, Aurora or game launch required. */
#include <assert.h>
#include <stdio.h>
#include <stdbool.h>
#include <stdlib.h>
#ifdef _WIN32
#include <windows.h>
static unsigned fixture_clock_calls;
static BOOL fixture_clock(LARGE_INTEGER *out) { fixture_clock_calls++; return QueryPerformanceCounter(out); }
#define QueryPerformanceCounter fixture_clock
#else
#include <time.h>
static unsigned fixture_clock_calls;
static int fixture_clock(clockid_t id, struct timespec *out) { fixture_clock_calls++; return clock_gettime(id,out); }
#define clock_gettime fixture_clock
#endif
static unsigned fixture_allocations;
static void *fixture_malloc(size_t n) { fixture_allocations++; return malloc(n); }
#define malloc fixture_malloc
void aurora_profiler_enable(bool enabled) { (void)enabled; }
#define GW_TEST_STANDALONE 1
#include "../platform/gw_profiler.c"
#ifdef _WIN32
static DWORD WINAPI worker(void *unused) {
#else
static void *worker(void *unused) {
#endif
    (void)unused;
    gw_prof_begin(GW_PROF_RENDER, 77);
    gw_prof_begin(GW_PROF_PRESENT, 77);
    gw_prof_end(); gw_prof_end();
    return 0;
}
int main(void) {
    GwProfStats s, child;
    unsigned i;
    unsigned clocks, allocations;
    double one[] = {5};
    double ranks[] = {1,2,3,4,5,6,7,8,9,10};
    assert(prof_percentile(one, 1, .99) == 5);
    assert(prof_percentile(ranks, 10, .5) == 5);
    assert(prof_percentile(ranks, 10, .95) == 10);
    assert(prof_percentile(ranks, 0, .99) == 0);
#ifdef _WIN32
    _putenv_s("MELEE_PROF_TRACE", ""); _putenv_s("MELEE_PROF_REPORT", "");
#else
    unsetenv("MELEE_PROF_TRACE"); unsetenv("MELEE_PROF_REPORT");
#endif
    gw_prof_init(); gw_prof_set_enabled(0); gw_prof_reset();
    clocks = fixture_clock_calls; allocations = fixture_allocations;
    for (i = 0; i < 100000; ++i) {
        gw_prof_begin(GW_PROF_LOGIC, 0); gw_prof_end();
        gw_prof_counter(GW_PROF_VERTICES, 10); gw_prof_sample(GW_PROF_GPU, 1, 0);
        gw_prof_counter_detail(GW_PROF_MEX_INSTRUCTIONS, 1, 10);
        gw_prof_cpu_completed(GW_PROF_RENDER, 0, 1);
        gw_prof_try_cpu_completed(GW_PROF_AUDIO, 0, 1);
        gw_prof_gpu_completed(GW_PROF_GPU, 0, 1, 1);
        gw_prof_frame_begin(); gw_prof_frame_end();
        assert(gw_prof_clock_ms() == 0);
    }
    assert(prof.event_count == 0 && prof_tls.depth == 0);
    assert(fixture_clock_calls == clocks && fixture_allocations == allocations);
    gw_prof_set_enabled(1);
    gw_prof_begin(GW_PROF_LOGIC, 1);
    gw_prof_begin(GW_PROF_INPUT, 1);
    gw_prof_end(); gw_prof_end();
    assert(gw_prof_stats(GW_PROF_LOGIC, &s) && s.count == 1);
    assert(gw_prof_stats(GW_PROF_INPUT, &child) && child.count == 1 && s.max >= child.max);
#ifdef _WIN32
    { HANDLE h = CreateThread(NULL, 0, worker, NULL, 0, NULL);
      assert(h); WaitForSingleObject(h, INFINITE); CloseHandle(h); }
#else
    { pthread_t t; assert(!pthread_create(&t,NULL,worker,NULL)); pthread_join(t,NULL); }
#endif
    assert(gw_prof_stats(GW_PROF_RENDER, &s) && s.count == 1);
    assert(prof_tls.depth == 0);
    for (i = 0; i < PROF_DEPTH + 9; ++i) gw_prof_begin(GW_PROF_FIGHTER, i);
    for (i = 0; i < PROF_DEPTH + 9; ++i) gw_prof_end();
    assert(prof.stack_overflow == 9 && prof_tls.depth == 0 && prof_tls.overflow == 0);
    gw_prof_end(); assert(prof.unmatched_end == 1);
    gw_prof_begin(999,0); gw_prof_end(); assert(prof.invalid_id == 1);
    gw_prof_begin(GW_PROF_LOGIC,0); gw_prof_reset(); gw_prof_end();
    assert(prof_tls.depth == 0 && prof.unmatched_end == 1);
    gw_prof_reset();
    prof_lock();
    gw_prof_try_cpu_completed(GW_PROF_AUDIO, 0, 1);
    prof_unlock();
    assert(prof_load(&prof_dropped) == 1);
    gw_prof_stats(GW_PROF_AUDIO, &s); assert(s.count == 0);
    gw_prof_try_cpu_completed(GW_PROF_AUDIO, 0, 1);
    gw_prof_stats(GW_PROF_AUDIO, &s); assert(s.count == 1);
    for (i = 1; i <= 100; ++i) gw_prof_sample(GW_PROF_GPU, (double)i, 99);
    assert(gw_prof_stats(GW_PROF_GPU, &s) && s.p50 == 50 && s.p95 == 95 && s.p99 == 99 && s.max == 100);
    assert(prof.details[0].count == 100 && prof.gpu_available);
    gw_prof_counter_detail(GW_PROF_MEX_INSTRUCTIONS, 0x81234567, 123);
    assert(prof.details[1].id == GW_PROF_MEX_INSTRUCTIONS && prof.details[1].detail == 0x81234567);
    gw_prof_gpu_completed(GW_PROF_GPU_PASS, 99, 2.5, 42);
    assert(prof.events[(prof.event_count-1)%PROF_EVENTS].render_frame == 42);
    assert(prof.events[(prof.event_count-1)%PROF_EVENTS].phase == 'G');
    gw_prof_detail_name(99, "gpu pass test");
    assert(gw_prof_trace("profiler-test-gpu.json", 3));
    gw_prof_reset();
    for (i = 0; i < 20000; ++i) gw_prof_sample(GW_PROF_GPU, i >= 10000 ? 1 : 0, 99);
    gw_prof_stats(GW_PROF_GPU, &s);
    assert(s.count == 20000 && s.mean == .5 && s.max == 1 && s.p95 == 1);
    /* A last-window buffer would contain only ones; a whole-run reservoir retains both eras. */
    { unsigned zeros = 0; for (i = 0; i < PROF_SAMPLES; ++i) zeros += prof.series[GW_PROF_GPU].sample[i] == 0;
      assert(zeros > 1000 && zeros < 3000); }
    gw_prof_reset();
    for (i = 0; i < PROF_EVENTS + 7; ++i) gw_prof_counter(GW_PROF_VERTICES, i);
    assert(prof.event_count == 8);
    assert(prof.event_sampled == PROF_EVENTS - 1);
    gw_prof_stats(GW_PROF_VERTICES, &s); assert(s.count == PROF_EVENTS + 7);
    gw_prof_context(0, "stage_add_line");
    gw_prof_context(1, "Goomba");
    gw_prof_context(2, "maze-room-7");
    gw_prof_cpu_completed(GW_PROF_CHUNK_LOAD, 0, 12);
    assert(!strcmp(prof.events[8].context[0], "stage_add_line"));
    assert(!strcmp(prof.events[8].context[1], "Goomba"));
    assert(!strcmp(prof.events[8].context[2], "maze-room-7"));
    for (i = 1; i <= PROF_DETAILS + 100; ++i) gw_prof_sample(GW_PROF_LUA_API, 1, i);
    { unsigned j; uint64_t total = 0;
      for (j = 0; j < prof.detail_count; ++j)
        if (prof.details[j].id == GW_PROF_LUA_API) total += prof.details[j].count;
      assert(total == PROF_DETAILS + 100 && prof.detail_overflow == 0);
      assert(prof.detail_aggregated > 0); }
    /* Restore schema samples after the independent ring-overwrite test. */
    for (i = 1; i <= 100; ++i) gw_prof_sample(GW_PROF_GPU, (double)i, 99);
    gw_prof_detail_name(99, "test \"gpu\"\npass");
    gw_prof_detail_name(100, "test \"gpu\"\npass");
    assert(gw_prof_report("profiler-test-report.json"));
    assert(gw_prof_trace("profiler-test-trace.json", 3));
    gw_prof_reset();
    /* A maze-like 840k tiny callbacks must retain all 120 frame neighbourhoods. */
    for (i = 1; i <= PROF_HISTORY; ++i) {
        unsigned j;
        for (j = 0; j < 7000; ++j)
            prof_event((ProfEvent){0, .01, 0, GW_PROF_OBJECT_CALLBACK, j, 1, i, 'X'});
        prof_event((ProfEvent){0, 12, 0, GW_PROF_CHUNK_LOAD, 0, 1, i, 'X'});
    }
    assert(prof.event_count == PROF_HISTORY * 9);
    assert(prof.events[0].frame == 1 && prof.events[prof.event_count-1].frame == PROF_HISTORY);
    gw_prof_reset();
    gw_prof_set_hitch(.000001);
    for (i = 0; i < 4; ++i) {
        gw_prof_frame_begin();
        gw_prof_begin(GW_PROF_LOGIC,0); gw_prof_end();
        gw_prof_frame_end();
    }
    assert(prof.completed == 4 && prof.hitch_last == 3 && !prof.hitch_pending);
    assert(gw_prof_trace("profiler-test-frames.json", 2));
    gw_prof_set_enabled(0);
    puts("profiler core: PASS");
    return 0;
}
