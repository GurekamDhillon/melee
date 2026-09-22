/* gw_console.cpp - the in-game console overlay and the scripts' draw list, in Aurora's ImGui pass
 * (called from gw_Overlay_DrawPanel, i.e. shim_vi.c's per-frame overlay slot).
 *
 *   ` (the key left of 1, "~")   open / close the console; Esc closes it
 *   Enter                        run the line (gw_Script_Exec: a built-in command or Lua)
 *   Up / Down                    history
 * While it is open the keyboard is text, not a controller: gw_script.c keeps gw_TextEntryUntil
 * in the future, which is what makes shim_pad.c's keyboard mapping stand down (the same
 * mechanism the online room-code entry uses).
 *
 * Script drawing (gd.text / gd.box / gd.fill / gd.line) is in a 640x480 virtual screen, scaled
 * uniformly to the window and centred, so an overlay lines up with the game's 4:3 picture.
 */
#include "gw_script.h"

#include <imgui.h>

#include <windows.h>

#include <cstdio>
#include <cstring>

extern "C" void gw_log(const char *fmt, ...);

namespace {

bool g_key_was_down = false;
bool g_focus = false;
char g_input[512] = {0};
char g_hist[32][512];
int g_hist_n = 0;
int g_hist_pos = -1;
int g_last_lines = 0;

ImU32 col(uint32_t rgba) {
  return IM_COL32((rgba >> 24) & 0xFF, (rgba >> 16) & 0xFF, (rgba >> 8) & 0xFF, rgba & 0xFF);
}

bool focused() {
  HWND fg = GetForegroundWindow();
  DWORD pid = 0;
  if (fg != nullptr) {
    GetWindowThreadProcessId(fg, &pid);
  }
  return pid == GetCurrentProcessId();
}

int input_cb(ImGuiInputTextCallbackData *d) {
  if (d->EventFlag == ImGuiInputTextFlags_CallbackCharFilter) {
    return (d->EventChar == '`' || d->EventChar == '~') ? 1 : 0; /* the toggle key is not text */
  }
  if (d->EventFlag == ImGuiInputTextFlags_CallbackHistory && g_hist_n > 0) {
    if (d->EventKey == ImGuiKey_UpArrow) {
      g_hist_pos = g_hist_pos < 0 ? g_hist_n - 1 : (g_hist_pos > 0 ? g_hist_pos - 1 : 0);
    } else if (d->EventKey == ImGuiKey_DownArrow) {
      g_hist_pos = (g_hist_pos >= 0 && g_hist_pos < g_hist_n - 1) ? g_hist_pos + 1 : -1;
    }
    d->DeleteChars(0, d->BufTextLen);
    if (g_hist_pos >= 0) {
      d->InsertChars(0, g_hist[g_hist_pos]);
    }
  }
  return 0;
}

void draw_script_list(const ImGuiIO &io) {
  const int n = gw_Script_DrawCount();
  if (n == 0) {
    return;
  }
  const float sx = io.DisplaySize.x / 640.0f, sy = io.DisplaySize.y / 480.0f;
  const float s = sx < sy ? sx : sy;
  const float ox = (io.DisplaySize.x - 640.0f * s) * 0.5f, oy = (io.DisplaySize.y - 480.0f * s) * 0.5f;
  ImDrawList *dl = ImGui::GetForegroundDrawList();
  ImFont *font = ImGui::GetFont();
  for (int i = 0; i < n; ++i) {
    const GwScriptDraw *d = gw_Script_DrawAt(i);
    if (d == nullptr) {
      continue;
    }
    const ImVec2 p(ox + d->x * s, oy + d->y * s);
    switch (d->kind) {
    case GW_SDRAW_TEXT: {
      const float size = 13.0f * s * (d->size > 0.0f ? d->size : 1.0f);
      dl->AddText(font, size, ImVec2(p.x + 1.0f, p.y + 1.0f), IM_COL32(0, 0, 0, (d->rgba & 0xFF) * 3 / 4),
                  d->text);
      dl->AddText(font, size, p, col(d->rgba), d->text);
      break;
    }
    case GW_SDRAW_BOX:
      dl->AddRect(p, ImVec2(p.x + d->w * s, p.y + d->h * s), col(d->rgba), 0.0f, 0, 1.5f);
      break;
    case GW_SDRAW_FILL:
      dl->AddRectFilled(p, ImVec2(p.x + d->w * s, p.y + d->h * s), col(d->rgba));
      break;
    case GW_SDRAW_LINE:
      dl->AddLine(p, ImVec2(ox + d->w * s, oy + d->h * s), col(d->rgba), 1.5f);
      break;
    }
  }
}

void draw_console(const ImGuiIO &io) {
  const float h = io.DisplaySize.y * 0.45f;
  ImGui::SetNextWindowPos(ImVec2(0, 0), ImGuiCond_Always);
  ImGui::SetNextWindowSize(ImVec2(io.DisplaySize.x, h), ImGuiCond_Always);
  ImGui::SetNextWindowBgAlpha(0.86f);
  const ImGuiWindowFlags flags = ImGuiWindowFlags_NoTitleBar | ImGuiWindowFlags_NoResize |
                                 ImGuiWindowFlags_NoMove | ImGuiWindowFlags_NoCollapse |
                                 ImGuiWindowFlags_NoSavedSettings;
  if (ImGui::Begin("##gw_console", nullptr, flags)) {
    const float input_h = ImGui::GetFrameHeightWithSpacing();
    ImGui::BeginChild("##scroll", ImVec2(0, -input_h), false, ImGuiWindowFlags_HorizontalScrollbar);
    const int lines = gw_Console_LineCount();
    for (int i = 0; i < lines; ++i) {
      ImGui::PushStyleColor(ImGuiCol_Text, col(gw_Console_LineColor(i)));
      ImGui::TextUnformatted(gw_Console_Line(i));
      ImGui::PopStyleColor();
    }
    if (lines != g_last_lines) {
      ImGui::SetScrollHereY(1.0f);
      g_last_lines = lines;
    }
    ImGui::EndChild();
    ImGui::PushItemWidth(-1);
    if (g_focus) {
      ImGui::SetKeyboardFocusHere();
      g_focus = false;
    }
    const ImGuiInputTextFlags in_flags = ImGuiInputTextFlags_EnterReturnsTrue |
                                         ImGuiInputTextFlags_CallbackHistory |
                                         ImGuiInputTextFlags_CallbackCharFilter;
    if (ImGui::InputText("##in", g_input, sizeof g_input, in_flags, input_cb)) {
      if (g_input[0] != '\0') {
        if (g_hist_n == 32) {
          std::memmove(g_hist[0], g_hist[1], sizeof g_hist[0] * 31);
          --g_hist_n;
        }
        std::snprintf(g_hist[g_hist_n++], sizeof g_hist[0], "%s", g_input);
        gw_Script_Exec(g_input, nullptr, 0);
      }
      g_input[0] = '\0';
      g_hist_pos = -1;
      g_focus = true; /* keep typing */
    }
    ImGui::PopItemWidth();
  }
  ImGui::End();
}

} // namespace

extern "C" void gw_Console_Draw(void) {
  if (ImGui::GetCurrentContext() == nullptr) {
    return;
  }
  const ImGuiIO &io = ImGui::GetIO();
  /* 0x0001 too: a tap shorter than a frame (or a synthesized one, from a test) still toggles */
  const SHORT ks = GetAsyncKeyState(VK_OEM_3);
  const bool down = focused() && (ks & 0x8001) != 0;
  if (down && !g_key_was_down) {
    gw_Console_SetOpen(!gw_Console_Open());
    g_focus = gw_Console_Open() != 0;
    gw_log("gw: console %s", gw_Console_Open() ? "open" : "closed");
  }
  g_key_was_down = down;
  if (gw_Console_Open() && focused() && (GetAsyncKeyState(VK_ESCAPE) & 0x8000) != 0) {
    gw_Console_SetOpen(0);
  }
  if (!gw_Console_Open()) {
    draw_script_list(io); /* the console covers the top of the screen; overlays pause under it */
  }
  if (gw_Console_Open()) {
    draw_console(io);
  }
}
