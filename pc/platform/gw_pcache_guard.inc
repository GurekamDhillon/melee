/* Pre-boot guard for Aurora's persistent pipeline cache (<cache dir>/pipeline_cache.db). Included by main.c.
 *
 * Why: Aurora replays EVERY row of that file at boot, in background workers, and validates none of it
 * (lib/gfx/pipeline_cache.cpp load_pipeline_cache_entries). A row whose shader cannot be generated is an
 * aurora FATAL, so one bad frame persisted in a run folder bricked every later launch of that folder
 * (2026-10-05: "unhandled tcg src 21" at log frame 2, before any scene; before it, 2026-09: "unmapped vtx
 * attr 13", the incident gw_cache_path()'s comment records). Rows are written by find_pipeline() BEFORE the
 * shader is built, so the very draw that crashes the game also writes its own poison.
 *
 * What: before aurora_initialize opens the file, drop every GX row gw_pcache_check_gx() rejects, with its tag
 * and origin side rows, and log each. Never fatal: any sqlite error leaves the file as it was and the boot
 * goes on. Aurora then loads only rows its own shader generator can build. This covers the boot replay fully
 * without an Aurora change. It cannot cover a bad configuration produced LIVE (that is the GX shim's
 * gw_gx_guard_indirect, shim_gx.c, and the Aurora patch in _build/audit-20261003/motion-crash/aurora.patch).
 *
 * sqlite3 is linked into the exe already (aurora_core uses it; extern\sqlite3.lib in melee_link_libs.rsp), so the
 * handful of entry points used are declared here rather than pulling the amalgamation header onto this file's
 * include path. They are the stable sqlite3 C ABI (cdecl). */
#include "gw_pcache_check.h"

typedef struct gw_sqlite3 gw_sqlite3;
typedef struct gw_sqlite3_stmt gw_sqlite3_stmt;
extern int sqlite3_open_v2(const char *, gw_sqlite3 **, int, const char *);
extern int sqlite3_close(gw_sqlite3 *);
extern int sqlite3_exec(gw_sqlite3 *, const char *, int (*)(void *, int, char **, char **), void *, char **);
extern int sqlite3_prepare_v2(gw_sqlite3 *, const char *, int, gw_sqlite3_stmt **, const char **);
extern int sqlite3_step(gw_sqlite3_stmt *);
extern int sqlite3_reset(gw_sqlite3_stmt *);
extern int sqlite3_finalize(gw_sqlite3_stmt *);
extern int sqlite3_bind_int64(gw_sqlite3_stmt *, int, long long);
extern long long sqlite3_column_int64(gw_sqlite3_stmt *, int);
extern int sqlite3_column_int(gw_sqlite3_stmt *, int);
extern const void *sqlite3_column_blob(gw_sqlite3_stmt *, int);
extern int sqlite3_column_bytes(gw_sqlite3_stmt *, int);
extern const char *sqlite3_errmsg(gw_sqlite3 *);
#define GW_SQLITE_ROW 100
#define GW_SQLITE_DONE 101
#define GW_SQLITE_OPEN_READWRITE 0x2
#define GW_PCACHE_SHADER_TYPE_GX 1 /* aurora::gfx::ShaderType::GX (rows of type 0 are the 8-byte clear configs) */
#define GW_PCACHE_DROP_MAX 32768

/* Returns the number of rows dropped (>= 0), or -1 when the file could not be examined. */
static int gw_pcache_guard_run(const char *cache_dir) {
  char path[1024];
  gw_sqlite3 *db = NULL;
  gw_sqlite3_stmt *scan = NULL;
  long long *drop = NULL;
  int dropped = 0, scanned = 0, unknown = 0, rc, i;

  if (cache_dir == NULL || cache_dir[0] == '\0') return -1; /* aurora's own default location: not ours to find */
  if (snprintf(path, sizeof path, "%spipeline_cache.db", cache_dir) >= (int)sizeof path) return -1;
  {
    size_t n = strlen(cache_dir);
    char last = cache_dir[n - 1];
    if (last != '/' && last != '\\') {
      if (snprintf(path, sizeof path, "%s/pipeline_cache.db", cache_dir) >= (int)sizeof path) return -1;
    }
  }
  {
    FILE *probe = fopen(path, "rb");
    if (probe == NULL) return 0; /* first run of this folder: nothing persisted yet */
    fclose(probe);
  }
  if (sqlite3_open_v2(path, &db, GW_SQLITE_OPEN_READWRITE, NULL) != 0) {
    gw_log("pcache: guard could not open %s (%s); left as is", path, db ? sqlite3_errmsg(db) : "?");
    if (db) sqlite3_close(db);
    return -1;
  }
  drop = (long long *)malloc(sizeof(long long) * GW_PCACHE_DROP_MAX);
  if (drop == NULL) {
    sqlite3_close(db);
    return -1;
  }
  rc = sqlite3_prepare_v2(db, "SELECT hash, config_version, config_size, config FROM pipeline_cache WHERE type = 1", -1,
                          &scan, NULL);
  if (rc != 0 || scan == NULL) {
    /* no such table yet (fresh or foreign file): Aurora creates or resets it itself */
    if (scan) sqlite3_finalize(scan);
    sqlite3_close(db);
    free(drop);
    return 0;
  }
  while (sqlite3_step(scan) == GW_SQLITE_ROW) {
    const unsigned char *blob = (const unsigned char *)sqlite3_column_blob(scan, 3);
    const int size = sqlite3_column_bytes(scan, 3);
    char why[160];
    int verdict;
    ++scanned;
    if (blob == NULL || size < 0) continue;
    verdict = gw_pcache_check_gx(blob, (size_t)size, (unsigned)sqlite3_column_int(scan, 1), why, sizeof why);
    if (verdict == GW_PCACHE_UNKNOWN_LAYOUT) {
      ++unknown;
      continue;
    }
    if (verdict == GW_PCACHE_OK) continue;
    if (dropped < GW_PCACHE_DROP_MAX) drop[dropped] = sqlite3_column_int64(scan, 0);
    if (dropped < 16)
      gw_log("pcache: dropping GX pipeline %lld from the persisted cache: %s (%s)", sqlite3_column_int64(scan, 0),
             gw_pcache_reason_name(verdict), why);
    ++dropped;
  }
  sqlite3_finalize(scan);
  if (dropped > 0) {
    const int n = dropped < GW_PCACHE_DROP_MAX ? dropped : GW_PCACHE_DROP_MAX;
    static const char *const stmts[] = {"DELETE FROM pipeline_cache WHERE type = 1 AND hash = ?",
                                        "DELETE FROM pipeline_tags WHERE type = 1 AND hash = ?",
                                        "DELETE FROM pipeline_origins WHERE type = 1 AND hash = ?"};
    int failed = 0;
    sqlite3_exec(db, "BEGIN IMMEDIATE", NULL, NULL, NULL);
    for (i = 0; i < 3; ++i) {
      gw_sqlite3_stmt *del = NULL;
      int k;
      if (sqlite3_prepare_v2(db, stmts[i], -1, &del, NULL) != 0 || del == NULL) {
        if (del) sqlite3_finalize(del);
        if (i == 0) failed = 1; /* the side tables may not exist: only the main table is required */
        continue;
      }
      for (k = 0; k < n; ++k) {
        sqlite3_reset(del);
        sqlite3_bind_int64(del, 1, drop[k]);
        if (sqlite3_step(del) != GW_SQLITE_DONE && i == 0) failed = 1;
      }
      sqlite3_finalize(del);
    }
    if (failed || sqlite3_exec(db, "COMMIT", NULL, NULL, NULL) != 0) {
      sqlite3_exec(db, "ROLLBACK", NULL, NULL, NULL);
      gw_log("pcache: guard found %d bad GX row(s) but could not delete them (%s); left as is", dropped,
             sqlite3_errmsg(db));
      dropped = -1;
    }
  }
  sqlite3_close(db);
  free(drop);
  if (dropped != 0 || unknown != 0)
    gw_log("pcache: guard checked %d GX row(s) in %s: %d dropped, %d of an unchecked layout", scanned, path,
           dropped < 0 ? 0 : dropped, unknown);
  return dropped;
}
