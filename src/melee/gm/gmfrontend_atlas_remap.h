/* gmfrontend_atlas_remap.h - the decisions of the remap editor's Atlas layer, moved out of fcr_frame (gmfrontend_controls.inc) with no game types, so
 * pc/tests/atlas_remap_test.c can run them against a fake Controls_* API with a scripted clock. No libc and no game headers.
 *
 * What the layer must keep, because the runtime depends on it (gw_controls_runtime.inc):
 *   - Controls_Menu(port) is a HEARTBEAT. The editor flag is a 100 ms lease; ctl_poll cancels a capture (result 4) when it lapses. So the layer renews it every frame
 *     the editor is open (the scene's own call at the top of gm_Scene_Frontend_OnFrame renews it too, as it always did).
 *   - Input is FROZEN while a capture runs and until the new input is released (a pressed input that binds must not also act as an A or a B on the screen), and the
 *     ORIGINAL layout's START+B (0x1200 on port `port`, read with Pad_Value, never through the host) cancels a capture.
 *   - The result message is told for 120 frames after a result changes; while capturing the prompt is shown.
 *   - The 22 binding texts are read only when the result, the profile or the controller changed, and at most every 30 frames otherwise: never 22 calls a frame.
 * The editor's capture state (fcr_target, fcr_also, fcr_port) and the Controls_* API stay where they were. */
#ifndef GMFRONTEND_ATLAS_REMAP_H
#define GMFRONTEND_ATLAS_REMAP_H

#define FSSR_TARGETS 22   /* the game inputs of fcr_targets: A B X Y Z START L R, the d-pad, the sticks, the C-stick, the analog triggers */
#define FSSR_TEXT 40
#define FSSR_REFRESH_FRAMES 30
#define FSSR_MESSAGE_FRAMES 120

extern void Controls_Menu(int editor);
extern int Controls_Port(void);
extern void Controls_Select(int port);
extern int Controls_Connected(int port);
extern int Controls_Profile(void);
extern void Controls_Capture(int target, int also);
extern void Controls_Cancel(void);
extern int Controls_Capturing(void);
extern int Controls_Result(void);
extern int Controls_Held(void);
extern void Controls_Binding(int target, char* out, int cap);
extern void Controls_Tester(char* out, int cap);
extern int Pad_Value(int port, int what);

typedef struct {
    int result;         /* the last Controls_Result seen */
    int message_frames; /* frames the result message is still told */
    int wait_release;   /* a capture ended with a held input: frozen until it is let go */
    int port;           /* the controller being edited */
    int cache_ok, cache_result, cache_port, cache_profile, cache_age;
    char bind[FSSR_TARGETS][FSSR_TEXT];
} FssRemap;

/* the legacy strings, word for word (they are notes now) */
static const char* fssr_message_for(int result)
{
    static const char* const messages[] = {
        "Original layout navigates; START+B restores.",
        "Press and release an input. START+B cancels. 8 seconds.",
        "Saved.",
        "Timed out; not changed.",
        "Cancelled; not changed.",
        "Defaults restored.",
        "Profile copied.",
        "All four profile slots are full.",
        "Conflict: bindings swapped.",
        "Conflict: both bindings kept.",
        "Could not save. Check settings storage; this change may be temporary."
    };
    return (result >= 0 && result < 11) ? messages[result] : messages[0];
}

/* the note kind of a result: 0 ok, 1 warn, 3 info (AT_NOTE_*) */
static int fssr_message_kind(int result)
{
    switch (result) {
    case 2: case 5: case 6: case 8: case 9:
        return 0;
    case 3: case 7: case 10:
        return 1;
    default:
        return 3;
    }
}

/* the line to tell now: the prompt while capturing, else the result for a while, else the default (legacy: messages[capturing ? 1 : frames && 0 <= result < 11 ? result : 0]) */
static const char* fssr_message(const FssRemap* r)
{
    if (Controls_Capturing()) {
        return fssr_message_for(1);
    }
    return fssr_message_for(r->message_frames > 0 && r->result >= 0 && r->result < 11 ? r->result : 0);
}

static void fssr_open(FssRemap* r, int port)
{
    r->port = port;
    r->result = Controls_Result();
    r->message_frames = 0;
    r->wait_release = 0;
    r->cache_ok = 0;
    r->cache_age = 0;
    Controls_Menu(port); /* the lease starts now */
}

/* the editor is left: a half-capture does not stay bound, and the lease ends */
static void fssr_close(FssRemap* r)
{
    (void) r;
    Controls_Cancel();
    Controls_Menu(-1);
}

/* input is frozen while a capture runs, and until the input that ended it is released */
static int fssr_frozen(const FssRemap* r)
{
    return Controls_Capturing() != 0 || r->wait_release != 0;
}

/* One frame (fcr_frame, moved): the heartbeat, START+B, the result, the release wait. Returns the frozen state. */
static int fssr_frame(FssRemap* r)
{
    int result = Controls_Result();
    r->port = Controls_Port();
    Controls_Menu(r->port);
    if (Controls_Capturing() && (Pad_Value(r->port, 0) & 0x1200) == 0x1200) {
        Controls_Cancel(); /* START+B on the original layout; never through the host */
        result = Controls_Result();
    }
    if (result != r->result) {
        r->result = result;
        r->message_frames = FSSR_MESSAGE_FRAMES;
        if (result != 1) {
            r->wait_release = 1;
        }
    }
    if (r->wait_release && !Controls_Held()) {
        r->wait_release = 0;
    }
    if (r->message_frames > 0) {
        --r->message_frames;
    }
    if (r->cache_age < 1000000) {
        r->cache_age++;
    }
    return fssr_frozen(r);
}

/* A on an input row: start a capture for it (1) unless there is no controller on the port or the target is not one of the 22 (0) */
static int fssr_bind(FssRemap* r, int target, int also)
{
    (void) r;
    if (target < 0 || target >= FSSR_TARGETS || !Controls_Connected(Controls_Port())) {
        return 0;
    }
    Controls_Capture(target, also);
    return 1;
}

/* the 22 binding texts, read again only when something they depend on changed (or 30 frames passed); 1 when it read them */
static int fssr_bindings(FssRemap* r)
{
    int t, result = Controls_Result(), port = Controls_Port(), profile = Controls_Profile();
    if (r->cache_ok && result == r->cache_result && port == r->cache_port && profile == r->cache_profile && r->cache_age < FSSR_REFRESH_FRAMES) {
        return 0;
    }
    for (t = 0; t < FSSR_TARGETS; t++) {
        Controls_Binding(t, r->bind[t], FSSR_TEXT);
    }
    r->cache_ok = 1;
    r->cache_result = result;
    r->cache_port = port;
    r->cache_profile = profile;
    r->cache_age = 0;
    return 1;
}

#endif
