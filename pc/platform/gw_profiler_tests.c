/* In-engine suite tests use only the public native diagnostics API. */
#include "gw_profiler.h"
#include "gw_test.h"
#include <stdio.h>
#include <string.h>
static int profiler_percentiles(void) {
    GwProfStats s;
    int enabled = gw_prof_enabled(), i, failed;
    gw_prof_set_enabled(1); gw_prof_reset();
    for (i = 1; i <= 100; ++i) gw_prof_sample(GW_PROF_GPU, i, 1);
    gw_prof_stats(GW_PROF_GPU, &s);
    failed = s.count != 100 || s.mean != 50.5 || s.p50 != 50 || s.p95 != 95 || s.p99 != 99 || s.max != 100;
    gw_prof_reset(); gw_prof_set_enabled(enabled);
    if (failed) gw_test_fail("nearest-rank GPU samples differ from 1..100 expected distribution");
    return failed;
}
static int profiler_nested_overflow(void) {
    GwProfStats s;
    int enabled = gw_prof_enabled(), i, failed;
    gw_prof_set_enabled(1); gw_prof_reset();
    for (i = 0; i < 73; ++i) gw_prof_begin(GW_PROF_FIGHTER, 1);
    for (i = 0; i < 73; ++i) gw_prof_end();
    gw_prof_stats(GW_PROF_FIGHTER, &s);
    failed = s.count != 64 || s.max < s.p99 || s.p99 < s.p95 || s.p95 < s.p50;
    gw_prof_begin(GW_PROF_INPUT, 0); gw_prof_end();
    gw_prof_stats(GW_PROF_INPUT, &s);
    failed |= s.count != 1;
    gw_prof_reset(); gw_prof_set_enabled(enabled);
    if (failed) gw_test_fail("bounded nested zones failed to unwind after depth overflow");
    return failed;
}
static int profiler_disabled_storage(void) {
    GwProfStats s;
    int enabled = gw_prof_enabled(), i, failed;
    gw_prof_set_enabled(0); gw_prof_reset();
    for (i = 0; i < 10000; ++i) {
        gw_prof_begin(GW_PROF_LOGIC, 0); gw_prof_end();
        gw_prof_sample(GW_PROF_GPU, 1, 0); gw_prof_counter(GW_PROF_VERTICES, 1);
        gw_prof_frame_begin(); gw_prof_frame_end();
    }
    gw_prof_stats(GW_PROF_LOGIC, &s); failed = s.count != 0;
    gw_prof_stats(GW_PROF_GPU, &s); failed |= s.count != 0;
    gw_prof_stats(GW_PROF_VERTICES, &s); failed |= s.count != 0;
    failed |= gw_prof_frames() != 0;
    gw_prof_set_enabled(enabled);
    if (failed) gw_test_fail("disabled diagnostics wrote measurements");
    return failed;
}
/* The perf record's accounting, headless: synthetic frames with known draw counts and zone times go in,
 * the scene's numbers come out. This is all a headless run can say about performance: it has no GPU, no
 * paced loop and no render, so it cannot time anything or count a real draw. What it CAN pin is that the
 * machinery that bench.sh / run.sh judge with is arithmetically right (means, maxima, hitch counting,
 * first-use vs steady split, the per-frame counters), and that a draw-call explosion (1,300+ draws for one
 * fighter, against ~100 for a retail one) is visible to the draw ceiling the judge applies.
 * Real draw counts come from a real scene: tools/port/bench.sh retail2 (ceiling in tools/port/perfjudge.py). */
static int perf_summary_accounting(void) {
    double mean = 0; unsigned max = 0; unsigned long long frames = 0, with = 0;
    int was_sum = gw_prof_summary_enabled(), i, failed = 0;
    char path[] = "perf-suite-test.json", status[200];
    gw_prof_summary_set(1);
    gw_prof_scene_begin("suite/synthetic");
    gw_prof_frame_begin();
    for (i = 0; i < 700; ++i) { /* 600 first-use frames, 100 steady */
        gw_prof_begin(GW_PROF_LOGIC, 0);
        gw_prof_tally(GW_PROF_ENV_SETUPS); gw_prof_tally(GW_PROF_ENV_SETUPS);
        gw_prof_end();
        gw_prof_frame_stats(i % 2 ? 100u : 300u, 5u, 7u, 1, (unsigned)i + 1u, 0.5);
        gw_prof_frame_end();
        gw_prof_frame_begin();
    }
    failed |= !gw_prof_scene_draws(&mean, &max, &frames, &with);
    failed |= frames != 700 || with != 700 || max != 300 || mean < 199.9 || mean > 200.1;
    failed |= !gw_prof_perf_write(path);
    gw_prof_perf_status(status, sizeof status);
    failed |= strstr(status, "frames=700") == NULL;
    gw_prof_scene_end();
    gw_prof_summary_set(was_sum);
    remove(path);
    if (failed) gw_test_fail("perf summary accounting: frames %llu draws mean %.1f max %u", frames, mean, max);
    return failed;
}
void gw_profiler_tests_register(void) {
    gw_test_register("perf_summary_accounting", perf_summary_accounting);
    gw_test_register("profiler_nearest_rank", profiler_percentiles);
    gw_test_register("profiler_nesting_overflow", profiler_nested_overflow);
    gw_test_register("profiler_disabled_storage", profiler_disabled_storage);
}
