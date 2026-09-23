/* gw_log.c - melee-pc.log: log categories, the OSReport line assembler, and the compact crash
 * report (crashlogs/crash-<time>.log).
 *
 * CATEGORIES. Every line is sorted by its prefix into one category. Most lines are "core" and are
 * always written. A handful of sources used to drown the log (0.1.0 player logs were mostly
 * these), so their chatty lines are quiet by default:
 *   render    gw: DIAG blocks (proj/posmtx/copy/depth grid/camdesc) - by default only the FIRST
 *             block after each scene change (still enough to tell a black screen's cause)
 *   watchdog  the sampler's "gw: at <where>" moves and its 2 s stats line (the last 8 samples
 *             still go into a crash report; "still at ... spinning" is always written)
 *   tex       gxtex: <file> -> handle N (every UI texture open)
 *   frontend  frontend: <file>.json: N bytes, N nodes
 *   mex       m-ex internals: interp: thunk/trap/on*, ftfunction:/itfunction: slots and relocs,
 *             grfunction: StageData words, Kirby hat table, effect banks, item registration
 *             ("interp: installed ..." and "grfunction: ... installed" - the loads - stay core)
 *   heap      lbHeap:/heap: census lines and ARAlloc (MELEE_HEAP_TRACE)
 *   dvd       gw: dvd: <path> (MELEE_DVD_TRACE) and the per-file lines of a mounting mod
 *   audio     synth: bank loads, lbAudioAx: m-ex sfx -> voice
 *   snap      snap: render pass ObjAlloc/ObjFree
 * (the once-per-scene render block is the periodic proj/posmtx/copy lines and their indented
 * report; the one-off vtxattrfmt/mtxidx setup lines count as the full "render" category)
 * and four that are ON by default: scene (scene: ...), pad (adapter/input/pad diagnostics - the
 * pad lane decides what it prints, MELEE_PAD_DIAG), net (netplay:, rb:), core (everything else).
 * A quiet line that looks like a problem (fail/error/cannot/missing/BAD/...) is written anyway.
 *
 * SWITCHES (the environment wins over settings.cfg, as everywhere else in the port):
 *   MELEE_LOG=<list>   or   settings.cfg  log=<list>
 *     tokens separated by , ; or space:  all | quiet | default | <category> | -<category>
 *     e.g. MELEE_LOG=mex,heap   MELEE_LOG=all,-render   MELEE_LOG=quiet
 *     "-render" drops even the once-per-scene block; "render" writes every block.
 *   Existing switches still work and turn their category on: MELEE_HEAP_TRACE (heap),
 *   MELEE_DVD_TRACE (dvd), MELEE_GR_TRACE / MELEE_MEX_TRACE_* / MELEE_CSS_TRACE (mex),
 *   MELEE_RB_LOG (net + snap), MELEE_PC_TRACE_OSREPORT (all). The other direction too: log=heap
 *   or log=dvd sets MELEE_HEAP_TRACE / MELEE_DVD_TRACE, so the emitters produce the lines.
 *   MELEE_SCENE_TRACE=0 still silences the scene reporter at its source.
 * A flood limit (400 lines per 10 s per prefix) guards the always-on categories; MELEE_LOG=all
 * lifts it. Quiet lines are counted, and each scene change writes one "log: ... not written" line.
 * A line identical to the one before it is counted, not written ("log: (the line above repeated
 * N more times)").
 *
 * CRASH REPORT. gw_crash_report() writes crashlogs/crash-<time>.log (at most 64 KB: header with
 * version/build/disc/mods/settings/OS, the fault lines with frames symbolized from melee-pc.map,
 * the last scene lines, pad/adapter lines, the last watchdog samples and the last ~200 written
 * lines, user paths replaced by %USERPROFILE%) and keeps the whole melee-pc.log beside it as
 * crash-<time>-full.log. A fault after "window closed, shutting down" is marked "during shutdown:
 * yes": the launcher neither alarms the player nor uploads those. */
#include "gw.h"
#include "gw_test.h"

#include <ctype.h>
#include <intrin.h>
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <windows.h>

extern int gw_Settings_Str(const char *key, char *out, int cap, const char *dflt);
extern int gw_Settings_Summary(char *out, int cap);

enum {
  GL_CORE,
  GL_SCENE,
  GL_PAD,
  GL_NET,
  GL_RENDER,
  GL_WATCHDOG,
  GL_TEX,
  GL_FRONTEND,
  GL_MEX,
  GL_HEAP,
  GL_DVD,
  GL_AUDIO,
  GL_SNAP,
  GL_COUNT
};
static const char *const gl_name[GL_COUNT] = {"core", "scene", "pad", "net", "render", "watchdog",
                                               "tex", "frontend", "mex", "heap", "dvd", "audio",
                                               "snap"};
#define GL_FIRST_QUIET GL_RENDER

#define GL_LINE 320
#define GL_TAIL 256
#define GL_SCENES 16
#define GL_PADS 16
#define GL_WATCH 8
#define GL_FLOOD_KEYS 16
#define GL_FLOOD_WINDOW_MS 10000u
#define GL_FLOOD_MAX 400u

static INIT_ONCE gl_once = INIT_ONCE_STATIC_INIT;
static CRITICAL_SECTION gl_cs;

static struct {
  int state;                /* 0 = not configured, 1 = configuring, 2 = ready */
  signed char on[GL_COUNT]; /* 1 written; 0 quiet; render: -1 never, 0 once per scene */
  int all;                  /* MELEE_LOG=all: also lifts the flood limit */
  char spec[128];           /* what configured it, for the report */
  FILE *file;
  int render_state; /* 0 open, 1 in the first copy block's tail, 2 closed until the next scene */
  int render_lines;
  unsigned dropped[GL_COUNT];
  int crashing;
  int shutting_down;
  unsigned crash_mark; /* tail count when the crash began */
  char last[GL_LINE]; /* the last line written through the filter, and how often it repeated */
  unsigned repeats;
  char pending[1024];  /* OSReport text not yet ended by a newline */
  int npending;
  struct {
    char key[16];
    DWORD since;
    unsigned n, over;
  } flood[GL_FLOOD_KEYS];
  unsigned long long t0;
  /* what a crash report needs, noted from the lines as they pass */
  char disc_path[MAX_PATH];
  char disc_id[8];
  char mods[512];
  char tail[GL_TAIL][GL_LINE];
  unsigned ntail;
  char scene[GL_SCENES][GL_LINE];
  unsigned nscene;
  char pad[GL_PADS][GL_LINE];
  unsigned npad;
  char watch[GL_WATCH][GL_LINE];
  unsigned nwatch;
} gl;

static BOOL CALLBACK gl_init_cs(PINIT_ONCE once, PVOID param, PVOID *ctx) {
  (void)once;
  (void)param;
  (void)ctx;
  InitializeCriticalSection(&gl_cs);
  return TRUE;
}

/* Recursive (a CRITICAL_SECTION is), so a fault inside the logger on the same thread can still
 * log its crash. */
static void gl_lock(void) {
  InitOnceExecuteOnce(&gl_once, gl_init_cs, NULL, NULL);
  EnterCriticalSection(&gl_cs);
}
static void gl_unlock(void) { LeaveCriticalSection(&gl_cs); }

static int gl_starts(const char *s, const char *p) { return strncmp(s, p, strlen(p)) == 0; }

static void gl_copy(char *dst, size_t cap, const char *src) {
  size_t n = strlen(src);
  if (n >= cap) n = cap - 1;
  memcpy(dst, src, n);
  dst[n] = '\0';
}

/* ---- classification ------------------------------------------------------------------------ */

/* A quiet line that reads like a problem is written anyway. */
static int gl_problem(const char *s) {
  static const char *const words[] = {"fail",    "error",  "warn",   "cannot", "can't",
                                      "unable",  "invalid", "missing", "refus", "reject",
                                      "assert",  "fatal",  "panic",  "bad ",   "overflow",
                                      "exhaust", "unsupported", "corrupt", "not ",
                                      "too ",    NULL};
  char low[GL_LINE];
  size_t i;
  for (i = 0; i + 1 < sizeof low && s[i] != '\0'; ++i) low[i] = (char)tolower((unsigned char)s[i]);
  low[i] = '\0';
  for (i = 0; words[i] != NULL; ++i) {
    if (strstr(low, words[i]) != NULL) return 1;
  }
  return 0;
}

static int gl_classify(const char *s) {
  if (gl_starts(s, "scene: ") || gl_starts(s, "gw: scene:")) return GL_SCENE;
  if (gl_starts(s, "gw: DIAG pads") || gl_starts(s, "gw: DIAG gcraw") || gl_starts(s, "gw: gc adapter") ||
      gl_starts(s, "gw: input") || gl_starts(s, "gw: pad") || gl_starts(s, "pad:") || gl_starts(s, "gw: PAD"))
    return GL_PAD;
  if (gl_starts(s, "netplay:") || gl_starts(s, "rb:") || gl_starts(s, "gw: netplay") || gl_starts(s, "net:"))
    return GL_NET;
  if (gl_starts(s, "gw: DIAG ")) return GL_RENDER;
  if (gl_starts(s, "gw: at ") || gl_starts(s, "gw:   retrace=")) return GL_WATCHDOG;
  if (gl_starts(s, "gxtex: ") && strstr(s, " -> handle ") != NULL) return GL_TEX;
  if (gl_starts(s, "frontend: ") && strstr(s, ".json: ") != NULL && strstr(s, " bytes, ") != NULL)
    return GL_FRONTEND;
  if (gl_starts(s, "interp: ")) return gl_starts(s, "interp: installed") ? GL_CORE : GL_MEX;
  if (gl_starts(s, "ftfunction: ") || gl_starts(s, "itfunction: ") || gl_starts(s, "grfunction:   ") ||
      gl_starts(s, "grtrace: ") || gl_starts(s, "mexdata: MEX_Index") || gl_starts(s, "mexeffect: bank ") || gl_starts(s, "mexeffect: fighter kind ") ||
      gl_starts(s, "item: kind "))
    return GL_MEX;
  if (gl_starts(s, "gw: kind ") && strstr(s, " Kirby hat: ") != NULL) return GL_MEX;
  if (gl_starts(s, "lbHeap: heap ") || gl_starts(s, "heap: ") || gl_starts(s, "gw: ARAlloc(")) return GL_HEAP;
  if (gl_starts(s, "gw: dvd: ") || gl_starts(s, "gw: mods:   /")) return GL_DVD;
  if (gl_starts(s, "synth: bank ") || gl_starts(s, "lbAudioAx: m-ex sfx ")) return GL_AUDIO;
  if (gl_starts(s, "snap: render pass")) return GL_SNAP;
  return GL_CORE;
}

/* ---- configuration ------------------------------------------------------------------------- */

static int gl_find(const char *name) {
  int i;
  for (i = 0; i < GL_COUNT; ++i) {
    if (_stricmp(name, gl_name[i]) == 0) return i;
  }
  return -1;
}

static void gl_defaults(void) {
  int i;
  for (i = 0; i < GL_COUNT; ++i) gl.on[i] = i < GL_FIRST_QUIET ? 1 : 0;
  gl.all = 0;
}

/* Applies a MELEE_LOG / log= list on top of the current state. */
static void gl_apply(const char *spec) {
  char buf[256];
  char *tok, *ctx = NULL;
  gl_copy(buf, sizeof buf, spec);
  for (tok = strtok_s(buf, ",; \t", &ctx); tok != NULL; tok = strtok_s(NULL, ",; \t", &ctx)) {
    int neg = 0, i;
    if (*tok == '-' || *tok == '!') {
      neg = 1;
      ++tok;
    } else if (*tok == '+') {
      ++tok;
    }
    if (_stricmp(tok, "all") == 0 || _stricmp(tok, "verbose") == 0) {
      for (i = 0; i < GL_COUNT; ++i) gl.on[i] = 1;
      gl.all = 1;
    } else if (_stricmp(tok, "default") == 0) {
      gl_defaults();
    } else if (_stricmp(tok, "quiet") == 0 || _stricmp(tok, "none") == 0) {
      gl_defaults();
      gl.on[GL_RENDER] = -1;
    } else if ((i = gl_find(tok)) >= 0) {
      gl.on[i] = neg ? (i == GL_RENDER ? -1 : 0) : 1;
    }
  }
}

static int gl_env_on(const char *name) {
  const char *v = getenv(name);
  return v != NULL && v[0] != '\0' && v[0] != '0';
}

static void gl_configure(void) {
  static const struct {
    int cat;
    const char *env;
    int push; /* category on -> set the env var so the emitter prints */
  } legacy[] = {
      {GL_HEAP, "MELEE_HEAP_TRACE", 1},    {GL_DVD, "MELEE_DVD_TRACE", 1},
      {GL_MEX, "MELEE_GR_TRACE", 0},       {GL_MEX, "MELEE_MEX_TRACE_CALLS", 0},
      {GL_MEX, "MELEE_MEX_TRACE_PARTS", 0}, {GL_MEX, "MELEE_MEX_TRACE_SCALE", 0},
      {GL_MEX, "MELEE_CSS_TRACE", 0},      {GL_NET, "MELEE_RB_LOG", 0},
      {GL_SNAP, "MELEE_RB_LOG", 0},
  };
  char saved[128];
  const char *spec = getenv("MELEE_LOG");
  size_t i;
  gl.state = 1;
  gl.t0 = GetTickCount64();
  gl_defaults();
  if (spec == NULL || spec[0] == '\0') {
    /* settings.cfg log= (the launcher's Diagnostics box writes MELEE_LOG instead) */
    gw_Settings_Str("log", saved, sizeof saved, "");
    spec = saved;
    snprintf(gl.spec, sizeof gl.spec, "%s%s", saved[0] != '\0' ? "settings log=" : "default", saved);
  } else {
    snprintf(gl.spec, sizeof gl.spec, "MELEE_LOG=%s", spec);
  }
  for (i = 0; i < sizeof legacy / sizeof legacy[0]; ++i) {
    if (gl_env_on(legacy[i].env)) gl.on[legacy[i].cat] = 1;
  }
  if (gl_env_on("MELEE_PC_TRACE_OSREPORT")) gl_apply("all");
  gl_apply(spec);
  for (i = 0; i < sizeof legacy / sizeof legacy[0]; ++i) {
    if (legacy[i].push && gl.on[legacy[i].cat] == 1 && getenv(legacy[i].env) == NULL) {
      _putenv_s(legacy[i].env, "1");
    }
  }
  gl.state = 2;
}

/* The categories as one line ("scene pad net render:scene ..."), for the log and the report. */
static void gl_describe(char *out, size_t cap) {
  size_t n = 0;
  int i;
  out[0] = '\0';
  for (i = 0; i < GL_COUNT && n + 24 < cap; ++i) {
    const char *st = gl.on[i] > 0 ? "on" : i == GL_RENDER && gl.on[i] == 0 ? "once-per-scene" : "off";
    n += (size_t)snprintf(out + n, cap - n, "%s%s=%s", i ? " " : "", gl_name[i], st);
  }
}

/* ---- the sink ------------------------------------------------------------------------------ */

static void gl_write(const char *s) {
  fputs(s, stdout);
  fputc('\n', stdout);
  fflush(stdout);
  if (gl.file == NULL) gl.file = fopen("melee-pc.log", "w");
  if (gl.file != NULL) {
    fputs(s, gl.file);
    fputc('\n', gl.file);
    fflush(gl.file);
  }
  gl_copy(gl.tail[gl.ntail % GL_TAIL], GL_LINE, s);
  ++gl.ntail;
}

static void gl_flush_repeats(void) {
  if (gl.repeats != 0) {
    char msg[64];
    snprintf(msg, sizeof msg, "log: (the line above repeated %u more time%s)", gl.repeats, gl.repeats == 1 ? "" : "s");
    gl.repeats = 0;
    gl_write(msg);
  }
}

static void gl_ring(char (*ring)[GL_LINE], unsigned cap, unsigned *n, const char *s) {
  gl_copy(ring[*n % cap], GL_LINE, s);
  ++*n;
}

/* Notes what the crash report needs from lines as they pass, written or not. */
static void gl_snoop(const char *s, int cat) {
  if (cat == GL_SCENE && gl_starts(s, "scene: ")) gl_ring(gl.scene, GL_SCENES, &gl.nscene, s);
  if (cat == GL_PAD || strstr(s, "adapter") != NULL) gl_ring(gl.pad, GL_PADS, &gl.npad, s);
  if (gl_starts(s, "gw: at ")) gl_ring(gl.watch, GL_WATCH, &gl.nwatch, s);
  if (gl_starts(s, "melee-pc: window closed")) gl.shutting_down = 1;
  if (gl_starts(s, "gw: disc image ")) {
    const char *p = s + strlen("gw: disc image ");
    const char *id = strstr(p, ", disk id ");
    if (id != NULL) {
      size_t n = (size_t)(id - p);
      if (n >= sizeof gl.disc_path) n = sizeof gl.disc_path - 1;
      memcpy(gl.disc_path, p, n);
      gl.disc_path[n] = '\0';
      gl_copy(gl.disc_id, sizeof gl.disc_id, id + strlen(", disk id "));
    }
  }
  if (gl_starts(s, "gw: mods: ") && strstr(s, " new paths)") != NULL) {
    const char *p = s + strlen("gw: mods: ");
    const char *e = strchr(p, ' ');
    size_t have = strlen(gl.mods);
    if (e != NULL && have + (size_t)(e - p) + 3 < sizeof gl.mods) {
      if (have) gl.mods[have++] = ',';
      memcpy(gl.mods + have, p, (size_t)(e - p));
      gl.mods[have + (size_t)(e - p)] = '\0';
    }
  }
}

static void gl_note_dropped(void) {
  char msg[GL_LINE];
  size_t n = 0;
  unsigned total = 0;
  int i;
  for (i = 0; i < GL_COUNT; ++i) total += gl.dropped[i];
  if (total == 0) return;
  n = (size_t)snprintf(msg, sizeof msg, "log: %u quiet line(s) not written since the last scene (", total);
  for (i = 0; i < GL_COUNT && n + 24 < sizeof msg; ++i) {
    if (gl.dropped[i] != 0) {
      n += (size_t)snprintf(msg + n, sizeof msg - n, "%s %u, ", gl_name[i], gl.dropped[i]);
    }
    gl.dropped[i] = 0;
  }
  if (n >= 2 && n < sizeof msg) n -= 2;
  snprintf(msg + n, sizeof msg - n, ") - MELEE_LOG=<category> writes them");
  gl_write(msg);
}

/* 1 = over this window's budget for the line's prefix (always-on categories only). */
static int gl_flooded(const char *s) {
  char key[16];
  DWORD now = GetTickCount();
  size_t k = 0;
  int i, slot = -1, oldest = 0;
  while (k + 1 < sizeof key && s[k] != '\0' && s[k] != ':' && s[k] != ' ') {
    key[k] = s[k];
    ++k;
  }
  key[k] = '\0';
  for (i = 0; i < GL_FLOOD_KEYS; ++i) {
    if (strcmp(gl.flood[i].key, key) == 0) {
      slot = i;
      break;
    }
    if (gl.flood[i].since < gl.flood[oldest].since) oldest = i;
  }
  if (slot < 0) {
    slot = oldest;
    gl_copy(gl.flood[slot].key, sizeof gl.flood[slot].key, key);
    gl.flood[slot].since = now;
    gl.flood[slot].n = gl.flood[slot].over = 0;
  }
  if (now - gl.flood[slot].since > GL_FLOOD_WINDOW_MS) {
    if (gl.flood[slot].over != 0) {
      char msg[96];
      snprintf(msg, sizeof msg, "log: flood limit: %u more '%s' line(s) in %u s were not written",
               gl.flood[slot].over, key, GL_FLOOD_WINDOW_MS / 1000u);
      gl_write(msg);
    }
    gl.flood[slot].since = now;
    gl.flood[slot].n = gl.flood[slot].over = 0;
  }
  if (++gl.flood[slot].n > GL_FLOOD_MAX) {
    ++gl.flood[slot].over;
    return 1;
  }
  return 0;
}

/* One finished line (no newline). Decides, notes, writes. Caller holds the lock. */
static void gl_line(const char *s) {
  int cat, pass;
  if (gl.crashing || gl.state == 1) {
    gl_flush_repeats();
    gl_write(s);
    return;
  }
  if (gl.state == 0) {
    gl_configure();
    {
      char d[GL_LINE];
      char msg[GL_LINE + 160];
      gl_describe(d, sizeof d);
      snprintf(msg, sizeof msg, "log: %s (%s) - MELEE_LOG=all writes everything", d, gl.spec);
      gl_write(s); /* the first line stays first */
      gl_write(msg);
      return;
    }
  }
  cat = gl_classify(s);
  gl_snoop(s, cat);
  if (cat == GL_SCENE && gl_starts(s, "scene: enter ")) {
    gl_note_dropped();
    gl.render_state = 0;
    gl.render_lines = 0;
  }
  if (gl.on[cat] > 0) {
    pass = 1;
  } else if (cat == GL_RENDER) {
    pass = 0;
    if (gl.on[cat] == 0) {
      /* the first block after a scene change: its matrices, the "copy #" line and the indented
       * lines that follow it */
      if (gl.render_state == 0) {
        /* one-off setup lines (vtxattrfmt, mtxidx) are not part of the block */
        if (gl_starts(s, "gw: DIAG copy #")) gl.render_state = 1;
        pass = gl_starts(s, "gw: DIAG proj") || gl_starts(s, "gw: DIAG posmtx") ||
               gl_starts(s, "gw: DIAG copy #") || gl_starts(s, "gw: DIAG   ");
        if (pass && ++gl.render_lines > 60) {
          pass = 0;
          gl.render_state = 2;
        }
      } else if (gl.render_state == 1) {
        pass = gl_starts(s, "gw: DIAG   ");
        if (!pass) gl.render_state = 2;
      }
    }
  } else {
    pass = cat != GL_WATCHDOG && gl_problem(s);
  }
  if (!pass) {
    ++gl.dropped[cat];
    return;
  }
  if (!gl.all && cat < GL_FIRST_QUIET && gl_flooded(s)) return;
  if (strncmp(s, gl.last, sizeof gl.last - 1) == 0) {
    ++gl.repeats; /* the same line again: counted, written once when something else comes */
    return;
  }
  gl_flush_repeats();
  gl_copy(gl.last, sizeof gl.last, s);
  gl_write(s);
}

/* Splits text into lines and hands each to gl_line. Caller holds the lock. */
static void gl_text(char *text) {
  char *p = text;
  for (;;) {
    char *nl = strchr(p, '\n');
    size_t n;
    if (nl != NULL) *nl = '\0';
    n = strlen(p);
    while (n > 0 && p[n - 1] == '\r') p[--n] = '\0';
    gl_line(p);
    if (nl == NULL) break;
    p = nl + 1;
    if (*p == '\0') break; /* a trailing newline ends the message, it is not an empty line */
  }
}

static void gl_flush_pending(void) {
  if (gl.npending > 0) {
    gl.pending[gl.npending] = '\0';
    gl.npending = 0;
    gl_line(gl.pending);
  }
}

void gw_logv(const char *fmt, va_list ap) {
  char buf[2048];
  vsnprintf(buf, sizeof buf, fmt, ap);
  gl_lock();
  gl_flush_pending();
  gl_text(buf);
  gl_unlock();
}

void gw_log(const char *fmt, ...) {
  va_list ap;
  va_start(ap, fmt);
  gw_logv(fmt, ap);
  va_end(ap);
}

/* OSReport: the game prints lines in pieces ("%s :" then " %5d KB" then "\n"), so text is held
 * until its newline and a report reads as the game meant it. A piece without a newline is
 * written when anything else logs, so nothing is lost before a crash. Empty lines are dropped. */
void gw_log_osreportv(const char *fmt, va_list ap) {
  char buf[2048];
  const char *p;
  vsnprintf(buf, sizeof buf, fmt, ap);
  gl_lock();
  for (p = buf; *p != '\0'; ++p) {
    if (*p == '\n') {
      gl.pending[gl.npending] = '\0';
      while (gl.npending > 0 && gl.pending[gl.npending - 1] == '\r') gl.pending[--gl.npending] = '\0';
      if (gl.npending > 0) gl_line(gl.pending);
      gl.npending = 0;
    } else {
      if (gl.npending >= (int)sizeof gl.pending - 1) gl_flush_pending();
      gl.pending[gl.npending++] = *p;
    }
  }
  gl_unlock();
}

/* Writes two strings verbatim, with no format expansion and no filtering, for use when the thing
 * being reported is itself a format string that may be about to crash the formatter. */
void gw_log_raw(const char *prefix, const char *text) {
  char buf[GL_LINE * 2];
  snprintf(buf, sizeof buf, "%s%s", prefix, text != NULL ? text : "(null)");
  gl_lock();
  gl_flush_pending();
  gl_write(buf);
  gl_unlock();
}

/* From here on every line is written unfiltered, and the report's fault section starts here. */
void gw_log_crash_begin(void) {
  gl_lock();
  gl_flush_pending();
  gl_flush_repeats();
  if (!gl.crashing) {
    gl.crashing = 1;
    gl.crash_mark = gl.ntail;
  }
  gl_unlock();
}

/* ---- the crash report ------------------------------------------------------------------------ */

#define GL_REPORT_MAX (64 * 1024)
static char gl_rep[GL_REPORT_MAX];
static size_t gl_nrep;
static char gl_user[64];    /* %USERNAME% */
static char gl_profile[MAX_PATH]; /* %USERPROFILE% */

static void rep_raw(const char *s) {
  size_t n = strlen(s);
  if (gl_nrep + n + 1 >= GL_REPORT_MAX) n = GL_REPORT_MAX - 1 - gl_nrep;
  memcpy(gl_rep + gl_nrep, s, n);
  gl_nrep += n;
  gl_rep[gl_nrep] = '\0';
}

/* Replaces the user's profile path (either slash, any case) with %USERPROFILE% and any other
 * appearance of the user name with <user>. */
static void gl_sanitize(const char *in, char *out, size_t cap) {
  size_t o = 0;
  size_t plen = strlen(gl_profile), ulen = strlen(gl_user);
  while (*in != '\0' && o + 1 < cap) {
    size_t k;
    int hit = plen >= 4;
    for (k = 0; hit && k < plen; ++k) {
      char a = (char)tolower((unsigned char)in[k]), b = (char)tolower((unsigned char)gl_profile[k]);
      if (a == '/') a = '\\';
      if (b == '/') b = '\\';
      if (in[k] == '\0' || a != b) hit = 0;
    }
    if (hit) {
      const char *r = "%USERPROFILE%";
      while (*r != '\0' && o + 1 < cap) out[o++] = *r++;
      in += plen;
      continue;
    }
    if (ulen >= 3 && _strnicmp(in, gl_user, ulen) == 0) {
      const char *r = "<user>";
      while (*r != '\0' && o + 1 < cap) out[o++] = *r++;
      in += ulen;
      continue;
    }
    out[o++] = *in++;
  }
  out[o] = '\0';
}

static void rep(const char *fmt, ...) {
  char raw[1024], clean[1200];
  va_list ap;
  va_start(ap, fmt);
  vsnprintf(raw, sizeof raw, fmt, ap);
  va_end(ap);
  gl_sanitize(raw, clean, sizeof clean);
  rep_raw(clean);
}

static void gl_exe_dir(char *out, size_t cap) {
  DWORD k = GetModuleFileNameA(NULL, out, (DWORD)cap);
  char *slash = k > 0 && k < cap ? strrchr(out, '\\') : NULL;
  if (slash != NULL) {
    slash[1] = '\0';
  } else {
    out[0] = '\0';
  }
}

static DWORD gl_link_stamp(void) {
  const unsigned char *base = (const unsigned char *)GetModuleHandleA(NULL);
  const IMAGE_DOS_HEADER *dos = (const IMAGE_DOS_HEADER *)base;
  const IMAGE_NT_HEADERS *nt;
  if (base == NULL || dos->e_magic != IMAGE_DOS_SIGNATURE) return 0;
  nt = (const IMAGE_NT_HEADERS *)(base + dos->e_lfanew);
  return nt->Signature == IMAGE_NT_SIGNATURE ? nt->FileHeader.TimeDateStamp : 0;
}

static void rep_header(const char *kind, const char *reason, const SYSTEMTIME *st) {
  char dir[MAX_PATH], path[MAX_PATH], line[512], settings[1024], cats[GL_LINE];
  FILE *f;
  rep("==== GD's Melee crash report ====\n");
  rep("time:            %04u-%02u-%02u %02u:%02u:%02u (up %llu s)\n", st->wYear, st->wMonth, st->wDay,
      st->wHour, st->wMinute, st->wSecond, (GetTickCount64() - gl.t0) / 1000ull);
  rep("exit path:       %s\n", kind);
  rep("reason:          %s\n", reason != NULL ? reason : "(none)");
  rep("during shutdown: %s\n", gl.shutting_down ? "yes (the window had been closed; the game was tearing down)" : "no");
  gl_exe_dir(dir, sizeof dir);
  snprintf(path, sizeof path, "%sversion.txt", dir);
  line[0] = '\0';
  f = fopen(path, "r");
  if (f != NULL) {
    if (fgets(line, sizeof line, f) == NULL) line[0] = '\0';
    fclose(f);
  }
  line[strcspn(line, "\r\n")] = '\0';
  rep("version:         %s\n", line[0] != '\0' ? line : "development build (no version.txt)");
  rep("build id:        %08lX (melee-pc.exe link time)\n", (unsigned long)gl_link_stamp());
  {
    char title[0x41] = "";
    const char *base = strrchr(gl.disc_path, '\\');
    const char *base2 = strrchr(gl.disc_path, '/');
    if (base2 != NULL && (base == NULL || base2 > base)) base = base2;
    f = gl.disc_path[0] != '\0' ? fopen(gl.disc_path, "rb") : NULL;
    if (f != NULL) {
      if (fseek(f, 0x20, SEEK_SET) == 0 && fread(title, 1, 0x40, f) == 0x40) title[0x40] = '\0';
      else title[0] = '\0';
      fclose(f);
    }
    rep("disc:            %s \"%s\" (%s)\n", gl.disc_id[0] ? gl.disc_id : "?", title,
        base != NULL ? base + 1 : gl.disc_path[0] ? gl.disc_path : "no disc yet");
  }
  rep("mods mounted:    %s\n", gl.mods[0] != '\0' ? gl.mods : "none");
  settings[0] = '\0';
  gw_Settings_Summary(settings, sizeof settings);
  rep("settings.cfg:    %s\n", settings[0] != '\0' ? settings : "(none)");
  gl_describe(cats, sizeof cats);
  rep("log:             %s (%s)\n", cats, gl.spec);
  {
    /* MELEE_* switches in effect (the launcher's toggles arrive this way) */
    char env[1024];
    size_t n = 0;
    LPCH block = GetEnvironmentStringsA();
    LPCH v;
    env[0] = '\0';
    for (v = block; v != NULL && *v != '\0'; v += strlen(v) + 1) {
      if (strncmp(v, "MELEE_", 6) == 0 && n + strlen(v) + 2 < sizeof env) {
        n += (size_t)snprintf(env + n, sizeof env - n, "%s%s", n ? " " : "", v);
      }
    }
    if (block != NULL) FreeEnvironmentStringsA(block);
    rep("environment:     %s\n", env[0] != '\0' ? env : "(no MELEE_* variables)");
  }
  {
    typedef LONG(WINAPI * RtlGetVersionFn)(OSVERSIONINFOW *);
    OSVERSIONINFOW v;
    RtlGetVersionFn fn = (RtlGetVersionFn)(void *)GetProcAddress(GetModuleHandleA("ntdll.dll"), "RtlGetVersion");
    BOOL wow = FALSE;
    SYSTEM_INFO si;
    MEMORYSTATUSEX ms;
    memset(&v, 0, sizeof v);
    v.dwOSVersionInfoSize = sizeof v;
    if (fn != NULL) fn(&v);
    IsWow64Process(GetCurrentProcess(), &wow);
    GetNativeSystemInfo(&si);
    ms.dwLength = sizeof ms;
    GlobalMemoryStatusEx(&ms);
    rep("os:              Windows %lu.%lu.%lu, %s, %lu cpus, %llu MB RAM\n", v.dwMajorVersion, v.dwMinorVersion,
        v.dwBuildNumber, wow ? "32-bit game on 64-bit Windows" : "32-bit Windows", si.dwNumberOfProcessors,
        ms.ullTotalPhys / (1024ull * 1024ull));
  }
}

/* Symbolizes every "rva 0x........" in the fault lines against melee-pc.map (only when the map's
 * timestamp matches this exe's link time). */
static void rep_symbols(unsigned from) {
  uint32_t rva[48];
  char best_name[48][80];
  uint32_t best_addr[48];
  int nrva = 0, i;
  unsigned t;
  char dir[MAX_PATH], path[MAX_PATH], line[512];
  FILE *f;
  for (t = from; t < gl.ntail && nrva < 48; ++t) {
    const char *s = gl.tail[t % GL_TAIL];
    const char *p = s;
    while ((p = strstr(p, "rva 0x")) != NULL && nrva < 48) {
      uint32_t a = (uint32_t)strtoul(p + 6, NULL, 16);
      int dup = 0;
      for (i = 0; i < nrva; ++i) dup |= rva[i] == a;
      if (!dup && a != 0) {
        rva[nrva] = a;
        best_addr[nrva] = 0;
        best_name[nrva][0] = '\0';
        ++nrva;
      }
      p += 6;
    }
  }
  if (nrva == 0) return;
  rep("\n---- frames, symbolized from melee-pc.map ----\n");
  gl_exe_dir(dir, sizeof dir);
  snprintf(path, sizeof path, "%smelee-pc.map", dir);
  f = fopen(path, "r");
  if (f == NULL) {
    rep("(no melee-pc.map beside the exe)\n");
    return;
  }
  {
    DWORD stamp = gl_link_stamp();
    int ok = 0, lines = 0;
    while (fgets(line, sizeof line, f) != NULL && lines++ < 8) {
      const char *ts = strstr(line, "Timestamp is ");
      if (ts != NULL) {
        ok = (DWORD)strtoul(ts + 13, NULL, 16) == stamp;
        break;
      }
    }
    if (!ok) {
      fclose(f);
      rep("(melee-pc.map is from a different build - not symbolized)\n");
      return;
    }
  }
  while (fgets(line, sizeof line, f) != NULL) {
    unsigned sect, off, addr;
    char name[256];
    if (sscanf(line, " %x:%x %255s %x", &sect, &off, name, &addr) != 4 || sect != 1) continue;
    for (i = 0; i < nrva; ++i) {
      if (addr <= rva[i] && addr >= best_addr[i]) {
        best_addr[i] = addr;
        gl_copy(best_name[i], sizeof best_name[i], name);
      }
    }
  }
  fclose(f);
  for (i = 0; i < nrva; ++i) {
    if (best_name[i][0] != '\0' && rva[i] - best_addr[i] < 0x20000u) {
      rep("  0x%08X  %s+0x%X\n", rva[i], best_name[i], rva[i] - best_addr[i]);
    } else {
      rep("  0x%08X  (not in a function: data, or a stale pointer)\n", rva[i]);
    }
  }
}

static void rep_ring(const char *title, char (*ring)[GL_LINE], unsigned cap, unsigned n) {
  unsigned t = n > cap ? n - cap : 0;
  rep("\n---- %s ----\n", title);
  if (n == 0) rep("(none)\n");
  for (; t < n; ++t) rep("%s\n", ring[t % cap]);
}

static int gl_write_report(const char *dst, const char *kind, const char *reason, const SYSTEMTIME *st) {
  FILE *out;
  unsigned first_tail = gl.ntail > GL_TAIL ? gl.ntail - GL_TAIL : 0;
  unsigned fault_from, t, start;
  size_t budget, used;
  gl_nrep = 0;
  gl_rep[0] = '\0';
  {
    DWORD n = sizeof gl_user;
    if (!GetUserNameA(gl_user, &n)) gl_user[0] = '\0';
    n = GetEnvironmentVariableA("USERPROFILE", gl_profile, sizeof gl_profile);
    if (n == 0 || n >= sizeof gl_profile) gl_profile[0] = '\0';
  }
  rep_header(kind, reason, st);

  /* the fault: a few lines of lead-in, then everything since the crash began (capped) */
  fault_from = gl.crashing ? gl.crash_mark : gl.ntail;
  fault_from = fault_from > 12 ? fault_from - 12 : 0;
  if (fault_from < first_tail) fault_from = first_tail;
  rep("\n---- fault ----\n");
  for (t = fault_from; t < gl.ntail && gl_nrep < 20 * 1024; ++t) rep("%s\n", gl.tail[t % GL_TAIL]);
  rep_symbols(fault_from);
  rep_ring("last scene lines", gl.scene, GL_SCENES, gl.nscene);
  rep_ring("pad / adapter", gl.pad, GL_PADS, gl.npad);
  rep_ring("last watchdog samples", gl.watch, GL_WATCH, gl.nwatch);

  /* the newest written lines that still fit, up to 200 */
  budget = GL_REPORT_MAX - 256 > gl_nrep ? GL_REPORT_MAX - 256 - gl_nrep : 0;
  used = 0;
  start = fault_from;
  while (start > first_tail && fault_from - (start - 1) <= 200) {
    size_t len = strlen(gl.tail[(start - 1) % GL_TAIL]) + 1;
    if (used + len > budget) break;
    used += len;
    --start;
  }
  rep("\n---- last %u log lines before the fault ----\n", fault_from - start);
  for (t = start; t < fault_from; ++t) rep("%s\n", gl.tail[t % GL_TAIL]);
  rep("---- end ----\n");

  out = fopen(dst, "wb");
  if (out == NULL) return -1;
  fwrite(gl_rep, 1, gl_nrep, out);
  fclose(out);
  return 0;
}

/* Writes crashlogs\crash-<time>.log (the compact report) and crashlogs\crash-<time>-full.log (the
 * whole melee-pc.log, as before). kind: "panic" | "unhandled exception" | "CRT invalid parameter". */
void gw_crash_report(const char *kind, const char *reason) {
  SYSTEMTIME st;
  char stem[MAX_PATH], dst[MAX_PATH];
  FILE *in, *out;
  gw_log_crash_begin();
  gl_lock();
  if (strncmp(kind, "panic", 5) == 0) {
    /* an assert has no fault frames of its own: walk this stack (frames in the exe only, as map
     * rvas, so the report symbolizes them) */
    uintptr_t frames[32];
    const unsigned char *base = (const unsigned char *)GetModuleHandleA(NULL);
    const IMAGE_NT_HEADERS *nt = (const IMAGE_NT_HEADERS *)(base + ((const IMAGE_DOS_HEADER *)base)->e_lfanew);
    int n = 0, i;
    char line[96];
    {
      /* a stack scan, as the fault handler does: every word between here and the stack's base
       * that points into the exe's code (optimised frames keep no frame-pointer chain) */
      ULONG_PTR lo = 0, hi = 0;
      const uintptr_t *p = (const uintptr_t *)_AddressOfReturnAddress();
      uintptr_t code_lo = (uintptr_t)base + nt->OptionalHeader.BaseOfCode;
      uintptr_t code_hi = code_lo + nt->OptionalHeader.SizeOfCode;
      GetCurrentThreadStackLimits(&lo, &hi);
      for (; (uintptr_t)p + sizeof *p <= hi && n < 32 && (uintptr_t)p - (uintptr_t)_AddressOfReturnAddress() < 16384; ++p) {
        if (*p >= code_lo && *p < code_hi) frames[n++] = *p;
      }
    }
    gl_write("gw:   stack scan (image addresses, innermost first):");
    for (i = 0; i < n; ++i) {
      uintptr_t a = frames[i];
      if (a >= (uintptr_t)base && a < (uintptr_t)base + nt->OptionalHeader.SizeOfImage) {
        snprintf(line, sizeof line, "gw:     melee-pc.map rva 0x%08X",
                 (unsigned)(a - (uintptr_t)base + 0x10000000u));
        gl_write(line);
      }
    }
  }
  GetLocalTime(&st);
  CreateDirectoryA("crashlogs", NULL);
  snprintf(stem, sizeof stem, "crashlogs\\crash-%04u%02u%02u-%02u%02u%02u", st.wYear, st.wMonth, st.wDay,
           st.wHour, st.wMinute, st.wSecond);
  if (gl.file != NULL) fflush(gl.file);

  snprintf(dst, sizeof dst, "%s.log", stem);
  gl_write_report(dst, kind, reason, &st);

  snprintf(dst, sizeof dst, "%s-full.log", stem);
  out = fopen(dst, "wb");
  if (out != NULL) {
    char header[512];
    int n = snprintf(header, sizeof header, "==== crash %04u-%02u-%02u %02u:%02u:%02u  %s: %s%s ====\n",
                     st.wYear, st.wMonth, st.wDay, st.wHour, st.wMinute, st.wSecond, kind,
                     reason != NULL ? reason : "(no reason)", gl.shutting_down ? " (during shutdown)" : "");
    if (n > 0) fwrite(header, 1, (size_t)n, out);
    in = fopen("melee-pc.log", "rb");
    if (in != NULL) {
      char buf[4096];
      size_t r;
      while ((r = fread(buf, 1, sizeof buf, in)) > 0) fwrite(buf, 1, r, out);
      fclose(in);
    }
    fclose(out);
  }
  gl_write("gw: crash report written to crashlogs (the compact .log and the -full.log)");
  gl_unlock();
}

void gw_archive_crash_log(const char *reason) { gw_crash_report("crash", reason); }

/* MELEE_CRASH_TEST="<seconds>[:fault|panic|shutdown]" crashes on purpose that many seconds after
 * start, to try the crash report and the launcher's crash handling:
 *   fault     an access violation (the unhandled-exception path; the default)
 *   panic     gw_panic, as a failed assert would
 *   shutdown  logs the window-closed line first, then faults: a teardown fault, which the report
 *             marks "during shutdown: yes" and the launcher neither announces nor uploads
 * Called from the watchdog thread, ten times a second. */
void gw_log_crash_test_tick(void) {
  static int armed = -1;
  static unsigned long long at;
  static char kind[16];
  if (armed < 0) {
    const char *v = getenv("MELEE_CRASH_TEST");
    armed = 0;
    if (v != NULL && v[0] >= '0' && v[0] <= '9') {
      const char *colon = strchr(v, ':');
      at = gl.t0 + (unsigned long long)strtoul(v, NULL, 10) * 1000ull;
      gl_copy(kind, sizeof kind, colon != NULL ? colon + 1 : "fault");
      armed = 1;
      gw_log("log: MELEE_CRASH_TEST will crash (%s) %s s after start", kind, v);
    }
  }
  if (armed != 1 || GetTickCount64() < at) return;
  armed = 2;
  if (strcmp(kind, "panic") == 0) {
    gw_panic("MELEE_CRASH_TEST: a deliberate panic");
  }
  if (strcmp(kind, "shutdown") == 0) {
    gw_log("melee-pc: window closed, shutting down");
  }
  gw_log("log: MELEE_CRASH_TEST: faulting now");
  *(volatile int *)(uintptr_t)0x10 = 1;
}

/* ---- tests ----------------------------------------------------------------------------------- */

static int test_log_categories(void) {
  signed char saved_on[GL_COUNT];
  int saved_all = gl.all, rc = 0;
  memcpy(saved_on, gl.on, sizeof saved_on);
  if (gl_classify("gxtex: C:/x/ui/glyph_a.gxtex -> handle 3: 16x16 fmt 0") != GL_TEX ||
      gl_classify("gw: DIAG proj 1 0 0 0") != GL_RENDER || gl_classify("gw: DIAG pads: PADCount=1") != GL_PAD ||
      gl_classify("interp: thunk 4 -> guest 0x812EE290 (m-ex stage callback)") != GL_MEX ||
      gl_classify("interp: installed PlSn.dat ftFunction for kind 37") != GL_CORE ||
      gl_classify("grfunction:   StageData word 1 -> 0x812EE020") != GL_MEX ||
      gl_classify("grfunction: mexData stage tables ready: 155 internal") != GL_CORE ||
      gl_classify("gw: at ntdll.dll+0x7B9DC") != GL_WATCHDOG ||
      gl_classify("gw: still at ntdll.dll+0x7B9DC after 4s - spinning") != GL_CORE ||
      gl_classify("gw: heartbeat retrace=600 presented=600") != GL_CORE ||
      gl_classify("frontend: hub_layout.json: 23513 bytes, 968 nodes") != GL_FRONTEND ||
      gl_classify("frontend: menu MAIN (kind 0, sel 0)") != GL_CORE ||
      gl_classify("scene: enter mode=GM_VS(2)") != GL_SCENE) {
    gw_test_fail("a line landed in the wrong category");
    rc = 1;
  }
  if (!gl_problem("interp: kind 49 overrides slot 37 but the port does not dispatch it") ||
      gl_problem("interp: thunk 4 -> guest 0x812EE290 (m-ex stage callback)")) {
    gw_test_fail("problem-word detection is off");
    rc = 1;
  }
  gl_defaults();
  gl_apply("mex, -scene;all");
  if (gl.on[GL_TEX] != 1 || !gl.all) {
    gw_test_fail("MELEE_LOG=all did not turn everything on");
    rc = 1;
  }
  gl_defaults();
  gl_apply("mex,-scene,-render");
  if (gl.on[GL_MEX] != 1 || gl.on[GL_SCENE] != 0 || gl.on[GL_RENDER] != -1 || gl.on[GL_TEX] != 0) {
    gw_test_fail("MELEE_LOG=mex,-scene,-render parsed wrong");
    rc = 1;
  }
  memcpy(gl.on, saved_on, sizeof saved_on);
  gl.all = saved_all;
  {
    char out[256], profile[64], line[128];
    /* Built at run time: a literal user-profile path in the exe fails check_release.ps1's
     * personal-path guard, test data or not. */
    snprintf(profile, sizeof profile, "C:\\%s\\Someone", "Users");
    snprintf(line, sizeof line, "gw: card at c:/%s/someone/Desktop/x and Someone's", "users");
    gl_copy(gl_profile, sizeof gl_profile, profile);
    gl_copy(gl_user, sizeof gl_user, "Someone");
    gl_sanitize(line, out, sizeof out);
    if (strcmp(out, "gw: card at %USERPROFILE%/Desktop/x and <user>'s") != 0) {
      gw_test_fail("sanitize gave \"%s\"", out);
      rc = 1;
    }
    gl_profile[0] = gl_user[0] = '\0';
  }
  return rc;
}

void gw_log_tests_register(void) { gw_test_register("log_categories", test_log_categories); }
