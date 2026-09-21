/* gw_replay.c - Slippi (.slp) playback: MELEE_SLP=<file>.
 *
 * Plays a Slippi replay through the port's own game code and, optionally, records what the port
 * computes so it can be diffed against what the console computed (tools/replay/replay_compare.py).
 *
 *   MELEE_SLP=C:/path/game.slp               play it
 *   MELEE_STATE_TRACE=trace.csv              also write the port's per-frame post-frame state
 *   MELEE_SLP_RESYNC=1                       Slippi's resync mode: restore the RNG seed each frame
 *
 * WHAT A REPLAY CARRIES, AND WHAT IS DONE WITH IT. This mirrors Slippi's own playback codes
 * (project-slippi/slippi-ssbm-asm, Playback/Core), mapped onto the decomp:
 *
 *   Game Start (0x36) +0x5, 0x138 bytes: the StartMeleeData the console started the match with -
 *     rules plus six PlayerInitData. RestoreGameInfo.asm memcpy's it over the match's own at
 *     0x8016E748, i.e. inside fn_8016E730 (gmvs.c) - gw_Replay_ApplyMatch here. The rules block's
 *     callback pointers are console addresses and are NOT copied; the port's own stay.
 *   Game Start +0x13D: the RNG seed, written to the game's seed at the same point.
 *   Pre-Frame (0x37), per frame per character: the PROCESSED inputs - sticks, C-stick, trigger and
 *     buttons as the fighter holds them after deadzones and the L/R/Z folding. RestoreGameFrame.asm
 *     writes them into the fighter (fp+0x620..0x65C) at 0x8006B0DC, inside
 *     Fighter_Spaghetti_8006AD10 just before pressed/released are derived - the same point here.
 *     Recording and playback both use this point, so pad sampling, the pad queue and controller
 *     calibration are out of the loop entirely.
 *
 * FRAMES. Slippi numbers a match's frames from -123 (the first frame fighters run) and the timer
 * starts at 0. gw_Replay_ApplyMatch arms the counter, and gw_Replay_Tick advances it once per
 * logic frame that actually runs the fighters - the loading hold's frozen frames do not count.
 *
 * The boundary with game code is pull-only and scalar-only (see gmscene.c's overlay comment):
 * game TUs byte-swap their own memory accesses, so values cross by return value, never by a host
 * write through a game pointer. The one exception is the match struct, a byte image in console
 * byte order on both sides - exactly what memcpy is right for.
 */
#include "gw.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define GW_RP_GAME_INFO 0x138u
#define GW_RP_SLOTS 8 /* 4 ports x (leader, follower) */
#define GW_RP_FIRST_FRAME (-123)
#define GW_RP_UNARMED (-0x7FFFFFFF)

typedef struct GwRpInput {
    float lx, ly, cx, cy, trigger;
    uint32_t buttons;
    uint32_t seed;
    int8_t raw[4]; /* raw stick bytes: x, y, c-x, c-y (0 where the replay predates the field) */
    uint8_t present;
} GwRpInput;

static struct {
    int tried;
    int active;
    uint8_t game_info[GW_RP_GAME_INFO];
    uint32_t seed;
    uint8_t version[4];
    int first, last; /* frame range with pre-frame data */
    GwRpInput *in;   /* (last - first + 1) * GW_RP_SLOTS */
    uint32_t *fs_seed; /* Frame Start (0x3A) seed per frame, Slippi 2.2+ */
    uint8_t *fs_has;
    int seed_diverged; /* first frame the port's seed left the console's, logged once */
    int frame;       /* current Slippi frame, GW_RP_UNARMED before the match */
    uint32_t ucf_dashback[4]; /* Game Start per-port UCF toggles: 0 off, 1 UCF, 2 arduino */
    uint32_t ucf_shield[4];
    int resync;
    FILE *trace;
    FILE *vel; /* <trace>.vel.csv: the velocities Slippi 3.5+ post-frame records */
    char scene[256];
} rp = { .frame = GW_RP_UNARMED };

static uint32_t rp_be32(const uint8_t *p) {
    return ((uint32_t) p[0] << 24) | ((uint32_t) p[1] << 16) | ((uint32_t) p[2] << 8) | p[3];
}
static float rp_bef(const uint8_t *p) {
    uint32_t u = rp_be32(p);
    float f;
    memcpy(&f, &u, 4);
    return f;
}

/* The `raw` UBJSON array of a .slp: `raw` `[` `$` `U` `#` `l` <u32 length> <events>. */
static int rp_parse(const uint8_t *d, size_t n) {
    size_t i, pos, end;
    uint16_t sizes[256];
    int pass;
    memset(sizes, 0, sizeof sizes);
    for (i = 0; i + 12 < n; ++i) {
        if (memcmp(d + i, "raw[$U#l", 8) == 0) {
            break;
        }
    }
    if (i + 12 >= n) {
        gw_log("replay: no raw event block");
        return -1;
    }
    pos = i + 12;
    end = pos + rp_be32(d + i + 8);
    if (end > n || end == pos) {
        end = n; /* a replay still being written has length 0 */
    }
    if (d[pos] != 0x35) {
        gw_log("replay: raw block does not start with the event-size table");
        return -1;
    }
    for (i = 0; i + 3 <= (size_t) d[pos + 1] - 1; i += 3) {
        sizes[d[pos + 2 + i]] = (uint16_t) ((d[pos + 3 + i] << 8) | d[pos + 4 + i]);
    }
    /* two passes: the frame range, then the inputs */
    for (pass = 0; pass < 2; ++pass) {
        size_t c = pos + 1 + d[pos + 1];
        while (c < end) {
            uint8_t cmd = d[c];
            size_t sz = sizes[cmd];
            const uint8_t *b = d + c;
            if (sz == 0 && cmd != 0x35) {
                break;
            }
            if (c + 1 + sz > end) {
                break;
            }
            if (cmd == 0x36 && pass == 0) {
                memcpy(rp.version, b + 1, 4);
                memcpy(rp.game_info, b + 5, GW_RP_GAME_INFO);
                rp.seed = rp_be32(b + 0x13D);
                if (sz >= 0x160) { /* 1.0.0+ */
                    int p;
                    for (p = 0; p < 4; ++p) {
                        rp.ucf_dashback[p] = rp_be32(b + 0x141 + 8 * p);
                        rp.ucf_shield[p] = rp_be32(b + 0x145 + 8 * p);
                    }
                }
            } else if (cmd == 0x3A && pass == 1) {
                int frame = (int) rp_be32(b + 1);
                if (frame >= rp.first && frame <= rp.last) {
                    rp.fs_seed[frame - rp.first] = rp_be32(b + 5);
                    rp.fs_has[frame - rp.first] = 1;
                }
            } else if (cmd == 0x37) {
                int frame = (int) rp_be32(b + 1);
                int port = b[5], fol = b[6] != 0;
                if (pass == 0) {
                    if (rp.first == 0 && rp.last == 0 && rp.in == NULL) {
                        rp.first = rp.last = frame;
                    }
                    if (frame < rp.first) rp.first = frame;
                    if (frame > rp.last) rp.last = frame;
                } else if (port < 4) {
                    GwRpInput *r = &rp.in[(frame - rp.first) * GW_RP_SLOTS + port * 2 + fol];
                    r->seed = rp_be32(b + 0x7);
                    r->lx = rp_bef(b + 0x19);
                    r->ly = rp_bef(b + 0x1D);
                    r->cx = rp_bef(b + 0x21);
                    r->cy = rp_bef(b + 0x25);
                    r->trigger = rp_bef(b + 0x29);
                    r->buttons = rp_be32(b + 0x2D);
                    /* the raw bytes UCF reads, each recorded from a later Slippi version on
                       (x 1.2.0, y 3.15.0, c-stick 3.17.0); absent ones read 0, as in Slippi's
                       own playback of an older replay */
                    r->raw[0] = sz >= 0x3B ? (int8_t) b[0x3B] : 0;
                    r->raw[1] = sz >= 0x40 ? (int8_t) b[0x40] : 0;
                    r->raw[2] = sz >= 0x41 ? (int8_t) b[0x41] : 0;
                    r->raw[3] = sz >= 0x42 ? (int8_t) b[0x42] : 0;
                    r->present = 1;
                }
            }
            c += 1 + sz;
        }
        if (pass == 0) {
            size_t count = (size_t) (rp.last - rp.first + 1) * GW_RP_SLOTS;
            rp.in = (GwRpInput *) calloc(count, sizeof *rp.in);
            rp.fs_seed = (uint32_t *) calloc((size_t) (rp.last - rp.first + 1), 4);
            rp.fs_has = (uint8_t *) calloc((size_t) (rp.last - rp.first + 1), 1);
            if (rp.in == NULL || rp.fs_seed == NULL || rp.fs_has == NULL) {
                return -1;
            }
        }
    }
    return 0;
}

static void rp_load(void) {
    const char *path;
    FILE *f;
    long n;
    uint8_t *d;
    if (rp.tried) {
        return;
    }
    rp.tried = 1;
    path = getenv("MELEE_SLP");
    if (path == NULL || path[0] == '\0') {
        return;
    }
    f = fopen(path, "rb");
    if (f == NULL) {
        gw_log("replay: cannot open MELEE_SLP=\"%s\"", path);
        return;
    }
    fseek(f, 0, SEEK_END);
    n = ftell(f);
    fseek(f, 0, SEEK_SET);
    d = (uint8_t *) malloc((size_t) n);
    if (d == NULL || fread(d, 1, (size_t) n, f) != (size_t) n) {
        fclose(f);
        free(d);
        return;
    }
    fclose(f);
    if (rp_parse(d, (size_t) n) == 0) {
        const uint8_t *gi = rp.game_info;
        int p, len;
        rp.active = 1;
        rp.resync = getenv("MELEE_SLP_RESYNC") != NULL && getenv("MELEE_SLP_RESYNC")[0] == '1';
        /* the scene launcher takes it from here: a VS match straight in, these fighters, this stage */
        len = snprintf(rp.scene, sizeof rp.scene, "mode=vs;at=match;stage=ext:%u",
                       (unsigned) ((gi[0x0E] << 8) | gi[0x0F]));
        for (p = 0; p < 4; ++p) {
            const uint8_t *pl = gi + 0x60 + 0x24 * p;
            if (pl[1] == 3) { /* player type 3 = none */
                continue;
            }
            len += snprintf(rp.scene + len, sizeof rp.scene - (size_t) len, ";p%d=ck:%u/c%u/%s",
                            p + 1, pl[0], pl[3], pl[1] == 1 ? "cpu" : "hu");
        }
        gw_log("replay: %s - Slippi %u.%u.%u, frames %d..%d, seed 0x%08X, scene \"%s\"%s", path,
               rp.version[0], rp.version[1], rp.version[2], rp.first, rp.last, rp.seed, rp.scene,
               rp.resync ? ", resync on" : "");
    }
    free(d);
    path = getenv("MELEE_STATE_TRACE");
    if (rp.active && path != NULL && path[0] != '\0') {
        rp.trace = fopen(path, "w");
        if (rp.trace != NULL) {
            char vp[600];
            fprintf(rp.trace, "frame,player,follower,char,action_state,x,y,facing,percent,stocks\n");
            snprintf(vp, sizeof vp, "%s.vel.csv", path);
            rp.vel = fopen(vp, "w");
            if (rp.vel != NULL) {
                fprintf(rp.vel, "frame,player,follower,air_x,air_y,kb_x,kb_y,ground_x\n");
            }
        }
    }
}

/* For gw_sl_load (gw_runtime.c): the scene a replay implies, or NULL. */
const char *gw_replay_scene(void) {
    rp_load();
    return rp.active ? rp.scene : NULL;
}

int gw_Replay_Active(void) {
    rp_load();
    return rp.active;
}

/* fn_8016E730 (gmvs.c), at the point RestoreGameInfo.asm hooks: overwrite the match struct with
 * the replay's, keeping the port's own callback pointers (rules +0x38..+0x5B), and arm the frame
 * counter. Returns the seed for the caller to store. */
uint32_t gw_Replay_ApplyMatch(void *start_melee_data) {
    uint8_t *d = (uint8_t *) start_melee_data;
    uint8_t keep[0x5C - 0x38];
    if (!gw_Replay_Active()) {
        return 0;
    }
    memcpy(keep, d + 0x38, sizeof keep);
    memcpy(d, rp.game_info, GW_RP_GAME_INFO);
    memcpy(d + 0x38, keep, sizeof keep);
    rp.frame = GW_RP_FIRST_FRAME - 1;
    gw_log("replay: match struct restored, seed 0x%08X", rp.seed);
    return rp.seed;
}

/* Once per logic frame that runs the fighters. Returns the frame now running. */
int gw_Replay_Tick(void) {
    if (!rp.active || rp.frame == GW_RP_UNARMED) {
        return GW_RP_UNARMED;
    }
    ++rp.frame;
    if (rp.frame == rp.last + 1) {
        gw_log("replay: past the replay's last frame (%d)", rp.last);
        if (rp.trace != NULL) {
            fflush(rp.trace);
        }
    }
    return rp.frame;
}

static const GwRpInput *rp_cur(int port, int follower) {
    const GwRpInput *r;
    if (!rp.active || rp.frame < rp.first || rp.frame > rp.last || port < 0 || port > 3) {
        return NULL;
    }
    r = &rp.in[(rp.frame - rp.first) * GW_RP_SLOTS + port * 2 + (follower != 0)];
    return r->present ? r : NULL;
}

int gw_Replay_HasInput(int port, int follower) { return rp_cur(port, follower) != NULL; }
float gw_Replay_StickX(int port, int follower) { return rp_cur(port, follower)->lx; }
float gw_Replay_StickY(int port, int follower) { return rp_cur(port, follower)->ly; }
float gw_Replay_CStickX(int port, int follower) { return rp_cur(port, follower)->cx; }
float gw_Replay_CStickY(int port, int follower) { return rp_cur(port, follower)->cy; }
float gw_Replay_Trigger(int port, int follower) { return rp_cur(port, follower)->trigger; }
uint32_t gw_Replay_Buttons(int port, int follower) { return rp_cur(port, follower)->buttons; }

/* The seed the console had at the start of the frame now running (Frame Start, 0x3A), and whether
 * the replay has one. */
static int rp_frame_seed(uint32_t *out) {
    if (!rp.active || rp.frame < rp.first || rp.frame > rp.last || !rp.fs_has[rp.frame - rp.first]) {
        return 0;
    }
    *out = rp.fs_seed[rp.frame - rp.first];
    return 1;
}

/* MELEE_SLP_RESYNC: the seed to force at the start of this frame, or 0 to leave it. */
uint32_t gw_Replay_ResyncSeed(void) {
    uint32_t s;
    return rp.resync && rp_frame_seed(&s) ? s : 0;
}

/* The RNG is the earliest thing to diverge: any difference in what consumed it shows here frames
 * before a position does. Called at the start of each running frame with the port's seed. */
void gw_Replay_CheckSeed(uint32_t port_seed) {
    uint32_t want;
    if (rp.seed_diverged || !rp_frame_seed(&want)) {
        return;
    }
    if (port_seed != want) {
        rp.seed_diverged = 1;
        gw_log("replay: RNG seed diverged at frame %d: port 0x%08X, console 0x%08X", rp.frame,
               port_seed, want);
    } else if (rp.frame == rp.last) {
        gw_log("replay: RNG seed matched the console on every frame through %d", rp.last);
    }
}

int gw_Replay_Tracing(void) { return rp.trace != NULL && rp.frame != GW_RP_UNARMED; }

void gw_Replay_TraceFighter(int port, int follower, int ckind, int action, float x, float y,
                            float facing, float percent, int stocks, float air_x, float air_y,
                            float kb_x, float kb_y, float ground_x) {
    if (rp.trace == NULL || rp.frame == GW_RP_UNARMED) {
        return;
    }
    fprintf(rp.trace, "%d,%d,%d,%d,%d,%.17g,%.17g,%.17g,%.17g,%d\n", rp.frame, port, follower != 0,
            ckind, action, x, y, facing, percent, stocks);
    if (rp.vel != NULL) {
        fprintf(rp.vel, "%d,%d,%d,%.17g,%.17g,%.17g,%.17g,%.17g\n", rp.frame, port, follower != 0,
                air_x, air_y, kb_x, kb_y, ground_x);
    }
    if (rp.frame > rp.last) {
        fflush(rp.trace);
    }
}

/* The Slippi frame now running, or a large negative number before the match. */
int gw_Replay_Frame(void) { return rp.frame; }

/* UCF, as the replay's Game Start set it for this port (dashback option 1 = UCF), and whether this
 * replay's Slippi carried UCF 0.84's 1.0 cardinals. 0.84 is what Slippi 3.x installs; the 2019-era
 * 1.x recordings ran an older UCF without them. */
int gw_Replay_UcfCardinals(int port) {
    if (!rp.active || port < 0 || port > 3 || rp.ucf_dashback[port] != 1) {
        return 0;
    }
    return rp.version[0] >= 3;
}

/* This frame's raw stick byte (0 x, 1 y, 2 c-x, 3 c-y) for the port. */
int gw_Replay_RawStick(int port, int which) {
    const GwRpInput *r = rp_cur(port, 0);
    return r != NULL && which >= 0 && which < 4 ? r->raw[which] : 0;
}

/* Which UCF the recording console ran for this port: 0 off, 73, or 84 (Slippi's per-port Game
 * Start dashback toggle, 1 = UCF). Slippi's console codes shipped UCF 0.73 "Check for Toggle" from
 * Jan 2019 until 0.74 in Sep 2019 (slippi-ssbm-asm acc6f71, b89e160), 0.8 from Mar 2021 (bb86519)
 * and 0.84 with 3.x - so 1.x replays are 0.73. 2.x (0.74/0.8) is approximated as 0.73. */
int gw_Replay_UcfVersion(int port) {
    if (!rp.active || port < 0 || port > 3 || rp.ucf_dashback[port] != 1) {
        return 0;
    }
    if (rp.version[0] >= 3) {
        return 84;
    }
    {   /* MELEE_SLP_NO_UCF073=1: leave 0.73 out, for before/after measurements */
        static int off = -1;
        if (off < 0) {
            const char *v = getenv("MELEE_SLP_NO_UCF073");
            off = v != NULL && v[0] == '1';
        }
        return off ? 0 : 73;
    }
}

/* The port's raw stick byte `back` frames before the one running (0 = this frame), as the pad
 * queue held it - UCF's dashback compares this frame's raw X with the one two frames earlier. */
int gw_Replay_RawStickBack(int port, int which, int back) {
    const GwRpInput *r;
    int f = rp.frame - back;
    if (!rp.active || f < rp.first || f > rp.last || port < 0 || port > 3 || which < 0 ||
        which > 3) {
        return 0;
    }
    r = &rp.in[(f - rp.first) * GW_RP_SLOTS + port * 2];
    return r->present ? r->raw[which] : 0;
}
