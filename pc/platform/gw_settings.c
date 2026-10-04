/* gw_settings.c - the port's own settings file (settings.cfg beside the exe).
 *
 * The in-game SETTINGS screens (gmfrontend_settings.inc) keep what has no other home here: the
 * player's name, the matchmaking server, online defaults (input delay, stage list), unlock
 * everything, the input device. Settings that already persist elsewhere keep their own files -
 * video.cfg (gw_Video_*), audio.cfg (gw_Audio_*), mods/enabled.txt (gw_Mods_*), and the memory
 * card for the game's own options and rules.
 *
 * Format: one `key=value` per line, '#' comments; unknown keys are kept on save. Environment
 * variables still win over a saved value wherever a feature has one (MELEE_UNLOCK_ALL,
 * MELEE_INPUT, MELEE_NETPLAY_SERVER), so scripted tests are never steered by a saved setting.
 * Game code calls these without the gw_ prefix (gwtool adds it); strings cross as bytes. */
#include "gw.h"
#include "gw_test.h"
#include "gw_view_math.h"
#include "gw_controls_model.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#ifdef _WIN32
#include <windows.h>
#else
#include "gw_compat_linux.h"
#endif

#define ST_MAX 512
#define ST_KEY 32
#define ST_VAL 512

static struct {
    int loaded;
    int n;
    char key[ST_MAX][ST_KEY];
    char val[ST_MAX][ST_VAL];
    char path[MAX_PATH];
} st;
static int st_save_ok = 1;
int gw_Settings_LastSaveOk(void) { return st_save_ok; }

static const char *st_path(void) {
    if (st.path[0] == '\0') {
        const char *e = getenv("MELEE_SETTINGS_CFG");
        if (e != NULL && e[0] != '\0') {
            snprintf(st.path, sizeof st.path, "%s", e);
        } else {
            DWORD k = GetModuleFileNameA(NULL, st.path, sizeof st.path);
            char *slash = k > 0 && k < sizeof st.path ? gw_path_separator(st.path) : NULL;
            if (slash != NULL) {
                snprintf(slash + 1, sizeof st.path - (size_t) (slash + 1 - st.path), "settings.cfg");
            } else {
                snprintf(st.path, sizeof st.path, "settings.cfg");
            }
        }
    }
    return st.path;
}

static void st_trim(char *s) {
    char *e = s + strlen(s);
    while (e > s && (e[-1] == '\n' || e[-1] == '\r' || e[-1] == ' ' || e[-1] == '\t')) *--e = '\0';
}

static void st_load(void) {
    FILE *f;
    char line[ST_KEY + ST_VAL + 4];
    if (st.loaded) return;
    st.loaded = 1;
    st.n = 0;
    f = fopen(st_path(), "r");
    if (f == NULL) return;
    while (fgets(line, sizeof line, f) != NULL && st.n < ST_MAX) {
        char *eq, *k = line;
        while (*k == ' ' || *k == '\t') k++;
        if (*k == '#' || *k == '\0' || *k == '\n' || *k == '\r') continue;
        eq = strchr(k, '=');
        if (eq == NULL) continue;
        *eq = '\0';
        st_trim(k);
        st_trim(eq + 1);
        snprintf(st.key[st.n], ST_KEY, "%s", k);
        snprintf(st.val[st.n], ST_VAL, "%s", eq + 1);
        st.n++;
    }
    fclose(f);
    gw_log("settings: %d value(s) from %s", st.n, st_path());
}

static int st_find(const char *key) {
    int i;
    st_load();
    for (i = 0; i < st.n; ++i) {
        if (strcmp(st.key[i], key) == 0) return i;
    }
    return -1;
}

static void st_save(void) {
    FILE *f = fopen(st_path(), "w");
    int i;
    st_save_ok = 0;
    if (f == NULL) {
        gw_log("settings: cannot write %s", st_path());
        return;
    }
    fprintf(f, "# GD's Melee settings (written by the SETTINGS screens; environment variables win)\n");
    for (i = 0; i < st.n; ++i) fprintf(f, "%s=%s\n", st.key[i], st.val[i]);
    {
        int failed = ferror(f);
        if (fclose(f) != 0) failed = 1;
        st_save_ok = !failed;
        if (failed) gw_log("settings: incomplete write to %s", st_path());
    }
}

/* The saved string for `key` (copied into out), or `dflt` when there is none. 1 = found. */
int gw_Settings_Str(const char *key, char *out, int cap, const char *dflt) {
    int i = st_find(key);
    const char *v = i >= 0 ? st.val[i] : dflt;
    int k = 0;
    if (out == NULL || cap <= 0) return i >= 0;
    for (; v != NULL && v[k] != '\0' && k < cap - 1; ++k) out[k] = v[k];
    out[k] = '\0';
    return i >= 0;
}

void gw_Settings_SetStr(const char *key, const char *value) {
    int i = st_find(key);
    if (i < 0) {
        if (st.n >= ST_MAX) { st_save_ok = 0; gw_log("settings: key capacity reached"); return; }
        i = st.n++;
        snprintf(st.key[i], ST_KEY, "%s", key);
    }
    snprintf(st.val[i], ST_VAL, "%s", value != NULL ? value : "");
    st_save();
}

int gw_Settings_Int(const char *key, int dflt) {
    int i = st_find(key);
    return i >= 0 && st.val[i][0] != '\0' ? atoi(st.val[i]) : dflt;
}

void gw_Settings_SetInt(const char *key, int value) {
    char b[24];
    snprintf(b, sizeof b, "%d", value);
    gw_Settings_SetStr(key, b);
}

/* Widescreen (16:9 Hor+). MELEE_WIDESCREEN wins over the saved "widescreen" key so scripted runs
 * and lanes can force it. Without either override, use the window (desktop before creation)
 * when wider than 4:3; never write this automatic choice into settings.cfg.
 * Game code calls Widescreen_Enabled() across the gw.h boundary.
 * Specification and credits: _research/widescreen.md. */
int gw_Widescreen_Enabled(void) {
    const char *v = getenv("MELEE_WIDESCREEN");
    extern int gw_View_DefaultWide(void);
    int dflt;
    if (v != NULL && v[0] != '\0') return atoi(v) != 0;
    dflt = gw_View_DefaultWide();
#ifdef _WIN32
    if (dflt < 0) dflt = GetSystemMetrics(SM_CXSCREEN) * 3 > GetSystemMetrics(SM_CYSCREEN) * 4;
#endif
    return gw_Settings_Int("widescreen", dflt > 0) != 0;
}

/* Every saved key=value on one line for a crash report, leaving out what can identify the
 * player (name) or a private address (server). Returns the count written. */
int gw_Settings_Summary(char *out, int cap) {
    int i, n = 0, k = 0;
    if (out == NULL || cap <= 0) return 0;
    out[0] = '\0';
    st_load();
    for (i = 0; i < st.n; ++i) {
        int w;
        if (strcmp(st.key[i], "name") == 0 || strcmp(st.key[i], "server") == 0) continue;
        w = snprintf(out + n, (size_t) (cap - n), "%s%s=%s", k ? " " : "", st.key[i], st.val[i]);
        if (w < 0 || n + w >= cap) break;
        n += w;
        k++;
    }
    return k;
}

/* Typing into a text setting (the SETTINGS screens' name / server rows): letters (Shift for
 * capitals), digits, space . - _ : ; Backspace deletes before the caret. Keys count only while
 * this window has focus, and gd.key reads nothing meanwhile (gw_TextEntryUntil). Returns 0 nothing, 1 changed, 2 Enter, 3 Escape. */
extern int gw_TextEntryUntil;
static int st_caret;
/* The caret after the last gw_Settings_TextKeys (an int does not cross back through a pointer:
   game memory is byte-swapped, a char buffer is not). */
int gw_Settings_TextCaret(void) { return st_caret; }
int gw_Settings_TextKeys(char *buf, int cap, int caret_in) {
    int *caret = &st_caret;
    static unsigned char was[256];
    HWND fg = GetForegroundWindow();
    DWORD pid = 0;
    int vk, r = 0, n = (int) strlen(buf);
    int shift = (GetAsyncKeyState(VK_SHIFT) & 0x8000) != 0;
    if (fg != NULL) GetWindowThreadProcessId(fg, &pid);
    st_caret = caret_in;
    if (pid != GetCurrentProcessId()) return 0;
    gw_TextEntryUntil = (int) GetTickCount() + 150;
    if (*caret < 0) *caret = 0;
    if (*caret > n) *caret = n;
    for (vk = 0x08; vk < 0xC0; ++vk) {
        int down = (GetAsyncKeyState(vk) & 0x8000) != 0, edge = down && !was[vk];
        char ch = 0;
        was[vk] = (unsigned char) down;
        if (!edge || r >= 2) continue;
        if (vk == VK_RETURN) { r = 2; continue; }
        if (vk == VK_ESCAPE) { r = 3; continue; }
        if (vk == VK_BACK) {
            if (*caret > 0) {
                memmove(buf + *caret - 1, buf + *caret, (size_t) (n - *caret + 1));
                (*caret)--;
                n--;
                r = 1;
            }
            continue;
        }
        if (vk >= 'A' && vk <= 'Z') ch = (char) (shift ? vk : vk + 32);
        else if (vk >= '0' && vk <= '9' && !shift) ch = (char) vk;
        else if (vk == VK_SPACE) ch = ' ';
        else if (vk == VK_OEM_PERIOD) ch = '.';
        else if (vk == VK_OEM_MINUS) ch = shift ? '_' : '-';
        else if (vk == VK_OEM_1) ch = shift ? ':' : ';';
        if (ch != 0 && n < cap - 1) {
            memmove(buf + *caret + 1, buf + *caret, (size_t) (n - *caret + 1));
            buf[(*caret)++] = ch;
            n++;
            r = 1;
        }
    }
    return r;
}

/* ---- tests ------------------------------------------------------------------------------------ */

static int test_settings_roundtrip(void) {
    char path[MAX_PATH], b[64];
    int rc = 0;
    GetTempPathA(sizeof path, path);
    strncat(path, "gw_settings_test.cfg", sizeof path - strlen(path) - 1);
    DeleteFileA(path);
    memset(&st, 0, sizeof st);
    snprintf(st.path, sizeof st.path, "%s", path);
    gw_Settings_SetStr("name", "GD");
    gw_Settings_SetInt("delay", 3);
    gw_Settings_SetInt("delay", 4);
    gw_Settings_SetInt("widescreen", 1);
    memset(&st, 0, sizeof st); /* read it back from the file */
    snprintf(st.path, sizeof st.path, "%s", path);
    if (!gw_Settings_Str("name", b, sizeof b, "") || strcmp(b, "GD") != 0) {
        gw_test_fail("name did not round-trip");
        rc = 1;
    }
    if (gw_Settings_Int("delay", 0) != 4 || gw_Settings_Int("missing", 7) != 7) {
        gw_test_fail("ints did not round-trip");
        rc = 1;
    }
    if (gw_Settings_Int("widescreen", 0) != 1) {
        gw_test_fail("widescreen did not persist");
        rc = 1;
    }
    gw_Settings_SetInt("widescreen", 0);
    memset(&st, 0, sizeof st);
    snprintf(st.path, sizeof st.path, "%s", path);
    if (gw_Settings_Int("widescreen", 1) != 0) {
        gw_test_fail("widescreen off did not persist");
        rc = 1;
    }
    if (st.n != 3) {
        gw_test_fail("a key was written twice");
        rc = 1;
    }
    {
        GcMap before, after;
        char encoded[512], loaded[512];
        int i;
        gc_map_default(&before);
        after = before;
        /* Migration: missing mappings preserve defaults and old user keys. */
        if (gw_Settings_Str("ctl00_p0_map", loaded, sizeof loaded, "") ||
            gc_map_decode(&after, loaded) || memcmp(&before, &after, sizeof before)) {
            gw_test_fail("old settings changed the default mapping"); rc = 1;
        }
        for (i = 0; i < GC_PHYSICAL; ++i) before.dest[i] = GC_DEST_MASK;
        before.swap = 1; before.analog_off = 1; before.shield = 80;
        before.dz[0] = 10; before.dz[1] = 30; before.rumble = 0;
        gc_map_encode(&before, encoded, sizeof encoded);
        gw_Settings_SetStr("ctl00_id", "gc-adapter-port-1");
        gw_Settings_SetStr("ctl00_p0_map", encoded);
        gw_Settings_SetStr("ctl00_p0_name", "Tournament");
        memset(&st, 0, sizeof st);
        snprintf(st.path, sizeof st.path, "%s", path);
        if (!gw_Settings_Str("ctl00_p0_map", loaded, sizeof loaded, "") ||
            !gc_map_decode(&after, loaded) || memcmp(&before, &after, sizeof before) ||
            gw_Settings_Int("delay", 0) != 4) {
            gw_test_fail("long controller profile or legacy values failed roundtrip"); rc = 1;
        }
        if (!gw_Settings_Str("ctl00_p0_name", loaded, sizeof loaded, "") || strcmp(loaded, "Tournament")) {
            gw_test_fail("controller profile name failed roundtrip"); rc = 1;
        }
    }
    DeleteFileA(path);
    memset(&st, 0, sizeof st);
    return rc;
}

#include "../tests/view_aspect_cases.h"
static int test_view_projection_rect(void) {
    int rc = gw_view_aspect_cases();
    if (rc) gw_test_fail("view aspect/projection/rectangle case %d", rc);
    return rc != 0;
}

static int test_view_canvas(void) {
    if (gw_view_default_wide(800, 600) != 0 || gw_view_default_wide(1280, 720) != 1 ||
        gw_view_default_wide(2520, 1080) != 1 || gw_view_default_wide(600, 800) != 0 ||
        gw_view_default_wide(0, 0) != -1) {
        gw_test_fail("automatic widescreen aspect default");
        return 1;
    }
    const float windows[5][3] = {
        {800, 600, 640}, {1280, 720, 853.333333f},
        {2520, 1080, 1120}, {2520, 1080, 640}, {0, 0, 640}
    };
    int i;
    for (i = 0; i < 5; i++) {
        float width = 480.0f * gw_view_aspect(windows[i][0], windows[i][1], i != 3);
        float left = gw_view_left(width), scale, ox, oy, px, x;
        float error = width - windows[i][2];
        if (error < -0.001f || error > 0.001f) {
            gw_test_fail("view canvas case %d width %.3f", i, width);
            return 1;
        }
        gw_view_map(windows[i][0], windows[i][1], width, &scale, &ox, &oy);
        px = (91.5f - left) * scale + ox;
        x = (px - ox) / scale + left;
        error = x - 91.5f;
        if (error < -0.001f || error > 0.001f || left + width * 0.5f != 320.0f) {
            gw_test_fail("view canvas case %d mouse/centering", i);
            return 1;
        }
    }
    return 0;
}

void gw_settings_tests_register(void) {
    gw_test_register("view_canvas", test_view_canvas);
    gw_test_register("view_projection_rect", test_view_projection_rect);
    gw_test_register("settings_roundtrip", test_settings_roundtrip);
}
