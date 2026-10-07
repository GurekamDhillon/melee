/* atlas_controls_fake.h - a fake of the Controls_* API the remap editor calls (gw_controls_runtime.inc), with a scripted clock. It models exactly what the runtime does that
 * the Atlas layer depends on: the editor flag is a 100 ms LEASE that Controls_Menu renews (ctl_menu_until); a capture ends with result 3 after 8 s, with result 4 when the
 * lease lapses or the controller goes, and with result 2 when a new input is pressed; Controls_Held is whether any physical input is down. It records every call. */
#ifndef ATLAS_CONTROLS_FAKE_H
#define ATLAS_CONTROLS_FAKE_H
#include <stdio.h>
#include <string.h>

static struct {
    unsigned now, menu_until, deadline;
    int editor, capturing, target, also, result, held[40], port, profile, connected[4];
    int n_menu, n_binding, n_tester, n_capture, n_cancel, n_select;
    int chord;                         /* the ORIGINAL layout's START+B (0x1200) is down */
} FK;

static void fake_reset(void)
{
    memset(&FK, 0, sizeof FK);
    FK.now = 1000; FK.connected[0] = 1; FK.result = 0;
}
static unsigned fake_clock(void) { return FK.now; }
static int fake_editor_active(void) { return FK.editor && (int) (FK.menu_until - FK.now) > 0; }
static int fake_any_held(void) { int i; for (i = 0; i < 40; i++) if (FK.held[i]) return 1; return 0; }

/* ctl_poll's decisions about a capture */
static void fake_poll(void)
{
    if (FK.capturing && (!fake_editor_active() || (int) (FK.now - FK.deadline) >= 0)) {
        FK.capturing = 0; FK.result = fake_editor_active() ? 3 : 4;
    }
    if (FK.capturing && !FK.connected[FK.port]) { FK.capturing = 0; FK.result = 4; }
}
static void fake_tick(unsigned ms) { FK.now += ms; fake_poll(); }
static void fake_physical_press(int i)
{
    FK.held[i] = 1;
    if (FK.capturing) { FK.capturing = 0; FK.result = 2; }                   /* a new input is saved */
}
static void fake_physical_release(int i) { FK.held[i] = 0; }
static void fake_chord_start_b(void) { FK.chord = 1; }
static void fake_unplug(int port) { FK.connected[port] = 0; fake_poll(); }
static void fake_set_result(int r) { FK.result = r; }

void Controls_Menu(int editor) { FK.n_menu++; FK.menu_until = FK.now + 100; FK.editor = editor >= 0 && editor < 4; }
int Controls_Port(void) { return FK.port; }
void Controls_Select(int port) { FK.n_select++; FK.port = port; FK.capturing = 0; FK.result = 0; }
int Controls_Connected(int port) { return port >= 0 && port < 4 && FK.connected[port]; }
int Controls_Profile(void) { return FK.profile; }
void Controls_Capture(int target, int also)
{
    FK.n_capture++;
    if (target >= 0 && !FK.connected[FK.port]) return;
    FK.capturing = 1; FK.target = target; FK.also = also; FK.deadline = FK.now + 8000; FK.result = 1;
}
void Controls_Cancel(void) { FK.n_cancel++; FK.capturing = 0; FK.result = 4; }
int Controls_Capturing(void) { return FK.capturing; }
int Controls_Result(void) { return FK.result; }
int Controls_Held(void) { return fake_any_held() && FK.connected[FK.port]; }
void Controls_Binding(int target, char *out, int cap) { FK.n_binding++; snprintf(out, (size_t) cap, "bind %d", target); }
void Controls_Tester(char *out, int cap) { FK.n_tester++; snprintf(out, (size_t) cap, "%s", FK.connected[FK.port] ? "A  S+0,+0" : "No controller"); }
int Pad_Value(int port, int what) { (void) port; return what == 0 && FK.chord ? 0x1200 : 0; }
#endif
