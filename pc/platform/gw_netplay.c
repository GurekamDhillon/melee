/* gw_netplay.c - online play: the game-side adapter between the rollback session (gw_rollback.c)
 * and the transport (gw_net.c), plus what it takes to reach a friend on another network.
 *
 * HOW A MATCH COMES TOGETHER
 *   1. Set up: the ONLINE PLAY screen (gmfrontend.c) calls Netplay_Begin with the player's choices
 *      (host or join, character, costume; the host also stage and rules). Scripted runs set the same
 *      through MELEE_NETPLAY* instead, and connect at boot (gw_Netplay_Scene).
 *   2. Connect: the host opens UDP port 51500, finds its public address (STUN) and asks the router to
 *      forward the port (UPnP); its "code" is that address, copied to the clipboard. The guest pastes
 *      it and dials. If neither router lets the other in, each side pastes the other's code and both
 *      punch a hole (the guest's HELLOs and the host's punch packets open both NATs).
 *   3. Agree: the guest's character travels in its HELLO; the host folds it into the match (a scene
 *      string, gmscenelaunch.h grammar, with the rules) and sends it back in the ACCEPT with a seed.
 *   4. Load: both seed VS mode from that scene and load the match. The transport holds the start
 *      (hold_start) until both have loaded, then agrees a start time.
 *   5. Play: at the point a replay would restore its match struct (gw_Replay_ApplyMatch) each side
 *      releases the hold and waits for the start; from there gw_Netplay_Tick runs every render tick
 *      inside gw_RB_Iterations - remote inputs in, ours out, checksums, time sync.
 *   6. When the match scene ends the session closes; the next match starts again from step 1 (the
 *      menu remembers everything, so a rematch is Host / Connect again).
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
    int rematch;          /* a room-code match just ended: the menu reconnects to the same room */
    int use_lobby;        /* the menu path: connected players meet in the pick/ban lobby first */
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
    snprintf(out, cap,
             "mode=vs;at=match;p1=ck:%d/c%d/hu;p2=ck:%d/c%d/hu;stage=ext:%d;match=stock;stocks=%d;"
             "minutes=%d;items=off;pause=0",
             host_ck, host_c, guest_ck, guest_c, np.stage_ext, np.stocks, np.minutes);
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
    }
}

static void np_cb_desync(void *user, int32_t frame, uint32_t local, uint32_t remote) {
    (void) user;
    np.desync_frame = frame;
    gw_log("netplay: DESYNC at frame %d - local checksum %08X, peer %08X", frame, local, remote);
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
    if (sscanf(s, "ck:%d/c%d", &gck, &gc) < 1 || gck < 0 || gck > 0x7F || gck == 0x21) {
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
void gw_Netplay_MenuSetLetter(int i, int v);

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
static int np_rdv_config(void) {
    char buf[160] = { 0 };
    const char *v = getenv("MELEE_NETPLAY_SERVER");
    struct addrinfo hints, *res = NULL;
    char host[128], port[16];
    char *colon;
    if (v != NULL && v[0] != '\0') {
        snprintf(buf, sizeof buf, "%s", v);
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

static void np_rdv_handle(const char *msg) {
    char w0[16] = { 0 }, w1[64] = { 0 }, w2[64] = { 0 };
    sscanf(msg, "%15s %63s %63s", w0, w1, w2);
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
        np_status("Room %s - copied. Send it to your friend", rdv.code);
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
        snprintf(rdv.err, sizeof rdv.err, "%s", r != NULL ? r + 1 : "server error");
        np_status("Server: %s", rdv.err);
    } else if (strcmp(w0, "BYE") == 0) {
        np_status("Your friend left the room");
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
static void np_rdv_close(void *ctx) {
    (void) ctx;
    if ((rdv.have_code || rdv.have_peer) && !np_keep_room) np_rdv_ctl("BYE");
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
 * The rules (a common modern singles ruleset; see LB_* below):
 *   Game 1: characters double-blind (each picks on the real CSS; revealed when both are locked),
 *           then stage striking over the six legal stages - a coin flip picks who strikes first,
 *           strikes go 1-2-2 until one stage remains.
 *   Game 2+: the previous game's winner bans 2 stages, the loser picks one of the rest; then the
 *           winner picks a character, then the loser (counterpick, not blind).
 *   Then both press Ready; a short countdown; the match.
 * Players: 0 = host (P1), 1 = guest (P2). */
static void np_close(void);
static void np_copy(char *out, int cap, const char *s);
enum { LB_OFF, LB_CHAR_BLIND, LB_STRIKE, LB_BAN, LB_PICK, LB_CHAR_WINNER, LB_CHAR_LOSER, LB_READY, LB_GO };
#define LB_NSTAGES 6
static const int lb_stage_ext[LB_NSTAGES] = { 31, 28, 32, 2, 3, 8 };
static const char *const lb_stage_name[LB_NSTAGES] = { "Battlefield", "Dream Land", "Final Destination",
                                                       "Fountain of Dreams", "Pokemon Stadium",
                                                       "Yoshi's Story" };
enum { LB_FREE = 0, LB_STRUCK_P1 = 1, LB_STRUCK_P2 = 2, LB_BANNED = 3, LB_PICKED = 4 };

static struct {
    int phase, game, winner, score[2];
    int turn;            /* whose action it is (strike/ban/pick/char counterpick) */
    int left;            /* actions left in this turn */
    int step;            /* game 1 striking: which entry of the 1-2-2 order */
    int first;           /* game 1: who strikes first (the coin flip) */
    int stage[LB_NSTAGES];
    int chosen;          /* index of the stage the game is on, -1 none yet */
    int ck[2], color[2], locked[2], ready[2];
    int countdown;       /* frames, once both are ready */
    uint32_t seq;        /* state version, for the menu to notice changes */
} lb;

static const int lb_strike_counts[3] = { 1, 2, 2 }; /* 1-2-2 */

static void lb_reset_game(void) {
    int i;
    for (i = 0; i < LB_NSTAGES; ++i) lb.stage[i] = LB_FREE;
    lb.chosen = -1;
    lb.locked[0] = lb.locked[1] = 0;
    lb.ready[0] = lb.ready[1] = 0;
    lb.countdown = 0;
    lb.step = 0;
    if (lb.game <= 1 || lb.winner < 0) {
        lb.phase = LB_CHAR_BLIND;
        lb.first = (int) ((np.seed >> 7) & 1u); /* the coin flip: from the host's seed, same for both */
        lb.turn = lb.first;
        lb.left = lb_strike_counts[0];
    } else {
        lb.phase = LB_BAN;                      /* winner bans 2, loser picks */
        lb.turn = lb.winner;
        lb.left = 2;
    }
    lb.seq++;
}

static void lb_send(const char *msg) {
    if (np.net != NULL) gw_net_lobby_send(np.net, msg, (int) strlen(msg));
}

/* Host: the whole state for the guest. */
static void lb_broadcast(void) {
    char m[GW_NET_LOBBY_MAX];
    snprintf(m, sizeof m, "S %d %d %d %d %d %d %d %d %d %d%d%d%d%d%d %d %d %d %d %d %d %d %d %d %d",
             lb.phase, lb.game, lb.winner, lb.score[0], lb.score[1], lb.turn, lb.left, lb.step, lb.first,
             lb.stage[0], lb.stage[1], lb.stage[2], lb.stage[3], lb.stage[4], lb.stage[5], lb.chosen,
             lb.ck[0], lb.color[0], lb.ck[1], lb.color[1], lb.locked[0], lb.locked[1], lb.ready[0],
             lb.ready[1], lb.countdown);
    lb_send(m);
}

static int lb_stages_free(void) {
    int i, k = 0;
    for (i = 0; i < LB_NSTAGES; ++i) k += lb.stage[i] == LB_FREE;
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

/* Host: apply one player's action. Returns 1 if the state changed. */
static int lb_apply(int who, const char *act, int a, int b) {
    if (strcmp(act, "CHAR") == 0) {
        if (lb.phase == LB_CHAR_BLIND && !lb.locked[who]) {
            lb.ck[who] = a;
            lb.color[who] = b;
            lb.locked[who] = 1;
            if (lb.locked[0] && lb.locked[1]) lb.phase = LB_STRIKE;
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
    if (a < 0 || a >= LB_NSTAGES) {
        if (strcmp(act, "READY") != 0) return 0;
    }
    if (strcmp(act, "STRIKE") == 0 && lb.phase == LB_STRIKE && lb.turn == who && lb.stage[a] == LB_FREE) {
        lb.stage[a] = who == 0 ? LB_STRUCK_P1 : LB_STRUCK_P2;
        if (--lb.left <= 0 && lb_stages_free() > 1) {
            lb.step++;
            lb.turn = 1 - lb.turn;
            lb.left = lb_strike_counts[lb.step < 3 ? lb.step : 2];
        }
        if (lb_stages_free() == 1) {
            int i;
            for (i = 0; i < LB_NSTAGES; ++i) {
                if (lb.stage[i] == LB_FREE) {
                    lb.stage[i] = LB_PICKED;
                    lb.chosen = i;
                }
            }
            lb_after_stage();
        }
        return 1;
    }
    if (strcmp(act, "BAN") == 0 && lb.phase == LB_BAN && lb.turn == who && lb.stage[a] == LB_FREE) {
        lb.stage[a] = LB_BANNED;
        if (--lb.left <= 0) {
            lb.phase = LB_PICK;
            lb.turn = 1 - who;
            lb.left = 1;
        }
        return 1;
    }
    if (strcmp(act, "PICK") == 0 && lb.phase == LB_PICK && lb.turn == who && lb.stage[a] == LB_FREE) {
        lb.stage[a] = LB_PICKED;
        lb.chosen = a;
        lb_after_stage();
        return 1;
    }
    if (strcmp(act, "READY") == 0 && lb.phase == LB_READY) {
        lb.ready[who] = a != 0;
        if (lb.ready[0] && lb.ready[1]) lb.countdown = 180; /* 3-2-1 */
        else lb.countdown = 0;
        return 1;
    }
    return 0;
}

/* The match, once the countdown is over (host). */
static void lb_go(void) {
    char m[GW_NET_LOBBY_MAX];
    np.stage_ext = lb_stage_ext[lb.chosen >= 0 ? lb.chosen : 0];
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
    if (np.host && m[0] == 'A') {
        char act[16] = { 0 };
        int a = 0, b = 0;
        if (sscanf(m + 2, "%15s %d %d", act, &a, &b) >= 1 && lb_apply(1, act, a, b)) {
            lb.seq++;
            lb_broadcast();
        }
    } else if (!np.host && m[0] == 'S') {
        char st[8] = { 0 };
        int k = sscanf(m + 2, "%d %d %d %d %d %d %d %d %d %7s %d %d %d %d %d %d %d %d %d %d", &lb.phase,
                       &lb.game, &lb.winner, &lb.score[0], &lb.score[1], &lb.turn, &lb.left, &lb.step,
                       &lb.first, st, &lb.chosen, &lb.ck[0], &lb.color[0], &lb.ck[1], &lb.color[1],
                       &lb.locked[0], &lb.locked[1], &lb.ready[0], &lb.ready[1], &lb.countdown);
        if (k >= 10) {
            int i;
            for (i = 0; i < LB_NSTAGES && st[i] != '\0'; ++i) lb.stage[i] = st[i] - '0';
        }
        lb.seq++;
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

/* Every frame while in the lobby - also while a player is away on the CSS (gmscene.c calls
 * Netplay_Background from every scene): keep the connection alive, run the countdown, and hand
 * the match to the menu once it is agreed. */
static void np_lobby_tick(void) {
    if (np.phase != NP_LOBBY || np.net == NULL) return;
    gw_net_poll(np.net, NP_FIRST_FRAME);
    np_rdv_service();
    if (np.dead) {
        np.phase = NP_FAILED;
        np_close();
        return;
    }
    {
        /* TESTING: MELEE_LOBBY_AUTOPLAY=1 - act whenever it is this side's move (lock the current
           fighter, strike/ban/pick the first free stage, ready up), about half a second apart */
        static int ap = -1, wait;
        if (ap < 0) ap = getenv("MELEE_LOBBY_AUTOPLAY") != NULL;
        if (ap && ++wait >= 30) {
            int me = np.host ? 0 : 1, i;
            wait = 0;
            if (lb.phase == LB_CHAR_BLIND && !lb.locked[me]) {
                lb_action("CHAR", np.ck, np.color);
            } else if ((lb.phase == LB_CHAR_WINNER || lb.phase == LB_CHAR_LOSER) && lb.turn == me) {
                lb_action("CHAR", np.ck, np.color);
            } else if ((lb.phase == LB_STRIKE || lb.phase == LB_BAN || lb.phase == LB_PICK) &&
                       lb.turn == me) {
                for (i = 0; i < LB_NSTAGES && lb.stage[i] != LB_FREE; ++i) {
                }
                if (i < LB_NSTAGES) {
                    lb_action(lb.phase == LB_BAN ? "BAN" : lb.phase == LB_PICK ? "PICK" : "STRIKE", i, 0);
                }
            } else if (lb.phase == LB_READY && !lb.ready[me]) {
                lb_action("READY", 1, 0);
            }
        }
    }
    if (np.host && lb.phase == LB_READY && lb.countdown > 0) {
        if (!(lb.ready[0] && lb.ready[1])) {
            lb.countdown = 0;
        } else if (--lb.countdown == 0) {
            lb_go();
        } else if ((lb.countdown % 60) == 0) {
            lb.seq++;
            lb_broadcast(); /* the guest's countdown follows */
        }
    } else if (!np.host && lb.phase == LB_READY && lb.countdown > 0) {
        lb.countdown--;
    }
    if (lb.phase == LB_GO && np.phase == NP_LOBBY) {
        np.phase = NP_CONNECTED; /* the menu launches the match */
        np_status("Starting the match...");
        np_arm();
    }
}

void gw_Netplay_Background(void) { np_lobby_tick(); }

/* Entering the lobby: a fresh room starts at game 1; a rematch keeps the set going. */
static void np_lobby_enter(void) {
    np.phase = NP_LOBBY;
    if (lb.game <= 0) {
        lb.game = 1;
        lb.winner = -1;
        lb.score[0] = lb.score[1] = 0;
        lb.ck[0] = np.host ? np.ck : 2;
        lb.ck[1] = np.host ? 9 : np.ck;
        lb.color[0] = lb.color[1] = 0;
    }
    lb_reset_game();
    if (np.host) lb_broadcast();
    np_status("Connected - pick and ban!");
}

/* ---- starting and running a connection --------------------------------------------------------- */

static void np_close(void) {
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
static int np_begin(int bind_local) {
    gw_net_config cfg;
    gw_net_addr peer, pub, ppub, plan;
    int phas_lan = 0;
    uint32_t bind_ip = 0;
    np_close();
    memset(&cfg, 0, sizeof cfg);
    memset(&peer, 0, sizeof peer);
    np.started = np.dead = np.accepted = 0;
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

    cfg.exe_hash = np_exe_hash();
    {
        /* both must play the same disc: its header, apploader and the start of the DOL are
           enough to tell versions and modified discs apart (the transport refuses a mismatch) */
        extern const char *gw_iso_path(void);
        const char *iso = gw_iso_path();
        cfg.iso_hash = iso != NULL ? gw_net_hash_file(iso, 4u << 20) : 0;
    }
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
    if (np.use_lobby) cfg.cb.lobby_msg = np_cb_lobby;
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
        if (rdv.configured) {
            np.code[0] = '\0';
            np_status("Asking the server for a room...");
        } else {
            if (bind_ip != 0x7F000001u) np_upnp_start(np.port);
            np_clip_set(np.code);
            np_status("Hosting. Your code %s is copied - send it to your friend", np.code);
        }
    } else {
        snprintf(np.info, sizeof np.info, "ck:%d/c%d", np.ck, np.color);
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

/* One step of connecting (menu frames, or the boot wait). Returns the phase. */
static int np_poll(void) {
    if (np.phase == NP_LOBBY) {
        np_lobby_tick();
        return np.phase;
    }
    if (np.phase != NP_WORKING) {
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
                    snprintf(np.status, sizeof np.status, "Room %s - waiting for your friend", rdv.code);
                else if (rdv.have_peer)
                    snprintf(np.status, sizeof np.status, "Friend found - connecting%s", rdv.relay ? " (relay)" : "...");
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

/* ---- the in-game ONLINE PLAY menu (gmfrontend.c) ----------------------------------------------
 * Game code calls these without the gw_ prefix (gwtool adds it). Integers in, text out by copy. */
int gw_Netplay_MenuBegin(int host, int ck, int color, int stage_ext, int stocks, int minutes, int delay) {
    np.host = host != 0;
    np.ck = ck;
    np.color = color;
    np.stage_ext = stage_ext;
    np.stocks = stocks;
    np.minutes = minutes;
    np.delay = delay;
    np.port = NP_DEFAULT_PORT;
    np.use_lobby = 1;
    return np_begin(0);
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
    default: return 0;
    }
}
int gw_Netplay_LobbyStage(int i) { return i >= 0 && i < LB_NSTAGES ? lb.stage[i] : 0; }
int gw_Netplay_LobbyStageExt(int i) { return i >= 0 && i < LB_NSTAGES ? lb_stage_ext[i] : 0; }
void gw_Netplay_LobbyStageName(int i, char *out, int cap) {
    np_copy(out, cap, i >= 0 && i < LB_NSTAGES ? lb_stage_name[i] : "");
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
void gw_Netplay_LobbyCode(char *out, int cap) { np_copy(out, cap, rdv.on && rdv.code[0] ? rdv.code : np.code); }
void gw_Netplay_LobbyChar(int ck, int color) { lb_action("CHAR", ck, color); }
void gw_Netplay_LobbyStageAct(int i) {
    lb_action(lb.phase == LB_BAN ? "BAN" : lb.phase == LB_PICK ? "PICK" : "STRIKE", i, 0);
}
void gw_Netplay_LobbyReady(int on) { lb_action("READY", on != 0, 0); }
int gw_Netplay_LobbyActive(void) { return np.phase == NP_LOBBY; }

/* Paste a code from the clipboard: the guest's is the host's code; the host's is the guest's code
 * (for when neither router lets the other in: the host then punches toward it). 1 if one was read. */
int gw_Netplay_MenuPaste(int as_host) {
    char buf[96];
    gw_net_addr a, lan;
    int has_lan;
    if (!as_host && gw_Netplay_MenuHasServer() && np_clip_get(buf, sizeof buf)) {
        /* a room code: 4 letters from the alphabet, spaces and dashes ignored */
        char c[5];
        int k = 0, j;
        for (j = 0; buf[j] != '\0' && k < 5; ++j) {
            char ch = buf[j] >= 'a' && buf[j] <= 'z' ? (char) (buf[j] - 32) : buf[j];
            const char *p = ch != '\0' ? strchr(NP_ALPHABET, ch) : NULL;
            if (p != NULL) c[k++] = ch; else if (ch != ' ' && ch != '-') { k = 99; break; }
        }
        if (k == 4) {
            for (j = 0; j < 4; ++j) gw_Netplay_MenuSetLetter(j, (int) (strchr(NP_ALPHABET, c[j]) - NP_ALPHABET));
            np_status("Room code %s pasted - press Connect", np.peer_code);
            return 1;
        }
    }
    if (!np_clip_get(buf, sizeof buf) || np_parse_code(buf, &a, &lan, &has_lan) != 0) {
        np_status("The clipboard doesn't hold a code (like 1.2.3.4:51500)");
        return 0;
    }
    if (as_host) {
        np.punch = np_pick(&a, &lan, has_lan);
        np.punch_on = 1;
        np.next_punch = 0;
        np_status("Opening a path to %s - ask your friend to press Connect again", buf);
    } else {
        snprintf(np.peer_code, sizeof np.peer_code, "%s", buf);
        np_status("Host code %s pasted - press Connect", buf);
    }
    return 1;
}

int gw_Netplay_MenuPoll(void) { return np_poll(); }

/* Persistent rooms: after a room-code match both players come back to ONLINE PLAY, which
 * reconnects them to the same room (the host re-hosts at once, the guest follows a moment later). */
int gw_Netplay_RematchPending(void) { return np.rematch; }
/* The game's winner (0 host/P1, 1 guest/P2, -1 none), from the results screen: the next game's
 * rules (winner bans, loser picks) and the set score. */
void gw_Netplay_GameResult(int winner) {
    if (lb.game <= 0) return;
    lb.winner = winner;
    if (winner == 0 || winner == 1) lb.score[winner]++;
    lb.game++;
    gw_log("netplay: game over - winner P%d, set %d-%d, next game %d", winner + 1, lb.score[0], lb.score[1], lb.game);
}
void gw_Netplay_RematchTaken(void) { np.rematch = 0; }

void gw_Netplay_MenuCancel(void) {
    np_close();
    np.phase = NP_IDLE;
    np.enabled = 0;
    np_status("Cancelled");
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
/* Room codes (a server is configured): the menu edits the join code letter by letter. */
int gw_Netplay_MenuHasServer(void) {
    static int checked = -1;
    if (checked < 0) checked = np_rdv_config();
    return checked;
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
int gw_Netplay_MenuGetLetter(int i) { np_code_init(); return i >= 0 && i < 4 ? rdv.letters[i] : 0; }
void gw_Netplay_MenuSetLetter(int i, int v) {
    np_code_init();
    if (i < 0 || i > 3) return;
    rdv.letters[i] = ((v % 32) + 32) % 32;
    np_code_sync();
}

/* ---- the single code field: slots, a caret, typing, pasting ---- */
int gw_Netplay_CodeSlot(void) { np_code_init(); return rdv.slot; }
void gw_Netplay_CodeStep(int dir) {        /* left/right: the active slot's letter */
    np_code_init();
    rdv.letters[rdv.slot] = rdv.letters[rdv.slot] < 0 ? (dir > 0 ? 0 : 31)
                                                      : ((rdv.letters[rdv.slot] + dir) % 32 + 32) % 32;
    np_code_sync();
}
void gw_Netplay_CodeNext(void) {           /* A: on to the next slot (wraps) */
    np_code_init();
    rdv.slot = (rdv.slot + 1) % 4;
}
int gw_Netplay_CodeComplete(void) {
    int k;
    np_code_init();
    for (k = 0; k < 4; ++k) if (rdv.letters[k] < 0) return 0;
    return 1;
}
void gw_Netplay_CodeText(char *out, int cap) { /* "K Q [7] _" */
    char b[32];
    int k, o = 0;
    np_code_init();
    for (k = 0; k < 4; ++k) {
        char ch = rdv.letters[k] >= 0 ? NP_ALPHABET[rdv.letters[k]] : '_';
        if (k == rdv.slot) { b[o++] = '['; b[o++] = ch; b[o++] = ']'; }
        else b[o++] = ch;
        if (k < 3) b[o++] = ' ';
    }
    b[o] = '\0';
    np_copy(out, cap, b);
}
/* A room code on the clipboard fills the field (4 letters of the alphabet, spaces/dashes ignored). */
static int np_code_from_text(const char *buf) {
    char c[5];
    int k = 0, j;
    for (j = 0; buf[j] != '\0' && k < 5; ++j) {
        char ch = buf[j] >= 'a' && buf[j] <= 'z' ? (char) (buf[j] - 32) : buf[j];
        if (ch != '\0' && strchr(NP_ALPHABET, ch) != NULL) c[k++] = ch;
        else if (ch != ' ' && ch != '-') return 0;
    }
    if (k != 4) return 0;
    for (j = 0; j < 4; ++j) rdv.letters[j] = (int) (strchr(NP_ALPHABET, c[j]) - NP_ALPHABET);
    rdv.slot = 0;
    np_code_sync();
    return 1;
}
int gw_Netplay_CodeAutofill(void) {
    char buf[96];
    np_code_init();
    if (np_clip_get(buf, sizeof buf) && np_code_from_text(buf)) {
        np_status("Room code %s filled in from the clipboard - press Connect", np.peer_code);
        return 1;
    }
    return 0;
}

/* Typing, while the menu's cursor is on the code field: letters/digits fill the active slot and
 * move on, Backspace clears and moves back, Ctrl+V pastes. Keys only count while this window has
 * focus, and the keyboard-as-controller mapping stands down meanwhile (shim_pad.c). Returns 1
 * when the code changed. */
int gw_TextEntryUntil; /* GetTickCount deadline: shim_pad skips its keyboard mapping until then */
int gw_Netplay_CodeKeys(void) {
    static unsigned char was[256];
    HWND fg = GetForegroundWindow();
    DWORD pid = 0;
    int changed = 0, vk;
    np_code_init();
    if (fg != NULL) GetWindowThreadProcessId(fg, &pid);
    if (pid != GetCurrentProcessId()) return 0;
    gw_TextEntryUntil = (int) GetTickCount() + 150;
    for (vk = 0x08; vk <= 0x5A; ++vk) {
        int down = (GetAsyncKeyState(vk) & 0x8000) != 0;
        int edge = down && !was[vk];
        was[vk] = (unsigned char) down;
        if (!edge) continue;
        if (vk == VK_RETURN || vk == VK_ESCAPE) {
            return 2; /* done: the menu closes the code entry */
        }
        if (vk == 'V' && (GetAsyncKeyState(VK_CONTROL) & 0x8000)) {
            changed |= gw_Netplay_CodeAutofill();
        } else if (vk == VK_BACK) {
            if (rdv.letters[rdv.slot] < 0 && rdv.slot > 0) rdv.slot--;
            rdv.letters[rdv.slot] = -1;
            changed = 1;
        } else if ((vk >= 'A' && vk <= 'Z') || (vk >= '2' && vk <= '9')) {
            const char *p = strchr(NP_ALPHABET, (char) vk);
            if (p != NULL && !(GetAsyncKeyState(VK_CONTROL) & 0x8000)) {
                rdv.letters[rdv.slot] = (int) (p - NP_ALPHABET);
                if (rdv.slot < 3) rdv.slot++;
                changed = 1;
            }
        }
    }
    if (changed) np_code_sync();
    return changed;
}

void gw_Netplay_MenuCode(char *out, int cap) { np_copy(out, cap, np.code[0] ? np.code : "-"); }
void gw_Netplay_MenuPeer(char *out, int cap) { np_copy(out, cap, np.peer_code[0] ? np.peer_code : "(none - press A to paste)"); }

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
    if (strncmp(v, "host", 4) == 0) {
        np.host = 1;
        if (v[4] == ':') np.port = (uint16_t) atoi(v + 5);
    } else if (strncmp(v, "join:", 5) == 0) {
        np.host = 0;
        snprintf(np.peer_code, sizeof np.peer_code, "%s", v + 5);
    } else {
        gw_log("netplay: MELEE_NETPLAY=\"%s\" not understood (host[:port] or join:<ip>[:port])", v);
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
    if (!np.enabled) return;
    gw_log("netplay: match over - closing the session%s", rdv.on ? ", keeping the room" : "");
    np.rematch = rdv.on; /* PERSISTENT ROOMS: back to ONLINE PLAY, same room, reconnect */
    np_keep_room = rdv.on;
    np_close();
    np_keep_room = 0;
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
