/* gw_net.h - rollback netplay transport: UDP, handshake, input exchange, time sync, checksums.
 *
 * Self-contained and game-agnostic. The rollback session layer (pc/platform/gw_rollback.h) sits on
 * top; this module knows nothing about the simulation and treats a slot's per-frame input as an
 * opaque payload of a fixed, negotiated size. Everything is driven from the game thread by ONE call
 * per logic-frame iteration (gw_net_poll) - no threads, no blocking. Protocol spec:
 * _research/rollback-net.md.
 *
 * SLOTS AND FRAMES follow the session layer: a slot is port * 2 + follower (0..7), frames are the
 * session's own signed numbers (Slippi's: the first frame fighters run is -123 - configure it as
 * cfg.first_frame). Each peer owns a set of slots (host_slots / guest_slots) and sends one payload
 * per owned slot per frame, CONTIGUOUSLY from first_frame. The module delivers each remote payload
 * exactly once, in strictly increasing frame order, through cb.remote_input - the session's own
 * gw_rb_submit_remote_input tolerates redelivery, but ordered exactly-once is what tests assert.
 *
 * TESTABILITY. The socket and the clock are pluggable (gw_net_transport, cfg.now_ms), so the whole
 * protocol runs in deterministic virtual time over a simulated lossy network in the headless tests
 * (gw_net_tests.c), plus a real-UDP loopback smoke test.
 *
 * THE GAME-SIDE ADAPTER (about 40 lines; it belongs to whoever wires netplay into gmscene.c):
 *
 *   cfg.first_frame   = -123;                          // GW_RB_FIRST_FRAME
 *   cfg.payload_bytes = 28 (GwRbInput, minus the seed) or 9 (raw pad; see the research doc).
 *   cfg.cb.local_input  -> gw_rb_local_input_for_send(slot, frame, &in); encode(in) -> out; return found
 *   cfg.cb.remote_input -> decode(payload) -> GwRbInput; gw_rb_submit_remote_input(slot, frame, &in)
 *   cfg.cb.checksum     -> h = gw_rb_checksum(frame); *out = h; return h != 0
 *   cfg.cb.desync       -> log + surface (the session counts its own desyncs too)
 *   each render tick:   gw_net_poll(net, gw_rb_current_frame());
 *                       stall = gw_net_recommend_wait(net)  -> add to the session's stall count
 *                       gw_net_unacked / gw_net_silent_ms   -> extra reasons to stall
 * The remote_input callback may fire before the local EV_STARTED (up to the network jitter): the
 * session must accept early remote inputs (its ring does).
 */
#ifndef GW_NET_H
#define GW_NET_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define GW_NET_PROTOCOL_VERSION 2u
#define GW_NET_MAX_BLOB 900        /* match-config blob: StartMeleeData + extras */
#define GW_NET_MAX_SLOTS 8         /* port * 2 + follower */
#define GW_NET_MAX_PAYLOAD 32      /* bytes of one slot's input for one frame */
#define GW_NET_DEFAULT_PAYLOAD 9   /* a raw PADStatus: buttons u16, 4 stick bytes, 2 trigger bytes, err */
#define GW_NET_RING 256            /* frames of input history/lookahead */
#define GW_NET_MAX_INPUTS_PER_PACKET 40
#define GW_NET_NO_FRAME ((int32_t)0x80000000)
#define GW_NET_MAX_GUEST_INFO 64
#define GW_NET_HOLD_TIMEOUT_MS 90000u

/* One slot's input for one frame, exactly cfg.payload_bytes long. */
typedef struct gw_net_input { uint8_t b[GW_NET_MAX_PAYLOAD]; } gw_net_input;

/* ---- transport ------------------------------------------------------------------------------ */

typedef struct gw_net_addr {
  uint32_t ip;                     /* host byte order */
  uint16_t port;                   /* host byte order */
} gw_net_addr;

typedef struct gw_net_transport {
  void *ctx;
  int (*send)(void *ctx, const gw_net_addr *to, const void *data, int len);
  /* Non-blocking. Returns the datagram length, 0 when nothing is waiting, < 0 on a hard error. */
  int (*recv)(void *ctx, gw_net_addr *from, void *buf, int cap);
  void (*close)(void *ctx);
} gw_net_transport;

/* Real UDP over Winsock. bind_ip 0 = all interfaces (triggers the Windows firewall prompt the first
 * time; tests bind 127.0.0.1); bind_port 0 = ephemeral. */
int gw_net_udp_open(uint32_t bind_ip, uint16_t bind_port, gw_net_transport *out);
uint16_t gw_net_udp_local_port(const gw_net_transport *t);
/* "1.2.3.4:5678" or "1.2.3.4" (default_port). Returns 0 on success. */
int gw_net_addr_parse(const char *text, uint16_t default_port, gw_net_addr *out);

/* ---- identity hashes (the game fills the config with these) ---------------------------------- */

uint64_t gw_net_hash64(const void *data, size_t len, uint64_t seed);       /* FNV-1a */
/* Hash of a file's first max_bytes (0 = whole file); 0 on error. An ISO is 1.4 GB: hash its header
 * and file table, not the lot. */
uint64_t gw_net_hash_file(const char *path, size_t max_bytes);

/* ---- session interface --------------------------------------------------------------------- */

enum gw_net_event {
  GW_NET_EV_ACCEPTED = 1,          /* guest: host's config received (gw_net_remote_config valid) */
  GW_NET_EV_STARTING,              /* both: start time known (gw_net_start_time_ms) */
  GW_NET_EV_STARTED,               /* both: the start time has arrived; frame first_frame begins now */
  GW_NET_EV_REFUSED,               /* guest: host refused, msg says why */
  GW_NET_EV_INTERRUPTED,           /* no packet from the peer for notify_timeout_ms */
  GW_NET_EV_RESUMED,               /* the peer is back */
  GW_NET_EV_DISCONNECTED           /* terminal: peer quit, timed out, or handshake failed */
};

typedef struct gw_net_callbacks {
  void *user;
  /* A remote payload, exactly once, strictly increasing frame order (every slot of frame f, in
   * ascending slot order, before any of frame f+1). */
  void (*remote_input)(void *user, int32_t frame, int slot, const uint8_t *payload);
  /* PULL model (optional): the local input for (frame, slot), written into out[payload_bytes].
   * Return 1 if available, 0 if not yet. gw_net_poll asks for consecutive frames until one is not
   * available. If NULL, the session pushes with gw_net_submit_local instead. */
  int (*local_input)(void *user, int32_t frame, int slot, uint8_t *out);
  /* PULL (optional): the checksum of the simulation at the START of `frame`. Return 1 if known.
   * Asked for consecutive frames up to the newest one whose remote inputs are all in. If NULL, the
   * session pushes with gw_net_report_checksum. */
  int (*checksum)(void *user, int32_t frame, uint32_t *out);
  /* Lifecycle. msg may be NULL. */
  void (*event)(void *user, int event, const char *msg);
  /* Both peers reported a checksum for `frame` and they differ. Fires once (the first). */
  void (*desync)(void *user, int32_t frame, uint32_t local, uint32_t remote);
  /* Host (optional): a guest's HELLO has been accepted. `info` is the guest's cfg.guest_info (its
   * own choices, e.g. a character). The host may rewrite the match blob it is about to send:
   * blob[0..*blob_len) in, at most `cap` bytes out. Called before the ACCEPT goes out. */
  void (*guest_hello)(void *user, const uint8_t *info, int info_len, uint8_t *blob, uint16_t *blob_len,
                      int cap);
} gw_net_callbacks;

typedef struct gw_net_config {
  /* identity: a mismatch on any refuses the session */
  uint64_t exe_hash, iso_hash, mods_hash;
  /* both peers must agree (mismatch refuses): */
  int32_t first_frame;             /* the session's first frame number (-123) */
  uint8_t payload_bytes;           /* 0 = GW_NET_DEFAULT_PAYLOAD; at most GW_NET_MAX_PAYLOAD */
  /* host only: authoritative match setup, sent to the guest */
  uint32_t seed;
  uint8_t input_delay;             /* frames */
  uint8_t host_slots;              /* bitmask of slots the host controls (bit i = slot i) */
  uint8_t guest_slots;             /* bitmask of slots the guest controls */
  const void *match_blob;
  uint16_t match_blob_len;         /* <= GW_NET_MAX_BLOB */
  /* guest only: its own choices, sent in the HELLO (see cb.guest_hello) */
  const void *guest_info;
  uint8_t guest_info_len;          /* <= GW_NET_MAX_GUEST_INFO */
  /* both: 1 = stop at ACCEPTED (connected, match agreed) until gw_net_release - so the peers can
   * load the match before the start time is agreed. While either side holds, silence up to
   * GW_NET_HOLD_TIMEOUT_MS is tolerated (the other may be loading). */
  uint8_t hold_start;
  /* both */
  gw_net_callbacks cb;
  uint32_t (*now_ms)(void *user);  /* NULL = the wall clock (QueryPerformanceCounter) */
  uint32_t disconnect_timeout_ms;  /* 0 = 5000 */
  uint32_t handshake_timeout_ms;   /* guest: give up dialling after this; 0 = 10000 */
  uint32_t notify_timeout_ms;      /* 0 = 1000 */
  uint32_t frame_us;               /* 0 = 16667 (60 Hz) */
} gw_net_config;

typedef struct gw_net gw_net;

/* Host: wait for one guest. Guest: connect to a host. The transport is owned by the gw_net after
 * a successful call (closed by gw_net_free). Returns NULL on bad arguments (and then the caller
 * still owns the transport). */
gw_net *gw_net_host(const gw_net_config *cfg, const gw_net_transport *t);
gw_net *gw_net_join(const gw_net_config *cfg, const gw_net_transport *t, const gw_net_addr *host);
void gw_net_free(gw_net *n);       /* sends QUIT if still connected */

/* Call once per logic-frame iteration (and every iteration while waiting to start). Receives,
 * dispatches callbacks, pulls local inputs/checksums (if the callbacks are set), sends.
 * `local_frame` = the frame the session will simulate next (for the frame-advantage estimate). */
void gw_net_poll(gw_net *n, int32_t local_frame);

/* cfg.hold_start: this side is ready (the match is loaded) - agree the start time once the peer
 * is ready too. */
void gw_net_release(gw_net *n);

/* PUSH model: this peer's inputs for `frame`, indexed by slot (only owned slots are read). Frames
 * must be submitted contiguously from first_frame. Returns 0, or < 0 if the send window is full
 * (the session must stall: see gw_net_unacked) or the frame is out of sequence. */
int gw_net_submit_local(gw_net *n, int32_t frame, const gw_net_input in[GW_NET_MAX_SLOTS]);

/* State */
enum gw_net_state {
  GW_NET_IDLE = 0, GW_NET_CONNECTING, GW_NET_LISTENING, GW_NET_ACCEPTED, GW_NET_STARTING,
  GW_NET_RUNNING, GW_NET_REFUSED, GW_NET_DEAD
};
int gw_net_state(const gw_net *n);
int gw_net_started(const gw_net *n);       /* RUNNING: the start time has arrived */
uint32_t gw_net_start_time_ms(const gw_net *n);   /* in the local clock; valid from STARTING */
/* The host's setup as this peer has it: seed, delay, slots, blob. Host: its own; guest: after
 * ACCEPTED. Returns 1 if valid. */
int gw_net_remote_config(const gw_net *n, gw_net_config *out_cfg, void *blob_buf, int blob_cap);
uint8_t gw_net_local_slots(const gw_net *n);
uint8_t gw_net_remote_slots(const gw_net *n);
const char *gw_net_last_reason(const gw_net *n);

/* Delivery / window */
int32_t gw_net_remote_confirmed_frame(const gw_net *n);  /* newest frame whose remote inputs are all delivered; first_frame - 1 if none */
uint32_t gw_net_unacked(const gw_net *n);  /* local frames the peer has not acknowledged */

/* Time sync. The number of frames the session should stall (not advance) right now to let the peer
 * catch up. Consumes the recommendation; returns 0 most of the time. */
int gw_net_recommend_wait(gw_net *n);
float gw_net_frame_advantage(const gw_net *n);   /* smoothed, RTT-compensated: positive = we are ahead */
uint32_t gw_net_rtt_ms(const gw_net *n);
uint32_t gw_net_silent_ms(const gw_net *n);      /* since the last packet from the peer */

/* Desync detection (PUSH model): report the checksum of each frame once its remote inputs are in;
 * they are exchanged piggybacked on input packets. */
void gw_net_report_checksum(gw_net *n, int32_t frame, uint32_t hash);
int32_t gw_net_desync_frame(const gw_net *n);    /* first mismatching frame, GW_NET_NO_FRAME if none */

typedef struct gw_net_stats {
  uint32_t packets_sent, packets_received, duplicates, out_of_order, bad_packets;
  uint32_t inputs_delivered, resent_frames;
} gw_net_stats;
void gw_net_get_stats(const gw_net *n, gw_net_stats *out);

/* Registered from gw_tests_core.c's gw_tests_register_all(). */
void gw_net_tests_register(void);

#ifdef __cplusplus
}
#endif
#endif
