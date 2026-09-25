/* gw_rollback.h - the rollback SESSION layer: per-port input sources, input delay, prediction,
 * rollback and stalls, on top of gw_snap.c's savestates.
 *
 * The transport (gw_net.c, another agent) talks to the session ONLY through the functions in
 * "THE NETWORK-FACING INTERFACE" below. Everything else here is the session's own wiring into the
 * scene loop (gmscene.c) and the replay module (gw_replay.c).
 *
 * MODEL
 *   Time is the LOGIC FRAME number, Slippi's numbering: the first frame fighters run is -123.
 *   Every frame, each fighter slot (port 0..3, leader/follower) gets one processed input - the
 *   same six fields the fighter holds after the game's own deadzone/UCF processing, plus the raw
 *   stick bytes UCF reads (GwRbInput). A slot's input for a frame is either
 *     CONFIRMED  - it is the real input (local, or a remote one that has arrived), or
 *     PREDICTED  - a remote input that has not arrived yet: the last confirmed one, repeated.
 *   The session records what it USED for every frame. When a remote input arrives for a frame
 *   that was already simulated and it differs from what was used, the session rolls back to the
 *   earliest such frame (load its snapshot, resimulate forward with the corrected inputs) - at
 *   most MAX frames (MELEE_RB_MAX, default 7). It never lets the simulation run more than MAX
 *   frames ahead of the newest frame whose remote inputs are all confirmed: it STALLS instead.
 *
 * INPUT DELAY. A local input sampled at frame t takes effect at frame t + D (MELEE_RB_DELAY,
 *   default 2). The peer therefore receives, at time t, the input for t + D; it costs nothing
 *   until the network latency exceeds D frames, after which the difference is rolled back.
 *
 * WHERE INPUTS ARE INJECTED. At the fighter's pad read (Fighter_Spaghetti_8006AD10), through
 *   gw_replay.c's accessors, exactly as .slp playback does; with a session active they read the
 *   session's per-frame ring instead of the replay. (Live pads join the ring through
 *   gw_rb_submit_local_input - see below - which is where the game's deadzone/calibration
 *   processing has already run.)
 *
 * CONFIGURATION (environment)
 *   MELEE_RB_FAKE=<lat>[,<jitter>[,<loss%>]]   fake network for the acceptance test: plays the
 *        MELEE_SLP replay with the "remote" ports' inputs delivered <lat> frames late, +-<jitter>,
 *        losing <loss>% of first transmissions (redelivered after a timeout).
 *   MELEE_RB_DELAY=<D>    input delay in frames (default 2)
 *   MELEE_RB_MAX=<M>      maximum rollback depth / prediction window (default 7, at most 12)
 *   MELEE_RB_REMOTE=<hex> mask of ports whose inputs arrive over the (fake) network (default 2:
 *                         port 1); all other present ports are local
 *   MELEE_RB_SEED=<n>     seed for the fake network's jitter and loss (default 12345)
 *   MELEE_RB_INPUT=replay|padgen|live   where the players' inputs come from (default replay):
 *        replay  the .slp's recorded PROCESSED inputs (the acceptance test)
 *        padgen  a deterministic per-frame RAW pad generator (MELEE_RB_PADSEED), through the game's
 *                own pad pipeline - the automated test of the raw-controller path
 *        live    the real controllers (PADRead), latched between logic frames; every present port
 *                is read locally, and the ports in MELEE_RB_REMOTE are delivered through the fake
 *                network - two humans on one machine standing in for a remote peer
 */
#ifndef GW_ROLLBACK_H
#define GW_ROLLBACK_H

#include <stdint.h>

#define GW_RB_SLOTS 8 /* 4 ports x (leader, follower): slot = port * 2 + follower */

/* One frame of one fighter slot's input, in the form the fighter consumes it. */
typedef struct GwRbInput {
    float lx, ly;       /* main stick, after the game's deadzone processing */
    float cx, cy;       /* c-stick */
    float trigger;      /* analog trigger */
    uint32_t buttons;   /* held buttons (processed: L/R/Z folded as the fighter sees them) */
    uint32_t seed;      /* replay's frame seed, when known (playback only; 0 otherwise) */
    int8_t raw[4];      /* raw stick bytes x, y, c-x, c-y (what UCF reads); 0 when unknown */
    uint8_t present;    /* 0: no input for this slot - the fighter reads the live pad */
    uint8_t confirmed;  /* 1: the real input; 0: a prediction (session ring only) */
    /* RAW CONTROLLER entries (is_raw = 1): a whole PADStatus - buttons = the 16-bit button word,
     * raw[] = stickX, stickY, substickX, substickY, the fields below = the rest. The floats are
     * unused. The session then feeds the game's own pad pipeline (HSD_PadRenewMasterStatus), so
     * deadzones, calibration and UCF's raw-byte reads all run as on a console, and fighters read
     * their pad normally instead of a replay-style processed input. */
    uint8_t is_raw;
    uint8_t pad_l, pad_r, pad_a, pad_b; /* triggerLeft/Right, analogA/B */
    int8_t pad_err;                     /* PADStatus.err (0 = ok) */
    /* Replay-only physical fields. Ordinary playback still consumes the processed values above.
     * A modern online .slp records these separately from the processed fighter input. */
    uint16_t physical_buttons;
    float physical_l, physical_r;
    uint8_t physical_complete;
} GwRbInput;

typedef struct GwSlippiPad GwSlippiPad;

/* External Slippi source. Only the experimental mode calls configure; it does so after
 * MELEE_SLP has loaded a validated two-human online fixture and before entering the VS scene.
 * Ports are zero-based. peer_tick runs on the game thread once per render tick before rollback
 * planning, with the online PAD frame for the NEXT applied simulation frame. */
int gw_rb_slippi_configure(int local_port, int delay, void (*peer_tick)(int online_frame));
void gw_rb_slippi_disable(void);
int gw_rb_slippi_local_pad(int online_frame, GwSlippiPad *out);
int gw_rb_slippi_receive(int epoch, int remote_port, int online_frame, const GwSlippiPad *pad);
int gw_rb_local_fixture_reads(int port);
int gw_rb_slippi_local_port(void); /* -1 outside the experimental mode */
/* Finalizes trace and recording through the last replay frame once all remote pads have arrived
 * and any correction has been resimulated. Does not simulate an extra game frame. */
int gw_rb_slippi_finalized(void);

/* ======================= THE NETWORK-FACING INTERFACE ======================================== */

/* True while a rollback session is running (a match is live and a session was configured). */
int gw_rb_active(void);

/* Deliver a REMOTE peer's input for `frame`. May be called any number of times, in any order,
 * for any frame (redelivery is harmless). If the frame was already simulated with a different
 * input, a rollback is scheduled for the next render tick. `slot` = port * 2 + follower. Frames
 * older than the session's snapshot window that differ are counted as a desync
 * (gw_rb_desyncs()). Frames more than 56 ahead of the simulation are REFUSED (the input rings
 * hold 64 frames): a peer stalls at MAX frames ahead of what it has confirmed, so this never
 * happens with a conforming transport - keep the lookahead window within it and resend.
 * Thread: game thread only (call it from the scene loop's hook, not from a socket thread - queue
 * and drain). */
void gw_rb_submit_remote_input(int slot, int frame, const GwRbInput *in);

/* The LOCAL input the session used (or will use) for `frame`, for sending to the peer. Valid for
 * frames up to gw_rb_current_frame() + delay. Returns 0 when there is none yet. */
int gw_rb_local_input_for_send(int slot, int frame, GwRbInput *out);

/* Feed a local input sampled NOW (live pads): it takes effect at gw_rb_current_frame() + delay
 * + 1... i.e. `frame` must be the frame it is FOR; the caller (the pad hook) computes it as
 * next simulated frame + delay. Replaced if submitted twice. */
void gw_rb_submit_local_input(int slot, int frame, const GwRbInput *in);

/* The newest frame F such that every remote slot's input is confirmed for all frames <= F. The
 * simulation may be at most MAX frames beyond it. -124 before the match. */
int gw_rb_confirmed_frame(void);

/* The frame the simulation will run next (the newest simulated frame + 1). */
int gw_rb_current_frame(void);

/* Frames the local side is AHEAD of the remote (simulated frame minus the newest frame the peer
 * has confirmed having received/sent) - what a time-sync layer slows the local clock by
 * (GGPO's "frame advantage"). Positive: we are ahead. */
int gw_rb_frame_advantage(void);

/* The CURATED GAMEPLAY checksum of the state at the START of `frame` of the current epoch
 * (32-bit, never 0): the RNG seed plus, per fighter, motion, position, velocity, damage and facing
 * (ftRb hash in fighter.c). It is what two real machines must agree on; it deliberately leaves out
 * render-owned bytes, sound handles and heap addresses, which legitimately differ. Published only
 * for FINAL frames - every input before `frame` confirmed and no correction pending - else 0.
 * Exchange them and compare for desync detection. (The whole-state hash SyncTest uses is
 * gw_rb_checksum_full: valid within one process only.) */
uint32_t gw_rb_checksum(int frame);
uint32_t gw_rb_checksum_full(int frame);

/* EPOCHS. Frames are numbered per scene (a VS match: -123.. again each time) and every scene
 * begins a new epoch; a (epoch, frame) pair is never reused, so a hash or an input labelled with an
 * old epoch can never be mistaken for the new scene's. gw_rb_epoch() is the current one (0 before
 * any scene). The _e variants drop labels from an earlier epoch and hold back ones from a later
 * epoch (a peer that entered the scene first) until this side reaches it. The plain functions
 * above mean "the current epoch". */
int gw_rb_epoch(void);
void gw_rb_submit_remote_input_e(int epoch, int slot, int frame, const GwRbInput *in);
uint32_t gw_rb_checksum_e(int epoch, int frame);

/* TIME SYNC, applied. The transport (gw_net) estimates who is ahead and by how much; the ahead
 * peer gives back half the gap. This is the actuator: skip the next `frames` NEW-frame iterations
 * (rollbacks and rendering still run, so the picture does not freeze), one per render tick. Not
 * counted as stalls. Calls add up; negative values are ignored. */
void gw_rb_request_wait(int frames);

/* Statistics since the session began. */
int gw_rb_rollbacks(void);
int gw_rb_desyncs(void);

/* ======================= SESSION WIRING (game side) ========================================== */

/* gmscene.c: how many logic iterations to run this render tick (0 = stalled; k+1 = a rollback of
 * k frames plus the new frame). Called once per tick, before the iterations. */
int gw_RB_Iterations(int count);

/* gmscene.c: at the top of every logic iteration. Loads the snapshot on a rollback's first
 * iteration, prepares the frame's inputs, saves the snapshot of the frame about to run and gates
 * side effects (sound, rumble) while resimulating. */
void gw_RB_IterStart(void);

/* gmscene.c: after the tick's render pass; closes the per-tick work measurement. */
void gw_RB_TickEnd(void);

/* gmscene.c, at the start of every scene: a session governs VS matches only. */
void gw_RB_SceneBegin(int scene_kind);

/* gmscene.c: is a rollback session configured (before it is armed)? Chooses which of SyncTest's
 * and the session's hooks the scene loop runs. */
int gw_RB_Enabled(void);

/* gw_replay.c: the session's PROCESSED input for (port, follower) at `frame` (replay-style
 * entries), or NULL when the fighter should read the pad pipeline (raw entries, or none). */
const GwRbInput *gw_RB_InputFor(int port, int follower, int frame);

/* gw_replay.c (UCF): the session's entry for (port, leader) at `frame` INCLUDING raw-controller
 * ones - what UCF reads raw stick bytes from. NULL when there is none. */
const GwRbInput *gw_RB_InputAny(int port, int frame);

/* controller.c, HSD_PadRenewRawStatus, after PADRead: latch one poll of one port. Between two
 * logic frames the latch ORs the button edges, keeps the trigger/analog peak and takes the newest
 * stick position - a tap or an air-dodge press that begins and ends between two logic frames is
 * not lost. Ignored unless MELEE_RB_INPUT=live. */
void gw_RB_PadLatch(int port, int button, int sx, int sy, int cx, int cy, int l, int r, int a,
                    int b, int err);

/* controller.c, HSD_PadRenewMasterStatus: does the session feed this logic iteration's pad
 * status (raw-controller sources), and, if so, its fields for `port`
 * (which: 0 button, 1 stickX, 2 stickY, 3 substickX, 4 substickY, 5 triggerL, 6 triggerR,
 * 7 analogA, 8 analogB, 9 err). Scalars only: the game side byte-swaps its own memory. */
int gw_RB_PadGoverned(void);
int gw_RB_PadField(int port, int which);

/* gw_replay.c: the replay's recorded input for (slot, frame) - the "player" side of the fake
 * network. Returns 0 when the replay has none. */
int gw_Replay_PeekInput(int slot, int frame, GwRbInput *out);

#endif
