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
extern int gw_Snap_SfxFrameEnd(int frame);
extern uint32_t gw_RB_GameHash(void); /* fighter.c: the curated gameplay hash */
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
    int epoch;            /* increments at every scene */
    int wait_ticks;       /* time sync: new-frame iterations still to skip */
    /* the curated gameplay hash of the state at the start of each recent frame (overwritten by a
       resimulation) and the optional log of the final ones (MELEE_RB_HASHLOG) */
    struct { int frame; uint32_t h; } hring[RB_RING];
    int hlog_next;
    FILE *hlog;
    /* inputs stamped with a later epoch: a peer that entered the scene first */
    struct { int epoch, slot, frame; GwRbInput in; } early[256];
    int nearly;
    int src;              /* input source: 0 replay (processed), 1 pad generator, 2 live pads */
    unsigned gen_seed;
    /* live pads: one latch per port, fed by every PADRead */
    struct {
        int have;
        uint16_t bacc, bnew;                /* buttons: OR since the last sample / newest poll */
        int8_t sx, sy, cx, cy, err;         /* newest */
        uint8_t l, r, a, b;                 /* newest */
        uint8_t lpk, rpk, apk, bpk;         /* peaks since the last sample */
    } latch[4];
    /* live pads: remote ports' samples in flight on the fake network */
    struct {
        int slot, frame;
        long arrival;
        GwRbInput in;
    } pend[128];
    int npend;
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
    /* whole-tick WORK time (RB_Iterations entry to the end of the render pass; the vsync wait
       before the next tick is not in it), by rollback depth of the tick: 0 = no rollback */
    double tick_t0, tk_sum[16], tk_max[16];
    int tk_n[16], tk_over[16], tick_depth;
    /* where a tick's time goes: snapshot saves, the load, resimulated iterations (logic+render inside
       the loop), the new frame's iteration, and the rest (the render pass after the loop) */
    double c_save, c_load, c_resim, c_new;
    double d_save[16], d_load[16], d_resim[16], d_new[16];
    double iter_t0;
    int iter_kind;                    /* 0 none, 1 new, 2 resim */
    int iter_frame;                   /* the frame the iteration in progress simulates */
    int n_sfx_killed;                 /* abandoned-timeline sounds released */
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
    {
        const char *sv = getenv("MELEE_RB_INPUT");
        rb.src = (sv != NULL && (sv[0] == 'p' || sv[0] == 'P')) ? 1 : (sv != NULL && (sv[0] == 'l' || sv[0] == 'L')) ? 2 : 0;
        rb.gen_seed = getenv("MELEE_RB_PADSEED") != NULL ? (unsigned) atoi(getenv("MELEE_RB_PADSEED")) : 777u;
    }
    rb.log = getenv("MELEE_RB_LOG") != NULL ? atoi(getenv("MELEE_RB_LOG")) : 0;
    rb.on = 1;
    gw_log("rb: session ON (fake network: latency %d, jitter %d, loss %d%%; input delay %d, max "
           "rollback %d, remote ports 0x%X; inputs from %s)", rb.fake_lat, rb.fake_jit, rb.fake_loss,
           rb.delay, rb.maxb, rb.remote_mask, rb.src == 2 ? "live pads" : rb.src == 1 ? "the pad generator" : "the replay");
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
    rb.epoch++;
    rb.wait_ticks = 0;
    rb.in_match = scene_kind == 2;
    if (rb.in_match) {
        int s;
        rb.opened = 0;
        rb.started = 0;
        rb.tk = 0;
        rb.first_wrong = RB_NONE;
        memset(&rb.plan, 0, sizeof rb.plan);
        rb.iter_kind = 0;
        rb.npend = 0;
        memset(rb.latch, 0, sizeof rb.latch);
        for (s = 0; s < GW_RB_SLOTS; ++s) {
            rb.conf_slot[s] = RB_FIRST - 1;
        }
        {
            int q;
            for (q = 0; q < RB_RING; ++q) {
                rb.hring[q].frame = INT_MIN;
            }
        }
        rb.hlog_next = RB_FIRST;
        if (rb.hlog == NULL && getenv("MELEE_RB_HASHLOG") != NULL) {
            rb.hlog = fopen(getenv("MELEE_RB_HASHLOG"), "w");
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
    return a->present == b->present && a->is_raw == b->is_raw &&
           memcmp(&a->lx, &b->lx, 5 * sizeof(float)) == 0 && a->buttons == b->buttons &&
           memcmp(a->raw, b->raw, 4) == 0 && a->pad_l == b->pad_l && a->pad_r == b->pad_r &&
           a->pad_a == b->pad_a && a->pad_b == b->pad_b && a->pad_err == b->pad_err;
}

/* A neutral raw controller: present, sticks centred, nothing held. */
static void rb_neutral_raw(GwRbInput *o) {
    memset(o, 0, sizeof *o);
    o->present = 1;
    o->is_raw = 1;
    o->confirmed = 1;
}

/* THE PAD GENERATOR (MELEE_RB_INPUT=padgen): a raw controller state that is a pure function of
 * (seed, port, frame), so both ends of the fake network (and every resimulation) derive the same
 * input. Eight-frame blocks pick a move (run, jump, attack, special, shield, ...); sticks jitter
 * by a few raw units every frame so a prediction (the last input repeated) is usually a little
 * wrong and rollbacks are exercised constantly. */
static unsigned rb_hash(unsigned a, unsigned b, unsigned c);
static void rb_gen(int slot, int frame, GwRbInput *o) {
    int port = slot >> 1, sx = 0, sy = 0, cx = 0, cy = 0, l = 0, r = 0;
    unsigned btn = 0;
    unsigned blk = (unsigned) ((frame - RB_FIRST) >> 3);
    unsigned h = rb_hash(rb.gen_seed + (unsigned) port * 7919u, blk, 0x51u);
    unsigned j = rb_hash(rb.gen_seed + (unsigned) port * 104729u, (unsigned) frame, 0x77u);
    int mag = 40 + (int) ((h >> 8) & 0x2F);
    rb_neutral_raw(o);
    if (frame < RB_FIRST + 24) {
        return; /* intro */
    }
    switch (h & 7u) {
    case 0: break;                                             /* stand */
    case 1: sx = -mag - 30; break;                             /* run left */
    case 2: sx = mag + 30; break;                              /* run right */
    case 3: btn = 0x0400u; sx = ((h >> 4) & 1u) ? mag : -mag; break; /* jump */
    case 4: btn = 0x0100u; sx = ((h >> 4) & 1u) ? 30 : -30; break;   /* A (tilt/jab) */
    case 5: btn = 0x0200u; sy = ((h >> 4) & 1u) ? 70 : -70; sx = ((h >> 5) & 1u) ? 60 : -60; break; /* B */
    case 6: btn = 0x0020u; r = 200; sx = (int) ((h >> 4) & 0x3F) - 32; break;                  /* shield */
    default: cx = ((h >> 4) & 1u) ? 85 : -85; cy = (int) ((h >> 5) & 0x1F) - 16; break;           /* smash */
    }
    if (((h >> 12) & 3u) == 0u) {
        btn |= 0x0100u; /* sometimes press A on top */
    }
    sx += (int) (j & 7u) - 3;
    sy += (int) ((j >> 3) & 7u) - 3;
    o->buttons = btn;
    o->raw[0] = (int8_t) sx;
    o->raw[1] = (int8_t) sy;
    o->raw[2] = (int8_t) cx;
    o->raw[3] = (int8_t) cy;
    o->pad_l = (uint8_t) l;
    o->pad_r = (uint8_t) r;
}

/* Where a slot's TRUE input for `frame` comes from in the fake-network modes. */
static int rb_src_peek(int slot, int frame, GwRbInput *out) {
    if (rb.src == 1) {
        if ((slot & 1) || !rb.slot_present[slot]) {
            return 0;
        }
        rb_gen(slot, frame, out);
        return 1;
    }
    return gw_Replay_PeekInput(slot, frame, out);
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
    if (rb.opened && frame > gw_Replay_Frame() + RB_RING - 8) {
        /* further ahead than the ring can hold without evicting frames not yet simulated: refuse.
           The transport must keep its lookahead window (maxb + delay + slack) inside this and
           resend - a peer never legitimately runs this far ahead. */
        static int warned;
        if (warned++ < 4) {
            gw_log("rb: remote input for frame %d refused: %d frames ahead of the simulation (ring %d)",
                   frame, frame - gw_Replay_Frame(), RB_RING);
        }
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

/* A frame's curated hash is final when every input BEFORE it is confirmed (the state at its start
   depends on frames < it) and no correction reaching back to it is pending. */
static int rb_hash_final(int frame) {
    return frame - 1 <= gw_rb_confirmed_frame() && frame <= gw_Replay_Frame() &&
           !(rb.first_wrong != RB_NONE && rb.first_wrong < frame);
}

uint32_t gw_rb_checksum(int frame) {
    if (!rb.on || !rb_hash_final(frame) || rb.hring[(unsigned) frame % RB_RING].frame != frame) {
        return 0;
    }
    return rb.hring[(unsigned) frame % RB_RING].h;
}

uint32_t gw_rb_checksum_full(int frame) {
    if (frame > gw_rb_confirmed_frame()) {
        return 0;
    }
    return gw_Snap_Checksum(frame);
}

int gw_rb_epoch(void) { return rb.epoch; }

uint32_t gw_rb_checksum_e(int epoch, int frame) {
    return epoch == rb.epoch ? gw_rb_checksum(frame) : 0;
}

void gw_rb_submit_remote_input_e(int epoch, int slot, int frame, const GwRbInput *in) {
    if (epoch == rb.epoch) {
        gw_rb_submit_remote_input(slot, frame, in);
    } else if (epoch > rb.epoch && rb.nearly < (int) (sizeof rb.early / sizeof rb.early[0]) && in != NULL) {
        rb.early[rb.nearly].epoch = epoch;
        rb.early[rb.nearly].slot = slot;
        rb.early[rb.nearly].frame = frame;
        rb.early[rb.nearly].in = *in;
        rb.nearly++;
    } /* an earlier epoch: dropped */
}

void gw_rb_request_wait(int frames) {
    if (frames > 0) {
        rb.wait_ticks += frames;
    }
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
    return e != NULL && e->in.present && !e->in.is_raw ? &e->in : NULL;
}

const GwRbInput *gw_RB_InputAny(int port, int frame) {
    RbEntry *e;
    if (!rb.on || port < 0 || port > 3) {
        return NULL;
    }
    e = rb_at(rb.used[port * 2], frame, 0);
    return e != NULL && e->in.present ? &e->in : NULL;
}

/* ---- live pads and the pad pipeline ---------------------------------------------------------- */

void gw_RB_PadLatch(int port, int button, int sx, int sy, int cx, int cy, int l, int r, int a, int b,
                    int err) {
    rb_init();
    if (!rb.on || rb.src != 2 || port < 0 || port > 3) {
        return;
    }
    {
        typeof(rb.latch[0]) *L = &rb.latch[port];
        if (!L->have) {
            L->bacc = (uint16_t) button;
            L->lpk = (uint8_t) l;
            L->rpk = (uint8_t) r;
            L->apk = (uint8_t) a;
            L->bpk = (uint8_t) b;
        } else {
            L->bacc |= (uint16_t) button;
            if ((uint8_t) l > L->lpk) L->lpk = (uint8_t) l;
            if ((uint8_t) r > L->rpk) L->rpk = (uint8_t) r;
            if ((uint8_t) a > L->apk) L->apk = (uint8_t) a;
            if ((uint8_t) b > L->bpk) L->bpk = (uint8_t) b;
        }
        L->have = 1;
        L->bnew = (uint16_t) button;
        L->sx = (int8_t) sx;
        L->sy = (int8_t) sy;
        L->cx = (int8_t) cx;
        L->cy = (int8_t) cy;
        L->l = (uint8_t) l;
        L->r = (uint8_t) r;
        L->a = (uint8_t) a;
        L->b = (uint8_t) b;
        L->err = (int8_t) err;
    }
}

int gw_RB_PadGoverned(void) {
    return rb.on && rb.in_match && rb.opened && rb.src != 0 && gw_Replay_Frame() >= RB_FIRST - 1;
}

int gw_RB_PadField(int port, int which) {
    RbEntry *e;
    const GwRbInput *in;
    if (port < 0 || port > 3) {
        return 0;
    }
    e = rb_at(rb.used[port * 2], gw_Replay_Frame() + 1, 0);
    if (e == NULL || !e->in.present) {
        return 0; /* no such pad: neutral */
    }
    in = &e->in;
    switch (which) {
    case 0: return (int) (in->buttons & 0xFFFFu);
    case 1: return in->raw[0];
    case 2: return in->raw[1];
    case 3: return in->raw[2];
    case 4: return in->raw[3];
    case 5: return in->pad_l;
    case 6: return in->pad_r;
    case 7: return in->pad_a;
    case 8: return in->pad_b;
    default: return in->pad_err;
    }
}

/* Take this frame's live sample: for frame `next` + D. Local ports' samples are inputs the
 * moment they are taken; remote ports' travel on the fake network first. */
static void rb_live_sample(int next) {
    int s, ff = next + rb.delay;
    for (s = 0; s < GW_RB_SLOTS; s += 2) {
        int p = s >> 1, i, dup = 0;
        GwRbInput in;
        typeof(rb.latch[0]) *L = &rb.latch[p];
        if (!rb.slot_present[s] || rb_at(rb.truth[s], ff, 0) != NULL) {
            continue; /* not in the match, or this frame is already sampled (a held frame) */
        }
        for (i = 0; i < rb.npend; ++i) {
            if (rb.pend[i].slot == s && rb.pend[i].frame == ff) {
                dup = 1;
            }
        }
        if (dup) {
            continue;
        }
        rb_neutral_raw(&in);
        if (L->have) {
            in.buttons = L->bacc;
            in.raw[0] = L->sx;
            in.raw[1] = L->sy;
            in.raw[2] = L->cx;
            in.raw[3] = L->cy;
            in.pad_l = L->lpk;
            in.pad_r = L->rpk;
            in.pad_a = L->apk;
            in.pad_b = L->bpk;
            in.pad_err = L->err;
            /* the next sample starts from what is held now: a held button persists, a press that
               began and ended in between was reported once, in this one */
            L->bacc = L->bnew;
            L->lpk = L->l;
            L->rpk = L->r;
            L->apk = L->a;
            L->bpk = L->b;
        }
        if (!rb.slot_remote[s]) {
            rb_at(rb.truth[s], ff, 1)->in = in;
        } else if (rb.npend < (int) (sizeof rb.pend / sizeof rb.pend[0])) {
            int j = 0;
            long at;
            if (rb.fake_jit > 0) {
                j = (int) (rb_hash(rb.seed, (unsigned) ff, (unsigned) s) % (unsigned) (2 * rb.fake_jit + 1)) - rb.fake_jit;
            }
            at = rb.tk + rb.fake_lat + j;
            if (rb.fake_loss > 0 &&
                (int) (rb_hash(rb.seed ^ 0xA5A5u, (unsigned) ff, (unsigned) s) % 100u) < rb.fake_loss) {
                at += 2L * rb.fake_lat + 3;
            }
            if (at < rb.tk) {
                at = rb.tk;
            }
            rb.pend[rb.npend].slot = s;
            rb.pend[rb.npend].frame = ff;
            rb.pend[rb.npend].arrival = at;
            rb.pend[rb.npend].in = in;
            rb.npend++;
        }
    }
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
    if (rb.src == 2) {
        int i = 0;
        while (i < rb.npend) {
            if (rb.pend[i].arrival <= rb.tk) {
                GwRbInput in = rb.pend[i].in;
                int sl = rb.pend[i].slot, fr = rb.pend[i].frame;
                rb.pend[i] = rb.pend[--rb.npend];
                gw_rb_submit_remote_input(sl, fr, &in);
            } else {
                ++i;
            }
        }
        return;
    }
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
        /* A real peer stops sending when it is MAX frames ahead of what it has confirmed from us
           (it stalls): it cannot run arbitrarily far ahead of a slow receiver. Without this cap a
           local side that waits (time sync) or stalls gets remote inputs hundreds of frames early,
           and the per-slot input rings (RB_RING frames) evict them before they are simulated. */
        {
            int cap = gw_Replay_Frame() + 1 + rb.maxb + rb.delay;
            if (top > cap) {
                top = cap;
            }
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
            if (!rb_src_peek(s, f, &in)) {
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
            rb.slot_present[s] = gw_Replay_PeekInput(s, RB_FIRST, &probe) && (rb.src == 0 || !(s & 1));
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
        if (rb.src == 2 && rb.delay > 0) {
            /* live pads: the first D frames have no sample behind them (a sample taken at frame F
               is for F + D): both peers agree they are neutral */
            for (s = 0; s < GW_RB_SLOTS; ++s) {
                int f;
                if (!rb.slot_present[s]) {
                    continue;
                }
                for (f = RB_FIRST; f < RB_FIRST + rb.delay; ++f) {
                    RbEntry *e = rb_at(rb.truth[s], f, 1);
                    rb_neutral_raw(&e->in);
                }
                if (rb.slot_remote[s]) {
                    rb.conf_slot[s] = RB_FIRST + rb.delay - 1;
                }
            }
        }
        {
            int q = 0;
            while (q < rb.nearly) {
                if (rb.early[q].epoch == rb.epoch) {
                    GwRbInput e = rb.early[q].in;
                    int sl = rb.early[q].slot, fr = rb.early[q].frame;
                    rb.early[q] = rb.early[--rb.nearly];
                    gw_rb_submit_remote_input(sl, fr, &e);
                } else if (rb.early[q].epoch < rb.epoch) {
                    rb.early[q] = rb.early[--rb.nearly];
                } else {
                    ++q;
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

    if (rb.hlog != NULL) {
        int c = rb_conf(), f;
        if (c == INT_MAX || c > frame) {
            c = frame;
        }
        for (f = rb.hlog_next; f <= c + 1 && f <= frame; ++f) { /* frame = the last simulated: its hash exists */
            uint32_t h = rb.hring[(unsigned) f % RB_RING].frame == f ? rb.hring[(unsigned) f % RB_RING].h : 0;
            if (h != 0) {
                fprintf(rb.hlog, "%d,%08X\n", f, h);
            }
        }
        rb.hlog_next = f;
        fflush(rb.hlog);
    }
    /* Frames whose inputs were all confirmed BEFORE this tick's deliveries are final: any wrong one
       was already resimulated by the previous tick's iterations, so their trace rows are the
       post-rollback ones. (Flushing after planning would write the old timeline's rows of frames
       this tick is about to resimulate.) */
    {
        int c = rb_conf();
        if (c == INT_MAX || c > frame) {
            c = frame;
        }
        gw_Replay_TraceFlushUpTo(c);
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
    } else if (rb.wait_ticks > 0 && frame >= RB_FIRST) {
        rb.wait_ticks--; /* time sync: give a frame back (gw_rb_request_wait) */
        rb.plan.new_frame = 0;
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
    rb.n_ticks++;
    {
        /* MELEE_RB_WAITTEST=1: ask for two frames of waiting every 40 ticks, as a time-sync layer
           would - waiting is pure timing, so the confirmed trace must not change */
        static int waittest = -1;
        if (waittest < 0) {
            waittest = getenv("MELEE_RB_WAITTEST") != NULL && atoi(getenv("MELEE_RB_WAITTEST")) != 0;
        }
        if (waittest && (rb.n_ticks % 40) == 0) {
            gw_rb_request_wait(2);
        }
    }
    rb.tick_t0 = rb_ms();
    rb.tick_depth = rb.plan.rollback ? (rb.plan.k > 15 ? 15 : rb.plan.k) : 0;
    if ((rb.n_ticks % 1200) == 0) {
        int d;
        for (d = 0; d < 16; ++d) {
            if (rb.tk_n[d] != 0) {
                double n = rb.tk_n[d];
                gw_log("rb: tick work, depth %2d: %5d ticks, avg %6.2f ms (save %5.2f, load %5.2f, "
                       "resim iters %6.2f, new iter %6.2f, other %6.2f), max %6.2f, over 16.7 ms: %d (%.0f%%)",
                       d, rb.tk_n[d], rb.tk_sum[d] / n, rb.d_save[d] / n, rb.d_load[d] / n,
                       rb.d_resim[d] / n, rb.d_new[d] / n,
                       (rb.tk_sum[d] - rb.d_save[d] - rb.d_load[d] - rb.d_resim[d] - rb.d_new[d]) / n,
                       rb.tk_max[d], rb.tk_over[d], 100.0 * rb.tk_over[d] / n);
            }
        }
    }
    if ((rb.n_ticks % 300) == 0) {
        gw_log("rb: tick %d frame %d confirmed %d | rollbacks %d (avg depth %.2f, max %d), resim "
               "frames %d, stalls %d, desyncs %d, abandoned sounds released %d | ms: save %.2f/load %.2f per op, "
               "resim %.2f/iter, new %.2f/iter", rb.n_ticks, frame, conf, rb.n_rollbacks,
               rb.n_rollbacks ? (double) rb.depth_sum / rb.n_rollbacks : 0.0, rb.depth_max,
               rb.n_resim, rb.n_stall_ticks, rb.n_desync, rb.n_sfx_killed,
               rb.ms_save / (rb.n_new ? rb.n_new : 1), rb.n_rollbacks ? rb.ms_load / rb.n_rollbacks : 0.0,
               rb.n_resim ? rb.ms_resim / rb.n_resim : 0.0, rb.n_new ? rb.ms_new / rb.n_new : 0.0);
    }
    return (rb.plan.rollback ? rb.plan.k : 0) + (rb.plan.new_frame ? 1 : 0);
}

/* gmscene.c: after the render pass - the tick's work is done. */
void gw_RB_TickEnd(void) {
    double d;
    int k;
    if (!rb.on || !rb.in_match || !rb.opened || rb.tick_t0 == 0) {
        return;
    }
    d = rb_ms() - rb.tick_t0;
    rb.tick_t0 = 0;
    k = rb.tick_depth;
    if (rb.iter_kind == 2) {
        rb.n_sfx_killed += gw_Snap_SfxFrameEnd(rb.iter_frame);
    }
    if (rb.iter_kind != 0) { /* the tick's last iteration ends here, render included */
        double di = rb_ms() - rb.iter_t0;
        if (rb.iter_kind == 2) {
            rb.c_resim += di;
        } else {
            rb.c_new += di;
        }
    }
    rb.d_save[k] += rb.c_save;
    rb.d_load[k] += rb.c_load;
    rb.d_resim[k] += rb.c_resim;
    rb.d_new[k] += rb.c_new;
    rb.c_save = rb.c_load = rb.c_resim = rb.c_new = 0;
    rb.tk_n[k]++;
    rb.tk_sum[k] += d;
    if (d > rb.tk_max[k]) {
        rb.tk_max[k] = d;
    }
    if (d > 16.7) {
        rb.tk_over[k]++;
    }
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
            /* a local slot: the fake "player" (the replay or the generator) provides it; live pads
               were sampled D frames ago (rb_live_sample) and only a hole reaches here */
            if (rb.src != 2 && rb_src_peek(s, next, &in)) {
                RbEntry *nt = rb_at(rb.truth[s], next, 1);
                nt->in = in;
                nt->in.confirmed = 1;
            } else if (rb.src != 0) {
                rb_neutral_raw(&in);
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
                in.is_raw = rb.src != 0;
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
    if (rb.iter_kind == 2) {
        rb.n_sfx_killed += gw_Snap_SfxFrameEnd(rb.iter_frame);
    }
    if (rb.iter_kind != 0) {
        double d = now - rb.iter_t0;
        if (rb.iter_kind == 2) {
            rb.ms_resim += d;
            rb.c_resim += d;
            rb.cur_extra_ms += d;
        } else {
            rb.ms_new += d;
            rb.c_new += d;
        }
    }
    if (rb.plan.i == 0 && rb.plan.rollback) {
        double t0 = rb_ms();
        gw_snap_load(rb.plan.first);
        rb.ms_load += rb_ms() - t0;
        rb.c_load += rb_ms() - t0;
        rb.cur_extra_ms += rb_ms() - t0;
    }
    next = gw_Replay_Frame() + 1;
    resim = rb.plan.rollback && next < rb.plan.n;
    if (!resim && rb.src == 2) {
        rb_live_sample(next);
    }
    rb_prepare(next);
    gw_Replay_TraceBeginIter(next);
    rb.hring[(unsigned) next % RB_RING].frame = next;
    rb.hring[(unsigned) next % RB_RING].h = gw_RB_GameHash() | 1u;
    if (!(rb.plan.i == 0 && rb.plan.rollback)) {
        double t0 = rb_ms();
        gw_snap_save(next);
        rb.ms_save += rb_ms() - t0;
        rb.c_save += rb_ms() - t0;
        if (!resim) {
            rb.n_new++;
        }
    }
    gw_Snap_SessionResim(resim, next);
    rb.iter_kind = resim ? 2 : 1;
    rb.iter_frame = next;
    rb.iter_t0 = rb_ms();
    rb.plan.i++;
}
