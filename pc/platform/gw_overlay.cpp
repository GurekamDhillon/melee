/* gw_overlay.cpp - see gw_overlay.h.
 *
 * This is the port's only C++ translation unit. It is C++ because Dear ImGui is, and ImGui is
 * already linked into the exe (extern\imgui.lib in melee_link_libs.rsp) and already driven by
 * Aurora: aurora::begin_frame() calls imgui::new_frame() and aurora::end_frame() calls
 * imgui::freeze(), so anything submitted between the two is composited over the game's output.
 * That is the whole reason this works without touching the GX path.
 */
#include "gw_overlay.h"

#include <imgui.h>

/* Only for GetAsyncKeyState: the overlay needs a key that does not go through the game's
 * pad path, and this port is Windows-only. */
#include <windows.h>

#include <chrono>
#include <cstdarg>
#include <cmath>
#include <cstdio>
#include <cstdlib>
#include <cstring>

extern "C" {
void gw_log(const char *fmt, ...);
}

namespace {

/* Everything here is touched only from the game thread: the DVD shims run inside the game's own
 * call stack, and gw_Overlay_Draw runs from VIWaitForRetrace on that same thread. */
struct State {
  bool         enabled = true;
  bool         checked_env = false;
  bool         content_seen = false;  /* the game has drawn real geometry at least once */
  uint32_t     files = 0;
  uint64_t     bytes = 0;
  double       last_read_at = 0.0;
  double       last_geometry_at = 0.0;  /* when the game last DREW something */
  uint32_t     last_prims = 0;
  double       first_frame_at = 0.0;
  char         current[96] = {0};
  /* F9 panel + toast. Pad state is mirrored in rather than read back out of game memory: the
   * PADStatus array is big-endian guest memory and the overlay has no business swapping it. */
  bool         panel = false;
  bool         f9_was_down = false;
  bool         f10_was_down = false;
  int          reshow = 0;  /* F10 presses, read by the game */
  char         toast[96] = {0};
  double       toast_at = 0.0;
  unsigned     pad_buttons[4] = {0, 0, 0, 0};
  int          pad_sx[4] = {0, 0, 0, 0};
  int          pad_sy[4] = {0, 0, 0, 0};
  uint32_t     frames = 0;
};

State g;

/* A context-free clock. This must NOT be ImGui::GetTime(): the DVD hooks below run in the
 * headless --test harness too, where Aurora never initialises and there is no ImGui context, and
 * calling into ImGui there took out every test that reads a file from the disc. */
double now_seconds() {
  using clock = std::chrono::steady_clock;
  static const clock::time_point t0 = clock::now();
  return std::chrono::duration<double>(clock::now() - t0).count();
}

/* True only once Aurora has stood ImGui up; false for the whole headless test run. */
bool imgui_ready() { return ImGui::GetCurrentContext() != nullptr; }

bool overlay_enabled() {
  if (!g.checked_env) {
    const char *v = std::getenv("MELEE_OVERLAY");
    g.enabled = !(v != nullptr && v[0] == '0');
    g.checked_env = true;
  }
  return g.enabled;
}

/* A loading screen belongs on frames where the game is NOT DRAWING. Keying it on "a disc read
 * happened recently" was wrong: a match streams files continuously, so the overlay sat on top of
 * live gameplay - visible in _build/runs/mcvs/frame_22s.png, over a running Meta Crystal match.
 *
 * So: show until the game first draws, and afterwards only when geometry has stopped arriving AND
 * the disc is busy. That is what a load actually looks like from here, and normal play can never
 * satisfy it. */
bool should_show() {
  double t;
  if (!g.content_seen) {
    return true;
  }
  t = now_seconds();
  return (t - g.last_geometry_at) > 0.40 && g.last_read_at != 0.0 &&
         (t - g.last_read_at) < 1.00;
}

void human_bytes(char *out, size_t n, uint64_t b) {
  if (b >= 1024u * 1024u) {
    std::snprintf(out, n, "%.1f MB", (double)b / (1024.0 * 1024.0));
  } else {
    std::snprintf(out, n, "%.0f KB", (double)b / 1024.0);
  }
}

/* MELEE_RUN_LABEL: a caption naming what this instance is testing, drawn every frame.
 *
 * The sweep runs four games side by side now, and from across the room they are four identical
 * windows. Two runs on the same stage testing different move types look like a repeat, and a
 * genuine repeat looks like progress. The harness knows exactly what each one is - the tag, the
 * disc, the unit, the stage, the fighters - so it should say so on the window rather than only
 * in a log nobody reads while watching. Read once; the value never changes within a run. */
/* The label, in order: set at runtime (console "label <text>", gd.label), MELEE_RUN_LABEL, else
 * derived from where the exe runs - a sandbox under _build\agents\<lane>\runs\<name>\ reads
 * "<lane> / <name>", _build\runs\<name>\ reads "main / <name>". A real install shows nothing. */
char g_label[160] = {0};
int g_label_state = 0; /* 0 = not decided, 1 = decided (g_label may be empty) */
int g_label_serial = 0; /* bumps when the label changes: the window title follows */

void derive_label_from_exe() {
  wchar_t wpath[MAX_PATH];
  char path[MAX_PATH * 2];
  DWORD n = GetModuleFileNameW(nullptr, wpath, MAX_PATH);
  if (n == 0 || n >= MAX_PATH) {
    return;
  }
  WideCharToMultiByte(CP_UTF8, 0, wpath, -1, path, (int) sizeof path, nullptr, nullptr);
  for (char *c = path; *c != '\0'; ++c) {
    if (*c == '/') *c = '\\';
  }
  /* find "\_build\" case-insensitively */
  char lower[MAX_PATH * 2];
  std::snprintf(lower, sizeof lower, "%s", path);
  for (char *c = lower; *c != '\0'; ++c) {
    if (*c >= 'A' && *c <= 'Z') *c = (char) (*c + 32);
  }
  const char *b = std::strstr(lower, "\\_build\\");
  if (b == nullptr) {
    return;
  }
  const char *rest = path + (b - lower) + 8; /* after "\_build\" */
  char lane[64] = "main", name[96] = "";
  if (_strnicmp(rest, "agents\\", 7) == 0) {
    const char *l = rest + 7, *e = std::strchr(l, '\\');
    if (e == nullptr || _strnicmp(e, "\\runs\\", 6) != 0) return;
    std::snprintf(lane, sizeof lane, "%.*s", (int) (e - l), l);
    rest = e + 6;
  } else if (_strnicmp(rest, "runs\\", 5) == 0) {
    rest += 5;
  } else {
    return;
  }
  const char *e = std::strchr(rest, '\\');
  if (e == nullptr) return; /* the exe itself must sit inside the sandbox folder */
  std::snprintf(name, sizeof name, "%.*s", (int) (e - rest), rest);
  if (name[0] != '\0') {
    std::snprintf(g_label, sizeof g_label, "%s / %s", lane, name);
  }
}

const char *run_label() {
  if (g_label_state == 0) {
    g_label_state = 1;
    const char *env = std::getenv("MELEE_RUN_LABEL");
    if (env != nullptr && env[0] != '\0') {
      std::snprintf(g_label, sizeof g_label, "%s", env);
    } else {
      derive_label_from_exe();
    }
    ++g_label_serial;
  }
  return g_label[0] != '\0' ? g_label : nullptr;
}

BOOL CALLBACK title_cb(HWND w, LPARAM title) {
  if (IsWindowVisible(w)) SetWindowTextA(w, (const char *) title);
  return TRUE;
}

/* The window title carries the label too ("Melee PC - charlie / c3live"). */
void apply_title() {
  static int applied = 0;
  run_label();
  if (applied == g_label_serial) return;
  char title[200];
  if (g_label[0] != '\0') {
    std::snprintf(title, sizeof title, "Melee PC - %s", g_label);
  } else {
    std::snprintf(title, sizeof title, "Melee PC");
  }
  EnumThreadWindows(GetCurrentThreadId(), title_cb, (LPARAM) title);
  applied = g_label_serial;
}

} // namespace

extern "C" void gw_Overlay_NoteFile(const char *path) {
  if (path == nullptr) {
    return;
  }
  ++g.files;
  g.last_read_at = now_seconds();
  /* Keep the leaf name; the full disc paths are long and the tail is the informative part. */
  const char *leaf = std::strrchr(path, '/');
  leaf = (leaf != nullptr) ? leaf + 1 : path;
  std::snprintf(g.current, sizeof g.current, "%s", leaf);
}

extern "C" void gw_Overlay_NoteBytes(uint32_t bytes) {
  g.bytes += bytes;
  g.last_read_at = now_seconds();
}

extern "C" void gw_Overlay_NoteContent(uint32_t prims) {
  if (prims != g.last_prims) {
    g.last_prims = prims;
    g.last_geometry_at = now_seconds();
  }
  if (!g.content_seen && prims > 0) {
    g.content_seen = true;
    gw_log("gw: overlay: first geometry at %.2fs after %u files / %llu bytes - overlay stands down",
           now_seconds() - g.first_frame_at, g.files, (unsigned long long)g.bytes);
  }
}

extern "C" void gw_Overlay_Draw(void) {
  if (!overlay_enabled() || !imgui_ready()) {
    return;
  }
  if (g.first_frame_at == 0.0) {
    g.first_frame_at = now_seconds();
  }
  if (!should_show()) {
    return;
  }

  const ImGuiIO &io = ImGui::GetIO();
  const float w = 420.0f;
  const float h = 104.0f;
  const ImVec2 size = io.DisplaySize;

  ImGui::SetNextWindowPos(ImVec2((size.x - w) * 0.5f, size.y - h - 48.0f), ImGuiCond_Always);
  ImGui::SetNextWindowSize(ImVec2(w, h), ImGuiCond_Always);
  ImGui::SetNextWindowBgAlpha(0.72f);

  const ImGuiWindowFlags flags = ImGuiWindowFlags_NoDecoration | ImGuiWindowFlags_NoMove |
                                 ImGuiWindowFlags_NoSavedSettings | ImGuiWindowFlags_NoInputs |
                                 ImGuiWindowFlags_NoFocusOnAppearing | ImGuiWindowFlags_NoNav;

  if (ImGui::Begin("##gw_loading", nullptr, flags)) {
    const double elapsed = now_seconds() - g.first_frame_at;

    ImGui::TextUnformatted("LOADING");
    ImGui::Separator();

    char bytes_str[32];
    human_bytes(bytes_str, sizeof bytes_str, g.bytes);
    ImGui::Text("%u files   %s   %.1fs", g.files, bytes_str, elapsed);

    if (g.current[0] != '\0') {
      ImGui::TextDisabled("%s", g.current);
    } else {
      ImGui::TextDisabled("starting up");
    }

    /* Indeterminate bar: the total is genuinely unknown on a first boot, so this animates to show
     * liveness instead of inventing a percentage. */
    const float t = (float)std::fmod(elapsed, 1.2) / 1.2f;
    const float frac = 0.18f;
    float x0 = t * (1.0f + frac) - frac;
    float x1 = x0 + frac;
    if (x0 < 0.0f) { x0 = 0.0f; }
    if (x1 > 1.0f) { x1 = 1.0f; }

    const ImVec2 p = ImGui::GetCursorScreenPos();
    const float bar_w = ImGui::GetContentRegionAvail().x;
    const float bar_h = 6.0f;
    ImDrawList *dl = ImGui::GetWindowDrawList();
    dl->AddRectFilled(p, ImVec2(p.x + bar_w, p.y + bar_h), IM_COL32(255, 255, 255, 40), 3.0f);
    dl->AddRectFilled(ImVec2(p.x + bar_w * x0, p.y), ImVec2(p.x + bar_w * x1, p.y + bar_h),
                      IM_COL32(255, 255, 255, 190), 3.0f);
    ImGui::Dummy(ImVec2(bar_w, bar_h));
  }
  ImGui::End();
}

extern "C" void gw_Overlay_Toast(const char *msg) {
  if (msg == nullptr) {
    return;
  }
  std::snprintf(g.toast, sizeof g.toast, "%s", msg);
  g.toast_at = now_seconds();
}

extern "C" void gw_Overlay_NotePad(int chan, unsigned buttons, int sx, int sy) {
  if (chan < 0 || chan > 3) {
    return;
  }
  g.pad_buttons[chan] = buttons;
  g.pad_sx[chan] = sx;
  g.pad_sy[chan] = sy;
}

/* ---- the F9 panel, the toast and the run label: host state, NATIVE drawing -----------------
 *
 * These three used to be ImGui windows. They are now drawn by the game itself, with DevText -
 * the engine's own screen-space font and panels, the same thing the loading screen uses - from
 * src/melee/gm/gmscene.c (mnOverlay_*). What stays here is only what the game cannot know: the
 * keyboard, the environment, the DVD counters and the mirrored pad state.
 *
 * The boundary is deliberately narrow. The game pulls, once a frame, through getters that return
 * a scalar or a pointer to a host string. Nothing is written through a pointer the game passes
 * in: a game TU's own locals are big-endian in memory (gwtool swaps every access it makes,
 * including to its own stack), so a host out-parameter would arrive byte-reversed. Strings are
 * bytes, so a const char* is safe to hand across as-is.
 *
 * The lines are formatted here, not in the game: they need %.1f and %04X, and the host's libc
 * is the one that is known to do those correctly. */

namespace {

enum { PANEL_COLS = 44, PANEL_MAX_LINES = 20 };

char panel_lines[PANEL_MAX_LINES][PANEL_COLS + 1];
int panel_dim[PANEL_MAX_LINES];
int panel_count = 0;

void panel_add(int dim, const char *fmt, ...) {
  if (panel_count >= PANEL_MAX_LINES) {
    return;
  }
  va_list ap;
  va_start(ap, fmt);
  std::vsnprintf(panel_lines[panel_count], sizeof panel_lines[0], fmt, ap);
  va_end(ap);
  panel_dim[panel_count] = dim;
  ++panel_count;
}

void panel_rule() {
  panel_add(1, "%.*s", PANEL_COLS, "--------------------------------------------");
}

/* DevText does not wrap on words, so a long MELEE_SCENE is cut into PANEL_COLS-wide rows here,
 * the first behind a "scene   " label and the rest indented under it. */
void panel_wrapped(const char *label, const char *text) {
  const int indent = 8;
  const int width = PANEL_COLS - indent;
  size_t len = std::strlen(text);
  size_t at = 0;
  do {
    size_t n = len - at < (size_t)width ? len - at : (size_t)width;
    panel_add(0, "%-*s%.*s", indent, at == 0 ? label : "", (int)n, text + at);
    at += n;
  } while (at < len);
}

void build_panel() {
  const char *scene = std::getenv("MELEE_SCENE");
  const char *pad = std::getenv("MELEE_PAD_SCRIPT");

  panel_count = 0;
  panel_add(0, "GD's Melee                          F9");
  panel_rule();
  panel_add(0, "frames  %u", g.frames);
  panel_add(0, "files   %u   %.1f MB", g.files, (double)g.bytes / (1024.0 * 1024.0));
  if (g.current[0] != '\0') {
    panel_add(1, "last    %s", g.current);
  }
  panel_rule();
  panel_wrapped("scene", scene != nullptr ? scene : "(none - booted to the menu)");
  if (pad != nullptr) {
    const char *leaf = std::strrchr(pad, '\\');
    panel_add(1, "pad     %s", leaf != nullptr ? leaf + 1 : pad);
  }
  panel_rule();
  for (int c = 0; c < 4; ++c) {
    panel_add(0, "p%d  btn %04X  stick %4d,%4d", c + 1, g.pad_buttons[c], g.pad_sx[c],
              g.pad_sy[c]);
  }
  panel_rule();
  panel_add(1, "L+R+Y+X+Start recalibrates on release");
}

} // namespace

/* Once per presented frame, from VIWaitForRetrace. Counts frames, edge-detects F9 (holding it
 * toggles once, not 60 times a second) and refreshes the panel text while it is open. Draws
 * nothing - the game draws. */
extern "C" void gw_Console_Draw(void); /* gw_console.cpp: the console and script drawing */

extern "C" void gw_Overlay_DrawPanel(void) {
  gw_Console_Draw(); /* independent of MELEE_OVERLAY */
  apply_title();
  if (!overlay_enabled()) {
    return;
  }
  ++g.frames;

  /* MELEE_OVERLAY_PANEL=1 opens the panel on the first frame. F9 is a keypress, and an
   * unattended capture cannot press it - which is exactly when the panel is worth having. */
  static bool panel_env = false;
  if (!panel_env) {
    const char *pv = std::getenv("MELEE_OVERLAY_PANEL");
    panel_env = true;
    if (pv != nullptr && pv[0] != 0 && pv[0] != '0') {
      g.panel = true;
    }
  }

  const bool down = (GetAsyncKeyState(VK_F9) & 0x8000) != 0;
  if (down && !g.f9_was_down) {
    g.panel = !g.panel;
    gw_log("gw: overlay: F9 panel %s", g.panel ? "open" : "closed");
  }
  g.f9_was_down = down;

  /* F10 puts every overlay box back on screen (mnOverlay_Frame re-shows any that fell out
   * of DevText's draw list) and opens the panel. */
  const bool down10 = (GetAsyncKeyState(VK_F10) & 0x8000) != 0;
  if (down10 && !g.f10_was_down) {
    ++g.reshow;
    g.panel = true;
    gw_log("gw: overlay: F10 - overlay put back on screen");
  }
  g.f10_was_down = down10;

  if (g.panel) {
    build_panel();
  }
}

/* Frame-rate readout (MELEE_SHOW_FPS / video.cfg show_fps, drawn by shim_vi.c every presented
 * frame, replays included): a small box in the top-left corner. Host-side ImGui, so it shows
 * what is actually presented rather than what the game drew. */
extern "C" void gw_Overlay_DrawStats(const char *text) {
  if (text == nullptr || !imgui_ready()) {
    return;
  }
  const ImGuiWindowFlags flags = ImGuiWindowFlags_NoDecoration | ImGuiWindowFlags_NoMove |
                                 ImGuiWindowFlags_NoSavedSettings | ImGuiWindowFlags_NoInputs |
                                 ImGuiWindowFlags_NoFocusOnAppearing | ImGuiWindowFlags_NoNav |
                                 ImGuiWindowFlags_AlwaysAutoResize;
  ImGui::SetNextWindowPos(ImVec2(8.0f, 8.0f), ImGuiCond_Always);
  ImGui::SetNextWindowBgAlpha(0.55f);
  if (ImGui::Begin("##gw_stats", nullptr, flags)) {
    ImGui::TextUnformatted(text);
  }
  ImGui::End();
}

/* ---- getters for the native overlay. gwtool prefixes every game symbol with gw_, so the game
 * declares these without it (extern const char* Overlay_GetRunLabel(void); and so on). ---- */

extern "C" const char *gw_Overlay_GetRunLabel(void) {
  return overlay_enabled() ? run_label() : nullptr;
}

/* How many times F10 has been pressed; the game re-shows its boxes when this changes. */
extern "C" int gw_Overlay_GetReshowCount(void) { return overlay_enabled() ? g.reshow : 0; }

/* The toast text while it is live (2.5 s), else NULL. */
extern "C" const char *gw_Overlay_GetToast(void) {
  if (!overlay_enabled() || g.toast[0] == '\0' || (now_seconds() - g.toast_at) >= 2.5) {
    return nullptr;
  }
  return g.toast;
}

extern "C" int gw_Overlay_GetPanelOpen(void) { return overlay_enabled() && g.panel ? 1 : 0; }

extern "C" int gw_Overlay_GetPanelLineCount(void) { return g.panel ? panel_count : 0; }

extern "C" const char *gw_Overlay_GetPanelLine(int i) {
  return (i >= 0 && i < panel_count) ? panel_lines[i] : nullptr;
}

/* 1 for the secondary lines the ImGui panel drew with TextDisabled - drawn grey. */
extern "C" int gw_Overlay_GetPanelLineDim(int i) {
  return (i >= 0 && i < panel_count) ? panel_dim[i] : 0;
}

/* Set the run label at runtime (console "label <text>", Lua gd.label): scripted tests say what
 * step they are on. An empty text clears it. */
extern "C" void gw_Overlay_SetRunLabel(const char *text) {
  run_label();
  std::snprintf(g_label, sizeof g_label, "%s", text != nullptr ? text : "");
  ++g_label_serial;
}
