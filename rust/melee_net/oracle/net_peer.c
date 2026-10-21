/* A real C peer (the shipping gw_net.c over Winsock) for tests/interop.rs: talks to the Rust port
 * over UDP on 127.0.0.1 and exchanges FRAMES frames of test payloads both ways.
 *   net_peer host           binds an ephemeral port, prints "PORT <n>", waits for one guest
 *   net_peer guest <port>   joins 127.0.0.1:<port>
 * Exit 0 when every remote payload arrived in order and intact (and the agreed config matched),
 * printing "OK <delivered> <rtt>"; otherwise "FAIL <why>" and exit 1. */
#include "gw_net.c"

#define FIRST (-123)
#define FRAMES 300
#define HOST_SLOTS 0x13
#define GUEST_SLOTS 0x44
#define PB 11

static int g_host, g_bad;
static int32_t g_exp = FIRST;
static int g_idx;
static uint32_t g_delivered;
static int g_rslots[8], g_nr;

static void mk_payload(int32_t frame, int slot, int who, uint8_t *out) {
  uint32_t h = (uint32_t)frame * 2654435761u ^ ((uint32_t)slot * 40503u) ^ ((uint32_t)(who + 1) * 0x9E3779B1u);
  int i;
  for (i = 0; i < PB; ++i) { h ^= h >> 15; h *= 2246822519u; h ^= h >> 13; out[i] = (uint8_t)(h >> (i & 7)); }
}
static void on_remote(void *u, int32_t frame, int slot, const uint8_t *payload) {
  uint8_t want[32];
  (void)u;
  if (g_bad) return;
  if (frame != g_exp || slot != g_rslots[g_idx]) { g_bad = 1; return; }
  mk_payload(frame, slot, g_host ? 1 : 0, want);
  if (memcmp(payload, want, PB) != 0) { g_bad = 2; return; }
  if (++g_idx == g_nr) { g_idx = 0; g_exp++; g_delivered++; }
}
static uint32_t g_chk_bad;
static void on_desync(void *u, int32_t f, uint32_t a, uint32_t b) { (void)u; (void)f; (void)a; (void)b; g_chk_bad = 1; }

int main(int argc, char **argv) {
  gw_net_transport t;
  gw_net_config c;
  gw_net *n;
  uint8_t blob[64];
  int i, rv;
  int32_t frame = FIRST;
  DWORD deadline;
  setvbuf(stdout, NULL, _IONBF, 0);
  if (argc < 2) return 2;
  g_host = strcmp(argv[1], "host") == 0;
  for (i = 0; i < 8; ++i) if ((g_host ? GUEST_SLOTS : HOST_SLOTS) & (1u << i)) g_rslots[g_nr++] = i;
  for (i = 0; i < 64; ++i) blob[i] = (uint8_t)(i * 5 + 1);
  if (gw_net_udp_open(0x7F000001u, 0, &t) != 0) { printf("FAIL udp\n"); return 1; }
  memset(&c, 0, sizeof c);
  c.exe_hash = 0x1111; c.iso_hash = 0x2222; c.mods_hash = 0x3333;
  c.first_frame = FIRST; c.payload_bytes = PB;
  c.seed = 0xC0FFEE11u; c.input_delay = 2; c.host_slots = HOST_SLOTS; c.guest_slots = GUEST_SLOTS;
  c.rules = 0x5FB; c.envoy = GW_ENVOY_MODE_V1;
  c.match_blob = blob; c.match_blob_len = sizeof blob;
  c.cb.remote_input = on_remote; c.cb.desync = on_desync;
  if (g_host) {
    printf("PORT %u\n", (unsigned)gw_net_udp_local_port(&t));
    n = gw_net_host(&c, &t);
  } else {
    gw_net_addr a;
    a.ip = 0x7F000001u; a.port = (uint16_t)atoi(argv[2]);
    n = gw_net_join(&c, &t, &a);
  }
  deadline = GetTickCount() + 10000;
  while ((int32_t)(deadline - GetTickCount()) > 0) {
    gw_net_poll(n, frame);
    if (gw_net_state(n) == GW_NET_DEAD || gw_net_state(n) == GW_NET_REFUSED) break;
    if (gw_net_started(n) && gw_net_unacked(n) < 30 && frame < FIRST + FRAMES) {
      gw_net_input in[GW_NET_MAX_SLOTS];
      int s;
      memset(in, 0, sizeof in);
      for (s = 0; s < 8; ++s) if (gw_net_local_slots(n) & (1u << s)) mk_payload(frame, s, g_host ? 0 : 1, in[s].b);
      if (gw_net_submit_local(n, frame, in) == 0) {
        gw_net_report_checksum(n, frame, (uint32_t)frame * 2654435761u ^ 0x5BD1E995u);
        frame++;
      }
    }
    if (g_delivered >= FRAMES && frame >= FIRST + FRAMES && gw_net_unacked(n) == 0) break;
    Sleep(1);
  }
  {
    gw_net_config rc;
    uint8_t rb[GW_NET_MAX_BLOB];
    int ok = gw_net_remote_config(n, &rc, rb, sizeof rb);
    if (!ok || rc.rules != 0x5FB || rc.envoy != GW_ENVOY_MODE_V1 || rc.match_blob_len != 64 || memcmp(rb, blob, 64) != 0) {
      printf("FAIL config\n"); gw_net_free(n); return 1;
    }
  }
  rv = !(g_bad == 0 && g_delivered >= FRAMES && !g_chk_bad);
  if (rv) printf("FAIL bad=%d delivered=%u state=%d chk=%u %s\n", g_bad, g_delivered, gw_net_state(n), g_chk_bad, gw_net_last_reason(n));
  else printf("OK %u %u\n", g_delivered, gw_net_rtt_ms(n));
  /* keep answering for a moment so the Rust side sees our acks, then QUIT (gw_net_free) */
  deadline = GetTickCount() + 300;
  while ((int32_t)(deadline - GetTickCount()) > 0) { gw_net_poll(n, frame); Sleep(1); }
  gw_net_free(n);
  return rv;
}
