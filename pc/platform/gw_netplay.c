/* gw_netplay.c - the game-side adapter between the rollback session (gw_rollback.c) and the
 * transport (gw_net.c): two copies of the port playing one match over UDP.
 *
 *   MELEE_NETPLAY=host[:port]          wait for a guest (default port 51500); this side is P1
 *   MELEE_NETPLAY=join:<ip>[:port]     connect to a host; this side is P2
 *   MELEE_NETPLAY_DELAY=<frames>       host: input delay for both sides (default 2)
 *   MELEE_NETPLAY_BIND=<ip>            local address to bind (default: 127.0.0.1 when the peer is
 *                                      on this machine, else all interfaces)
 *   MELEE_NET_SIM / MELEE_NET_SIM_FILE simulated lag, jitter, loss, duplication and lag spikes on
 *                                      this copy's outgoing packets (see the simulator below)
 *
 * Both sides boot straight into the match with the same MELEE_SCENE (gmscenelaunch.h). At the
 * point a replay would restore its match struct (gw_Replay_ApplyMatch, fn_8016E730) the peers
 * connect: the host sends its StartMeleeData and a seed, the guest adopts both, and both begin
 * frame -123 at the start time the transport agrees. From there each render tick:
 *
 *   gw_RB_Iterations -> gw_Netplay_Tick -> gw_net_poll: remote inputs go to the session
 *   (gw_rb_submit_remote_input), ours are pulled from it (gw_rb_local_input_for_send), checksums
 *   of final frames are exchanged (gw_rb_checksum), and the transport's time-sync advice becomes
 *   session waits (gw_rb_request_wait).
 *
 * Inputs travel as raw controller statuses (the session's live-pad source), so both machines run
 * the game's own pad pipeline - calibration, deadzones, UCF - on identical bytes.
 * Protocol: _research/rollback-net.md. Session model: _research/rollback-session.md.
 */
#include "gw.h"
#include "gw_net.h"
#include "gw_rollback.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <windows.h>

#define NP_DEFAULT_PORT 51500
#define NP_PAYLOAD 11 /* buttons u16, stick x/y, c-stick x/y, L, R, analog A, analog B, err */
#define NP_FIRST_FRAME (-123)

extern void gw_RB_SetDelay(int d);

static struct {
    int tried, enabled, host;
    char peer[128];
    uint16_t port;
    int delay;
    gw_net *net;
    int started, dead;
    long ticks;
    int desync_frame;
} np;

static void np_config(void) {
    const char *v;
    if (np.tried) {
        return;
    }
    np.tried = 1;
    np.port = NP_DEFAULT_PORT;
    v = getenv("MELEE_NETPLAY");
    if (v == NULL || v[0] == '\0' || v[0] == '0') {
        return;
    }
    if (strncmp(v, "host", 4) == 0) {
        np.host = 1;
        if (v[4] == ':') {
            np.port = (uint16_t) atoi(v + 5);
        }
    } else if (strncmp(v, "join:", 5) == 0) {
        np.host = 0;
        snprintf(np.peer, sizeof np.peer, "%s", v + 5);
    } else {
        gw_log("netplay: MELEE_NETPLAY=\"%s\" not understood (host[:port] or join:<ip>[:port])", v);
        return;
    }
    v = getenv("MELEE_NETPLAY_DELAY");
    np.delay = v != NULL ? atoi(v) : 2;
    if (np.delay < 0) np.delay = 0;
    if (np.delay > 8) np.delay = 8;
    np.enabled = 1;
    np.desync_frame = GW_NET_NO_FRAME;
}

int gw_Netplay_Enabled(void) {
    np_config();
    return np.enabled;
}
int gw_Netplay_LocalPort(void) { return np.host ? 0 : 1; }
int gw_Netplay_RemotePort(void) { return np.host ? 1 : 0; }
int gw_Netplay_Delay(void) {
    np_config();
    return np.delay;
}

/* ---- the wire format of one slot's input ----------------------------------------------------- */

static void np_encode(const GwRbInput *in, uint8_t *o) {
    o[0] = (uint8_t) (in->buttons >> 8);
    o[1] = (uint8_t) in->buttons;
    o[2] = (uint8_t) in->raw[0];
    o[3] = (uint8_t) in->raw[1];
    o[4] = (uint8_t) in->raw[2];
    o[5] = (uint8_t) in->raw[3];
    o[6] = in->pad_l;
    o[7] = in->pad_r;
    o[8] = in->pad_a;
    o[9] = in->pad_b;
    o[10] = (uint8_t) in->pad_err;
}

static void np_decode(const uint8_t *b, GwRbInput *in) {
    memset(in, 0, sizeof *in);
    in->present = 1;
    in->is_raw = 1;
    in->confirmed = 1;
    in->buttons = ((uint32_t) b[0] << 8) | b[1];
    in->raw[0] = (int8_t) b[2];
    in->raw[1] = (int8_t) b[3];
    in->raw[2] = (int8_t) b[4];
    in->raw[3] = (int8_t) b[5];
    in->pad_l = b[6];
    in->pad_r = b[7];
    in->pad_a = b[8];
    in->pad_b = b[9];
    in->pad_err = (int8_t) b[10];
}

/* ---- transport callbacks --------------------------------------------------------------------- */

static void np_cb_remote(void *user, int32_t frame, int slot, const uint8_t *payload) {
    GwRbInput in;
    (void) user;
    np_decode(payload, &in);
    gw_rb_submit_remote_input(slot, frame, &in);
}

static int np_cb_local(void *user, int32_t frame, int slot, uint8_t *out) {
    GwRbInput in;
    (void) user;
    if (!gw_rb_local_input_for_send(slot, frame, &in)) {
        return 0;
    }
    np_encode(&in, out);
    return 1;
}

static int np_cb_checksum(void *user, int32_t frame, uint32_t *out) {
    uint32_t h;
    (void) user;
    h = gw_rb_checksum(frame);
    *out = h;
    return h != 0;
}

static void np_cb_event(void *user, int ev, const char *msg) {
    static const char *const names[] = { "?", "accepted", "starting", "started", "refused",
                                         "interrupted", "resumed", "disconnected" };
    (void) user;
    gw_log("netplay: %s%s%s", ev >= 1 && ev <= 7 ? names[ev] : "?", msg != NULL ? " - " : "",
           msg != NULL ? msg : "");
    if (ev == GW_NET_EV_STARTED) {
        np.started = 1;
    }
    if (ev == GW_NET_EV_REFUSED || ev == GW_NET_EV_DISCONNECTED) {
        np.dead = 1;
    }
}

static void np_cb_desync(void *user, int32_t frame, uint32_t local, uint32_t remote) {
    (void) user;
    np.desync_frame = frame;
    gw_log("netplay: DESYNC at frame %d - local checksum %08X, peer %08X", frame, local, remote);
}

/* ---- network conditions simulator ------------------------------------------------------------
 * Wraps the real UDP transport, so everything above it - handshake, redundancy, resends, time
 * sync, the rollback session - meets the conditions exactly as it would on a bad connection.
 * Applied to what THIS copy sends, so each direction takes its sender's settings (the launcher
 * gives both copies the same ones: round trip = 2 x lag).
 *
 *   MELEE_NET_SIM="lag=60,jitter=15,loss=5"     at start
 *   MELEE_NET_SIM_FILE=<path>                   same syntax, re-read every second: edit it live
 *
 *   lag=<ms>       one-way delay added to every packet
 *   jitter=<ms>    +- random extra delay per packet (so packets also arrive out of order)
 *   loss=<%>       packets dropped at random
 *   burst=<n>      a drop also takes the next n-1 packets (bursty loss, as on Wi-Fi)
 *   dup=<%>        packets delivered twice
 *   spike=<every_ms>:<len_ms>   every `every_ms`, hold ALL packets for `len_ms` (a lag spike)
 *   off            no simulation (the same as an empty setting)
 */
typedef struct NpSimPkt {
    uint32_t due;
    gw_net_addr to;
    int len;
    uint8_t data[1500];
} NpSimPkt;

#define NP_SIM_MAX 2048

static struct {
    int on;
    int lag, jitter, loss, burst, dup, spike_every, spike_len;
    gw_net_transport inner;
    NpSimPkt *q[NP_SIM_MAX];
    int nq;
    uint32_t rng;
    int burst_left;
    uint32_t spike_next, spike_end;
    const char *file;
    FILETIME file_time;
    uint32_t file_checked;
    uint32_t n_sent, n_dropped, n_dup;
    int changed; /* settings changed since the window title last showed them */
} sim;

static uint32_t np_ms(void) {
    static LARGE_INTEGER f;
    LARGE_INTEGER c;
    if (f.QuadPart == 0) {
        QueryPerformanceFrequency(&f);
    }
    QueryPerformanceCounter(&c);
    return (uint32_t) (c.QuadPart * 1000 / f.QuadPart);
}

static uint32_t np_rand(void) {
    sim.rng ^= sim.rng << 13;
    sim.rng ^= sim.rng >> 17;
    sim.rng ^= sim.rng << 5;
    return sim.rng;
}

static void np_sim_parse(const char *s) {
    char buf[256];
    char *tok, *ctx = NULL;
    sim.lag = sim.jitter = sim.loss = sim.burst = sim.dup = sim.spike_every = sim.spike_len = 0;
    snprintf(buf, sizeof buf, "%s", s != NULL ? s : "");
    for (tok = strtok_s(buf, ",; \t\r\n", &ctx); tok != NULL; tok = strtok_s(NULL, ",; \t\r\n", &ctx)) {
        char *eq = strchr(tok, '=');
        int v = eq != NULL ? atoi(eq + 1) : 0;
        if (strncmp(tok, "lag", 3) == 0 || strncmp(tok, "delay", 5) == 0) sim.lag = v;
        else if (strncmp(tok, "jitter", 6) == 0) sim.jitter = v;
        else if (strncmp(tok, "loss", 4) == 0 || strncmp(tok, "drop", 4) == 0) sim.loss = v;
        else if (strncmp(tok, "burst", 5) == 0) sim.burst = v;
        else if (strncmp(tok, "dup", 3) == 0) sim.dup = v;
        else if (strncmp(tok, "spike", 5) == 0 && eq != NULL) {
            sim.spike_every = atoi(eq + 1);
            sim.spike_len = strchr(eq, ':') != NULL ? atoi(strchr(eq, ':') + 1) : 250;
        }
    }
    if (sim.lag < 0) sim.lag = 0;
    if (sim.jitter < 0) sim.jitter = 0;
    if (sim.loss > 100) sim.loss = 100;
    if (sim.dup > 100) sim.dup = 100;
    sim.spike_next = sim.spike_every > 0 ? np_ms() + (uint32_t) sim.spike_every : 0;
    sim.changed = 1;
    gw_log("netplay: network sim - lag %d ms, jitter %d ms, loss %d%% (burst %d), dup %d%%, "
           "spike %d ms every %d ms", sim.lag, sim.jitter, sim.loss, sim.burst, sim.dup,
           sim.spike_len, sim.spike_every);
}

/* MELEE_NET_SIM_FILE: re-read when it changes, checked once a second. */
static void np_sim_poll_file(void) {
    WIN32_FILE_ATTRIBUTE_DATA a;
    uint32_t now = np_ms();
    if (sim.file == NULL || now - sim.file_checked < 1000u) {
        return;
    }
    sim.file_checked = now;
    if (!GetFileAttributesExA(sim.file, GetFileExInfoStandard, &a) ||
        CompareFileTime(&a.ftLastWriteTime, &sim.file_time) == 0) {
        return;
    }
    sim.file_time = a.ftLastWriteTime;
    {
        char buf[256] = { 0 };
        FILE *f = fopen(sim.file, "r");
        if (f != NULL) {
            size_t n = fread(buf, 1, sizeof buf - 1, f);
            buf[n] = '\0';
            fclose(f);
            np_sim_parse(buf);
        }
    }
}

/* Send everything due, in due order (the queue is small; a linear scan is fine). */
static void np_sim_flush(void) {
    uint32_t now = np_ms();
    int i = 0;
    if (sim.spike_every > 0 && sim.spike_len > 0 && (int32_t) (now - sim.spike_next) >= 0) {
        sim.spike_end = now + (uint32_t) sim.spike_len;
        sim.spike_next = now + (uint32_t) sim.spike_every;
        gw_log("netplay: network sim - lag spike, %d ms", sim.spike_len);
    }
    if ((int32_t) (now - sim.spike_end) < 0) {
        return; /* inside a spike: everything waits */
    }
    while (i < sim.nq) {
        NpSimPkt *p = sim.q[i];
        if ((int32_t) (now - p->due) >= 0) {
            sim.inner.send(sim.inner.ctx, &p->to, p->data, p->len);
            free(p);
            sim.q[i] = sim.q[--sim.nq];
        } else {
            ++i;
        }
    }
}

static void np_sim_enqueue(const gw_net_addr *to, const void *data, int len) {
    NpSimPkt *p;
    int j = 0;
    if (sim.nq >= NP_SIM_MAX || len > (int) sizeof p->data) {
        sim.n_dropped++;
        return;
    }
    p = (NpSimPkt *) malloc(sizeof *p);
    if (p == NULL) {
        return;
    }
    if (sim.jitter > 0) {
        j = (int) (np_rand() % (uint32_t) (2 * sim.jitter + 1)) - sim.jitter;
    }
    p->due = np_ms() + (uint32_t) (sim.lag + j > 0 ? sim.lag + j : 0);
    p->to = *to;
    p->len = len;
    memcpy(p->data, data, (size_t) len);
    sim.q[sim.nq++] = p;
}

static int np_sim_send(void *ctx, const gw_net_addr *to, const void *data, int len) {
    (void) ctx;
    np_sim_poll_file();
    sim.n_sent++;
    if (sim.burst_left > 0) {
        sim.burst_left--;
        sim.n_dropped++;
    } else if (sim.loss > 0 && (int) (np_rand() % 100u) < sim.loss) {
        sim.n_dropped++;
        sim.burst_left = sim.burst > 1 ? sim.burst - 1 : 0;
    } else {
        np_sim_enqueue(to, data, len);
        if (sim.dup > 0 && (int) (np_rand() % 100u) < sim.dup) {
            sim.n_dup++;
            np_sim_enqueue(to, data, len);
        }
    }
    np_sim_flush();
    return len; /* the caller only learns about loss the way it would on a real network */
}

static int np_sim_recv(void *ctx, gw_net_addr *from, void *buf, int cap) {
    (void) ctx;
    np_sim_flush();
    return sim.inner.recv(sim.inner.ctx, from, buf, cap);
}

static void np_sim_close(void *ctx) {
    int i;
    (void) ctx;
    for (i = 0; i < sim.nq; ++i) {
        free(sim.q[i]);
    }
    sim.nq = 0;
    sim.inner.close(sim.inner.ctx);
}

/* Wrap `t` in the simulator when MELEE_NET_SIM or MELEE_NET_SIM_FILE is set. */
static void np_sim_wrap(gw_net_transport *t) {
    const char *v = getenv("MELEE_NET_SIM");
    const char *f = getenv("MELEE_NET_SIM_FILE");
    if ((v == NULL || v[0] == '\0') && (f == NULL || f[0] == '\0')) {
        return;
    }
    sim.on = 1;
    sim.rng = (uint32_t) GetCurrentProcessId() * 2654435761u | 1u;
    sim.inner = *t;
    sim.file = f != NULL && f[0] != '\0' ? f : NULL;
    np_sim_parse(v);
    np_sim_poll_file();
    t->ctx = &sim;
    t->send = np_sim_send;
    t->recv = np_sim_recv;
    t->close = np_sim_close;
}

/* ---- the handshake --------------------------------------------------------------------------- */

static void np_pump(void) {
    MSG m;
    while (PeekMessageA(&m, NULL, 0, 0, PM_REMOVE)) {
        TranslateMessage(&m);
        DispatchMessageA(&m);
    }
}

static BOOL CALLBACK np_title_cb(HWND w, LPARAM title) {
    if (IsWindowVisible(w)) {
        SetWindowTextA(w, (const char *) title);
    }
    return TRUE;
}

static void np_set_title(const char *t) {
    EnumThreadWindows(GetCurrentThreadId(), np_title_cb, (LPARAM) t);
}

/* The window title once connected: role, port, delay, and the simulated conditions if any. */
static void np_connected_title(char *title, size_t cap) {
    size_t n;
    snprintf(title, cap, "Melee netplay - %s (P%d) - connected, input delay %d",
             np.host ? "HOST" : "GUEST", gw_Netplay_LocalPort() + 1, np.delay);
    if (sim.on) {
        n = strlen(title);
        snprintf(title + n, cap - n, " | SIMULATED: lag %d, jitter %d, loss %d%%, dup %d%%%s%s",
                 sim.lag, sim.jitter, sim.loss, sim.dup, sim.spike_every > 0 ? ", spikes" : "",
                 sim.file != NULL ? " (live file)" : "");
    }
    sim.changed = 0;
    np_set_title(title);
}

static uint64_t np_exe_hash(void) {
    char path[MAX_PATH];
    DWORD n = GetModuleFileNameA(NULL, path, sizeof path);
    return n > 0 && n < sizeof path ? gw_net_hash_file(path, 0) : 0;
}

/* gw_Replay_ApplyMatch, live mode: connect, agree on the match and the seed, and return when
 * frame -123 is due. `d` is the StartMeleeData the scene launcher built; the guest's is replaced by
 * the host's except bytes [keep_off, keep_off + keep_len) (the port's own callback pointers).
 * Returns the seed. */
uint32_t gw_Netplay_Handshake(uint8_t *d, int len, int keep_off, int keep_len) {
    gw_net_config cfg;
    gw_net_transport t;
    gw_net_addr peer;
    uint32_t bind_ip = 0;
    uint32_t seed = 0;
    char title[160];
    DWORD t0;
    int accepted = 0;

    np_config();
    if (!np.enabled || np.net != NULL) {
        return 0;
    }
    memset(&cfg, 0, sizeof cfg);
    memset(&peer, 0, sizeof peer);
    if (!np.host && gw_net_addr_parse(np.peer, NP_DEFAULT_PORT, &peer) != 0) {
        gw_log("netplay: cannot parse the host address \"%s\"", np.peer);
        np.dead = 1;
        return 0;
    }
    {
        const char *b = getenv("MELEE_NETPLAY_BIND");
        gw_net_addr ba;
        if (b != NULL && b[0] != '\0' && gw_net_addr_parse(b, 0, &ba) == 0) {
            bind_ip = ba.ip;
        } else if (!np.host && (peer.ip >> 24) == 127) {
            bind_ip = 0x7F000001u; /* the peer is on this machine: no firewall prompt */
        }
    }
    if (gw_net_udp_open(bind_ip, np.host ? np.port : 0, &t) != 0) {
        gw_log("netplay: cannot open UDP port %u", np.host ? np.port : 0);
        np.dead = 1;
        return 0;
    }
    np_sim_wrap(&t);

    cfg.exe_hash = np_exe_hash();
    cfg.first_frame = NP_FIRST_FRAME;
    cfg.payload_bytes = NP_PAYLOAD;
    cfg.cb.remote_input = np_cb_remote;
    cfg.cb.local_input = np_cb_local;
    cfg.cb.checksum = np_cb_checksum;
    cfg.cb.event = np_cb_event;
    cfg.cb.desync = np_cb_desync;
    if (np.host) {
        LARGE_INTEGER c;
        QueryPerformanceCounter(&c);
        seed = (uint32_t) (c.QuadPart * 2654435761u) ^ GetCurrentProcessId();
        cfg.seed = seed != 0 ? seed : 1;
        seed = cfg.seed;
        cfg.input_delay = (uint8_t) np.delay;
        cfg.host_slots = (uint8_t) (1u << (2 * gw_Netplay_LocalPort()));
        cfg.guest_slots = (uint8_t) (1u << (2 * gw_Netplay_RemotePort()));
        cfg.match_blob = d;
        cfg.match_blob_len = (uint16_t) len;
        np.net = gw_net_host(&cfg, &t);
        snprintf(title, sizeof title, "Melee netplay - HOST (P1), waiting for a guest on port %u",
                 np.port);
    } else {
        np.net = gw_net_join(&cfg, &t, &peer);
        snprintf(title, sizeof title, "Melee netplay - GUEST (P2), connecting to %s", np.peer);
    }
    if (np.net == NULL) {
        t.close(t.ctx);
        gw_log("netplay: could not start the session");
        np.dead = 1;
        return 0;
    }
    gw_log("netplay: %s", title);
    np_set_title(title);

    /* Wait for the peer, then for the agreed start time. The window keeps pumping messages. */
    t0 = GetTickCount();
    while (!np.started && !np.dead) {
        gw_net_poll(np.net, NP_FIRST_FRAME);
        if (!np.host && !accepted && gw_net_state(np.net) >= GW_NET_ACCEPTED &&
            gw_net_state(np.net) <= GW_NET_RUNNING) {
            gw_net_config hc;
            uint8_t blob[GW_NET_MAX_BLOB];
            if (gw_net_remote_config(np.net, &hc, blob, sizeof blob) && hc.match_blob_len == len) {
                uint8_t keep[256];
                if (keep_len > (int) sizeof keep) keep_len = (int) sizeof keep;
                memcpy(keep, d + keep_off, (size_t) keep_len);
                memcpy(d, blob, (size_t) len);
                memcpy(d + keep_off, keep, (size_t) keep_len);
                seed = hc.seed;
                np.delay = hc.input_delay;
                gw_RB_SetDelay(np.delay);
                accepted = 1;
                gw_log("netplay: took the host's match (seed 0x%08X, input delay %d)", seed,
                       np.delay);
            } else if (!accepted) {
                gw_log("netplay: the host's match struct is %d bytes, expected %d", hc.match_blob_len,
                       len);
                np.dead = 1;
            }
        }
        if (np.host && GetTickCount() - t0 > 600000u) {
            gw_log("netplay: no guest after 10 minutes - giving up");
            np.dead = 1;
        }
        np_pump();
        Sleep(1);
    }
    if (np.dead) {
        gw_log("netplay: not connected (%s) - the match will wait for a peer that never comes",
               gw_net_last_reason(np.net));
        np_set_title("Melee netplay - NOT CONNECTED");
        return seed;
    }
    np_connected_title(title, sizeof title);
    gw_log("netplay: %s, rtt %u ms", title, gw_net_rtt_ms(np.net));
    return seed;
}

/* Once per render tick during the match (gw_RB_Iterations). */
void gw_Netplay_Tick(void) {
    int w;
    if (np.net == NULL) {
        return;
    }
    gw_net_poll(np.net, gw_rb_current_frame());
    w = gw_net_recommend_wait(np.net);
    if (w > 0) {
        gw_rb_request_wait(w);
    }
    if (sim.changed && np.started) {
        char title[200];
        np_connected_title(title, sizeof title);
    }
    if ((++np.ticks % 600) == 0) {
        gw_net_stats s;
        gw_net_get_stats(np.net, &s);
        gw_log("netplay: frame %d, rtt %u ms, advantage %.1f, remote confirmed %d, rollbacks %d, "
               "desyncs %d, packets %u/%u, resent %u%s", gw_rb_current_frame(), gw_net_rtt_ms(np.net),
               gw_net_frame_advantage(np.net), gw_net_remote_confirmed_frame(np.net),
               gw_rb_rollbacks(), gw_rb_desyncs(), s.packets_sent, s.packets_received,
               s.resent_frames, np.desync_frame != GW_NET_NO_FRAME ? " - CHECKSUM DESYNC" : "");
        if (sim.on) {
            gw_log("netplay: network sim - %u sent, %u dropped, %u duplicated, %d in flight",
                   sim.n_sent, sim.n_dropped, sim.n_dup, sim.nq);
        }
    }
}
