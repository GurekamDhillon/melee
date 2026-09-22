/* gw_net.c - rollback netplay transport. See gw_net.h and _research/rollback-net.md.
 *
 * Native x86 code (a shim, not gwtool input): plain C99, Winsock2 for the default transport, and
 * NO dependence on game memory. Everything is driven by gw_net_poll from the game thread.
 *
 * WIRE FORMAT (little-endian, explicit field encoding - never a struct memcpy). Every packet:
 *
 *   +0  u16 magic 0x4E47 ("GN")     +8  u32 t_send   sender's clock, ms
 *   +2  u8  type                    +12 u32 t_echo   last t_send received from the peer
 *   +3  u8  flags (bit0: t_echo valid)  +16 u16 echo_delay_ms  how long we held it
 *   +4  u32 session id (0 in HELLO)
 *
 * followed by a type-specific body (see the send_* / on_* pairs below). t_send/t_echo/echo_delay
 * give an RTT sample on every packet, TCP-timestamp style, with no dedicated ping. Frames on the
 * wire are INDEXES from cfg.first_frame (u32); the API speaks the session's signed frame numbers.
 */
#include "gw_net.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define WIN32_LEAN_AND_MEAN
#include <winsock2.h>
#include <ws2tcpip.h>
#include <windows.h>
#pragma comment(lib, "ws2_32.lib")

#define MAGIC 0x4E47u
#define HDR_LEN 18

enum {
  T_HELLO = 1, T_ACCEPT, T_REFUSE, T_READY, T_START, T_START_ACK, T_INPUT, T_QUIT
};

#define MAX_PACKET 1200
#define HELLO_INTERVAL_MS 250u
#define READY_INTERVAL_MS 150u
#define START_INTERVAL_MS 100u
#define HANDSHAKE_TIMEOUT_MS 10000u
#define MIN_SEND_GAP_MS 4u
#define MAX_CHECKSUMS 8
#define ADV_EMA 0.125f
#define MAX_PULL_PER_POLL 64

/* ---- byte helpers --------------------------------------------------------------------------- */

static void put16(uint8_t **p, uint32_t v) { (*p)[0] = (uint8_t)v; (*p)[1] = (uint8_t)(v >> 8); *p += 2; }
static void put32(uint8_t **p, uint32_t v) {
  (*p)[0] = (uint8_t)v; (*p)[1] = (uint8_t)(v >> 8); (*p)[2] = (uint8_t)(v >> 16);
  (*p)[3] = (uint8_t)(v >> 24); *p += 4;
}
static void put64(uint8_t **p, uint64_t v) { put32(p, (uint32_t)v); put32(p, (uint32_t)(v >> 32)); }
static uint32_t get16(const uint8_t **p) { uint32_t v = (*p)[0] | ((uint32_t)(*p)[1] << 8); *p += 2; return v; }
static uint32_t get32(const uint8_t **p) {
  uint32_t v = (*p)[0] | ((uint32_t)(*p)[1] << 8) | ((uint32_t)(*p)[2] << 16) | ((uint32_t)(*p)[3] << 24);
  *p += 4;
  return v;
}
static uint64_t get64(const uint8_t **p) { uint64_t lo = get32(p); uint64_t hi = get32(p); return lo | (hi << 32); }

/* ---- hashing -------------------------------------------------------------------------------- */

uint64_t gw_net_hash64(const void *data, size_t len, uint64_t seed) {
  const uint8_t *b = (const uint8_t *)data;
  uint64_t h = seed ? seed : 0xcbf29ce484222325ull;
  size_t i;
  for (i = 0; i < len; ++i) { h ^= b[i]; h *= 0x100000001b3ull; }
  return h;
}

uint64_t gw_net_hash_file(const char *path, size_t max_bytes) {
  FILE *f = fopen(path, "rb");
  uint64_t h = 0;
  static uint8_t buf[65536];
  size_t total = 0, n;
  if (f == NULL) return 0;
  h = 0xcbf29ce484222325ull;
  while ((n = fread(buf, 1, sizeof buf, f)) > 0) {
    if (max_bytes != 0 && total + n > max_bytes) n = max_bytes - total;
    h = gw_net_hash64(buf, n, h);
    total += n;
    if (max_bytes != 0 && total >= max_bytes) break;
  }
  fclose(f);
  return h == 0 ? 1 : h;
}

/* ---- default UDP transport ------------------------------------------------------------------ */

typedef struct udp_ctx { SOCKET s; uint16_t port; } udp_ctx;
static int g_wsa_refs;

static int udp_send(void *ctx, const gw_net_addr *to, const void *data, int len) {
  udp_ctx *u = (udp_ctx *)ctx;
  struct sockaddr_in sa;
  memset(&sa, 0, sizeof sa);
  sa.sin_family = AF_INET;
  sa.sin_addr.s_addr = htonl(to->ip);
  sa.sin_port = htons(to->port);
  return sendto(u->s, (const char *)data, len, 0, (struct sockaddr *)&sa, sizeof sa);
}

static int udp_recv(void *ctx, gw_net_addr *from, void *buf, int cap) {
  udp_ctx *u = (udp_ctx *)ctx;
  struct sockaddr_in sa;
  int sl = (int)sizeof sa;
  int n = recvfrom(u->s, (char *)buf, cap, 0, (struct sockaddr *)&sa, &sl);
  if (n < 0) {
    int e = WSAGetLastError();
    /* WSAECONNRESET on UDP is an ICMP port-unreachable for an earlier send: not our problem */
    if (e == WSAEWOULDBLOCK || e == WSAECONNRESET || e == WSAEMSGSIZE) return 0;
    return -1;
  }
  from->ip = ntohl(sa.sin_addr.s_addr);
  from->port = ntohs(sa.sin_port);
  return n;
}

static void udp_close(void *ctx) {
  udp_ctx *u = (udp_ctx *)ctx;
  if (u == NULL) return;
  if (u->s != INVALID_SOCKET) closesocket(u->s);
  free(u);
  if (--g_wsa_refs == 0) WSACleanup();
}

int gw_net_udp_open(uint32_t bind_ip, uint16_t bind_port, gw_net_transport *out) {
  WSADATA wsa;
  udp_ctx *u;
  struct sockaddr_in sa;
  int sl = (int)sizeof sa;
  u_long nb = 1;
  if (g_wsa_refs++ == 0 && WSAStartup(MAKEWORD(2, 2), &wsa) != 0) { g_wsa_refs = 0; return -1; }
  u = (udp_ctx *)calloc(1, sizeof *u);
  if (u == NULL) { if (--g_wsa_refs == 0) WSACleanup(); return -1; }
  u->s = socket(AF_INET, SOCK_DGRAM, IPPROTO_UDP);
  if (u->s == INVALID_SOCKET) { udp_close(u); return -1; }
  ioctlsocket(u->s, FIONBIO, &nb);
  memset(&sa, 0, sizeof sa);
  sa.sin_family = AF_INET;
  sa.sin_addr.s_addr = htonl(bind_ip);
  sa.sin_port = htons(bind_port);
  if (bind(u->s, (struct sockaddr *)&sa, sizeof sa) != 0) { udp_close(u); return -1; }
  if (getsockname(u->s, (struct sockaddr *)&sa, &sl) == 0) u->port = ntohs(sa.sin_port);
  out->ctx = u;
  out->send = udp_send;
  out->recv = udp_recv;
  out->close = udp_close;
  return 0;
}

uint16_t gw_net_udp_local_port(const gw_net_transport *t) {
  return t != NULL && t->ctx != NULL && t->send == udp_send ? ((udp_ctx *)t->ctx)->port : 0;
}

int gw_net_addr_parse(const char *text, uint16_t default_port, gw_net_addr *out) {
  unsigned a, b, c, d, port = default_port;
  int n = sscanf(text, "%u.%u.%u.%u:%u", &a, &b, &c, &d, &port);
  if (n < 4 || a > 255 || b > 255 || c > 255 || d > 255 || port == 0 || port > 65535) return -1;
  out->ip = (a << 24) | (b << 16) | (c << 8) | d;
  out->port = (uint16_t)port;
  return 0;
}

/* ---- the session object --------------------------------------------------------------------- */

struct gw_net {
  gw_net_config cfg;
  gw_net_transport tp;
  int is_host;
  int state;
  gw_net_addr peer;
  int peer_locked;
  uint32_t session;
  char reason[96];

  /* clock */
  uint32_t (*now_fn)(void *);
  uint32_t last_rx_ms, last_rx_tsend, last_rx_at, have_echo;
  uint32_t last_send_ms;
  uint32_t rtt_ms;                   /* smoothed, 0 = no sample yet */
  int interrupted;

  /* handshake */
  uint32_t t_begin, next_hello, next_ready, next_start;
  uint32_t start_time;
  int start_time_valid, start_acked, started_fired;
  uint8_t host_slots, guest_slots;
  uint32_t seed;
  uint8_t input_delay;
  uint8_t blob[GW_NET_MAX_BLOB];
  uint16_t blob_len;
  int have_remote_cfg;
  int held;                          /* cfg.hold_start and gw_net_release not called yet */
  int peer_ready;                    /* host: the guest's READY arrived while we were held */

  /* slots and framing */
  int32_t first;                     /* the session's first frame number */
  int pb;                            /* payload bytes per slot per frame */
  int lslots[GW_NET_MAX_SLOTS], nl, rslots[GW_NET_MAX_SLOTS], nr;
  int max_per_packet;

  /* local inputs (sent, awaiting ack): [frame % RING][slot][payload] */
  uint8_t (*local)[GW_NET_MAX_SLOTS][GW_NET_MAX_PAYLOAD];
  uint32_t local_next;               /* next frame index the session will supply */
  uint32_t peer_ack;                 /* frame indexes < peer_ack are acknowledged */

  /* remote inputs (received) */
  uint8_t (*remote)[GW_NET_MAX_SLOTS][GW_NET_MAX_PAYLOAD];
  uint32_t rtag[GW_NET_RING];        /* frame index + 1 held in the slot, 0 = empty */
  uint32_t remote_next;              /* next frame index to deliver */

  /* time sync */
  uint32_t local_frame;              /* index */
  float adv_local, adv_remote;
  int have_adv;
  int cooldown;
  float frame_ms;

  /* checksums (by frame index) */
  uint32_t lchk[GW_NET_RING], lchk_tag[GW_NET_RING];
  uint32_t rchk[GW_NET_RING], rchk_tag[GW_NET_RING];
  uint32_t recent[MAX_CHECKSUMS];
  int nrecent;
  uint32_t next_chk;                 /* pull model: next frame index to ask the session for */
  int32_t desync_frame;
  int desync_fired;

  gw_net_stats st;
};

static uint32_t wall_now_ms(void *user) {
  LARGE_INTEGER f, c;
  (void)user;
  QueryPerformanceFrequency(&f);
  QueryPerformanceCounter(&c);
  return (uint32_t)((uint64_t)c.QuadPart * 1000ull / (uint64_t)f.QuadPart);
}

static uint32_t now(const gw_net *n) { return n->now_fn(n->cfg.cb.user); }
static int reached(uint32_t t, uint32_t at) { return (int32_t)(t - at) >= 0; }
static int32_t to_frame(const gw_net *n, uint32_t idx) { return n->first + (int32_t)idx; }

static void set_reason(gw_net *n, const char *msg) {
  strncpy(n->reason, msg != NULL ? msg : "", sizeof n->reason - 1);
  n->reason[sizeof n->reason - 1] = 0;
}
static void fire(gw_net *n, int ev, const char *msg) {
  if (n->cfg.cb.event != NULL) n->cfg.cb.event(n->cfg.cb.user, ev, msg);
}

/* ---- packet building / sending -------------------------------------------------------------- */

static uint8_t *begin_packet(gw_net *n, uint8_t *buf, int type) {
  uint8_t *p = buf;
  uint32_t t = now(n);
  put16(&p, MAGIC);
  *p++ = (uint8_t)type;
  *p++ = n->have_echo ? 1 : 0;
  put32(&p, n->session);
  put32(&p, t);
  put32(&p, n->have_echo ? n->last_rx_tsend : 0);
  put16(&p, n->have_echo ? (t - n->last_rx_at) & 0xFFFFu : 0);
  return p;
}

static void send_packet(gw_net *n, const uint8_t *buf, int len) {
  if (n->tp.send != NULL && n->peer_locked) {
    n->tp.send(n->tp.ctx, &n->peer, buf, len);
    n->st.packets_sent++;
    n->last_send_ms = now(n);
  }
}

static void send_simple(gw_net *n, int type) {
  uint8_t buf[MAX_PACKET];
  uint8_t *p = begin_packet(n, buf, type);
  send_packet(n, buf, (int)(p - buf));
}

static void send_refuse_to(gw_net *n, const gw_net_addr *to, int code, const char *msg) {
  uint8_t buf[MAX_PACKET];
  uint8_t *p;
  int len = (int)strlen(msg);
  uint32_t save = n->session;
  n->session = 0;
  p = begin_packet(n, buf, T_REFUSE);
  n->session = save;
  if (len > 90) len = 90;
  *p++ = (uint8_t)code;
  *p++ = (uint8_t)len;
  memcpy(p, msg, (size_t)len);
  p += len;
  n->tp.send(n->tp.ctx, to, buf, (int)(p - buf));
  n->st.packets_sent++;
}

static void send_hello(gw_net *n) {
  uint8_t buf[MAX_PACKET];
  uint8_t *p = begin_packet(n, buf, T_HELLO);
  put16(&p, GW_NET_PROTOCOL_VERSION);
  put64(&p, n->cfg.exe_hash);
  put64(&p, n->cfg.iso_hash);
  put64(&p, n->cfg.mods_hash);
  *p++ = (uint8_t)n->pb;
  put32(&p, (uint32_t)n->first);
  /* optional tail (older hosts ignore it): the guest's own choices */
  *p++ = n->cfg.guest_info_len;
  if (n->cfg.guest_info_len != 0) {
    memcpy(p, n->cfg.guest_info, n->cfg.guest_info_len);
    p += n->cfg.guest_info_len;
  }
  send_packet(n, buf, (int)(p - buf));
}

/* Host: agree the start time (the guest is ready, and so are we). */
static void begin_start(gw_net *n) {
  uint32_t delay = 2u * n->rtt_ms + 150u;
  if (delay < 300u) delay = 300u;
  n->start_time = now(n) + delay;
  n->start_time_valid = 1;
  n->state = GW_NET_STARTING;
  n->next_start = now(n);
  fire(n, GW_NET_EV_STARTING, NULL);
}

static void send_accept(gw_net *n) {
  uint8_t buf[MAX_PACKET];
  uint8_t *p = begin_packet(n, buf, T_ACCEPT);
  put16(&p, GW_NET_PROTOCOL_VERSION);
  put32(&p, n->seed);
  *p++ = n->input_delay;
  *p++ = n->host_slots;
  *p++ = n->guest_slots;
  put16(&p, n->blob_len);
  memcpy(p, n->blob, n->blob_len);
  p += n->blob_len;
  send_packet(n, buf, (int)(p - buf));
}

static void send_start(gw_net *n) {
  uint8_t buf[MAX_PACKET];
  uint8_t *p = begin_packet(n, buf, T_START);
  uint32_t t = now(n);
  int32_t remaining = (int32_t)(n->start_time - t);
  put32(&p, remaining > 0 ? (uint32_t)remaining : 0u);  /* time until start, on the host's clock */
  send_packet(n, buf, (int)(p - buf));
}

/* INPUT body: u32 local_frame idx | s8 adv | u32 ack_next | u32 first idx | u8 count |
 *             count x (nl x payload bytes) | u8 nchk | nchk x (u32 frame idx, u32 hash) */
static void send_inputs(gw_net *n) {
  uint8_t buf[MAX_PACKET];
  uint8_t *p = begin_packet(n, buf, T_INPUT);
  uint32_t first = n->peer_ack, avail = n->local_next - n->peer_ack;
  uint32_t count = avail > (uint32_t)n->max_per_packet ? (uint32_t)n->max_per_packet : avail;
  uint32_t i;
  int j, adv;
  adv = (int)(n->adv_local + (n->adv_local >= 0 ? 0.5f : -0.5f));
  if (adv > 127) adv = 127;
  if (adv < -127) adv = -127;
  put32(&p, n->local_frame);
  *p++ = (uint8_t)(int8_t)adv;
  put32(&p, n->remote_next);           /* every remote frame below this has been delivered */
  put32(&p, first);
  *p++ = (uint8_t)count;
  for (i = 0; i < count; ++i)
    for (j = 0; j < n->nl; ++j) {
      memcpy(p, n->local[(first + i) % GW_NET_RING][n->lslots[j]], (size_t)n->pb);
      p += n->pb;
    }
  {
    int c = n->nrecent;
    *p++ = (uint8_t)c;
    for (j = 0; j < c; ++j) {
      uint32_t f = n->recent[j];
      put32(&p, f);
      put32(&p, n->lchk[f % GW_NET_RING]);
    }
  }
  n->st.resent_frames += count > 1 ? count - 1 : 0;
  send_packet(n, buf, (int)(p - buf));
}

/* ---- slot lists ----------------------------------------------------------------------------- */

static void slots_from_mask(uint8_t mask, int *list, int *count) {
  int i;
  *count = 0;
  for (i = 0; i < GW_NET_MAX_SLOTS; ++i)
    if (mask & (1u << i)) list[(*count)++] = i;
}

static void setup_slots(gw_net *n) {
  uint8_t lm = n->is_host ? n->host_slots : n->guest_slots;
  uint8_t rm = n->is_host ? n->guest_slots : n->host_slots;
  slots_from_mask(lm, n->lslots, &n->nl);
  slots_from_mask(rm, n->rslots, &n->nr);
  n->max_per_packet = n->nl > 0 ? 1000 / (n->pb * n->nl) : 1;
  if (n->max_per_packet > GW_NET_MAX_INPUTS_PER_PACKET) n->max_per_packet = GW_NET_MAX_INPUTS_PER_PACKET;
  if (n->max_per_packet < 1) n->max_per_packet = 1;
}

/* ---- construction --------------------------------------------------------------------------- */

static gw_net *make(const gw_net_config *cfg, const gw_net_transport *t, int is_host) {
  gw_net *n;
  int pb;
  if (cfg == NULL || t == NULL || t->send == NULL || t->recv == NULL) return NULL;
  pb = cfg->payload_bytes != 0 ? cfg->payload_bytes : GW_NET_DEFAULT_PAYLOAD;
  if (pb > GW_NET_MAX_PAYLOAD) return NULL;
  n = (gw_net *)calloc(1, sizeof *n);
  if (n == NULL) return NULL;
  n->local = (uint8_t (*)[GW_NET_MAX_SLOTS][GW_NET_MAX_PAYLOAD])calloc(GW_NET_RING, sizeof *n->local);
  n->remote = (uint8_t (*)[GW_NET_MAX_SLOTS][GW_NET_MAX_PAYLOAD])calloc(GW_NET_RING, sizeof *n->remote);
  if (n->local == NULL || n->remote == NULL) { free(n->local); free(n->remote); free(n); return NULL; }
  n->cfg = *cfg;
  n->tp = *t;
  n->is_host = is_host;
  n->pb = pb;
  n->first = cfg->first_frame;
  n->now_fn = cfg->now_ms != NULL ? cfg->now_ms : wall_now_ms;
  if (n->cfg.disconnect_timeout_ms == 0) n->cfg.disconnect_timeout_ms = 5000;
  if (n->cfg.notify_timeout_ms == 0) n->cfg.notify_timeout_ms = 1000;
  n->frame_ms = (cfg->frame_us != 0 ? cfg->frame_us : 16667u) / 1000.0f;
  n->desync_frame = GW_NET_NO_FRAME;
  n->t_begin = now(n);
  n->last_rx_ms = n->t_begin;
  n->held = cfg->hold_start != 0;
  if (n->cfg.guest_info_len > GW_NET_MAX_GUEST_INFO) n->cfg.guest_info_len = GW_NET_MAX_GUEST_INFO;
  return n;
}

void gw_net_release(gw_net *n) {
  if (n == NULL || !n->held) return;
  n->held = 0;
  if (n->is_host) {
    if (n->state == GW_NET_ACCEPTED && n->peer_ready) begin_start(n);
  } else {
    n->next_ready = now(n);              /* READY goes out on the next poll */
  }
}

gw_net *gw_net_host(const gw_net_config *cfg, const gw_net_transport *t) {
  gw_net *n;
  if (cfg == NULL || (cfg->host_slots & cfg->guest_slots) != 0 || cfg->host_slots == 0 ||
      cfg->guest_slots == 0 || cfg->match_blob_len > GW_NET_MAX_BLOB ||
      (cfg->match_blob_len != 0 && cfg->match_blob == NULL))
    return NULL;
  n = make(cfg, t, 1);
  if (n == NULL) return NULL;
  n->state = GW_NET_LISTENING;
  n->host_slots = cfg->host_slots;
  n->guest_slots = cfg->guest_slots;
  n->seed = cfg->seed;
  n->input_delay = cfg->input_delay;
  n->blob_len = cfg->match_blob_len;
  if (n->blob_len != 0) memcpy(n->blob, cfg->match_blob, n->blob_len);
  n->have_remote_cfg = 1;
  setup_slots(n);
  return n;
}

gw_net *gw_net_join(const gw_net_config *cfg, const gw_net_transport *t, const gw_net_addr *host) {
  gw_net *n;
  if (host == NULL) return NULL;
  n = make(cfg, t, 0);
  if (n == NULL) return NULL;
  n->state = GW_NET_CONNECTING;
  n->peer = *host;
  n->peer_locked = 1;
  n->next_hello = n->t_begin;
  return n;
}

void gw_net_free(gw_net *n) {
  if (n == NULL) return;
  if (n->peer_locked &&
      (n->state == GW_NET_ACCEPTED || n->state == GW_NET_STARTING || n->state == GW_NET_RUNNING)) {
    int i;
    for (i = 0; i < 3; ++i) send_simple(n, T_QUIT);
  }
  if (n->tp.close != NULL) n->tp.close(n->tp.ctx);
  free(n->local);
  free(n->remote);
  free(n);
}

/* ---- receiving ------------------------------------------------------------------------------ */

static void rtt_sample(gw_net *n, uint32_t t_echo, uint32_t echo_delay) {
  uint32_t t = now(n);
  int32_t s = (int32_t)(t - t_echo - echo_delay);
  if (s < 0 || s > 10000) return;
  if (s < 1) s = 1;
  n->rtt_ms = n->rtt_ms == 0 ? (uint32_t)s : (n->rtt_ms * 7u + (uint32_t)s + 4u) / 8u;
}

static void check_checksum_pair(gw_net *n, uint32_t idx) {
  uint32_t slot = idx % GW_NET_RING;
  int32_t frame = to_frame(n, idx);
  if (n->lchk_tag[slot] != idx + 1 || n->rchk_tag[slot] != idx + 1) return;
  if (n->lchk[slot] == n->rchk[slot]) return;
  if (n->desync_frame == GW_NET_NO_FRAME || frame < n->desync_frame) n->desync_frame = frame;
  if (!n->desync_fired) {
    n->desync_fired = 1;
    if (n->cfg.cb.desync != NULL) n->cfg.cb.desync(n->cfg.cb.user, frame, n->lchk[slot], n->rchk[slot]);
  }
}

static void go_dead(gw_net *n, const char *msg) {
  if (n->state == GW_NET_DEAD) return;
  n->state = GW_NET_DEAD;
  set_reason(n, msg);
  fire(n, GW_NET_EV_DISCONNECTED, msg);
}

static void deliver_remote(gw_net *n) {
  for (;;) {
    uint32_t slot = n->remote_next % GW_NET_RING;
    int j;
    if (n->rtag[slot] != n->remote_next + 1) break;
    for (j = 0; j < n->nr; ++j)
      if (n->cfg.cb.remote_input != NULL)
        n->cfg.cb.remote_input(n->cfg.cb.user, to_frame(n, n->remote_next), n->rslots[j], n->remote[slot][n->rslots[j]]);
    n->rtag[slot] = 0;
    n->remote_next++;
    n->st.inputs_delivered++;
  }
}

static void on_input(gw_net *n, const uint8_t *p, const uint8_t *end) {
  uint32_t lf, ack_next, first, count, i, nchk;
  int adv_remote;
  int j, gap = 0;
  size_t per_frame = (size_t)n->nr * (size_t)n->pb;
  if (n->state != GW_NET_STARTING && n->state != GW_NET_RUNNING) return;
  if (end - p < 4 + 1 + 4 + 4 + 1) { n->st.bad_packets++; return; }
  lf = get32(&p);
  adv_remote = (int8_t)*p++;
  ack_next = get32(&p);
  first = get32(&p);
  count = *p++;
  if ((size_t)(end - p) < (size_t)count * per_frame + 1) { n->st.bad_packets++; return; }
  n->start_acked = 1;                    /* an input packet also acknowledges START */

  if (ack_next > n->peer_ack && ack_next <= n->local_next) n->peer_ack = ack_next;

  for (i = 0; i < count; ++i) {
    uint32_t frame = first + i;
    uint32_t slot = frame % GW_NET_RING;
    if (frame < n->remote_next) {        /* already delivered: a redundant resend */
      p += per_frame;
      n->st.duplicates++;
      continue;
    }
    if (frame >= n->remote_next + GW_NET_RING) {
      p += per_frame;                    /* too far ahead to buffer; it will be resent */
      continue;
    }
    if (frame > n->remote_next) gap = 1;
    if (n->rtag[slot] == frame + 1) {
      p += per_frame;
      n->st.duplicates++;
      continue;
    }
    for (j = 0; j < n->nr; ++j) {
      memcpy(n->remote[slot][n->rslots[j]], p, (size_t)n->pb);
      p += n->pb;
    }
    n->rtag[slot] = frame + 1;
  }
  if (gap) n->st.out_of_order++;
  deliver_remote(n);

  nchk = *p++;
  if ((size_t)(end - p) >= (size_t)nchk * 8) {
    for (i = 0; i < nchk; ++i) {
      uint32_t f = get32(&p), h = get32(&p), slot = f % GW_NET_RING;
      n->rchk[slot] = h;
      n->rchk_tag[slot] = f + 1;
      check_checksum_pair(n, f);
    }
  }

  /* frame advantage: where the peer is NOW, given the frame it stamped and half the round trip */
  {
    float remote_now = (float)lf + (n->rtt_ms / 2.0f) / n->frame_ms;
    float adv = (float)n->local_frame - remote_now;
    if (!n->have_adv) { n->adv_local = adv; n->adv_remote = (float)adv_remote; n->have_adv = 1; }
    else {
      n->adv_local += (adv - n->adv_local) * ADV_EMA;
      n->adv_remote += ((float)adv_remote - n->adv_remote) * ADV_EMA;
    }
  }
}

static const char *refuse_check(gw_net *n, uint32_t ver, uint64_t exe, uint64_t iso, uint64_t mods,
                                int pb, int32_t first, char *why, size_t cap) {
  if (ver != GW_NET_PROTOCOL_VERSION) {
    snprintf(why, cap, "protocol version mismatch (host %u, you %u)", GW_NET_PROTOCOL_VERSION, ver);
    return why;
  }
  if (exe != n->cfg.exe_hash) {
    snprintf(why, cap, "different melee-pc.exe build (host %08x, you %08x)",
             (unsigned)n->cfg.exe_hash, (unsigned)exe);
    return why;
  }
  if (iso != n->cfg.iso_hash) {
    snprintf(why, cap, "different disc image (host %08x, you %08x)", (unsigned)n->cfg.iso_hash, (unsigned)iso);
    return why;
  }
  if (mods != n->cfg.mods_hash) {
    snprintf(why, cap, "different mod pack (host %08x, you %08x)", (unsigned)n->cfg.mods_hash, (unsigned)mods);
    return why;
  }
  if (pb != n->pb || first != n->first) {
    snprintf(why, cap, "different netplay settings (input size %d vs %d, first frame %d vs %d)", n->pb, pb,
             (int)n->first, (int)first);
    return why;
  }
  return NULL;
}

static void on_packet(gw_net *n, const gw_net_addr *from, const uint8_t *buf, int len) {
  const uint8_t *p = buf, *end = buf + len;
  uint32_t magic, session, t_send, t_echo, echo_delay;
  int type, flags;
  if (len < HDR_LEN) { n->st.bad_packets++; return; }
  magic = get16(&p);
  if (magic != MAGIC) { n->st.bad_packets++; return; }
  type = *p++;
  flags = *p++;
  session = get32(&p);
  t_send = get32(&p);
  t_echo = get32(&p);
  echo_delay = get16(&p);

  /* Host, not yet locked to a guest: only a HELLO means anything. */
  if (n->is_host && !n->peer_locked) {
    char why[96];
    uint32_t ver;
    uint64_t exe, iso, mods;
    int pb;
    int32_t first;
    if (type != T_HELLO || end - p < 2 + 24 + 1 + 4) return;
    ver = get16(&p); exe = get64(&p); iso = get64(&p); mods = get64(&p);
    pb = *p++;
    first = (int32_t)get32(&p);
    if (refuse_check(n, ver, exe, iso, mods, pb, first, why, sizeof why) != NULL) {
      send_refuse_to(n, from, 1, why);
      return;                            /* stay listening: a refused guest does not consume the slot */
    }
    {
      int il = end - p >= 1 ? *p++ : 0;
      if (il > end - p) il = (int)(end - p);
      if (il > GW_NET_MAX_GUEST_INFO) il = GW_NET_MAX_GUEST_INFO;
      if (n->cfg.cb.guest_hello != NULL) {
        n->cfg.cb.guest_hello(n->cfg.cb.user, p, il, n->blob, &n->blob_len, GW_NET_MAX_BLOB);
      }
    }
    n->peer = *from;
    n->peer_locked = 1;
    n->session = (now(n) * 2654435761u) ^ (from->ip * 40503u) ^ ((uint32_t)from->port << 16) ^
                 (uint32_t)(uintptr_t)n;
    if (n->session == 0) n->session = 1;
    n->state = GW_NET_ACCEPTED;
    n->last_rx_ms = now(n);
    n->last_rx_tsend = t_send; n->last_rx_at = now(n); n->have_echo = 1;
    send_accept(n);
    return;
  }

  if (from->ip != n->peer.ip || from->port != n->peer.port) {
    if (n->is_host && type == T_HELLO) send_refuse_to(n, from, 2, "session full");
    return;
  }

  if (type == T_HELLO) {
    /* A HELLO repeated because our ACCEPT was lost (it carries session 0, so it cannot pass the
       session check below). Answer it from the locked peer, ignore it otherwise. */
    if (n->is_host && n->state == GW_NET_ACCEPTED) send_accept(n);
    return;
  }

  if (!n->is_host && n->state == GW_NET_CONNECTING) {
    /* the guest hears only ACCEPT or REFUSE from the host it dialled */
    if (type == T_REFUSE && end - p >= 2) {
      int l;
      char why[96];
      p++;                               /* code */
      l = *p++;
      if (l > end - p) l = (int)(end - p);
      if (l > 90) l = 90;
      memcpy(why, p, (size_t)l);
      why[l] = 0;
      n->state = GW_NET_REFUSED;
      set_reason(n, why);
      fire(n, GW_NET_EV_REFUSED, why);
      return;
    }
    if (type == T_ACCEPT) {
      uint32_t ver, bl;
      if (end - p < 2 + 4 + 3 + 2) { n->st.bad_packets++; return; }
      ver = get16(&p);
      if (ver != GW_NET_PROTOCOL_VERSION) {
        n->state = GW_NET_REFUSED;
        set_reason(n, "host speaks a different protocol version");
        fire(n, GW_NET_EV_REFUSED, n->reason);
        return;
      }
      n->seed = get32(&p);
      n->input_delay = *p++;
      n->host_slots = *p++;
      n->guest_slots = *p++;
      bl = get16(&p);
      if (bl > GW_NET_MAX_BLOB || (size_t)(end - p) < bl) { n->st.bad_packets++; return; }
      n->blob_len = (uint16_t)bl;
      memcpy(n->blob, p, bl);
      n->session = session;
      n->have_remote_cfg = 1;
      setup_slots(n);
      n->state = GW_NET_ACCEPTED;
      n->last_rx_ms = now(n);
      n->last_rx_tsend = t_send; n->last_rx_at = now(n); n->have_echo = 1;
      if (flags & 1) rtt_sample(n, t_echo, echo_delay);
      n->next_ready = now(n);
      fire(n, GW_NET_EV_ACCEPTED, NULL);
    }
    return;
  }

  if (session != n->session) return;     /* a stray from an old session */

  n->last_rx_ms = now(n);
  n->last_rx_tsend = t_send;
  n->last_rx_at = now(n);
  n->have_echo = 1;
  n->st.packets_received++;
  if (flags & 1) rtt_sample(n, t_echo, echo_delay);
  if (n->interrupted) { n->interrupted = 0; fire(n, GW_NET_EV_RESUMED, NULL); }

  switch (type) {
  case T_ACCEPT:
    break;                               /* a duplicate */
  case T_READY:
    if (n->is_host && (n->state == GW_NET_ACCEPTED)) {
      if (n->held) {
        n->peer_ready = 1;               /* start once we are released */
      } else {
        begin_start(n);
      }
    }
    break;
  case T_START:
    if (!n->is_host && (n->state == GW_NET_ACCEPTED) && end - p >= 4) {
      uint32_t delay = get32(&p);
      uint32_t half = n->rtt_ms / 2u;
      n->start_time = now(n) + (delay > half ? delay - half : 0u);
      n->start_time_valid = 1;
      n->state = GW_NET_STARTING;
      fire(n, GW_NET_EV_STARTING, NULL);
    }
    if (!n->is_host && (n->state == GW_NET_STARTING || n->state == GW_NET_RUNNING)) send_simple(n, T_START_ACK);
    break;
  case T_START_ACK:
    n->start_acked = 1;
    break;
  case T_INPUT:
    on_input(n, p, end);
    break;
  case T_QUIT:
    go_dead(n, "peer quit");
    break;
  default:
    n->st.bad_packets++;
    break;
  }
}

/* ---- pulling from the session --------------------------------------------------------------- */

static int accept_local(gw_net *n, uint32_t idx, const uint8_t (*src)[GW_NET_MAX_PAYLOAD]) {
  int j;
  if (idx != n->local_next || n->local_next - n->peer_ack >= GW_NET_RING - 8) return -1;
  for (j = 0; j < n->nl; ++j) memcpy(n->local[idx % GW_NET_RING][n->lslots[j]], src[n->lslots[j]], (size_t)n->pb);
  n->local_next++;
  return 0;
}

static void pull_local(gw_net *n) {
  int k, j;
  if (n->cfg.cb.local_input == NULL || !n->have_remote_cfg || n->nl == 0) return;
  for (k = 0; k < MAX_PULL_PER_POLL; ++k) {
    uint8_t tmp[GW_NET_MAX_SLOTS][GW_NET_MAX_PAYLOAD];
    int ok = 1;
    if (n->local_next - n->peer_ack >= GW_NET_RING - 8) break;
    for (j = 0; j < n->nl && ok; ++j)
      ok = n->cfg.cb.local_input(n->cfg.cb.user, to_frame(n, n->local_next), n->lslots[j], tmp[n->lslots[j]]);
    if (!ok) break;
    accept_local(n, n->local_next, (const uint8_t (*)[GW_NET_MAX_PAYLOAD])tmp);
  }
}

static void report_idx(gw_net *n, uint32_t idx, uint32_t hash) {
  uint32_t slot = idx % GW_NET_RING;
  int i;
  n->lchk[slot] = hash;
  n->lchk_tag[slot] = idx + 1;
  if (n->nrecent == MAX_CHECKSUMS) {
    for (i = 1; i < MAX_CHECKSUMS; ++i) n->recent[i - 1] = n->recent[i];
    n->nrecent--;
  }
  n->recent[n->nrecent++] = idx;
  check_checksum_pair(n, idx);
}

/* The state at the START of frame f depends on the inputs of frames < f, so f is final once every
 * remote input below it has been delivered: f <= remote_next. */
static void pull_checksums(gw_net *n) {
  int k;
  if (n->cfg.cb.checksum == NULL) return;
  for (k = 0; k < 8 && n->next_chk <= n->remote_next; ++k) {
    uint32_t h;
    if (n->cfg.cb.checksum(n->cfg.cb.user, to_frame(n, n->next_chk), &h)) {
      report_idx(n, n->next_chk, h);
      n->next_chk++;
    } else if (n->remote_next - n->next_chk > 8) {
      n->next_chk++;                     /* too old: the session no longer has it */
    } else {
      break;                             /* not simulated yet: ask again next poll */
    }
  }
}

/* ---- the frame poll ------------------------------------------------------------------------- */

void gw_net_poll(gw_net *n, int32_t local_frame) {
  uint8_t buf[MAX_PACKET + 64];
  gw_net_addr from;
  int i, r;
  uint32_t t;
  if (n == NULL || n->state == GW_NET_DEAD || n->state == GW_NET_REFUSED) return;
  n->local_frame = local_frame > n->first ? (uint32_t)(local_frame - n->first) : 0u;
  if (n->cooldown > 0) n->cooldown--;

  for (i = 0; i < 64; ++i) {
    r = n->tp.recv(n->tp.ctx, &from, buf, (int)sizeof buf);
    if (r < 0) { go_dead(n, "network error"); return; }
    if (r == 0) break;
    on_packet(n, &from, buf, r);
    if (n->state == GW_NET_DEAD || n->state == GW_NET_REFUSED) return;
  }
  t = now(n);

  /* handshake timers */
  if (n->state == GW_NET_CONNECTING) {
    if (reached(t, n->t_begin + (n->cfg.handshake_timeout_ms != 0 ? n->cfg.handshake_timeout_ms : HANDSHAKE_TIMEOUT_MS))) { go_dead(n, "handshake timed out (no host answered)"); return; }
    if (reached(t, n->next_hello)) { send_hello(n); n->next_hello = t + HELLO_INTERVAL_MS; }
  } else if (n->state == GW_NET_ACCEPTED) {
    if (!n->is_host && !n->held && reached(t, n->next_ready)) { send_simple(n, T_READY); n->next_ready = t + READY_INTERVAL_MS; }
  } else if (n->state == GW_NET_STARTING) {
    if (n->is_host && !n->start_acked && reached(t, n->next_start)) { send_start(n); n->next_start = t + START_INTERVAL_MS; }
  }

  /* start arrival */
  if (n->state == GW_NET_STARTING && n->start_time_valid && reached(t, n->start_time)) {
    n->state = GW_NET_RUNNING;
    if (!n->started_fired) { n->started_fired = 1; fire(n, GW_NET_EV_STARTED, NULL); }
  }

  /* silence */
  if (n->peer_locked && n->state >= GW_NET_ACCEPTED && n->state < GW_NET_DEAD) {
    uint32_t silent = t - n->last_rx_ms;
    /* before the start (hold_start): the peer may be loading the match and not polling at all */
    uint32_t limit = (n->cfg.hold_start && n->state < GW_NET_STARTING) ? GW_NET_HOLD_TIMEOUT_MS
                                                                      : n->cfg.disconnect_timeout_ms;
    if (silent >= limit) { go_dead(n, "peer timed out"); return; }
    if (silent >= n->cfg.notify_timeout_ms && !n->interrupted) { n->interrupted = 1; fire(n, GW_NET_EV_INTERRUPTED, NULL); }
  }

  if (n->state == GW_NET_RUNNING) {
    pull_local(n);
    pull_checksums(n);
    if (reached(t, n->last_send_ms + MIN_SEND_GAP_MS)) send_inputs(n);
  }
}

/* ---- session-facing API --------------------------------------------------------------------- */

int gw_net_submit_local(gw_net *n, int32_t frame, const gw_net_input in[GW_NET_MAX_SLOTS]) {
  uint8_t tmp[GW_NET_MAX_SLOTS][GW_NET_MAX_PAYLOAD];
  int j;
  if (n == NULL || !n->have_remote_cfg || n->nl == 0) return -1;
  if (frame < n->first || (uint32_t)(frame - n->first) != n->local_next) return -2;
  for (j = 0; j < n->nl; ++j) memcpy(tmp[n->lslots[j]], in[n->lslots[j]].b, GW_NET_MAX_PAYLOAD);
  return accept_local(n, n->local_next, (const uint8_t (*)[GW_NET_MAX_PAYLOAD])tmp) == 0 ? 0 : -3;
}

int gw_net_state(const gw_net *n) { return n != NULL ? n->state : GW_NET_IDLE; }
int gw_net_started(const gw_net *n) { return n != NULL && n->state == GW_NET_RUNNING; }
uint32_t gw_net_start_time_ms(const gw_net *n) { return n->start_time; }
uint8_t gw_net_local_slots(const gw_net *n) { return n->is_host ? n->host_slots : n->guest_slots; }
uint8_t gw_net_remote_slots(const gw_net *n) { return n->is_host ? n->guest_slots : n->host_slots; }
const char *gw_net_last_reason(const gw_net *n) { return n->reason; }

int gw_net_remote_config(const gw_net *n, gw_net_config *out_cfg, void *blob_buf, int blob_cap) {
  if (n == NULL || !n->have_remote_cfg) return 0;
  if (out_cfg != NULL) {
    memset(out_cfg, 0, sizeof *out_cfg);
    out_cfg->seed = n->seed;
    out_cfg->input_delay = n->input_delay;
    out_cfg->host_slots = n->host_slots;
    out_cfg->guest_slots = n->guest_slots;
    out_cfg->match_blob_len = n->blob_len;
    out_cfg->first_frame = n->first;
    out_cfg->payload_bytes = (uint8_t)n->pb;
  }
  if (blob_buf != NULL && blob_cap >= (int)n->blob_len) memcpy(blob_buf, n->blob, n->blob_len);
  return 1;
}

int32_t gw_net_remote_confirmed_frame(const gw_net *n) { return n->first + (int32_t)n->remote_next - 1; }
uint32_t gw_net_unacked(const gw_net *n) { return n->local_next - n->peer_ack; }
uint32_t gw_net_rtt_ms(const gw_net *n) { return n->rtt_ms; }
uint32_t gw_net_silent_ms(const gw_net *n) { return now(n) - n->last_rx_ms; }
float gw_net_frame_advantage(const gw_net *n) { return n->adv_local; }

/* GGPO's rule: the peer that is ahead waits half the gap between the two advantage reports. Each
 * side sees itself as +A and the other as -A, so (local - remote) / 2 is A frames for the peer that
 * is ahead and negative for the one that is behind. After a recommendation the estimate needs time
 * to reflect the stall, so recommendations are spaced out. */
int gw_net_recommend_wait(gw_net *n) {
  int w;
  if (n == NULL || !n->have_adv || n->cooldown > 0 || n->state != GW_NET_RUNNING) return 0;
  w = (int)((n->adv_local - n->adv_remote) / 2.0f);
  if (w < 1) return 0;
  if (w > 8) w = 8;
  n->cooldown = 2 * w + 6;
  return w;
}

void gw_net_report_checksum(gw_net *n, int32_t frame, uint32_t hash) {
  if (frame < n->first) return;
  report_idx(n, (uint32_t)(frame - n->first), hash);
}

int32_t gw_net_desync_frame(const gw_net *n) { return n->desync_frame; }
void gw_net_get_stats(const gw_net *n, gw_net_stats *out) { *out = n->st; }
