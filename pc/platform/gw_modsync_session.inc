/* gw_modsync_session.inc - the LIVE half of gw_modsync.h: "Match the host's mods" as a small state machine, included by gw_mods.c after
 * gw_modsync.inc. States (gw_ms_phase): IDLE -> DOWNLOADING -> APPLYING -> RELAUNCH, or FAILED (anything that goes wrong; the journal is rolled
 * back first, so a failure leaves the player's own set exactly as it was).
 *
 *   start      plan from the live manifests (refused when a mod is not obtainable); journal begins (the player's enabled set is saved);
 *              enabled.txt is made an explicit list; the room is left; the Nucleus downloads are queued (the engine does no network work
 *              while a netplay session is open, so the room goes first - the host keeps the room open for the guest's return).
 *   DOWNLOADING  tick polls the engine; all done -> APPLYING; one failed -> FAILED + restore.
 *   APPLYING   enabled.txt (as the installer left it) + ENABLE + downloaded folders - DISABLE, written atomically; journal APPLIED.
 *   RELAUNCH   journal LAUNCHING, the game starts itself again with the same command line and environment, this process exits. The new
 *              process's boot (gw_ms_boot) sees LAUNCHING and makes the session ACTIVE with the room code to rejoin.
 * The session ends (gw_ms_end_session) when the room is left: enabled.txt gets the player's own set back at once; the mounted set stays until
 * the next boot. "Keep these mods" (gw_ms_keep) drops the journal instead. A game that dies first is restored by the next boot.
 *
 * Test hooks: MELEE_MODSYNC_CRASH_AT=applied   die (TerminateProcess) right after the journal says APPLIED, before relaunching;
 *             MELEE_MODSYNC_NO_RELAUNCH=1      stop in RELAUNCH with the journal LAUNCHING instead of starting the game again. */

/* engine hooks, defined in gw_script.c next to the Nucleus engine (gw_nucleus_engine.inc) */
extern int gw_ms_nc_queue(const int *posts, int n);                                     /* queue and start; 0 = ok */
extern int gw_ms_nc_state(int post, int *pct, char *why, int cap);                      /* 0 waiting/running, 1 done, -1 failed */
extern int gw_ms_nc_folder(int post, char *folder, int cap);                            /* installed folder of that post, 1 = found */
extern void gw_ms_nc_pump(void);
extern int gw_ms_nc_queue(const int *posts, int n);
extern void gw_Netplay_Leave(void);
extern const char *gw_Netplay_Code(void);

static struct {
    int phase;
    char status[200];
    int active;                 /* this process runs a session (ACTIVE journal) */
    char room[8];
    gw_ms_manifest full;        /* every installed mod at start */
    gw_ms_row rows[GW_MS_MAX * 2];
    int nrows, have_plan;
    gw_ms_plan plan;
    int posts[GW_MS_MAX], nposts;
    int done_ms;
} ms_s;

static int64_t ms_now(void) { return (int64_t) time(NULL); }
static const char *ms_dir(void) { return gw_Mods_Dir(); }
static void ms_say(const char *fmt, ...) {
    va_list ap;
    va_start(ap, fmt);
    vsnprintf(ms_s.status, sizeof ms_s.status, fmt, ap);
    va_end(ap);
    gw_log("modsync: %s", ms_s.status);
}

int gw_ms_phase(void) { return ms_s.phase; }
const char *gw_ms_status(void) { return ms_s.status; }
int gw_ms_session_active(void) { return ms_s.active; }
const char *gw_ms_session_room(void) { return ms_s.room; }
int gw_ms_plan_view(const gw_ms_plan **plan, const gw_ms_row **rows, int *nrows) {
    if (!ms_s.have_plan) return 0;
    if (plan != NULL) *plan = &ms_s.plan;
    if (rows != NULL) *rows = ms_s.rows;
    if (nrows != NULL) *nrows = ms_s.nrows;
    return 1;
}

/* Called by boot() before enabled.txt is read. */
static int ms_pid_alive(uint32_t pid) {
    HANDLE h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, FALSE, pid);
    DWORD code = 0;
    int alive;
    if (h == NULL) return 0;
    alive = GetExitCodeProcess(h, &code) && code == STILL_ACTIVE;
    CloseHandle(h);
    return alive;
}
static void ms_boot_check(const char *dir) {
    char room[8] = { 0 };
    int r = gw_ms_boot(dir, ms_now(), GetCurrentProcessId(), ms_pid_alive, room, (int) sizeof room);
    if (r == 1) {
        ms_s.active = 1;
        snprintf(ms_s.room, sizeof ms_s.room, "%s", room);
        gw_log("modsync: a synced mod set is active (room %s) - it is temporary and is put back when you leave the room", room[0] ? room : "?");
    } else if (r == 2) {
        gw_log("modsync: the last game ended during a mod sync - your own mod set was restored");
    }
}

static void ms_fail(const char *why) {
    gw_ms_j_restore(ms_dir());
    ms_s.phase = GW_MS_PH_FAILED;
    ms_say("%s", why);
}

int gw_ms_start(const char *room_code) {
    const gw_ms_manifest *theirs;
    const char *ids[GW_MS_MAX];
    int i, n = 0;
    if (ms_s.phase != GW_MS_PH_IDLE && ms_s.phase != GW_MS_PH_FAILED) return -1;
    if (ms_s.active) { ms_say("A mod sync is already active - leave the room first"); return -1; }
    if (!gw_ms_peer_ready()) { ms_say("The other player's mod list has not arrived yet"); return -1; }
    theirs = gw_ms_peer();
    gw_ms_build_local(&ms_s.full, 0);
    ms_s.nrows = gw_ms_diff(&ms_s.full, theirs, ms_s.rows, GW_MS_MAX * 2);
    if (ms_s.nrows > GW_MS_MAX * 2) ms_s.nrows = GW_MS_MAX * 2;
    gw_ms_plan_build(&ms_s.full, theirs, &ms_s.plan);
    ms_s.have_plan = 1;
    if (!ms_s.plan.can_complete) { ms_s.phase = GW_MS_PH_FAILED; ms_say("%s", ms_s.plan.summary); return -1; }
    if (ms_s.plan.ndownload + ms_s.plan.nenable + ms_s.plan.ndisable == 0) { ms_say("Nothing to change"); return -1; }
    for (i = 0; i < ms_s.full.n && n < GW_MS_MAX; ++i) if (ms_s.full.e[i].enabled) ids[n++] = ms_s.full.e[i].id;
    if (gw_ms_j_begin(ms_dir(), room_code != NULL ? room_code : "", ids, n, ms_now()) != 0) { ms_say("Could not start (a sync is already recorded)"); return -1; }
    gw_ms_write_enabled(ms_dir(), ids, n); /* an explicit list from here on: the installer and the plan edit it */
    ms_s.nposts = 0;
    for (i = 0; i < ms_s.plan.n; ++i) if (ms_s.plan.a[i].kind == GW_MS_ACT_DOWNLOAD && ms_s.nposts < GW_MS_MAX) ms_s.posts[ms_s.nposts++] = ms_s.plan.a[i].post;
    gw_Netplay_Leave(); /* no downloads run while a session is open; the host keeps the room for our return */
    if (ms_s.nposts > 0) {
        if (gw_ms_nc_queue(ms_s.posts, ms_s.nposts) != 0) { ms_fail("Could not start the downloads"); return -1; }
        ms_s.phase = GW_MS_PH_DOWNLOADING;
        ms_say("Downloading %d mod%s from SSBM Nucleus", ms_s.nposts, ms_s.nposts == 1 ? "" : "s");
    } else {
        ms_s.phase = GW_MS_PH_APPLYING;
        ms_say("Setting up the mods");
    }
    return 0;
}

/* enabled.txt -> ids (explicit list the journal made); returns the count */
static int ms_read_enabled(char ids[][GW_MS_ID], int cap) {
    char path[MAX_PATH], line[256];
    FILE *f;
    int n = 0;
    snprintf(path, sizeof path, "%s\\enabled.txt", ms_dir());
    f = fopen(path, "rb");
    if (f == NULL) return 0;
    while (fgets(line, sizeof line, f) != NULL && n < cap) {
        char *p = line, *e;
        while (*p == ' ' || *p == '\t') ++p;
        if (*p == '#' || *p == '\r' || *p == '\n' || *p == '\0') continue;
        e = p + strlen(p);
        while (e > p && (e[-1] == '\r' || e[-1] == '\n' || e[-1] == ' ' || e[-1] == '\t')) --e;
        *e = '\0';
        snprintf(ids[n++], GW_MS_ID, "%s", p);
    }
    fclose(f);
    return n;
}

static int ms_has(char ids[][GW_MS_ID], int n, const char *id) {
    int i;
    for (i = 0; i < n; ++i) if (_stricmp(ids[i], id) == 0) return 1;
    return 0;
}

static void ms_apply_and_relaunch(void) {
    static char ids[GW_MS_MAX * 2][GW_MS_ID];
    const char *pp[GW_MS_MAX * 2];
    int n = ms_read_enabled(ids, GW_MS_MAX * 2), i, k;
    for (i = 0; i < ms_s.plan.n; ++i) {
        const gw_ms_action *a = &ms_s.plan.a[i];
        if (a->kind == GW_MS_ACT_DISABLE) {
            for (k = 0; k < n; ++k) if (_stricmp(ids[k], a->id) == 0) { memmove(ids[k], ids[k + 1], (size_t) (n - k - 1) * GW_MS_ID); --n; break; }
        }
    }
    for (i = 0; i < ms_s.plan.n; ++i) {
        const gw_ms_action *a = &ms_s.plan.a[i];
        char folder[GW_MS_ID];
        const char *id = a->id;
        if (a->kind == GW_MS_ACT_DOWNLOAD) {
            if (!gw_ms_nc_folder(a->post, folder, (int) sizeof folder)) { ms_fail("A downloaded mod is not in the mods folder"); return; }
            id = folder;
        } else if (a->kind != GW_MS_ACT_ENABLE) {
            continue;
        }
        if (!ms_has(ids, n, id) && n < GW_MS_MAX * 2) snprintf(ids[n++], GW_MS_ID, "%s", id);
    }
    for (i = 0; i < n; ++i) pp[i] = ids[i];
    if (gw_ms_write_enabled(ms_dir(), pp, n) != 0) { ms_fail("Could not write the mod list"); return; }
    gw_ms_j_set(ms_dir(), GW_MS_J_APPLIED, 0, ms_now());
    gw_log("modsync: applied - %d mods enabled for the next boot", n);
    {
        const char *crash = getenv("MELEE_MODSYNC_CRASH_AT");
        if (crash != NULL && strcmp(crash, "applied") == 0) { gw_log("modsync: TEST - dying after APPLIED"); TerminateProcess(GetCurrentProcess(), 3); }
    }
    ms_s.phase = GW_MS_PH_RELAUNCH;
    gw_ms_j_set(ms_dir(), GW_MS_J_LAUNCHING, 0, ms_now());
    {
        const char *nr = getenv("MELEE_MODSYNC_NO_RELAUNCH");
        if (nr != NULL && nr[0] == '1') { ms_say("Restart the game to finish (test: relaunch skipped)"); return; }
    }
    {
        char exe[MAX_PATH];
        STARTUPINFOA si;
        PROCESS_INFORMATION pi;
        char *cmd = _strdup(GetCommandLineA());
        GetModuleFileNameA(NULL, exe, sizeof exe);
        memset(&si, 0, sizeof si);
        si.cb = sizeof si;
        memset(&pi, 0, sizeof pi);
        ms_say("Restarting the game with the host's mods");
        if (cmd != NULL && CreateProcessA(exe, cmd, NULL, NULL, FALSE, 0, NULL, NULL, &si, &pi)) {
            CloseHandle(pi.hThread);
            CloseHandle(pi.hProcess);
            gw_log("modsync: relaunched; this process exits");
            ExitProcess(0);
        }
        free(cmd);
        ms_fail("Could not restart the game");
    }
}

void gw_ms_tick(void) {
    int i, pending = 0;
    gw_ms_nc_pump();
    if (ms_s.phase == GW_MS_PH_DOWNLOADING) {
        for (i = 0; i < ms_s.nposts; ++i) {
            char why[120] = { 0 };
            int pct = 0, st = gw_ms_nc_state(ms_s.posts[i], &pct, why, (int) sizeof why);
            if (st < 0) { char m[160]; snprintf(m, sizeof m, "A download failed: %s", why[0] ? why : "unknown"); ms_fail(m); return; }
            if (st == 0) { ++pending; ms_say("Downloading %d of %d (%d%%)", i + 1, ms_s.nposts, pct); break; }
        }
        if (pending == 0) ms_s.phase = GW_MS_PH_APPLYING;
    } else if (ms_s.phase == GW_MS_PH_APPLYING) {
        ms_apply_and_relaunch();
    }
}

void gw_ms_cancel(void) {
    if (ms_s.phase == GW_MS_PH_DOWNLOADING || ms_s.phase == GW_MS_PH_APPLYING || ms_s.phase == GW_MS_PH_FAILED) {
        gw_ms_j_restore(ms_dir());
        ms_s.phase = GW_MS_PH_IDLE;
        ms_say("Mod sync cancelled - your mods are as they were");
    }
}

int gw_ms_keep(void) {
    if (!ms_s.active) return -1;
    gw_ms_j_keep(ms_dir());
    ms_s.active = 0;
    ms_say("Keeping these mods");
    return 0;
}

void gw_ms_end_session(void) {
    if (!ms_s.active) return;
    gw_ms_j_restore(ms_dir());
    ms_s.active = 0;
    ms_say("You left the room - your own mods are back on the next time the game starts");
}
