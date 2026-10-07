/* atlas-remap: the decisions of the remap editor's Atlas layer (gmfrontend_atlas_remap.h) against a fake Controls_* API with a scripted clock.
 * The timeline is the legacy one (fcr_frame in gmfrontend_controls.inc, ctl_poll and gw_Controls_Capture in gw_controls_runtime.inc) and must survive the move:
 * the heartbeat every frame, the freeze while capturing and until the input is released, START+B on the original layout, the 8 second timeout, the messages, the cache. */
#include "atlas_check.h"
#include "atlas_controls_fake.h"
#include "../../src/melee/gm/gmfrontend_atlas_remap.h"

static FssRemap R;
static void remap_open(int port) { fake_reset(); memset(&R, 0, sizeof R); FK.port = port; fssr_open(&R, port); }
static void remap_frame(void) { fssr_frame(&R); }
static int remap_input_frozen(void) { return fssr_frozen(&R); }
static void remap_press_a_on_row(int target) { fssr_bind(&R, target, 0); remap_frame(); }

static void heartbeat_every_frame(void)                               /* Review Focus 3 */
{
    int f;
    remap_open(0);
    CHECK(FK.n_menu == 1);                                             /* the open renews the lease once */
    for (f = 0; f < 600; f++) { fake_tick(16); remap_frame(); CHECK(fake_editor_active()); }     /* ten seconds open: the editor flag never lapses */
    CHECK(FK.n_menu == 600 + 1);                                       /* once per layer frame (the scene's own heartbeat is the game's), plus the open */
}
static void a_capture_is_cancelled_by_a_lapsed_heartbeat(void)         /* why the heartbeat matters: the runtime cancels a capture within 100 ms of it stopping */
{
    remap_open(0); remap_press_a_on_row(5);
    CHECK(FK.capturing == 1);
    fake_tick(150);                                                    /* no frame ran: the lease lapsed */
    CHECK(FK.capturing == 0 && FK.result == 4);
}
static void capture_timeline(void)
{
    remap_open(0); fake_tick(16); remap_frame();
    remap_press_a_on_row(5);                                           /* A on the START row */
    CHECK(FK.capturing == 1 && FK.target == 5 && FK.also == 0);
    CHECK(remap_input_frozen() == 1);                                  /* the host reads nothing while capturing: the press that binds is not an accept */
    { int f; for (f = 0; f < 62; f++) { fake_tick(16); remap_frame(); } }   /* a second of frames, the heartbeat renewing the lease on each */
    CHECK(remap_input_frozen() == 1 && FK.capturing == 1);              /* still waiting for the input */
    fake_physical_press(7); remap_frame();                             /* the player presses a new input */
    CHECK(FK.result == 2 /* saved */ && FK.capturing == 0);
    CHECK(remap_input_frozen() == 1);                                  /* STILL frozen: the legacy wait-for-release holds until the input is released */
    fake_tick(500); remap_frame(); CHECK(remap_input_frozen() == 1);
    fake_physical_release(7); fake_tick(16); remap_frame();
    CHECK(remap_input_frozen() == 0);                                  /* released: input returns (the host primes from the held state: no stray accept) */
    CHECK_STR(fssr_message(&R), "Saved.");                              /* the result is told for two seconds */
    { int f; for (f = 0; f < 130; f++) { fake_tick(16); remap_frame(); } }
    CHECK_STR(fssr_message(&R), "Original layout navigates; START+B restores.");   /* then the default line */
}
static void timeout_and_cancel(void)
{
    remap_open(0); remap_press_a_on_row(2);
    { int f; for (f = 0; f < 499; f++) { fake_tick(16); remap_frame(); CHECK(FK.capturing == 1); } }   /* 7984 ms of frames, the heartbeat renewing the lease every one */
    fake_tick(16); remap_frame(); fake_tick(16); remap_frame();         /* the 8 second timeout (the legacy message "Timed out; not changed."): result 3, not 4, because the lease never lapsed */
    CHECK(FK.capturing == 0 && FK.result == 3);
    CHECK_STR(fssr_message(&R), "Timed out; not changed.");
    CHECK(remap_input_frozen() == 0);                                  /* nothing is held: the input returns at once */
    remap_press_a_on_row(2);
    CHECK_STR(fssr_message(&R), "Press and release an input. START+B cancels. 8 seconds.");   /* while capturing, the prompt */
    fake_chord_start_b(); fake_tick(16); remap_frame();                /* START+B on the ORIGINAL layout cancels: read natively, never through the host */
    CHECK(FK.capturing == 0 && FK.result == 4 && FK.n_cancel == 1);
    CHECK_STR(fssr_message(&R), "Cancelled; not changed.");
    /* the chord does nothing when no capture runs: START+B alone is the legacy menu restore, not this layer's business */
    remap_open(0); fake_chord_start_b(); remap_frame(); CHECK(FK.n_cancel == 0);
}
static void messages_are_notes(void)
{
    int r;
    static const char *want[] = { "Original layout navigates; START+B restores.", "Press and release an input. START+B cancels. 8 seconds.", "Saved.",
        "Timed out; not changed.", "Cancelled; not changed.", "Defaults restored.", "Profile copied.", "All four profile slots are full.",
        "Conflict: bindings swapped.", "Conflict: both bindings kept.", "Could not save. Check settings storage; this change may be temporary." };
    for (r = 0; r < 11; r++) CHECK(strcmp(fssr_message_for(r), want[r]) == 0);                     /* every legacy message survives, word for word, as a note */
    CHECK(fssr_message_for(11) != NULL && fssr_message_for(-1) != NULL);                          /* out of range: the default line, never a read past the array */
    CHECK_STR(fssr_message_for(11), want[0]); CHECK_STR(fssr_message_for(-1), want[0]);
    CHECK(fssr_message_kind(2) == 0 && fssr_message_kind(7) == 1 && fssr_message_kind(10) == 1 && fssr_message_kind(4) == 3);   /* ok, warn, warn, info */
}
static void disconnect_mid_capture(void)
{
    remap_open(0); remap_press_a_on_row(1); fake_unplug(0); fake_tick(16); remap_frame();
    CHECK(FK.capturing == 0 && remap_input_frozen() == 0);             /* ctl_poll cancels with result 4 when the device goes: input must not stay frozen forever */
}
static void no_controller_no_capture(void)
{
    remap_open(1);                                                     /* port 2 has nothing plugged in */
    CHECK(fssr_bind(&R, 3, 0) == 0 && FK.n_capture == 0 && FK.capturing == 0);   /* a disabled row never reaches the runtime */
    remap_open(0);
    CHECK(fssr_bind(&R, 3, 1) == 1 && FK.n_capture == 1 && FK.also == 1);      /* Also mode is passed through */
    CHECK(fssr_bind(&R, 99, 0) == 0 && fssr_bind(&R, -1, 0) == 0);            /* a bad target is refused here too */
}
static void bindings_cached(void)
{
    int before;
    remap_open(0); remap_frame(); fssr_bindings(&R); before = FK.n_binding;
    CHECK(before == FSSR_TARGETS);                                      /* the first time: every row */
    remap_frame(); fssr_bindings(&R); remap_frame(); fssr_bindings(&R); remap_frame(); fssr_bindings(&R);
    CHECK(FK.n_binding == before);                                      /* nothing changed: no 22 calls per frame */
    fake_set_result(2); remap_frame(); fssr_bindings(&R); CHECK(FK.n_binding == before + 22);   /* a result changed: all 22 again */
    FK.profile = 1; remap_frame(); fssr_bindings(&R); CHECK(FK.n_binding == before + 44);       /* a profile change too */
    FK.port = 0; FK.connected[1] = 1; R.port = 0; Controls_Select(1); remap_frame(); fssr_bindings(&R); CHECK(FK.n_binding == before + 66);   /* and another controller */
    { int f, base = FK.n_binding; for (f = 0; f < 29; f++) { remap_frame(); fssr_bindings(&R); } CHECK(FK.n_binding == base); }
    remap_frame(); fssr_bindings(&R); remap_frame(); fssr_bindings(&R); CHECK(FK.n_binding == before + 88);   /* at most every 30 frames otherwise: the live mapping is read, not stale */
    CHECK_STR(R.bind[5], "bind 5"); CHECK_STR(R.bind[21], "bind 21");
}
static void frozen_state_is_exact(void)
{
    remap_open(0);
    CHECK(remap_input_frozen() == 0);                                   /* a fresh editor takes input */
    remap_press_a_on_row(0); CHECK(remap_input_frozen() == 1);
    fake_physical_press(3); remap_frame(); fake_physical_release(3);    /* pressed and released between two frames: nothing to wait for */
    remap_frame(); CHECK(remap_input_frozen() == 0);
    remap_press_a_on_row(0); fssr_close(&R); CHECK(Controls_Capturing() == 0 && FK.n_cancel >= 1);   /* leaving the editor ends a capture: B is not left bound to a half-capture */
}
int main(void)
{
    heartbeat_every_frame(); a_capture_is_cancelled_by_a_lapsed_heartbeat(); capture_timeline(); timeout_and_cancel(); messages_are_notes();
    disconnect_mid_capture(); no_controller_no_capture(); bindings_cached(); frozen_state_is_exact();
    ATLAS_DONE("atlas remap");
}
