/* gw_netplay.c - online play: the game-side adapter between the rollback session (gw_rollback.c)
 * and the transport (gw_net.c), plus what it takes to reach a friend on another network.
 *
 * HOW A MATCH COMES TOGETHER
 *   1. Set up: VERSUS > ONLINE (gmfrontend.c) hosts a room or joins one by its code
 *      (gw_Netplay_MenuBegin; the host also sets the rules). Scripted runs set the same through
 *      MELEE_NETPLAY* instead, and connect at boot (gw_Netplay_Scene), without the lobby.
 *   2. Connect: the matchmaking server (netplay_server.txt) gives the host a 4-character room code
 *      and introduces the guest who types it; both punch toward each other and fall back to the
 *      server's relay when no direct path opens (the rendezvous layer below). Without a server the
 *      scripted path still dials an address directly (host: STUN + UPnP; guest: the host's code).
 *   3. Agree: in the LOBBY (menu path) the host runs the pick/ban rules and sends the match (a scene
 *      string, gmscenelaunch.h grammar, with the rules) once both are ready; the scripted path
 *      folds the guest's HELLO choice into the match straight away.
 *   4. Load: both seed VS mode from that scene and load the match. The transport holds the start
 *      (hold_start) until both have loaded, then agrees a start time.
 *   5. Play: at the point a replay would restore its match struct (gw_Replay_ApplyMatch) each side
 *      releases the hold and waits for the start; from there gw_Netplay_Tick runs every render tick
 *      inside gw_RB_Iterations - remote inputs in, ours out, checksums, time sync.
 *   6. When the match scene ends the session closes, but a room-code room stays open on the server:
 *      after the results both players land back in the lobby, which reconnects them to the same
 *      room (gw_Netplay_Rejoin) and carries on with the set.
 *
 * Inputs travel as raw controller statuses, so both machines run the game's own pad pipeline -
 * calibration, deadzones, UCF - on identical bytes. Protocol: _research/rollback-net.md.
 *
 * Scripted (two windows on one machine: _build/netplay_local.ps1):
 *   MELEE_NETPLAY=host[:port] | join:<ip>[:port]     connect at boot
 *   MELEE_NETPLAY_CHAR / _COLOR / _STAGE / _STOCKS / _MINUTES / _DELAY
 *   MELEE_NETPLAY_BIND=<ip>                           local address (default 127.0.0.1 for a local peer)
 *   MELEE_NET_SIM / MELEE_NET_SIM_FILE                simulated network conditions (see below)
 */
#include "gw.h"
#include "gw_net.h"
#include "gw_rollback.h"
#include "gw_mexid.h" /* delta: content identities - mods online (gw_mexid.h) */
#include "gw_script.h" /* charlie: gameplay scripts join the must-match set */
#include <stdio.h>
#include <stdint.h>

/* The must-match set: the global game data (gw_mexid.c) and every enabled gameplay-affecting
 * script (gw_script.c; cosmetic/overlay scripts are not in it). Used for the room handshake
 * (cfg.mods_hash, refused with a message naming both) and for random matchmaking (the server
 * only pairs identical hashes). */
static uint64_t np_global_hash(void) {
    const uint64_t g = gw_MexId_GlobalHash();
    const uint64_t sc = gw_Script_GameplayHash();
    return sc == 0 ? g : g ^ (sc * 0x9E3779B97F4A7C15ull + 0x5CB1u);
}

static const char *np_global_describe(void) {
    static char buf[1024];
    const char *sc = gw_Script_GameplayDescribe();
    if (sc == NULL || sc[0] == '\0') return gw_MexId_GlobalDescribe();
    snprintf(buf, sizeof buf, "%s; gameplay scripts: %s", gw_MexId_GlobalDescribe(), sc);
    return buf;
}
#include "gw_mods.h"

/* gw_net.c refuse_check's text for a mods_hash (= global game data) mismatch; the host's
 * gw_MexId_GlobalDescribe follows it */
#define NP_GLOBAL_REFUSAL "different global game data; host has: "

#include <winsock2.h>
#include <ws2tcpip.h>

#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <windows.h>

#define NP_DEFAULT_PORT 51500
#define NP_PAYLOAD 11 /* buttons u16, stick x/y, c-stick x/y, L, R, analog A, analog B, err */
#define NP_FIRST_FRAME (-123)

extern void gw_RB_SetDelay(int d);
extern void gw_SceneLaunch_SetText(const char *text);
extern void gw_Replay_ArmLive(int on);

enum { NP_IDLE, NP_WORKING, NP_CONNECTED, NP_FAILED, NP_RUNNING, NP_LOBBY };

static struct {
    int env_tried;
    int enabled;          /* a netplay match is agreed and armed (the session and live mode run) */
    int host;
    /* the setup */
    int ck, color, stage_ext, stocks, minutes, delay;
    char peer_code[96];   /* guest: the host's code; host: the guest's code, for hole punching */
    char peer_name[24];   /* the opponent's player name (lobby "N"), "" none */
    uint16_t port;
    /* the connection */
    gw_net *net;
    gw_net_transport t;
    int phase;
    char status[96];
    char code[96];        /* this side's code: "public:port/lan:port" (see np_parse_code) */
    gw_net_addr mypub;    /* this socket's public address, from STUN */
    int have_mypub;
    char scene[400];      /* the agreed match */
    char info[64];        /* guest: its choices, sent in the HELLO */
    uint32_t seed;
    gw_net_config cfg;    /* kept: a guest joins only once the server has introduced the host */
    int t_open;           /* np.t is open and not yet owned by a gw_net */
    int rematch;          /* a room-code match just ended: the lobby reconnects to the same room */
    int rejoining;        /* this connection is a return to the room after a match (gw_Netplay_Rejoin) */
    uint32_t rejoin_until;/* a returning guest keeps asking for the room until then (GetTickCount) */
    int peer_left;        /* the connection ended because the opponent left (not an error of ours) */
    int use_lobby;        /* the menu path: connected players meet in the pick/ban lobby first */
    int stage_mode;       /* rooms this side hosts: 0 competitive stage list, 1 all stages */
    int t_open_session;   /* random matchmaking: the session on the queue socket has started */
    int started, dead, accepted;
    long ticks;
    int desync_frame;
    /* hole punching */
    gw_net_addr punch;
    int punch_on;
    uint32_t next_punch;
    /* UPnP */
    HANDLE upnp_proc;
    char upnp_out[MAX_PATH];
    int upnp_state;       /* 0 not tried, 1 working, 2 opened, 3 unavailable */
} np = { .phase = NP_IDLE, .desync_frame = GW_NET_NO_FRAME };

int gw_Netplay_Enabled(void) { return np.enabled; }
int gw_Netplay_LocalPort(void) { return np.host ? 0 : 1; }
int gw_Netplay_RemotePort(void) { return np.host ? 1 : 0; }
int gw_Netplay_Delay(void) { return np.delay; }
/* A netplay MATCH is running against a connected peer: the window must keep its input in the
 * background, so shim_pad.c does not release the GC adapter on focus loss. Searching (random
 * matchmaking), waiting for a guest, the lobby and the menus release normally; if a match starts
 * while the window is in the background, shim_pad.c takes the adapter back for it. */
int gw_Netplay_SessionActive(void) {
    return np.phase == NP_RUNNING && np.net != NULL;
}

static void np_status(const char *fmt, ...) {
    va_list ap;
    va_start(ap, fmt);
    vsnprintf(np.status, sizeof np.status, fmt, ap);
    va_end(ap);
    gw_log("netplay: %s", np.status);
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

/* ---- the match both sides play ---------------------------------------------------------------
 * A scene string (gmscenelaunch.h / _research/scene-launch.md). The rules are part of it, and the
 * scene launcher writes them into the saved rules, so both peers play the same match whatever
 * their memory cards say. */
static void np_build_scene(char *out, size_t cap, int host_ck, int host_c, int guest_ck, int guest_c) {
    /* delta: fighters and the stage by CONTENT IDENTITY ("id:<hex>"), so each side loads its own
       local ids for them - they differ between installs with different mods (gw_mexid.c) */
    char hk[24], gk[24], sk[24];
    snprintf(hk, sizeof hk, "%s", gw_MexId_TokenForCk(host_ck));
    snprintf(gk, sizeof gk, "%s", gw_MexId_TokenForCk(guest_ck));
    snprintf(sk, sizeof sk, "%s", gw_MexId_TokenForExt(np.stage_ext));
    snprintf(out, cap,
             "mode=vs;at=match;p1=%s/c%d/hu;p2=%s/c%d/hu;stage=%s;match=stock;stocks=%d;"
             "minutes=%d;items=off;pause=0",
             hk, host_c, gk, guest_c, sk, np.stocks, np.minutes);
}

static void np_cb_lobby_any(void *user, const uint8_t *data, int len); /* delta */
static void np_mx_pump(void); /* delta */

/* delta: a CharacterKind in the PEER's install -> ours, through the identity lists (gw_mexid.c).
 * Before they arrive only the retail kinds are trusted to mean the same thing. */
static int np_ck_from_peer(int ck) {
    int m;
    if (ck < 0) return ck; /* nothing picked yet */
    m = gw_MexId_LocalCkForPeer(ck);
    if (m >= 0) return m;
    if (gw_MexId_PeerReady() || ck >= 0x22) {
        gw_log("netplay: the peer's fighter %d is not on this install - using Marth", ck);
        return 9;
    }
    return ck;
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
        np_status("%s: %s", ev == GW_NET_EV_REFUSED ? "Refused" : "Disconnected",
                  msg != NULL ? msg : "the other player left");
        /* delta: a global-data mismatch names what differs (gw_mexid.c) */
        if (ev == GW_NET_EV_REFUSED && msg != NULL &&
            strncmp(msg, NP_GLOBAL_REFUSAL, strlen(NP_GLOBAL_REFUSAL)) == 0) {
            char diff[160], peer[256];
            const char *mine = gw_Script_GameplayDescribe();
            char *sc;
            snprintf(peer, sizeof peer, "%s", msg + strlen(NP_GLOBAL_REFUSAL));
            sc = strstr(peer, "; gameplay scripts: "); /* np_global_describe's script part */
            if (sc != NULL) *sc = '\0';
            diff[0] = '\0';
            gw_MexId_GlobalDiff(peer, diff, sizeof diff);
            if (diff[0] != '\0' && strncmp(diff, "same", 4) != 0) {
                np_status("Refused: game data differs from the host's: %s", diff);
            } else {
                np_status("Refused: gameplay scripts differ - host: %s; you: %s",
                          sc != NULL ? sc + strlen("; gameplay scripts: ") : "none",
                          mine != NULL && mine[0] != '\0' ? mine : "none");
            }
        }
    }
}

static void np_cb_desync(void *user, int32_t frame, uint32_t local, uint32_t remote) {
    (void) user;
    np.desync_frame = frame;
    gw_log("netplay: DESYNC at frame %d - local checksum %08X, peer %08X", frame, local, remote);
    {
        extern void gw_RbViz_Desync(int frame, uint32_t local, uint32_t peer);
        gw_RbViz_Desync(frame, local, remote); /* the Geno Lab's rollback visualiser */
    }
}

/* Host: the guest's HELLO carried its choices ("ck:N/cM"); make the match. */
static void np_cb_guest_hello(void *user, const uint8_t *info, int info_len, uint8_t *blob,
                              uint16_t *blob_len, int cap) {
    char s[65];
    int gck = 9, gc = 0;
    (void) user;
    if (info_len > 64) info_len = 64;
    memcpy(s, info, (size_t) info_len);
    s[info_len] = '\0';
    /* any CharacterKind the disc has: m-ex builds add fighters past the 26 retail ones (ACE's run
       to 0x40); both peers play the same disc, so the guest's pick is valid for the host too */
    if (strncmp(s, "id:", 3) == 0) { /* delta: the guest names its fighter by identity */
        char *slash = strchr(s, '/');
        gck = gw_MexId_CkForHex(s + 3);
        if (slash != NULL) sscanf(slash, "/c%d", &gc);
        if (gck < 0) {
            gw_log("netplay: the guest's fighter %s is not on this install - using Marth", s);
            gck = 9;
        }
    } else if (sscanf(s, "ck:%d/c%d", &gck, &gc) < 1 || gck < 0 || gck > 0x7F || gck == 0x21) {
        gck = 9;
    }
    if (gc < 0 || gc > 15) gc = 0;
    np_build_scene(np.scene, sizeof np.scene, np.ck, np.color, gck, gc);
    snprintf((char *) blob, (size_t) cap, "%s", np.scene);
    *blob_len = (uint16_t) (strlen(np.scene) + 1);
    gw_log("netplay: the guest picked \"%s\" - match \"%s\"", s, np.scene);
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


/* ---- reaching the other network ---------------------------------------------------------------- */

/* STUN (RFC 5389) over the game's own socket: the public address and port the router gave THIS
 * socket - exactly what the friend must dial. Before the session starts, so nothing else reads
 * the socket yet. Returns 1 and fills `out` on success. */
static int np_stun(gw_net_transport *t, gw_net_addr *out) {
    static const char *const servers[][2] = {
        { "stun.l.google.com", "19302" },
        { "stun1.l.google.com", "19302" },
        { "stun.cloudflare.com", "3478" },
    };
    int s;
    for (s = 0; s < 3; ++s) {
        struct addrinfo hints, *res = NULL;
        uint8_t req[20], rsp[512];
        gw_net_addr to, from;
        DWORD t0;
        int k;
        memset(&hints, 0, sizeof hints);
        hints.ai_family = AF_INET;
        hints.ai_socktype = SOCK_DGRAM;
        if (getaddrinfo(servers[s][0], servers[s][1], &hints, &res) != 0 || res == NULL) {
            continue;
        }
        to.ip = ntohl(((struct sockaddr_in *) res->ai_addr)->sin_addr.s_addr);
        to.port = ntohs(((struct sockaddr_in *) res->ai_addr)->sin_port);
        freeaddrinfo(res);
        memset(req, 0, sizeof req);
        req[1] = 0x01;                                   /* Binding Request, length 0 */
        req[4] = 0x21; req[5] = 0x12; req[6] = 0xA4; req[7] = 0x42; /* magic cookie */
        for (k = 8; k < 20; ++k) {
            req[k] = (uint8_t) (rand() ^ (GetTickCount() >> k));
        }
        t0 = GetTickCount();
        t->send(t->ctx, &to, req, 20);
        while (GetTickCount() - t0 < 1500) {
            int n = t->recv(t->ctx, &from, rsp, (int) sizeof rsp);
            if (n <= 0) {
                Sleep(5);
                if (GetTickCount() - t0 > 500 && GetTickCount() - t0 < 520) {
                    t->send(t->ctx, &to, req, 20); /* one retry */
                }
                continue;
            }
            if (n >= 20 && rsp[0] == 0x01 && rsp[1] == 0x01 && memcmp(rsp + 4, req + 4, 16) == 0) {
                int off = 20;
                while (off + 4 <= n) {
                    int type = (rsp[off] << 8) | rsp[off + 1];
                    int len = (rsp[off + 2] << 8) | rsp[off + 3];
                    const uint8_t *v = rsp + off + 4;
                    if (off + 4 + len > n) break;
                    if ((type == 0x0020 || type == 0x0001) && len >= 8 && v[1] == 0x01) {
                        uint16_t port = (uint16_t) ((v[2] << 8) | v[3]);
                        uint32_t ip = ((uint32_t) v[4] << 24) | ((uint32_t) v[5] << 16) |
                                      ((uint32_t) v[6] << 8) | v[7];
                        if (type == 0x0020) {
                            port ^= 0x2112;
                            ip ^= 0x2112A442u;
                        }
                        out->ip = ip;
                        out->port = port;
                        gw_log("netplay: STUN %s says this socket is %u.%u.%u.%u:%u", servers[s][0],
                               ip >> 24, (ip >> 16) & 255, (ip >> 8) & 255, ip & 255, port);
                        return 1;
                    }
                    off += 4 + ((len + 3) & ~3);
                }
            }
        }
    }
    gw_log("netplay: STUN - no answer from any server (offline, or UDP blocked)");
    return 0;
}

/* This machine's LAN address: the interface the default route uses. 0 if none. */
static uint32_t np_lan_ip(void) {
    SOCKET s = socket(AF_INET, SOCK_DGRAM, 0);
    struct sockaddr_in a;
    int len = sizeof a;
    uint32_t ip = 0;
    if (s == INVALID_SOCKET) return 0;
    memset(&a, 0, sizeof a);
    a.sin_family = AF_INET;
    a.sin_port = htons(53);
    a.sin_addr.s_addr = htonl(0x08080808u);
    if (connect(s, (struct sockaddr *) &a, sizeof a) == 0 &&
        getsockname(s, (struct sockaddr *) &a, &len) == 0) {
        ip = ntohl(a.sin_addr.s_addr);
    }
    closesocket(s);
    return ip;
}

/* UPnP: ask the router to forward the UDP port to this machine, through Windows' own UPnP client
 * (HNetCfg.NATUPnP), in a hidden PowerShell so the game never waits on the router. */
static void np_upnp_start(uint16_t port) {
    char cmd[1024], tmp[MAX_PATH];
    uint32_t lan = np_lan_ip();
    STARTUPINFOA si;
    PROCESS_INFORMATION pi;
    if (lan == 0) {
        np.upnp_state = 3;
        return;
    }
    GetTempPathA(sizeof tmp, tmp);
    snprintf(np.upnp_out, sizeof np.upnp_out, "%sgdmelee_upnp_%lu.txt", tmp, GetCurrentProcessId());
    DeleteFileA(np.upnp_out);
    snprintf(cmd, sizeof cmd,
             "powershell.exe -NoProfile -NonInteractive -WindowStyle Hidden -Command \"try { "
             "$m = (New-Object -ComObject HNetCfg.NATUPnP).StaticPortMappingCollection; "
             "if ($m) { try { $m.Remove(%u,'UDP') } catch {}; $m.Add(%u,'UDP',%u,'%u.%u.%u.%u',$true,"
             "'GD Melee netplay') | Out-Null; 'ok' } else { 'none' } } catch { 'none' }\" > \"%s\"",
             port, port, port, lan >> 24, (lan >> 16) & 255, (lan >> 8) & 255, lan & 255,
             np.upnp_out);
    memset(&si, 0, sizeof si);
    si.cb = sizeof si;
    si.dwFlags = STARTF_USESHOWWINDOW;
    si.wShowWindow = SW_HIDE;
    {
        char full[1200];
        snprintf(full, sizeof full, "cmd.exe /c %s", cmd);
        if (CreateProcessA(NULL, full, NULL, NULL, FALSE, CREATE_NO_WINDOW, NULL, NULL, &si, &pi)) {
            CloseHandle(pi.hThread);
            np.upnp_proc = pi.hProcess;
            np.upnp_state = 1;
        } else {
            np.upnp_state = 3;
        }
    }
}

static void np_upnp_poll(void) {
    if (np.upnp_state != 1 || np.upnp_proc == NULL ||
        WaitForSingleObject(np.upnp_proc, 0) != WAIT_OBJECT_0) {
        return;
    }
    CloseHandle(np.upnp_proc);
    np.upnp_proc = NULL;
    {
        char buf[64] = { 0 };
        FILE *f = fopen(np.upnp_out, "r");
        if (f != NULL) {
            fread(buf, 1, sizeof buf - 1, f);
            fclose(f);
        }
        DeleteFileA(np.upnp_out);
        np.upnp_state = strstr(buf, "ok") != NULL ? 2 : 3;
    }
    gw_log("netplay: UPnP port forwarding %s", np.upnp_state == 2 ? "opened" : "not available");
}

/* The clipboard (CF_TEXT). */
static int np_clip_get(char *out, int cap) {
    int ok = 0;
    out[0] = '\0';
    if (OpenClipboard(NULL)) {
        HANDLE h = GetClipboardData(CF_TEXT);
        if (h != NULL) {
            const char *p = (const char *) GlobalLock(h);
            if (p != NULL) {
                int i = 0;
                while (*p == ' ' || *p == '\t' || *p == '\r' || *p == '\n') p++;
                while (*p != '\0' && *p != '\r' && *p != '\n' && i < cap - 1) out[i++] = *p++;
                while (i > 0 && (out[i - 1] == ' ' || out[i - 1] == '\t')) i--;
                out[i] = '\0';
                ok = i > 0;
                GlobalUnlock(h);
            }
        }
        CloseClipboard();
    }
    return ok;
}

static void np_clip_set(const char *s) {
    size_t n = strlen(s) + 1;
    HGLOBAL g;
    if (!OpenClipboard(NULL)) return;
    EmptyClipboard();
    g = GlobalAlloc(GMEM_MOVEABLE, n);
    if (g != NULL) {
        memcpy(GlobalLock(g), s, n);
        GlobalUnlock(g);
        SetClipboardData(CF_TEXT, g);
    }
    CloseClipboard();
}

static void np_fmt_addr(char *out, size_t cap, const gw_net_addr *a) {
    snprintf(out, cap, "%u.%u.%u.%u:%u", a->ip >> 24, (a->ip >> 16) & 255, (a->ip >> 8) & 255,
             a->ip & 255, a->port);
}

/* A CODE is "public:port/lan:port" - where the internet reaches this socket, and where this network
 * reaches it. A plain "ip[:port]" (a Tailscale address, say) works too. Returns 0 on success. */
static int np_parse_code(const char *s, gw_net_addr *pub, gw_net_addr *lan, int *has_lan) {
    char a[96];
    char *slash;
    snprintf(a, sizeof a, "%s", s);
    slash = strchr(a, '/');
    *has_lan = 0;
    if (slash != NULL) {
        *slash = '\0';
        if (gw_net_addr_parse(slash + 1, NP_DEFAULT_PORT, lan) == 0 && lan->ip != 0) {
            *has_lan = 1;
        }
    }
    return gw_net_addr_parse(a, NP_DEFAULT_PORT, pub) == 0 && pub->ip != 0 ? 0 : -1;
}

/* Which of a peer's addresses to dial. Behind the same public address as us, the peer is on our
 * own network - often this very machine - and most routers do not loop a connection to their own
 * public address back inside ("NAT hairpinning"), so the local address is the one that works. */
static gw_net_addr np_pick(const gw_net_addr *pub, const gw_net_addr *lan, int has_lan) {
    if (has_lan && (!np.have_mypub || np.mypub.ip == pub->ip)) {
        gw_log("netplay: the other side is on this network - using its local address");
        return *lan;
    }
    return *pub;
}

/* ---- the matchmaking server (tools/netplay/server) -------------------------------------------------
 * With a server configured (netplay_server.txt next to the exe, or MELEE_NETPLAY_SERVER), the
 * player never sees an address: the host gets a 4-letter ROOM CODE, the guest types it, and the
 * server introduces them. They then connect directly (each punching toward the other, local
 * addresses when they share a network) and fall back to the server's RELAY if no direct packet has
 * arrived after NP_RELAY_AFTER_MS. The layer sits under the netcode as a transport wrapper: it eats
 * the server's control packets, and in relay mode it wraps outgoing traffic for the server and
 * unwraps what comes back as if it came from the peer - gw_net never knows. */
#define NP_SERVER_DEFAULT_PORT 51600
#define NP_RELAY_AFTER_MS 3000u
static const char NP_ALPHABET[] = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
int gw_Netplay_MenuHasServer(void);
int gw_Netplay_CodeComplete(void);

static struct {
    int configured;           /* a server address is set and resolved */
    char name[128];
    gw_net_addr srv;
    gw_net_transport inner;   /* the real socket */
    int on;                   /* the layer wraps np.t */
    int have_code, have_peer, relay, got_direct;
    char code[8];
    char err[80];
    gw_net_addr peer;         /* where gw_net thinks the peer is (the address we dial/punch) */
    uint32_t next_send, peer_at;
    int letters[4];           /* the join code as the menu shows it: indexes into NP_ALPHABET, -1 empty */
    int slot;                 /* the code field's active slot */
    int letters_init;
} rdv;

/* The server address: MELEE_NETPLAY_SERVER, else netplay_server.txt beside the exe. */
extern int gw_Settings_Str(const char *key, char *out, int cap, const char *dflt);
static int np_rdv_config(void) {
    char buf[160] = { 0 };
    const char *v = getenv("MELEE_NETPLAY_SERVER");
    struct addrinfo hints, *res = NULL;
    char host[128], port[16];
    char *colon;
    if (v != NULL && v[0] != '\0') {
        snprintf(buf, sizeof buf, "%s", v);
    } else if (gw_Settings_Str("server", buf, sizeof buf, "") && buf[0] != '\0') {
        /* SETTINGS > Online > Server (settings.cfg) */
    } else {
        char path[MAX_PATH];
        DWORD k = GetModuleFileNameA(NULL, path, sizeof path);
        char *slash = k > 0 && k < sizeof path ? strrchr(path, '\\') : NULL;
        if (slash != NULL) {
            FILE *f;
            snprintf(slash + 1, sizeof path - (size_t) (slash + 1 - path), "netplay_server.txt");
            f = fopen(path, "r");
            if (f != NULL) {
                size_t m = fread(buf, 1, sizeof buf - 1, f);
                buf[m] = '\0';
                fclose(f);
            }
        }
    }
    {   /* trim */
        char *s = buf, *e;
        while (*s == ' ' || *s == '\t' || *s == '\r' || *s == '\n') s++;
        e = s + strlen(s);
        while (e > s && (e[-1] == ' ' || e[-1] == '\t' || e[-1] == '\r' || e[-1] == '\n')) *--e = '\0';
        memmove(buf, s, strlen(s) + 1);
    }
    rdv.configured = 0;
    if (buf[0] == '\0' || buf[0] == '#') return 0;
    snprintf(rdv.name, sizeof rdv.name, "%s", buf);
    snprintf(host, sizeof host, "%s", buf);
    snprintf(port, sizeof port, "%d", NP_SERVER_DEFAULT_PORT);
    colon = strrchr(host, ':');
    if (colon != NULL) {
        *colon = '\0';
        snprintf(port, sizeof port, "%s", colon + 1);
    }
    {
        /* the menu asks before any socket exists, and getaddrinfo needs Winsock started */
        static int wsa;
        WSADATA w;
        if (!wsa && WSAStartup(MAKEWORD(2, 2), &w) == 0) wsa = 1;
    }
    memset(&hints, 0, sizeof hints);
    hints.ai_family = AF_INET;
    hints.ai_socktype = SOCK_DGRAM;
    if (getaddrinfo(host, port, &hints, &res) != 0 || res == NULL) {
        gw_log("netplay: cannot resolve the server \"%s\" (%d)", buf, WSAGetLastError());
        return 0;
    }
    rdv.srv.ip = ntohl(((struct sockaddr_in *) res->ai_addr)->sin_addr.s_addr);
    rdv.srv.port = ntohs(((struct sockaddr_in *) res->ai_addr)->sin_port);
    freeaddrinfo(res);
    rdv.configured = 1;
    return 1;
}

static void np_rdv_ctl(const char *text) {
    uint8_t b[200];
    size_t l = strlen(text);
    if (l > sizeof b - 4) l = sizeof b - 4;
    memcpy(b, "GDMR", 4);
    memcpy(b + 4, text, l);
    rdv.inner.send(rdv.inner.ctx, &rdv.srv, b, (int) (4 + l));
}

static void np_rdv_lan(char *out, size_t cap) {
    gw_net_addr lan;
    lan.ip = np_lan_ip();
    lan.port = np.host ? np.port : gw_net_udp_local_port(&rdv.inner);
    if (lan.ip == 0) snprintf(out, cap, "-"); else np_fmt_addr(out, cap, &lan);
}

/* ---- random matchmaking (lane beta, B1) ----------------------------------------------------------
 * RandomBegin opens the session's socket and queues it on the server ("RAND <global-data hash>
 * <lan>", repeated every 2 s); the server pairs two players whose GLOBAL game data matches
 * (fighters and stages are intersected per match, so different mod sets pair fine) and answers
 * "MATCH HOST|GUEST <code> <other-public> <other-lan>". The pair is an ordinary persistent room
 * with that code, so from there on it is exactly the room-code path on the same socket: the host
 * hosts (and keeps the code across rematches), the guest joins the code. */
enum { NP_RAND_OFF, NP_RAND_WAITING, NP_RAND_MATCHED, NP_RAND_TIMEOUT, NP_RAND_FAILED };
static struct {
    int state;
    int host;                 /* the role the server gave */
    char code[8];
    char peer_pub[64], peer_lan[64];
    int waiting;              /* players waiting with this global data, as the server last said */
    uint32_t since, next_send;
} rnd;

static void np_rdv_handle(const char *msg) {
    char w0[16] = { 0 }, w1[64] = { 0 }, w2[64] = { 0 };
    sscanf(msg, "%15s %63s %63s", w0, w1, w2);
    if (strcmp(w0, "QUEUED") == 0 && rnd.state == NP_RAND_WAITING) {
        rnd.waiting = atoi(w1);
        return;
    }
    if (strcmp(w0, "MATCH") == 0 && rnd.state == NP_RAND_WAITING) {
        char w3[64] = { 0 }, w4[64] = { 0 };
        sscanf(msg, "%*s %*s %63s %63s %63s", w3, w4, w2);
        rnd.host = strcmp(w1, "HOST") == 0;
        snprintf(rnd.code, sizeof rnd.code, "%.4s", w3);
        snprintf(rnd.peer_pub, sizeof rnd.peer_pub, "%s", w4);
        snprintf(rnd.peer_lan, sizeof rnd.peer_lan, "%s", w2[0] != '\0' ? w2 : "-");
        rnd.state = NP_RAND_MATCHED;
        gw_log("netplay: random: matched as %s in room %s with %s", rnd.host ? "HOST" : "GUEST", rnd.code,
               rnd.peer_pub);
        return;
    }
    if (strcmp(w0, "TIMEOUT") == 0 && rnd.state == NP_RAND_WAITING) {
        rnd.state = NP_RAND_TIMEOUT;
        return;
    }
    if (strcmp(w0, "CANCELED") == 0) {
        return;
    }
    if (strcmp(w0, "CODE") == 0 && !rdv.have_code) {
        gw_net_addr me;
        snprintf(rdv.code, sizeof rdv.code, "%.4s", w1);
        rdv.have_code = 1;
        if (gw_net_addr_parse(w2, 1, &me) == 0 && !np.have_mypub) {
            np.mypub = me; /* the server saw it: as good as STUN */
            np.have_mypub = 1;
        }
        snprintf(np.code, sizeof np.code, "%s", rdv.code);
        np_clip_set(rdv.code);
        np_status("Waiting for an opponent to join...");
    } else if (strcmp(w0, "PEER") == 0 && !rdv.have_peer) {
        gw_net_addr pub, lan;
        int has_lan = strcmp(w2, "-") != 0 && gw_net_addr_parse(w2, 1, &lan) == 0 && lan.ip != 0;
        if (gw_net_addr_parse(w1, 1, &pub) != 0) return;
        rdv.peer = np_pick(&pub, &lan, has_lan);
        rdv.have_peer = 1;
        rdv.peer_at = GetTickCount();
        np.punch = rdv.peer;       /* both sides punch toward the other */
        np.punch_on = 1;
        np.next_punch = 0;
        gw_log("netplay: the server introduced %s (dialling %u.%u.%u.%u:%u)", w1, rdv.peer.ip >> 24,
               (rdv.peer.ip >> 16) & 255, (rdv.peer.ip >> 8) & 255, rdv.peer.ip & 255, rdv.peer.port);
    } else if (strcmp(w0, "RELAY") == 0) {
        if (!rdv.relay) gw_log("netplay: the other side asked for the relay");
        rdv.relay = 1;
    } else if (strcmp(w0, "ERR") == 0) {
        const char *r = strchr(msg, ' ');
        if (!np.host && np.rejoining && (int32_t) (np.rejoin_until - GetTickCount()) > 0) {
            /* back from a match before the host: its room answers again once it has re-registered */
            snprintf(np.status, sizeof np.status, "Waiting for the host to come back...");
            rdv.next_send = GetTickCount() + 1000;
            return;
        }
        snprintf(rdv.err, sizeof rdv.err, "%s", r != NULL ? r + 1 : "server error");
        if (!np.host && strstr(rdv.err, "no room") != NULL) {
            np_status("No room %s. Check the code with your host.", np.peer_code);
        } else {
            np_status("Server: %s", rdv.err);
        }
    } else if (strcmp(w0, "BYE") == 0) {
        /* the host closed the room (a guest leaving never sends BYE: see np_rdv_close) */
        if (!np.host) {
            np.dead = 1;
            np_status("The host closed the room");
        }
    }
}

static int np_rdv_send(void *ctx, const gw_net_addr *to, const void *data, int len) {
    (void) ctx;
    if (rdv.relay) {
        uint8_t b[1600];
        if (len > (int) sizeof b - 4) return -1;
        memcpy(b, "GDMD", 4);
        memcpy(b + 4, data, (size_t) len);
        rdv.inner.send(rdv.inner.ctx, &rdv.srv, b, len + 4);
        return len;
    }
    return rdv.inner.send(rdv.inner.ctx, to, data, len);
}

static int np_rdv_recv(void *ctx, gw_net_addr *from, void *buf, int cap) {
    uint8_t b[1600];
    (void) ctx;
    for (;;) {
        int n = rdv.inner.recv(rdv.inner.ctx, from, b, (int) sizeof b);
        if (n <= 0) return n;
        if (from->ip == rdv.srv.ip && from->port == rdv.srv.port) {
            if (n >= 4 && memcmp(b, "GDMR", 4) == 0) {
                char msg[200];
                int l = n - 4 < (int) sizeof msg - 1 ? n - 4 : (int) sizeof msg - 1;
                memcpy(msg, b + 4, (size_t) l);
                msg[l] = '\0';
                np_rdv_handle(msg);
                continue;
            }
            if (n >= 4 && memcmp(b, "GDMD", 4) == 0) {
                if (!rdv.have_peer) continue;
                if (n - 4 > cap) continue;
                memcpy(buf, b + 4, (size_t) (n - 4));
                *from = rdv.peer;       /* as if it came straight from the peer */
                return n - 4;
            }
            continue;
        }
        if (rdv.have_peer && from->ip == rdv.peer.ip) {
            rdv.got_direct = 1;
        }
        if (n > cap) continue;
        memcpy(buf, b, (size_t) n);
        return n;
    }
}

static int np_keep_room; /* close without leaving the room (persistent rooms, between matches) */

/* ---- the room between matches ----------------------------------------------------------------
 * The session socket is closed while the players sit on the results screen; the server still
 * holds the room (15 min for a paired room), but a keepalive from the same address keeps it, and
 * the router's mapping for that port, fresh however long the results take. From
 * gw_Netplay_Background (every scene), until the next session opens (np_close). */
static struct {
    int on;
    gw_net_transport t;
    uint32_t next;
} ka;

static void np_ka_stop(void) {
    if (ka.on) {
        ka.t.close(ka.t.ctx);
        ka.on = 0;
    }
}

static void np_ka_start(uint16_t port) {
    np_ka_stop();
    if (!rdv.configured || port == 0) return;
    if (gw_net_udp_open(0, port, &ka.t) == 0) {
        ka.on = 1;
        ka.next = 0;
        gw_log("netplay: holding the room from port %u until the next game", port);
    }
}

static void np_ka_tick(void) {
    uint32_t now = GetTickCount();
    uint8_t b[64];
    gw_net_addr from;
    if (!ka.on) return;
    while (ka.t.recv(ka.t.ctx, &from, b, (int) sizeof b) > 0) {
    }
    if ((int32_t) (now - ka.next) >= 0) {
        memcpy(b, "GDMRKA", 6);
        ka.t.send(ka.t.ctx, &rdv.srv, b, 6);
        ka.next = now + 3000;
    }
}
static void np_rdv_close(void *ctx) {
    (void) ctx;
    /* Only the host closes the room. The server drops the whole room on a BYE, so a guest that
       leaves just goes quiet (its QUIT tells the host) and the host keeps its code for the next
       opponent. */
    if (np.host && rdv.have_code && !np_keep_room) np_rdv_ctl("BYE");
    rdv.inner.close(rdv.inner.ctx);
    rdv.on = 0;
}

static void np_rdv_wrap(gw_net_transport *t) {
    rdv.inner = *t;
    rdv.on = 1;
    rdv.have_code = rdv.have_peer = rdv.relay = rdv.got_direct = 0;
    rdv.err[0] = '\0';
    rdv.next_send = 0;
    t->ctx = &rdv;
    t->send = np_rdv_send;
    t->recv = np_rdv_recv;
    t->close = np_rdv_close;
}

/* Registration, joining, keepalives and the relay decision; call every poll. Before the netcode
 * exists (a guest waiting for its introduction) this also drains the socket itself. */
static void np_rdv_service(void) {
    uint32_t now = GetTickCount();
    char lan[64], msg[128];
    if (!rdv.on) return;
    if (np.net == NULL) {
        gw_net_addr from;
        uint8_t b[1600];
        while (np_rdv_recv(NULL, &from, b, (int) sizeof b) > 0) {
        }
    }
    if ((int32_t) (now - rdv.next_send) >= 0) {
        np_rdv_lan(lan, sizeof lan);
        if (np.host && !rdv.have_code) {
            snprintf(msg, sizeof msg, "REG %s", lan);
            np_rdv_ctl(msg);
            rdv.next_send = now + 500;
        } else if (!np.host && !rdv.have_peer) {
            snprintf(msg, sizeof msg, "JOIN %s %s", np.peer_code, lan);
            np_rdv_ctl(msg);
            rdv.next_send = now + 500;
        } else {
            np_rdv_ctl("KA");
            rdv.next_send = now + 3000;
        }
    }
    if (!np.host && np.rejoining && rdv.have_peer && !np.accepted && now - rdv.peer_at > 5000) {
        /* a returning guest introduced before the host was back (the host re-registering empties
           the seat): ask again so the server introduces us to the host that is listening now */
        rdv.have_peer = 0;
        rdv.relay = rdv.got_direct = 0; /* and give the new introduction its own chance at a direct path */
        rdv.next_send = now;
        gw_log("netplay: not accepted 5 s after the introduction - asking for the room again");
    }
    if (rdv.have_peer && !rdv.relay && getenv("MELEE_NETPLAY_FORCE_RELAY") != NULL) {
        rdv.relay = 1; /* testing: take the relay path from the start */
        np_rdv_ctl("RELAY");
        gw_log("netplay: relay forced (MELEE_NETPLAY_FORCE_RELAY)");
    }
    if (rdv.have_peer && !rdv.relay && !rdv.got_direct && now - rdv.peer_at > NP_RELAY_AFTER_MS) {
        rdv.relay = 1;
        np_rdv_ctl("RELAY");
        gw_log("netplay: no direct path after %u ms - relaying through the server", NP_RELAY_AFTER_MS);
    }
}

/* ---- the lobby: competitive pick/ban and ready-up ----------------------------------------------
 * Once connected (menu path) the two players meet here before every game. The HOST runs the rules:
 * the guest sends requests ("A <action> ..."), the host validates them, applies them and sends the
 * whole state back ("S ...") after every change; its own actions it applies directly. When both are
 * ready the host sends the match ("G <seed> <scene>") and both go into it. Messages travel on the
 * transport's lobby channel (reliable, ordered).
 *
 * The rules (a common modern singles ruleset):
 *   Game 1: characters double-blind (each picks on the real CSS; revealed when both are locked),
 *           then stage striking over the STARTERS - a coin flip picks who strikes first, strikes
 *           go 1-2-2 (then alternate 2s), capped so one stage remains: 1-2-1 over five starters.
 *           The COUNTERPICKS sit out game 1.
 *   Game 2+: the previous game's winner bans 2 stages, the loser picks one of the rest; then the
 *           winner picks a character, then the loser (counterpick, not blind).
 *   Then both press Ready (either can take it back); a 3-second countdown; the match.
 * Players: 0 = host (P1), 1 = guest (P2).
 *
 * STAGE LISTS - the host picks one when hosting (gw_Netplay_SetStageMode):
 *   0 Competitive: the six legal stages, the strike/ban rules above. Starters: Battlefield, Final
 *     Destination, Dream Land, Yoshi's Story, Fountain of Dreams; counterpick: Pokemon Stadium
 *     (lb_starter_ext - the one table to change for another ruleset).
 *   1 All stages: every stage both players have (delta's content identities). Striking a long
 *     list would take forever, so every game is ban-and-pick: game 1 the coin winner bans 2 and
 *     the other picks; game 2+ the last winner bans 2 and the loser picks.
 * The host builds the list (only stages both have, once the peer's identity list is in - rebuilt
 * then if nothing was struck yet) and sends it as identities ("L" chunks); each side maps them
 * to its own external ids, so the grids match whatever each install numbers them.
 * Either list is ordered starters first, then counterpicks; which group a stage is in follows
 * from its (mapped) external id, so both sides agree without it travelling
 * (gw_Netplay_LobbyStageGroup). A list with fewer than two starters has no groups: every stage
 * counts as a starter.
 *
 * The rules are pure functions of `lb` (lb_new_set / lb_reset_game / lb_apply), so they are tested
 * headless (gw_netplay_tests_register). */
static void np_close(void);
static void np_copy(char *out, int cap, const char *s);
enum { LB_OFF, LB_CHAR_BLIND, LB_STRIKE, LB_BAN, LB_PICK, LB_CHAR_WINNER, LB_CHAR_LOSER, LB_READY, LB_GO };
enum { LB_FREE = 0, LB_STRUCK_P1 = 1, LB_STRUCK_P2 = 2, LB_BANNED = 3, LB_PICKED = 4 };
#define LB_MAX_STAGES 256
#define LB_LIST_CHUNK 8 /* identity tokens per "L" message (they are 19 bytes each) */
#define LB_COUNTDOWN 180 /* 3-2-1 */
/* The starters (external ids), in the order the grid shows them: Battlefield, Final Destination,
   Dream Land, Yoshi's Story, Fountain of Dreams. Every other stage is a counterpick. */
static const int lb_starter_ext[] = { 31, 32, 28, 8, 2 };
/* the competitive list: the starters, then the counterpick (Pokemon Stadium) */
static const int lb_default_ext[] = { 31, 32, 28, 8, 2, 3 };
static struct {
    int phase, game, winner, score[2];
    int turn;            /* whose action it is (strike/ban/pick/char counterpick) */
    int left;            /* actions left in this turn */
    int step;            /* game 1 striking: which entry of the 1-2-2 order */
    int first;           /* game 1: who strikes first (the coin flip) */
    int mode;            /* the stage list: 0 competitive, 1 all stages */
    int nstages;
    int stage_ext[LB_MAX_STAGES];
    int list_id;         /* the list's version (host counts; the guest's copy says which it holds) */
    int list_final;      /* host: built with the peer's identities in (no more rebuilds) */
    int list_tx;         /* host: next index to send in "L" chunks, -1 = all sent */
    int rx_id, rx_total, rx_count; /* guest: the list arriving */
    int list_ok;         /* guest: holds the list the host's state names */
    int stage[LB_MAX_STAGES];
    int chosen;          /* index of the stage the game is on, -1 none yet */
    int ck[2], color[2], locked[2], ready[2];
    int countdown;       /* frames, once both are ready */
    uint32_t seed;       /* the coin flip's source (the host's session seed; the same on both) */
    int have_state;      /* guest: the host's state has arrived on this connection */
    uint32_t cd_t0;      /* GetTickCount at the countdown's zero, 0 = not running */
    uint32_t seq;        /* state version, for the menu to notice changes */
} lb;

static const int lb_strike_counts[3] = { 1, 2, 2 }; /* 1-2-2 */

/* MODS ONLINE: whether both players have this fighter (port CharacterKind) / stage (external
 * stage id), from delta's content identities (gw_mexid.c): 1 both have it, 0 not common, -1 the
 * peer's list is not in yet (counted as available until it is). The lobby's stage list and the
 * online CSS pick ask these; A2's online CSS/SSS grey out what they refuse. */
int gw_Netplay_FighterAvailable(int ck) { return ck >= 0 && gw_MexId_OnlineFighter(ck) != 0; }
int gw_Netplay_StageAvailable(int ext) { return ext >= 0 && gw_MexId_OnlineStage(ext) != 0; }

static void lb_default_stages(void) {
    int i;
    lb.nstages = 0;
    for (i = 0; i < (int) (sizeof lb_default_ext / sizeof lb_default_ext[0]); ++i) {
        if (gw_Netplay_StageAvailable(lb_default_ext[i])) lb.stage_ext[lb.nstages++] = lb_default_ext[i];
    }
}

/* ---- starters and counterpicks ---- */
static int lb_ext_starter_rank(int ext) {
    int i;
    for (i = 0; i < (int) (sizeof lb_starter_ext / sizeof lb_starter_ext[0]); ++i) {
        if (lb_starter_ext[i] == ext) return i;
    }
    return -1;
}

static int lb_count_starters(void) {
    int i, k = 0;
    for (i = 0; i < lb.nstages; ++i) k += lb_ext_starter_rank(lb.stage_ext[i]) >= 0;
    return k;
}

/* 1 if list entry i is a starter. A list with fewer than two starters has no groups. */
static int lb_is_starter(int i) {
    if (i < 0 || i >= lb.nstages) return 0;
    if (lb_count_starters() < 2) return 1;
    return lb_ext_starter_rank(lb.stage_ext[i]) >= 0;
}

/* Starters first (in lb_starter_ext's order), then the rest in list order. */
static void lb_order_groups(void) {
    int tmp[LB_MAX_STAGES], n = 0, r, i;
    for (r = 0; r < (int) (sizeof lb_starter_ext / sizeof lb_starter_ext[0]); ++r) {
        for (i = 0; i < lb.nstages; ++i) {
            if (lb.stage_ext[i] == lb_starter_ext[r]) {
                tmp[n++] = lb.stage_ext[i];
                break;
            }
        }
    }
    for (i = 0; i < lb.nstages; ++i) {
        if (lb_ext_starter_rank(lb.stage_ext[i]) < 0) tmp[n++] = lb.stage_ext[i];
    }
    memcpy(lb.stage_ext, tmp, sizeof tmp[0] * (size_t) n);
    lb.nstages = n;
}

/* Host: the stage list for the room's mode - only stages both players have (a stage whose peer
   answer is not in yet counts, and the list is rebuilt once it is). */
static void lb_build_list(void) {
    int i, n;
    if (lb.mode == 0) {
        lb_default_stages();
    } else {
        lb.nstages = 0;
        n = gw_MexId_Count();
        for (i = 0; i < n && lb.nstages < LB_MAX_STAGES; ++i) {
            int ext, k, dup = 0;
            if (gw_MexId_Kind(i) != GW_MEXID_STAGE) continue;
            ext = gw_MexId_LocalId(i);
            if (ext <= 0 || !gw_Netplay_StageAvailable(ext)) continue;
            for (k = 0; k < lb.nstages; ++k) dup |= lb.stage_ext[k] == ext;
            if (!dup) lb.stage_ext[lb.nstages++] = ext;
        }
        if (lb.nstages == 0) lb_default_stages();
        lb_order_groups();
    }
    lb.list_id++;
    lb.list_tx = 0;
    lb.list_final = gw_MexId_PeerReady();
    gw_log("netplay: lobby - stage list %d (%s): %d stages%s", lb.list_id, lb.mode ? "all" : "competitive",
           lb.nstages, lb.list_final ? "" : " (the opponent's stages are not in yet)");
}

/* A new opponent: game 1, no winner, 0-0. */
static void lb_new_set(int host_ck, int guest_ck) {
    lb.game = 1;
    lb.winner = -1;
    lb.score[0] = lb.score[1] = 0;
    lb.ck[0] = host_ck;
    lb.ck[1] = guest_ck;
    lb.color[0] = lb.color[1] = 0;
    if (lb.nstages <= 0) lb_default_stages();
    lb.seq++;
}

/* The next game of the set: game 1 (or after a game nobody won) starts with blind characters;
   otherwise the last winner bans. */
static void lb_reset_game(void) {
    int i;
    if (lb.nstages <= 0) lb_default_stages();
    for (i = 0; i < LB_MAX_STAGES; ++i) lb.stage[i] = LB_FREE;
    lb.chosen = -1;
    lb.locked[0] = lb.locked[1] = 0;
    lb.ready[0] = lb.ready[1] = 0;
    lb.countdown = 0;
    lb.step = 0;
    if (lb.game <= 1 || lb.winner < 0) {
        lb.phase = LB_CHAR_BLIND;
        lb.first = (int) ((lb.seed >> 7) & 1u); /* the coin flip: from the host's seed, same for both */
        lb.turn = lb.first;
        lb.left = lb_strike_counts[0];
    } else {
        lb.phase = LB_BAN;                      /* winner bans 2, loser picks */
        lb.turn = lb.winner;
        lb.left = 2;
    }
    lb.seq++;
}

static int lb_stages_free(void) {
    int i, k = 0;
    for (i = 0; i < lb.nstages; ++i) k += lb.stage[i] == LB_FREE;
    return k;
}

/* Striking is over the free starters only. */
static int lb_starters_free(void) {
    int i, k = 0;
    for (i = 0; i < lb.nstages; ++i) k += lb.stage[i] == LB_FREE && lb_is_starter(i);
    return k;
}

static void lb_after_stage(void) {
    /* the stage is settled: game 1 goes to Ready (characters were picked blind first); game 2+ goes
       to the counterpick characters, winner first */
    if (lb.game <= 1 || lb.winner < 0) {
        lb.phase = LB_READY;
    } else {
        lb.phase = LB_CHAR_WINNER;
        lb.turn = lb.winner;
    }
}

/* Apply one player's action (host). Returns 1 if the state changed. */
static int lb_apply_(int who, const char *act, int a, int b) {
    if (who < 0 || who > 1) return 0;
    if (strcmp(act, "CHAR") == 0) {
        if (lb.phase == LB_CHAR_BLIND && !lb.locked[who]) {
            lb.ck[who] = a;
            lb.color[who] = b;
            lb.locked[who] = 1;
            if (lb.locked[0] && lb.locked[1]) {
                if (lb.mode == 1) {
                    lb.phase = LB_BAN; /* all stages: the coin winner bans 2, the other picks */
                    lb.turn = lb.first;
                    lb.left = 2;
                } else {
                    lb.phase = LB_STRIKE;
                }
            }
            return 1;
        }
        if ((lb.phase == LB_CHAR_WINNER || lb.phase == LB_CHAR_LOSER) && lb.turn == who) {
            lb.ck[who] = a;
            lb.color[who] = b;
            lb.locked[who] = 1;
            if (lb.phase == LB_CHAR_WINNER) {
                lb.phase = LB_CHAR_LOSER;
                lb.turn = 1 - who;
            } else {
                lb.phase = LB_READY;
            }
            return 1;
        }
        return 0;
    }
    if (strcmp(act, "READY") == 0) {
        if (lb.phase != LB_READY || lb.ready[who] == (a != 0)) return 0;
        lb.ready[who] = a != 0;
        lb.countdown = lb.ready[0] && lb.ready[1] ? LB_COUNTDOWN : 0;
        return 1;
    }
    if (a < 0 || a >= lb.nstages || lb.turn != who || lb.stage[a] != LB_FREE) return 0;
    if (strcmp(act, "STRIKE") == 0 && lb.phase == LB_STRIKE) {
        if (!lb_is_starter(a)) return 0; /* a counterpick: not in game 1 */
        lb.stage[a] = who == 0 ? LB_STRUCK_P1 : LB_STRUCK_P2;
        if (--lb.left <= 0 && lb_starters_free() > 1) {
            lb.step++;
            lb.turn = 1 - lb.turn;
            lb.left = lb_strike_counts[lb.step < 3 ? lb.step : 2];
        }
        if (lb.left > lb_starters_free() - 1) lb.left = lb_starters_free() - 1;
        if (lb_starters_free() <= 1) {
            int i;
            for (i = 0; i < lb.nstages; ++i) {
                if (lb.stage[i] == LB_FREE && lb_is_starter(i)) {
                    lb.stage[i] = LB_PICKED;
                    lb.chosen = i;
                }
            }
            lb.left = 0;
            lb_after_stage();
        }
        return 1;
    }
    if (strcmp(act, "BAN") == 0 && lb.phase == LB_BAN) {
        lb.stage[a] = LB_BANNED;
        if (--lb.left <= 0 || lb_stages_free() <= 1) {
            lb.phase = LB_PICK;
            lb.turn = 1 - who;
            lb.left = 1;
        }
        return 1;
    }
    if (strcmp(act, "PICK") == 0 && lb.phase == LB_PICK) {
        lb.stage[a] = LB_PICKED;
        lb.chosen = a;
        lb.left = 0;
        lb_after_stage();
        return 1;
    }
    return 0;
}

static int lb_apply(int who, const char *act, int a, int b) {
    int before = lb.phase, r = lb_apply_(who, act, a, b);
    gw_log("netplay: lobby - P%d %s %d %d -> %s (phase %d -> %d, turn P%d, %d left)", who + 1, act, a, b,
           r ? "applied" : "refused", before, lb.phase, lb.turn + 1, lb.left);
    return r;
}

/* The countdown, by the clock: `elapsed` frames (1/60 s) since both became ready. Wall time, not
   frames counted by whoever calls, so it lasts three seconds at any frame rate. Returns 1 when it
   ran out with both still ready: the match starts. */
static int lb_countdown_at(int elapsed) {
    if (lb.phase != LB_READY || lb.countdown <= 0) return 0;
    if (!(lb.ready[0] && lb.ready[1])) {
        lb.countdown = 0;
        return 0;
    }
    lb.countdown = LB_COUNTDOWN - (elapsed < 0 ? 0 : elapsed);
    if (lb.countdown <= 0) {
        lb.countdown = 0;
        return 1;
    }
    return 0;
}

/* "S ..." - the whole state, host to guest. Stage states travel sparse ("i.s,i.s", "-" when every
   stage is free), so a long list fits one message; the list itself (mode, length, version) is at
   the end and arrives separately ("L"). */
static void lb_encode(char *m, int cap) {
    char st[120];
    int i, o = 0;
    st[0] = '\0';
    for (i = 0; i < lb.nstages && o < (int) sizeof st - 12; ++i) {
        if (lb.stage[i] != LB_FREE) o += snprintf(st + o, sizeof st - (size_t) o, "%s%d.%d", o ? "," : "", i, lb.stage[i]);
    }
    snprintf(m, (size_t) cap, "S %d %d %d %d %d %d %d %d %d %s %d %d %d %d %d %d %d %d %d %d %d %d %d", lb.phase,
             lb.game, lb.winner, lb.score[0], lb.score[1], lb.turn, lb.left, lb.step, lb.first,
             o ? st : "-", lb.chosen, lb.ck[0], lb.color[0], lb.ck[1], lb.color[1],
             lb.locked[0], lb.locked[1], lb.ready[0], lb.ready[1], lb.countdown, lb.mode, lb.nstages,
             lb.list_id);
}

static int lb_decode(const char *m) {
    char st[124] = { 0 };
    int mode = 0, n = 0, id = 0;
    int k = sscanf(m + 2, "%d %d %d %d %d %d %d %d %d %123s %d %d %d %d %d %d %d %d %d %d %d %d %d", &lb.phase,
                   &lb.game, &lb.winner, &lb.score[0], &lb.score[1], &lb.turn, &lb.left, &lb.step,
                   &lb.first, st, &lb.chosen, &lb.ck[0], &lb.color[0], &lb.ck[1], &lb.color[1],
                   &lb.locked[0], &lb.locked[1], &lb.ready[0], &lb.ready[1], &lb.countdown, &mode, &n, &id);
    if (k >= 10) {
        const char *c = st;
        int i;
        for (i = 0; i < LB_MAX_STAGES; ++i) lb.stage[i] = LB_FREE;
        while (*c != '\0' && *c != '-') {
            int ix = 0, v = 0, used = 0;
            if (sscanf(c, "%d.%d%n", &ix, &v, &used) < 2 || used <= 0) break;
            if (ix >= 0 && ix < LB_MAX_STAGES) lb.stage[ix] = v;
            c += used;
            if (*c == ',') c++;
        }
    }
    if (k >= 23) {
        lb.mode = mode;
        /* the grid is the list with that version - until it has arrived, say so */
        lb.list_ok = id == lb.rx_id && lb.rx_count >= lb.rx_total && lb.rx_total == n;
        if (lb.list_ok) lb.nstages = n;
    }
    lb.have_state = 1;
    lb.seq++;
    return k;
}

/* Guest: one "L <id> <start> <total> <tok,tok,...>" chunk of the host's stage list. Tokens are
   content identities ("id:<hex>", or "ext:<n>" for a stage without one), mapped to this install's
   own external ids. Returns 1 when the list is complete. */
static int lb_list_rx(const char *m) {
    int id = 0, start = 0, total = 0, used = 0;
    const char *c;
    if (sscanf(m + 2, "%d %d %d %n", &id, &start, &total, &used) < 3) return 0;
    if (total < 0 || total > LB_MAX_STAGES) return 0;
    if (id != lb.rx_id) {
        lb.rx_id = id;
        lb.rx_total = total;
        lb.rx_count = 0;
    }
    c = m + 2 + used;
    while (*c != '\0' && start < total) {
        char tok[40];
        int k = 0, ext = -1;
        while (*c != '\0' && *c != ',' && k < (int) sizeof tok - 1) tok[k++] = *c++;
        tok[k] = '\0';
        if (*c == ',') c++;
        if (strncmp(tok, "id:", 3) == 0) ext = gw_MexId_ExtForHex(tok + 3);
        else if (strncmp(tok, "ext:", 4) == 0) ext = atoi(tok + 4);
        lb.stage_ext[start++] = ext;
        lb.rx_count++;
    }
    return lb.rx_count >= lb.rx_total;
}

/* The "L" chunk of the list from `start`; returns the index after it. */
static int lb_list_encode(int start, char *m, int cap, int *len) {
    int o, i, end = start + LB_LIST_CHUNK;
    if (end > lb.nstages) end = lb.nstages;
    o = snprintf(m, (size_t) cap, "L %d %d %d ", lb.list_id, start, lb.nstages);
    for (i = start; i < end; ++i) {
        o += snprintf(m + o, (size_t) cap - (size_t) o, "%s%s", i > start ? "," : "",
                      gw_MexId_TokenForExt(lb.stage_ext[i]));
    }
    *len = o;
    return end;
}

/* Host: send what is left of the list, a chunk at a time while the lobby channel has room. */
static void lb_list_pump(void) {
    while (np.net != NULL && lb.list_tx >= 0 && lb.list_tx < lb.nstages) {
        char m[GW_NET_LOBBY_MAX];
        int len, end = lb_list_encode(lb.list_tx, m, sizeof m, &len);
        if (gw_net_lobby_send(np.net, m, len) != 0) return; /* the queue is full: next tick */
        lb.list_tx = end;
    }
    if (lb.list_tx >= lb.nstages) lb.list_tx = -1;
}

static void lb_send(const char *msg) {
    if (np.net != NULL) gw_net_lobby_send(np.net, msg, (int) strlen(msg));
}

/* Host: the whole state for the guest. */
static void lb_broadcast(void) {
    char m[GW_NET_LOBBY_MAX];
    lb_encode(m, sizeof m);
    lb_send(m);
}

/* The match, once the countdown is over (host). */
static void lb_go(void) {
    char m[GW_NET_LOBBY_MAX];
    np.stage_ext = lb.stage_ext[lb.chosen >= 0 ? lb.chosen : 0];
    np_build_scene(np.scene, sizeof np.scene, lb.ck[0], lb.color[0], lb.ck[1], lb.color[1]);
    lb.phase = LB_GO;
    snprintf(m, sizeof m, "G %u %s", np.seed, np.scene);
    lb_send(m);
    lb.seq++;
}

/* A local action (the menu). The host applies it; the guest asks the host. */
static void lb_action(const char *act, int a, int b) {
    if (np.host) {
        if (lb_apply(0, act, a, b)) {
            lb.seq++;
            lb_broadcast();
        }
    } else {
        char m[64];
        snprintf(m, sizeof m, "A %s %d %d", act, a, b);
        lb_send(m);
        gw_log("netplay: lobby - asking the host: %s %d %d", act, a, b);
    }
}

static void np_arm(void);

/* Lobby messages from the peer (gw_net's lobby channel). */
static void np_cb_lobby(void *user, const uint8_t *data, int len) {
    char m[GW_NET_LOBBY_MAX + 1];
    (void) user;
    if (len > GW_NET_LOBBY_MAX) len = GW_NET_LOBBY_MAX;
    memcpy(m, data, (size_t) len);
    m[len] = '\0';
    if (m[0] == 'N' && m[1] == ' ') {
        /* the opponent's player name (SETTINGS > Online > Player Name) - printable ASCII only */
        int i, k = 0;
        for (i = 2; m[i] != '\0' && k < (int) sizeof np.peer_name - 1; ++i) {
            if (m[i] >= 32 && m[i] < 127) np.peer_name[k++] = m[i];
        }
        np.peer_name[k] = '\0';
        lb.seq++;
        return;
    }
    if (np.host && m[0] == 'A') {
        char act[16] = { 0 };
        int a = 0, b = 0;
        if (sscanf(m + 2, "%15s %d %d", act, &a, &b) >= 1 && strcmp(act, "CHAR") == 0) {
            a = np_ck_from_peer(a); /* delta: the guest's CharacterKind -> the host's */
        }
        if (act[0] != '\0' && lb_apply(1, act, a, b)) {
            lb.seq++;
            lb_broadcast();
        }
    } else if (!np.host && m[0] == 'L') {
        if (lb_list_rx(m)) {
            gw_log("netplay: lobby - stage list %d: %d stages", lb.rx_id, lb.rx_total);
            lb.seq++;
        }
    } else if (!np.host && m[0] == 'S') {
        lb_decode(m);
        /* delta: the host's CharacterKinds -> ours (gw_mexid.c) */
        lb.ck[0] = np_ck_from_peer(lb.ck[0]);
        lb.ck[1] = np_ck_from_peer(lb.ck[1]);
        gw_log("netplay: lobby - state from the host: phase %d, turn P%d, %d left, locked %d/%d, ready %d/%d",
               lb.phase, lb.turn + 1, lb.left, lb.locked[0], lb.locked[1], lb.ready[0], lb.ready[1]);
    } else if (!np.host && m[0] == 'G') {
        unsigned seed = 0;
        int off = 0;
        if (sscanf(m + 2, "%u %n", &seed, &off) >= 1) {
            np.seed = seed;
            snprintf(np.scene, sizeof np.scene, "%s", m + 2 + off);
            lb.phase = LB_GO;
            lb.seq++;
        }
    }
}

/* The opponent is gone (a disconnect, a QUIT, or the server's BYE). The host keeps its room open
   for the next opponent (same code); the guest is out. */
static void np_peer_gone(void) {
    int keep = np.host && rdv.on;
    np.phase = NP_FAILED;
    np.peer_left = 1;
    np_keep_room = keep;
    np_close();
    np_keep_room = 0;
    lb.game = 0; /* the next opponent starts a new set */
}

/* Every frame while in the lobby - also while a player is away on the CSS (gmscene.c calls
 * Netplay_Background from every scene): keep the connection alive, run the countdown, and hand
 * the match to the menu once it is agreed. */
static void np_lobby_tick(void) {
    if (np.phase != NP_LOBBY || np.net == NULL) return;
    gw_net_poll(np.net, NP_FIRST_FRAME);
    np_rdv_service();
    if (np.dead) {
        np_peer_gone();
        return;
    }
    /* MELEE_LOBBY_AUTOPLAY is now the built-in script lobby_autoplay (pc/scripts/examples),
       which gw_script.c loads when the variable is set; it acts through gd.netplay_act. */
    if (np.host) {
        lb_list_pump();
        if (!lb.list_final && gw_MexId_PeerReady()) {
            /* the opponent's stages are in: the list can now hold only what both have */
            int i, touched = 0;
            for (i = 0; i < lb.nstages; ++i) touched |= lb.stage[i] != LB_FREE;
            if (!touched) {
                lb_build_list();
                lb.seq++;
                lb_list_pump();
                lb_broadcast();
            } else {
                lb.list_final = 1;
            }
        }
    }
    if (lb.countdown > 0 && lb.cd_t0 == 0) {
        /* started (host: both just readied; guest: the host's state says so): the clock's zero,
           back-dated by what already ran */
        lb.cd_t0 = GetTickCount() - (uint32_t) ((LB_COUNTDOWN - lb.countdown) * 1000 / 60);
        if (lb.cd_t0 == 0) lb.cd_t0 = 1;
    } else if (lb.countdown <= 0) {
        lb.cd_t0 = 0;
    }
    if (lb.countdown > 0) {
        int was = (lb.countdown + 59) / 60;
        int elapsed = (int) ((GetTickCount() - lb.cd_t0) * 60 / 1000);
        if (np.host) {
            if (lb_countdown_at(elapsed)) {
                gw_log("netplay: lobby - countdown done after %u ms", GetTickCount() - lb.cd_t0);
                lb_go();
            } else if (lb.countdown == 0 || (lb.countdown + 59) / 60 != was) {
                lb.seq++;
                lb_broadcast(); /* the guest's numerals follow */
            }
        } else if (lb.phase == LB_READY && elapsed < LB_COUNTDOWN - 1) {
            lb.countdown = LB_COUNTDOWN - elapsed; /* the host's GO ends it */
            if ((lb.countdown + 59) / 60 != was) lb.seq++;
        }
    }
    if (lb.phase == LB_GO && np.phase == NP_LOBBY) {
        np.phase = NP_CONNECTED; /* the menu launches the match */
        np_status("Starting the match...");
        np_arm();
    }
}

/* Every scene frame (gmscene.c). The lobby's own tick; and between the agreed match and its
 * start (the loading screen, the match loading) the connection is kept serviced, so a longer
 * warm-up on one side never looks like a dead peer to the other. */
void gw_Netplay_Background(void) {
    np_ka_tick();
    np_mx_pump(); /* delta: the identity lists, while the lobby sits on the CSS */
    if (np.phase == NP_CONNECTED && np.use_lobby && np.net != NULL) {
        gw_net_poll(np.net, NP_FIRST_FRAME);
        np_rdv_service();
        if (np.dead) np_peer_gone();
        return;
    }
    np_lobby_tick();
}

/* Entering the lobby: a fresh room (or a new opponent) starts at game 1; a rematch keeps the set. */
static void np_lobby_enter(void) {
    char nm[40];
    np.phase = NP_LOBBY;
    np.peer_name[0] = '\0';
    gw_Settings_Str("name", nm + 2, (int) sizeof nm - 2, "");
    if (nm[2] != '\0') {
        nm[0] = 'N';
        nm[1] = ' ';
        lb_send(nm); /* this player's name, for the opponent's cards */
    }
    lb.seed = np.seed;
    if (np.host) {
        if (lb.game <= 0) lb.mode = np.stage_mode; /* a new set takes the room's stage list */
        lb_build_list();
    } else if (!lb.have_state) {
        lb.rx_id = -1; /* the host sends its list on this connection */
        lb.list_ok = 0;
    }
    if (lb.nstages <= 0 || lb.stage_ext[0] == 0) lb_default_stages();
    if (np.host || !lb.have_state) {
        /* the host starts the game; a guest shows the same start until the host's state arrives
           (it may already have: the state can overtake the handshake's last step) */
        if (lb.game <= 0) lb_new_set(np.host ? np.ck : 2, np.host ? 9 : np.ck);
        lb_reset_game();
    }
    if (np.host) {
        lb_list_pump();
        lb_broadcast();
    }
    np_status("Opponent found! Opening the lobby...");
}

/* ---- starting and running a connection --------------------------------------------------------- */

static void np_close(void) {
    np_ka_stop();
    if (np.net != NULL) {
        gw_net_free(np.net); /* sends QUIT; closes the transport */
        np.net = NULL;
    } else if (np.t_open) {
        np.t.close(np.t.ctx); /* a guest still waiting for the server: nothing owns the socket yet */
    }
    np.t_open = 0;
    if (np.upnp_proc != NULL) {
        CloseHandle(np.upnp_proc);
        np.upnp_proc = NULL;
    }
    np.punch_on = 0;
}

static uint64_t np_exe_hash(void) {
    char path[MAX_PATH];
    DWORD n = GetModuleFileNameA(NULL, path, sizeof path);
    return n > 0 && n < sizeof path ? gw_net_hash_file(path, 0) : 0;
}

/* Start hosting or joining with the current setup. 0 on success (the connection proceeds in
 * Netplay_Poll), -1 with np.status saying why. */
static int np_start_session(const gw_net_addr *peer_in, uint32_t bind_ip);
static void np_rand_poll(void);
static int np_begin(int bind_local) {
    gw_net_addr peer, pub, ppub, plan;
    int phas_lan = 0;
    uint32_t bind_ip = 0;
    np_close();
    rnd.state = NP_RAND_OFF; /* a room-code session: not random matchmaking */
    memset(&peer, 0, sizeof peer);
    np.started = np.dead = np.accepted = 0;
    lb.have_state = 0;
    np.enabled = 0;
    np.desync_frame = GW_NET_NO_FRAME;
    np.code[0] = '\0';
    np.upnp_state = 0;
    np.have_mypub = 0;
    np_rdv_config();
    if (!np.host && rdv.configured && strlen(np.peer_code) == 4 && strchr(np.peer_code, '.') == NULL &&
        strchr(np.peer_code, '?') == NULL) {
        /* a room code: the server knows where the host is */
        memset(&ppub, 0, sizeof ppub);
    } else if (!np.host) {
        rdv.configured = 0; /* an address was typed: dial it directly */
        if (np_parse_code(np.peer_code, &ppub, &plan, &phas_lan) != 0) {
            np_status("Paste the host's code first (it looks like 1.2.3.4:51500)");
            np.phase = NP_FAILED;
            return -1;
        }
        peer = ppub;
        if ((ppub.ip >> 24) == 127) bind_ip = 0x7F000001u;
    }
    if (bind_local) bind_ip = 0x7F000001u;
    {
        const char *b = getenv("MELEE_NETPLAY_BIND");
        gw_net_addr ba;
        if (b != NULL && b[0] != '\0' && gw_net_addr_parse(b, 1, &ba) == 0) bind_ip = ba.ip;
    }
    if (gw_net_udp_open(bind_ip, np.host ? np.port : 0, &np.t) != 0) {
        np_status("Could not open UDP port %u (is another copy hosting?)", np.host ? np.port : 0);
        np.phase = NP_FAILED;
        return -1;
    }
    np.t_open = 1;
    /* this side's code: the public address the friend dials, then the local one for players on
       the same network ("public:port/lan:port"; just the local one if STUN fails) */
    {
        gw_net_addr lan;
        lan.ip = bind_ip == 0x7F000001u ? 0x7F000001u : np_lan_ip();
        lan.port = np.host ? np.port : gw_net_udp_local_port(&np.t);
        if (bind_ip != 0x7F000001u && np_stun(&np.t, &pub)) {
            size_t len;
            np.mypub = pub;
            np.have_mypub = 1;
            if (np.host && pub.port != np.port) {
                gw_log("netplay: the router maps port %u to %u (a symmetric or port-shifting NAT)",
                       np.port, pub.port);
            }
            np_fmt_addr(np.code, sizeof np.code, &pub);
            if (lan.ip != 0) {
                len = strlen(np.code);
                np.code[len] = '/';
                np_fmt_addr(np.code + len + 1, sizeof np.code - len - 1, &lan);
            }
        } else {
            np_fmt_addr(np.code, sizeof np.code, &lan);
        }
    }
    if (!np.host && !rdv.configured) {
        peer = np_pick(&ppub, &plan, phas_lan);
    }
    if (rdv.configured) {
        np_rdv_wrap(&np.t); /* under the simulator: simulated conditions apply to relayed traffic too */
    }
    np_sim_wrap(&np.t);
    return np_start_session(&peer, bind_ip);
}

/* The session on np.t, which is open (and wrapped for the server when there is one): the netcode's
 * config, then host, or join now (an address) / later (np_poll, once the server introduced the
 * host). Shared by np_begin and random matchmaking (np_rand_matched). */
static int np_start_session(const gw_net_addr *peer_in, uint32_t bind_ip) {
    gw_net_config cfg;
    gw_net_addr peer = *peer_in;
    memset(&cfg, 0, sizeof cfg);
    cfg.exe_hash = np_exe_hash();
    /* delta: the disc image is no longer compared. Only GLOBAL game data must match (PlCo.dat's
       global tables, ItCo.dat, the m-ex feature flags); fighters and stages are matched one by
       one by content identity and the match offers only what both have (gw_mexid.h). So a mod
       ISO and a vanilla ISO with the same content as loose mods can play each other. */
    cfg.iso_hash = 0;
    cfg.mods_hash = np_global_hash();
    cfg.mods_desc = np_global_describe();
    gw_MexId_PeerReset();
    gw_log("netplay: global game data %s; %d fighter/stage identities; mods: %s", cfg.mods_desc,
           gw_MexId_Count(), gw_Mods_Describe()[0] != '\0' ? gw_Mods_Describe() : "none");
    cfg.first_frame = NP_FIRST_FRAME;
    cfg.payload_bytes = NP_PAYLOAD;
    cfg.hold_start = 1;
    cfg.handshake_timeout_ms = 180000u; /* the host may be waiting on a friend's router */
    cfg.cb.remote_input = np_cb_remote;
    cfg.cb.local_input = np_cb_local;
    cfg.cb.checksum = np_cb_checksum;
    cfg.cb.event = np_cb_event;
    cfg.cb.desync = np_cb_desync;
    cfg.cb.guest_hello = np_cb_guest_hello;
    cfg.cb.lobby_msg = np_cb_lobby_any; /* delta: the identity lists travel on it too */
    if (np.host) {
        LARGE_INTEGER c;
        QueryPerformanceCounter(&c);
        np.seed = (uint32_t) (c.QuadPart * 2654435761u) ^ GetCurrentProcessId();
        if (np.seed == 0) np.seed = 1;
        cfg.seed = np.seed;
        cfg.input_delay = (uint8_t) np.delay;
        cfg.host_slots = (uint8_t) (1u << (2 * 0));
        cfg.guest_slots = (uint8_t) (1u << (2 * 1));
        np_build_scene(np.scene, sizeof np.scene, np.ck, np.color, 9, 0); /* until the guest says */
        cfg.match_blob = np.scene;
        cfg.match_blob_len = (uint16_t) (strlen(np.scene) + 1);
        np.net = gw_net_host(&cfg, &np.t);
        if (rnd.state == NP_RAND_MATCHED) {
            np_status("Opponent found - connecting...");
        } else if (rdv.configured) {
            np.code[0] = '\0';
            np_status("Asking the server for a room...");
        } else {
            if (bind_ip != 0x7F000001u) np_upnp_start(np.port);
            np_clip_set(np.code);
            np_status("Hosting. Your code %s is copied - send it to your friend", np.code);
        }
    } else {
        snprintf(np.info, sizeof np.info, "%s/c%d", gw_MexId_TokenForCk(np.ck), np.color); /* delta */
        cfg.guest_info = np.info;
        cfg.guest_info_len = (uint8_t) strlen(np.info);
        if (rdv.configured) {
            np.cfg = cfg;   /* joins in np_poll once the server has introduced the host */
            np.cfg.guest_info = np.info;
            np.phase = NP_WORKING;
            np_status("Looking for room %s...", np.peer_code);
            return 0;
        }
        np.net = gw_net_join(&cfg, &np.t, &peer);
        np_clip_set(np.code);
        np_status("Connecting to %s ...", np.peer_code);
    }
    if (np.net == NULL) {
        np.t.close(np.t.ctx);
        np_status("Could not start the session");
        np.phase = NP_FAILED;
        return -1;
    }
    np.phase = NP_WORKING;
    return 0;
}

/* Arm everything for the agreed match: the scene, live mode, the rollback session. */
static void np_arm(void) {
    np.enabled = 1;
    gw_Replay_ArmLive(1);
    gw_log("netplay: match agreed - \"%s\" (seed 0x%08X, input delay %d)", np.scene, np.seed,
           np.delay);
}

/* delta: every lobby message goes past the identity exchange first (gw_mexid.c). */
static void np_cb_lobby_any(void *user, const uint8_t *data, int len) {
    if (gw_MexId_WireFeed(data, len)) return;
    if (np.use_lobby) np_cb_lobby(user, data, len);
}

/* delta: send this install's identity list, one lobby chunk every few polls so the lobby's own
 * messages always find room in the channel's queue. */
static void np_mx_pump(void) {
    static unsigned tick;
    uint8_t m[GW_NET_LOBBY_MAX];
    int st, len;
    if (np.net == NULL || (++tick & 3u) != 0) return;
    st = gw_net_state(np.net);
    if (st != GW_NET_ACCEPTED && st != GW_NET_STARTING) return;
    len = gw_MexId_WireNext(m, sizeof m);
    if (len > 0 && gw_net_lobby_send(np.net, m, len) == 0) gw_MexId_WireSent();
}

/* One step of connecting (menu frames, or the boot wait). Returns the phase. */
static int np_poll(void) {
    np_mx_pump();
    if (np.phase == NP_LOBBY) {
        return np.phase; /* ticked once a frame from every scene (gw_Netplay_Background) */
    }
    if (np.phase != NP_WORKING) {
        return np.phase;
    }
    if (rnd.state == NP_RAND_WAITING || (rnd.state == NP_RAND_MATCHED && !np.t_open_session)) {
        np_rand_poll();
        if (rnd.state == NP_RAND_TIMEOUT || rnd.state == NP_RAND_FAILED) {
            if (rnd.state == NP_RAND_TIMEOUT) np_status("No opponent found - try again in a moment");
            np.phase = NP_FAILED;
            np_close();
        }
        return np.phase;
    }
    np_rdv_service();
    if (rdv.on && rdv.err[0] != '\0') {
        np.phase = NP_FAILED;
        np_close();
        return np.phase;
    }
    if (np.net == NULL) {
        if (!rdv.on || !rdv.have_peer) {
            return np.phase; /* a guest waiting for the server's introduction */
        }
        np.net = gw_net_join(&np.cfg, &np.t, &rdv.peer);
        if (np.net == NULL) {
            np_status("Could not start the session");
            np.phase = NP_FAILED;
            return np.phase;
        }
        np_status("Found room %s - connecting...", np.peer_code);
    }
    gw_net_poll(np.net, NP_FIRST_FRAME);
    np_upnp_poll();
    if (np.punch_on && (np.host || rdv.on) && (int32_t) (GetTickCount() - np.next_punch) >= 0 &&
        !(rdv.on && rdv.relay)) {
        static const uint8_t punch[4] = { 'G', 'D', 'P', 'H' };
        gw_net_transport *raw = rdv.on ? &rdv.inner : &np.t;
        raw->send(raw->ctx, &np.punch, punch, 4); /* opens our router toward the other side */
        np.next_punch = GetTickCount() + 200;
    }
    if (np.dead) {
        np.phase = NP_FAILED;
        np_close();
        return np.phase;
    }
    if (!np.accepted) {
        int st = gw_net_state(np.net);
        if (st >= GW_NET_ACCEPTED && st <= GW_NET_RUNNING) {
            if (!np.host) {
                gw_net_config hc;
                char blob[GW_NET_MAX_BLOB];
                if (!gw_net_remote_config(np.net, &hc, blob, sizeof blob)) {
                    return np.phase;
                }
                blob[sizeof blob - 1] = '\0';
                snprintf(np.scene, sizeof np.scene, "%s", blob);
                {
                    /* delta: the host names m-ex content by identity; if this install lacks one
                       of them, say which instead of failing to load the match (gw_mexid.c) */
                    char why[160];
                    if (gw_MexId_SceneCheck(np.scene, why, sizeof why) > 0) {
                        np_status("The host's match uses content you don't have: %s", why);
                        np.phase = NP_FAILED;
                        np_close();
                        return np.phase;
                    }
                }
                np.seed = hc.seed;
                np.delay = hc.input_delay;
                gw_RB_SetDelay(np.delay);
            }
            np.accepted = 1;
            if (np.use_lobby) {
                np_lobby_enter();
            } else {
                np.phase = NP_CONNECTED;
                np_status("Connected! Starting the match...");
                np_arm();
            }
        } else if (rdv.on) {
            if (np.host) {
                if (rdv.have_code && !rdv.have_peer)
                    snprintf(np.status, sizeof np.status, "Waiting for an opponent to join...");
                else if (rdv.have_peer)
                    snprintf(np.status, sizeof np.status, "Opponent found - connecting%s", rdv.relay ? " (relay)" : "...");
            } else if (rdv.have_peer) {
                snprintf(np.status, sizeof np.status, "Room %s found - connecting%s", np.peer_code,
                         rdv.relay ? " (relay)" : "...");
            }
        } else if (np.host) {
            const char *u = np.upnp_state == 1 ? "checking your router..."
                          : np.upnp_state == 2 ? "router port opened, friend can join directly"
                          : np.upnp_state == 3 ? "no auto port forward - if they can't join, paste their code"
                                               : "";
            snprintf(np.status, sizeof np.status, "Waiting for friend (code %s copied) - %s", np.code, u);
        } else {
            snprintf(np.status, sizeof np.status,
                     "Connecting to %s... If it takes long, send your code %s to the host", np.peer_code,
                     np.code);
        }
    }
    return np.phase;
}

/* Random matchmaking, one poll step while queued. */
static void np_rand_poll(void) {
    uint32_t now = GetTickCount();
    if (np.net == NULL && rdv.on) {
        gw_net_addr from;
        uint8_t b[1600];
        while (np_rdv_recv(NULL, &from, b, (int) sizeof b) > 0) {
        }
    }
    if (rnd.state == NP_RAND_WAITING) {
        if ((int32_t) (now - rnd.next_send) >= 0) {
            char lan[64], msg[128];
            np_rdv_lan(lan, sizeof lan);
            snprintf(msg, sizeof msg, "RAND %016llx %s", (unsigned long long) np_global_hash(), lan);
            np_rdv_ctl(msg);
            rnd.next_send = now + 2000;
        }
        snprintf(np.status, sizeof np.status, "Looking for an opponent... %us%s", (now - rnd.since) / 1000u,
                 rnd.waiting > 1 ? " (someone else is looking)" : "");
        if (rdv.err[0] != '\0') {
            rnd.state = NP_RAND_FAILED;
            np_status("Server: %s", rdv.err);
        }
        return;
    }
    if (rnd.state == NP_RAND_MATCHED && np.net == NULL && np.phase == NP_WORKING && !np.t_open_session) {
        /* the pair is a room: continue exactly as that room's host or guest, on this socket */
        char intro[160];
        gw_net_addr none;
        np.host = rnd.host;
        snprintf(np.peer_code, sizeof np.peer_code, "%s", rnd.code);
        snprintf(rdv.code, sizeof rdv.code, "%s", rnd.code);
        snprintf(np.code, sizeof np.code, "%s", rnd.code);
        rdv.have_code = np.host;
        snprintf(intro, sizeof intro, "PEER %s %s", rnd.peer_pub, rnd.peer_lan);
        np_rdv_handle(intro); /* sets rdv.peer and starts punching, as the server's PEER would */
        memset(&none, 0, sizeof none);
        np.t_open_session = 1;
        if (np_start_session(&none, 0) != 0) {
            rnd.state = NP_RAND_FAILED;
        }
    }
}

/* Start looking for a random opponent with these settings (the lobby then works as for a room).
 * 0 = queued; -1 = np.status says why. Poll with gw_Netplay_MenuPoll as for a room; while queued
 * the phase is NP_WORKING and gw_Netplay_RandomStatus says how it is going. */
int gw_Netplay_RandomBegin(int ck, int color, int stocks, int minutes, int delay) {
    uint16_t port = NP_DEFAULT_PORT;
    np_close();
    memset(&rnd, 0, sizeof rnd);
    np.ck = ck;
    np.color = color;
    np.stage_ext = 31;
    np.stocks = stocks;
    np.minutes = minutes;
    np.delay = delay;
    np.use_lobby = 1;
    np.rejoining = np.peer_left = np.rematch = 0;
    np.started = np.dead = np.accepted = 0;
    np.enabled = 0;
    np.desync_frame = GW_NET_NO_FRAME;
    np.code[0] = np.peer_code[0] = '\0';
    np.have_mypub = 0;
    np.t_open_session = 0;
    lb.have_state = 0;
    lb.game = 0;
    if (!np_rdv_config()) {
        np_status("Online play needs the server address (netplay_server.txt beside the game)");
        np.phase = NP_FAILED;
        return -1;
    }
    /* the host's port if it is free (a random pair may become the host, and a rematch reopens the
       same port so the server keeps the room), else any */
    if (gw_net_udp_open(0, port, &np.t) != 0) {
        port = 0;
        if (gw_net_udp_open(0, 0, &np.t) != 0) {
            np_status("Could not open a UDP port");
            np.phase = NP_FAILED;
            return -1;
        }
    }
    np.t_open = 1;
    np.port = port != 0 ? port : gw_net_udp_local_port(&np.t);
    np_rdv_wrap(&np.t);
    np_sim_wrap(&np.t);
    rnd.state = NP_RAND_WAITING;
    rnd.since = GetTickCount();
    rnd.next_send = 0;
    np.phase = NP_WORKING;
    np_status("Looking for an opponent...");
    return 0;
}

/* Stop looking (a no-op once matched: use gw_Netplay_Leave to leave the room). */
void gw_Netplay_RandomCancel(void) {
    if (rnd.state == NP_RAND_WAITING && rdv.on) {
        np_rdv_ctl("RANDCANCEL");
    }
    if (rnd.state != NP_RAND_MATCHED) {
        rnd.state = NP_RAND_OFF;
        np_close();
        np.phase = NP_IDLE;
        np_status("Stopped looking for an opponent");
    }
}

/* 0 off, 1 looking, 2 matched (the room/lobby flow has taken over), 3 nobody found in time,
 * 4 failed (np.status says why). */
int gw_Netplay_RandomStatus(void) { return rnd.state; }

/* Seconds spent looking so far. */
int gw_Netplay_RandomSeconds(void) {
    return rnd.state == NP_RAND_WAITING ? (int) ((GetTickCount() - rnd.since) / 1000u) : 0;
}

/* ---- the in-game ONLINE screens (gmfrontend.c) --------------------------------------------------
 * Game code calls these without the gw_ prefix (gwtool adds it). Integers in, text out by copy.
 *
 *   ONLINE      Host a Room -> gw_Netplay_MenuBegin(1, ...)        -> waiting room (host)
 *               Join a Room -> code entry -> gw_Netplay_MenuBegin(0, ...) -> waiting room (guest)
 *   waiting     gw_Netplay_MenuPoll until NP_LOBBY (or NP_FAILED: gw_Netplay_MenuStatus says why)
 *   lobby       gw_Netplay_Lobby* (state) and LobbyChar / LobbyStageAct / LobbyReady (actions);
 *               NP_CONNECTED = the match is agreed: gw_Netplay_MenuLaunch, then VS mode
 *   after       results -> the lobby again: gw_Netplay_RematchPending -> gw_Netplay_Rejoin
 *   leaving     gw_Netplay_Leave at any step (the host's room closes; a guest just goes) */
int gw_Netplay_MenuBegin(int host, int ck, int color, int stocks, int minutes, int delay) {
    np.host = host != 0;
    np.ck = ck;
    np.color = color;
    np.stage_ext = 31;
    np.stocks = stocks;
    np.minutes = minutes;
    np.delay = delay;
    np.port = NP_DEFAULT_PORT;
    np.use_lobby = 1;
    np.rejoining = 0;
    np.peer_left = 0;
    np.rematch = 0;
    if (!gw_Netplay_MenuHasServer()) {
        np_status("Online play needs the server address (netplay_server.txt beside the game)");
        np.phase = NP_FAILED;
        return -1;
    }
    if (!np.host && !gw_Netplay_CodeComplete()) {
        np_status("Enter the 4-character room code first");
        np.phase = NP_FAILED;
        return -1;
    }
    lb.game = 0; /* a new room: a new set */
    return np_begin(0);
}

/* Back into the same room after a match (or, for a host, after the opponent left): the same
 * settings, the same code. A returning guest keeps asking for the room for a while, since the
 * host may still be on its results screen. */
int gw_Netplay_Rejoin(void) {
    np.rematch = 0;
    np.peer_left = 0;
    np.rejoining = 1;
    np.rejoin_until = GetTickCount() + 30000u;
    np_status(np.host ? "Reopening room %s..." : "Getting back into room %s...",
              np.host ? rdv.code : np.peer_code);
    return np_begin(0);
}

/* Leave the room, whatever the step: the connection closes, the set is over. */
void gw_Netplay_Leave(void) {
    if (rnd.state == NP_RAND_WAITING && rdv.on) np_rdv_ctl("RANDCANCEL");
    rnd.state = NP_RAND_OFF;
    np_close();
    np.phase = NP_IDLE;
    np.enabled = 0;
    np.rematch = 0;
    np.rejoining = 0;
    np.peer_left = 0;
    lb.game = 0;
    lb.phase = LB_OFF;
    lb.seq++;
    np_status("You left the room");
}

int gw_Netplay_MenuPoll(void) { return np_poll(); }
static void np_code_init(void);
static int np_code_from_text(const char *s);
/* For scripts (gd.netplay): the connection phase without advancing it, and the room code. */
int gw_Netplay_Phase(void) { return np.phase; }
const char *gw_Netplay_Status(void) { return np.status; }
const char *gw_Netplay_Code(void) { return np.host ? rdv.code : np.peer_code; }
int gw_Netplay_LocalCk(void) { return np.ck; }
int gw_Netplay_LocalColor(void) { return np.color; }
/* Fill the join code from text (scripts, test drivers); 1 when it was a valid room code. */
int gw_Netplay_SetCode(const char *code) {
    np_code_init();
    return code != NULL && np_code_from_text(code);
}
int gw_Netplay_IsHost(void) { return np.host; }
int gw_Netplay_PeerLeft(void) { return np.peer_left; }
int gw_Netplay_Rejoining(void) { return np.rejoining; }
int gw_Netplay_Ping(void) { return np.net != NULL ? (int) gw_net_rtt_ms(np.net) : -1; }
/* The room: the server's code (host), the code typed (guest). */
void gw_Netplay_RoomCode(char *out, int cap) {
    np_copy(out, cap, np.host ? (rdv.have_code ? rdv.code : "") : np.peer_code);
}
/* Put the room code on the clipboard again (the waiting room's X). */
void gw_Netplay_CopyCode(void) {
    if (np.host && rdv.have_code) np_clip_set(rdv.code);
}

/* A fighter's display name: the m-ex fighter table's for m-ex fighters, the retail name
 * otherwise. */
void gw_Netplay_FighterName(int ck, char *out, int cap) {
    static const char *const retail[] = {
        "Captain Falcon", "Donkey Kong", "Fox", "Mr. Game & Watch", "Kirby", "Bowser", "Link",
        "Luigi", "Mario", "Marth", "Mewtwo", "Ness", "Peach", "Pikachu", "Ice Climbers",
        "Jigglypuff", "Samus", "Yoshi", "Zelda", "Sheik", "Falco", "Young Link", "Dr. Mario", "Roy",
        "Pichu", "Ganondorf",
    };
    extern const char *gw_Mex_FighterName(int ext);
    extern int gw_Mex_PortCKindToExt(int ckind);
    const char *m = ck >= 0 ? gw_Mex_FighterName(gw_Mex_PortCKindToExt(ck)) : NULL;
    char buf[32];
    if (m != NULL && m[0] != '\0') {
        np_copy(out, cap, m);
    } else if (ck >= 0 && ck < (int) (sizeof retail / sizeof retail[0])) {
        np_copy(out, cap, retail[ck]);
    } else {
        snprintf(buf, sizeof buf, "Fighter %d", ck);
        np_copy(out, cap, buf);
    }
}

/* The stage list of rooms this side hosts (0 competitive, 1 all stages). */
void gw_Netplay_SetStageMode(int mode) { np.stage_mode = mode ? 1 : 0; }
int gw_Netplay_StageMode(void) { return np.stage_mode; }

/* A player's name in the room (who: 0 host, 1 guest): this side's from SETTINGS, the opponent's
 * as they sent it. "" when there is none. */
void gw_Netplay_PlayerName(int who, char *out, int cap) {
    if (who == (np.host ? 0 : 1)) {
        gw_Settings_Str("name", out, cap, "");
    } else {
        np_copy(out, cap, np.peer_name);
    }
}

/* ---- the lobby, for the menu ---- */
int gw_Netplay_LobbyPhase(void) { return np.phase == NP_LOBBY || np.phase == NP_CONNECTED ? lb.phase : LB_OFF; }
int gw_Netplay_LobbySeq(void) { return (int) lb.seq; }
int gw_Netplay_LobbyMe(void) { return np.host ? 0 : 1; }
int gw_Netplay_LobbyInfo(int what) {
    switch (what) {
    case 0: return lb.game;
    case 1: return lb.winner;
    case 2: return lb.score[0];
    case 3: return lb.score[1];
    case 4: return lb.turn;
    case 5: return lb.left;
    case 6: return lb.first;
    case 7: return lb.chosen;
    case 8: return (lb.countdown + 59) / 60;
    case 9: return lb.nstages;
    case 10: return lb.countdown;
    case 11: return lb.mode;
    case 12: return np.host || lb.list_ok; /* the stage list is here (a guest waits for it) */
    default: return 0;
    }
}
int gw_Netplay_LobbyStage(int i) { return i >= 0 && i < lb.nstages ? lb.stage[i] : 0; }
int gw_Netplay_LobbyStageExt(int i) { return i >= 0 && i < lb.nstages ? lb.stage_ext[i] : 0; }
/* A stage's group in the list: 0 starter, 1 counterpick (the list has starters first). */
int gw_Netplay_LobbyStageGroup(int i) { return lb_is_starter(i) ? 0 : 1; }
/* Whether the player whose turn it is may act on stage i now: free, and a starter while striking. */
int gw_Netplay_LobbyStageOpen(int i) {
    if (i < 0 || i >= lb.nstages || lb.stage[i] != LB_FREE) return 0;
    return lb.phase != LB_STRIKE || lb_is_starter(i);
}
/* A stage's display name by its external id (the stage select's numbering; m-ex stages past the
 * retail ones are "Stage n" until their names are read from the disc). */
void gw_Netplay_StageNameExt(int ext, char *out, int cap) {
    static const char *const retail[] = {
        "", "Test", "Fountain of Dreams", "Pokemon Stadium", "Princess Peach's Castle",
        "Kongo Jungle", "Brinstar", "Corneria", "Yoshi's Story", "Onett", "Mute City",
        "Rainbow Cruise", "Jungle Japes", "Great Bay", "Hyrule Temple", "Brinstar Depths",
        "Yoshi's Island", "Green Greens", "Fourside", "Mushroom Kingdom", "Mushroom Kingdom II",
        "Akaneia", "Venom", "Poke Floats", "Big Blue", "Icicle Mountain", "Icetop", "Flat Zone",
        "Dream Land", "Yoshi's Island N64", "Kongo Jungle N64", "Battlefield", "Final Destination",
    };
    int mi = gw_MexId_FindStage(ext);
    const char *mn = mi >= 0 ? gw_MexId_Name(mi) : NULL;
    if (mn != NULL && mn[0] != '?' && mn[0] != 0 && !(ext > 0 && ext < 33)) {
        np_copy(out, cap, mn); /* an added stage: the disc's own name */
    } else if (ext > 0 && ext < (int) (sizeof retail / sizeof retail[0])) {
        np_copy(out, cap, retail[ext]);
    } else {
        char buf[24];
        snprintf(buf, sizeof buf, "Stage %d", ext);
        np_copy(out, cap, buf);
    }
}
void gw_Netplay_LobbyStageName(int i, char *out, int cap) {
    gw_Netplay_StageNameExt(gw_Netplay_LobbyStageExt(i), out, cap);
}
int gw_Netplay_LobbyPlayer(int who, int what) {
    if (who < 0 || who > 1) return 0;
    switch (what) {
    case 0: return lb.ck[who];
    case 1: return lb.color[who];
    case 2: return lb.locked[who];
    case 3: return lb.ready[who];
    default: return 0;
    }
}
void gw_Netplay_LobbyChar(int ck, int color) {
    np.ck = ck; /* remembered for the next room too */
    np.color = color;
    lb_action("CHAR", ck, color);
}
void gw_Netplay_LobbyStageAct(int i) {
    lb_action(lb.phase == LB_BAN ? "BAN" : lb.phase == LB_PICK ? "PICK" : "STRIKE", i, 0);
}
void gw_Netplay_LobbyReady(int on) { lb_action("READY", on != 0, 0); }
int gw_Netplay_LobbyActive(void) { return np.phase == NP_LOBBY; }

/* Persistent rooms: after a room-code match both players come back to the lobby, which
 * reconnects them to the same room (gw_Netplay_Rejoin). */
int gw_Netplay_RematchPending(void) { return np.rematch; }
/* The game's winner (0 host/P1, 1 guest/P2, -1 none), from the results screen: the next game's
 * rules (winner bans, loser picks) and the set score. */
void gw_Netplay_GameResult(int winner) {
    if (lb.game <= 0) return;
    lb.winner = winner;
    if (winner == 0 || winner == 1) lb.score[winner]++;
    lb.game++;
    lb.seq++;
    gw_log("netplay: game over - winner P%d, set %d-%d, next game %d", winner + 1, lb.score[0], lb.score[1], lb.game);
}

/* Once connected: make the agreed match the configured scene (VS mode seeds from it). */
void gw_Netplay_MenuLaunch(void) {
    gw_SceneLaunch_SetText(np.scene);
}

static void np_copy(char *out, int cap, const char *s) {
    int i = 0;
    for (; s[i] != '\0' && i < cap - 1; ++i) out[i] = s[i];
    out[i] = '\0';
}
void gw_Netplay_MenuStatus(char *out, int cap) { np_copy(out, cap, np.status); }
/* A matchmaking server is configured (room codes). */
static int np_server_checked = -1;
int gw_Netplay_MenuHasServer(void) {
    if (np_server_checked < 0) np_server_checked = np_rdv_config();
    return np_server_checked;
}
/* SETTINGS changed the server: resolve it again next time. */
void gw_Netplay_ReloadServer(void) { np_server_checked = -1; }
/* The server in use, for the SETTINGS readout ("" none). */
void gw_Netplay_ServerName(char *out, int cap) {
    gw_Netplay_MenuHasServer();
    np_copy(out, cap, rdv.configured ? rdv.name : "");
}
static void np_code_init(void) {
    if (!rdv.letters_init) {
        rdv.letters_init = 1;
        rdv.letters[0] = rdv.letters[1] = rdv.letters[2] = rdv.letters[3] = -1;
        rdv.slot = 0;
    }
}
static void np_code_sync(void) {
    int k;
    for (k = 0; k < 4; ++k) np.peer_code[k] = rdv.letters[k] >= 0 ? NP_ALPHABET[rdv.letters[k]] : '?';
    np.peer_code[4] = '\0';
}

/* ---- the code entry: four slots, a caret, typing, pasting ---- */
int gw_Netplay_CodeSlot(void) { np_code_init(); return rdv.slot; }
/* The character in slot i, or 0 when it is empty. */
int gw_Netplay_CodeChar(int i) {
    np_code_init();
    return i >= 0 && i < 4 && rdv.letters[i] >= 0 ? NP_ALPHABET[rdv.letters[i]] : 0;
}
void gw_Netplay_CodeStep(int dir) {        /* up/down: the active slot's letter, wrapping */
    np_code_init();
    rdv.letters[rdv.slot] = rdv.letters[rdv.slot] < 0 ? (dir > 0 ? 0 : 31)
                                                      : ((rdv.letters[rdv.slot] + dir) % 32 + 32) % 32;
    np_code_sync();
}
int gw_Netplay_CodeMove(int dir) {         /* left/right: the active slot; 0 at an end */
    int to;
    np_code_init();
    to = rdv.slot + (dir > 0 ? 1 : -1);
    if (to < 0 || to > 3) return 0;
    rdv.slot = to;
    return 1;
}
int gw_Netplay_CodeComplete(void) {
    int k;
    np_code_init();
    for (k = 0; k < 4; ++k) if (rdv.letters[k] < 0) return 0;
    return 1;
}
/* The first empty slot (A with gaps moves the caret there), -1 when the code is complete. */
int gw_Netplay_CodeFirstEmpty(void) {
    int k;
    np_code_init();
    for (k = 0; k < 4; ++k) {
        if (rdv.letters[k] < 0) {
            rdv.slot = k;
            return k;
        }
    }
    return -1;
}
void gw_Netplay_CodeClear(void) {
    np_code_init();
    rdv.letters[0] = rdv.letters[1] = rdv.letters[2] = rdv.letters[3] = -1;
    rdv.slot = 0;
    np_code_sync();
}
/* A room code on the clipboard fills the field (the first 4 alphabet characters; case folded,
 * spaces and dashes dropped). */
static int np_code_from_text(const char *buf) {
    char c[5];
    int k = 0, j;
    for (j = 0; buf[j] != '\0' && k < 5; ++j) {
        char ch = buf[j] >= 'a' && buf[j] <= 'z' ? (char) (buf[j] - 32) : buf[j];
        if (ch != '\0' && strchr(NP_ALPHABET, ch) != NULL) c[k++] = ch;
        else if (ch != ' ' && ch != '-' && ch != '\r' && ch != '\n') return 0;
    }
    if (k != 4) return 0;
    for (j = 0; j < 4; ++j) rdv.letters[j] = (int) (strchr(NP_ALPHABET, c[j]) - NP_ALPHABET);
    rdv.slot = 3;
    np_code_sync();
    return 1;
}
/* Y / Ctrl+V. 1 when the clipboard held a room code. */
int gw_Netplay_CodePaste(void) {
    char buf[96];
    np_code_init();
    return np_clip_get(buf, sizeof buf) && np_code_from_text(buf);
}

/* Typing, while the code entry is open: letters/digits fill the active slot and move on,
 * Backspace clears and moves back, arrows move/step, Ctrl+V pastes. Keys only count while this
 * window has focus, and gd.key reads nothing meanwhile (the keys are text, not hotkeys).
 * Returns 0 nothing, 1 the code changed, 2 Enter (join), 3 Escape (back), 4 refused (a key
 * outside the alphabet, or an arrow at an end: the slot bumps). */
int gw_TextEntryUntil; /* GetTickCount deadline: keys are text until then (gd.key reads none) */
int gw_Netplay_CodeKeys(void) {
    static unsigned char was[256];
    HWND fg = GetForegroundWindow();
    DWORD pid = 0;
    int r = 0, vk;
    np_code_init();
    if (fg != NULL) GetWindowThreadProcessId(fg, &pid);
    if (pid != GetCurrentProcessId()) return 0;
    gw_TextEntryUntil = (int) GetTickCount() + 150;
    for (vk = 0x08; vk <= 0x5A; ++vk) {
        int down = (GetAsyncKeyState(vk) & 0x8000) != 0;
        int edge = down && !was[vk];
        was[vk] = (unsigned char) down;
        if (!edge || r >= 2) continue;
        if (vk == VK_RETURN) {
            r = 2;
        } else if (vk == VK_ESCAPE) {
            r = 3;
        } else if (vk == 'V' && (GetAsyncKeyState(VK_CONTROL) & 0x8000)) {
            r = gw_Netplay_CodePaste() ? 1 : 4;
        } else if (vk == VK_BACK) {
            if (rdv.letters[rdv.slot] < 0 && rdv.slot > 0) rdv.slot--;
            rdv.letters[rdv.slot] = -1;
            r = 1;
        } else if (vk == VK_LEFT || vk == VK_RIGHT) {
            r = gw_Netplay_CodeMove(vk == VK_RIGHT ? 1 : -1) ? 1 : 4;
        } else if (vk == VK_UP || vk == VK_DOWN) {
            gw_Netplay_CodeStep(vk == VK_UP ? 1 : -1);
            r = 1;
        } else if ((vk >= 'A' && vk <= 'Z') || (vk >= '0' && vk <= '9')) {
            const char *p = strchr(NP_ALPHABET, (char) vk);
            if (GetAsyncKeyState(VK_CONTROL) & 0x8000) continue;
            if (p != NULL) {
                rdv.letters[rdv.slot] = (int) (p - NP_ALPHABET);
                if (rdv.slot < 3) rdv.slot++;
                r = 1;
            } else {
                r = 4; /* I, O, 0 and 1 are not in room codes */
            }
        }
    }
    np_code_sync();
    return r;
}

/* ---- scripted: MELEE_NETPLAY connects at boot ---------------------------------------------------- */

static int np_env_int(const char *name, int dflt) {
    const char *v = getenv(name);
    return v != NULL && v[0] != '\0' ? atoi(v) : dflt;
}

/* gw_sl_load (gw_runtime.c), before MELEE_SCENE: with MELEE_NETPLAY set, connect now and return the
 * agreed match as the boot scene. NULL otherwise. */
const char *gw_Netplay_Scene(void) {
    const char *v;
    if (np.env_tried) return NULL;
    np.env_tried = 1;
    v = getenv("MELEE_NETPLAY");
    if (v == NULL || v[0] == '\0' || v[0] == '0') return NULL;
    np.port = NP_DEFAULT_PORT;
    if (strcmp(v, "random") == 0) {
        /* random matchmaking through the server, straight into the match (no lobby) */
        if (gw_Netplay_RandomBegin(np_env_int("MELEE_NETPLAY_CHAR", 2), np_env_int("MELEE_NETPLAY_COLOR", 0),
                                   np_env_int("MELEE_NETPLAY_STOCKS", 4), np_env_int("MELEE_NETPLAY_MINUTES", 8),
                                   np_env_int("MELEE_NETPLAY_DELAY", 2)) != 0) {
            return NULL;
        }
        np.use_lobby = 0;
        np.stage_ext = np_env_int("MELEE_NETPLAY_STAGE", 31);
        goto wait;
    }
    if (strncmp(v, "host", 4) == 0) {
        np.host = 1;
        if (v[4] == ':') np.port = (uint16_t) atoi(v + 5);
    } else if (strncmp(v, "join:", 5) == 0) {
        np.host = 0;
        snprintf(np.peer_code, sizeof np.peer_code, "%s", v + 5);
    } else {
        gw_log("netplay: MELEE_NETPLAY=\"%s\" not understood (host[:port], join:<ip>[:port] or random)", v);
        return NULL;
    }
    np.ck = np_env_int("MELEE_NETPLAY_CHAR", np.host ? 2 : 9); /* Fox / Marth */
    np.color = np_env_int("MELEE_NETPLAY_COLOR", 0);
    np.stage_ext = np_env_int("MELEE_NETPLAY_STAGE", 31);     /* Battlefield */
    np.stocks = np_env_int("MELEE_NETPLAY_STOCKS", 4);
    np.minutes = np_env_int("MELEE_NETPLAY_MINUTES", 8);
    np.delay = np_env_int("MELEE_NETPLAY_DELAY", 2);
    np.use_lobby = 0;
    if (np_begin(0) != 0) return NULL;
wait:
    {
        DWORD t0 = GetTickCount();
        while (np_poll() == NP_WORKING && GetTickCount() - t0 < 600000u) {
            MSG m;
            while (PeekMessageA(&m, NULL, 0, 0, PM_REMOVE)) {
                TranslateMessage(&m);
                DispatchMessageA(&m);
            }
            Sleep(2);
        }
    }
    if (np.phase != NP_CONNECTED) {
        gw_log("netplay: not connected - %s", np.status);
        return NULL;
    }
    return np.scene;
}

/* ---- the match ------------------------------------------------------------------------------------ */

static BOOL CALLBACK np_title_cb(HWND w, LPARAM title) {
    if (IsWindowVisible(w)) SetWindowTextA(w, (const char *) title);
    return TRUE;
}
static void np_set_title(const char *t) {
    EnumThreadWindows(GetCurrentThreadId(), np_title_cb, (LPARAM) t);
}

static void np_connected_title(char *title, size_t cap) {
    size_t n;
    snprintf(title, cap, "Melee netplay - %s (P%d) - input delay %d, ping %u ms",
             np.host ? "HOST" : "GUEST", gw_Netplay_LocalPort() + 1, np.delay,
             np.net != NULL ? gw_net_rtt_ms(np.net) : 0);
    if (sim.on) {
        n = strlen(title);
        snprintf(title + n, cap - n, " | SIMULATED: lag %d, jitter %d, loss %d%%, dup %d%%%s%s",
                 sim.lag, sim.jitter, sim.loss, sim.dup, sim.spike_every > 0 ? ", spikes" : "",
                 sim.file != NULL ? " (live file)" : "");
    }
    sim.changed = 0;
    np_set_title(title);
}

/* gw_Replay_ApplyMatch, live mode, just before frame -123: this side has loaded the match. Release
 * the hold and wait for the agreed start. Returns the seed. */
uint32_t gw_Netplay_Handshake(uint8_t *d, int len, int keep_off, int keep_len) {
    DWORD t0 = GetTickCount();
    (void) d; (void) len; (void) keep_off; (void) keep_len;
    if (np.net == NULL) {
        return np.seed;
    }
    gw_net_release(np.net);
    np_set_title("Melee netplay - waiting for the other player to load...");
    while (!np.started && !np.dead && GetTickCount() - t0 < GW_NET_HOLD_TIMEOUT_MS) {
        MSG m;
        gw_net_poll(np.net, NP_FIRST_FRAME);
        np_rdv_service();
        while (PeekMessageA(&m, NULL, 0, 0, PM_REMOVE)) {
            TranslateMessage(&m);
            DispatchMessageA(&m);
        }
        Sleep(1);
    }
    if (!np.started) {
        gw_log("netplay: the match never started (%s)", gw_net_last_reason(np.net));
        np_set_title("Melee netplay - NOT CONNECTED");
        return np.seed;
    }
    np.phase = NP_RUNNING;
    {
        char title[200];
        np_connected_title(title, sizeof title);
        gw_log("netplay: %s", title);
    }
    return np.seed;
}

/* The match scene is over (gw_RB_SceneBegin saw a non-VS scene after a live match): close the
 * session and put everything back for offline play. */
void gw_Netplay_MatchOver(void) {
    uint16_t port = rdv.on ? gw_net_udp_local_port(&rdv.inner) : 0;
    if (!np.enabled) return;
    gw_log("netplay: match over - closing the session%s", rdv.on ? ", keeping the room" : "");
    np.rematch = rdv.on; /* PERSISTENT ROOMS: back to ONLINE PLAY, same room, reconnect */
    np_keep_room = rdv.on;
    np_close();
    np_keep_room = 0;
    if (np.rematch) np_ka_start(port); /* keep the room through the results screen */
    np.enabled = 0;
    np.phase = NP_IDLE;
    gw_Replay_ArmLive(0);
    gw_SceneLaunch_SetText(NULL);
    np_set_title("Melee");
}

/* Once per render tick during the match (gw_RB_Iterations). */
void gw_Netplay_Tick(void) {
    int w;
    if (np.net == NULL) {
        return;
    }
    gw_net_poll(np.net, gw_rb_current_frame());
    if (rdv.on && (np.ticks % 60) == 0) {
        np_rdv_service(); /* keepalives: the room and the router mapping stay open */
    }
    w = gw_net_recommend_wait(np.net);
    if (w > 0) {
        gw_rb_request_wait(w);
    }
    if ((sim.changed || (np.ticks % 300) == 0) && np.started) {
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

/* ---- tests: the lobby rules (headless; registered from gw_tests_core.c) ---------------------- */
#include "gw_test.h"

static int lbt_fail(const char *what) {
    gw_test_fail("%s (phase %d, turn P%d, left %d)", what, lb.phase, lb.turn + 1, lb.left);
    return 1;
}

static void lbt_start(uint32_t seed) {
    memset(&lb, 0, sizeof lb);
    lb.seed = seed;
    lb_default_stages();
    lb_new_set(2, 9);
    lb_reset_game();
}

/* Game 1: blind characters, then strikes over the five starters from the coin winner (1-2-2,
   capped to 1-2-1 so one remains), the survivor is picked, then Ready from both runs the
   countdown out. The counterpick sits game 1 out. */
static int test_lobby_game1(void) {
    int first, i, n = 0;
    lbt_start(1u << 7); /* the coin: P2 strikes first */
    first = lb.first;
    if (lb.phase != LB_CHAR_BLIND || first != 1) return lbt_fail("game 1 starts blind, coin from the seed");
    if (lb_apply_(0, "STRIKE", 0, 0)) return lbt_fail("a strike before the characters");
    if (!lb_apply_(0, "CHAR", 34, 1) || lb.phase != LB_CHAR_BLIND) return lbt_fail("P1 locks, still blind");
    if (lb_apply_(0, "CHAR", 35, 0) || lb.ck[0] != 34) return lbt_fail("a locked blind pick is final");
    if (!lb_apply_(1, "CHAR", 36, 2) || lb.phase != LB_STRIKE) return lbt_fail("both locked -> strike");
    if (lb.turn != first || lb.left != 1) return lbt_fail("the coin winner strikes 1");
    if (lb_apply_(1 - first, "STRIKE", 0, 0)) return lbt_fail("striking out of turn");
    if (lb.nstages != 6 || lb_count_starters() != 5 || !lb_is_starter(4) || lb_is_starter(5) ||
        lb.stage_ext[5] != 3)
        return lbt_fail("the list: five starters, then Pokemon Stadium");
    if (lb_apply_(first, "STRIKE", 5, 0)) return lbt_fail("striking the counterpick in game 1");
    if (gw_Netplay_LobbyStageOpen(5) || !gw_Netplay_LobbyStageOpen(0) || gw_Netplay_LobbyStageGroup(5) != 1)
        return lbt_fail("the counterpick is not open while striking");
    /* 1-2-1: first 1, other 2, first 1 */
    {
        static const int order[4][2] = { { 1, 0 }, { 0, 1 }, { 0, 2 }, { 1, 3 } };
        for (i = 0; i < 4; ++i) {
            int who = order[i][0] == 1 ? first : 1 - first;
            if (lb.turn != who) return lbt_fail("1-2-1 order");
            if (!lb_apply_(who, "STRIKE", order[i][1], 0)) return lbt_fail("a legal strike refused");
            if (i == 1 && lb_apply_(who, "STRIKE", order[i][1], 0)) return lbt_fail("striking a struck stage");
        }
    }
    if (lb.phase != LB_READY || lb.chosen != 4 || lb.stage[4] != LB_PICKED) return lbt_fail("the last starter is picked");
    if (lb.stage[5] != LB_FREE) return lbt_fail("the counterpick is untouched");
    for (i = 0; i < lb.nstages; ++i) n += lb.stage[i] == LB_STRUCK_P1 || lb.stage[i] == LB_STRUCK_P2;
    if (n != 4) return lbt_fail("four strikes");
    if (!lb_apply_(0, "READY", 1, 0) || lb.countdown != 0) return lbt_fail("one ready: no countdown");
    if (lb_apply_(0, "READY", 1, 0)) return lbt_fail("ready twice is no change");
    if (!lb_apply_(1, "READY", 1, 0) || lb.countdown != LB_COUNTDOWN) return lbt_fail("both ready: countdown");
    if (!lb_apply_(1, "READY", 0, 0) || lb.countdown != 0) return lbt_fail("un-ready stops the countdown");
    lb_apply_(1, "READY", 1, 0);
    if (lb_countdown_at(LB_COUNTDOWN - 1) || lb.countdown != 1) return lbt_fail("the countdown ends early");
    if (!lb_countdown_at(LB_COUNTDOWN)) return lbt_fail("the countdown ends after 3 s");
    return 0;
}

/* Game 2+: the winner bans 2, the loser picks, then the winner's character, then the loser's. */
static int test_lobby_game2(void) {
    lbt_start(0);
    lb.winner = 0;
    lb.score[0] = 1;
    lb.game = 2;
    lb_reset_game();
    if (lb.phase != LB_BAN || lb.turn != 0 || lb.left != 2) return lbt_fail("the winner bans 2");
    if (lb_apply_(1, "BAN", 0, 0)) return lbt_fail("the loser cannot ban");
    if (!lb_apply_(0, "BAN", 0, 0) || !lb_apply_(0, "BAN", 2, 0)) return lbt_fail("two bans");
    if (lb.phase != LB_PICK || lb.turn != 1) return lbt_fail("then the loser picks");
    if (lb_apply_(1, "PICK", 2, 0)) return lbt_fail("picking a banned stage");
    if (!gw_Netplay_LobbyStageOpen(5)) return lbt_fail("the counterpick is open from game 2");
    if (!lb_apply_(1, "PICK", 5, 0) || lb.chosen != 5) return lbt_fail("the counterpick");
    if (lb.phase != LB_CHAR_WINNER || lb.turn != 0) return lbt_fail("the winner picks a character first");
    if (lb_apply_(1, "CHAR", 35, 0)) return lbt_fail("the loser waits");
    if (!lb_apply_(0, "CHAR", 34, 0) || lb.phase != LB_CHAR_LOSER || lb.turn != 1) return lbt_fail("then the loser");
    if (!lb_apply_(1, "CHAR", 35, 0) || lb.phase != LB_READY) return lbt_fail("then ready");
    if (lb_apply_(0, "STRIKE", 1, 0) || lb_apply_(0, "CHAR", 1, 0)) return lbt_fail("nothing else in READY");
    return 0;
}

/* The state message round-trips (host -> guest), and a new set / a game nobody won restart at
   blind characters. */
static int test_lobby_wire(void) {
    char m[GW_NET_LOBBY_MAX];
    int stage[LB_MAX_STAGES], i;
    lbt_start(0);
    lb_apply_(0, "CHAR", 34, 1);
    lb_apply_(1, "CHAR", 59, 3);
    lb_apply_(lb.turn, "STRIKE", 2, 0);
    lb_encode(m, sizeof m);
    memcpy(stage, lb.stage, sizeof stage);
    {
        int phase = lb.phase, turn = lb.turn, left = lb.left, ck1 = lb.ck[1], col1 = lb.color[1];
        memset(&lb, 0, sizeof lb);
        lb.nstages = 6;
        if (lb_decode(m) < 20) return lbt_fail("the state message parses");
        if (lb.phase != phase || lb.turn != turn || lb.left != left || lb.ck[1] != ck1 ||
            lb.color[1] != col1)
            return lbt_fail("the state message round-trips");
        for (i = 0; i < 6; ++i) {
            if (lb.stage[i] != stage[i]) return lbt_fail("the stage states round-trip");
        }
    }
    lb.game = 3;
    lb.winner = -1;
    lb_reset_game();
    if (lb.phase != LB_CHAR_BLIND) return lbt_fail("a game nobody won restarts blind");
    return 0;
}

/* All stages: game 1 is ban-and-pick from the coin winner; the list travels as identities in
   chunks and comes back as the same external ids; the state names the list it goes with. */
static int test_lobby_all_stages(void) {
    char m[GW_NET_LOBBY_MAX];
    int host_ext[LB_MAX_STAGES], n = 20, i, first, len;
    lbt_start(1u << 7);
    lb.mode = 1;
    lb.nstages = n;
    for (i = 0; i < n; ++i) lb.stage_ext[i] = i + 1;
    lb_reset_game();
    first = lb.first;
    lb_apply_(0, "CHAR", 34, 0);
    lb_apply_(1, "CHAR", 38, 0);
    if (lb.phase != LB_BAN || lb.turn != first || lb.left != 2) return lbt_fail("all stages: the coin winner bans 2");
    if (lb_apply_(first, "STRIKE", 0, 0)) return lbt_fail("no striking in all-stages mode");
    if (!lb_apply_(first, "BAN", 3, 0) || !lb_apply_(first, "BAN", 17, 0)) return lbt_fail("two bans");
    if (lb.phase != LB_PICK || lb.turn != 1 - first) return lbt_fail("then the other picks");
    if (!lb_apply_(1 - first, "PICK", 12, 0) || lb.phase != LB_READY || lb.chosen != 12)
        return lbt_fail("the pick settles game 1");
    /* the list on the wire */
    lb.list_id = 7;
    memcpy(host_ext, lb.stage_ext, sizeof host_ext);
    {
        int start = 0, host_n = lb.nstages;
        char chunks[4][GW_NET_LOBBY_MAX];
        int nc = 0;
        while (start < host_n && nc < 4) start = lb_list_encode(start, chunks[nc++], GW_NET_LOBBY_MAX, &len);
        lb_encode(m, sizeof m);
        memset(lb.stage_ext, 0, sizeof lb.stage_ext);
        lb.rx_id = -1;
        for (i = 0; i < nc; ++i) {
            if (strlen(chunks[i]) >= GW_NET_LOBBY_MAX) return lbt_fail("a list chunk overflows a lobby message");
            lb_list_rx(chunks[i]);
        }
        for (i = 0; i < host_n; ++i) {
            if (lb.stage_ext[i] != host_ext[i]) return lbt_fail("the list round-trips as identities");
        }
        lb.nstages = 0;
        lb_decode(m);
        if (!lb.list_ok || lb.nstages != host_n || lb.mode != 1) return lbt_fail("the state takes the list it names");
        if (lb.stage[3] != LB_BANNED || lb.stage[17] != LB_BANNED || lb.stage[12] != LB_PICKED)
            return lbt_fail("sparse stage states round-trip");
    }
    return 0;
}

/* The groups: any list is ordered starters first; a list with fewer than two starters has none. */
static int test_lobby_groups(void) {
    int i;
    lbt_start(0);
    lb.nstages = 40;
    for (i = 0; i < 40; ++i) lb.stage_ext[i] = i + 1;
    lb_order_groups();
    if (lb.nstages != 40) return lbt_fail("ordering keeps every stage");
    for (i = 0; i < 5; ++i) {
        if (lb.stage_ext[i] != lb_starter_ext[i] || !lb_is_starter(i)) return lbt_fail("the starters come first, in order");
    }
    for (i = 5; i < 40; ++i) {
        if (lb_is_starter(i) || gw_Netplay_LobbyStageGroup(i) != 1) return lbt_fail("the rest are counterpicks");
        if (i > 5 && lb.stage_ext[i] < lb.stage_ext[i - 1]) return lbt_fail("the counterpicks keep their order");
    }
    /* one starter only: no groups, strike over everything */
    lbt_start(0);
    lb.nstages = 3;
    lb.stage_ext[0] = 3;
    lb.stage_ext[1] = 5;
    lb.stage_ext[2] = 31;
    lb_reset_game();
    lb_apply_(0, "CHAR", 2, 0);
    lb_apply_(1, "CHAR", 9, 0);
    for (i = 0; i < 3; ++i) {
        if (!lb_is_starter(i)) return lbt_fail("no groups with one starter");
    }
    if (!lb_apply_(lb.turn, "STRIKE", 0, 0)) return lbt_fail("strikes over the whole list");
    return 0;
}

void gw_netplay_tests_register(void) {
    gw_test_register("netplay_lobby_groups", test_lobby_groups);
    gw_test_register("netplay_lobby_all_stages", test_lobby_all_stages);
    gw_test_register("netplay_lobby_game1", test_lobby_game1);
    gw_test_register("netplay_lobby_game2", test_lobby_game2);
    gw_test_register("netplay_lobby_wire", test_lobby_wire);
}
