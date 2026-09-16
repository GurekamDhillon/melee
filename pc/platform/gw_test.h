#ifndef GW_TEST_H
#define GW_TEST_H

#include <stdbool.h>

/* In-engine test runner.
 *
 * Tests link the real gwtool-retargeted game objects and run in real guest memory, so they
 * exercise the same program that ships -- unlike a native x86 build, which would be
 * little-endian and test a different program. See _research/engine-test-suite-plan.md.
 *
 * A test returns 0 for pass, non-zero for fail, and may call gw_test_fail() to record why. */

typedef int (*gw_test_fn)(void);

/* Registers a test. Call from a registry translation unit at load time; the runner does not
 * allocate, so the registry is a fixed array and silently drops overflow. */
void gw_test_register(const char *name, gw_test_fn fn);

/* Records a failure reason for the currently running test. Only the first call is kept. */
void gw_test_fail(const char *fmt, ...);

/* Runs every registered test, prints one TAP-style line each plus a greppable summary, and
 * returns the number of failures (0 = all passed). */
int gw_test_run_all(void);

/* True when argv requests test mode ("--test"). */
bool gw_test_requested(int argc, char **argv);

#endif
