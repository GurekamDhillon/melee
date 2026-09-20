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

#include <chrono>
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
  double       first_frame_at = 0.0;
  char         current[96] = {0};
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

/* Show while the game has not yet drawn anything, and afterwards for a short tail whenever the
 * disc is being read - so in-game loads get the overlay too, not just boot. */
bool should_show() {
  if (!g.content_seen) {
    return true;
  }
  return g.last_read_at != 0.0 && (now_seconds() - g.last_read_at) < 0.35;
}

void human_bytes(char *out, size_t n, uint64_t b) {
  if (b >= 1024u * 1024u) {
    std::snprintf(out, n, "%.1f MB", (double)b / (1024.0 * 1024.0));
  } else {
    std::snprintf(out, n, "%.0f KB", (double)b / 1024.0);
  }
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
