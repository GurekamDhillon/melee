/* gw_net_tests.c - headless tests for gw_net.c (registered from gw_tests_core.c).
 *
 * Two REAL gw_net instances talk over a simulated network in deterministic virtual time: loss,
 * duplication, jitter (hence reordering) and per-peer clock offsets - one of which wraps the
 * uint32 millisecond clock mid-run. One test also runs over real UDP on 127.0.0.1. Nothing here
 * touches game memory. Frames are the session's signed numbers, starting at FIRST (-123).
 */
#include "gw_net.h"
#include "gw_test.h"

#include <stdio.h>
#include <string.h>
#include <windows.h>

#define FIRST (-123)

/* ---- simulated network ---------------------------------------------------------------------- */

#define SIM_ENDPOINTS 3
#define SIM_QUEUE 4096

typedef struct simpkt {
  uint32_t at;
  int to;
  gw_net_addr from;
  int len;
  uint8_t data[1300];
} simpkt;

typedef struct simnet {
  uint32_t now;                    /* virtual ms */
  uint64_t time_us;
  uint64_t rng;
  int loss_pct, dup_pct;
  uint32_t base_ms, jitter_ms;
  int blackhole;                   /* drop everything while set */
  simpkt q[SIM_QUEUE];
  int nq;
  uint32_t sent, dropped;
} simnet;

typedef struct simep { simnet *sim; int id; } simep;

static uint32_t rnd(simnet *s) {   /* xorshift64* */
  s->rng ^= s->rng >> 12; s->rng ^= s->rng << 25; s->rng ^= s->rng >> 27;
  return (uint32_t)((s->rng * 2685821657736338717ull) >> 32);
}

static void sim_init(simnet *s, uint64_t seed) {
  memset(s, 0, sizeof *s);
  s->rng = seed ? seed : 88172645463325252ull;
  s->base_ms = 20;
}
static gw_net_addr sim_addr(int id) { gw_net_addr a; a.ip = (10u << 24) | (uint32_t)(id + 1); a.port = 7000; return a; }
static void sim_step(simnet *s) { s->time_us += 16667; s->now = (uint32_t)(s->time_us / 1000); }

static void sim_enqueue(simnet *s, int from_id, int to, const void *data, int len) {
  simpkt *p;
  if (s->nq >= SIM_QUEUE || len > (int)sizeof p->data) return;
  p = &s->q[s->nq++];
  p->at = s->now + s->base_ms + (s->jitter_ms ? rnd(s) % (s->jitter_ms + 1) : 0);
  p->to = to;
  p->from = sim_addr(from_id);
  p->len = len;
  memcpy(p->data, data, (size_t)len);
}

static int sim_send(void *ctx, const gw_net_addr *to, const void *data, int len) {
  simep *e = (simep *)ctx;
  simnet *s = e->sim;
  int dest = (int)(to->ip & 0xFF) - 1;
  s->sent++;
  if (dest < 0 || dest >= SIM_ENDPOINTS) return len;
  if (s->blackhole || (int)(rnd(s) % 100) < s->loss_pct) { s->dropped++; return len; }
  sim_enqueue(s, e->id, dest, data, len);
  if ((int)(rnd(s) % 100) < s->dup_pct) sim_enqueue(s, e->id, dest, data, len);
  return len;
}

static int sim_recv(void *ctx, gw_net_addr *from, void *buf, int cap) {
  simep *e = (simep *)ctx;
  simnet *s = e->sim;
  int i, best = -1;
  for (i = 0; i < s->nq; ++i)
    if (s->q[i].to == e->id && (int32_t)(s->now - s->q[i].at) >= 0 &&
        (best < 0 || (int32_t)(s->q[i].at - s->q[best].at) < 0))
      best = i;
  if (best < 0) return 0;
  {
    simpkt *p = &s->q[best];
    int n = p->len < cap ? p->len : cap;
    *from = p->from;
    memcpy(buf, p->data, (size_t)n);
    s->q[best] = s->q[--s->nq];
    return n;
  }
}

static void sim_close(void *ctx) { free(ctx); }

static gw_net_transport sim_transport(simnet *s, int id) {
  gw_net_transport t;
  simep *e = (simep *)malloc(sizeof *e);
  e->sim = s;
  e->id = id;
  t.ctx = e;
  t.send = sim_send;
  t.recv = sim_recv;
  t.close = sim_close;
  return t;
}

/* ---- a test peer: a stand-in for the rollback session --------------------------------------- */

#define HOST_SLOTS 0x13                  /* slots 0, 1 (port 0 and its follower) and 4 (port 2) */
#define GUEST_SLOTS 0x44                 /* slots 2 and 6 (ports 1 and 3) */

static int g_pb = GW_NET_DEFAULT_PAYLOAD;   /* payload bytes for the test being run */

typedef struct peer {
  simnet *sim;
  int id, sender_id;                     /* sender_id: whose payloads we expect to receive */
  uint32_t offset;                       /* this peer's clock = sim.now + offset */
  gw_net *net;
  int events[32], nev;
  char reasons[32][100];
  /* the "session" */
  int32_t frame, limit;                  /* next frame to produce; stop before `limit` */
  float speed, acc;
  int stall, use_sync, stalls_taken;
  int32_t diverge_at;                    /* report a wrong checksum from this frame on */
  int report_checksums, pull;            /* pull: the callbacks supply inputs/checksums */
  /* delivery check */
  int rslots[GW_NET_MAX_SLOTS], nrslots;
  int32_t exp_frame;
  uint32_t delivered;
  int exp_idx, bad;
  char bad_msg[100];
  int desync_calls;
  int32_t desync_frame;
} peer;

static void mk_payload(int32_t frame, int slot, int who, uint8_t *out) {
  uint32_t h = (uint32_t)frame * 2654435761u ^ ((uint32_t)slot * 40503u) ^ ((uint32_t)(who + 1) * 0x9E3779B1u);
  int i;
  for (i = 0; i < g_pb; ++i) { h ^= h >> 15; h *= 2246822519u; h ^= h >> 13; out[i] = (uint8_t)(h >> (i & 7)); }
}
static uint32_t chk_of(int32_t f) { return (uint32_t)f * 2654435761u ^ 0x5BD1E995u; }

static uint32_t peer_now(void *user) { peer *p = (peer *)user; return p->sim->now + p->offset; }

static void peer_remote_input(void *user, int32_t frame, int slot, const uint8_t *payload) {
  peer *p = (peer *)user;
  uint8_t want[GW_NET_MAX_PAYLOAD];
  if (p->bad) return;
  if (frame != p->exp_frame || slot != p->rslots[p->exp_idx]) {
    snprintf(p->bad_msg, sizeof p->bad_msg, "delivered frame %d slot %d, expected frame %d slot %d",
             (int)frame, slot, (int)p->exp_frame, p->rslots[p->exp_idx]);
    p->bad = 1;
    return;
  }
  mk_payload(frame, slot, p->sender_id, want);
  if (memcmp(payload, want, (size_t)g_pb) != 0) {
    snprintf(p->bad_msg, sizeof p->bad_msg, "frame %d slot %d: wrong payload contents", (int)frame, slot);
    p->bad = 1;
    return;
  }
  if (++p->exp_idx == p->nrslots) { p->exp_idx = 0; p->exp_frame++; p->delivered++; }
}

static int peer_local_input(void *user, int32_t frame, int slot, uint8_t *out) {
  peer *p = (peer *)user;
  if (frame >= p->frame) return 0;       /* the session has not produced that frame yet */
  mk_payload(frame, slot, p->id, out);
  return 1;
}

static int peer_checksum(void *user, int32_t frame, uint32_t *out) {
  peer *p = (peer *)user;
  if (frame >= p->frame) return 0;
  *out = chk_of(frame) ^ (frame >= p->diverge_at ? 0xDEADBEEFu : 0);
  return 1;
}

static void peer_event(void *user, int ev, const char *msg) {
  peer *p = (peer *)user;
  if (p->nev < 32) {
    p->events[p->nev] = ev;
    snprintf(p->reasons[p->nev], sizeof p->reasons[0], "%s", msg != NULL ? msg : "");
    p->nev++;
  }
}
static int has_event(const peer *p, int ev) {
  int i;
  for (i = 0; i < p->nev; ++i) if (p->events[i] == ev) return 1;
  return 0;
}
static const char *event_msg(const peer *p, int ev) {
  int i;
  for (i = 0; i < p->nev; ++i) if (p->events[i] == ev) return p->reasons[i];
  return "";
}

static void peer_desync(void *user, int32_t frame, uint32_t local, uint32_t remote) {
  peer *p = (peer *)user;
  (void)local; (void)remote;
  p->desync_calls++;
  p->desync_frame = frame;
}

static void peer_init(peer *p, simnet *s, int id, uint32_t offset) {
  int i;
  memset(p, 0, sizeof *p);
  p->sim = s; p->id = id; p->sender_id = id == 0 ? 1 : 0; p->offset = offset;
  p->speed = 1.0f; p->frame = FIRST; p->exp_frame = FIRST;
  p->limit = 0x7FFFFFFF; p->diverge_at = 0x7FFFFFFF; p->report_checksums = 1;
  {
    uint8_t rmask = (id == 0) ? GUEST_SLOTS : HOST_SLOTS;
    for (i = 0; i < GW_NET_MAX_SLOTS; ++i) if (rmask & (1u << i)) p->rslots[p->nrslots++] = i;
  }
}

static uint8_t g_blob[300];

static gw_net_config peer_cfg(peer *p, uint64_t exe, uint64_t iso, uint64_t mods) {
  gw_net_config c;
  memset(&c, 0, sizeof c);
  c.exe_hash = exe; c.iso_hash = iso; c.mods_hash = mods;
  c.first_frame = FIRST; c.payload_bytes = (uint8_t)g_pb;
  c.seed = 0xC0FFEE11u; c.input_delay = 2; c.host_slots = HOST_SLOTS; c.guest_slots = GUEST_SLOTS;
  c.match_blob = g_blob; c.match_blob_len = sizeof g_blob;
  c.cb.user = p;
  c.cb.remote_input = peer_remote_input;
  c.cb.event = peer_event;
  c.cb.desync = peer_desync;
  c.now_ms = peer_now;
  return c;
}

static void init_blob(void) {
  int i;
  for (i = 0; i < (int)sizeof g_blob; ++i) g_blob[i] = (uint8_t)(i * 7 + 3);
}

static void make_pair(simnet *s, peer *host, peer *guest, uint32_t guest_offset, int pull) {
  gw_net_config hc, gc;
  gw_net_transport ht = sim_transport(s, 0), gt = sim_transport(s, 1);
  gw_net_addr haddr = sim_addr(0);
  init_blob();
  peer_init(host, s, 0, 0);
  peer_init(guest, s, 1, guest_offset);
  host->pull = guest->pull = pull;
  hc = peer_cfg(host, 0x1111, 0x2222, 0x3333);
  gc = peer_cfg(guest, 0x1111, 0x2222, 0x3333);
  if (pull) {
    hc.cb.local_input = gc.cb.local_input = peer_local_input;
    hc.cb.checksum = gc.cb.checksum = peer_checksum;
  }
  host->net = gw_net_host(&hc, &ht);
  guest->net = gw_net_join(&gc, &gt, &haddr);
}

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
      gw_net_input in[GW_NET_MAX_SLOTS];
      int slot;
      memset(in, 0, sizeof in);
      for (slot = 0; slot < GW_NET_MAX_SLOTS; ++slot)
        if (gw_net_local_slots(p->net) & (1u << slot)) mk_payload(p->frame, slot, p->id, in[slot].b);
      if (gw_net_submit_local(p->net, p->frame, in) != 0) break;
      if (p->report_checksums)
        gw_net_report_checksum(p->net, p->frame, chk_of(p->frame) ^ (p->frame >= p->diverge_at ? 0xDEADBEEFu : 0));
    }
    p->frame++;                          /* in pull mode this is what makes the frame available */
  }
}

static void pair_tick(simnet *s, peer *a, peer *b) {
  sim_step(s);
  session_tick(a);
  session_tick(b);
}

static void free_pair(peer *a, peer *b) { gw_net_free(a->net); gw_net_free(b->net); }

/* ---- tests ---------------------------------------------------------------------------------- */

static int test_hash_addr(void) {
  gw_net_addr a;
  if (gw_net_hash64("", 0, 0) != 0xcbf29ce484222325ull) { gw_test_fail("fnv1a of empty"); return 1; }
  if (gw_net_hash64("a", 1, 0) != 0xaf63dc4c8601ec8cull) { gw_test_fail("fnv1a of 'a'"); return 1; }
  if (gw_net_addr_parse("192.168.1.20:5555", 1, &a) != 0 || a.ip != 0xC0A80114u || a.port != 5555) {
    gw_test_fail("addr parse with port"); return 1;
  }
  if (gw_net_addr_parse("10.0.0.1", 7777, &a) != 0 || a.port != 7777) { gw_test_fail("addr default port"); return 1; }
  if (gw_net_addr_parse("10.0.0", 7777, &a) == 0 || gw_net_addr_parse("300.1.1.1", 7777, &a) == 0) {
    gw_test_fail("addr parse accepted garbage"); return 1;
  }
  return 0;
}

static int test_handshake(void) {
  simnet *s = (simnet *)malloc(sizeof *s);
  peer h, g;
  gw_net_config rc;
  uint8_t blob[GW_NET_MAX_BLOB];
  int i, rv = 0;
  g_pb = GW_NET_DEFAULT_PAYLOAD;
  sim_init(s, 1);
  make_pair(s, &h, &g, 0xFFFFF000u, 0);   /* the guest's clock wraps 4 s in */
  for (i = 0; i < 400 && !(gw_net_started(h.net) && gw_net_started(g.net)); ++i) pair_tick(s, &h, &g);
  if (!gw_net_started(h.net) || !gw_net_started(g.net)) {
    gw_test_fail("handshake did not complete (host state %d, guest state %d)", gw_net_state(h.net), gw_net_state(g.net));
    rv = 1;
  } else if (!has_event(&g, GW_NET_EV_ACCEPTED) || !has_event(&h, GW_NET_EV_STARTING) ||
             !has_event(&g, GW_NET_EV_STARTING) || !has_event(&h, GW_NET_EV_STARTED) ||
             !has_event(&g, GW_NET_EV_STARTED)) {
    gw_test_fail("missing lifecycle event"); rv = 1;
  } else if (!gw_net_remote_config(g.net, &rc, blob, sizeof blob) || rc.seed != 0xC0FFEE11u ||
             rc.input_delay != 2 || rc.host_slots != HOST_SLOTS || rc.guest_slots != GUEST_SLOTS ||
             rc.first_frame != FIRST || rc.payload_bytes != g_pb ||
             rc.match_blob_len != sizeof g_blob || memcmp(blob, g_blob, sizeof g_blob) != 0) {
    gw_test_fail("guest did not receive the host's match config intact"); rv = 1;
  } else if (gw_net_local_slots(g.net) != GUEST_SLOTS || gw_net_remote_slots(g.net) != HOST_SLOTS) {
    gw_test_fail("slot masks"); rv = 1;
  } else {
    /* both peers agree on when frame FIRST began, in real time (the guest's clock is offset) */
    int32_t hs = (int32_t)gw_net_start_time_ms(h.net);
    int32_t gs = (int32_t)(gw_net_start_time_ms(g.net) - g.offset);
    int32_t d = hs > gs ? hs - gs : gs - hs;
    if (d > 6) { gw_test_fail("start times differ by %d ms", (int)d); rv = 1; }
    /* 20 ms each way, but packets are only picked up on 16.7 ms ticks: two ticks per direction */
    if (gw_net_rtt_ms(g.net) < 40 || gw_net_rtt_ms(g.net) > 100) { gw_test_fail("guest rtt %u ms, expected ~67", gw_net_rtt_ms(g.net)); rv = 1; }
  }
  free_pair(&h, &g);
  free(s);
  return rv;
}

/* which: 0 exe, 1 iso, 2 mods, 3 input size */
static int refuse_case(int which, const char *needle) {
  simnet *s = (simnet *)malloc(sizeof *s);
  peer h, g, g2;
  gw_net_config gc, hc;
  gw_net_transport ht, gt, g2t;
  gw_net_addr haddr = sim_addr(0);
  int i, rv = 0;
  g_pb = GW_NET_DEFAULT_PAYLOAD;
  sim_init(s, 7);
  init_blob();
  peer_init(&h, s, 0, 0); peer_init(&g, s, 1, 0); peer_init(&g2, s, 1, 0);
  hc = peer_cfg(&h, 0x1111, 0x2222, 0x3333);
  hc.mods_desc = "plco#12ab,itco#34cd";
  gc = peer_cfg(&g, which == 0 ? 0x9999 : 0x1111, which == 1 ? 0x9999 : 0x2222, which == 2 ? 0x9999 : 0x3333);
  if (which == 3) gc.payload_bytes = 12;
  ht = sim_transport(s, 0); gt = sim_transport(s, 1); g2t = sim_transport(s, 2);
  h.net = gw_net_host(&hc, &ht);
  g.net = gw_net_join(&gc, &gt, &haddr);
  for (i = 0; i < 100 && gw_net_state(g.net) == GW_NET_CONNECTING; ++i) pair_tick(s, &h, &g);
  if (gw_net_state(g.net) != GW_NET_REFUSED || !has_event(&g, GW_NET_EV_REFUSED)) {
    gw_test_fail("mismatch %d was not refused (guest state %d)", which, gw_net_state(g.net)); rv = 1;
  } else if (strstr(event_msg(&g, GW_NET_EV_REFUSED), needle) == NULL) {
    gw_test_fail("refusal reason \"%s\" does not mention \"%s\"", event_msg(&g, GW_NET_EV_REFUSED), needle); rv = 1;
  } else if (gw_net_state(h.net) != GW_NET_LISTENING) {
    gw_test_fail("host left LISTENING after refusing a guest (state %d)", gw_net_state(h.net)); rv = 1;
  } else {
    /* a refused guest does not use up the slot: a correct one can still join, and a third is told the session is full */
    gw_net_config g2c = peer_cfg(&g2, 0x1111, 0x2222, 0x3333);
    g2.net = gw_net_join(&g2c, &g2t, &haddr);
    g2.id = 2; g2.sender_id = 0;
    for (i = 0; i < 400 && !(gw_net_started(h.net) && gw_net_started(g2.net)); ++i) {
      sim_step(s);
      session_tick(&h);
      session_tick(&g2);
    }
    if (!gw_net_started(g2.net)) { gw_test_fail("a matching guest could not join after a refusal"); rv = 1; }
    g2t = sim_transport(s, 1);           /* endpoint 1 is free again: a third would-be guest */
    {
      peer g3;
      gw_net_config g3c;
      peer_init(&g3, s, 1, 0);
      g3c = peer_cfg(&g3, 0x1111, 0x2222, 0x3333);
      g3.net = gw_net_join(&g3c, &g2t, &haddr);
      for (i = 0; i < 100 && gw_net_state(g3.net) == GW_NET_CONNECTING; ++i) {
        sim_step(s);
        session_tick(&h);
        session_tick(&g2);
        session_tick(&g3);
      }
      if (gw_net_state(g3.net) != GW_NET_REFUSED || strstr(event_msg(&g3, GW_NET_EV_REFUSED), "full") == NULL) {
        gw_test_fail("a second guest was not told the session is full (state %d, \"%s\")",
                     gw_net_state(g3.net), event_msg(&g3, GW_NET_EV_REFUSED));
        rv = 1;
      }
      gw_net_free(g3.net);
    }
    gw_net_free(g2.net);
  }
  gw_net_free(h.net);
  gw_net_free(g.net);
  free(s);
  return rv;
}
static int test_refuse_exe(void)  { return refuse_case(0, "melee-pc.exe"); }
static int test_refuse_iso(void)  { return refuse_case(1, "disc"); }
/* the host's global-data components travel in the refusal so the guest can name the difference */
static int test_refuse_mods(void) { return refuse_case(2, "different global game data; host has: plco#12ab,itco#34cd"); }
static int test_refuse_settings(void) { return refuse_case(3, "netplay settings"); }

/* N frames each way, exactly once and in order, over a hostile network. */
static int inputs_case(int loss, uint32_t jitter, int dup, int32_t frames, uint64_t seed, int pb, int pull) {
  simnet *s = (simnet *)malloc(sizeof *s);
  peer h, g;
  uint32_t t, ticks = 0;
  int rv = 0;
  gw_net_stats hs;
  g_pb = pb;
  sim_init(s, seed);
  make_pair(s, &h, &g, 0xFFFFF000u, pull);
  s->loss_pct = loss; s->jitter_ms = jitter; s->dup_pct = dup;
  h.limit = g.limit = FIRST + frames;
  h.report_checksums = g.report_checksums = 0;
  for (t = 0; t < (uint32_t)frames + 4000u; ++t) {
    pair_tick(s, &h, &g);
    ticks++;
    if (h.delivered >= (uint32_t)frames && g.delivered >= (uint32_t)frames) break;
  }
  if (h.bad || g.bad) {
    gw_test_fail("%s", h.bad ? h.bad_msg : g.bad_msg); rv = 1;
  } else if (h.delivered != (uint32_t)frames || g.delivered != (uint32_t)frames) {
    gw_test_fail("loss %d%% jitter %ums payload %d: delivered host %u / guest %u of %d frames after %u ticks",
                 loss, jitter, pb, h.delivered, g.delivered, (int)frames, ticks);
    rv = 1;
  } else if (gw_net_remote_confirmed_frame(h.net) != FIRST + frames - 1) {
    gw_test_fail("remote_confirmed_frame %d, expected %d", (int)gw_net_remote_confirmed_frame(h.net), (int)(FIRST + frames - 1));
    rv = 1;
  }
  gw_net_get_stats(h.net, &hs);
  if (rv == 0 && loss >= 20 && hs.resent_frames == 0) { gw_test_fail("no redundancy was needed at %d%% loss?", loss); rv = 1; }
  if (rv == 0 && dup > 0 && hs.duplicates == 0) { gw_test_fail("duplicates were not detected"); rv = 1; }
  free_pair(&h, &g);
  free(s);
  g_pb = GW_NET_DEFAULT_PAYLOAD;
  return rv;
}
static int test_inputs_clean(void)   { return inputs_case(0, 0, 0, 10000, 11, 9, 0); }
static int test_inputs_lossy(void)   { return inputs_case(20, 50, 10, 10000, 22, 28, 0); }   /* GwRbInput-sized payloads */
static int test_inputs_hostile(void) { return inputs_case(50, 150, 10, 10000, 33, 9, 0); }
static int test_inputs_pull(void)    { return inputs_case(10, 30, 5, 3000, 44, 28, 1); }    /* callbacks supply inputs */

/* One peer runs slow. With time sync the fast one waits and the frame gap stays small; without it
 * the gap grows without bound (the control run proves the test measures something). */
static int sync_case(int use_sync, int32_t *max_gap_tail, int32_t *final_gap, int *stalls) {
  simnet *s = (simnet *)malloc(sizeof *s);
  peer h, g;
  uint32_t t;
  int32_t worst = 0;
  g_pb = GW_NET_DEFAULT_PAYLOAD;
  sim_init(s, 5);
  make_pair(s, &h, &g, 0x00001000u, 0);
  s->base_ms = 50; s->jitter_ms = 10; s->loss_pct = 5;
  g.speed = 0.85f;
  h.use_sync = g.use_sync = use_sync;
  h.report_checksums = g.report_checksums = 0;
  for (t = 0; t < 3600; ++t) {
    pair_tick(s, &h, &g);
    if (t >= 2400) {
      int32_t d = h.frame - g.frame;
      if (d < 0) d = -d;
      if (d > worst) worst = d;
    }
  }
  *max_gap_tail = worst;
  *final_gap = h.frame - g.frame;
  *stalls = h.stalls_taken + g.stalls_taken;
  free_pair(&h, &g);
  free(s);
  return 0;
}
static int test_frame_sync(void) {
  int32_t wg, fg, wg0, fg0;
  int st, st0;
  sync_case(1, &wg, &fg, &st);
  sync_case(0, &wg0, &fg0, &st0);
  if (wg0 < 200) { gw_test_fail("control run gap only %d frames: the test is not measuring drift", (int)wg0); return 1; }
  if (st == 0) { gw_test_fail("time sync never asked anyone to wait"); return 1; }
  if (wg > 6) { gw_test_fail("with sync the frame gap still reached %d (control %d)", (int)wg, (int)wg0); return 1; }
  return 0;
}

static int test_interrupt_resume(void) {
  simnet *s = (simnet *)malloc(sizeof *s);
  peer h, g;
  uint32_t t;
  int rv = 0;
  g_pb = GW_NET_DEFAULT_PAYLOAD;
  sim_init(s, 9);
  make_pair(s, &h, &g, 0, 0);
  for (t = 0; t < 1500; ++t) {
    if (t == 300) s->blackhole = 1;
    if (t == 300 + 90) s->blackhole = 0;   /* 1.5 s of silence: interrupted, not disconnected */
    pair_tick(s, &h, &g);
  }
  if (!has_event(&h, GW_NET_EV_INTERRUPTED) || !has_event(&g, GW_NET_EV_INTERRUPTED)) {
    gw_test_fail("no INTERRUPTED event after 1.5 s of silence"); rv = 1;
  } else if (!has_event(&h, GW_NET_EV_RESUMED) || !has_event(&g, GW_NET_EV_RESUMED)) {
    gw_test_fail("no RESUMED event when the network came back"); rv = 1;
  } else if (gw_net_state(h.net) != GW_NET_RUNNING || gw_net_state(g.net) != GW_NET_RUNNING) {
    gw_test_fail("a short outage killed the session"); rv = 1;
  } else if (h.delivered < 600 || g.delivered < 600) {
    gw_test_fail("input flow did not resume (%u / %u frames)", h.delivered, g.delivered); rv = 1;
  } else if (h.bad || g.bad) {
    gw_test_fail("%s", h.bad ? h.bad_msg : g.bad_msg); rv = 1;
  }
  free_pair(&h, &g);
  free(s);
  return rv;
}

static int test_peer_timeout(void) {
  simnet *s = (simnet *)malloc(sizeof *s);
  peer h, g;
  uint32_t t, dead_at = 0;
  int rv = 0;
  g_pb = GW_NET_DEFAULT_PAYLOAD;
  sim_init(s, 4);
  make_pair(s, &h, &g, 0, 0);
  for (t = 0; t < 900; ++t) {
    if (t == 200) s->blackhole = 1;
    pair_tick(s, &h, &g);
    if (dead_at == 0 && gw_net_state(h.net) == GW_NET_DEAD) dead_at = t;
  }
  if (gw_net_state(h.net) != GW_NET_DEAD || gw_net_state(g.net) != GW_NET_DEAD) {
    gw_test_fail("a silent peer was never timed out (host state %d)", gw_net_state(h.net)); rv = 1;
  } else if (dead_at < 200 + 280 || dead_at > 200 + 320) {   /* 5 s = 300 ticks after the cut */
    gw_test_fail("disconnect fired %u ticks after the cut, expected ~300", (unsigned)(dead_at - 200)); rv = 1;
  } else if (!has_event(&h, GW_NET_EV_DISCONNECTED) || strstr(event_msg(&h, GW_NET_EV_DISCONNECTED), "timed out") == NULL) {
    gw_test_fail("disconnect reason: \"%s\"", event_msg(&h, GW_NET_EV_DISCONNECTED)); rv = 1;
  }
  free_pair(&h, &g);
  free(s);
  return rv;
}

static int desync_case(int pull) {
  simnet *s = (simnet *)malloc(sizeof *s);
  peer h, g;
  uint32_t t;
  int rv = 0;
  g_pb = GW_NET_DEFAULT_PAYLOAD;
  sim_init(s, 3);
  make_pair(s, &h, &g, 0, pull);
  s->loss_pct = 20; s->jitter_ms = 40;
  g.diverge_at = FIRST + 700;             /* the guest's simulation goes wrong at this frame */
  for (t = 0; t < 1400; ++t) pair_tick(s, &h, &g);
  if (h.desync_calls != 1 || g.desync_calls != 1) {
    gw_test_fail("desync callback fired %d / %d times, expected once each", h.desync_calls, g.desync_calls); rv = 1;
  } else if (h.desync_frame != FIRST + 700 || g.desync_frame != FIRST + 700 || gw_net_desync_frame(h.net) != FIRST + 700) {
    gw_test_fail("desync reported at frame %d / %d, expected %d", (int)h.desync_frame, (int)g.desync_frame, FIRST + 700); rv = 1;
  }
  free_pair(&h, &g);
  free(s);
  /* and no false alarm when both agree */
  s = (simnet *)malloc(sizeof *s);
  sim_init(s, 3);
  make_pair(s, &h, &g, 0, pull);
  s->loss_pct = 30; s->jitter_ms = 60;
  for (t = 0; t < 1500; ++t) pair_tick(s, &h, &g);
  if (h.desync_calls != 0 || g.desync_calls != 0 || gw_net_desync_frame(h.net) != GW_NET_NO_FRAME) {
    gw_test_fail("false desync alarm with identical checksums"); rv = 1;
  }
  free_pair(&h, &g);
  free(s);
  return rv;
}
static int test_desync(void)      { return desync_case(0); }
static int test_desync_pull(void) { return desync_case(1); }   /* checksums fetched through the callback */

static int test_quit(void) {
  simnet *s = (simnet *)malloc(sizeof *s);
  peer h, g;
  uint32_t t;
  int rv = 0;
  g_pb = GW_NET_DEFAULT_PAYLOAD;
  sim_init(s, 6);
  make_pair(s, &h, &g, 0, 0);
  for (t = 0; t < 200; ++t) pair_tick(s, &h, &g);
  gw_net_free(g.net);                     /* the guest leaves */
  g.net = NULL;
  for (t = 0; t < 40; ++t) { sim_step(s); session_tick(&h); }
  if (gw_net_state(h.net) != GW_NET_DEAD || strstr(event_msg(&h, GW_NET_EV_DISCONNECTED), "quit") == NULL) {
    gw_test_fail("host did not see the guest quit (state %d)", gw_net_state(h.net)); rv = 1;
  }
  gw_net_free(h.net);
  free(s);
  return rv;
}

static int test_api_misc(void) {
  simnet *s = (simnet *)malloc(sizeof *s);
  peer h, g;
  gw_net_input in[GW_NET_MAX_SLOTS];
  gw_net_config bad;
  gw_net_transport t = sim_transport(s, 0);
  int i, rv = 0;
  g_pb = GW_NET_DEFAULT_PAYLOAD;
  memset(in, 0, sizeof in);
  sim_init(s, 2);
  make_pair(s, &h, &g, 0, 0);
  for (i = 0; i < 400 && !gw_net_started(h.net); ++i) pair_tick(s, &h, &g);
  h.limit = h.frame;                      /* the test session stops submitting; the API is driven by hand */
  if (gw_net_submit_local(h.net, h.frame + 5, in) >= 0) { gw_test_fail("out-of-sequence submit accepted"); rv = 1; }
  if (gw_net_submit_local(h.net, h.frame - 1, in) >= 0) { gw_test_fail("already-submitted frame accepted"); rv = 1; }
  if (gw_net_submit_local(h.net, h.frame, in) != 0) { gw_test_fail("in-sequence submit rejected"); rv = 1; }
  h.frame++;
  /* fill the send window with the peer silent: submit must eventually refuse */
  s->blackhole = 1;
  for (i = 0; i < 400; ++i) if (gw_net_submit_local(h.net, h.frame++, in) < 0) break;
  if (i >= 400) { gw_test_fail("send window never filled"); rv = 1; }
  /* invalid configs are refused up front */
  memset(&bad, 0, sizeof bad);
  bad.host_slots = 0x03; bad.guest_slots = 0x02;              /* overlap */
  if (gw_net_host(&bad, &t) != NULL) { gw_test_fail("overlapping slot masks accepted"); rv = 1; }
  bad.host_slots = 0x01; bad.guest_slots = 0x00;              /* the guest controls nothing */
  if (gw_net_host(&bad, &t) != NULL) { gw_test_fail("empty guest slots accepted"); rv = 1; }
  bad.host_slots = 0x01; bad.guest_slots = 0x02; bad.payload_bytes = GW_NET_MAX_PAYLOAD + 1;
  if (gw_net_host(&bad, &t) != NULL) { gw_test_fail("oversized payload accepted"); rv = 1; }
  free_pair(&h, &g);
  free(s);
  return rv;
}

/* The real Winsock path: two sockets on 127.0.0.1, real clock, real time. */
static int test_udp_loopback(void) {
  gw_net_transport ht, gt;
  gw_net_config hc, gc;
  peer h, g;
  gw_net_addr haddr;
  simnet *dummy = (simnet *)calloc(1, sizeof *dummy);   /* peers want a simnet pointer; ~5 MB, so not on the stack */
  uint32_t deadline;
  int32_t frame_h = FIRST, frame_g = FIRST;
  int rv = 0;
  g_pb = GW_NET_DEFAULT_PAYLOAD;
  if (gw_net_udp_open(0x7F000001u, 0, &ht) != 0 || gw_net_udp_open(0x7F000001u, 0, &gt) != 0) {
    gw_test_fail("could not open UDP sockets on 127.0.0.1"); free(dummy); return 1;
  }
  haddr.ip = 0x7F000001u; haddr.port = gw_net_udp_local_port(&ht);
  init_blob();
  peer_init(&h, dummy, 0, 0); peer_init(&g, dummy, 1, 0);
  hc = peer_cfg(&h, 0x1111, 0x2222, 0x3333);
  gc = peer_cfg(&g, 0x1111, 0x2222, 0x3333);
  hc.now_ms = NULL; gc.now_ms = NULL;                          /* the real clock */
  h.net = gw_net_host(&hc, &ht);
  g.net = gw_net_join(&gc, &gt, &haddr);
  deadline = GetTickCount() + 8000;
  while ((int32_t)(deadline - GetTickCount()) > 0) {
    gw_net_input in[GW_NET_MAX_SLOTS];
    int slot;
    gw_net_poll(h.net, frame_h);
    gw_net_poll(g.net, frame_g);
    if (gw_net_started(h.net) && gw_net_unacked(h.net) < 30 && frame_h < FIRST + 200) {
      memset(in, 0, sizeof in);
      for (slot = 0; slot < GW_NET_MAX_SLOTS; ++slot) if (HOST_SLOTS & (1u << slot)) mk_payload(frame_h, slot, 0, in[slot].b);
      gw_net_submit_local(h.net, frame_h++, in);
    }
    if (gw_net_started(g.net) && gw_net_unacked(g.net) < 30 && frame_g < FIRST + 200) {
      memset(in, 0, sizeof in);
      for (slot = 0; slot < GW_NET_MAX_SLOTS; ++slot) if (GUEST_SLOTS & (1u << slot)) mk_payload(frame_g, slot, 1, in[slot].b);
      gw_net_submit_local(g.net, frame_g++, in);
    }
    if (h.delivered >= 200 && g.delivered >= 200) break;
    Sleep(1);
  }
  if (h.bad || g.bad) { gw_test_fail("%s", h.bad ? h.bad_msg : g.bad_msg); rv = 1; }
  else if (h.delivered < 200 || g.delivered < 200) {
    gw_test_fail("real UDP: delivered host %u / guest %u of 200 (states %d/%d)", h.delivered, g.delivered,
                 gw_net_state(h.net), gw_net_state(g.net));
    rv = 1;
  }
  gw_net_free(h.net);
  gw_net_free(g.net);
  free(dummy);
  return rv;
}

void gw_net_tests_register(void) {
  gw_test_register("net_hash_addr", test_hash_addr);
  gw_test_register("net_handshake", test_handshake);
  gw_test_register("net_refuse_exe", test_refuse_exe);
  gw_test_register("net_refuse_iso", test_refuse_iso);
  gw_test_register("net_refuse_mods", test_refuse_mods);
  gw_test_register("net_refuse_settings", test_refuse_settings);
  gw_test_register("net_inputs_clean", test_inputs_clean);
  gw_test_register("net_inputs_lossy", test_inputs_lossy);
  gw_test_register("net_inputs_hostile", test_inputs_hostile);
  gw_test_register("net_inputs_pull", test_inputs_pull);
  gw_test_register("net_frame_sync", test_frame_sync);
  gw_test_register("net_interrupt_resume", test_interrupt_resume);
  gw_test_register("net_peer_timeout", test_peer_timeout);
  gw_test_register("net_desync", test_desync);
  gw_test_register("net_desync_pull", test_desync_pull);
  gw_test_register("net_quit", test_quit);
  gw_test_register("net_api_misc", test_api_misc);
  gw_test_register("net_udp_loopback", test_udp_loopback);
}
