#include "gw_test.h"

#include <setjmp.h>
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <windows.h>

#include "gw.h"

#define GW_TEST_MAX 256
#define GW_TEST_MSG_MAX 256

static struct {
  const char *name;
  gw_test_fn fn;
} gw_tests[GW_TEST_MAX];
static int gw_test_count;

static char gw_test_msg[GW_TEST_MSG_MAX];
static int gw_test_msg_used;

void gw_test_register(const char *name, gw_test_fn fn) {
  if (gw_test_count >= GW_TEST_MAX) {
    return;
  }
  gw_tests[gw_test_count].name = name;
  gw_tests[gw_test_count].fn = fn;
  gw_test_count++;
}

void gw_test_fail(const char *fmt, ...) {
  va_list ap;
  if (gw_test_msg_used) {
    return;
  }
  gw_test_msg_used = 1;
  va_start(ap, fmt);
  vsnprintf(gw_test_msg, sizeof gw_test_msg, fmt, ap);
  va_end(ap);
}

bool gw_test_requested(int argc, char **argv) {  int i;
  for (i = 1; i < argc; ++i) {
    if (strcmp(argv[i], "--test") == 0) {
      return true;
    }
  }
  return false;
}

static unsigned char *gw_test_mem_snapshot;

static void gw_test_isolate_begin(void) {
  if (gw_mem1 == NULL || gw_mem1_size == 0) {
    return;
  }
  if (gw_test_mem_snapshot == NULL) {
    gw_test_mem_snapshot = (unsigned char *)malloc(gw_mem1_size);
  }
  if (gw_test_mem_snapshot != NULL) {
    memcpy(gw_test_mem_snapshot, gw_mem1, gw_mem1_size);
  }
}

/* Restoring MEM1 puts the link-time pointers back to their pre-fixup values, so the fixups have to
 * run again or every game global points at its unrelocated self. Platform-side statics are not in
 * MEM1 and are deliberately not restored: a test must set up any platform state it depends on. */
static void gw_test_isolate_end(void) {
  if (gw_test_mem_snapshot == NULL) {
    return;
  }
  memcpy(gw_mem1, gw_test_mem_snapshot, gw_mem1_size);
  gw_apply_fixups();
}

/* Bridge for game-side tests. gwtool prefixes every game symbol, so a game translation unit that
 * calls `TestRegister`/`TestFail` references these names. The message is a single string rather
 * than varargs to keep the retargeted calling convention trivial. */
void gw_TestRegister(const char *name, int (*fn)(void)) { gw_test_register(name, fn); }

void gw_TestFail(const char *msg) { gw_test_fail("%s", msg); }

static jmp_buf gw_test_jmp;
static int gw_test_running;

/* gw_panic routes here while a test is running, so an assert fired inside game code fails the
 * test instead of aborting the process. longjmp is acceptable because the next test starts from a
 * restored MEM1 snapshot regardless. */
int gw_test_active(void) { return gw_test_running; }

void gw_test_panic_hit(const char *msg) {
  if (!gw_test_running) {
    return;
  }
  gw_test_fail("panic: %s", msg != NULL ? msg : "(none)");
  longjmp(gw_test_jmp, 1);
}

/* A structured exception (bad dereference, divide by zero) is a test failure, not a dead run. */
static int gw_test_invoke(gw_test_fn fn) {
  __try {
    return fn();
  } __except (EXCEPTION_EXECUTE_HANDLER) {
    gw_test_fail("test raised a structured exception");
    return 1;
  }
}

/* Per-test timeout. SEH contains faults but cannot interrupt an infinite loop, so without this a
 * hung test freezes the run with no diagnostic. A worker thread watches the current test's
 * deadline and, on expiry, names the test and exits the process with a distinctive code so CI
 * sees a bounded timeout rather than an unbounded hang. The hung thread itself is not killed:
 * there is no safe way to do that in-process. */

static volatile LONG gw_test_index = -1;
static volatile DWORD gw_test_deadline;
static DWORD gw_test_timeout_ms = 30000;

static DWORD WINAPI gw_test_watchdog(LPVOID unused) {
  (void)unused;
  for (;;) {
    Sleep(250);
    if (gw_test_index >= 0 && gw_test_deadline != 0 &&
        (LONG)(GetTickCount() - gw_test_deadline) > 0) {
      gw_log("TESTS: TIMEOUT in \"%s\" after %lu ms", gw_tests[gw_test_index].name,
             (unsigned long)gw_test_timeout_ms);
      ExitProcess(2);
    }
  }
  return 0;
}

int gw_test_run_all(void) {
  int i;
  int pass = 0;
  int fail = 0;
  {
    extern void gw_tests_register_all(void);
    gw_tests_register_all();
  }
  gw_log("TESTS: begin (%d registered)", gw_test_count);
  {
    const char *v = getenv("MELEE_TEST_TIMEOUT");
    if (v != NULL) {
      const long secs = strtol(v, NULL, 0);
      if (secs > 0) {
        gw_test_timeout_ms = (DWORD)(secs * 1000);
      }
    }
    CreateThread(NULL, 0, gw_test_watchdog, NULL, 0, NULL);
  }
  for (i = 0; i < gw_test_count; ++i) {
    int rc;
    gw_test_msg[0] = '\0';
    gw_test_msg_used = 0;
    gw_test_isolate_begin();
    gw_test_running = 1;
    gw_test_index = i;
    gw_test_deadline = GetTickCount() + gw_test_timeout_ms;
    if (setjmp(gw_test_jmp) == 0) {
      rc = gw_test_invoke(gw_tests[i].fn);
    } else {
      rc = 1;
    }
    gw_test_index = -1;
    gw_test_running = 0;
    gw_test_isolate_end();
    if (rc == 0) {
      pass++;
      gw_log("ok %d - %s", i + 1, gw_tests[i].name);
    } else {
      fail++;
      gw_log("not ok %d - %s", i + 1, gw_tests[i].name);
      gw_log("  %s", gw_test_msg_used ? gw_test_msg : "(no reason recorded)");
    }
  }
  gw_log("TESTS: pass=%d fail=%d total=%d", pass, fail, gw_test_count);
  return fail;
}
