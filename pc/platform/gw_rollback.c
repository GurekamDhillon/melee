/* gw_rollback.c - the rollback session: input delay, prediction, rollback and stalls.
 * See gw_rollback.h for the model and the interface a network layer calls. Design notes and the
 * acceptance test: _research/rollback-session.md.
 *
 * FLOW, once per render tick (gmscene.c gm_801A4D34):
 *   gw_RB_Iterations(count)   deliver whatever the (fake) network has for this tick, decide
 *                             whether to roll back and whether to stall, and return how many
 *                             logic iterations to run: k resimulated + 1 new (or fewer).
 *   gw_RB_IterStart()         at the top of each iteration: on a rollback's first one, load the
 *                             snapshot of the earliest wrong frame; then, for the frame about to
 *                             run, fix its inputs (confirmed, or predicted), save its snapshot,
 *                             and gate side effects (sound) while it is a resimulation.
 *   fighters read their inputs through gw_replay.c's accessors, which ask gw_RB_InputFor.
 *
 * The frame counter is gw_replay.c's (Slippi's numbering, -123 first); a rollback rewinds it with
 * the snapshot. The session is armed by MELEE_RB_FAKE for now (a replay whose remote ports'
 * inputs are delivered late); live pads and a real transport plug into the same rings.
 */
#include "gw.h"
#include "gw_rollback.h"

#include <limits.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <windows.h>

extern int gw_Replay_Active(void);
extern int gw_Replay_Frame(void);
extern int gw_Replay_LastFrame(void);
extern void gw_snap_save(int frame);
extern int gw_snap_load(int frame);
extern int gw_Snap_OpenSession(int k);
extern void gw_Snap_SessionResim(int on, int frame);
extern int gw_Snap_HasFrame(int frame);
extern uint32_t gw_Snap_Checksum(int frame);
extern void gw_Replay_TraceBeginIter(int iter);
extern void gw_Replay_TraceFlushUpTo(int iter);

#define RB_RING 64        /* frames of history per slot: prediction window + delay + latency */
#define RB_FIRST (-123)   /* Slippi's first frame */
#define RB_NONE INT_MAX

typedef struct {
    int frame; /* the frame this entry is for; INT_MIN = empty */
    GwRbInput in;
} RbEntry;

static struct {
    int tried, on;
    int log;              /* MELEE_RB_LOG=<n>: log the first n rollbacks and mismatches */
    int in_match;         /* the scene now running is a VS match (gw_RB_SceneBegin) */
    int delay, maxb;
    int fake_lat, fake_jit, fake_loss;
    unsigned seed;
    unsigned remote_mask; /* ports whose inputs arrive over the network */
    int opened;
    int started;          /* frame -123 has been simulated: the network clock runs */
    long tk;              /* the fake network clock: render ticks since the match began */
    int last;             /* the replay's last frame (fake) */

    int slot_present[GW_RB_SLOTS];
    int slot_remote[GW_RB_SLOTS];
    int conf_slot[GW_RB_SLOTS];       /* newest frame with every remote input <= it delivered */
    int fake_done_upto[GW_RB_SLOTS];  /* fake delivery: frames <= this have all been delivered */
    RbEntry used[GW_RB_SLOTS][RB_RING];
    RbEntry truth[GW_RB_SLOTS][RB_RING];
    unsigned char fake_done[GW_RB_SLOTS][RB_RING];
    int fake_done_frame[GW_RB_SLOTS][RB_RING];

    int first_wrong;                  /* earliest simulated frame whose confirmed input differs */
    /* the plan for this tick */
    struct {
        int rollback, first, n, i, k, new_frame;
        int active;
    } plan;

    /* statistics */
    int n_rollbacks, n_resim, depth_sum, depth_max, n_stall_ticks, n_desync, n_late_deliveries;
    int n_ticks, n_new;
    double ms_save, ms_load, ms_resim, ms_new;
    double rb_extra_ms_max[16], rb_extra_ms_sum[16];
    int rb_extra_n[16];
    double iter_t0;
    int iter_kind;                    /* 0 none, 1 new, 2 resim */
    double cur_extra_ms;              /* load + resimulated iterations of the current rollback */
    int cur_depth;
} rb;

static double rb_ms(void) {
    static LARGE_INTEGER f;
    LARGE_INTEGER c;
    if (f.QuadPart == 0) {
        QueryPerformanceFrequency(&f);
    }
    QueryPerformanceCounter(&c);
    return (double) c.QuadPart * 1000.0 / (double) f.QuadPart;
}

static void rb_init(void) {
    const char *v;
    if (rb.tried) {
        return;
    }
    rb.tried = 1;
    v = getenv("MELEE_RB_FAKE");
    if (v == NULL || v[0] == '\0' || !gw_Replay_Active()) {
        return;
    }
    rb.fake_lat = atoi(v);
    if (strchr(v, ',') != NULL) {
        const char *j = strchr(v, ',') + 1;
        rb.fake_jit = atoi(j);
        if (strchr(j, ',') != NULL) {
            rb.fake_loss = atoi(strchr(j, ',') + 1);
        }
    }
    rb.delay = (getenv("MELEE_RB_DELAY") != NULL) ? atoi(getenv("MELEE_RB_DELAY")) : 2;
    rb.maxb = (getenv("MELEE_RB_MAX") != NULL) ? atoi(getenv("MELEE_RB_MAX")) : 7;
    rb.seed = (getenv("MELEE_RB_SEED") != NULL) ? (unsigned) atoi(getenv("MELEE_RB_SEED")) : 12345u;
    rb.remote_mask = (getenv("MELEE_RB_REMOTE") != NULL)
                         ? (unsigned) strtoul(getenv("MELEE_RB_REMOTE"), NULL, 16) : 0x2u;
    if (rb.delay < 0) rb.delay = 0;
    if (rb.delay > 8) rb.delay = 8;
    if (rb.maxb < 1) rb.maxb = 1;
    if (rb.maxb > 12) rb.maxb = 12;
    rb.first_wrong = RB_NONE;
    rb.log = getenv("MELEE_RB_LOG") != NULL ? atoi(getenv("MELEE_RB_LOG")) : 0;
    rb.on = 1;
    gw_log("rb: session ON (fake network: latency %d, jitter %d, loss %d%%; input delay %d, max "
           "rollback %d, remote ports 0x%X)", rb.fake_lat, rb.fake_jit, rb.fake_loss, rb.delay,
           rb.maxb, rb.remote_mask);
}

int gw_RB_Enabled(void) {
    rb_init();
    return rb.on;
}

/* The session's inputs govern fighters once the match is armed (frame -124 = ApplyMatch ran). */
int gw_rb_active(void) {
    rb_init();
    return rb.on && rb.in_match && gw_Replay_Frame() >= RB_FIRST - 1;
}

/* gmscene.c, at the start of every scene: a session governs VS matches only. A new match starts a
 * fresh session state (the snapshot slots are kept). GS_VS is scene kind 2. */
void gw_RB_SceneBegin(int scene_kind) {
    rb_init();
    if (!rb.on) {
        return;
    }
    rb.in_match = scene_kind == 2;
    if (rb.in_match) {
        int s;
        rb.opened = 0;
        rb.started = 0;
        rb.tk = 0;
        rb.first_wrong = RB_NONE;
        memset(&rb.plan, 0, sizeof rb.plan);
        rb.iter_kind = 0;
        for (s = 0; s < GW_RB_SLOTS; ++s) {
            rb.conf_slot[s] = RB_FIRST - 1;
        }
    }
}

/* ---- small helpers ---------------------------------------------------------------------- */

static unsigned rb_hash(unsigned a, unsigned b, unsigned c) {
    unsigned h = a * 2654435761u ^ (b + 0x9E3779B9u) * 2246822519u ^ (c + 0x85EBCA6Bu) * 3266489917u;
    h ^= h >> 15;
    h *= 2246822519u;
    h ^= h >> 13;
    return h;
}

static RbEntry *rb_at(RbEntry ring[RB_RING], int frame, int create) {
    RbEntry *e = &ring[(unsigned) frame % RB_RING];
    if (e->frame != frame) {
        if (!create) {
            return NULL;
        }
        e->frame = frame;
        memset(&e->in, 0, sizeof e->in);
    }
    return e;
}

static int rb_same(const GwRbInput *a, const GwRbInput *b) {
    return a->present == b->present && memcmp(&a->lx, &b->lx, 5 * sizeof(float)) == 0 &&
           a->buttons == b->buttons && memcmp(a->raw, b->raw, 4) == 0;
}

static int rb_conf(void) {
    int s, m = INT_MAX;
    for (s = 0; s < GW_RB_SLOTS; ++s) {
        if (rb.slot_present[s] && rb.slot_remote[s] && rb.conf_slot[s] < m) {
            m = rb.conf_slot[s];
        }
    }
    return m;
}

/* ---- the network-facing interface ------------------------------------------------------- */

void gw_rb_submit_remote_input(int slot, int frame, const GwRbInput *in) {
    RbEntry *t, *u;
    if (!rb.on || slot < 0 || slot >= GW_RB_SLOTS || in == NULL) {
        return;
    }
    t = rb_at(rb.truth[slot], frame, 1);
    t->in = *in;
    t->in.confirmed = 1;
    /* the contiguous confirmed frame advances over everything now present */
    while (1) {
        RbEntry *n = rb_at(rb.truth[slot], rb.conf_slot[slot] + 1, 0);
        if (n == NULL) {
            break;
        }
        rb.conf_slot[slot]++;
    }
    u = rb_at(rb.used[slot], frame, 0);
    if (u != NULL) {
        /* the frame was already simulated with something else? */
        if (!rb_same(&u->in, &t->in)) {
            if (rb.log > 0) {
                rb.log--;
                gw_log("rb: input for slot %d frame %d arrived at tick %ld and differs from what was used "
                       "(used lx %.3f ly %.3f btn %08X pred=%d; true lx %.3f ly %.3f btn %08X)", slot,
                       frame, rb.tk, u->in.lx, u->in.ly, u->in.buttons, !u->in.confirmed, t->in.lx,
                       t->in.ly, t->in.buttons);
            }
            if (frame < rb.first_wrong) {
                rb.first_wrong = frame;
            }
        }
        u->in.confirmed = 1;
    }
}

void gw_rb_submit_local_input(int slot, int frame, const GwRbInput *in) {
    RbEntry *t;
    if (!rb.on || slot < 0 || slot >= GW_RB_SLOTS || in == NULL) {
        return;
    }
    t = rb_at(rb.truth[slot], frame, 1);
    t->in = *in;
    t->in.confirmed = 1;
}

int gw_rb_local_input_for_send(int slot, int frame, GwRbInput *out) {
    RbEntry *t;
    if (!rb.on || slot < 0 || slot >= GW_RB_SLOTS) {
        return 0;
    }
    t = rb_at(rb.truth[slot], frame, 0);
    if (t == NULL || out == NULL) {
        return 0;
    }
    *out = t->in;
    return 1;
}

int gw_rb_confirmed_frame(void) {
    int c = rb_conf();
    return c == INT_MAX ? gw_Replay_Frame() : c;
}

int gw_rb_current_frame(void) {
    return gw_Replay_Frame() + 1;
}

int gw_rb_frame_advantage(void) {
    return gw_Replay_Frame() - gw_rb_confirmed_frame();
}

uint32_t gw_rb_checksum(int frame) {
    if (frame > gw_rb_confirmed_frame()) {
        return 0; /* not final: the inputs it depends on are not all confirmed */
    }
    return gw_Snap_Checksum(frame);
}

int gw_rb_rollbacks(void) { return rb.n_rollbacks; }
int gw_rb_desyncs(void) { return rb.n_desync; }

const GwRbInput *gw_RB_InputFor(int port, int follower, int frame) {
    RbEntry *e;
    int slot = port * 2 + (follower != 0);
    if (!rb.on || port < 0 || port > 3) {
        return NULL;
    }
    e = rb_at(rb.used[slot], frame, 0);
    return e != NULL && e->in.present ? &e->in : NULL;
}

/* ---- the fake network -------------------------------------------------------------------- */

/* The tick at which frame F's input from the remote peer reaches us. The peer samples it at
 * frame F - D (input delay), so it is SENT at network tick F + 123 - D; it then takes `latency`
 * ticks +- jitter, and a lost first transmission costs a retransmission timeout. */
static long rb_arrival(int slot, int frame) {
    long sent = (long) frame - RB_FIRST - rb.delay;
    long at;
    int j = 0;
    if (sent < 0) {
        sent = 0; /* before the match began: the peers start with these already exchanged */
    }
    if (rb.fake_jit > 0) {
        j = (int) (rb_hash(rb.seed, (unsigned) frame, (unsigned) slot) % (unsigned) (2 * rb.fake_jit + 1))
            - rb.fake_jit;
    }
    at = sent + rb.fake_lat + j;
    if (rb.fake_loss > 0 &&
        (int) (rb_hash(rb.seed ^ 0xA5A5u, (unsigned) frame, (unsigned) slot) % 100u) < rb.fake_loss) {
        at += 2L * rb.fake_lat + 3; /* retransmit after a timeout */
    }
    return at < sent ? sent : at;
}

static void rb_fake_deliver(void) {
    int s;
    for (s = 0; s < GW_RB_SLOTS; ++s) {
        int f, top;
        if (!rb.slot_present[s] || !rb.slot_remote[s]) {
            continue;
        }
        /* frames the peer has sent by now: F + 123 - D <= tk */
        top = (int) (rb.tk + RB_FIRST + rb.delay);
        if (top > rb.last) {
            top = rb.last;
        }
        for (f = rb.conf_slot[s] + 1; f <= top; ++f) {
            GwRbInput in;
            RbEntry *t = rb_at(rb.truth[s], f, 0);
            if (t != NULL) {
                continue; /* already delivered */
            }
            if (rb_arrival(s, f) > rb.tk) {
                continue;
            }
            if (!gw_Replay_PeekInput(s, f, &in)) {
                memset(&in, 0, sizeof in);
                in.present = 0;
            }
            if (f < gw_Replay_Frame() + 1 - 0 && rb.started) {
                rb.n_late_deliveries++;
            }
            gw_rb_submit_remote_input(s, f, &in);
        }
    }
}

/* ---- the per-tick plan ------------------------------------------------------------------- */

int gw_RB_Iterations(int count) {
    int frame, n, conf;
    rb_init();
    if (!rb.on || !rb.in_match) {
        return count;
    }
    frame = gw_Replay_Frame();
    if (frame < RB_FIRST - 1) {
        return count; /* the match is not armed yet */
    }
    if (!rb.opened) {
        int s;
        rb.opened = 1;
        rb.last = gw_Replay_LastFrame();
        gw_Snap_OpenSession(rb.maxb + 1);
        for (s = 0; s < GW_RB_SLOTS; ++s) {
            GwRbInput probe;
            int p = s >> 1;
            rb.slot_present[s] = gw_Replay_PeekInput(s, RB_FIRST, &probe);
            rb.slot_remote[s] = (rb.remote_mask >> p) & 1u;
            rb.conf_slot[s] = RB_FIRST - 1;
            {
                int q;
                for (q = 0; q < RB_RING; ++q) {
                    rb.used[s][q].frame = INT_MIN;
                    rb.truth[s][q].frame = INT_MIN;
                }
            }
        }
        gw_log("rb: snapshots open (%d), slots present 0x%02X", rb.maxb + 3,
               (unsigned) ((rb.slot_present[0]) | (rb.slot_present[1] << 1) | (rb.slot_present[2] << 2) |
                           (rb.slot_present[3] << 3) | (rb.slot_present[4] << 4) |
                           (rb.slot_present[5] << 5) | (rb.slot_present[6] << 6) |
                           (rb.slot_present[7] << 7)));
    }
    /* close the previous tick's last iteration for the timing statistics */
    if (rb.iter_kind != 0) {
        double d = rb_ms() - rb.iter_t0;
        if (rb.iter_kind == 2) {
            rb.ms_resim += d;
            rb.cur_extra_ms += d;
        } else {
            rb.ms_new += d;
        }
        rb.iter_kind = 0;
    }
    if (rb.cur_depth > 0) {
        int d = rb.cur_depth > 15 ? 15 : rb.cur_depth;
        rb.rb_extra_n[d]++;
        rb.rb_extra_ms_sum[d] += rb.cur_extra_ms;
        if (rb.cur_extra_ms > rb.rb_extra_ms_max[d]) {
            rb.rb_extra_ms_max[d] = rb.cur_extra_ms;
        }
        rb.cur_depth = 0;
        rb.cur_extra_ms = 0;
    }

    if (!rb.started) {
        if (frame >= RB_FIRST) {
            rb.started = 1;
            rb.tk = 1;
        }
    } else {
        rb.tk++;
    }
    rb_fake_deliver();

    n = frame + 1; /* the next frame to simulate */
    conf = rb_conf();
    if (conf == INT_MAX) {
        conf = frame; /* no remote slots: everything is confirmed */
    }
    memset(&rb.plan, 0, sizeof rb.plan);
    rb.plan.n = n;
    rb.plan.new_frame = 1;
    if (rb.first_wrong != RB_NONE) {
        int f = rb.first_wrong;
        rb.first_wrong = RB_NONE;
        if (f <= frame) {
            if (!gw_Snap_HasFrame(f)) {
                rb.n_desync++;
                gw_log("rb: DESYNC - frame %d needs a correction but its snapshot is gone (now at "
                       "%d, confirmed %d)", f, n, conf);
            } else {
                rb.plan.rollback = 1;
                rb.plan.first = f;
                rb.plan.k = n - f;
                if (rb.log > 0) {
                    rb.log--;
                    gw_log("rb: rollback to frame %d (k=%d) at next frame %d, tick %ld, confirmed %d", f,
                           n - f, n, rb.tk, conf);
                }
            }
        }
    }
    /* stall: never simulate more than maxb frames beyond the newest fully-confirmed one; and at
       the replay's end wait for every frame to be confirmed before the "past the end" tick */
    if (frame >= RB_FIRST && (n - conf > rb.maxb || (n > rb.last && conf < rb.last))) {
        rb.plan.new_frame = 0;
        rb.n_stall_ticks++;
    }
    rb.plan.active = rb.plan.rollback || rb.plan.new_frame;
    if (rb.plan.rollback) {
        rb.n_rollbacks++;
        rb.n_resim += rb.plan.k;
        rb.depth_sum += rb.plan.k;
        if (rb.plan.k > rb.depth_max) {
            rb.depth_max = rb.plan.k;
        }
        rb.cur_depth = rb.plan.k;
    }
    /* frames whose inputs are all confirmed are final: write their trace rows */
    gw_Replay_TraceFlushUpTo(conf < frame ? conf : frame);
    rb.n_ticks++;
    if ((rb.n_ticks % 300) == 0) {
        gw_log("rb: tick %d frame %d confirmed %d | rollbacks %d (avg depth %.2f, max %d), resim "
               "frames %d, stalls %d, desyncs %d | ms: save %.2f/load %.2f per op, resim %.2f/iter, "
               "new %.2f/iter", rb.n_ticks, frame, conf, rb.n_rollbacks,
               rb.n_rollbacks ? (double) rb.depth_sum / rb.n_rollbacks : 0.0, rb.depth_max,
               rb.n_resim, rb.n_stall_ticks, rb.n_desync,
               rb.ms_save / (rb.n_new ? rb.n_new : 1), rb.n_rollbacks ? rb.ms_load / rb.n_rollbacks : 0.0,
               rb.n_resim ? rb.ms_resim / rb.n_resim : 0.0, rb.n_new ? rb.ms_new / rb.n_new : 0.0);
    }
    return (rb.plan.rollback ? rb.plan.k : 0) + (rb.plan.new_frame ? 1 : 0);
}

/* Fix the inputs frame `next` will use: the local ones (confirmed) and the remote ones
 * (confirmed when they have arrived, else the last confirmed input repeated). */
static void rb_prepare(int next) {
    int s;
    for (s = 0; s < GW_RB_SLOTS; ++s) {
        RbEntry *u, *t;
        GwRbInput in;
        if (!rb.slot_present[s]) {
            continue;
        }
        u = rb_at(rb.used[s], next, 1);
        t = rb_at(rb.truth[s], next, 0);
        if (t != NULL) {
            in = t->in;
        } else if (!rb.slot_remote[s]) {
            /* a local slot: the fake "player" (the replay) provides it, D frames ahead of use */
            if (gw_Replay_PeekInput(s, next, &in)) {
                RbEntry *nt = rb_at(rb.truth[s], next, 1);
                nt->in = in;
                nt->in.confirmed = 1;
            } else {
                memset(&in, 0, sizeof in);
            }
            in.confirmed = 1;
        } else {
            /* remote, not arrived: repeat the last confirmed input */
            RbEntry *c = rb.conf_slot[s] >= RB_FIRST ? rb_at(rb.truth[s], rb.conf_slot[s], 0) : NULL;
            if (c != NULL) {
                in = c->in;
            } else {
                memset(&in, 0, sizeof in);
                in.present = 1; /* neutral */
            }
            in.confirmed = 0;
            in.seed = 0;
        }
        u->in = in;
    }
}

void gw_RB_IterStart(void) {
    int next, resim;
    double now;
    if (!rb.on || !rb.in_match || !rb.opened || !rb.plan.active) {
        return;
    }
    now = rb_ms();
    if (rb.iter_kind != 0) {
        double d = now - rb.iter_t0;
        if (rb.iter_kind == 2) {
            rb.ms_resim += d;
            rb.cur_extra_ms += d;
        } else {
            rb.ms_new += d;
        }
    }
    if (rb.plan.i == 0 && rb.plan.rollback) {
        double t0 = rb_ms();
        gw_snap_load(rb.plan.first);
        rb.ms_load += rb_ms() - t0;
        rb.cur_extra_ms += rb_ms() - t0;
    }
    next = gw_Replay_Frame() + 1;
    resim = rb.plan.rollback && next < rb.plan.n;
    rb_prepare(next);
    gw_Replay_TraceBeginIter(next);
    if (!(rb.plan.i == 0 && rb.plan.rollback)) {
        double t0 = rb_ms();
        gw_snap_save(next);
        rb.ms_save += rb_ms() - t0;
        if (!resim) {
            rb.n_new++;
        }
    }
    gw_Snap_SessionResim(resim, next);
    rb.iter_kind = resim ? 2 : 1;
    rb.iter_t0 = rb_ms();
    rb.plan.i++;
}
