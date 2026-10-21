/* melee_net oracle: the shipping gw_net.c (copied verbatim by build.sh) driven by the simulated
 * network and test peers of gw_net_tests.c, printing a transcript of every datagram handed to the
 * network (time, endpoints, length, FNV-1a 64 of the bytes with the session id masked - the session
 * id mixes in a heap address), every lifecycle event, every lobby message, and a summary per
 * scenario. tests/oracle.rs runs the same scenarios against the Rust port and requires the
 * identical transcript. The harness below follows gw_net_tests.c line for line (simnet, peer,
 * session_tick), plus logging and a lobby/hold scenario. */
#include "gw_net.c"

#include <stdio.h>
#include <string.h>
#include <stdlib.h>

#define FIRST (-123)
#define SIM_ENDPOINTS 3
#define SIM_QUEUE 4096

typedef struct simpkt { uint32_t at; int to; gw_net_addr from; int len; uint8_t data[1300]; } simpkt;
typedef struct simnet {
  uint32_t now; uint64_t time_us; uint64_t rng;
  int loss_pct, dup_pct; uint32_t base_ms, jitter_ms; int blackhole; int old_heap_peer;
  simpkt q[SIM_QUEUE]; int nq; uint32_t sent, dropped;
} simnet;
typedef struct simep { simnet *sim; int id; } simep;

static uint32_t rnd(simnet *s) {
  s->rng ^= s->rng >> 12; s->rng ^= s->rng << 25; s->rng ^= s->rng >> 27;
  return (uint32_t)((s->rng * 2685821657736338717ull) >> 32);
}
static void sim_init(simnet *s, uint64_t seed) { memset(s, 0, sizeof *s); s->rng = seed ? seed : 88172645463325252ull; s->base_ms = 20; }
static gw_net_addr sim_addr(int id) { gw_net_addr a; a.ip = (10u << 24) | (uint32_t)(id + 1); a.port = 7000; return a; }
static void sim_step(simnet *s) { s->time_us += 16667; s->now = (uint32_t)(s->time_us / 1000); }

static void sim_enqueue(simnet *s, int from_id, int to, const void *data, int len) {
  simpkt *p;
  if (s->nq >= SIM_QUEUE || len > (int)sizeof p->data) return;
  p = &s->q[s->nq++];
  p->at = s->now + s->base_ms + (s->jitter_ms ? rnd(s) % (s->jitter_ms + 1) : 0);
  p->to = to; p->from = sim_addr(from_id); p->len = len;
  memcpy(p->data, data, (size_t)len);
  if (s->old_heap_peer == from_id + 1 && len >= 20 && (p->data[2] == 1 || p->data[2] == 2)) { p->data[18] = 5; p->data[19] = 0; }
}

static void log_send(simnet *s, int from, int dest, const uint8_t *data, int len) {
  uint8_t m[1400];
  int n = len < (int)sizeof m ? len : (int)sizeof m;
  memcpy(m, data, (size_t)n);
  if (n >= 8) memset(m + 4, 0, 4);
  printf("%u %d>%d %d %016llx\n", s->now, from, dest, len, (unsigned long long)gw_net_hash64(m, (size_t)n, 0));
}

static int sim_send(void *ctx, const gw_net_addr *to, const void *data, int len) {
  simep *e = (simep *)ctx; simnet *s = e->sim;
  int dest = (int)(to->ip & 0xFF) - 1;
  s->sent++;
  log_send(s, e->id, dest, (const uint8_t *)data, len);
  if (dest < 0 || dest >= SIM_ENDPOINTS) return len;
  if (s->blackhole || (int)(rnd(s) % 100) < s->loss_pct) { s->dropped++; return len; }
  sim_enqueue(s, e->id, dest, data, len);
  if ((int)(rnd(s) % 100) < s->dup_pct) sim_enqueue(s, e->id, dest, data, len);
  return len;
}
static int sim_recv(void *ctx, gw_net_addr *from, void *buf, int cap) {
  simep *e = (simep *)ctx; simnet *s = e->sim;
  int i, best = -1;
  for (i = 0; i < s->nq; ++i)
    if (s->q[i].to == e->id && (int32_t)(s->now - s->q[i].at) >= 0 && (best < 0 || (int32_t)(s->q[i].at - s->q[best].at) < 0)) best = i;
  if (best < 0) return 0;
  { simpkt *p = &s->q[best]; int n = p->len < cap ? p->len : cap; *from = p->from; memcpy(buf, p->data, (size_t)n); s->q[best] = s->q[--s->nq]; return n; }
}
static void sim_close(void *ctx) { free(ctx); }
static gw_net_transport sim_transport(simnet *s, int id) {
  gw_net_transport t; simep *e = (simep *)malloc(sizeof *e);
  e->sim = s; e->id = id; t.ctx = e; t.send = sim_send; t.recv = sim_recv; t.close = sim_close; return t;
}

#define HOST_SLOTS 0x13
#define GUEST_SLOTS 0x44
static int g_pb = GW_NET_DEFAULT_PAYLOAD;

typedef struct peer {
  simnet *sim; int id, sender_id; uint32_t offset; gw_net *net;
  int32_t frame, limit; float speed, acc; int stall, use_sync, stalls_taken;
  int32_t diverge_at; int report_checksums, pull;
  int rslots[GW_NET_MAX_SLOTS], nrslots; int32_t exp_frame; uint32_t delivered; int exp_idx, bad;
  int desync_calls; int32_t desync_frame; int lobby_rx;
} peer;

static void mk_payload(int32_t frame, int slot, int who, uint8_t *out) {
  uint32_t h = (uint32_t)frame * 2654435761u ^ ((uint32_t)slot * 40503u) ^ ((uint32_t)(who + 1) * 0x9E3779B1u);
  int i;
  for (i = 0; i < g_pb; ++i) { h ^= h >> 15; h *= 2246822519u; h ^= h >> 13; out[i] = (uint8_t)(h >> (i & 7)); }
}
static uint32_t chk_of(int32_t f) { return (uint32_t)f * 2654435761u ^ 0x5BD1E995u; }
static uint32_t peer_now(void *user) { peer *p = (peer *)user; return p->sim->now + p->offset; }

static void peer_remote_input(void *user, int32_t frame, int slot, const uint8_t *payload) {
  peer *p = (peer *)user; uint8_t want[GW_NET_MAX_PAYLOAD];
  if (p->bad) return;
  if (frame != p->exp_frame || slot != p->rslots[p->exp_idx]) { p->bad = 1; return; }
  mk_payload(frame, slot, p->sender_id, want);
  if (memcmp(payload, want, (size_t)g_pb) != 0) { p->bad = 2; return; }
  if (++p->exp_idx == p->nrslots) { p->exp_idx = 0; p->exp_frame++; p->delivered++; }
}
static int peer_local_input(void *user, int32_t frame, int slot, uint8_t *out) {
  peer *p = (peer *)user; if (frame >= p->frame) return 0; mk_payload(frame, slot, p->id, out); return 1;
}
static int peer_checksum(void *user, int32_t frame, uint32_t *out) {
  peer *p = (peer *)user; if (frame >= p->frame) return 0;
  *out = chk_of(frame) ^ (frame >= p->diverge_at ? 0xDEADBEEFu : 0); return 1;
}
static void peer_event(void *user, int ev, const char *msg) {
  peer *p = (peer *)user; printf("E %u %d %d %s\n", p->sim->now, p->id, ev, msg != NULL ? msg : "");
}
static void peer_desync(void *user, int32_t frame, uint32_t local, uint32_t remote) {
  peer *p = (peer *)user; p->desync_calls++; p->desync_frame = frame;
  printf("D %u %d %d %08x %08x\n", p->sim->now, p->id, (int)frame, local, remote);
}
static void peer_lobby(void *user, const uint8_t *data, int len) {
  peer *p = (peer *)user; int i;
  printf("L %u %d ", p->sim->now, p->id);
  for (i = 0; i < len; ++i) printf("%02x", data[i]);
  printf("\n");
  p->lobby_rx++;
}
static void peer_init(peer *p, simnet *s, int id, uint32_t offset) {
  int i; memset(p, 0, sizeof *p);
  p->sim = s; p->id = id; p->sender_id = id == 0 ? 1 : 0; p->offset = offset;
  p->speed = 1.0f; p->frame = FIRST; p->exp_frame = FIRST;
  p->limit = 0x7FFFFFFF; p->diverge_at = 0x7FFFFFFF; p->report_checksums = 1;
  { uint8_t rmask = (id == 0) ? GUEST_SLOTS : HOST_SLOTS; for (i = 0; i < GW_NET_MAX_SLOTS; ++i) if (rmask & (1u << i)) p->rslots[p->nrslots++] = i; }
}
static uint8_t g_blob[300];
static gw_net_config peer_cfg(peer *p, uint64_t exe, uint64_t iso, uint64_t mods) {
  gw_net_config c; memset(&c, 0, sizeof c);
  c.exe_hash = exe; c.iso_hash = iso; c.mods_hash = mods;
  c.first_frame = FIRST; c.payload_bytes = (uint8_t)g_pb;
  c.seed = 0xC0FFEE11u; c.input_delay = 2; c.host_slots = HOST_SLOTS; c.guest_slots = GUEST_SLOTS;
  c.match_blob = g_blob; c.match_blob_len = sizeof g_blob;
  c.cb.user = p; c.cb.remote_input = peer_remote_input; c.cb.event = peer_event; c.cb.desync = peer_desync;
  c.now_ms = peer_now;
  return c;
}
static void init_blob(void) { int i; for (i = 0; i < (int)sizeof g_blob; ++i) g_blob[i] = (uint8_t)(i * 7 + 3); }

/* lobby: 1 = both sides have lobby callbacks and hold_start */
static void make_pair_x(simnet *s, peer *host, peer *guest, uint32_t guest_offset, int pull, int lobby) {
  gw_net_config hc, gc;
  gw_net_transport ht = sim_transport(s, 0), gt = sim_transport(s, 1);
  gw_net_addr haddr = sim_addr(0);
  init_blob();
  peer_init(host, s, 0, 0); peer_init(guest, s, 1, guest_offset);
  host->pull = guest->pull = pull;
  hc = peer_cfg(host, 0x1111, 0x2222, 0x3333);
  gc = peer_cfg(guest, 0x1111, 0x2222, 0x3333);
  if (pull) { hc.cb.local_input = gc.cb.local_input = peer_local_input; hc.cb.checksum = gc.cb.checksum = peer_checksum; }
  if (lobby) { hc.cb.lobby_msg = gc.cb.lobby_msg = peer_lobby; hc.hold_start = gc.hold_start = 1; }
  host->net = gw_net_host(&hc, &ht);
  guest->net = gw_net_join(&gc, &gt, &haddr);
}
static void make_pair(simnet *s, peer *host, peer *guest, uint32_t guest_offset, int pull) { make_pair_x(s, host, guest, guest_offset, pull, 0); }

static void session_tick(peer *p) {
  gw_net_poll(p->net, p->frame);
  if (!gw_net_started(p->net)) return;
  if (p->use_sync) {
    int w;
    if (p->stall > 0) { p->stall--; return; }
    w = gw_net_recommend_wait(p->net);
    if (w > 0) { p->stall = w - 1; p->stalls_taken += w; return; }
  }
  p->acc += p->speed;
  while (p->acc >= 1.0f) {
    p->acc -= 1.0f;
    if (p->frame >= p->limit || gw_net_unacked(p->net) >= 60) break;
    if (!p->pull) {
      gw_net_input in[GW_NET_MAX_SLOTS]; int slot;
      memset(in, 0, sizeof in);
      for (slot = 0; slot < GW_NET_MAX_SLOTS; ++slot) if (gw_net_local_slots(p->net) & (1u << slot)) mk_payload(p->frame, slot, p->id, in[slot].b);
      if (gw_net_submit_local(p->net, p->frame, in) != 0) break;
      if (p->report_checksums) gw_net_report_checksum(p->net, p->frame, chk_of(p->frame) ^ (p->frame >= p->diverge_at ? 0xDEADBEEFu : 0));
    }
    p->frame++;
  }
}
static void pair_tick(simnet *s, peer *a, peer *b) { sim_step(s); session_tick(a); session_tick(b); }

static void summary(const char *name, peer *h, peer *g) {
  gw_net_stats a, b; float fa, fb; uint32_t ua, ub;
  gw_net_get_stats(h->net, &a); gw_net_get_stats(g->net, &b);
  fa = gw_net_frame_advantage(h->net); fb = gw_net_frame_advantage(g->net);
  memcpy(&ua, &fa, 4); memcpy(&ub, &fb, 4);
  printf("R %s del=%u/%u st=%d/%d bad=%d/%d dsf=%d/%d rtt=%u/%u adv=%08x/%08x fr=%d/%d stl=%d/%d conf=%d/%d unack=%u/%u "
         "h=%u,%u,%u,%u,%u,%u,%u g=%u,%u,%u,%u,%u,%u,%u why=%s|%s\n",
         name, h->delivered, g->delivered, gw_net_state(h->net), gw_net_state(g->net), h->bad, g->bad,
         (int)gw_net_desync_frame(h->net), (int)gw_net_desync_frame(g->net), gw_net_rtt_ms(h->net), gw_net_rtt_ms(g->net),
         ua, ub, (int)h->frame, (int)g->frame, h->stalls_taken, g->stalls_taken,
         (int)gw_net_remote_confirmed_frame(h->net), (int)gw_net_remote_confirmed_frame(g->net),
         gw_net_unacked(h->net), gw_net_unacked(g->net),
         a.packets_sent, a.packets_received, a.duplicates, a.out_of_order, a.bad_packets, a.inputs_delivered, a.resent_frames,
         b.packets_sent, b.packets_received, b.duplicates, b.out_of_order, b.bad_packets, b.inputs_delivered, b.resent_frames,
         gw_net_last_reason(h->net), gw_net_last_reason(g->net));
}

static simnet g_sim;

static void sc_handshake(void) {
  simnet *s = &g_sim; peer h, g; int i;
  printf("S handshake\n"); g_pb = 9; sim_init(s, 1);
  make_pair(s, &h, &g, 0xFFFFF000u, 0);
  for (i = 0; i < 600; ++i) pair_tick(s, &h, &g);
  printf("T %u %u\n", gw_net_start_time_ms(h.net), gw_net_start_time_ms(g.net));
  summary("handshake", &h, &g); gw_net_free(h.net); gw_net_free(g.net);
}
static void sc_inputs(const char *name, int loss, uint32_t jitter, int dup, int32_t frames, uint64_t seed, int pb, int pull) {
  simnet *s = &g_sim; peer h, g; uint32_t t;
  printf("S %s\n", name); g_pb = pb; sim_init(s, seed);
  make_pair(s, &h, &g, 0xFFFFF000u, pull);
  s->loss_pct = loss; s->jitter_ms = jitter; s->dup_pct = dup;
  h.limit = g.limit = FIRST + frames; h.report_checksums = g.report_checksums = 0;
  for (t = 0; t < (uint32_t)frames + 4000u; ++t) { pair_tick(s, &h, &g); if (h.delivered >= (uint32_t)frames && g.delivered >= (uint32_t)frames) break; }
  summary(name, &h, &g); gw_net_free(h.net); gw_net_free(g.net); g_pb = 9;
}
static void sc_sync(int use_sync) {
  simnet *s = &g_sim; peer h, g; uint32_t t;
  printf("S sync%d\n", use_sync); g_pb = 9; sim_init(s, 5);
  make_pair(s, &h, &g, 0x00001000u, 0);
  s->base_ms = 50; s->jitter_ms = 10; s->loss_pct = 5;
  g.speed = 0.85f; h.use_sync = g.use_sync = use_sync; h.report_checksums = g.report_checksums = 0;
  for (t = 0; t < 1500; ++t) pair_tick(s, &h, &g);
  summary(use_sync ? "sync1" : "sync0", &h, &g); gw_net_free(h.net); gw_net_free(g.net);
}
static void sc_desync(int pull) {
  simnet *s = &g_sim; peer h, g; uint32_t t;
  printf("S desync%d\n", pull); g_pb = 9; sim_init(s, 3);
  make_pair(s, &h, &g, 0, pull);
  s->loss_pct = 20; s->jitter_ms = 40; g.diverge_at = FIRST + 300;
  for (t = 0; t < 700; ++t) pair_tick(s, &h, &g);
  summary(pull ? "desync1" : "desync0", &h, &g); gw_net_free(h.net); gw_net_free(g.net);
}
static void sc_refuse(int which) {
  simnet *s = &g_sim; peer h, g; gw_net_config gc, hc; gw_net_transport ht, gt; gw_net_addr haddr = sim_addr(0); int i;
  char name[32];
  snprintf(name, sizeof name, "refuse%d", which);
  printf("S %s\n", name); g_pb = 9; sim_init(s, 7); init_blob();
  peer_init(&h, s, 0, 0); peer_init(&g, s, 1, 0);
  hc = peer_cfg(&h, 0x1111, 0x2222, 0x3333);
  hc.mods_desc = which == 9 ? "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa,bbbb" : "plco#12ab,itco#34cd";
  gc = peer_cfg(&g, which == 0 ? 0x9999 : 0x1111, which == 1 ? 0x9999 : 0x2222, (which == 2 || which == 9) ? 0x9999 : 0x3333);
  if (which == 3) gc.payload_bytes = 12;
  if (which == 4 || which == 5) { hc.rules = 0x5FBu; gc.rules = which == 4 ? 0u : 0x5FBu ^ 1u; gc.rules_expect = 1; }
  if (which == 6) { hc.envoy = GW_ENVOY_MODE_V1; gc.envoy = 0; gc.envoy_expect = 1; }
  if (which == 7) { hc.envoy = 0; gc.envoy = GW_ENVOY_MODE_V1; gc.envoy_expect = 1; }
  if (which == 8) { hc.envoy = GW_ENVOY_MODE_V1; gc.envoy = 0x1234u; }
  if (which == 10) { hc.rules = 0x77u; gc.rules = 0x78u; }                  /* not insisting: takes the host's */
  ht = sim_transport(s, 0); gt = sim_transport(s, 1);
  h.net = gw_net_host(&hc, &ht); g.net = gw_net_join(&gc, &gt, &haddr);
  for (i = 0; i < 100 && gw_net_state(g.net) == GW_NET_CONNECTING; ++i) pair_tick(s, &h, &g);
  for (i = 0; i < 30; ++i) pair_tick(s, &h, &g);
  {
    gw_net_config rc; uint8_t blob[GW_NET_MAX_BLOB];
    int ok = gw_net_remote_config(g.net, &rc, blob, sizeof blob);
    printf("C %d %08x %u %02x %02x %d %u %08x %08x %u\n", ok, ok ? rc.seed : 0, ok ? rc.input_delay : 0, ok ? rc.host_slots : 0, ok ? rc.guest_slots : 0,
           ok ? (int)rc.first_frame : 0, ok ? rc.payload_bytes : 0, ok ? rc.rules : 0, ok ? rc.envoy : 0, ok ? rc.match_blob_len : 0);
  }
  summary(name, &h, &g); gw_net_free(h.net); gw_net_free(g.net);
}
static void sc_old_heap(int pass) {
  simnet *s = &g_sim; peer h, g; int i;
  printf("S oldheap%d\n", pass); g_pb = 9; sim_init(s, 37 + pass); s->old_heap_peer = pass;
  make_pair(s, &h, &g, 0, 0);
  for (i = 0; i < 400 && gw_net_state(g.net) != GW_NET_REFUSED; ++i) pair_tick(s, &h, &g);
  summary(pass == 1 ? "oldheap1" : "oldheap2", &h, &g); gw_net_free(h.net); gw_net_free(g.net);
}
static void sc_interrupt(void) {
  simnet *s = &g_sim; peer h, g; uint32_t t;
  printf("S interrupt\n"); g_pb = 9; sim_init(s, 9);
  make_pair(s, &h, &g, 0, 0);
  for (t = 0; t < 800; ++t) { if (t == 300) s->blackhole = 1; if (t == 390) s->blackhole = 0; pair_tick(s, &h, &g); }
  summary("interrupt", &h, &g); gw_net_free(h.net); gw_net_free(g.net);
}
static void sc_timeout(void) {
  simnet *s = &g_sim; peer h, g; uint32_t t;
  printf("S timeout\n"); g_pb = 9; sim_init(s, 4);
  make_pair(s, &h, &g, 0, 0);
  for (t = 0; t < 600; ++t) { if (t == 200) s->blackhole = 1; pair_tick(s, &h, &g); }
  summary("timeout", &h, &g); gw_net_free(h.net); gw_net_free(g.net);
}
static void sc_quit(void) {
  simnet *s = &g_sim; peer h, g; uint32_t t;
  printf("S quit\n"); g_pb = 9; sim_init(s, 6);
  make_pair(s, &h, &g, 0, 0);
  for (t = 0; t < 200; ++t) pair_tick(s, &h, &g);
  gw_net_free(g.net); g.net = NULL;
  for (t = 0; t < 40; ++t) { sim_step(s); session_tick(&h); }
  printf("Q %d %s\n", gw_net_state(h.net), gw_net_last_reason(h.net));
  gw_net_free(h.net);
}
static void sc_lobby(void) {
  simnet *s = &g_sim; peer h, g; uint32_t t; int k;
  printf("S lobby\n"); g_pb = 9; sim_init(s, 12);
  make_pair_x(s, &h, &g, 0x12345u, 0, 1);
  s->loss_pct = 25; s->jitter_ms = 30;
  for (t = 0; t < 700; ++t) {
    if (t == 20) for (k = 0; k < 5; ++k) { uint8_t m[40]; int j; for (j = 0; j < 10 + k * 7; ++j) m[j] = (uint8_t)(k * 31 + j); gw_net_lobby_send(h.net, m, 10 + k * 7); gw_net_lobby_send(g.net, m, 3 + k); }
    if (t == 150) gw_net_release(g.net);
    if (t == 260) gw_net_release(h.net);
    if (t == 400) { uint8_t m[3] = {9, 8, 7}; gw_net_lobby_send(h.net, m, 3); }   /* after the start: resent while RUNNING */
    pair_tick(s, &h, &g);
  }
  printf("P %llx\n", (unsigned long long)gw_net_peer_mods_hash(h.net));
  summary("lobby", &h, &g); gw_net_free(h.net); gw_net_free(g.net);
}

int main(void) {
  sc_handshake();
  sc_inputs("clean", 0, 0, 0, 1500, 11, 9, 0);
  sc_inputs("lossy", 20, 50, 10, 1500, 22, 28, 0);
  sc_inputs("hostile", 50, 150, 10, 1500, 33, 9, 0);
  sc_inputs("pull", 10, 30, 5, 1000, 44, 28, 1);
  sc_sync(1); sc_sync(0);
  sc_desync(0); sc_desync(1);
  { int w; for (w = 0; w <= 10; ++w) sc_refuse(w); }
  sc_old_heap(1); sc_old_heap(2);
  sc_interrupt(); sc_timeout(); sc_quit(); sc_lobby();
  printf("END\n");
  return 0;
}
