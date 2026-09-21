/* gw_net.h - rollback netplay transport: UDP, handshake, input exchange, time sync, checksums.
 *
 * Self-contained and game-agnostic. The rollback session layer (pc/platform/gw_rollback.h) sits on
 * top; this module knows nothing about the simulation. Everything is driven from the game thread by
 * ONE call per logic-frame iteration (gw_net_poll) - no threads, no blocking. Protocol spec:
 * _research/rollback-net.md.
 *
 * FRAMES. Net frames start at 0 at the synchronized start signal and are the session's own frame
 * numbers. The session submits one set of local pads per frame, CONTIGUOUSLY from frame 0 (input
 * delay is the session's business: it submits frame F+delay's pad for frame F+delay ahead of time
 * and prefills 0..delay-1 with neutral). The module delivers each remote pad exactly once, in
 * strictly increasing frame order, through cb.remote_input.
 *
 * TESTABILITY. The socket and the clock are pluggable (gw_net_transport, cfg.now_ms), so the whole
 * protocol runs in deterministic virtual time over a simulated lossy network in the headless tests
 * (gw_net_tests.c), plus a real-UDP loopback smoke test.
 */
#ifndef GW_NET_H
#define GW_NET_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define GW_NET_PROTOCOL_VERSION 1u
#define GW_NET_MAX_BLOB 900        /* match-config blob: StartMeleeData + extras */
#define GW_NET_MAX_PORTS 4
#define GW_NET_RING 256            /* frames of input history/lookahead */
#define GW_NET_MAX_INPUTS_PER_PACKET 40

/* One controller's state for one frame: what the game reads (PADStatus, minus the analog A/B the
 * game never uses). 9 bytes on the wire. */
typedef struct gw_net_pad {
  uint16_t buttons;
  int8_t stick_x, stick_y;
  int8_t cstick_x, cstick_y;
  uint8_t trig_l, trig_r;
  int8_t err;                      /* PAD_ERR_*: the connected state is simulation state */
} gw_net_pad;

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

/* ---- session interface (the rollback layer implements the callbacks) ------------------------ */

enum gw_net_event {
  GW_NET_EV_ACCEPTED = 1,          /* guest: host's config received (gw_net_remote_config valid) */
  GW_NET_EV_STARTING,              /* both: start time known (gw_net_start_time_ms) */
  GW_NET_EV_STARTED,               /* both: the start time has arrived; frame 0 begins now */
  GW_NET_EV_REFUSED,               /* guest: host refused, msg says why */
  GW_NET_EV_INTERRUPTED,           /* no packet from the peer for notify_timeout_ms */
  GW_NET_EV_RESUMED,               /* the peer is back */
  GW_NET_EV_DISCONNECTED           /* terminal: peer quit, timed out, or handshake failed */
};

typedef struct gw_net_callbacks {
  void *user;
  /* A remote pad, exactly once, strictly increasing frame order (every port of frame f before any
   * of frame f+1). The adapter for gw_rb_submit_remote_input(port, frame, pad). */
  void (*remote_input)(void *user, uint32_t frame, int port, const gw_net_pad *pad);
  /* Lifecycle. msg may be NULL. */
  void (*event)(void *user, int event, const char *msg);
  /* Both peers reported a checksum for `frame` and they differ. Fires once (the first). */
  void (*desync)(void *user, uint32_t frame, uint32_t local, uint32_t remote);
} gw_net_callbacks;

typedef struct gw_net_config {
  /* identity: a mismatch on any refuses the session */
  uint64_t exe_hash, iso_hash, mods_hash;
  /* host only: authoritative match setup, sent to the guest */
  uint32_t seed;
  uint8_t input_delay;             /* frames */
  uint8_t host_ports;              /* bitmask of ports the host controls (bit i = port i) */
  uint8_t guest_ports;             /* bitmask of ports the guest controls */
  const void *match_blob;
  uint16_t match_blob_len;         /* <= GW_NET_MAX_BLOB */
  /* both */
  gw_net_callbacks cb;
  uint32_t (*now_ms)(void *user);  /* NULL = the wall clock (QueryPerformanceCounter) */
  uint32_t disconnect_timeout_ms;  /* 0 = 5000 */
  uint32_t notify_timeout_ms;      /* 0 = 1000 */
  uint32_t frame_us;               /* 0 = 16667 (60 Hz) */
} gw_net_config;

typedef struct gw_net gw_net;

/* Host: wait for one guest. Guest: connect to a host. The transport is owned by the gw_net after
 * this call (closed by gw_net_free). Returns NULL on bad arguments. */
gw_net *gw_net_host(const gw_net_config *cfg, const gw_net_transport *t);
gw_net *gw_net_join(const gw_net_config *cfg, const gw_net_transport *t, const gw_net_addr *host);
void gw_net_free(gw_net *n);       /* sends QUIT if still connected */

/* Call once per logic-frame iteration (and every iteration while waiting to start). Receives,
 * dispatches callbacks, sends. `local_frame` = the frame the session is about to simulate (the
 * newest frame for which it has submitted local input is submitted with gw_net_submit_local). */
void gw_net_poll(gw_net *n, uint32_t local_frame);

/* The session's local pads for `frame` (only the ports it controls are read). Frames must be
 * submitted contiguously from 0. Returns 0 on success, < 0 if the send window is full (the session
 * must stall: see gw_net_unacked) or frame is out of sequence. */
int gw_net_submit_local(gw_net *n, uint32_t frame, const gw_net_pad pads[GW_NET_MAX_PORTS]);

/* State */
int gw_net_state(const gw_net *n);         /* enum gw_net_state */
enum gw_net_state {
  GW_NET_IDLE = 0, GW_NET_CONNECTING, GW_NET_LISTENING, GW_NET_ACCEPTED, GW_NET_STARTING,
  GW_NET_RUNNING, GW_NET_REFUSED, GW_NET_DEAD
};
int gw_net_started(const gw_net *n);       /* RUNNING and the start time has arrived */
uint32_t gw_net_start_time_ms(const gw_net *n);   /* in the local clock; valid from STARTING */
/* Host's setup as the guest received it (or the host's own): seed, delay, ports, blob. */
int gw_net_remote_config(const gw_net *n, gw_net_config *out_cfg, void *blob_buf, int blob_cap);
uint8_t gw_net_local_ports(const gw_net *n);
uint8_t gw_net_remote_ports(const gw_net *n);
const char *gw_net_last_reason(const gw_net *n);

/* Delivery / window */
int32_t gw_net_remote_confirmed_frame(const gw_net *n);  /* highest contiguous remote frame delivered, -1 none */
uint32_t gw_net_unacked(const gw_net *n);  /* local frames the peer has not acknowledged */

/* Time sync. The number of frames the session should stall (not advance) right now to let the peer
 * catch up. Consumes the recommendation; returns 0 most of the time. */
int gw_net_recommend_wait(gw_net *n);
float gw_net_frame_advantage(const gw_net *n);   /* smoothed: positive = we are ahead */
uint32_t gw_net_rtt_ms(const gw_net *n);
uint32_t gw_net_silent_ms(const gw_net *n);      /* since the last packet from the peer */

/* Desync detection. Report the checksum of each CONFIRMED frame (identical on both peers when in
 * sync); they are exchanged piggybacked on input packets. */
void gw_net_report_checksum(gw_net *n, uint32_t frame, uint32_t hash);
int32_t gw_net_desync_frame(const gw_net *n);    /* first mismatching frame, -1 none */

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
